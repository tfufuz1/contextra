use super::db_transaction::DbTransaction;
use super::intent::CommitIntent;
use crate::Collection;
use contextra_graph::CsrGraph;
use contextra_store::LsmStorage;
use contextra_types::DocId;
use contextra_vector::HnswIndex;
use std::sync::atomic::AtomicU64;
use std::sync::Arc;
use tempfile::tempdir;

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
        "tx_test".to_string(),
        storage,
        index,
        graph,
        next_tx,
        4,
        contextra_text::Language::English,
    )
}

#[tokio::test]
async fn test_db_transaction_staging_and_commit() {
    use contextra_types::{Edge, Entity, EntityId};

    let col = create_test_collection().await;
    let tx_id = col.allocate_tx().unwrap();
    let tx = DbTransaction::new(col.clone(), tx_id);

    let doc_id = DocId::new(100);
    tx.record_keys(vec![1, 2, 3], vec![3, 2, 1], doc_id);
    tx.stage_text_insert(doc_id, "hello transaction world".to_string());
    tx.stage_graph_entity(Entity::new(EntityId(1), "NodeA", "Concept"));
    tx.stage_graph_edge(Edge::new(EntityId(1), EntityId(2), "relates_to"));

    let commit_res = tx.commit().await;
    assert!(commit_res.is_ok());
}

#[tokio::test]
async fn test_db_transaction_staging_and_rollback() {
    use contextra_types::EntityId;

    let col = create_test_collection().await;
    let tx_id = col.allocate_tx().unwrap();
    let tx = DbTransaction::new(col.clone(), tx_id);

    let doc_id = DocId::new(200);
    tx.stage_text_insert(doc_id, "staged text for rollback".to_string());
    tx.stage_text_delete(doc_id);
    tx.stage_graph_entity_delete(EntityId(10));
    tx.stage_graph_edge_delete(EntityId(10), EntityId(20));

    let rollback_res = tx.rollback().await;
    assert!(rollback_res.is_ok());
}

#[tokio::test]
async fn test_db_transaction_drop_triggers_cleanup() {
    let col = create_test_collection().await;
    let tx_id = col.allocate_tx().unwrap();

    {
        let tx = DbTransaction::new(col.clone(), tx_id);
        let doc_id = DocId::new(300);
        tx.stage_text_insert(doc_id, "uncommitted text".to_string());
    }

    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    let results = col.text_index.search_bm25("uncommitted", 1, None).await;
    assert!(results.is_ok());
    assert!(results.unwrap().is_empty());
}

#[tokio::test]
#[cfg(feature = "encryption-at-rest")]
async fn test_db_transaction_rollback_cleans_kv_store_segments() {
    use contextra_crypto::kv_segment::KvSegment;
    use contextra_crypto::TenantIsolatedKvStore;

    let kv_store = Arc::new(TenantIsolatedKvStore::new());
    let tenant = contextra_types::TenantId::try_new(1).unwrap();
    let doc_id = DocId::new(500);

    kv_store.insert_segment(
        tenant,
        KvSegment::new(tenant, doc_id.inner(), vec![0xAA; 16]),
    );
    assert_eq!(kv_store.get_tenant_segment_len(tenant), 1);

    let mut col = create_test_collection().await;
    col.set_kv_store(kv_store.clone());

    let tx_id = col.allocate_tx().unwrap();
    let tx = DbTransaction::new(col, tx_id);
    tx.record_keys(vec![1], vec![2], doc_id);

    let rollback_res = tx.rollback().await;
    assert!(rollback_res.is_ok());

    assert_eq!(
        kv_store.get_tenant_segment_len(tenant),
        0,
        "Rollback must purge KV store segment"
    );
}

#[test]
fn test_commit_intent_arc_serde_kompatibel_mit_vec() {
    let doc_ids_vec = vec![
        contextra_types::DocId::new(1),
        contextra_types::DocId::new(2),
    ];
    let doc_ids_arc: Arc<Vec<contextra_types::DocId>> = Arc::new(doc_ids_vec.clone());

    let intent_arc = CommitIntent::Pending {
        doc_ids: doc_ids_arc,
        has_text: false,
        has_graph: false,
        stages_completed: 0,
    };
    let json_arc = serde_json::to_string(&intent_arc).expect("serialize Arc");

    let legacy_json =
        r#"{"Pending":{"doc_ids":[1,2],"has_text":false,"has_graph":false,"stages_completed":0}}"#;

    assert_eq!(
        json_arc, legacy_json,
        "Arc<Vec<DocId>> and Vec<DocId> must produce identical JSON representations"
    );

    let roundtrip: CommitIntent = serde_json::from_str(&json_arc).expect("deserialize json");
    match roundtrip {
        CommitIntent::Pending { doc_ids, .. } => {
            assert_eq!(doc_ids.len(), 2);
            assert_eq!(doc_ids[0], contextra_types::DocId::new(1));
            assert_eq!(doc_ids[1], contextra_types::DocId::new(2));
        }
        _ => panic!("Unexpected CommitIntent variant"),
    }
}

