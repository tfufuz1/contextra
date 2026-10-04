// FILE-CONTEXT
// ZWECK: Engine transaction rollback test using a mock KvLifecycleHooks implementation.

use contextra_engine::transaction::DbTransaction;
use contextra_engine::Collection;
use contextra_graph::CsrGraph;
use contextra_ports::KvLifecycleHooks;
use contextra_store::LsmStorage;
use contextra_types::{DocId, TenantId};
use contextra_vector::HnswIndex;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use tempfile::tempdir;

#[derive(Default)]
struct MockKvLifecycleHooks {
    rollback_count: AtomicUsize,
    last_tenant: parking_lot::Mutex<Option<TenantId>>,
    last_chunk_ids: parking_lot::Mutex<Vec<u64>>,
}

impl KvLifecycleHooks for MockKvLifecycleHooks {
    fn on_rollback(&self, tenant: TenantId, chunk_ids: &[u64]) {
        self.rollback_count.fetch_add(1, Ordering::SeqCst);
        *self.last_tenant.lock() = Some(tenant);
        *self.last_chunk_ids.lock() = chunk_ids.to_vec();
    }

    fn remove_doc_segments(&self, _tenant: TenantId, _doc: DocId) {}

    fn purge_tenant(&self, _tenant: TenantId) {}
}

async fn create_test_collection() -> Collection<LsmStorage, HnswIndex> {
    let dir = tempdir().unwrap();
    let lsm_config = contextra_store::LsmConfig {
        path: dir.path().to_path_buf(),
        ..Default::default()
    };
    let storage = Arc::new(LsmStorage::new(lsm_config).await.unwrap());
    let index = Arc::new(
        HnswIndex::try_new(contextra_vector::HnswConfig {
            dimension: 4,
            ..Default::default()
        })
        .unwrap(),
    );
    let graph = Arc::new(CsrGraph::new());
    let next_tx = Arc::new(AtomicU64::new(1));

    Collection::new(
        "tx_kv_hooks_test".to_string(),
        storage,
        index,
        graph,
        next_tx,
        4,
        contextra_text::Language::English,
    )
}

#[tokio::test]
async fn test_transaction_rollback_triggers_kv_hooks_on_rollback() {
    let mock_hooks = Arc::new(MockKvLifecycleHooks::default());
    let col = create_test_collection()
        .await
        .with_kv_hooks(mock_hooks.clone());

    let tx_id = col.allocate_tx().unwrap();
    let tx = DbTransaction::new(col, tx_id);

    let doc_id1 = DocId::new(101);
    let doc_id2 = DocId::new(202);
    tx.record_keys(vec![1, 2, 3], vec![3, 2, 1], doc_id1);
    tx.record_keys(vec![4, 5, 6], vec![6, 5, 4], doc_id2);

    let rollback_res = tx.rollback().await;
    assert!(rollback_res.is_ok());

    assert_eq!(
        mock_hooks.rollback_count.load(Ordering::SeqCst),
        1,
        "on_rollback must be called exactly once during transaction rollback"
    );

    let last_chunk_ids = mock_hooks.last_chunk_ids.lock().clone();
    assert_eq!(last_chunk_ids.len(), 2);
    assert!(last_chunk_ids.contains(&101));
    assert!(last_chunk_ids.contains(&202));
}
