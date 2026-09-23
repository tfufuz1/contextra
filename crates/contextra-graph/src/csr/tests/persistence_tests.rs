use super::super::*;
use crate::csr::types::{CsrGraphConfig, PersistedEdgePayload};
use crate::csr::visibility::is_edge_visible;
use crate::GraphIndexExt;
use contextra_core::{DocId, Edge, Entity, EntityId, GraphIndex, ContextraError, StorageEngine, TxId};
use std::sync::Arc;

#[tokio::test]
async fn test_remove_edge_uncompacted_and_compacted() {
    let graph = CsrGraph::new();
    let tx1 = TxId::new(1);
    let id_a = EntityId::new(1);
    let id_b = EntityId::new(2);

    graph
        .add_entity(tx1, Entity::new(id_a, "A", "T"))
        .await
        .unwrap(); // unwrap
    graph
        .add_entity(tx1, Entity::new(id_b, "B", "T"))
        .await
        .unwrap(); // unwrap
    graph
        .add_edge(tx1, Edge::new(id_a, id_b, "rel"))
        .await
        .unwrap(); // unwrap
    graph.commit(tx1).await.unwrap(); // unwrap

    assert!(graph.neighbors(id_a).await.unwrap().contains(&id_b)); // unwrap

    // Remove edge in tx2
    let tx2 = TxId::new(2);
    graph.remove_edge(tx2, id_a, id_b).await.unwrap(); // unwrap
    graph.commit(tx2).await.unwrap(); // unwrap

    assert!(
        !graph.neighbors(id_a).await.unwrap().contains(&id_b), // unwrap
        "Edge A->B should not exist after remove_edge commit"
    );

    // Compact graph and verify edge remains removed
    graph.compact();
    assert!(
        !graph.neighbors(id_a).await.unwrap().contains(&id_b), // unwrap
        "Edge A->B should remain removed after compact"
    );
}

#[tokio::test]
async fn test_add_bidirectional() {
    let graph = CsrGraph::new();
    let tx = TxId::new(1);
    let id_a = EntityId::new(1);
    let id_b = EntityId::new(2);

    graph
        .add_entity(tx, Entity::new(id_a, "A", "T"))
        .await
        .unwrap(); // unwrap
    graph
        .add_entity(tx, Entity::new(id_b, "B", "T"))
        .await
        .unwrap(); // unwrap
    graph
        .add_bidirectional(tx, id_a, id_b, "knows")
        .await
        .unwrap(); // unwrap
    graph.commit(tx).await.unwrap(); // unwrap

    let n_a = graph.neighbors(id_a).await.unwrap(); // unwrap
    let n_b = graph.neighbors(id_b).await.unwrap(); // unwrap

    assert!(n_a.contains(&id_b), "neighbors(A) must contain B");
    assert!(n_b.contains(&id_a), "neighbors(B) must contain A");
}

#[tokio::test]
async fn test_pagerank_linear_chain() {
    let graph = CsrGraph::new();
    let tx = TxId::new(1);

    for i in 1..=3 {
        graph
            .add_entity(tx, Entity::new(EntityId::new(i), format!("N{i}"), "Type"))
            .await
            .unwrap(); // unwrap
    }

    // 1 -> 2 -> 3
    graph
        .add_edge(tx, Edge::new(EntityId::new(1), EntityId::new(2), "edge"))
        .await
        .unwrap(); // unwrap
    graph
        .add_edge(tx, Edge::new(EntityId::new(2), EntityId::new(3), "edge"))
        .await
        .unwrap(); // unwrap
    graph.commit(tx).await.unwrap(); // unwrap

    let ranks = graph.pagerank(0.85, 100, 1e-6).await;
    assert_eq!(ranks.len(), 3);

    let r1 = ranks[&EntityId::new(1)];
    let r2 = ranks[&EntityId::new(2)];
    let r3 = ranks[&EntityId::new(3)];

    // Downstream nodes in linear chain receive PageRank flow
    assert!(
        r2 > r1,
        "Node 2 rank ({r2}) should be higher than Node 1 ({r1})"
    );
    assert!(
        r3 > r2,
        "Node 3 rank ({r3}) should be higher than Node 2 ({r2})"
    );
}

