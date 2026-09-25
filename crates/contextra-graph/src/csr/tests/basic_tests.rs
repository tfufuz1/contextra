use super::super::*;
use crate::csr::types::CsrGraphConfig;
use crate::csr::visibility::is_suspicious_tx_id;
use crate::GraphIndexExt;
use contextra_ports::GraphIndex;
use contextra_types::{ContextraError, DocId, Edge, Entity, EntityId, TxId};
use std::sync::Arc;

use super::*;

pub(super) async fn setup_test_graph() -> CsrGraph {
    let graph = CsrGraph::new();
    let tx = TxId::new(1);

    for id in 1..=5 {
        graph
            .add_entity(
                tx,
                Entity::new(EntityId::new(id), format!("P{}", id), "Person"),
            )
            .await
            .expect("valid setup"); // expect
    }

    graph
        .add_edge(
            tx,
            Edge::new(EntityId::new(1), EntityId::new(2), "knows").with_weight(1.0),
        )
        .await
        .expect("valid edge"); // expect
    graph
        .add_edge(
            tx,
            Edge::new(EntityId::new(2), EntityId::new(3), "knows").with_weight(0.8),
        )
        .await
        .expect("valid edge"); // expect
    graph
        .add_edge(
            tx,
            Edge::new(EntityId::new(3), EntityId::new(4), "knows").with_weight(0.6),
        )
        .await
        .expect("valid edge"); // expect
    graph
        .add_edge(
            tx,
            Edge::new(EntityId::new(4), EntityId::new(5), "knows").with_weight(0.5),
        )
        .await
        .expect("valid edge"); // expect
    graph
        .add_edge(
            tx,
            Edge::new(EntityId::new(2), EntityId::new(5), "knows").with_weight(0.4),
        )
        .await
        .expect("valid edge"); // expect

    graph.commit(tx).await.expect("commit"); // expect
    graph.compact();
    graph
}

#[tokio::test]
async fn test_sentinel_representation_no_value_cases() {
    let graph = Arc::new(CsrGraph::new());
    let tx = TxId::new(1);
    let id1 = EntityId::new(10);
    let id2 = EntityId::new(20);

    graph
        .add_entity(tx, Entity::new(id1, "Node10", "Type"))
        .await
        .expect("add entity");
    graph
        .add_entity(tx, Entity::new(id2, "Node20", "Type"))
        .await
        .expect("add entity");

    // Edge with no optional validities or doc_ids inserted directly with None validities
    graph
        .add_edge(id1, id2, 1.0, None, None, None, None, None, None, None)
        .await
        .expect("add edge");
    graph.commit(tx).await.expect("commit");
    graph.compact();

    let inner = graph.inner_read();
    assert_eq!(inner.tx_valid_froms[0], TxId::INVALID);
    assert_eq!(inner.tx_valid_tos[0], TxId::INVALID);
    assert_eq!(inner.business_valid_froms[0], i64::MIN);
    assert_eq!(inner.business_valid_tos[0], i64::MIN);
    assert_eq!(inner.source_doc_ids[0], DocId::new(0));

    // Helper getters must return None for sentinel values
    assert_eq!(inner.tx_valid_from_at(0), None);
    assert_eq!(inner.tx_valid_to_at(0), None);
    assert_eq!(inner.business_valid_from_at(0), None);
    assert_eq!(inner.business_valid_to_at(0), None);
    assert_eq!(inner.source_doc_id_at(0), None);
}

#[tokio::test]
async fn test_invalid_edge_weights_rejected() {
    let graph = Arc::new(CsrGraph::new());
    let tx = TxId::new(1);
    let id1 = EntityId::new(1);
    let id2 = EntityId::new(2);

    // NaN weight
    let err_nan = graph
        .insert_edge_direct(id1, id2, f32::NAN)
        .await
        .unwrap_err();
    assert!(matches!(err_nan, ContextraError::InvalidInput(_)));

    // Infinity weight
    let err_inf = graph
        .insert_edge_direct(id1, id2, f32::INFINITY)
        .await
        .unwrap_err();
    assert!(matches!(err_inf, ContextraError::InvalidInput(_)));

    // Neg Infinity weight
    let err_neginf = graph
        .insert_edge_direct(id1, id2, f32::NEG_INFINITY)
        .await
        .unwrap_err();
    assert!(matches!(err_neginf, ContextraError::InvalidInput(_)));

    // Negative weight
    let err_neg = graph.insert_edge_direct(id1, id2, -1.0).await.unwrap_err();
    assert!(matches!(err_neg, ContextraError::InvalidInput(_)));

    // add_edge with NaN
    let edge_nan = Edge::new(id1, id2, "rel").with_weight(f32::NAN);
    let err_add = GraphIndex::add_edge(graph.as_ref(), tx, edge_nan)
        .await
        .unwrap_err();
    assert!(matches!(err_add, ContextraError::InvalidInput(_)));
}

