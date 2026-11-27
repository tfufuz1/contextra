//! # Scanner-Unit-Tests (Ring 1)
//!
//! Validiert Scan-Semantiken und Fehlerbehandlung.

// FILE-CONTEXT
// ZWECK: Tests für LSM Scanner & Purger (Vertrag V1, PURGE-PIPELINE W1-P01).
// INVARIANTEN: Negativtests zuerst; Sabotage und Restdaten werden zuverlässig erkannt.
// HOTSPOTS: [1-180]

#![forbid(unsafe_code)]

use super::*;
use crate::lsm::{LsmConfig, LsmStorage};
use crate::tenant_codec::TenantKeyCodec;
use contextra_ports::clock::SystemClock;
use contextra_ports::scanner::{ScanTarget, ScanTechnique};
use contextra_ports::StorageEngine;
use contextra_types::{RequestContext, TenantId, TxId};
use tempfile::tempdir;

async fn create_test_storage() -> (Arc<LsmStorage>, tempfile::TempDir) {
    let dir = tempdir().expect("tempdir creation");
    let config = LsmConfig {
        path: dir.path().to_path_buf(),
        ..Default::default()
    };
    let storage = Arc::new(LsmStorage::open(config).await.expect("LsmStorage open"));
    (storage, dir)
}

#[tokio::test]
async fn test_logical_deletion_without_purge_shows_remaining() {
    let (storage, _dir) = create_test_storage().await;
    let clock = SystemClock::new();
    let tenant_id = TenantId::try_new(1).unwrap();
    let codec = TenantKeyCodec::new(tenant_id);

    let key1 = codec.encode_chunk_key(
        &contextra_types::CollectionId(1),
        contextra_types::DocId(101),
    );
    let key2 = codec.encode_chunk_key(
        &contextra_types::CollectionId(1),
        contextra_types::DocId(102),
    );

    let tx = TxId(100);
    storage.put(tx, &key1, b"val1").await.unwrap();
    storage.put(tx, &key2, b"val2").await.unwrap();
    storage.commit(tx).await.unwrap();

    // Flush to SSTable
    storage.flush().await.unwrap();

    // Logical delete
    let tx_del = TxId(101);
    storage
        .delete_prefix(tx_del, codec.scan_prefix())
        .await
        .unwrap();
    storage.commit(tx_del).await.unwrap();

    // Scanners run without physical purge
    let ctx = RequestContext::new(tenant_id, TxId(200));
    let target = ScanTarget::new(format!("t:{}:", tenant_id.inner()), [0xAA; 32]);

    let scanners = lsm_scanners(&storage);
    let sstable_scanner = &scanners[1]; // SsTableAllLevels
    let wal_scanner = &scanners[2]; // WalAllSegments

    let witness_sst = sstable_scanner
        .scan(&ctx, target.clone(), ScanTechnique::Structural, &clock)
        .unwrap();

    let witness_wal = wal_scanner
        .scan(&ctx, target, ScanTechnique::Structural, &clock)
        .unwrap();

    assert!(
        witness_sst.remaining() > 0,
        "SSTable scanner must report remaining > 0 for logical tombstones"
    );
    assert!(
        witness_wal.remaining() > 0,
        "WAL scanner must report remaining > 0 for WAL delete records"
    );
}

struct TestCancel;
impl contextra_ports::erasure::Cancel for TestCancel {
    fn is_cancelled(&self) -> bool {
        false
    }
    fn cancelled(&self) -> contextra_ports::BoxFuture<'_, ()> {
        Box::pin(std::future::pending())
    }
}

struct TestBudget;
impl contextra_ports::erasure::IoBudget for TestBudget {
    fn acquire(
        &self,
        _class: contextra_ports::erasure::IoClass,
        _bytes: u64,
    ) -> contextra_ports::BoxFuture<'_, Result<(), contextra_types::ErasureError>> {
        Box::pin(async { Ok(()) })
    }
    fn pressure(&self) -> contextra_ports::erasure::PressureHint {
        contextra_ports::erasure::PressureHint::Normal
    }
}