#[tokio::test]
async fn traverse_handles_cycles_without_infinite_loop() {
    let graph = CsrGraph::new();
    let tx = TxId::new(1);
    let id_a = EntityId::from_key("node_a").expect("test: non-empty key must succeed"); // expect
    let id_b = EntityId::from_key("node_b").expect("test: non-empty key must succeed"); // expect

    graph
        .add_entity(tx, Entity::new(id_a, "Node A", "Type"))
        .await
        .unwrap(); // unwrap
    graph
        .add_entity(tx, Entity::new(id_b, "Node B", "Type"))
        .await
        .unwrap(); // unwrap

    // A -> B and B -> A cycle
    graph
        .add_edge(tx, Edge::new(id_a, id_b, "relates"))
        .await
        .unwrap(); // unwrap
    graph
        .add_edge(tx, Edge::new(id_b, id_a, "relates"))
        .await
        .unwrap(); // unwrap
    graph.commit(tx).await.unwrap(); // unwrap

    // traverse with max_hops=10 (capped by MAX_TRAVERSAL_HOPS internal logic)
    let results = graph.traverse(id_a, 10).await.unwrap(); // unwrap

    // Must return finite results without duplicates
    let ids: Vec<_> = results.iter().map(|(id, _)| *id).collect();
    let unique_ids: std::collections::HashSet<_> = ids.iter().copied().collect();
    assert_eq!(
        ids.len(),
        unique_ids.len(),
        "Results must not contain duplicates"
    );
    assert!(ids.contains(&id_b), "Must contain node B");
    assert!(!ids.contains(&id_a), "Must not contain start node A");
}

#[tokio::test]
async fn multi_traverse_keeps_highest_score_per_entity() {
    let graph = CsrGraph::new();
    let tx = TxId::new(1);
    let id_a = EntityId::from_key("node_a").expect("test: non-empty key must succeed"); // expect
    let id_b = EntityId::from_key("node_b").expect("test: non-empty key must succeed"); // expect
    let id_c = EntityId::from_key("node_c").expect("test: non-empty key must succeed"); // expect

    graph
        .add_entity(tx, Entity::new(id_a, "Node A", "Type"))
        .await
        .unwrap(); // unwrap
    graph
        .add_entity(tx, Entity::new(id_b, "Node B", "Type"))
        .await
        .unwrap(); // unwrap
    graph
        .add_entity(tx, Entity::new(id_c, "Node C", "Type"))
        .await
        .unwrap(); // unwrap

    // A -> C (weight 1.0) => hop score = 1.0 * 0.7 = 0.7
    graph
        .add_edge(tx, Edge::new(id_a, id_c, "relates").with_weight(1.0))
        .await
        .unwrap(); // unwrap
                   // B -> C (weight 0.7) => hop score = 1.0 * 0.7 * 0.7 = 0.49
    graph
        .add_edge(tx, Edge::new(id_b, id_c, "relates").with_weight(0.7))
        .await
        .unwrap(); // unwrap
    graph.commit(tx).await.unwrap(); // unwrap

    let results = graph.multi_traverse(&[id_a, id_b], 1).await.unwrap(); // unwrap
    let c_score = results.iter().find(|(id, _)| *id == id_c).map(|(_, s)| *s);

    assert!(c_score.is_some(), "Node C must be in traversal results");
    let score = c_score.unwrap(); // unwrap
    assert!(
        (score - 0.7).abs() < 1e-4,
        "Multi-traverse must keep max score 0.7, got {score}"
    );
}