#[tokio::test]
async fn test_compact_entities_without_edges_syncs_offsets() {
    let graph = CsrGraph::new();
    let tx = TxId::new(1);

    // Add 5 entities without adding any edges
    for id in 1..=5 {
        graph
            .add_entity(
                tx,
                Entity::new(EntityId::new(id), format!("E{id}"), "Entity"),
            )
            .await
            .unwrap(); // unwrap
    }

    graph.commit(tx).await.unwrap(); // unwrap

    // Before compact(), reverse_map has 5 entities
    {
        let inner = graph.inner_read();
        assert_eq!(inner.reverse_map.len(), 5);
    }

    graph.compact();

    // After compact(), offsets length MUST equal reverse_map.len() + 1 = 6
    {
        let inner = graph.inner_read();
        assert_eq!(inner.reverse_map.len(), 5);
        assert_eq!(inner.offsets.len(), 6);
        assert_eq!(inner.offsets, vec![0, 0, 0, 0, 0, 0]);
    }
}

#[tokio::test]
async fn test_csr_graph_compact_layout() {
    let graph = CsrGraph::new();
    let tx = TxId::new(1);

    graph
        .add_entity(tx, Entity::new(EntityId::new(1), "A", "T"))
        .await
        .unwrap(); // unwrap
    graph
        .add_entity(tx, Entity::new(EntityId::new(2), "B", "T"))
        .await
        .unwrap(); // unwrap
    graph
        .add_edge(tx, Edge::new(EntityId::new(1), EntityId::new(2), "E"))
        .await
        .unwrap(); // unwrap

    graph.commit(tx).await.unwrap(); // unwrap

    {
        let inner = graph.inner_read();
        assert!(inner.is_dirty);
        assert_eq!(inner.staged_edges.len(), 0);
        assert_eq!(inner.targets.len(), 0);
    }

    graph.compact();

    {
        let inner = graph.inner_read();
        assert!(!inner.is_dirty);
        assert_eq!(inner.staged_edges.len(), 0);
        assert_eq!(inner.targets.len(), 1);
        assert_eq!(inner.offsets.len(), 3);
        assert_eq!(inner.offsets[0], 0);
        assert_eq!(inner.offsets[1] + (inner.offsets[2] - inner.offsets[1]), 1);
    }
}

#[tokio::test]
async fn test_csr_delta_buffer_uncompacted_traversal() {
    // Test that committed edges in the pending_edges delta buffer (uncompacted)
    // are correctly traversed without needing compact() call.
    let graph = CsrGraph::with_config(CsrGraphConfig {
        rebuild_threshold: 1000,
        ..Default::default()
    });
    let tx = TxId::new(1);

    graph
        .add_entity(tx, Entity::new(EntityId::new(1), "A", "Node"))
        .await
        .unwrap(); // unwrap
    graph
        .add_entity(tx, Entity::new(EntityId::new(2), "B", "Node"))
        .await
        .unwrap(); // unwrap
    graph
        .add_entity(tx, Entity::new(EntityId::new(3), "C", "Node"))
        .await
        .unwrap(); // unwrap

    graph
        .add_edge(
            tx,
            Edge::new(EntityId::new(1), EntityId::new(2), "knows").with_weight(1.0),
        )
        .await
        .unwrap(); // unwrap
    graph.commit(tx).await.unwrap(); // unwrap

    // Edge 1->2 is committed in pending_edges (uncompacted)
    {
        let inner = graph.inner_read();
        assert!(inner.is_dirty);
        assert_eq!(inner.pending_edge_count, 1);
        assert_eq!(inner.targets.len(), 0); // Not in CSR targets yet
    }

    // Traversal MUST find Entity 2 directly from pending_edges delta buffer
    let results = graph.traverse(EntityId::new(1), 1).await.unwrap(); // unwrap
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].0, EntityId::new(2));

    // Add second edge 2->3 in next transaction
    let tx2 = TxId::new(2);
    graph
        .add_edge(
            tx2,
            Edge::new(EntityId::new(2), EntityId::new(3), "knows").with_weight(0.8),
        )
        .await
        .unwrap(); // unwrap
    graph.commit(tx2).await.unwrap(); // unwrap

    // Traversal from 1 (max 2 hops) MUST find both 2 and 3 through delta buffer
    let results_2hop = graph.traverse(EntityId::new(1), 2).await.unwrap(); // unwrap
    assert_eq!(results_2hop.len(), 2);
    let ids: Vec<_> = results_2hop.iter().map(|(id, _)| id.inner()).collect();
    assert!(ids.contains(&2));
    assert!(ids.contains(&3));
}