struct TestProgress<'a> {
    kernel: &'a dyn contextra_ports::ErasureKernel,
    id: contextra_types::ErasureId,
}
impl contextra_ports::erasure::ProgressSink for TestProgress<'_> {
    fn record<'a>(
        &'a self,
        layer: &'a contextra_ports::scanner::WitnessLayer,
        p: &'a contextra_types::LayerProgress,
    ) -> contextra_ports::BoxFuture<'a, Result<(), contextra_types::ErasureError>> {
        self.kernel.record_layer(self.id, layer, p)
    }
}

async fn run_kernel_purge_for_tenant(storage: &LsmStorage, tenant_id: TenantId, id_byte: u8) {
    use contextra_ports::erasure::{
        EraseRequest, EraseTarget, KeyPattern, KeyScope, PurgeCx, ResolvedScope, Verdict,
    };
    use contextra_types::{ErasureId, ErasurePolicy, ErasureReason, TargetKind};

    let kernel = storage.erasure_kernel();
    let id = ErasureId([id_byte; 16]);
    let prefix = format!("t:{}:", tenant_id.inner()).into_bytes();
    let keys = KeyScope::new(vec![(KeyPattern::Prefix(prefix), Verdict::In)]).unwrap();
    let req = EraseRequest {
        id,
        target: EraseTarget::Tenant { tenant: tenant_id },
        reason: ErasureReason::UserRequest,
        policy: ErasurePolicy::default(),
    };
    let plan = ResolvedScope {
        id,
        target_kind: TargetKind::Tenant,
        keys,
        docs: vec![],
        plan_hash: [id_byte; 32],
        unprovenanced: 0,
    };

    kernel.accept(&req, &plan).await.unwrap();
    kernel.publish_gate(id).await.unwrap();
    let reader = kernel.privileged_reader(id).unwrap();
    let clock = SystemClock::new();
    let cancel = TestCancel;
    let budget = TestBudget;
    let progress = TestProgress {
        kernel: kernel.as_ref(),
        id,
    };
    let mut cursor = None;
    loop {
        let cx = PurgeCx {
            erasure_id: id,
            clock: &clock,
            cancel: &cancel,
            budget: &budget,
            cursor: cursor.take(),
            reader: reader.as_ref(),
            progress: &progress,
            max_units: 64,
        };
        let out = kernel.purge_store(id, &cx).await.unwrap();
        if out.progress.cursor.is_none() {
            break;
        }
        cursor = out.progress.cursor;
    }
}

/// TST-STORE-E2E-1 (1/3): Physischer Kernel-Purge entfernt alle Restdaten in Memtable, SSTable und WAL.
#[tokio::test]
async fn test_purge_clears_remaining_residuals() {
    let (storage, _dir) = create_test_storage().await;
    let clock = SystemClock::new();
    let tenant_id = TenantId::try_new(1).unwrap();
    let codec = TenantKeyCodec::new(tenant_id);

    let key = codec.encode_chunk_key(&contextra_types::CollectionId(1), contextra_types::DocId(1));
    let tx = TxId(10);
    storage.put(tx, &key, b"value_to_purge").await.unwrap();
    storage.commit(tx).await.unwrap();

    storage.flush().await.unwrap();

    let tx_del = TxId(11);
    storage
        .delete_prefix(tx_del, codec.scan_prefix())
        .await
        .unwrap();
    storage.commit(tx_del).await.unwrap();

    run_kernel_purge_for_tenant(&storage, tenant_id, 0xBB).await;

    let ctx = RequestContext::new(tenant_id, TxId(50));
    let target = ScanTarget::new(format!("t:{}:", tenant_id.inner()), [0xBB; 32]);
    let scanners = lsm_scanners(&storage);

    for scanner in &scanners {
        let witness = scanner
            .scan(&ctx, target.clone(), ScanTechnique::Structural, &clock)
            .expect("scan should succeed after purge");

        assert_eq!(
            witness.remaining(),
            0,
            "Remaining must be 0 for layer {}",
            witness.layer()
        );
    }
}

