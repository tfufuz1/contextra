use super::super::*;
use crate::csr::inner::GraphInner;
use crate::csr::types::{CsrGraphConfig, EdgeType};
use crate::GraphIndexExt;
use contextra_types::{DocId, Edge, Entity, EntityId, ContextraError, TxId};
use contextra_ports::GraphIndex;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

#[tokio::test]
async fn test_rcu_snapshot_isolation_no_torn_reads() {
    let graph = Arc::new(CsrGraph::new());
    let tx = TxId::new(1);

    for i in 1..=500 {
        graph
            .add_entity(tx, Entity::new(EntityId::new(i), format!("N{i}"), "Type"))
            .await
            .unwrap();
    }
    for i in 1..500 {
        GraphIndex::add_edge(
            graph.as_ref(),
            tx,
            Edge::new(EntityId::new(i), EntityId::new(i + 1), "rel").with_weight(1.0),
        )
        .await
        .unwrap();
    }
    graph.commit(tx).await.unwrap();

    // Reader acquires point-in-time RCU snapshot BEFORE compaction
    let snapshot_v1 = graph.inner_read();
    assert_eq!(
        snapshot_v1
            .entities
            .iter()
            .filter(|e| e.id != EntityId::new(0))
            .count(),
        500
    );

    // Mutate graph with additional transaction
    let tx2 = TxId::new(2);
    for i in 501..=1000 {
        graph
            .add_entity(tx2, Entity::new(EntityId::new(i), format!("N{i}"), "Type"))
            .await
            .unwrap();
    }
    for i in 500..1000 {
        GraphIndex::add_edge(
            graph.as_ref(),
            tx2,
            Edge::new(EntityId::new(i), EntityId::new(i + 1), "rel").with_weight(1.0),
        )
        .await
        .unwrap();
    }
    graph.commit(tx2).await.unwrap();
    graph.compact(); // Trigger full CSR compaction

    // Snapshot held by v1 MUST remain isolated on v1 state without torn reads
    assert_eq!(
        snapshot_v1
            .entities
            .iter()
            .filter(|e| e.id != EntityId::new(0))
            .count(),
        500
    );

    // New reader loads published post-compaction v2 snapshot
    let snapshot_v2 = graph.inner_read();
    assert_eq!(
        snapshot_v2
            .entities
            .iter()
            .filter(|e| e.id != EntityId::new(0))
            .count(),
        1000
    );
}

#[tokio::test]
async fn test_rcu_concurrent_readers_never_block_during_compaction() {
    let graph = Arc::new(CsrGraph::with_config(CsrGraphConfig {
        rebuild_threshold: 10,
        ..Default::default()
    }));

    let setup_tx = TxId::new(1);
    for i in 1..=200 {
        graph
            .add_entity(
                setup_tx,
                Entity::new(EntityId::new(i), format!("N{i}"), "Node"),
            )
            .await
            .unwrap();
    }
    graph.commit(setup_tx).await.unwrap();

    let stop_flag = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let max_reader_load_nanos = Arc::new(AtomicU64::new(0));

    // Spawn 8 parallel reader tasks
    let mut reader_handles = Vec::new();
    for _ in 0..8 {
        let g = graph.clone();
        let stop = stop_flag.clone();
        let max_load = max_reader_load_nanos.clone();
        reader_handles.push(tokio::spawn(async move {
            while !stop.load(Ordering::Relaxed) {
                let start = std::time::Instant::now();
                let snapshot = g.inner_read();
                let load_duration_nanos = start.elapsed().as_nanos() as u64;
                max_load.fetch_max(load_duration_nanos, Ordering::Relaxed);

                // Traversal runs lock-free on snapshot
                let _ = g.traverse(EntityId::new(1), 2).await;
                drop(snapshot);
                tokio::task::yield_now().await;
            }
        }));
    }

    // Writer task continuously adds edges and triggers compact_async()
    let g_writer = graph.clone();
    let stop_writer = stop_flag.clone();
    let writer_handle = tokio::spawn(async move {
        let mut iter = 0u64;
        while !stop_writer.load(Ordering::Relaxed) {
            iter += 1;
            let tx = TxId::new(10 + iter);
            let src = EntityId::new((iter % 150) + 1);
            let dst = EntityId::new(((iter * 3) % 150) + 1);
            let _ = GraphIndex::add_edge(
                g_writer.as_ref(),
                tx,
                Edge::new(src, dst, "link").with_weight(0.9),
            )
            .await;
            g_writer.commit(tx).await.ok();
            g_writer.compact_async().await.ok();
            tokio::task::yield_now().await;
        }
    });

    // Run concurrent test loop for 2 seconds
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    stop_flag.store(true, Ordering::Relaxed);

    for h in reader_handles {
        h.await.unwrap();
    }
    writer_handle.await.unwrap();

    let max_ns = max_reader_load_nanos.load(Ordering::Relaxed);
    let max_ms = max_ns as f64 / 1_000_000.0;
    println!(
        "RCU ArcSwap::load max reader latency: {:.4} ms ({} ns)",
        max_ms, max_ns
    );

    assert!(
        max_ms < 5.0,
        "RCU ArcSwap::load must remain lock-free (< 5.0 ms), got {:.4} ms",
        max_ms
    );
}

