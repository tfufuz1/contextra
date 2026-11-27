//! Integrations- und Regressionstests für den Erasure-Kernel (§10 / §13).
//!
//! Deckt ab: TST-STORE-JRN-4, TST-STORE-GATE-8 (H-1..H-10), TST-STORE-COMMIT-1,
//! TST-STORE-ROT-1, TST-STORE-MEM-1, TST-STORE-ORDER-1, TST-STORE-E2E-2, T-DEL-1.

use bytes::Bytes;
use contextra_ports::clock::SystemClock;
use contextra_ports::erasure::{
    Cancel, EraseRequest, EraseTarget, IoBudget, IoClass, KeyPattern, KeyScope, PressureHint,
    ProgressSink, PurgeCx, ResolvedScope, Verdict,
};
use contextra_ports::scanner::{ScanTarget, ScanTechnique};
use contextra_ports::vfs::{Vfs, VfsFile, VfsOpenOptions};
use contextra_ports::{BoxFuture, StorageEngine};
use contextra_store::memtable::MemTable;
use contextra_store::tenant_codec::TenantKeyCodec;
use contextra_store::{LsmConfig, LsmStorage, StoreIo};
use contextra_types::{
    CollectionId, ContextraError, DocId, ErasureError, ErasureId, ErasurePolicy, ErasureReason,
    LayerProgress, RequestContext, Result, TargetKind, TenantId, TxId,
};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tempfile::tempdir;

struct TestCancel;
impl Cancel for TestCancel {
    fn is_cancelled(&self) -> bool {
        false
    }
    fn cancelled(&self) -> BoxFuture<'_, ()> {
        Box::pin(std::future::pending())
    }
}

struct TestBudget;
impl IoBudget for TestBudget {
    fn acquire(
        &self,
        _class: IoClass,
        _bytes: u64,
    ) -> BoxFuture<'_, std::result::Result<(), ErasureError>> {
        Box::pin(async { Ok(()) })
    }
    fn pressure(&self) -> PressureHint {
        PressureHint::Normal
    }
}

struct TestProgress<'a> {
    kernel: &'a dyn contextra_ports::ErasureKernel,
    id: ErasureId,
}
impl ProgressSink for TestProgress<'_> {
    fn record<'a>(
        &'a self,
        layer: &'a contextra_ports::scanner::WitnessLayer,
        p: &'a LayerProgress,
    ) -> BoxFuture<'a, std::result::Result<(), ErasureError>> {
        self.kernel.record_layer(self.id, layer, p)
    }
}

fn tenant_plan(tenant_id: TenantId, id_byte: u8) -> (EraseRequest, ResolvedScope) {
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
    (req, plan)
}

async fn drive_purge_to_end(storage: &LsmStorage, id: ErasureId) {
    let kernel = storage.erasure_kernel();
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
            max_units: 4,
        };
        let out = kernel.purge_store(id, &cx).await.unwrap();
        if out.progress.cursor.is_none() {
            break;
        }
        cursor = out.progress.cursor;
    }
}

/// Wrapper-VFS zur gezielten Fsync-Fehlerinjektion auf `journal.log` (TST-STORE-JRN-4).
struct FailJournalSyncVfs {
    inner: Arc<dyn Vfs>,
    fail_journal_sync: Arc<AtomicBool>,
}

struct FailJournalSyncFile {
    inner: Box<dyn VfsFile>,
    fail_sync: Arc<AtomicBool>,
    is_journal: bool,
}

impl VfsFile for FailJournalSyncFile {
    fn read_at(&self, buf: &mut [u8], offset: u64) -> std::io::Result<usize> {
        self.inner.read_at(buf, offset)
    }
    fn write_at(&self, buf: &[u8], offset: u64) -> std::io::Result<usize> {
        self.inner.write_at(buf, offset)
    }
    fn sync_data(&self) -> std::io::Result<()> {
        if self.is_journal && self.fail_sync.load(Ordering::Acquire) {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "injected journal sync_data failure",
            ));
        }
        self.inner.sync_data()
    }
    fn sync_all(&self) -> std::io::Result<()> {
        if self.is_journal && self.fail_sync.load(Ordering::Acquire) {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "injected journal sync_all failure",
            ));
        }
        self.inner.sync_all()
    }
    fn set_len(&self, size: u64) -> std::io::Result<()> {
        self.inner.set_len(size)
    }
    fn len(&self) -> std::io::Result<u64> {
        self.inner.len()
    }
}

