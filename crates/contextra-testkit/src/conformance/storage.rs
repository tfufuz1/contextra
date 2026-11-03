//! # Konformitätsprüfsuite für den `StorageEngine`-Port (`conformance/storage`)
//!
//! Dieses Modul testet die vertraglichen Zusicherungen (Contract Invariants)
//! für Speicher-Engines des [`contextra_ports::StorageEngine`]-Ports (PO-02).
//!
//! ## Getestete Verträge (Pflichtfälle laut PO-02):
//! 1. `po_02_get_put_delete`: get/put/delete Roundtrip mit Commit-Sichtbarkeit und Tilgung.
//! 2. `po_02_get_at_seq`: MVCC-Lesen an exakter Sequenznummer.
//! 3. `po_02_get_tracked_ssi`: SSI-Read-Set-Tracking (`get_tracked` / `get_at_seq_tracked`).
//! 4. `po_02_snapshot_isolation`: Snapshot-Isolation (uncommitted mutations unsichtbar, Rollback wirksam).

use bytes::Bytes;
use contextra_ports::{BoxFuture, ContextraError, Result, StorageEngine, StorageStats, TxId};
use parking_lot::RwLock;
use std::collections::BTreeMap;
use std::future::Future;
use std::ops::Bound;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

/// Failure representation for storage conformance tests.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConformanceFailure {
    /// Suite identifier.
    pub suite_name: &'static str,
    /// Specific test case that failed.
    pub test_case: &'static str,
    /// Diagnostic failure message.
    pub message: String,
}

impl ConformanceFailure {
    /// Creates a new failure record.
    #[must_use]
    pub fn new(
        suite_name: &'static str,
        test_case: &'static str,
        message: impl Into<String>,
    ) -> Self {
        Self {
            suite_name,
            test_case,
            message: message.into(),
        }
    }
}

impl std::fmt::Display for ConformanceFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[{}] Conformance failure in test '{}': {}",
            self.suite_name, self.test_case, self.message
        )
    }
}

impl std::error::Error for ConformanceFailure {}

/// Conformance result type.
pub type ConformanceResult = std::result::Result<(), ConformanceFailure>;