/// CONTRACT STUB: Demonstrates downstream PPR / Graph search access contract.
/// Follow-up agents updating `ppr.rs` or `search.rs` consume `graph.inner_read()`.
/// Read-your-own-write consistency requires inspecting both snapshot and uncompacted pending buffers.
#[tokio::test]
async fn test_ppr_read_contract_stub() {
    let graph = CsrGraph::new();
    let tx = TxId::new(1);
    let id1 = EntityId::new(1);
    let id2 = EntityId::new(2);

    graph
        .add_entity(tx, Entity::new(id1, "P1", "Person"))
        .await
        .unwrap();
    graph
        .add_entity(tx, Entity::new(id2, "P2", "Person"))
        .await
        .unwrap();
    GraphIndex::add_edge(&graph, tx, Edge::new(id1, id2, "knows"))
        .await
        .unwrap();
    graph.commit(tx).await.unwrap();

    // PPR contract: inner_read() returns point-in-time immutable Guard<Arc<GraphInner>>
    let snapshot = graph.inner_read();
    assert_eq!(
        snapshot
            .entities
            .iter()
            .filter(|e| e.id != EntityId::new(0))
            .count(),
        2
    );
    assert!(!snapshot.reverse_map.is_empty());

    // Snapshot coerces to &GraphInner for PPR
    fn consume_ppr_inner(inner: &GraphInner) -> usize {
        inner.reverse_map.len()
    }
    assert_eq!(consume_ppr_inner(&snapshot), 2);
}

#[tokio::test]
async fn test_source_doc_id_provenance_and_compact_isolation() {
    let graph = Arc::new(CsrGraph::new());
    let tx = TxId::new(1);

    let id1 = EntityId::new(10);
    let id2 = EntityId::new(20);
    let id3 = EntityId::new(30);

    let doc100 = DocId(100);
    let doc200 = DocId(200);

    graph
        .insert_entity_direct(Entity::new(id1, "N1", "Type"))
        .unwrap();
    graph
        .insert_entity_direct(Entity::new(id2, "N2", "Type"))
        .unwrap();
    graph
        .insert_entity_direct(Entity::new(id3, "N3", "Type"))
        .unwrap();

    // Edge 10 -> 20 from doc100
    graph
        .add_edge(
            id1,
            id2,
            1.0,
            Some(tx),
            None,
            None,
            None,
            Some(doc100),
            None,
            None,
        )
        .await
        .unwrap();

    // Edge 20 -> 30 from doc200
    graph
        .add_edge(
            id2,
            id3,
            1.0,
            Some(tx),
            None,
            None,
            None,
            Some(doc200),
            None,
            None,
        )
        .await
        .unwrap();

    // Before compact
    assert_eq!(graph.source_doc_id_at(id1, id2), Some(doc100));
    assert_eq!(graph.source_doc_id_at(id2, id3), Some(doc200));

    let edges_doc100_pre = graph.edges_for_doc(doc100);
    assert!(edges_doc100_pre.contains(&(id1, id2)));
    assert!(!edges_doc100_pre.contains(&(id2, id3)));

    // Ensure querying DocId(10) (which equals EntityId 10) does NOT contain foreign edge (id1, id2)
    let edges_bogus_doc = graph.edges_for_doc(DocId(10));
    assert!(!edges_bogus_doc.contains(&(id1, id2)));

    // Compact CSR graph
    graph.compact();

    // After compact
    assert_eq!(graph.source_doc_id_at(id1, id2), Some(doc100));
    assert_eq!(graph.source_doc_id_at(id2, id3), Some(doc200));

    let edges_doc100_post = graph.edges_for_doc(doc100);
    assert!(edges_doc100_post.contains(&(id1, id2)));
    assert!(!edges_doc100_post.contains(&(id2, id3)));

    let edges_doc200_post = graph.edges_for_doc(doc200);
    assert!(edges_doc200_post.contains(&(id2, id3)));
    assert!(!edges_doc200_post.contains(&(id1, id2)));

    // Verify parallel arrays source_doc_ids in GraphInner
    let inner = graph.inner_read();
    assert_eq!(inner.targets.len(), inner.source_doc_ids.len());
    for (idx, target_idx) in inner.targets.iter().enumerate() {
        let target_id = inner.reverse_map[*target_idx];
        let source_doc = inner.source_doc_ids[idx];
        if target_id == id2 {
            assert_eq!(source_doc, doc100);
        } else if target_id == id3 {
            assert_eq!(source_doc, doc200);
        }
    }
}