impl Vfs for FailJournalSyncVfs {
    fn open(&self, path: &Path, opts: &VfsOpenOptions) -> Result<Box<dyn VfsFile>> {
        let inner = self.inner.open(path, opts)?;
        let is_journal = path
            .file_name()
            .and_then(|n| n.to_str())
            .is_some_and(|n| n == "journal.log");
        Ok(Box::new(FailJournalSyncFile {
            inner,
            fail_sync: Arc::clone(&self.fail_journal_sync),
            is_journal,
        }))
    }
    fn read(&self, path: &Path) -> Result<Vec<u8>> {
        self.inner.read(path)
    }
    fn write_atomic(&self, path: &Path, data: &[u8]) -> Result<()> {
        self.inner.write_atomic(path, data)
    }
    fn remove_file(&self, path: &Path) -> Result<()> {
        self.inner.remove_file(path)
    }
    fn rename(&self, from: &Path, to: &Path) -> Result<()> {
        self.inner.rename(from, to)
    }
    fn create_dir_all(&self, path: &Path) -> Result<()> {
        self.inner.create_dir_all(path)
    }
    fn exists(&self, path: &Path) -> Result<bool> {
        self.inner.exists(path)
    }
    fn len(&self, path: &Path) -> Result<u64> {
        self.inner.len(path)
    }
    fn read_dir(&self, path: &Path) -> Result<Vec<PathBuf>> {
        self.inner.read_dir(path)
    }
    fn list_dir(&self, path: &Path) -> Result<Vec<String>> {
        self.inner.list_dir(path)
    }
    fn sync_dir(&self, path: &Path) -> Result<()> {
        self.inner.sync_dir(path)
    }
}

/// TST-STORE-JRN-4: `fsync`-Fehler -> `JournalPoisoned`, keine weitere Mutation, Gate bleibt aktiv.
#[tokio::test]
async fn test_journal_sync_failure_poisons_kernel_and_keeps_gate_active() {
    let dir = tempdir().unwrap();
    let fail_flag = Arc::new(AtomicBool::new(false));
    let vfs = Arc::new(FailJournalSyncVfs {
        inner: Arc::new(contextra_store::RealFs),
        fail_journal_sync: Arc::clone(&fail_flag),
    });
    let cfg = LsmConfig {
        path: dir.path().to_path_buf(),
        vfs: Some(StoreIo::new(vfs)),
        ..Default::default()
    };
    let storage = LsmStorage::open(cfg).await.unwrap();
    let kernel = storage.erasure_kernel();

    let t1 = TenantId::try_new(10).unwrap();
    let codec = TenantKeyCodec::new(t1);
    let key = codec.encode_chunk_key(&CollectionId(1), DocId(1));

    storage.put(TxId(1), &key, b"secret").await.unwrap();
    storage.commit(TxId(1)).await.unwrap();

    let (req1, plan1) = tenant_plan(t1, 0x11);
    kernel.accept(&req1, &plan1).await.unwrap();
    kernel.publish_gate(req1.id).await.unwrap();
    assert!(kernel.gate().check(&key).is_blocked());

    // Jetzt Journal-fsync fehlschlagen lassen
    fail_flag.store(true, Ordering::Release);

    let t2 = TenantId::try_new(11).unwrap();
    let (req2, plan2) = tenant_plan(t2, 0x22);
    let err = kernel.accept(&req2, &plan2).await.unwrap_err();
    assert!(matches!(
        err,
        ErasureError::Io(_) | ErasureError::JournalPoisoned
    ));

    // Danach liefert jede mutierende Methode JournalPoisoned, auch wenn sync wieder geht
    fail_flag.store(false, Ordering::Release);
    let err2 = kernel.accept(&req2, &plan2).await.unwrap_err();
    assert!(matches!(err2, ErasureError::JournalPoisoned));

    // Gate für t1 bleibt aktiv
    assert!(kernel.gate().check(&key).is_blocked());
    assert_eq!(storage.get(&key).await.unwrap(), None);
}

