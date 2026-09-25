#![allow(deprecated)]

use contextra_core::{Edge, Entity, EntityId, GraphIndex, TxId};
use contextra_db::{Contextra, ContextraConfig, DistanceMetric};
use serde_json::json;
use tempfile::TempDir;

#[tokio::test]
#[allow(deprecated)]
async fn test_hybrid_search_includes_graph_signal() {
    let tmp = TempDir::new().expect("temp dir");
    let config = ContextraConfig {
        dimension: 4,
        distance_metric: DistanceMetric::Cosine,
        ..Default::default()
    };

    let db = Contextra::open_with_config(tmp.path(), config)
        .await
        .expect("open db");
    let col = db.collection("graph-test").await.expect("col");

    // 1. Insert documents into collection
    col.insert(
        "anchor_doc",
        &[1.0, 0.0, 0.0, 0.0],
        Some(json!({"text": "anchor document about quantum physics"})),
    )
    .await
    .expect("insert anchor_doc");

    col.insert(
        "target_doc",
        &[0.0, 0.0, 0.0, 1.0], // Orthogonal vector
        Some(json!({"text": "unrelated topic description xyz"})), // No text match
    )
    .await
    .expect("insert target_doc");

    // 2. Setup graph relationship between anchor_doc and target_doc
    let graph = col.graph_index();
    let tx = TxId::new(100);

    let anchor_eid = EntityId::from("anchor_doc");
    let target_eid = EntityId::from("target_doc");

    graph
        .add_entity(tx, Entity::new(anchor_eid, "anchor_doc", "Document"))
        .await
        .expect("add anchor entity");
    graph
        .add_entity(tx, Entity::new(target_eid, "target_doc", "Document"))
        .await
        .expect("add target entity");

    GraphIndex::add_edge(
        &*graph,
        tx,
        Edge::new(anchor_eid, target_eid, "references").with_weight(1.0),
    )
    .await
    .expect("add edge");

    graph.commit(tx).await.expect("commit graph tx");

    // 3. Perform hybrid_search with anchor_doc as anchor_entities
    let results = col
        .hybrid_search(
            "nonmatchingquerytext",
            &[0.0, 1.0, 0.0, 0.0],
            10,
            Some(&[anchor_eid]),
        )
        .await
        .expect("hybrid_search with anchors");

    // 4. Verify target_doc is present in hybrid search results due to the graph signal
    assert!(
        results.iter().any(|r| r.id == "target_doc"),
        "target_doc should be included in hybrid search results via graph signal"
    );
}

#[tokio::test]
#[allow(deprecated)]
async fn test_relate_updates_graph_index_and_affects_hybrid_search() {
    let tmp = TempDir::new().expect("temp dir");
    let config = ContextraConfig {
        dimension: 4,
        distance_metric: DistanceMetric::Cosine,
        ..Default::default()
    };

    let db = Contextra::open_with_config(tmp.path(), config)
        .await
        .expect("open db");
    let col = db.collection("relate-graph-test").await.expect("col");

    // 1. Insert documents
    col.insert(
        "doc_a",
        &[1.0, 0.0, 0.0, 0.0],
        Some(json!({"text": "document A"})),
    )
    .await
    .expect("insert doc_a");

    col.insert(
        "doc_b",
        &[0.0, 0.0, 0.0, 1.0], // Orthogonal vector
        Some(json!({"text": "unrelated content B"})),
    )
    .await
    .expect("insert doc_b");

    // 2. Call relate() via public API
    col.relate("doc_a", "doc_b", "references")
        .await
        .expect("relate doc_a -> doc_b");

    // 3. Perform hybrid_search with doc_a as anchor entity
    let anchor_eid =
        contextra_core::EntityId::from_key("doc_a").expect("test: non-empty key must succeed");
    let results = col
        .hybrid_search(
            "nonmatchingtext",
            &[0.0, 1.0, 0.0, 0.0],
            10,
            Some(&[anchor_eid]),
        )
        .await
        .expect("hybrid_search with anchor doc_a");

    // 4. Verify doc_b is returned via graph signal created by relate()
    assert!(
        results.iter().any(|r| r.id == "doc_b"),
        "doc_b should be included in hybrid search results via graph signal created by relate()"
    );
}