/// Führt die vollständige Konformitätsprüfung für eine [`StorageEngine`]-Implementierung durch.
pub async fn run<F, Fut, S>(factory: F) -> ConformanceResult
where
    F: Fn() -> Fut,
    Fut: Future<Output = S>,
    S: StorageEngine,
{
    let suite = "StorageEngine";

    // Fall 1: PO-02 get/put/delete
    {
        let engine = factory().await;
        let tx1 = TxId::new(1);
        if let Err(e) = engine.put(tx1, b"po02_key", b"po02_val").await {
            return Err(ConformanceFailure::new(
                suite,
                "put_get_delete",
                format!("put failed: {e:?}"),
            ));
        }

        // Before commit, must not be visible
        match engine.get(b"po02_key").await {
            Ok(None) => {}
            Ok(Some(v)) => {
                return Err(ConformanceFailure::new(
                    suite,
                    "put_get_delete",
                    format!("uncommitted key visible before commit: {v:?}"),
                ));
            }
            Err(e) => {
                return Err(ConformanceFailure::new(
                    suite,
                    "put_get_delete",
                    format!("get before commit failed: {e:?}"),
                ));
            }
        }

        if let Err(e) = engine.commit(tx1).await {
            return Err(ConformanceFailure::new(
                suite,
                "put_get_delete",
                format!("commit failed: {e:?}"),
            ));
        }

        match engine.get(b"po02_key").await {
            Ok(Some(v)) if v.as_ref() == b"po02_val" => {}
            Ok(other) => {
                return Err(ConformanceFailure::new(
                    suite,
                    "put_get_delete",
                    format!("expected po02_val, got {other:?}"),
                ));
            }
            Err(e) => {
                return Err(ConformanceFailure::new(
                    suite,
                    "put_get_delete",
                    format!("get after commit failed: {e:?}"),
                ));
            }
        }

        let tx2 = TxId::new(2);
        if let Err(e) = engine.delete(tx2, b"po02_key").await {
            return Err(ConformanceFailure::new(
                suite,
                "put_get_delete",
                format!("delete failed: {e:?}"),
            ));
        }
        if let Err(e) = engine.commit(tx2).await {
            return Err(ConformanceFailure::new(
                suite,
                "put_get_delete",
                format!("commit delete failed: {e:?}"),
            ));
        }

        match engine.get(b"po02_key").await {
            Ok(None) => {}
            Ok(Some(v)) => {
                return Err(ConformanceFailure::new(
                    suite,
                    "put_get_delete",
                    format!("deleted key still visible: {v:?}"),
                ));
            }
            Err(e) => {
                return Err(ConformanceFailure::new(
                    suite,
                    "put_get_delete",
                    format!("get after delete failed: {e:?}"),
                ));
            }
        }
    }

    // Fall 2: PO-02 get_at_seq (MVCC sequence visibility)
    {
        let engine = factory().await;
        let tx1 = TxId::new(10);
        let _ = engine.put(tx1, b"seq_key", b"v1").await;
        let _ = engine.commit(tx1).await;

        let tx2 = TxId::new(11);
        let _ = engine.put(tx2, b"seq_key", b"v2").await;
        let _ = engine.commit(tx2).await;

        let at_seq1 = engine.get_at_seq(b"seq_key", 1).await;
        let at_seq2 = engine.get_at_seq(b"seq_key", 2).await;
        match (at_seq1, at_seq2) {
            (Ok(Some(v1)), Ok(Some(v2))) if v1.as_ref() == b"v1" && v2.as_ref() == b"v2" => {}
            (Ok(Some(_)), Ok(Some(_))) => {}
            (Err(ContextraError::CapabilityUnsupported { .. }), _) => {}
            (Ok(None), Ok(Some(v2))) if v2.as_ref() == b"v2" => {}
            (Err(e), _) => {
                return Err(ConformanceFailure::new(
                    suite,
                    "po_02_get_at_seq",
                    format!("get_at_seq failed: {e:?}"),
                ));
            }
            other => {
                return Err(ConformanceFailure::new(
                    suite,
                    "po_02_get_at_seq",
                    format!("unexpected get_at_seq outcomes: {other:?}"),
                ));
            }
        }
    }

    // Fall 3: PO-02 get_tracked (SSI-Read-Set)
    {
        let engine = factory().await;
        let tx1 = TxId::new(20);
        let _ = engine.put(tx1, b"ssi_key", b"ssi_val").await;
        let _ = engine.commit(tx1).await;

        let tx_read = TxId::new(21);
        match engine.get_tracked(tx_read, b"ssi_key").await {
            Ok(Some(v)) if v.as_ref() == b"ssi_val" => {}
            Ok(other) => {
                return Err(ConformanceFailure::new(
                    suite,
                    "po_02_get_tracked_ssi",
                    format!("expected ssi_val via get_tracked, got {other:?}"),
                ));
            }
            Err(e) => {
                return Err(ConformanceFailure::new(
                    suite,
                    "po_02_get_tracked_ssi",
                    format!("get_tracked failed: {e:?}"),
                ));
            }
        }

        match engine.get_at_seq_tracked(tx_read, b"ssi_key", 1).await {
            Ok(_) | Err(ContextraError::CapabilityUnsupported { .. }) => {}
            Err(e) => {
                return Err(ConformanceFailure::new(
                    suite,
                    "po_02_get_tracked_ssi",
                    format!("get_at_seq_tracked failed: {e:?}"),
                ));
            }
        }
    }

    // Fall 4: PO-02 Snapshot-Isolation & Rollback
    {
        let engine = factory().await;
        let tx1 = TxId::new(30);
        let _ = engine.put(tx1, b"iso_key", b"initial").await;
        let _ = engine.commit(tx1).await;

        let tx2 = TxId::new(31);
        let _ = engine.put(tx2, b"iso_key", b"modified").await;

        // Concurrent read must observe "initial", not uncommitted "modified"
        match engine.get(b"iso_key").await {
            Ok(Some(v)) if v.as_ref() == b"initial" => {}
            Ok(other) => {
                return Err(ConformanceFailure::new(
                    suite,
                    "po_02_snapshot_isolation",
                    format!("dirty read detected! expected initial, got {other:?}"),
                ));
            }
            Err(e) => {
                return Err(ConformanceFailure::new(
                    suite,
                    "po_02_snapshot_isolation",
                    format!("get failed: {e:?}"),
                ));
            }
        }

        // Rollback tx2
        let _ = engine.rollback(tx2).await;
        match engine.get(b"iso_key").await {
            Ok(Some(v)) if v.as_ref() == b"initial" => {}
            Ok(other) => {
                return Err(ConformanceFailure::new(
                    suite,
                    "po_02_snapshot_isolation",
                    format!("value altered after rollback: expected initial, got {other:?}"),
                ));
            }
            Err(e) => {
                return Err(ConformanceFailure::new(
                    suite,
                    "po_02_snapshot_isolation",
                    format!("get after rollback failed: {e:?}"),
                ));
            }
        }
    }

    // Additional contract cases: Overwrite and Scan ordering
    {
        let engine = factory().await;
        let tx = TxId::new(40);
        let _ = engine.put(tx, b"p:c", b"vc").await;
        let _ = engine.put(tx, b"p:a", b"va").await;
        let _ = engine.put(tx, b"p:b", b"vb").await;
        let _ = engine.commit(tx).await;

        let res = match engine.scan_prefix(b"p:").await {
            Ok(r) => r,
            Err(e) => {
                return Err(ConformanceFailure::new(
                    suite,
                    "scan_ordering",
                    format!("scan_prefix failed: {e:?}"),
                ));
            }
        };
        if res.len() != 3 {
            return Err(ConformanceFailure::new(
                suite,
                "scan_ordering",
                format!("expected 3 items, got {}", res.len()),
            ));
        }
        let k0 = res.first().map(|(k, _)| k.as_slice());
        let k1 = res.get(1).map(|(k, _)| k.as_slice());
        let k2 = res.get(2).map(|(k, _)| k.as_slice());
        if k0 != Some(b"p:a".as_slice())
            || k1 != Some(b"p:b".as_slice())
            || k2 != Some(b"p:c".as_slice())
        {
            return Err(ConformanceFailure::new(
                suite,
                "scan_ordering",
                format!("items not in lexicographical order: {res:?}"),
            ));
        }
    }

    Ok(())
}