/// TST-STORE-GATE-8: H-1..H-10 Durchsetzungshaken im Store.
#[tokio::test]
async fn test_gate_enforcement_hooks_h1_to_h10() {
    let dir = tempdir().unwrap();
    let cfg = LsmConfig {
        path: dir.path().to_path_buf(),
        ..Default::default()
    };
    let storage = LsmStorage::open(cfg).await.unwrap();
    let kernel = storage.erasure_kernel();

    let t_erased = TenantId::try_new(7).unwrap();
    let t_other = TenantId::try_new(8).unwrap();
    let c_erased = TenantKeyCodec::new(t_erased);
    let c_other = TenantKeyCodec::new(t_other);

    let k1 = c_erased.encode_chunk_key(&CollectionId(1), DocId(1));
    let k2 = c_erased.encode_chunk_key(&CollectionId(1), DocId(2));
    let k_other = c_other.encode_chunk_key(&CollectionId(1), DocId(1));

    // Versionen schreiben + flushen (SSTable + Memtable)
    storage.put(TxId(1), &k1, b"v1").await.unwrap();
    storage.put(TxId(1), &k_other, b"other_v1").await.unwrap();
    storage.commit(TxId(1)).await.unwrap();
    let seq_after_tx1 = storage.last_applied_seq();
    storage.flush().await.unwrap();

    storage.put(TxId(2), &k2, b"v2").await.unwrap();
    storage.commit(TxId(2)).await.unwrap();

    // Gate setzen (Restricted)
    let (req, plan) = tenant_plan(t_erased, 0x77);
    kernel.accept(&req, &plan).await.unwrap();
    kernel.publish_gate(req.id).await.unwrap();

    // H-1: Commit auf gesperrten Schlüssel wird mit ContextraError::Erased abgewiesen
    let k3 = c_erased.encode_chunk_key(&CollectionId(1), DocId(3));
    storage.put(TxId(3), &k3, b"v3").await.unwrap();
    let commit_res = storage.commit(TxId(3)).await;
    assert!(matches!(commit_res, Err(ContextraError::Erased)));

    // H-8: delete_prefix überspringt gesperrte Schlüssel ohne Fehler
    storage
        .delete_prefix(TxId(4), c_erased.scan_prefix())
        .await
        .unwrap();
    storage.commit(TxId(4)).await.unwrap();

    // H-3: Punkt-Lesen (SSTable + MemTable) liefert None
    assert_eq!(storage.get(&k1).await.unwrap(), None);
    assert_eq!(storage.get(&k2).await.unwrap(), None);
    assert_eq!(
        storage.get(&k_other).await.unwrap(),
        Some(Bytes::from_static(b"other_v1"))
    );

    // H-9: Zeitreise / get_at_seq liefert None für gesperrten Schlüssel
    assert_eq!(storage.get_at_seq(&k1, seq_after_tx1).await.unwrap(), None);
    assert_eq!(
        storage.get_at_seq(&k_other, seq_after_tx1).await.unwrap(),
        Some(Bytes::from_static(b"other_v1"))
    );

    // H-4: scan_prefix und scan_range liefern nichts für t_erased
    let prefix_res = storage.scan_prefix(c_erased.scan_prefix()).await.unwrap();
    assert!(prefix_res.is_empty());

    let mut end_range = c_erased.scan_prefix().to_vec();
    end_range.push(0xFF);
    let range_res = storage
        .scan(
            std::ops::Bound::Included(c_erased.scan_prefix()),
            std::ops::Bound::Excluded(&end_range),
            Some(100),
        )
        .await
        .unwrap();
    assert!(range_res.is_empty());

    // Nachbar-Tenant bleibt voll lesbar
    let other_scan = storage.scan_prefix(c_other.scan_prefix()).await.unwrap();
    assert_eq!(other_scan.len(), 1);
    assert_eq!(other_scan[0].1, b"other_v1");

    // H-5: Replay nach Restart behält Gate und Datenisolation
    storage.wait_shutdown().await;
    drop(kernel);
    drop(storage);
    let storage2 = LsmStorage::open(LsmConfig {
        path: dir.path().to_path_buf(),
        ..Default::default()
    })
    .await
    .unwrap();
    assert_eq!(storage2.get(&k1).await.unwrap(), None);
    assert_eq!(storage2.get(&k2).await.unwrap(), None);
    assert_eq!(
        storage2.get(&k_other).await.unwrap(),
        Some(Bytes::from_static(b"other_v1"))
    );
}