#[tokio::test]
async fn test_ssi_write_skew_two_transactions_second_fails() {
    use contextra_mvcc::SequenceLogSsiValidator;
    use contextra_types::ContextraError;

    let col = create_test_collection().await;
    let ssi_val = Arc::new(SequenceLogSsiValidator::new());

    let tx_id1 = col.allocate_tx().unwrap();
    let tx_id2 = col.allocate_tx().unwrap();

    let tx1 = DbTransaction::new(col.clone(), tx_id1).with_ssi_validator(ssi_val.clone());
    let tx2 = DbTransaction::new(col.clone(), tx_id2).with_ssi_validator(ssi_val.clone());

    // tx1 reads key_k at snapshot sequence 10
    tx1.record_read(b"key_k".to_vec(), 10);

    // Concurrent transaction tx2 writes key_k and commits at sequence 12
    tx2.stage_text_insert(DocId::new(99), "concurrent write".to_string());
    let res2 = tx2.commit().await;
    assert!(res2.is_ok());
    ssi_val.record_commit_key(b"key_k", 12);

    // tx1 now attempts to commit; must fail with ContextraError::Conflict due to write skew
    let res1 = tx1.commit().await;
    assert!(
        matches!(res1, Err(ContextraError::Conflict(_))),
        "tx1 must be rejected with ContextraError::Conflict due to write skew on key_k"
    );
}

#[tokio::test]
async fn test_two_tenant_commit_isolation() {
    use contextra_types::TenantId;

    let col = create_test_collection().await;

    let tenant_a = TenantId::try_new(2).unwrap();
    let tenant_b = TenantId::try_new(3).unwrap();

    let tx_id_a = col.allocate_tx().unwrap();
    let tx_id_b = col.allocate_tx().unwrap();

    let tx_a = DbTransaction::new_for_tenant(col.clone(), tx_id_a, tenant_a);
    let tx_b = DbTransaction::new_for_tenant(col.clone(), tx_id_b, tenant_b);

    assert_eq!(tx_a.tenant_id, tenant_a);
    assert_eq!(tx_b.tenant_id, tenant_b);

    let doc_id_a = DocId::new(701);
    let doc_id_b = DocId::new(702);

    tx_a.record_keys(vec![10, 1], vec![1, 10], doc_id_a);
    tx_b.record_keys(vec![20, 2], vec![2, 20], doc_id_b);

    let res_a = tx_a.commit().await;
    let res_b = tx_b.commit().await;

    assert!(res_a.is_ok());
    assert!(res_b.is_ok());
}

#[tokio::test]
async fn test_rollback_completeness_and_compensation_idempotency() {
    use super::compensating_actions::{CompensateHnswAction, CompensatingAction};
    use contextra_types::TenantId;

    let col = create_test_collection().await;
    let tenant = TenantId::try_new(4).unwrap();
    let tx_id = col.allocate_tx().unwrap();

    let tx = DbTransaction::new_for_tenant(col.clone(), tx_id, tenant);
    let doc_id = DocId::new(801);

    tx.stage_text_insert(doc_id, "idempotency test text".to_string());
    tx.record_keys(vec![30, 3], vec![3, 30], doc_id);

    // First rollback
    let rollback_res1 = tx.rollback().await;
    assert!(rollback_res1.is_ok());

    // Executing compensating action repeatedly must be idempotent and succeed
    let comp_action = CompensateHnswAction::new(col.clone(), tenant, Arc::new(vec![doc_id]));
    let comp_res1 = comp_action.execute().await;
    assert!(comp_res1.is_ok());

    let comp_res2 = comp_action.execute().await;
    assert!(
        comp_res2.is_ok(),
        "Second compensating execution must be idempotent"
    );
}