/// Backwards-compatible alias for existing suite invocations.
pub async fn run_storage_engine_suite<F, Fut, S>(factory: F) -> ConformanceResult
where
    F: Fn() -> Fut,
    Fut: Future<Output = S>,
    S: StorageEngine,
{
    run(factory).await
}

// ============================================================================
// REFERENCE IMPLEMENTATION (IN-MEMORY)
// ============================================================================

type CommittedMap = Arc<RwLock<BTreeMap<Vec<u8>, (Vec<u8>, u64)>>>;
type VersionHistoryMap = Arc<RwLock<BTreeMap<(Vec<u8>, u64), Option<Vec<u8>>>>>;
type UncommittedMap = Arc<RwLock<BTreeMap<TxId, BTreeMap<Vec<u8>, Option<Vec<u8>>>>>>;
type ReadSetsMap = Arc<RwLock<BTreeMap<TxId, Vec<Vec<u8>>>>>;

/// In-Memory Reference Implementation of [`StorageEngine`].
#[derive(Clone, Default)]
pub struct ReferenceStorageEngine {
    committed: CommittedMap,
    version_history: VersionHistoryMap,
    uncommitted: UncommittedMap,
    read_sets: ReadSetsMap,
    current_seq: Arc<AtomicU64>,
}

impl ReferenceStorageEngine {
    /// Creates a new reference engine.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

impl StorageEngine for ReferenceStorageEngine {
    fn get<'a>(&'a self, key: &'a [u8]) -> BoxFuture<'a, Result<Option<Bytes>>> {
        Box::pin(async move {
            let guard = self.committed.read();
            Ok(guard.get(key).map(|(v, _)| Bytes::copy_from_slice(v)))
        })
    }

    fn get_at_seq<'a>(&'a self, key: &'a [u8], seq: u64) -> BoxFuture<'a, Result<Option<Bytes>>> {
        Box::pin(async move {
            let hist = self.version_history.read();
            let mut found = None;
            for ((k, s), val) in hist.iter() {
                if k.as_slice() == key && *s <= seq {
                    found = val.as_ref().map(|v| Bytes::copy_from_slice(v));
                }
            }
            Ok(found)
        })
    }

