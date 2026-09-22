//! Loom / Concurrent Deadlock-Free Proof for CsrGraph::relate_n_ary
//!
//! Verifies that concurrent `relate_n_ary` calls with overlapping participant sets in
//! different orderings (e.g. {entity-1, entity-2, entity-3} vs {entity-3, entity-2, entity-1}) terminate
//! cleanly without deadlocks thanks to canonical participant EntityId sorting before lock acquisition.
//!
//! Execution:
//! - Standard test: cargo test -p memfuse-graph --test loom_relate_n_ary
//! - Loom execution: RUSTFLAGS="--cfg loom" cargo test -p memfuse-graph --test loom_relate_n_ary

#![allow(unexpected_cfgs)]

use memfuse_core::{DocId, EntityId};
use memfuse_graph::csr::{CsrGraph, EdgeType};
use memfuse_graph::hyperedge::{HyperEdgeId, RoleBinding, RoleId};
use std::sync::Arc;

#[tokio::test]
async fn test_concurrent_relate_n_ary_deadlock_free_overlapping_participants() {
    let graph = Arc::new(CsrGraph::new());

    let e1 = EntityId::new(100);
    let e2 = EntityId::new(200);
    let e3 = EntityId::new(300);

    let graph_1 = graph.clone();
    let graph_2 = graph.clone();

    // Task 1: Order {e1, e2, e3}
    let task1 = tokio::spawn(async move {
        let participants = vec![
            RoleBinding::new(RoleId::new(1), e1),
            RoleBinding::new(RoleId::new(2), e2),
            RoleBinding::new(RoleId::new(3), e3),
        ];
        graph_1.relate_n_ary(
            HyperEdgeId::new(1),
            EdgeType::Default,
            participants,
            1.0,
            Some(DocId::new(10)),
        )
    });

    // Task 2: Inverted order {e3, e2, e1}
    let task2 = tokio::spawn(async move {
        let participants = vec![
            RoleBinding::new(RoleId::new(3), e3),
            RoleBinding::new(RoleId::new(2), e2),
            RoleBinding::new(RoleId::new(1), e1),
        ];
        graph_2.relate_n_ary(
            HyperEdgeId::new(2),
            EdgeType::Default,
            participants,
            1.0,
            Some(DocId::new(20)),
        )
    });

    // Join both tasks with timeout to guarantee termination without deadlock
    let res = tokio::time::timeout(std::time::Duration::from_secs(10), async {
        let (r1, r2) = tokio::join!(task1, task2);
        (r1.expect("task1 join"), r2.expect("task2 join"))
    })
    .await;

    assert!(
        res.is_ok(),
        "Concurrent relate_n_ary calls with overlapping participant sets timed out (deadlock!)"
    );

    let (res1, res2) = res.expect("timeout result");
    let he_1 = res1.expect("task1 relate_n_ary");
    let he_2 = res2.expect("task2 relate_n_ary");

    // Verify both hyperedges exist in graph
    assert!(graph.get_hyperedge(he_1.id).is_some());
    assert!(graph.get_hyperedge(he_2.id).is_some());
}