#[tokio::test]
async fn test_sabotage_unremoved_orphan_file_detected_by_scanner() {
    let (storage, _dir) = create_test_storage().await;
    let clock = SystemClock::new();
    let tenant_id = TenantId::try_new(2).unwrap();
    let codec = TenantKeyCodec::new(tenant_id);

    // Create an unremoved orphan file in data path
    let orphan_path = storage.data_path().join("orphan_sabotage.sst");
    let key = codec.encode_chunk_key(
        &contextra_types::CollectionId(1),
        contextra_types::DocId(999),
    );

    // Build transient SSTable file
    let mut builder = crate::sstable::SstableBuilder::create(&orphan_path)
        .await
        .unwrap();
    builder.add(&key, b"sabotage_value", 1, 1).await.unwrap();
    builder.finish().await.unwrap();

    let ctx = RequestContext::new(tenant_id, TxId(100));
    let target = ScanTarget::new(format!("t:{}:", tenant_id.inner()), [0xCC; 32]);

    let scanners = lsm_scanners(&storage);
    let sstable_scanner = &scanners[1];

    let witness = sstable_scanner
        .scan(&ctx, target, ScanTechnique::Structural, &clock)
        .unwrap();

    assert!(
        witness.remaining() > 0,
        "SSTable scanner must detect residual in unremoved orphan sabotage file"
    );
}

/// TST-STORE-E2E-1 (2/3): Offener Snapshot verhindert Gate-Abschluss (`complete` liefert `InvalidTransition`).
#[tokio::test]
async fn test_open_snapshot_blocks_purge() {
    let (storage, _dir) = create_test_storage().await;
    let tenant_id = TenantId::try_new(3).unwrap();
    let codec = TenantKeyCodec::new(tenant_id);
    let key = codec.encode_chunk_key(&contextra_types::CollectionId(1), contextra_types::DocId(1));
    let tx = TxId(10);
    storage.put(tx, &key, b"snapshot_pinned").await.unwrap();
    storage.commit(tx).await.unwrap();
    storage.flush().await.unwrap();

    run_kernel_purge_for_tenant(&storage, tenant_id, 0xDD).await;

    let kernel = storage.erasure_kernel();
    let id = contextra_types::ErasureId([0xDD; 16]);
    let clock = SystemClock::new();
    let witnesses: Vec<_> = kernel
        .store_scanners()
        .into_iter()
        .map(|s| {
            let ctx = RequestContext::new(tenant_id, TxId(1));
            let target = ScanTarget::new(format!("t:{}:", tenant_id.inner()), [0xDD; 32]);
            s.scan(&ctx, target, ScanTechnique::Structural, &clock)
                .unwrap()
        })
        .collect();
    kernel.seal(id, &witnesses).await.unwrap();

    // Offener Snapshot <= gated_seq blockiert complete() ohne OverlappingSnapshotPinned-Ausnahme (§6.5)
    let guard = storage.snapshot_registry.register(1);
    let res = kernel.complete(id).await;
    assert!(
        matches!(res, Err(contextra_types::ErasureError::InvalidTransition)),
        "complete must remain blocked while open snapshot <= gated_seq is active, got: {res:?}"
    );
    drop(guard);
    kernel.complete(id).await.unwrap();
}

/// TST-STORE-E2E-1 (3/3): WAL-only Purge entfernt unbeflushte WAL-Einträge über WAL-Schnitt.
#[tokio::test]
async fn test_multi_level_sstable_and_wal_only_purge() {
    let (storage, _dir) = create_test_storage().await;
    let clock = SystemClock::new();
    let tenant_id = TenantId::try_new(4).unwrap();
    let codec = TenantKeyCodec::new(tenant_id);

    // 1. Put data only in WAL (unflushed)
    let key_wal =
        codec.encode_chunk_key(&contextra_types::CollectionId(1), contextra_types::DocId(1));
    let tx1 = TxId(1);
    storage.put(tx1, &key_wal, b"wal_data").await.unwrap();
    storage.commit(tx1).await.unwrap();

    let ctx = RequestContext::new(tenant_id, TxId(10));
    let target = ScanTarget::new(format!("t:{}:", tenant_id.inner()), [0xEE; 32]);

    let scanners = lsm_scanners(&storage);
    let wal_scanner = &scanners[2];

    let witness_wal_before = wal_scanner
        .scan(&ctx, target.clone(), ScanTechnique::Structural, &clock)
        .unwrap();

    assert!(
        witness_wal_before.remaining() > 0,
        "WAL scanner must detect unflushed WAL data"
    );

    // Purge WAL data via ErasureKernel
    run_kernel_purge_for_tenant(&storage, tenant_id, 0xEE).await;

    let witness_wal_after = wal_scanner
        .scan(&ctx, target, ScanTechnique::Structural, &clock)
        .unwrap();

    assert_eq!(
        witness_wal_after.remaining(),
        0,
        "WAL scanner must report remaining == 0 after purge"
    );
}