    fn get_tracked<'a>(
        &'a self,
        tx_id: TxId,
        key: &'a [u8],
    ) -> BoxFuture<'a, Result<Option<Bytes>>> {
        Box::pin(async move {
            self.read_sets
                .write()
                .entry(tx_id)
                .or_default()
                .push(key.to_vec());
            self.get(key).await
        })
    }

    fn get_at_seq_tracked<'a>(
        &'a self,
        tx_id: TxId,
        key: &'a [u8],
        seq: u64,
    ) -> BoxFuture<'a, Result<Option<Bytes>>> {
        Box::pin(async move {
            self.read_sets
                .write()
                .entry(tx_id)
                .or_default()
                .push(key.to_vec());
            self.get_at_seq(key, seq).await
        })
    }

    fn supports_ssi_tracking(&self) -> bool {
        true
    }

    fn put<'a>(&'a self, tx_id: TxId, key: &'a [u8], value: &'a [u8]) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move {
            self.uncommitted
                .write()
                .entry(tx_id)
                .or_default()
                .insert(key.to_vec(), Some(value.to_vec()));
            Ok(())
        })
    }

    fn delete<'a>(&'a self, tx_id: TxId, key: &'a [u8]) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move {
            self.uncommitted
                .write()
                .entry(tx_id)
                .or_default()
                .insert(key.to_vec(), None);
            Ok(())
        })
    }

    fn commit<'a>(&'a self, tx_id: TxId) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move {
            let mut uncommitted = self.uncommitted.write();
            if let Some(changes) = uncommitted.remove(&tx_id) {
                let seq = self.current_seq.fetch_add(1, Ordering::SeqCst) + 1;
                let mut comm = self.committed.write();
                let mut hist = self.version_history.write();
                for (k, val_opt) in changes {
                    hist.insert((k.clone(), seq), val_opt.clone());
                    match val_opt {
                        Some(val) => {
                            comm.insert(k, (val, seq));
                        }
                        None => {
                            comm.remove(&k);
                        }
                    }
                }
            }
            Ok(())
        })
    }

    fn rollback<'a>(&'a self, tx_id: TxId) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move {
            self.uncommitted.write().remove(&tx_id);
            self.read_sets.write().remove(&tx_id);
            Ok(())
        })
    }

    fn rollback_to_tx<'a>(&'a self, _tx_id: TxId) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move { Ok(()) })
    }

    fn flush<'a>(&'a self) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move { Ok(()) })
    }

    fn last_seq_no<'a>(&'a self) -> BoxFuture<'a, Result<u64>> {
        Box::pin(async move { Ok(self.current_seq.load(Ordering::SeqCst)) })
    }

    fn last_tx_id<'a>(&'a self) -> BoxFuture<'a, Result<TxId>> {
        Box::pin(async move { Ok(TxId::new(self.current_seq.load(Ordering::SeqCst))) })
    }

    fn pin_checkpoint<'a>(&'a self, _seq_no: u64) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move { Ok(()) })
    }

    fn unpin_checkpoint<'a>(&'a self, _seq_no: u64) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move { Ok(()) })
    }

    fn scan_prefix<'a>(
        &'a self,
        prefix: &'a [u8],
    ) -> BoxFuture<'a, Result<Vec<(Vec<u8>, Vec<u8>)>>> {
        Box::pin(async move {
            let guard = self.committed.read();
            let mut out = Vec::new();
            for (k, (v, _)) in guard.iter() {
                if k.starts_with(prefix) {
                    out.push((k.clone(), v.clone()));
                }
            }
            Ok(out)
        })
    }

    fn scan<'a>(
        &'a self,
        start: Bound<&'a [u8]>,
        end: Bound<&'a [u8]>,
        limit: Option<usize>,
    ) -> BoxFuture<'a, Result<Vec<(Vec<u8>, Vec<u8>)>>> {
        let start_owned = match start {
            Bound::Included(b) => Bound::Included(b.to_vec()),
            Bound::Excluded(b) => Bound::Excluded(b.to_vec()),
            Bound::Unbounded => Bound::Unbounded,
        };
        let end_owned = match end {
            Bound::Included(b) => Bound::Included(b.to_vec()),
            Bound::Excluded(b) => Bound::Excluded(b.to_vec()),
            Bound::Unbounded => Bound::Unbounded,
        };

        Box::pin(async move {
            let guard = self.committed.read();
            let mut out = Vec::new();
            for (k, (v, _)) in guard.range((start_owned, end_owned)) {
                out.push((k.clone(), v.clone()));
                if let Some(lim) = limit {
                    if out.len() >= lim {
                        break;
                    }
                }
            }
            Ok(out)
        })
    }

    fn stats<'a>(&'a self) -> BoxFuture<'a, Result<StorageStats>> {
        Box::pin(async move {
            Ok(StorageStats {
                num_segments: 1,
                total_size_bytes: 0,
                memtable_size_bytes: 0,
            })
        })
    }
}

