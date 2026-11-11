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

    let ctx = RequestContext::new(tenant_id, TxId(50));
    let target = ScanTarget::new(format!("t:{}:", tenant_id.inner()), [0xBB; 32]);

    let purgers = lsm_purgers(&storage);
    purgers[0]
        .purge(&ctx, &target)
        .expect("purge should succeed");

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
        assert!(
            witness.scanned() > 0,
            "Scanned must be > 0 for layer {}",
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

#[tokio::test]
async fn test_open_snapshot_blocks_purge() {
    let (storage, _dir) = create_test_storage().await;
    let tenant_id = TenantId::try_new(3).unwrap();

    let _guard = storage.snapshot_registry.register(100);

    let ctx = RequestContext::new(tenant_id, TxId(1));
    let target = ScanTarget::new(format!("t:{}:", tenant_id.inner()), [0xDD; 32]);

    let purgers = lsm_purgers(&storage);
    let res = purgers[0].purge(&ctx, &target);

    assert!(
        res.is_err(),
        "Purge must fail closed when open snapshot is active"
    );
    let err_msg = res.unwrap_err().to_string();
    assert!(
        err_msg.contains("open snapshot"),
        "Error message must mention open snapshot, got: {}",
        err_msg
    );
}

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

    // Purge WAL data
    let purgers = lsm_purgers(&storage);
    purgers[0].purge(&ctx, &target).unwrap();

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
fn test_scanner_and_purger_without_runtime_returns_err() {
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

    // Create scanners/purgers with empty runtime handle
    let empty_runtime = ScannerRuntime::empty();
    let scanners = lsm_scanners_with_runtime(&storage, empty_runtime.clone());
    let purgers = lsm_purgers_with_runtime(&storage, empty_runtime);

    let ctx = RequestContext::new(TenantId::try_new(99).unwrap(), TxId(1));
    let target = ScanTarget::new("t:99:".to_string(), [0x99; 32]);
    let clock = SystemClock::new();

    // 1. Purger without runtime returns Err
    let purge_res = purgers[0].purge(&ctx, &target);
    assert!(
        purge_res.is_err(),
        "Purger without runtime handle must return Err"
    );

    // 2. All scanners without runtime return Err
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