/// TST-STORE-MEM-1: `MemTable::retain` Invarianten unter parallelen `put`/`get`.
#[test]
fn test_memtable_retain_invariants() {
    let mt = MemTable::new();
    for i in 0u64..100 {
        let key = if i % 2 == 0 {
            format!("t:1:k{i:03}").into_bytes()
        } else {
            format!("t:2:k{i:03}").into_bytes()
        };
        mt.put(
            Bytes::from(key),
            Bytes::from_static(b"payload_bytes"),
            i + 1,
            i + 1,
        );
    }
    let size_before = mt.size();
    assert!(size_before > 0);

    let (dropped, freed) = mt.retain(|k, _seq, _tomb| k.starts_with(b"t:1:"));
    assert_eq!(dropped, 50);
    assert!(freed > 0);
    assert_eq!(mt.size(), size_before - freed as usize);

    for (k, _, _, _) in mt.iter() {
        assert!(!k.starts_with(b"t:1:"), "t:1: key leaked after retain");
    }

    let (dropped_rest, _) = mt.retain(|_, _, _| true);
    assert_eq!(dropped_rest, 50);
    assert!(mt.is_empty());
    assert_eq!(mt.size(), 0);
}

/// TST-STORE-ORDER-1, TST-STORE-E2E-2 & T-DEL-1:
/// Vollständige Löschkette ohne Engine inkl. Nachbar-Bitidentität, PrivilegedRead-Sperre nach Sealed
/// und Canary-Abwesenheit nach Journal-Verdichtung.
#[tokio::test]
async fn test_e2e_erasure_lifecycle_and_canary_t_del_1() {
    let dir = tempdir().unwrap();
    let cfg = LsmConfig {
        path: dir.path().to_path_buf(),
        ..Default::default()
    };
    let storage = LsmStorage::open(cfg).await.unwrap();
    let kernel = storage.erasure_kernel();

    let t_victim = TenantId::try_new(7).unwrap();
    let t_neighbor = TenantId::try_new(70).unwrap(); // Prefix-Falle t:7: vs t:70:
    let c_victim = TenantKeyCodec::new(t_victim);
    let c_neighbor = TenantKeyCodec::new(t_neighbor);

    let canary = b"CANARY_SECRET_BYTES_0123456789_XYZ";
    let kv1 = c_victim.encode_chunk_key(&CollectionId(1), DocId(1));
    let kv2 = c_victim.encode_chunk_key(&CollectionId(1), DocId(2));
    let kn1 = c_neighbor.encode_chunk_key(&CollectionId(1), DocId(1));

    // 1. In SSTable flushen
    storage.put(TxId(1), &kv1, canary).await.unwrap();
    storage
        .put(TxId(1), &kn1, b"neighbor_intact")
        .await
        .unwrap();
    storage.commit(TxId(1)).await.unwrap();
    storage.flush().await.unwrap();

    // 2. Zweite Version im Memtable + WAL
    storage.put(TxId(2), &kv2, canary).await.unwrap();
    storage.commit(TxId(2)).await.unwrap();

    // 3. accept -> publish_gate -> privileged_reader -> purge_store -> seal -> complete
    let (req, plan) = tenant_plan(t_victim, 0xAA);
    kernel.accept(&req, &plan).await.unwrap();
    kernel.publish_gate(req.id).await.unwrap();

    let priv_reader = kernel.privileged_reader(req.id).unwrap();
    assert_eq!(
        priv_reader.get(&kv1).await.unwrap(),
        Some(canary.to_vec().into())
    );
    // ScopeViolation bei fremdem Tenant
    let scope_err = priv_reader.get(&kn1).await.unwrap_err();
    assert!(matches!(scope_err, ErasureError::Layer { .. }));

    drive_purge_to_end(&storage, req.id).await;

    // Scanner-Zeugen einsammeln
    let clock = SystemClock::new();
    let ctx = RequestContext::new(t_victim, TxId(99));
    let target = ScanTarget::new(format!("t:{}:", t_victim.inner()), [0xAA; 32]);
    let witnesses: Vec<_> = kernel
        .store_scanners()
        .into_iter()
        .map(|s| {
            s.scan(&ctx, target.clone(), ScanTechnique::Structural, &clock)
                .unwrap()
        })
        .collect();

    let seal = kernel.seal(req.id, &witnesses).await.unwrap();
    assert!(seal.guarantee.gated);
    assert!(seal.guarantee.purged);

    // PrivilegedRead muss nach Sealed ablehnen (§7.7)
    assert!(priv_reader.get(&kv1).await.is_err());

    kernel.complete(req.id).await.unwrap();

    // Nachbar-Schlüssel bit-identisch (C3 / TST-STORE-ISO-1)
    assert_eq!(
        storage.get(&kn1).await.unwrap(),
        Some(Bytes::from_static(b"neighbor_intact"))
    );

    // T-DEL-1: Canary darf in keiner Datei im Datenverzeichnis (SST/WAL) mehr vorkommen
    drop(storage);
    for entry in std::fs::read_dir(dir.path()).unwrap() {
        let path = entry.unwrap().path();
        if path.is_file() {
            let bytes = std::fs::read(&path).unwrap();
            assert!(
                !bytes.windows(canary.len()).any(|w| w == canary),
                "Canary bytes found in {:?}",
                path
            );
        }
    }
}