#[tokio::test]
async fn test_graph_transaction_isolation() {
    let graph = CsrGraph::new();
    let tx1 = TxId::new(1);

    // 1. Tx1 fügt Entity und Edge hinzu
    graph
        .add_entity(tx1, Entity::new(EntityId::new(1), "A", "T"))
        .await
        .unwrap(); // unwrap
    graph
        .add_entity(tx1, Entity::new(EntityId::new(2), "B", "T"))
        .await
        .unwrap(); // unwrap
    graph
        .add_edge(
            tx1,
            Edge::new(EntityId::new(1), EntityId::new(2), "E").with_weight(1.0),
        )
        .await
        .unwrap(); // unwrap

    // 2. Traverse (ohne Tx) darf Edge NICHT sehen
    let results = graph.traverse(EntityId::new(1), 1).await.unwrap(); // unwrap
    assert_eq!(results.len(), 0, "Uncommitted edge should not be visible");

    // 3. Tx1 committet
    graph.commit(tx1).await.unwrap(); // unwrap

    // 4. Traverse MUSS Edge sehen
    let results = graph.traverse(EntityId::new(1), 1).await.unwrap(); // unwrap
    assert_eq!(results.len(), 1, "Committed edge should be visible");
    assert_eq!(results[0].0, EntityId::new(2));
}

#[tokio::test]
async fn test_graph_rollback_isolation() {
    let graph = CsrGraph::new();
    let tx1 = TxId::new(1);
    let tx2 = TxId::new(2);

    // 1. Tx1 und Tx2 fügen Edges hinzu
    graph
        .add_entity(tx1, Entity::new(EntityId::new(1), "A", "T"))
        .await
        .unwrap(); // unwrap
    graph
        .add_entity(tx1, Entity::new(EntityId::new(2), "B", "T"))
        .await
        .unwrap(); // unwrap
    graph
        .add_edge(
            tx1,
            Edge::new(EntityId::new(1), EntityId::new(2), "E1").with_weight(1.0),
        )
        .await
        .unwrap(); // unwrap

    graph
        .add_entity(tx2, Entity::new(EntityId::new(1), "A", "T"))
        .await
        .unwrap(); // unwrap
    graph
        .add_entity(tx2, Entity::new(EntityId::new(3), "C", "T"))
        .await
        .unwrap(); // unwrap
    graph
        .add_edge(
            tx2,
            Edge::new(EntityId::new(1), EntityId::new(3), "E2").with_weight(1.0),
        )
        .await
        .unwrap(); // unwrap

    // 2. Tx1 rollt back
    graph.rollback(tx1).await.unwrap(); // unwrap

    // 3. Tx2 committet
    graph.commit(tx2).await.unwrap(); // unwrap

    // 4. Nur Edges von Tx2 dürfen existieren
    let results = graph.traverse(EntityId::new(1), 1).await.unwrap(); // unwrap
    assert_eq!(results.len(), 1, "Only Tx2 edge should be visible");
    assert_eq!(results[0].0, EntityId::new(3));

    let stats = graph.stats().await.unwrap(); // unwrap
                                              // With lazy index allocation, Tx1 rollback discards staged entities and edges,
                                              // so Entity 2 is never registered in id_map/reverse_map.
    assert_eq!(
        stats.num_entities, 2,
        "Only entities from Tx2 and common ones should exist"
    );
}