#[tokio::test]
async fn test_concurrent_add_edge() {
    let graph = Arc::new(CsrGraph::new());
    let tx0 = TxId::new(1);

    // Pre-create center entity
    let center_id = EntityId::new(999);
    graph
        .add_entity(tx0, Entity::new(center_id, "Center", "Type"))
        .await
        .unwrap(); // unwrap

    for i in 1..=20 {
        graph
            .add_entity(
                tx0,
                Entity::new(EntityId::new(i), format!("Node{i}"), "Type"),
            )
            .await
            .unwrap(); // unwrap
    }
    graph.commit(tx0).await.unwrap(); // unwrap

    let mut handles = Vec::new();

    for i in 1..=20 {
        let g = graph.clone();
        let handle = tokio::spawn(async move {
            let tx = TxId::new(100 + i);
            let target = EntityId::new(i);
            GraphIndex::add_edge(g.as_ref(), tx, Edge::new(center_id, target, "connect"))
                .await
                .unwrap(); // unwrap
            g.commit(tx).await.unwrap(); // unwrap
        });
        handles.push(handle);
    }

    for h in handles {
        h.await.unwrap(); // unwrap
    }

    let neighbors = graph.neighbors(center_id).await.unwrap(); // unwrap
    assert_eq!(
        neighbors.len(),
        20,
        "All 20 concurrent edges must be committed without lost updates"
    );
}

#[tokio::test]
async fn test_staged_edges_invisible_to_concurrent_readers() {
    let graph = CsrGraph::new();
    let tx_a = TxId::new(10);
    let id_1 = EntityId::new(1);
    let id_2 = EntityId::new(2);

    // Stage entity 1 & 2, and edge 1->2 in Tx A
    graph
        .add_entity(tx_a, Entity::new(id_1, "Node 1", "Type"))
        .await
        .unwrap(); // unwrap
    graph
        .add_entity(tx_a, Entity::new(id_2, "Node 2", "Type"))
        .await
        .unwrap(); // unwrap
    graph
        .add_edge(tx_a, Edge::new(id_1, id_2, "staged_edge"))
        .await
        .unwrap(); // unwrap

    // Concurrent read (no TxId context): neighbors(1) must NOT include node 2
    let n_before = graph.neighbors(id_1).await.unwrap(); // unwrap
    assert!(
        !n_before.contains(&id_2),
        "Uncommitted staged edge must not be visible to readers"
    );

    // Tx A commits
    graph.commit(tx_a).await.unwrap(); // unwrap

    // Second read: neighbors(1) MUST include node 2
    let n_after = graph.neighbors(id_1).await.unwrap(); // unwrap
    assert!(
        n_after.contains(&id_2),
        "Committed edge must be visible to readers"
    );
}

#[tokio::test]
async fn graph_edges_survive_storage_roundtrip() {
    use contextra_store::{LsmConfig, LsmStorage};

    let dir = tempfile::tempdir().unwrap(); // unwrap allowed
    let storage = Arc::new(
        LsmStorage::new(LsmConfig {
            path: dir.path().to_path_buf(),
            ..Default::default()
        })
        .await
        .unwrap(), // unwrap allowed
    );
    let graph = CsrGraph::with_config_and_storage(CsrGraphConfig::default(), storage.clone());
    let tx = TxId::new(1);
    let id_a = EntityId::from_key("alice").unwrap(); // unwrap allowed
    let id_b = EntityId::from_key("bob").unwrap(); // unwrap allowed
    graph
        .add_entity(tx, Entity::new(id_a, "alice", "Person"))
        .await
        .unwrap(); // unwrap allowed
    graph
        .add_entity(tx, Entity::new(id_b, "bob", "Person"))
        .await
        .unwrap(); // unwrap allowed
    graph
        .add_edge(tx, Edge::new(id_a, id_b, "knows"))
        .await
        .unwrap(); // unwrap allowed
    graph.commit(tx).await.unwrap(); // unwrap allowed
    storage.commit(tx).await.unwrap(); // unwrap allowed
    storage.flush().await.unwrap(); // unwrap allowed
    drop(graph);

    let graph2 = CsrGraph::load_from_storage(storage.as_ref()).await.unwrap(); // unwrap allowed
    let neighbors = graph2.traverse(id_a, 1).await.unwrap(); // unwrap allowed
    assert!(
        !neighbors.is_empty(),
        "Kante muss storage-roundtrip überleben"
    );
    assert!(neighbors.iter().any(|(id, _)| *id == id_b));
}

