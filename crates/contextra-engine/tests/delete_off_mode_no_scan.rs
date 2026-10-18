#![forbid(unsafe_code)]

use contextra_engine::collection::crud::DocEraseOptions;
use contextra_engine::collection::Collection;
use contextra_engine::deletion::forget_options::ForgetOptions;
use contextra_ports::{
    BoxFuture, Clock, ErasureBackend, KvLifecycleHooks, MetricsSink, StorageEngine, StorageStats, VectorIndex,
};
use contextra_store::tenant_codec::TenantScopedStorage;
use contextra_store::{LsmConfig, LsmStorage};
use contextra_testkit::erasure_harness::tenant_matrix::TenantMatrix;
use contextra_types::{Result, TxId};
use contextra_vector::HnswIndex;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use tempfile::TempDir;

struct CounterStorage<S: StorageEngine> {
    inner: Arc<S>,
    scan_count: Arc<AtomicUsize>,
}

impl<S: StorageEngine> StorageEngine for CounterStorage<S> {
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
        self.scan_count.fetch_add(1, Ordering::SeqCst);
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
        self.scan_count.fetch_add(1, Ordering::SeqCst);
        self.inner.scan_prefix(prefix)
    }

    fn scan_prefix_bounded<'a>(
        &'a self,
        prefix: &'a [u8],
        limit: usize,
        cursor: Option<&'a [u8]>,
    ) -> BoxFuture<'a, Result<(Vec<(Vec<u8>, Vec<u8>)>, Option<Vec<u8>>)>> {
        self.scan_count.fetch_add(1, Ordering::SeqCst);
        self.inner.scan_prefix_bounded(prefix, limit, cursor)
    }

    fn scan_prefix_at<'a>(
        &'a self,
        prefix: &'a [u8],
        seq_no: u64,
    ) -> BoxFuture<'a, Result<Vec<(Vec<u8>, Vec<u8>)>>> {
        self.scan_count.fetch_add(1, Ordering::SeqCst);
        self.inner.scan_prefix_at(prefix, seq_no)
    }

    fn scan<'a>(
        &'a self,
        start: std::ops::Bound<&'a [u8]>,
        end: std::ops::Bound<&'a [u8]>,
        limit: Option<usize>,
    ) -> BoxFuture<'a, Result<Vec<(Vec<u8>, Vec<u8>)>>> {
        self.scan_count.fetch_add(1, Ordering::SeqCst);
        self.inner.scan(start, end, limit)
    }

    fn scan_bounded<'a>(
        &'a self,
        start: std::ops::Bound<&'a [u8]>,
        end: std::ops::Bound<&'a [u8]>,
        limit: usize,
        cursor: Option<&'a [u8]>,
    ) -> BoxFuture<'a, Result<(Vec<(Vec<u8>, Vec<u8>)>, Option<Vec<u8>>)>> {
        self.scan_count.fetch_add(1, Ordering::SeqCst);
        self.inner.scan_bounded(start, end, limit, cursor)
    }

    fn erasure_backend(&self) -> Option<&dyn ErasureBackend> {
        self.inner.erasure_backend()
    }
}

#[tokio::test]
async fn test_off_mode_no_footprint_scan_counter() {
    let tmp = TempDir::new().expect("temp dir");
    let lsm = LsmStorage::open(tmp.path(), LsmConfig::default()).expect("open lsm");
    let scan_count = Arc::new(AtomicUsize::new(0));

    let counter_storage = Arc::new(CounterStorage {
        inner: lsm,
        scan_count: scan_count.clone(),
    });

    let index = Arc::new(HnswIndex::new(128, contextra_types::DistanceMetric::Cosine));
    let next_tx = Arc::new(AtomicU64::new(1));

    let col = Collection::new(
        "default".to_string(),
        counter_storage,
        index,
        next_tx,
        128,
        contextra_text::Language::English,
    );

    let doc_id = "doc_off_mode_test";
    let embedding = vec![0.1f32; col.dimension()];
    col.insert(doc_id, &embedding, None)
        .await
        .expect("insert");

    // Reset scan counter after insert
    scan_count.store(0, Ordering::SeqCst);

    let opts_off = DocEraseOptions {
        options: ForgetOptions {
            assurance: contextra_crypto::deletion_proof::claim_text::WitAssurance::S0,
            purge: contextra_engine::deletion::forget_options::PurgeMode::None,
            ..Default::default()
        },
    };

    let erasure = col
        .erase_document(doc_id, &opts_off)
        .await
        .expect("off-mode erase should succeed");

    assert_eq!(
        scan_count.load(Ordering::SeqCst),
        0,
        "Off mode deletion path must perform EXACTLY zero footprint calculations or storage scans"
    );

    assert_eq!(
        erasure.footprint_hash,
        [0u8; 32],
        "Footprint hash must be zero in Off mode"
    );
    assert!(
        erasure.prefixes.is_empty(),
        "Prefixes must be empty in Off mode"
    );
    assert_eq!(
        erasure.keys.len(),
        2,
        "Explicit keys must strictly contain user_key and doc_key in Off mode"
    );
}

#[tokio::test]
async fn test_k1_14_delete_under_lsm_and_tenant_scoped_storage() {
    TenantMatrix::run(|harness| async move {
        let col = harness.collection();

        let doc_id = "doc_tenant_test";
        let embedding = vec![0.2f32; col.dimension()];
        col.insert(doc_id, &embedding, None)
            .await
            .expect("insert");

        let res = col.delete(doc_id).await;
        assert!(
            res.is_ok(),
            "Standard delete under TenantMatrix (LsmStorage and TenantScopedStorage) must succeed without CapabilityUnsupported: {:?}",
            res.err()
        );
    })
    .await;
}