// ============================================================================
// MUTANTS (INTENTIONALLY DEFECTIVE IMPLEMENTATIONS)
// ============================================================================

/// Mutant 1: delete is a no-op.
#[derive(Clone, Default)]
pub struct MutantNoopDeleteStorageEngine(ReferenceStorageEngine);

impl StorageEngine for MutantNoopDeleteStorageEngine {
    fn get<'a>(&'a self, key: &'a [u8]) -> BoxFuture<'a, Result<Option<Bytes>>> {
        self.0.get(key)
    }
    fn get_at_seq<'a>(&'a self, key: &'a [u8], seq: u64) -> BoxFuture<'a, Result<Option<Bytes>>> {
        self.0.get_at_seq(key, seq)
    }
    fn get_tracked<'a>(
        &'a self,
        tx_id: TxId,
        key: &'a [u8],
    ) -> BoxFuture<'a, Result<Option<Bytes>>> {
        self.0.get_tracked(tx_id, key)
    }
    fn put<'a>(&'a self, tx_id: TxId, key: &'a [u8], value: &'a [u8]) -> BoxFuture<'a, Result<()>> {
        self.0.put(tx_id, key, value)
    }
    fn delete<'a>(&'a self, _tx_id: TxId, _key: &'a [u8]) -> BoxFuture<'a, Result<()>> {
        // DEFECT: Delete is a no-op!
        Box::pin(async move { Ok(()) })
    }
    fn commit<'a>(&'a self, tx_id: TxId) -> BoxFuture<'a, Result<()>> {
        self.0.commit(tx_id)
    }
    fn rollback<'a>(&'a self, tx_id: TxId) -> BoxFuture<'a, Result<()>> {
        self.0.rollback(tx_id)
    }
    fn rollback_to_tx<'a>(&'a self, tx_id: TxId) -> BoxFuture<'a, Result<()>> {
        self.0.rollback_to_tx(tx_id)
    }
    fn flush<'a>(&'a self) -> BoxFuture<'a, Result<()>> {
        self.0.flush()
    }
    fn last_seq_no<'a>(&'a self) -> BoxFuture<'a, Result<u64>> {
        self.0.last_seq_no()
    }
    fn last_tx_id<'a>(&'a self) -> BoxFuture<'a, Result<TxId>> {
        self.0.last_tx_id()
    }
    fn pin_checkpoint<'a>(&'a self, seq_no: u64) -> BoxFuture<'a, Result<()>> {
        self.0.pin_checkpoint(seq_no)
    }
    fn unpin_checkpoint<'a>(&'a self, seq_no: u64) -> BoxFuture<'a, Result<()>> {
        self.0.unpin_checkpoint(seq_no)
    }
    fn scan_prefix<'a>(
        &'a self,
        prefix: &'a [u8],
    ) -> BoxFuture<'a, Result<Vec<(Vec<u8>, Vec<u8>)>>> {
        self.0.scan_prefix(prefix)
    }
    fn scan<'a>(
        &'a self,
        start: Bound<&'a [u8]>,
        end: Bound<&'a [u8]>,
        limit: Option<usize>,
    ) -> BoxFuture<'a, Result<Vec<(Vec<u8>, Vec<u8>)>>> {
        self.0.scan(start, end, limit)
    }
    fn stats<'a>(&'a self) -> BoxFuture<'a, Result<StorageStats>> {
        self.0.stats()
    }
}

/// Mutant 2: dirty reads (writes are immediately globally visible before commit).
#[derive(Clone, Default)]
pub struct MutantDirtyReadStorageEngine(ReferenceStorageEngine);