#[test]
fn test_hyperedge_insertion_and_lookup() {
    use crate::hyperedge::{HyperEdge, HyperEdgeId, RoleBinding, RoleId};

    let graph = CsrGraph::new();
    let e1 = EntityId::from("entity_a");
    let e2 = EntityId::from("entity_b");

    let rb1 = RoleBinding::new(RoleId::new(1), e1);
    let rb2 = RoleBinding::new(RoleId::new(2), e2);
    let he_id = HyperEdgeId(101);
    let edge = HyperEdge::new(he_id, EdgeType::Default, vec![rb1, rb2], 0.85);

    graph.insert_hyperedge(edge.clone());

    let retrieved = graph.get_hyperedge(he_id);
    assert_eq!(retrieved.as_deref(), Some(&edge));

    let e1_hes = graph.hyperedges_for_entity(e1);
    assert_eq!(e1_hes, vec![he_id]);

    let e2_hes = graph.hyperedges_for_entity(e2);
    assert_eq!(e2_hes, vec![he_id]);
}

#[test]
fn test_hyperedge_multiple_participants() {
    use crate::hyperedge::{HyperEdge, HyperEdgeId, RoleBinding, RoleId};

    let graph = CsrGraph::new();
    let e1 = EntityId::from("p1");
    let e2 = EntityId::from("p2");
    let e3 = EntityId::from("p3");

    let edge = HyperEdge::new(
        HyperEdgeId(202),
        EdgeType::Default,
        vec![
            RoleBinding::new(RoleId::new(1), e1),
            RoleBinding::new(RoleId::new(2), e2),
            RoleBinding::new(RoleId::new(3), e3),
        ],
        1.0,
    );

    graph.insert_hyperedge(edge);

    for entity in &[e1, e2, e3] {
        let hes = graph.hyperedges_for_entity(*entity);
        assert_eq!(hes, vec![HyperEdgeId(202)]);
    }
}

#[test]
fn test_entity_multiple_hyperedges() {
    use crate::hyperedge::{HyperEdge, HyperEdgeId, RoleBinding, RoleId};

    let graph = CsrGraph::new();
    let e1 = EntityId::from("shared_entity");
    let e2 = EntityId::from("other_entity");

    let he1 = HyperEdge::new(
        HyperEdgeId(1),
        EdgeType::Default,
        vec![
            RoleBinding::new(RoleId::new(1), e1),
            RoleBinding::new(RoleId::new(2), e2),
        ],
        0.5,
    );
    let he2 = HyperEdge::new(
        HyperEdgeId(2),
        EdgeType::Default,
        vec![
            RoleBinding::new(RoleId::new(2), e1),
            RoleBinding::new(RoleId::new(1), e2),
        ],
        0.7,
    );

    graph.insert_hyperedge(he1);
    graph.insert_hyperedge(he2);

    let hes = graph.hyperedges_for_entity(e1);
    assert_eq!(hes.len(), 2);
    assert!(hes.contains(&HyperEdgeId(1)));
    assert!(hes.contains(&HyperEdgeId(2)));
}

#[test]
fn test_hyperedge_nonexistent_entity() {
    use crate::hyperedge::HyperEdgeId;

    let graph = CsrGraph::new();
    let nonexistent = EntityId::from("missing_entity");

    let hes = graph.hyperedges_for_entity(nonexistent);
    assert!(hes.is_empty());

    let he = graph.get_hyperedge(HyperEdgeId(9999));
    assert!(he.is_none());
}

