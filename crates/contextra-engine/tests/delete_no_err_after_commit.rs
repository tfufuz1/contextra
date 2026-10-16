#![forbid(unsafe_code)]

use contextra_engine::collection::crud::DocEraseOptions;
use contextra_engine::collection::Collection;
use contextra_engine::deletion::forget_options::ForgetOptions;
use contextra_engine::Contextra;
use contextra_ports::erasure::{
    ErasureBackend, FenceGuard, FreeSpaceProbe, LogicalScope, PhysicalScope, PurgeError, PurgeId, PurgeOutput,
};
use contextra_ports::{BoxFuture, StorageEngine, StorageStats, VectorIndex};
use contextra_store::{LsmConfig, LsmStorage};
use contextra_types::{ContextraError, Result, TxId};
use contextra_vector::HnswIndex;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use tempfile::TempDir;

struct MockChangingErasureBackend {
    inner_storage: Arc<LsmStorage>,
    call_count: Arc<AtomicUsize>,
}

impl ErasureBackend for MockChangingErasureBackend {
    fn physical_scope(&self, scope: &LogicalScope) -> std::result::Result<PhysicalScope, ContextraError> {
        if let Some(b) = self.inner_storage.erasure_backend() {
            b.physical_scope(scope)
        } else {
            Err(ContextraError::InvalidInput("mock".to_string()))
        }
    }

    fn acquire_fence(&self, scope: &PhysicalScope, id: PurgeId) -> std::result::Result<FenceGuard, ContextraError> {
        if let Some(b) = self.inner_storage.erasure_backend() {
            b.acquire_fence(scope, id)
        } else {
            Ok(FenceGuard { purge_id: id })
        }
    }

    fn purge_physical(
        &self,
        scope: &PhysicalScope,
        as_of: TxId,
        probe: &dyn FreeSpaceProbe,
    ) -> std::result::Result<PurgeOutput, PurgeError> {
        if let Some(b) = self.inner_storage.erasure_backend() {
            b.purge_physical(scope, as_of, probe)
        } else {
            Ok(PurgeOutput::default())
        }
    }

    fn snapshot_write_seq(&self, prefixes: &[Vec<u8>]) -> u64 {
        let base = if let Some(b) = self.inner_storage.erasure_backend() {
            b.snapshot_write_seq(prefixes)
        } else {
            0
        };
        let count = self.call_count.fetch_add(1, Ordering::SeqCst);
        base + (count as u64)
    }
}

struct MockStorageWithBackend<S: StorageEngine> {
    inner: Arc<S>,
    backend: MockChangingErasureBackend,
}

impl<S: StorageEngine> StorageEngine for MockStorageWithBackend<S> {
    fn get<'a>(&'a self, key: &'a [u8]) -> BoxFuture<'a, Result<Option<bytes::Bytes>>> {
        self.inner.get(key)
    }

    fn get_at_seq<'a>(&'a self, key: &'a [u8], seq: u64) -> BoxFuture<'a, Result<Option<bytes::Bytes>>> {
        self.inner.get_at_seq(key, seq)
    }

    fn get_tracked<'a>(&'a self, tx_id: TxId, key: &'a [u8]) -> BoxFuture<'a, Result<Option<bytes::Bytes>>> {
        self.inner.get_tracked(tx_id, key)
    }

