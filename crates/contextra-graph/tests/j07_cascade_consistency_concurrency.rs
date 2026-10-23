#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::indexing_slicing)]
// FILE-CONTEXT
// ZWECK: Parallelitäts- und Edge-Case-Tests für Cascade-Invalidation & Consistency-Enforcement.
// INVARIANTEN: Determinismus, keine wall-clock sleep(), Snapshot-Isolation, Zero-Panic.
// TASK-KARTE: J07

use std::sync::Arc;
use tokio::sync::Barrier;

use contextra_graph::cascade::{
    cascade_invalidate_edges_for_superseded_doc, cascade_invalidate_hyperedges_for_superseded_doc,
    topological_sort_hyperedge_closure,
};
use contextra_graph::consistency_enforcement::{
    ConsistencyEnforcer, EdgeAssertion, ExactPredicateConflictDetector,
};
use contextra_graph::csr::EdgeType;
use contextra_graph::hyperedge::{HyperEdge, HyperEdgeId, RoleBinding, RoleId};
use contextra_graph::CsrGraph;
use contextra_ports::GraphIndex;
use contextra_types::{DocId, Edge, Entity, EntityId, PprAlgorithm, PprConfig, TxId};

/// INVARIANTE: Cascade-Delete parallel zu Traversal/PPR auf einem gepinnten Snapshot
/// wahrt Snapshot-Isolation. Der Snapshot bei `seq = 10` sieht alle Kanten ungeachtet des
/// zeitgleich stattfindenden Cascade-Deletes bei `seq = 20`.
#[tokio::test]
async fn test_j07_cascade_delete_parallel_to_snapshot_ppr() {
    let graph = Arc::new(CsrGraph::new());
    let doc_a = DocId::from_key("doc-cascade-snapshot").unwrap();

    let node_1 = EntityId::new(1001);
    let node_2 = EntityId::new(1002);
    let node_3 = EntityId::new(1003);

    let tx10 = TxId::try_new(10).unwrap();
    graph
        .add_entity(tx10, Entity::new(node_1, "n1", "Node"))
        .await
        .unwrap();
    graph
        .add_entity(tx10, Entity::new(node_2, "n2", "Node"))
        .await
        .unwrap();
    graph
        .add_entity(tx10, Entity::new(node_3, "n3", "Node"))
        .await
        .unwrap();

    let edge12 = Edge::new(node_1, node_2, "rel1").with_source_doc_id(doc_a);
    let edge23 = Edge::new(node_2, node_3, "rel2").with_source_doc_id(doc_a);

    GraphIndex::add_edge(graph.as_ref(), tx10, edge12)
        .await
        .unwrap();
    GraphIndex::add_edge(graph.as_ref(), tx10, edge23)
        .await
        .unwrap();
    graph.commit(tx10).await.unwrap();

    let barrier = Arc::new(Barrier::new(2));

    // Thread 1: Cascade-Delete ausführen bei seq 20
    let graph_del = Arc::clone(&graph);
    let barrier_del = Arc::clone(&barrier);
    let del_task = tokio::spawn(async move {
        barrier_del.wait().await;
        let report = cascade_invalidate_edges_for_superseded_doc(&graph_del, doc_a, 20)
            .await
            .expect("Cascade invalidation must succeed");
        report
    });

    // Thread 2: Snapshot PPR bei seq 10 ausführen
    let graph_ppr = Arc::clone(&graph);
    let barrier_ppr = Arc::clone(&barrier);
    let ppr_task = tokio::spawn(async move {
        let ppr_cfg = PprConfig {
            damping_factor: 0.85,
            max_iterations: 100,
            convergence_epsilon: 1e-6,
            algorithm: PprAlgorithm::ForwardPush,
            warn_on_non_convergence: true,
        };
        barrier_ppr.wait().await;
        let ppr_res = graph_ppr
            .personalized_page_rank_at(&[node_1], &ppr_cfg, 10)
            .await
            .expect("Snapshot PPR must succeed");
        ppr_res
    });

    let (report, ppr_res) = tokio::join!(del_task, ppr_task);
    let report = report.unwrap();
    let ppr_res = ppr_res.unwrap();

    assert_eq!(
        report.tombstoned_edge_count, 2,
        "Exactly 2 edges must be tombstoned"
    );

    let ppr_nodes: Vec<_> = ppr_res.iter().map(|(e, _)| *e).collect();
    assert!(
        ppr_nodes.contains(&node_2),
        "node_2 must be visible in snapshot at seq 10 despite concurrent cascade delete at seq 20"
    );
    assert!(
        ppr_nodes.contains(&node_3),
        "node_3 must be visible in snapshot at seq 10 despite concurrent cascade delete at seq 20"
    );

    // Live-Abfrage post-Cascade zeigt leere Nachbarschaft
    let live_neighbors = graph.neighbors(node_1).await.unwrap();
    assert!(
        live_neighbors.is_empty(),
        "Live neighbors at seq 20+ must be empty post-cascade"
    );
}