#[tokio::test]
async fn test_csr_graph_traverse_at_filtering() {
    let graph = CsrGraph::new();
    let id1 = EntityId::new(1);
    let id2 = EntityId::new(2);
    let id3 = EntityId::new(3);

    // Tx 1: Add nodes 1, 2, 3 and edge 1->2
    let tx1 = TxId::new(1);
    graph
        .add_entity(tx1, Entity::new(id1, "N1", "T"))
        .await
        .unwrap(); // unwrap
    graph
        .add_entity(tx1, Entity::new(id2, "N2", "T"))
        .await
        .unwrap(); // unwrap
    graph
        .add_entity(tx1, Entity::new(id3, "N3", "T"))
        .await
        .unwrap(); // unwrap
    graph
        .add_edge(tx1, Edge::new(id1, id2, "rel1"))
        .await
        .unwrap(); // unwrap
    graph.commit(tx1).await.unwrap(); // unwrap

    // Tx 2: Add edge 2->3
    let tx2 = TxId::new(2);
    graph
        .add_edge(tx2, Edge::new(id2, id3, "rel2"))
        .await
        .unwrap(); // unwrap
    graph.commit(tx2).await.unwrap(); // unwrap

    // traverse_at seq 1: 1->2 visible, but edge 2->3 (tx2) NOT visible
    let res_seq1 = graph.traverse_at(id1, 2, 1).await.unwrap(); // unwrap
    let ids_seq1: Vec<_> = res_seq1.iter().map(|(id, _)| id.inner()).collect();
    assert!(ids_seq1.contains(&2), "seq 1 traverse must include node 2");
    assert!(
        !ids_seq1.contains(&3),
        "seq 1 traverse must NOT include node 3"
    );

    // traverse_at seq 2: both 1->2 and 2->3 visible
    let res_seq2 = graph.traverse_at(id1, 2, 2).await.unwrap(); // unwrap
    let ids_seq2: Vec<_> = res_seq2.iter().map(|(id, _)| id.inner()).collect();
    assert!(ids_seq2.contains(&2), "seq 2 traverse must include node 2");
    assert!(ids_seq2.contains(&3), "seq 2 traverse must include node 3");
}

#[tokio::test]
async fn test_last_tx_id_tracking() {
    let graph = CsrGraph::new();
    assert_eq!(graph.last_tx_id().await.unwrap(), TxId(0)); // unwrap

    let tx1 = TxId::new(5);
    graph
        .add_entity(tx1, Entity::new(EntityId::new(1), "E1", "T"))
        .await
        .unwrap(); // unwrap
    graph.commit(tx1).await.unwrap(); // unwrap

    assert_eq!(
        graph.last_tx_id().await.unwrap(), // unwrap
        TxId(5),
        "last_tx_id should be updated to 5 after committing Tx 5"
    );

    let tx2 = TxId::new(12);
    graph
        .add_entity(tx2, Entity::new(EntityId::new(2), "E2", "T"))
        .await
        .unwrap(); // unwrap
    graph.commit(tx2).await.unwrap(); // unwrap

    assert_eq!(
        graph.last_tx_id().await.unwrap(), // unwrap
        TxId(12),
        "last_tx_id should be updated to 12 after committing Tx 12"
    );
}

