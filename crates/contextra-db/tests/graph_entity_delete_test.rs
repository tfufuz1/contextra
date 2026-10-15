use contextra_db::transaction::DbTransaction;
use contextra_db::Collection;
use contextra_store::{LsmConfig, LsmStorage};
use contextra_types::{Edge, Entity, EntityId};
use contextra_vector::HnswIndex;
use std::sync::atomic::AtomicU64;
use std::sync::Arc;
use tempfile::tempdir;

type TestTx = DbTransaction<LsmStorage, HnswIndex>;

async fn create_test_collection(
    dir: &tempfile::TempDir,
) -> (Collection<LsmStorage, HnswIndex>, Arc<LsmStorage>) {
    let lsm_config = LsmConfig {
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
    let next_tx = Arc::new(AtomicU64::new(1));

    let col = Collection::new_without_graph(
        "graph_delete_test".to_string(),
        storage.clone(),
        index,
        next_tx,
        4,
        contextra_text::Language::English,
    );
    (col, storage)
}

#[tokio::test]
async fn test_delete_entity_staging_and_commit() {
    let dir = tempdir().unwrap();
    let (col, _) = create_test_collection(&dir).await;

    let tx_id1 = col.allocate_tx().unwrap();
    let tx1: TestTx = DbTransaction::new(col.clone(), tx_id1);

    let id_a = EntityId::new(1);
    let id_b = EntityId::new(2);
    let id_c = EntityId::new(3);

    tx1.stage_graph_entity(Entity::new(id_a, "EntityA", "Concept"));
    tx1.stage_graph_entity(Entity::new(id_b, "EntityB", "Concept"));
    tx1.stage_graph_entity(Entity::new(id_c, "EntityC", "Concept"));

    tx1.stage_graph_edge(Edge::new(id_a, id_b, "relates_to"));
    tx1.stage_graph_edge(Edge::new(id_a, id_c, "relates_to"));

    tx1.commit().await.unwrap();

    // Delete Entity A
    let tx_id2 = col.allocate_tx().unwrap();
    let tx2: TestTx = DbTransaction::new(col.clone(), tx_id2);
    tx2.stage_graph_entity_delete(id_a);
    tx2.commit().await.unwrap();
}

#[tokio::test]
async fn test_delete_incoming_entity_staging_and_commit() {
    let dir = tempdir().unwrap();
    let (col, _) = create_test_collection(&dir).await;

    let tx_id1 = col.allocate_tx().unwrap();
    let tx1: TestTx = DbTransaction::new(col.clone(), tx_id1);

    let id_a = EntityId::new(1);
    let id_b = EntityId::new(2);

    tx1.stage_graph_entity(Entity::new(id_a, "EntityA", "Concept"));
    tx1.stage_graph_entity(Entity::new(id_b, "EntityB", "Concept"));
    tx1.stage_graph_edge(Edge::new(id_a, id_b, "points_to"));

    tx1.commit().await.unwrap();

    // Delete Entity B
    let tx_id2 = col.allocate_tx().unwrap();
    let tx2: TestTx = DbTransaction::new(col.clone(), tx_id2);
    tx2.stage_graph_entity_delete(id_b);
    tx2.commit().await.unwrap();
}