/// INVARIANTE: Hyperedge-Cascade mit Zyklen, Selbstreferenzen, 0 Nachfolgern, tiefer Kette
/// und TxId-Randwerten (`MAX_COLLECTION_SEQUENCE` & `INTERNAL_BASE`).
#[tokio::test]
async fn test_j07_hyperedge_cascade_cycles_self_ref_deep_chain_and_txid_bounds() {
    let graph = CsrGraph::new();
    let doc_cyclic = DocId::from_key("doc-cyclic").unwrap();

    const ROLE_S: RoleId = RoleId::new(1);
    const ROLE_O: RoleId = RoleId::new(2);

    let h1_id = HyperEdgeId::new(101);
    let h2_id = HyperEdgeId::new(102);

    // Selbstreferenz und gegenseitige Eltern-Kind-Beziehung (Zyklus)
    let h1 = HyperEdge::new(
        h1_id,
        EdgeType::Default,
        vec![
            RoleBinding::new(ROLE_S, EntityId::new(1)),
            RoleBinding::new(ROLE_O, EntityId::new(2)),
        ],
        1.0,
    )
    .with_source_doc_id(Some(doc_cyclic))
    .with_child_edge_ids(vec![h2_id]);

    let h2 = HyperEdge::new(
        h2_id,
        EdgeType::Default,
        vec![
            RoleBinding::new(ROLE_S, EntityId::new(2)),
            RoleBinding::new(ROLE_O, EntityId::new(1)),
        ],
        1.0,
    )
    .with_source_doc_id(Some(doc_cyclic))
    .with_child_edge_ids(vec![h1_id]);

    graph.insert_hyperedge_direct(h1);
    graph.insert_hyperedge_direct(h2);

    // Topologische Sortierung darf bei Zyklen nicht in Endlosschleife geraten
    let sorted = topological_sort_hyperedge_closure(&graph, &[h1_id, h2_id]);
    assert_eq!(sorted.len(), 2);

    // Invalidation mit MAX_COLLECTION_SEQUENCE (Grenzwert 10^12)
    let seq_max_coll = TxId::MAX_COLLECTION_SEQUENCE;
    let report_coll =
        cascade_invalidate_hyperedges_for_superseded_doc(&graph, doc_cyclic, seq_max_coll)
            .await
            .unwrap();
    assert_eq!(report_coll.invalidated.len(), 2);

    // Deep-Chain: 100 tief geschachtelte Hyperkanten
    // (H_1 ist die vom Dokument abgeleitete Hyperkante. Für i > 1 besitzt H_i die Kind-Kante H_{i-1} in child_edge_ids,
    // so dass H_i der Superedge-Elternknoten von H_{i-1} ist. dfs_upward ausgehend von H_1 klettert die gesamte
    // 100-Stufen-Elternhierarchie empor.)
    let graph_deep = CsrGraph::new();
    let doc_deep = DocId::from_key("doc-deep-chain").unwrap();

    let deep_count = 100usize;
    for i in 1..=deep_count {
        let hid = HyperEdgeId::new(i as u64);
        let child_ids = if i > 1 {
            vec![HyperEdgeId::new((i - 1) as u64)]
        } else {
            vec![]
        };
        let he = HyperEdge::new(
            hid,
            EdgeType::Default,
            vec![
                RoleBinding::new(ROLE_S, EntityId::new(i as u64)),
                RoleBinding::new(ROLE_O, EntityId::new((i + 1) as u64)),
            ],
            1.0,
        )
        .with_source_doc_id(if i == 1 { Some(doc_deep) } else { None })
        .with_child_edge_ids(child_ids);

        graph_deep.insert_hyperedge_direct(he);
    }

    // Invalidation mit INTERNAL_BASE (Grenzwert u64::MAX - 1_000_000)
    let seq_internal = TxId::INTERNAL_BASE;
    let report_deep =
        cascade_invalidate_hyperedges_for_superseded_doc(&graph_deep, doc_deep, seq_internal)
            .await
            .unwrap();

    assert_eq!(
        report_deep.invalidated.len(),
        deep_count,
        "All 100 ancestors in deep chain must be cascade invalidated"
    );

    // TxId-Gap-Validierung: Ungültige wal_seq im Gap muss mit ContextraError::Transaction ablehnen
    let invalid_gap_seq = TxId::MAX_COLLECTION_SEQUENCE + 100_000;
    let err_res =
        cascade_invalidate_hyperedges_for_superseded_doc(&graph_deep, doc_deep, invalid_gap_seq)
            .await;
    assert!(
        err_res.is_err(),
        "wal_seq in AGT-GRAPH-001 invalid gap must return an error"
    );
}

