use contextra_graph::csr::CsrGraph;
use contextra_ports::GraphIndex;
use contextra_types::{Edge, Entity, EntityId, TxId};
use std::collections::HashMap;
use std::sync::Arc;

#[tokio::test]
async fn test_rollback_on_uncommitted_guard_drop() {
    let graph = Arc::new(CsrGraph::new());
    let tx1 = TxId::try_new(1).unwrap();

    GraphIndex::add_entity(
        graph.as_ref(),
        tx1,
        Entity::new(EntityId::new(1), "E1", "Type"),
    )
    .await
    .unwrap();
    GraphIndex::commit(graph.as_ref(), tx1).await.unwrap();

    assert_eq!(graph.entity_count(), 1);

    // Staging / mutating write_state without committing
    {
        let mut inner = graph.inner_write();
        inner.get_or_create_index(EntityId::new(2));
        // Guard dropped HERE without calling commit()!
    }

    // After uncommitted drop, write_state must be rolled back and reader snapshot unchanged
    assert_eq!(
        graph.entity_count(),
        1,
        "Entity count in reader snapshot must remain 1 after uncommitted drop"
    );

    // Verify write_state was also reverted and does not contain entity 2
    {
        let inner = graph.inner_write();
        assert!(
            !inner.entity_exists(EntityId::new(2)),
            "write_state must not contain EntityId(2) after rollback"
        );
    }
}

#[tokio::test]
async fn test_rollback_on_panic_during_write() {
    let graph = Arc::new(CsrGraph::new());
    let tx1 = TxId::try_new(1).unwrap();

    GraphIndex::add_entity(
        graph.as_ref(),
        tx1,
        Entity::new(EntityId::new(10), "Node10", "Type"),
    )
    .await
    .unwrap();
    GraphIndex::commit(graph.as_ref(), tx1).await.unwrap();

    let graph_clone = graph.clone();
    let panic_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(move || {
        let mut inner = graph_clone.inner_write();
        inner.get_or_create_index(EntityId::new(999));
        panic!("Simulated write failure!");
    }));

    assert!(panic_result.is_err(), "Panic must be caught");

    // After panic, graph must remain intact and roll back
    assert_eq!(graph.entity_count(), 1);
    let inner = graph.inner_write();
    assert!(
        !inner.entity_exists(EntityId::new(999)),
        "Uncommitted changes from panicked thread must be rolled back"
    );
}

#[tokio::test]
async fn test_compaction_edge_cases() {
    let graph = Arc::new(CsrGraph::new());
    let tx = TxId::try_new(1).unwrap();

    // 1. Empty graph compaction
    graph.compact();
    assert_eq!(graph.edge_count(), 0);

    // 2. Node with degree 0 (isolated) between nodes with edges
    let e1 = EntityId::new(1);
    let e2 = EntityId::new(2); // Isolated node
    let e3 = EntityId::new(3);

    for e in [e1, e2, e3] {
        GraphIndex::add_entity(
            graph.as_ref(),
            tx,
            Entity::new(e, format!("E{}", e.inner()), "Type"),
        )
        .await
        .unwrap();
    }

    // e1 -> e3, self-loop on e3, multi-edge e1 -> e3
    GraphIndex::add_edge(
        graph.as_ref(),
        tx,
        Edge::new(e1, e3, "rel1").with_weight(1.0),
    )
    .await
    .unwrap();
    GraphIndex::add_edge(
        graph.as_ref(),
        tx,
        Edge::new(e1, e3, "rel2").with_weight(2.0),
    )
    .await
    .unwrap();
    GraphIndex::add_edge(
        graph.as_ref(),
        tx,
        Edge::new(e3, e3, "self_loop").with_weight(0.5),
    )
    .await
    .unwrap();
    GraphIndex::commit(graph.as_ref(), tx).await.unwrap();

    // Verify neighbors before compact
    let n1_before = graph.neighbors(e1).await.unwrap();
    let n2_before = graph.neighbors(e2).await.unwrap();
    let n3_before = graph.neighbors(e3).await.unwrap();

    assert_eq!(n1_before, vec![e3]);
    assert!(n2_before.is_empty());
    assert_eq!(n3_before, vec![e3]);

    // Compact
    graph.compact();

    // Verify neighbors after compact
    let n1_after = graph.neighbors(e1).await.unwrap();
    let n2_after = graph.neighbors(e2).await.unwrap();
    let n3_after = graph.neighbors(e3).await.unwrap();

    assert_eq!(n1_after, vec![e3]);
    assert!(n2_after.is_empty());
    assert_eq!(n3_after, vec![e3]);

    // 3. Tombstones only
    let (t_count, _, _) = graph
        .tombstone_edges_direct(&[(e1, e3), (e3, e3)], TxId::try_new(2).unwrap())
        .unwrap();
    assert_eq!(t_count, 2);

    graph.compact();

    let n1_tomb = graph.neighbors(e1).await.unwrap();
    let n3_tomb = graph.neighbors(e3).await.unwrap();
    assert!(n1_tomb.is_empty());
    assert!(n3_tomb.is_empty());
}