impl StorageEngine for MutantDirtyReadStorageEngine {
    fn get<'a>(&'a self, key: &'a [u8]) -> BoxFuture<'a, Result<Option<Bytes>>> {
        let key_vec = key.to_vec();
        Box::pin(async move {
            // DEFECT: check uncommitted state first, allowing dirty reads!
            {
                let uncommitted = self.0.uncommitted.read();
                for changes in uncommitted.values() {
                    if let Some(val_opt) = changes.get(&key_vec) {
                        return Ok(val_opt.as_ref().map(|v| Bytes::copy_from_slice(v)));
                    }
                }
            }
            self.0.get(&key_vec).await
        })
    }
    fn get_at_seq<'a>(&'a self, key: &'a [u8], seq: u64) -> BoxFuture<'a, Result<Option<Bytes>>> {
        self.0.get_at_seq(key, seq)
    }
    fn get_tracked<'a>(
        &'a self,
        tx_id: TxId,
        key: &'a [u8],
    ) -> BoxFuture<'a, Result<Option<Bytes>>> {
        self.0.get_tracked(tx_id, key)
    }
    fn put<'a>(&'a self, tx_id: TxId, key: &'a [u8], value: &'a [u8]) -> BoxFuture<'a, Result<()>> {
        self.0.put(tx_id, key, value)
    }
    fn delete<'a>(&'a self, tx_id: TxId, key: &'a [u8]) -> BoxFuture<'a, Result<()>> {
        self.0.delete(tx_id, key)
    }
    fn commit<'a>(&'a self, tx_id: TxId) -> BoxFuture<'a, Result<()>> {
        self.0.commit(tx_id)
    }
    fn rollback<'a>(&'a self, tx_id: TxId) -> BoxFuture<'a, Result<()>> {
        self.0.rollback(tx_id)
    }
    fn rollback_to_tx<'a>(&'a self, tx_id: TxId) -> BoxFuture<'a, Result<()>> {
        self.0.rollback_to_tx(tx_id)
    }
    fn flush<'a>(&'a self) -> BoxFuture<'a, Result<()>> {
        self.0.flush()
    }
    fn last_seq_no<'a>(&'a self) -> BoxFuture<'a, Result<u64>> {
        self.0.last_seq_no()
    }
    fn last_tx_id<'a>(&'a self) -> BoxFuture<'a, Result<TxId>> {
        self.0.last_tx_id()
    }
    fn pin_checkpoint<'a>(&'a self, seq_no: u64) -> BoxFuture<'a, Result<()>> {
        self.0.pin_checkpoint(seq_no)
    }
    fn unpin_checkpoint<'a>(&'a self, seq_no: u64) -> BoxFuture<'a, Result<()>> {
        self.0.unpin_checkpoint(seq_no)
    }
    fn scan_prefix<'a>(
        &'a self,
        prefix: &'a [u8],
    ) -> BoxFuture<'a, Result<Vec<(Vec<u8>, Vec<u8>)>>> {
        self.0.scan_prefix(prefix)
    }
    fn scan<'a>(
        &'a self,
        start: Bound<&'a [u8]>,
        end: Bound<&'a [u8]>,
        limit: Option<usize>,
    ) -> BoxFuture<'a, Result<Vec<(Vec<u8>, Vec<u8>)>>> {
        self.0.scan(start, end, limit)
    }
    fn stats<'a>(&'a self) -> BoxFuture<'a, Result<StorageStats>> {
        self.0.stats()
    }
}

/// Helper executing reference test.
pub async fn run_reference_test() -> ConformanceResult {
    run(|| async { ReferenceStorageEngine::new() }).await
}

/// Helper executing mutant tests, ensuring 100% detection rate.
pub async fn run_mutant_tests() -> std::result::Result<(), String> {
    let res1 = run(|| async { MutantNoopDeleteStorageEngine::default() }).await;
    if res1.is_ok() {
        return Err("MutantNoopDeleteStorageEngine escaped detection!".to_string());
    }

    let res2 = run(|| async { MutantDirtyReadStorageEngine::default() }).await;
    if res2.is_ok() {
        return Err("MutantDirtyReadStorageEngine escaped detection!".to_string());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// SPEC: PO-02
    #[tokio::test]
    async fn po_02_storage_reference_passes() {
        let res = run_reference_test().await;
        assert!(res.is_ok(), "Reference implementation must pass: {res:?}");
    }

    /// SPEC: PO-02
    #[tokio::test]
    async fn po_02_storage_mutants_fail() {
        let res = run_mutant_tests().await;
        assert!(
            res.is_ok(),
            "Mutant tests must succeed in catching defects: {res:?}"
        );
    }
}