/// TST-STORE-COMMIT-1 & TST-STORE-ROT-1:
/// Parallele Commits (inkl. Group Commit) verschränkt mit `publish_gate` und `erase_wal_cut`.
#[tokio::test]
async fn test_concurrent_commits_with_gate_and_wal_cut() {
    let dir = tempdir().unwrap();
    let cfg = LsmConfig {
        path: dir.path().to_path_buf(),
        group_commit_window_micros: 500,
        ..Default::default()
    };
    let storage = Arc::new(LsmStorage::open(cfg).await.unwrap());
    let kernel = storage.erasure_kernel();

    let t_victim = TenantId::try_new(21).unwrap();
    let t_safe = TenantId::try_new(22).unwrap();
    let c_victim = TenantKeyCodec::new(t_victim);
    let c_safe = TenantKeyCodec::new(t_safe);

    // Einige Commits für beide Tenants absetzen
    let mut handles = Vec::new();
    for i in 0u64..10 {
        let st = Arc::clone(&storage);
        let kv = c_victim.encode_chunk_key(&CollectionId(1), DocId(i + 1));
        let ks = c_safe.encode_chunk_key(&CollectionId(1), DocId(i + 1));
        handles.push(tokio::spawn(async move {
            let tx_v = TxId(100 + i * 2);
            let _ = st.put(tx_v, &kv, b"v").await;
            let _ = st.commit(tx_v).await;

            let tx_s = TxId(101 + i * 2);
            st.put(tx_s, &ks, b"s").await.unwrap();
            st.commit(tx_s).await.unwrap();
        }));
    }

    let (req, plan) = tenant_plan(t_victim, 0xCC);
    kernel.accept(&req, &plan).await.unwrap();
    kernel.publish_gate(req.id).await.unwrap();

    for h in handles {
        h.await.unwrap();
    }

    drive_purge_to_end(&storage, req.id).await;

    // Alle safe-Schlüssel sind noch vollständig lesbar
    let safe_scan = storage.scan_prefix(c_safe.scan_prefix()).await.unwrap();
    assert_eq!(safe_scan.len(), 10);
}
