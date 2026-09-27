use contextra_engine::{Contextra, ContextraConfig};
use contextra_types::{DocId, Edge, Entity, EntityId};
use serde_json::json;
use tempfile::TempDir;

#[tokio::test]
async fn test_db_transaction_multi_index_staging_commit_and_rollback() -> contextra_types::Result<()> {
    let tmp = TempDir::new().expect("Failed to create temporary directory");
    let config = ContextraConfig {
        dimension: 4,
        consolidation_enabled: false,
        ..Default::default()
    };

    let db = Contextra::open_with_config(tmp.path(), config).await?;
    let col = db.collection("tx_multi_index_col").await?;

    // 1. Commit Scenario: Staging document insert, graph entity, and graph edge, then committing
    let mut tx_commit = col.begin_transaction()?;

    col.insert_op(
        &mut tx_commit,
        "tx_doc_commit",
        &[0.1, 0.2, 0.3, 0.4],
        Some(json!({ "text": "Transaktions-Text für Commit-Test" })),
    )
    .await?;

    let doc_id_committed = DocId::from_key("tx_doc_commit")?;
    tx_commit.stage_text_insert(doc_id_committed, "Transaktions-Text für Commit-Test".to_string());

    let entity1 = Entity::new(EntityId::from_key("tx_entity_1")?, "Node1", "Concept");
    let entity2 = Entity::new(EntityId::from_key("tx_entity_2")?, "Node2", "Concept");
    tx_commit.stage_graph_entity(entity1);
    tx_commit.stage_graph_entity(entity2);

    let edge = Edge::new(
        EntityId::from_key("tx_entity_1")?,
        EntityId::from_key("tx_entity_2")?,
        "CONNECTS_TO",
    );
    tx_commit.stage_graph_edge(edge);

    // Execute commit
    tx_commit.commit().await?;

    // Verify committed document in storage
    let doc_comm = col.get("tx_doc_commit").await?;
    assert!(
        doc_comm.is_some(),
        "Committed transaction document must exist in storage"
    );

    // Verify graph index contains committed edge
    let node1_id = EntityId::from_key("tx_entity_1")?;
    let node2_id = EntityId::from_key("tx_entity_2")?;
    let neighbors = col.graph_index().neighbors(node1_id).await?;
    assert!(
        neighbors.contains(&node2_id),
        "Graph index must contain committed edge from tx_entity_1 to tx_entity_2"
    );

    // 2. Rollback Scenario: Staging text and graph entities, then rolling back
    let mut tx_rollback = col.begin_transaction()?;

    col.insert_op(
        &mut tx_rollback,
        "tx_doc_rollback",
        &[0.5, 0.6, 0.7, 0.8],
        Some(json!({ "text": "Transaktions-Text für Rollback-Test" })),
    )
    .await?;

    let doc_id_rolled_back = DocId::from_key("tx_doc_rollback")?;
    tx_rollback.stage_text_insert(
        doc_id_rolled_back,
        "Transaktions-Text für Rollback-Test".to_string(),
    );

    let entity_rb1 = Entity::new(EntityId::from_key("tx_rb_1")?, "RB1", "Concept");
    let entity_rb2 = Entity::new(EntityId::from_key("tx_rb_2")?, "RB2", "Concept");
    tx_rollback.stage_graph_entity(entity_rb1);
    tx_rollback.stage_graph_entity(entity_rb2);

    let edge_rb = Edge::new(
        EntityId::from_key("tx_rb_1")?,
        EntityId::from_key("tx_rb_2")?,
        "TEMP_EDGE",
    );
    tx_rollback.stage_graph_edge(edge_rb);

    // Execute rollback
    tx_rollback.rollback().await?;

    // Verify document DOES NOT exist in storage after rollback
    let rb_doc = col.get("tx_doc_rollback").await?;
    assert!(
        rb_doc.is_none(),
        "Storage must NOT contain rolled-back transaction document"
    );

    // Verify graph index DOES NOT contain rolled-back edge
    let rb1_id = EntityId::from_key("tx_rb_1")?;
    let rb2_id = EntityId::from_key("tx_rb_2")?;
    let rb_neighbors = col.graph_index().neighbors(rb1_id).await?;
    assert!(
        !rb_neighbors.contains(&rb2_id),
        "Graph index must NOT contain rolled-back edge"
    );

    db.close().await?;
    Ok(())
}