#[test]
fn test_scanner_without_runtime_returns_err() {
    let dir = tempfile::tempdir().unwrap();
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();

    let storage = rt.block_on(async {
        let config = LsmConfig {
            path: dir.path().to_path_buf(),
            ..Default::default()
        };
        Arc::new(LsmStorage::open(config).await.unwrap())
    });

    // Create scanners with empty runtime handle
    let empty_runtime = ScannerRuntime::empty();
    let scanners = lsm_scanners_with_runtime(&storage, empty_runtime);

    let ctx = RequestContext::new(TenantId::try_new(99).unwrap(), TxId(1));
    let target = ScanTarget::new("t:99:".to_string(), [0x99; 32]);
    let clock = SystemClock::new();

    // All scanners without runtime return Err
    for scanner in &scanners {
        let scan_res = scanner.scan(&ctx, target.clone(), ScanTechnique::Structural, &clock);
        assert!(
            scan_res.is_err(),
            "Scanner {:?} without runtime handle must return Err",
            scanner.layer()
        );
    }
}

#[test]
fn test_current_thread_and_multithread_runtime_parity() {
    let dir = tempfile::tempdir().unwrap();

    // 1. Setup data using a multithread runtime
    let mt_rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()
        .unwrap();

    let storage = mt_rt.block_on(async {
        let config = LsmConfig {
            path: dir.path().to_path_buf(),
            ..Default::default()
        };
        let s = Arc::new(LsmStorage::open(config).await.unwrap());
        let tenant_id = TenantId::try_new(42).unwrap();
        let codec = TenantKeyCodec::new(tenant_id);
        let key = codec.encode_chunk_key(&contextra_types::CollectionId(1), contextra_types::DocId(7));
        s.put(TxId(1), &key, b"parity_test_value").await.unwrap();
        s.commit(TxId(1)).await.unwrap();
        s.flush().await.unwrap();
        s
    });

    let ctx = RequestContext::new(TenantId::try_new(42).unwrap(), TxId(10));
    let target = ScanTarget::new("t:42:".to_string(), [0x42; 32]);
    let clock = SystemClock::new();

    // Run scan with multi-thread runtime
    let mt_scanners = lsm_scanners_with_handle(&storage, mt_rt.handle().clone());
    let mt_witness = mt_scanners[1] // SsTableAllLevels
        .scan(&ctx, target.clone(), ScanTechnique::Structural, &clock)
        .expect("Multi-thread scan should succeed");

    // Run scan with current-thread runtime
    let ct_rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();

    let ct_scanners = lsm_scanners_with_handle(&storage, ct_rt.handle().clone());
    let ct_witness = ct_scanners[1]
        .scan(&ctx, target, ScanTechnique::Structural, &clock)
        .expect("Current-thread scan should succeed");

    assert_eq!(
        mt_witness.remaining(),
        ct_witness.remaining(),
        "Remaining count must match between MultiThread and CurrentThread runtimes"
    );
    assert_eq!(
        mt_witness.scanned(),
        ct_witness.scanned(),
        "Scanned count must match between MultiThread and CurrentThread runtimes"
    );
    assert_eq!(
        mt_witness.digest(),
        ct_witness.digest(),
        "Witness digest must match between MultiThread and CurrentThread runtimes"
    );
}

#[tokio::test]
async fn test_worker_panic_mapped_to_err() {
    let handle = tokio::runtime::Handle::current();
    let runtime = ScannerRuntime::with_handle(handle);

    let res = runtime.block_on(async {
        if true {
            panic!("Simulierter Worker Panic in Port-Operation");
        }
        Ok::<(), contextra_types::ContextraError>(())
    });

    assert!(res.is_err(), "Panics in worker must be caught and mapped to Err");
    let err_str = res.unwrap_err().to_string();
    assert!(
        err_str.contains("Simulierter Worker Panic"),
        "Error message must contain panic payload message, got: {err_str}"
    );
}