#[tokio::test]
async fn test_txid_boundary_traverse_at_time() {
    let graph = Arc::new(CsrGraph::new());
    let tx1 = TxId::try_new(10).unwrap();
    let e1 = EntityId::new(1);
    let e2 = EntityId::new(2);

    GraphIndex::add_entity(graph.as_ref(), tx1, Entity::new(e1, "E1", "Type"))
        .await
        .unwrap();
    GraphIndex::add_entity(graph.as_ref(), tx1, Entity::new(e2, "E2", "Type"))
        .await
        .unwrap();

    let edge = Edge::new(e1, e2, "link").with_validity(
        Some(TxId::try_new(10).unwrap()),
        Some(TxId::try_new(100).unwrap()),
    );
    GraphIndex::add_edge(graph.as_ref(), tx1, edge)
        .await
        .unwrap();
    GraphIndex::commit(graph.as_ref(), tx1).await.unwrap();

    // 1. Valid TxId within MAX_COLLECTION_SEQUENCE range
    let res_at_50 = graph
        .traverse_at_time(e1, 1, TxId::try_new(50).unwrap())
        .await
        .unwrap();
    assert_eq!(res_at_50.len(), 1);
    assert_eq!(res_at_50[0].0, e2);

    // 2. TxId outside valid range (TxId(0) / sentinel / gap value)
    let res_invalid = graph.traverse_at_time(e1, 1, TxId::INVALID).await.unwrap();
    assert!(
        res_invalid.is_empty(),
        "Traversal at TxId::INVALID (0) must return empty results"
    );

    // 3. Internal base TxId
    let internal_tx = TxId::internal();
    let res_internal = graph.traverse_at_time(e1, 1, internal_tx).await.unwrap();
    // Edge valid_to is 100 < INTERNAL_BASE, so not visible at internal_tx
    assert!(res_internal.is_empty());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_concurrent_writer_and_readers_during_compaction() {
    let graph = Arc::new(CsrGraph::new());
    let tx = TxId::try_new(1).unwrap();

    // Setup initial nodes
    for i in 1..=50 {
        GraphIndex::add_entity(
            graph.as_ref(),
            tx,
            Entity::new(EntityId::new(i), format!("N{i}"), "Test"),
        )
        .await
        .unwrap();
    }
    GraphIndex::commit(graph.as_ref(), tx).await.unwrap();

    let running = Arc::new(std::sync::atomic::AtomicBool::new(true));

    // Spawn 4 reader tasks
    let mut handles = Vec::new();
    for _r in 0..4 {
        let g = graph.clone();
        let run = running.clone();
        handles.push(tokio::spawn(async move {
            let mut read_count = 0usize;
            while run.load(std::sync::atomic::Ordering::Relaxed) {
                let start = EntityId::new((read_count % 45 + 1) as u64);
                let _neighbors = g.neighbors(start).await.unwrap();
                read_count += 1;
                tokio::task::yield_now().await;
            }
            read_count
        }));
    }

    // Writer task doing updates and explicit compactions
    for step in 1..=100 {
        let src = EntityId::new((step % 40 + 1) as u64);
        let dst = EntityId::new(((step + 1) % 40 + 1) as u64);
        let step_tx = TxId::try_new(step as u64).unwrap();
        GraphIndex::add_edge(
            graph.as_ref(),
            step_tx,
            Edge::new(src, dst, "link").with_weight(1.0),
        )
        .await
        .unwrap();
        GraphIndex::commit(graph.as_ref(), step_tx).await.unwrap();

        if step % 10 == 0 {
            graph.compact();
        }
        tokio::task::yield_now().await;
    }

    running.store(false, std::sync::atomic::Ordering::Relaxed);

    for h in handles {
        let reads = h.await.unwrap();
        assert!(reads > 0, "Reader must complete reads concurrently");
    }
}

// Proptest for edge multiset oracle
#[test]
fn prop_test_compaction_multiset_oracle() {
    use proptest::prelude::*;

    proptest!(|(num_edges in 1..30usize, num_nodes in 2..10u64)| {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        rt.block_on(async {
            let graph = Arc::new(CsrGraph::new());
            let tx = TxId::try_new(1).unwrap();

            for i in 1..=num_nodes {
                GraphIndex::add_entity(graph.as_ref(), tx, Entity::new(EntityId::new(i), format!("N{i}"), "Type")).await.unwrap();
            }

            let mut expected_counts: HashMap<(EntityId, EntityId), usize> = HashMap::new();

            for e in 0..num_edges {
                let src = EntityId::new((e as u64 % num_nodes) + 1);
                let dst = EntityId::new(((e as u64 + 1) % num_nodes) + 1);

                GraphIndex::add_edge(graph.as_ref(), tx, Edge::new(src, dst, "rel")).await.unwrap();
                *expected_counts.entry((src, dst)).or_default() += 1;
            }

            GraphIndex::commit(graph.as_ref(), tx).await.unwrap();

            // Verify before compaction: track multiset frequencies
            let mut before_counts: HashMap<(EntityId, EntityId), usize> = HashMap::new();
            for src_i in 1..=num_nodes {
                let src = EntityId::new(src_i);
                let neighbors = graph.neighbors(src).await.unwrap();
                for dst in neighbors {
                    *before_counts.entry((src, dst)).or_default() += 1;
                }
            }

            // Compact
            graph.compact();

            // Verify after compaction: track multiset frequencies
            let mut after_counts: HashMap<(EntityId, EntityId), usize> = HashMap::new();
            for src_i in 1..=num_nodes {
                let src = EntityId::new(src_i);
                let neighbors = graph.neighbors(src).await.unwrap();
                for dst in neighbors {
                    *after_counts.entry((src, dst)).or_default() += 1;
                }
            }

            assert_eq!(before_counts, after_counts, "Neighbors multiset must be identical before and after compaction");
        });
    });
}

#[cfg(loom)]
#[test]
fn test_loom_rcu_reader_writer_compaction() {
    loom::model(|| {
        let graph = Arc::new(CsrGraph::new());
        let tx = TxId::try_new(1).unwrap();

        let g1 = graph.clone();
        let handle = loom::thread::spawn(move || {
            let inner = g1.inner_read();
            let _cnt = inner.reverse_map.len();
        });

        let mut inner = graph.inner_write();
        inner.get_or_create_index(EntityId::new(1));
        inner.commit();

        handle.join().unwrap();
    });
}