#[tokio::test]
#[allow(deprecated)]
async fn test_hybrid_search_with_ppr_strategy() {
    use contextra_core::{GraphTraversalStrategy, PprConfig};

    let tmp = TempDir::new().expect("temp dir");
    let config = ContextraConfig {
        dimension: 4,
        distance_metric: DistanceMetric::Cosine,
        ..Default::default()
    };

    let db = Contextra::open_with_config(tmp.path(), config)
        .await
        .expect("open db");
    let col = db.collection("ppr-hybrid-test").await.expect("col");

    // Insert chain of documents: doc_a -> doc_b -> doc_c
    col.insert(
        "doc_a",
        &[1.0, 0.0, 0.0, 0.0],
        Some(json!({"text": "document A"})),
    )
    .await
    .expect("insert doc_a");
    col.insert(
        "doc_b",
        &[0.0, 0.0, 0.0, 1.0],
        Some(json!({"text": "unrelated content B"})),
    )
    .await
    .expect("insert doc_b");
    col.insert(
        "doc_c",
        &[0.0, 0.0, 1.0, 0.0],
        Some(json!({"text": "unrelated content C"})),
    )
    .await
    .expect("insert doc_c");

    col.relate("doc_a", "doc_b", "rel")
        .await
        .expect("relate a->b");
    col.relate("doc_b", "doc_c", "rel")
        .await
        .expect("relate b->c");

    let anchor_eid = EntityId::from_key("doc_a").expect("anchor doc_a");

    let ppr_strategy = GraphTraversalStrategy::PersonalizedPageRank(PprConfig {
        damping_factor: 0.85,
        max_iterations: 100,
        convergence_epsilon: 1e-6,
        algorithm: contextra_core::PprAlgorithm::Auto,
        warn_on_non_convergence: true,
    });

    let res = col
        .hybrid_search_with_strategy(
            "nonmatchingtext",
            &[0.0, 1.0, 0.0, 0.0],
            10,
            Some(&[anchor_eid]),
            None,
            Some(&ppr_strategy),
            None,
        )
        .await
        .expect("PPR under snapshot isolation must succeed");

    assert!(
        res.iter().any(|r| r.id == "doc_b"),
        "doc_b should be included in PPR hybrid search results"
    );
}

#[tokio::test]
async fn test_hybrid_search_with_pathrag_strategy() {
    use contextra_core::GraphTraversalStrategy;
    use contextra_db::SearchStrategy;

    let tmp = TempDir::new().expect("temp dir");
    let config = ContextraConfig {
        dimension: 4,
        distance_metric: DistanceMetric::Cosine,
        ..Default::default()
    };

    let db = Contextra::open_with_config(tmp.path(), config)
        .await
        .expect("open db");
    let col = db.collection("pathrag-hybrid-test").await.expect("col");

    // Insert chain of documents: doc_a -> doc_b -> doc_c
    col.insert(
        "doc_a",
        &[1.0, 0.0, 0.0, 0.0],
        Some(json!({"text": "document A"})),
    )
    .await
    .expect("insert doc_a");
    col.insert(
        "doc_b",
        &[0.0, 0.0, 0.0, 1.0],
        Some(json!({"text": "unrelated content B"})),
    )
    .await
    .expect("insert doc_b");
    col.insert(
        "doc_c",
        &[0.0, 0.0, 1.0, 0.0],
        Some(json!({"text": "unrelated content C"})),
    )
    .await
    .expect("insert doc_c");

    col.relate("doc_a", "doc_b", "rel")
        .await
        .expect("relate a->b");
    col.relate("doc_b", "doc_c", "rel")
        .await
        .expect("relate b->c");

    let anchor_eid = EntityId::from_key("doc_a").expect("anchor doc_a");

    let pathrag_strategy = GraphTraversalStrategy::PathRag {
        max_hops: 3,
        sufficiency_threshold: 0.1,
    };

    let res = col
        .hybrid_search_with_strategy(
            "nonmatchingtext",
            &[0.0, 1.0, 0.0, 0.0],
            10,
            Some(&[anchor_eid]),
            None,
            Some(&pathrag_strategy),
            None,
        )
        .await;

    assert!(
        res.is_err(),
        "PathRag under snapshot isolation must fail-closed with error"
    );
    match res.unwrap_err() {
        contextra_core::ContextraError::SnapshotUnsupportedForSignal(msg) => {
            assert!(msg.contains("PathRag"));
        }
        other => panic!(
            "Expected SnapshotUnsupportedForSignal error, got: {:?}",
            other
        ),
    }

    // Test query builder API with SearchStrategy::PathRag
    let builder_res = col
        .query()
        .anchors(vec![anchor_eid])
        .strategy(SearchStrategy::PathRag {
            max_hops: 3,
            sufficiency_threshold: 0.1,
        })
        .fusion_weights(contextra_core::FusionWeights::new(0.33, 0.33, 0.34).expect("weights"))
        .k(10)
        .execute()
        .await;

    assert!(
        builder_res.is_err(),
        "QueryBuilder PathRag under snapshot isolation must fail-closed with error"
    );
    match builder_res.unwrap_err() {
        contextra_core::ContextraError::SnapshotUnsupportedForSignal(msg) => {
            assert!(msg.contains("PathRag"));
        }
        other => panic!(
            "Expected SnapshotUnsupportedForSignal error, got: {:?}",
            other
        ),
    }
}
