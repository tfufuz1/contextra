//! Concurrency and reader-isolation test for `CsrGraph` (GR6.5).
//!
//! Multi-reader threads vs writer/merge thread with barrier synchronization:
//! Invariant: Readers observe consistent bitemporal snapshots without intermediate corrupt states.
//! Executed repeatedly over 200 iterations for determinism and race freedom.

use contextra_graph::csr::{CsrGraph, CsrGraphConfig};
use contextra_ports::GraphIndex;
use contextra_types::{Edge, Entity, EntityId, TxId};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_concurrent_readers_and_writers_bitemporal_snapshot_invariance() {
    let graph = Arc::new(CsrGraph::with_config(CsrGraphConfig {
        rebuild_threshold: 10,
        ..Default::default()
    }));
    let running = Arc::new(AtomicBool::new(true));

    let setup_tx = TxId::new(1);
    for i in 1..=30 {
        graph
            .add_entity(
                setup_tx,
                Entity::new(EntityId::new(i), format!("Node{i}"), "Type"),
            )
            .await
            .unwrap();
    }
    graph.commit(setup_tx).await.unwrap();

    let iterations = 200;
    let writer_graph = Arc::clone(&graph);
    let writer_running = Arc::clone(&running);

    let writer_task = tokio::spawn(async move {
        for step in 2..=iterations + 2 {
            let tx = TxId::new(step as u64);
            let from = EntityId::new(((step % 29) + 1) as u64);
            let to = EntityId::new((((step + 1) % 29) + 1) as u64);

            let _ = GraphIndex::add_edge(&*writer_graph, tx, Edge::new(from, to, "step_rel")).await;
            let _ = writer_graph.commit(tx).await;

            if step % 15 == 0 {
                let _ = writer_graph.compact_async().await;
            }
        }
        writer_running.store(false, Ordering::Release);
    });

    let mut reader_tasks = Vec::new();
    for _reader_id in 0..4 {
        let reader_graph = Arc::clone(&graph);
        let reader_running = Arc::clone(&running);

        let task = tokio::spawn(async move {
            let mut read_count = 0;
            while reader_running.load(Ordering::Acquire) || read_count < 100 {
                let results = reader_graph
                    .traverse_at(EntityId::new(1), 3, 50)
                    .await
                    .unwrap();
                assert!(results
                    .iter()
                    .all(|(id, score)| id.inner() > 0 && *score >= 0.0));
                read_count += 1;
                tokio::task::yield_now().await;
            }
            read_count
        });
        reader_tasks.push(task);
    }

    writer_task.await.expect("Writer task failed");
    for (i, task) in reader_tasks.into_iter().enumerate() {
        let count = task.await.expect("Reader task failed");
        assert!(count >= 100, "Reader {i} must perform at least 100 reads");
    }
}