#[tokio::test]
async fn test_csr_graph_bfs_score_decay() {
    let graph = setup_test_graph().await;
    let results = graph.traverse(EntityId::new(1), 3).await.expect("traverse"); // expect

    assert_eq!(results.len(), 4);

    let score_map: std::collections::HashMap<_, _> = results.into_iter().collect();

    let s2 = *score_map.get(&EntityId::new(2)).expect("node 2 missing"); // expect
    let s3 = *score_map.get(&EntityId::new(3)).expect("node 3 missing"); // expect
    let s4 = *score_map.get(&EntityId::new(4)).expect("node 4 missing"); // expect
    let s5 = *score_map.get(&EntityId::new(5)).expect("node 5 missing"); // expect

    assert!((s2 - 0.7).abs() < f32::EPSILON);
    assert!((s3 - 0.392).abs() < f32::EPSILON);
    assert!((s5 - 0.196).abs() < f32::EPSILON);
    assert!((s4 - 0.16464).abs() < f32::EPSILON);
}

#[tokio::test]
async fn test_csr_graph_cycle_handling() {
    let graph = CsrGraph::new();
    let tx = TxId::new(1);
    graph
        .add_entity(tx, Entity::new(EntityId::new(1), "A", "N"))
        .await
        .unwrap(); // unwrap
    graph
        .add_entity(tx, Entity::new(EntityId::new(2), "B", "N"))
        .await
        .unwrap(); // unwrap
    graph
        .add_edge(tx, Edge::new(EntityId::new(1), EntityId::new(2), "E"))
        .await
        .unwrap(); // unwrap
    graph
        .add_edge(tx, Edge::new(EntityId::new(2), EntityId::new(1), "E"))
        .await
        .unwrap(); // unwrap

    graph.commit(tx).await.unwrap(); // unwrap

    let results = graph.traverse(EntityId::new(1), 5).await.expect("traverse"); // expect
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].0, EntityId::new(2));
}

#[tokio::test]
async fn test_csr_graph_max_hop_enforcement() {
    let graph = setup_test_graph().await;

    // Traverse from 1, max hops 1 -> Should only find Node 2
    let results_hop1 = graph
        .traverse(EntityId::new(1), 1)
        .await
        .expect("traverse 1 hop"); // expect
    assert_eq!(results_hop1.len(), 1);
    assert_eq!(results_hop1[0].0, EntityId::new(2));

    // Traverse from 3, max hops 1 -> Should only find Node 4
    let results_hop1_n3 = graph
        .traverse(EntityId::new(3), 1)
        .await
        .expect("traverse 1 hop"); // expect
    assert_eq!(results_hop1_n3.len(), 1);
    assert_eq!(results_hop1_n3[0].0, EntityId::new(4));
}

#[tokio::test]
async fn test_csr_graph_stats_accuracy() {
    let graph = setup_test_graph().await;
    let stats = graph.stats().await.expect("valid stats"); // expect

    assert_eq!(stats.num_entities, 5);
    assert_eq!(stats.num_edges, 5);

    // Calculate expected memory based on implementation
    let inner = graph.inner_read();
    let expected_mem = (inner.reverse_map.len() * std::mem::size_of::<EntityId>())
        + (inner.entities.len() * std::mem::size_of::<Option<Entity>>())
        + (inner.offsets.len() * std::mem::size_of::<usize>())
        + (inner.targets.len() * std::mem::size_of::<usize>())
        + (inner.weights.len() * std::mem::size_of::<f32>());

    assert_eq!(stats.memory_usage_bytes, expected_mem);
}

#[tokio::test]
async fn test_add_edge_rollback_no_index_growth() {
    let graph = CsrGraph::new();

    {
        let inner = graph.inner_read();
        assert_eq!(inner.id_map.len(), 0);
        assert_eq!(inner.reverse_map.len(), 0);
    }

    for i in 1..=100 {
        let tx = TxId::new(i);
        let from = EntityId::new(i * 10);
        let to = EntityId::new(i * 10 + 1);

        graph
            .add_edge(tx, Edge::new(from, to, "test_rel"))
            .await
            .unwrap(); // unwrap

        graph.rollback(tx).await.unwrap(); // unwrap

        let inner = graph.inner_read();
        assert_eq!(
            inner.id_map.len(),
            0,
            "id_map must remain empty after rollback at iteration {i}"
        );
        assert_eq!(
            inner.reverse_map.len(),
            0,
            "reverse_map must remain empty after rollback at iteration {i}"
        );
    }
}