#[test]
fn test_hyperedge_memory_estimation() {
    use crate::hyperedge::{HyperEdge, HyperEdgeId, RoleBinding, RoleId};

    let graph = CsrGraph::new();
    let initial_bytes = graph.inner_read().estimate_memory_bytes();

    let e1 = EntityId::from("m1");
    let e2 = EntityId::from("m2");

    let edge = HyperEdge::new(
        HyperEdgeId(500),
        EdgeType::Default,
        vec![
            RoleBinding::new(RoleId::new(1), e1),
            RoleBinding::new(RoleId::new(2), e2),
        ],
        0.9,
    );

    graph.insert_hyperedge(edge);

    let new_bytes = graph.inner_read().estimate_memory_bytes();
    assert!(
        new_bytes > initial_bytes,
        "Expected memory estimation to increase from {initial_bytes} but got {new_bytes}"
    );
}

#[tokio::test]
async fn test_hyperedge_rcu_concurrent_compaction_consistency() {
    use crate::hyperedge::{HyperEdge, HyperEdgeId, RoleBinding, RoleId};

    let graph = std::sync::Arc::new(CsrGraph::new());
    let iterations = 50;

    let g_writer = graph.clone();
    let writer_handle = tokio::spawn(async move {
        for i in 0..iterations {
            let e1 = EntityId::from(format!("entity_{i}"));
            let e2 = EntityId::from(format!("entity_{}", i + 1));
            let he_id = HyperEdgeId(i as u64);

            let edge = HyperEdge::new(
                he_id,
                EdgeType::Default,
                vec![
                    RoleBinding::new(RoleId::new(1), e1),
                    RoleBinding::new(RoleId::new(2), e2),
                ],
                1.0,
            );

            g_writer.insert_hyperedge(edge);
            g_writer.insert_edge_direct(e1, e2, 1.0).await.ok();

            if i % 5 == 0 {
                g_writer.compact_async().await.ok();
            }
        }
    });

    let g_reader = graph.clone();
    let reader_handle = tokio::spawn(async move {
        for i in 0..iterations {
            let e1 = EntityId::from(format!("entity_{i}"));
            let hes = g_reader.hyperedges_for_entity(e1);
            for he_id in hes {
                let he = g_reader.get_hyperedge(he_id);
                assert!(
                    he.is_some(),
                    "RCU inconsistency: hyperedge {he_id:?} found in index but missing in snapshot"
                );
                let found_he = he.expect("checked above");
                assert_eq!(found_he.id, he_id);
            }
            tokio::task::yield_now().await;
        }
    });

    let (r1, r2) = tokio::join!(writer_handle, reader_handle);
    assert!(r1.is_ok());
    assert!(r2.is_ok());
}

#[tokio::test]
async fn test_hyperedge_doc_and_entity_rcu_atomicity() {
    use crate::hyperedge::{HyperEdge, HyperEdgeId, RoleBinding, RoleId};

    let graph = std::sync::Arc::new(CsrGraph::new());
    let doc_id = DocId::new(42);
    let e1 = EntityId::from("doc_entity_1");
    let e2 = EntityId::from("doc_entity_2");
    let he_id = HyperEdgeId(999);

    // Before insertion, both lookups return empty
    assert!(graph.hyperedges_for_doc(doc_id).is_empty());
    assert!(graph.hyperedges_for_entity(e1).is_empty());
    assert!(graph.get_hyperedge(he_id).is_none());

    // Insert hyperedge with source_doc_id
    let edge = HyperEdge::new(
        he_id,
        EdgeType::Default,
        vec![
            RoleBinding::new(RoleId::new(1), e1),
            RoleBinding::new(RoleId::new(2), e2),
        ],
        1.0,
    )
    .with_source_doc_id(Some(doc_id));

    graph.insert_hyperedge(edge);

    // Acquire snapshot via inner_read() and verify all 3 indices are atomically populated
    let snapshot = graph.inner_read();
    assert!(snapshot.hyperedges.contains_key(&he_id));
    assert!(snapshot
        .doc_to_hyperedges
        .get(&doc_id)
        .is_some_and(|set| set.contains(&he_id)));
    assert!(snapshot
        .hyperedge_index
        .get(&e1)
        .is_some_and(|set| set.contains(&he_id)));
    assert!(snapshot
        .hyperedge_index
        .get(&e2)
        .is_some_and(|set| set.contains(&he_id)));
    drop(snapshot);

    // Atomically tombstone hyperedge
    let success = graph.tombstone_hyperedge(he_id, TxId::new(10));
    assert!(success);

    // After tombstone, helper getters filter tombstoned hyperedges
    assert!(graph.hyperedges_for_doc(doc_id).is_empty());
    assert!(graph.hyperedges_for_entity(e1).is_empty());
    assert!(graph.get_hyperedge(he_id).is_none());
}
