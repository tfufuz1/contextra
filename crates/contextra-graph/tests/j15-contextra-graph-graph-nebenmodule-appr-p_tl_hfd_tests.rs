//! Integration tests for `CsrGraph::thresholded_local_hfd`.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use contextra_graph::csr::{CsrGraph, EdgeType};
use contextra_graph::tl_hfd::TlHfdParams;
use contextra_graph::{HyperEdgeId, RoleBinding, RoleId};
use contextra_ports::{Edge, GraphIndex};
use contextra_types::{Entity, EntityId, TxId};

#[tokio::test]
async fn test_csr_graph_thresholded_local_hfd_integration() {
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
    graph.commit(tx).await.unwrap();

    let roles = vec![
        RoleBinding::new(RoleId::new(1), e1),
        RoleBinding::new(RoleId::new(2), e2),
        RoleBinding::new(RoleId::new(3), e3),
    ];
    graph
        .relate_n_ary(HyperEdgeId::new(100), EdgeType::Default, roles, 1.0, None)
        .unwrap();

    let seeds = vec![e1];
    let params = TlHfdParams::default();

    let res = graph.thresholded_local_hfd(&seeds, &params).unwrap();

    assert!(!res.is_empty());
    assert!(res.contains_key(&e1));
}