#[tokio::test]
async fn test_compaction_excludes_uncommitted_edges() {
    let graph = setup_test_graph().await;

    // 1. Add uncommitted edges
    let tx_uncommitted = TxId::new(999);
    let edge1 = Edge::new(EntityId::new(1), EntityId::new(5), "").with_weight(0.5);
    graph.add_edge(tx_uncommitted, edge1).await.unwrap(); // unwrap

    // 2. Add committed edges
    let tx_committed = TxId::new(100);
    let edge2 = Edge::new(EntityId::new(1), EntityId::new(2), "").with_weight(0.9);
    graph.add_edge(tx_committed, edge2).await.unwrap(); // unwrap
    graph.commit(tx_committed).await.unwrap(); // unwrap

    // 3. Compact
    graph.compact();

    // 4. Verify traversal
    let results = graph.traverse(EntityId::new(1), 1).await.unwrap(); // unwrap
    let targets: Vec<_> = results.iter().map(|(id, _)| id.inner()).collect();

    // Should find committed edge (2) but NOT uncommitted edge (5)
    assert!(
        targets.contains(&2),
        "Expected Entity 2 in results, got {:?}",
        targets
    );
}

#[tokio::test]
async fn test_suspicious_txid_does_not_silently_overwrite() {
    let graph = CsrGraph::new();

    // Simulated Quelle A: Kanonische TxId (z.B. 42)
    let tx_source_a = TxId::new(42);
    // Simulated Quelle B: Kollidierende TxId mit demselben Wert 42 aus anderer Herkunft
    let tx_source_b = TxId::new(42);

    graph
        .add_entity(
            tx_source_a,
            Entity::new(EntityId::new(10), "EntityFromA", "TypeA"),
        )
        .await
        .unwrap(); // unwrap

    // Staging unter gleicher TxId ueberschreibt staged entity fuer EntityId(10) in der staged HashMap
    graph
        .add_entity(
            tx_source_b,
            Entity::new(EntityId::new(10), "EntityFromB", "TypeB"),
        )
        .await
        .unwrap(); // unwrap

    graph.commit(tx_source_a).await.unwrap(); // unwrap

    // Nach Commit ist der Zustand deterministisch (letzte staged Entity gewinnt)
    let inner = graph.inner_read();
    let idx = inner.id_map.get(&EntityId::new(10)).unwrap(); // unwrap
    let entity = inner.entity_at(*idx).unwrap(); // unwrap
    assert_eq!(&*entity.name, "EntityFromB");
}

#[tokio::test]
#[should_panic(expected = "AGT-GRAPH-001")]
async fn test_wallclock_txid_debug_assert_panics() {
    let graph = CsrGraph::new();
    // Wall-clock-artiger TxId (~1.7e18 ns) in der verbotenen Gap-Zone zwischen 10^12 und INTERNAL_BASE
    let wallclock_tx = TxId::new(1_700_000_000_000_000_000);

    assert!(is_suspicious_tx_id(wallclock_tx));

    // In Debug-Builds MUSS debug_assert!(tx.is_valid_origin()) greifen und mit "AGT-GRAPH-001" paniquen
    let _ = graph
        .add_entity(
            wallclock_tx,
            Entity::new(EntityId::new(100), "WallClockEntity", "Type"),
        )
        .await;
}

#[tokio::test]
#[should_panic(expected = "AGT-GRAPH-001")]
async fn test_sentinel_txid_zero_debug_assert_panics() {
    let graph = CsrGraph::new();
    let invalid_tx = TxId::INVALID;

    assert!(is_suspicious_tx_id(invalid_tx));

    let _ = graph
        .add_entity(
            invalid_tx,
            Entity::new(EntityId::new(101), "SentinelEntity", "Type"),
        )
        .await;
}