#[tokio::test]
async fn test_traverse_at_time_exact_boundary_off_by_one() {
    let graph = CsrGraph::new();
    let tx_setup = TxId::new(1);
    let id1 = EntityId::new(1);
    let id2 = EntityId::new(2);

    graph
        .add_entity(tx_setup, Entity::new(id1, "Node1", "Type"))
        .await
        .unwrap(); // unwrap
    graph
        .add_entity(tx_setup, Entity::new(id2, "Node2", "Type"))
        .await
        .unwrap(); // unwrap

    let valid_until = TxId::new(100);
    let edge =
        Edge::new(id1, id2, "valid_rel").with_validity(Some(TxId::new(10)), Some(valid_until));

    graph.add_edge(tx_setup, edge).await.unwrap(); // unwrap
    graph.commit(tx_setup).await.unwrap(); // unwrap

    // 1. Before valid_from (< 10) -> Should NOT return edge
    let res_before = graph.traverse_at_time(id1, 1, TxId::new(9)).await.unwrap(); // unwrap
    assert!(
        res_before.is_empty(),
        "Edge must not be valid before valid_from (9 < 10)"
    );

    // 2. Exactly at valid_from (10) -> MUST return edge
    let res_from = graph.traverse_at_time(id1, 1, TxId::new(10)).await.unwrap(); // unwrap
    assert_eq!(res_from.len(), 1, "Edge must be valid at valid_from (10)");

    // 3. One step before valid_to (valid_until - 1 = 99) -> MUST return edge
    let res_before_to = graph.traverse_at_time(id1, 1, TxId::new(99)).await.unwrap(); // unwrap
    assert_eq!(
        res_before_to.len(),
        1,
        "Edge must be valid at valid_to - 1 (99)"
    );

    // 4. Exactly at valid_to (valid_until = 100) -> MUST NOT return edge
    let res_at_to = graph.traverse_at_time(id1, 1, valid_until).await.unwrap(); // unwrap
    assert!(
        res_at_to.is_empty(),
        "Edge must NOT be valid at exact valid_to boundary (100)"
    );

    // 5. After valid_to (101) -> MUST NOT return edge
    let res_after_to = graph
        .traverse_at_time(id1, 1, TxId::new(101))
        .await
        .unwrap(); // unwrap
    assert!(
        res_after_to.is_empty(),
        "Edge must NOT be valid after valid_to (101)"
    );

    // 6. Test compacted CSR path boundary behavior
    graph.compact();

    let res_compact_valid = graph.traverse_at_time(id1, 1, TxId::new(99)).await.unwrap(); // unwrap
    assert_eq!(
        res_compact_valid.len(),
        1,
        "Compacted edge must be valid at 99"
    );

    let res_compact_invalid = graph.traverse_at_time(id1, 1, valid_until).await.unwrap(); // unwrap
    assert!(
        res_compact_invalid.is_empty(),
        "Compacted edge must NOT be valid at 100"
    );
}

#[tokio::test]
async fn test_traverse_at_time_unbounded_validity() {
    let graph = CsrGraph::new();
    let tx = TxId::new(1);
    let id1 = EntityId::new(1);
    let id2 = EntityId::new(2);

    graph
        .add_entity(tx, Entity::new(id1, "N1", "T"))
        .await
        .unwrap(); // unwrap
    graph
        .add_entity(tx, Entity::new(id2, "N2", "T"))
        .await
        .unwrap(); // unwrap

    let edge = Edge::new(id1, id2, "always_valid");
    graph.add_edge(tx, edge).await.unwrap(); // unwrap
    graph.commit(tx).await.unwrap(); // unwrap

    let res = graph
        .traverse_at_time(id1, 1, TxId::new(500))
        .await
        .unwrap(); // unwrap
    assert_eq!(
        res.len(),
        1,
        "Unbounded edge must be valid at any point in time"
    );
}