/// INVARIANTE: Paralleles `record_contradiction` auf demselben Pattern-Hash
/// muss exakte Zähler-Akkumulation und Idempotenz gewährleisten.
#[tokio::test]
async fn test_j07_record_contradiction_parallel_idempotency_and_empty_pattern() {
    let enforcer = Arc::new(tokio::sync::RwLock::new(ConsistencyEnforcer::new(10)));
    let pattern_hash = [0x42u8; 32];

    let num_threads = 10;
    let iterations_per_thread = 20;
    let barrier = Arc::new(Barrier::new(num_threads));

    let mut tasks = Vec::new();
    for t_idx in 0..num_threads {
        let enforcer_cloned = Arc::clone(&enforcer);
        let barrier_cloned = Arc::clone(&barrier);
        tasks.push(tokio::spawn(async move {
            barrier_cloned.wait().await;
            for i in 0..iterations_per_thread {
                let tx_val = ((t_idx * iterations_per_thread + i + 1) as u64) % 1_000 + 1;
                let tx = TxId::try_new(tx_val).expect("Valid tx");
                let mut guard = enforcer_cloned.write().await;
                guard.record_contradiction(pattern_hash, tx);
            }
        }));
    }

    for task in tasks {
        task.await.unwrap();
    }

    let guard = enforcer.read().await;
    let pattern = guard
        .get_pattern(&pattern_hash)
        .expect("Pattern must exist");
    assert_eq!(
        pattern.contradiction_count,
        (num_threads * iterations_per_thread) as u32,
        "Total contradiction count must exactly equal total parallel updates"
    );
    assert!(
        pattern.suppressed,
        "Pattern must be suppressed after exceeding threshold 10"
    );

    // Edge Case: Leere Repräsentation in EdgeAssertion und TxId::INVALID Handling
    let mut local_enforcer = ConsistencyEnforcer::new(1);
    let empty_assertion = EdgeAssertion {
        subject: 99,
        predicate_hash: [0u8; 32],
        object_repr: Vec::new(),
    };

    let cp = local_enforcer
        .check_before_insert(&empty_assertion)
        .expect("check_before_insert must return ConflictPattern");
    assert_eq!(cp.first_detected_tx, TxId::INVALID);
    assert_eq!(cp.last_detected_tx, TxId::INVALID);
    assert!(cp.suppressed);

    // ContradictionDetector verifiziert exakte Prädikats-Semantik
    let detector = ExactPredicateConflictDetector;
    let a1 = EdgeAssertion {
        subject: 1,
        predicate_hash: [5u8; 32],
        object_repr: b"val1".to_vec(),
    };
    let a2 = EdgeAssertion {
        subject: 1,
        predicate_hash: [5u8; 32],
        object_repr: b"val2".to_vec(),
    };
    assert!(
        local_enforcer.detect_contradiction(&detector, &a1, &a2),
        "Detector must detect conflict on different object_repr with same subject/predicate"
    );
}