    fn get_at_seq_tracked<'a>(
        &'a self,
        tx_id: TxId,
        key: &'a [u8],
        seq: u64,
    ) -> BoxFuture<'a, Result<Option<bytes::Bytes>>> {
        self.inner.get_at_seq_tracked(tx_id, key, seq)
    }

    fn scan_prefix_tracked<'a>(
        &'a self,
        tx_id: TxId,
        prefix: &'a [u8],
    ) -> BoxFuture<'a, Result<Vec<(Vec<u8>, Vec<u8>)>>> {
        self.inner.scan_prefix_tracked(tx_id, prefix)
    }

    fn supports_ssi_tracking(&self) -> bool {
        self.inner.supports_ssi_tracking()
    }

    fn put<'a>(&'a self, tx_id: TxId, key: &'a [u8], value: &'a [u8]) -> BoxFuture<'a, Result<()>> {
        self.inner.put(tx_id, key, value)
    }

    fn put_if_absent<'a>(
        &'a self,
        tx_id: TxId,
        key: &'a [u8],
        value: &'a [u8],
    ) -> BoxFuture<'a, Result<bool>> {
        self.inner.put_if_absent(tx_id, key, value)
    }

    fn put_batch<'a>(
        &'a self,
        tx_id: TxId,
        entries: &'a [(Vec<u8>, Vec<u8>)],
    ) -> BoxFuture<'a, Result<()>> {
        self.inner.put_batch(tx_id, entries)
    }

    fn delete<'a>(&'a self, tx_id: TxId, key: &'a [u8]) -> BoxFuture<'a, Result<()>> {
        self.inner.delete(tx_id, key)
    }

    fn delete_many<'a>(&'a self, tx_id: TxId, keys: Vec<Vec<u8>>) -> BoxFuture<'a, Result<u64>> {
        self.inner.delete_many(tx_id, keys)
    }

    fn delete_prefix<'a>(&'a self, tx_id: TxId, prefix: &'a [u8]) -> BoxFuture<'a, Result<u64>> {
        self.inner.delete_prefix(tx_id, prefix)
    }

    fn commit<'a>(&'a self, tx_id: TxId) -> BoxFuture<'a, Result<()>> {
        self.inner.commit(tx_id)
    }

    fn rollback<'a>(&'a self, tx_id: TxId) -> BoxFuture<'a, Result<()>> {
        self.inner.rollback(tx_id)
    }

    fn rollback_to_tx<'a>(&'a self, tx_id: TxId) -> BoxFuture<'a, Result<()>> {
        self.inner.rollback_to_tx(tx_id)
    }

    fn flush<'a>(&'a self) -> BoxFuture<'a, Result<()>> {
        self.inner.flush()
    }

    fn stats<'a>(&'a self) -> BoxFuture<'a, Result<StorageStats>> {
        self.inner.stats()
    }

    fn last_seq_no<'a>(&'a self) -> BoxFuture<'a, Result<u64>> {
        self.inner.last_seq_no()
    }

    fn last_tx_id<'a>(&'a self) -> BoxFuture<'a, Result<TxId>> {
        self.inner.last_tx_id()
    }

    fn pin_checkpoint<'a>(&'a self, seq_no: u64) -> BoxFuture<'a, Result<()>> {
        self.inner.pin_checkpoint(seq_no)
    }

    fn unpin_checkpoint<'a>(&'a self, seq_no: u64) -> BoxFuture<'a, Result<()>> {
        self.inner.unpin_checkpoint(seq_no)
    }

    fn scan_prefix<'a>(
        &'a self,
        prefix: &'a [u8],
    ) -> BoxFuture<'a, Result<Vec<(Vec<u8>, Vec<u8>)>>> {
        self.inner.scan_prefix(prefix)
    }

    fn scan_prefix_bounded<'a>(
        &'a self,
        prefix: &'a [u8],
        limit: usize,
        cursor: Option<&'a [u8]>,
    ) -> BoxFuture<'a, Result<(Vec<(Vec<u8>, Vec<u8>)>, Option<Vec<u8>>)>> {
        self.inner.scan_prefix_bounded(prefix, limit, cursor)
    }

    fn scan_prefix_at<'a>(
        &'a self,
        prefix: &'a [u8],
        seq_no: u64,
    ) -> BoxFuture<'a, Result<Vec<(Vec<u8>, Vec<u8>)>>> {
        self.inner.scan_prefix_at(prefix, seq_no)
    }

    fn scan<'a>(
        &'a self,
        start: std::ops::Bound<&'a [u8]>,
        end: std::ops::Bound<&'a [u8]>,
        limit: Option<usize>,
    ) -> BoxFuture<'a, Result<Vec<(Vec<u8>, Vec<u8>)>>> {
        self.inner.scan(start, end, limit)
    }

    fn scan_bounded<'a>(
        &'a self,
        start: std::ops::Bound<&'a [u8]>,
        end: std::ops::Bound<&'a [u8]>,
        limit: usize,
        cursor: Option<&'a [u8]>,
    ) -> BoxFuture<'a, Result<(Vec<(Vec<u8>, Vec<u8>)>, Option<Vec<u8>>)>> {
        self.inner.scan_bounded(start, end, limit, cursor)
    }

    fn erasure_backend(&self) -> Option<&dyn ErasureBackend> {
        Some(&self.backend)
    }
}

#[tokio::test]
async fn test_concurrent_modification_err_before_commit() {
    let tmp = TempDir::new().expect("temp dir");
    let lsm = Arc::new(LsmStorage::open(tmp.path(), LsmConfig::default()).expect("open lsm"));

    let call_count = Arc::new(AtomicUsize::new(0));
    let mock_storage = Arc::new(MockStorageWithBackend {
        inner: lsm.clone(),
        backend: MockChangingErasureBackend {
            inner_storage: lsm.clone(),
            call_count: call_count.clone(),
        },
    });

    let index = Arc::new(HnswIndex::new(128, contextra_types::DistanceMetric::Cosine));
    let next_tx = Arc::new(AtomicU64::new(1));

    let col = Collection::new(
        "default".to_string(),
        mock_storage,
        index,
        next_tx,
        128,
        contextra_text::Language::English,
    );

    let doc_id = "doc_concurrent_test";
    let embedding = vec![0.1f32; col.dimension()];
    col.insert(doc_id, &embedding, None)
        .await
        .expect("insert");

    // Enable purge mode / non-off mode so that erasure_footprint and write sequence checks run
    let opts = DocEraseOptions {
        options: ForgetOptions {
            assurance: contextra_crypto::deletion_proof::claim_text::WitAssurance::S0,
            purge: contextra_engine::deletion::forget_options::PurgeMode::Immediate,
            ..Default::default()
        },
    };

    let res = col.erase_document(doc_id, &opts).await;
    assert!(
        matches!(res, Err(ContextraError::Conflict(_))),
        "Concurrent modification before commit must return ContextraError::Conflict, got: {:?}",
        res
    );
}

#[tokio::test]
async fn test_no_err_returned_after_tombstone_commit() {
    let tmp = TempDir::new().expect("temp dir");
    let db = Contextra::open(tmp.path()).await.expect("open db");
    let col = db.collection("default").await.expect("col");

    let doc_id = "doc_post_commit_test";
    let embedding = vec![0.3f32; col.dimension()];
    col.insert(doc_id, &embedding, None)
        .await
        .expect("insert");

    // Deleting a committed document succeeds and guarantees no post-commit error propagation
    let res = col.delete(doc_id).await;
    assert!(res.is_ok(), "delete must return Ok(()) after tombstone commit");

    // Repeat delete on already deleted/non-existent document also returns Ok(())
    let res_repeat = col.delete(doc_id).await;
    assert!(res_repeat.is_ok(), "Repeat delete on missing document returns Ok(())");

    db.close().await.unwrap();
}