proptest::proptest! {
    #[test]
    fn prop_add_edge_rollback_no_index_growth(
        edge_specs in proptest::collection::vec((1u64..1000, 1001u64..2000), 10..100)
    ) {
        let rt = tokio::runtime::Builder::new_current_thread().build().unwrap(); // unwrap
        let res: std::result::Result<(), proptest::test_runner::TestCaseError> = rt.block_on(async {
            let graph = CsrGraph::new();
            let initial_id_len = graph.inner_read().id_map.len();
            let initial_rev_len = graph.inner_read().reverse_map.len();

            for (i, (from_val, to_val)) in edge_specs.into_iter().enumerate() {
                let tx = TxId::new(i as u64 + 1);
                let edge = Edge::new(EntityId::new(from_val), EntityId::new(to_val), "rel");
                let _ = graph.add_edge(tx, edge).await;
                let _ = graph.rollback(tx).await;

                let inner = graph.inner_read();
                proptest::prop_assert_eq!(inner.id_map.len(), initial_id_len);
                proptest::prop_assert_eq!(inner.reverse_map.len(), initial_rev_len);
            }
            Ok(())
        });
        res?;
    }

    #[test]
    fn prop_edge_visible_monotone(
        vf in 0u64..100,
        vt in 100u64..200,
        as_of in 0u64..200,
    ) {
        let valid_from = Some(TxId::new(vf));
        let valid_to = Some(TxId::new(vt));
        let visible = is_edge_visible(valid_from, valid_to, TxId::new(as_of));
        let expected = vf <= as_of && as_of < vt;
        proptest::prop_assert_eq!(visible, expected);
    }

    #[test]
    fn prop_traverse_at_time_never_panics(
        node_count in 1..=15usize,
        edge_specs in proptest::collection::vec((0..15usize, 0..15usize, 0u64..50u64, 50u64..100u64), 0..30),
        start_idx in 0..15usize,
        hops in 0usize..5,
        as_of in 0u64..150u64,
    ) {
        let rt = tokio::runtime::Builder::new_current_thread().build().unwrap(); // unwrap
        let res: std::result::Result<(), proptest::test_runner::TestCaseError> = rt.block_on(async {
            let graph = CsrGraph::new();
            let tx = TxId::new(1);

            for i in 0..node_count {
                let _ = graph
                    .add_entity(tx, Entity::new(EntityId::new(i as u64 + 1), format!("N{i}"), "Node"))
                    .await;
            }

            for (src, dst, vf, vt) in edge_specs {
                let src_id = EntityId::new((src % node_count) as u64 + 1);
                let dst_id = EntityId::new((dst % node_count) as u64 + 1);
                let edge = Edge::new(src_id, dst_id, "link")
                    .with_validity(Some(TxId::new(vf)), Some(TxId::new(vt)));
                let _ = graph.add_edge(tx, edge).await;
            }
            let _ = graph.commit(tx).await;

            let start = EntityId::new((start_idx % node_count) as u64 + 1);
            let _res = graph.traverse_at_time(start, hops, TxId::new(as_of)).await;
            Ok(())
        });
        res?;
    }
}

