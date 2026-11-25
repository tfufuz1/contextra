//! Integration tests for `PprCandidateStream::with_exclude_seeds`.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use contextra_graph::csr::CsrGraph;
use contextra_graph::PprCandidateStream;
use contextra_ports::{Edge, GraphIndex};
use contextra_types::{Entity, EntityId, PprConfig, TxId};

#[tokio::test]
async fn test_ppr_candidate_stream_with_exclude_seeds_integration() {
    let graph = CsrGraph::new();
    let tx = TxId::new(1);

    let e1 = EntityId::new(1);
    let e2 = EntityId::new(2);
    let e3 = EntityId::new(3);

    graph
        .add_entity(tx, Entity::new(e1, "E1", "Node"))
        .await
        .unwrap();
    graph
        .add_entity(tx, Entity::new(e2, "E2", "Node"))
        .await
        .unwrap();
    graph
        .add_entity(tx, Entity::new(e3, "E3", "Node"))
        .await
        .unwrap();

    graph
        .add_edge(tx, Edge::new(e1, e2, "rel").with_weight(1.0))
        .await
        .unwrap();
    graph
        .add_edge(tx, Edge::new(e2, e3, "rel").with_weight(1.0))
        .await
        .unwrap();
    graph.commit(tx).await.unwrap();

    let seeds = vec![e1];
    let config = PprConfig::default();

    let mut stream = PprCandidateStream::new(&graph, seeds, config)
        .with_batch_size(10)
        .with_exclude_seeds(true);

    let batch = stream.next_batch().await.unwrap();

    assert!(!batch.is_empty());
    assert!(!batch.iter().any(|(eid, _)| *eid == e1));
    assert!(batch.iter().any(|(eid, _)| *eid == e2));
}