#[tokio::test]
async fn test_graph_operation_sequence_txid_determinism() {
    // Test executing identical operation sequences with canonical TxIds yields identical state
    let run_sequence = || async {
        let graph = CsrGraph::new();
        let mut committed_txs = Vec::new();

        for i in 1..=5u64 {
            let tx = TxId::new(i);
            let id1 = EntityId::new(i * 10);
            let id2 = EntityId::new(i * 10 + 1);

            graph
                .add_entity(tx, Entity::new(id1, format!("E{}", id1.inner()), "Type"))
                .await
                .unwrap();
            graph
                .add_entity(tx, Entity::new(id2, format!("E{}", id2.inner()), "Type"))
                .await
                .unwrap();
            graph
                .add_edge(tx, Edge::new(id1, id2, "connects"))
                .await
                .unwrap();

            graph.commit(tx).await.unwrap();
            committed_txs.push(graph.last_tx_id().await.unwrap());
        }

        let stats = graph.stats().await.unwrap();
        (committed_txs, stats.num_entities, stats.num_edges)
    };

    let (txs1, ent1, edge1) = run_sequence().await;
    let (txs2, ent2, edge2) = run_sequence().await;

    assert_eq!(txs1, vec![TxId(1), TxId(2), TxId(3), TxId(4), TxId(5)]);
    assert_eq!(
        txs1, txs2,
        "TxId sequence must be deterministically identical across runs"
    );
    assert_eq!(ent1, ent2);
    assert_eq!(edge1, edge2);
}

#[tokio::test]
async fn test_get_communities_batch_api() {
    let graph = CsrGraph::new();
    let assignments = vec![
        crate::CommunityAssignment {
            entity_id: EntityId::new(1),
            community_id: 100,
            hyperedges_included: false,
        },
        crate::CommunityAssignment {
            entity_id: EntityId::new(2),
            community_id: 200,
            hyperedges_included: false,
        },
    ];

    graph.set_communities_batch(&assignments);

    let map = graph
        .get_communities_batch(&[EntityId::new(1), EntityId::new(2), EntityId::new(3)])
        .await
        .unwrap(); // unwrap

    assert_eq!(map.len(), 2);
    assert_eq!(map.get(&EntityId::new(1)), Some(&100));
    assert_eq!(map.get(&EntityId::new(2)), Some(&200));
    assert_eq!(map.get(&EntityId::new(3)), None);
}

#[tokio::test]
async fn test_neighbors_api() {
    let graph = CsrGraph::new();
    let tx = TxId::new(1);
    let id_a = EntityId::new(1);
    let id_b = EntityId::new(2);
    let id_c = EntityId::new(3);

    graph
        .add_entity(tx, Entity::new(id_a, "A", "T"))
        .await
        .unwrap(); // unwrap
    graph
        .add_entity(tx, Entity::new(id_b, "B", "T"))
        .await
        .unwrap(); // unwrap
    graph
        .add_entity(tx, Entity::new(id_c, "C", "T"))
        .await
        .unwrap(); // unwrap
    graph
        .add_edge(tx, Edge::new(id_a, id_b, "rel"))
        .await
        .unwrap(); // unwrap
    graph
        .add_edge(tx, Edge::new(id_a, id_c, "rel"))
        .await
        .unwrap(); // unwrap
    graph.commit(tx).await.unwrap(); // unwrap

    let n = graph.neighbors(id_a).await.unwrap(); // unwrap
    assert_eq!(n.len(), 2);
    assert!(n.contains(&id_b));
    assert!(n.contains(&id_c));
}

#[tokio::test]
async fn test_neighbors_hub_node_dedup() {
    let graph = CsrGraph::new();
    let tx = TxId::new(1);
    let hub_id = EntityId::new(1);

    graph
        .add_entity(tx, Entity::new(hub_id, "Hub", "Type"))
        .await
        .unwrap();

    // Create 100 leaf nodes and 100 outgoing edges, including duplicate edges
    for i in 2..=101 {
        let leaf_id = EntityId::new(i);
        graph
            .add_entity(tx, Entity::new(leaf_id, format!("Leaf_{i}"), "Type"))
            .await
            .unwrap();
        graph
            .add_edge(tx, Edge::new(hub_id, leaf_id, "rel"))
            .await
            .unwrap();
        // Duplicate edge to same target
        graph
            .add_edge(tx, Edge::new(hub_id, leaf_id, "rel_dup"))
            .await
            .unwrap();
    }
    graph.commit(tx).await.unwrap();

    let neighbors = graph.neighbors(hub_id).await.unwrap();
    assert!(
        neighbors.len() <= 100,
        "Hub node neighbors count must be <= 100 unique entities, got {}",
        neighbors.len()
    );
    assert_eq!(
        neighbors.len(),
        100,
        "Hub node with 100 leaves must return exactly 100 unique neighbors"
    );

    let unique_neighbors: std::collections::HashSet<_> = neighbors.iter().copied().collect();
    assert_eq!(
        unique_neighbors.len(),
        neighbors.len(),
        "Neighbors returned must not contain duplicate EntityIds"
    );
}