#[test]
fn prop_csr_graph_traverse_at_consistency() {
    use proptest::prelude::*;

    #[derive(Debug, Clone)]
    enum Op {
        AddEntity(u64),
        AddEdge(u64, u64),
        RemoveEdge(u64, u64),
    }

    let op_strategy = proptest::collection::vec(
        prop_oneof![
            (1u64..20).prop_map(Op::AddEntity),
            (1u64..20, 1u64..20).prop_map(|(f, t)| Op::AddEdge(f, t)),
            (1u64..20, 1u64..20).prop_map(|(f, t)| Op::RemoveEdge(f, t)),
        ],
        10..80,
    );

    proptest!(ProptestConfig::with_cases(20), |(ops in op_strategy)| {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap(); // unwrap

        rt.block_on(async {
            let graph = CsrGraph::new();
            let mut current_tx = 1u64;
            let mut tx_checkpoints = Vec::new();

            for op in ops {
                let tx = TxId::new(current_tx);
                match op {
                    Op::AddEntity(id) => {
                        let _ = graph.add_entity(tx, Entity::new(EntityId::new(id), "N", "T")).await;
                    }
                    Op::AddEdge(from, to) => {
                        if from != to {
                            let _ = graph.add_edge(tx, Edge::new(EntityId::new(from), EntityId::new(to), "E")).await;
                        }
                    }
                    Op::RemoveEdge(from, to) => {
                        let _ = graph.remove_edge(tx, EntityId::new(from), EntityId::new(to)).await;
                    }
                }
                if graph.commit(tx).await.is_ok() {
                    tx_checkpoints.push(current_tx);
                    current_tx += 1;
                }
            }

            // Verify traverse_at at each target sequence against reference replay model
            for &target_seq in &tx_checkpoints {
                // Reconstruct expected edges and entities up to target_seq
                let (entities, active_edges) = {
                    let mut entities = std::collections::HashSet::new();
                    let mut active_edges = std::collections::HashSet::new();

                    let inner = graph.inner_read();
                    let num_nodes = inner.reverse_map.len();
                    for i in 0..num_nodes {
                        if let Some(&id) = inner.reverse_map.get(i) {
                            if inner.entity_at(i).is_some() {
                                entities.insert(id.inner());
                            }
                        }

                        let old_start = if i < inner.offsets.len() - 1 { inner.offsets[i] } else { 0 };
                        let old_end = if i < inner.offsets.len() - 1 { inner.offsets[i + 1] } else { 0 };
                        for j in old_start..old_end {
                            let target_idx = inner.targets[j];
                            let vf = inner.tx_valid_from_at(j).unwrap_or(TxId::new(0)).inner();
                            let vt = inner.tx_valid_to_at(j).map(|t| t.inner());

                            if vf <= target_seq && vt.is_none_or(|t| target_seq < t) {
                                if let (Some(&f), Some(&t)) = (inner.reverse_map.get(i), inner.reverse_map.get(target_idx)) {
                                    if !inner.tombstoned_edges.contains(&(i, target_idx)) {
                                        active_edges.insert((f.inner(), t.inner()));
                                    }
                                }
                            }
                        }

                        if let Some(pending) = inner.pending_edges.get(&i) {
                            for edge in pending {
                                let vf = edge.tx_valid_from.unwrap_or(TxId::new(0)).inner();
                                let vt = edge.tx_valid_to.map(|t| t.inner());
                                if vf <= target_seq && vt.is_none_or(|t| target_seq < t) {
                                    if let (Some(&f), Some(&t)) = (inner.reverse_map.get(i), inner.reverse_map.get(edge.target)) {
                                        if !inner.tombstoned_edges.contains(&(i, edge.target)) {
                                            active_edges.insert((f.inner(), t.inner()));
                                        }
                                    }
                                }
                            }
                        }
                    }
                    (entities, active_edges)
                };

                // Check traverse_at for each active node
                for &start in &entities {
                    let res = graph.traverse_at(EntityId::new(start), 1, target_seq).await.unwrap(); // unwrap
                    let actual_neighbors: std::collections::HashSet<_> = res.into_iter().map(|(id, _)| id.inner()).collect();

                    let expected_neighbors: std::collections::HashSet<_> = active_edges
                        .iter()
                        .filter(|(f, t)| *f == start && entities.contains(t))
                        .map(|(_, t)| *t)
                        .collect();

                    prop_assert_eq!(actual_neighbors, expected_neighbors, "Neighbors at seq {} from node {} must match reference model", target_seq, start);
                }
            }
            Ok(())
        }).unwrap(); // unwrap
    });
}

#[tokio::test]
#[allow(non_snake_case)]
async fn compact_async_CASE_dirty_and_clean_states() {
    let graph = Arc::new(CsrGraph::new());
    let tx = TxId::new(1);

    // Stage and commit an edge to mark graph dirty
    graph
        .add_entity(tx, Entity::new(EntityId::new(1), "A", "T"))
        .await
        .unwrap(); // unwrap allowed
    graph
        .add_entity(tx, Entity::new(EntityId::new(2), "B", "T"))
        .await
        .unwrap(); // unwrap allowed
    GraphIndex::add_edge(
        graph.as_ref(),
        tx,
        Edge::new(EntityId::new(1), EntityId::new(2), "rel"),
    )
    .await
    .unwrap(); // unwrap allowed
    graph.commit(tx).await.unwrap(); // unwrap allowed

    assert!(graph.inner_read().is_dirty);

    // compact_async on dirty graph
    graph.compact_async().await.unwrap(); // unwrap allowed

    assert!(!graph.inner_read().is_dirty);
    assert_eq!(graph.inner_read().targets.len(), 1);

    // compact_async no-op on clean graph
    graph.compact_async().await.unwrap(); // unwrap allowed
    assert!(!graph.inner_read().is_dirty);
}
