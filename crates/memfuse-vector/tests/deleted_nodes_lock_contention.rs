use memfuse_core::{DocId, TxId, VectorIndex};
use memfuse_vector::hnsw::{HnswConfig, HnswIndex};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

// BEWEIST: [Invariante] 64 parallele Writer-Tasks (Delete & Commit) und 64 Search-Tasks laufen über 5 Sekunden ohne Deadlocks, Race-Conditions oder Task-Panics.
#[tokio::test]
async fn proof_no_deadlock_concurrent_delete_search() {
    let res = tokio::time::timeout(Duration::from_secs(5), async {
        let config = HnswConfig {
            dimension: 16,
            m: 16,
            ef_construction: 64,
            ef_search: 64,
            rebuild_threshold: 0.0, // Disable auto rebuild for isolated testing
            ..Default::default()
        };
        let index = Arc::new(HnswIndex::try_new(config).expect("failed to create hnsw index"));

        // Insert 500 documents
        let tx1 = TxId::new(1);
        for i in 1..=500u64 {
            let vec = vec![(i % 10) as f32; 16];
            index
                .insert(tx1, DocId::new(i), &vec)
                .await
                .expect("insert failed");
        }
        index.commit(tx1).await.expect("commit failed");

        let stop_flag = Arc::new(AtomicBool::new(false));
        let mut handles = Vec::new();

        // 64 Search tasks running concurrently
        for _ in 0..64 {
            let idx = Arc::clone(&index);
            let stop = Arc::clone(&stop_flag);
            handles.push(tokio::spawn(async move {
                let query = vec![1.0f32; 16];
                while !stop.load(Ordering::Relaxed) {
                    let _ = idx.search(&query, 10).await;
                    tokio::task::yield_now().await;
                }
            }));
        }

        // 64 Writer tasks (each deleting 1 unique document and committing)
        for t in 0..64u64 {
            let idx = Arc::clone(&index);
            handles.push(tokio::spawn(async move {
                let doc_id = DocId::new(t + 1);
                let tx = TxId::new(100 + t);
                let _ = idx.delete(tx, doc_id).await;
                let _ = idx.commit(tx).await;
            }));
        }

        // Run workload for 1 second then signal stop
        tokio::time::sleep(Duration::from_secs(1)).await;
        stop_flag.store(true, Ordering::Relaxed);

        for handle in handles {
            let task_res = handle.await;
            assert!(
                task_res.is_ok(),
                "Task panicked or produced JoinError: {:?}",
                task_res
            );
        }
    })
    .await;

    assert!(
        res.is_ok(),
        "proof_no_deadlock_concurrent_delete_search timed out! Potential deadlock or heavy lock contention."
    );
}

// BEWEIST: [Invariante] deleted_nodes.read() wird in search_layer_with_context() einmalig VOR der candidates.pop() Traversal-Schleife gehoistet und nicht pro Kandidat neu akquiriert.
#[test]
fn proof_deleted_nodes_lock_hoisted_before_loop() {
    let source = include_str!("../src/hnsw/mod.rs");

    // Locate search_layer_with_context implementation
    let fn_marker = "fn search_layer_with_context";
    let fn_start = source
        .find(fn_marker)
        .expect("search_layer_with_context function definition must exist in hnsw/mod.rs");

    let fn_body = &source[fn_start..];

    // Find end of function (approximate or next pub fn / fn boundary)
    let loop_marker = "while let Some(Reverse(current)) = candidates.pop()";
    let loop_pos = fn_body
        .find(loop_marker)
        .expect("traversal loop must exist in search_layer_with_context");

    let before_loop = &fn_body[..loop_pos];

    // 1. deleted_nodes.read() MUST occur before the loop
    assert!(
        before_loop.contains("deleted_nodes.read()"),
        "deleted_nodes.read() must be hoisted before the candidates.pop() while loop"
    );

    // 2. deleted_nodes.read() MUST NOT occur inside the loop body
    // Find loop block body
    let loop_body_start = loop_pos;
    // Find next function or 3000 chars down fn_body
    let loop_body_end = (loop_pos + 2500).min(fn_body.len());
    let loop_body = &fn_body[loop_body_start..loop_body_end];

    assert!(
        !loop_body.contains("deleted_nodes.read()"),
        "deleted_nodes.read() re-acquisition found inside candidates.pop() loop! Fix regression detected."
    );
}

// BEWEIST: [Invariante] Soft-Deletes führen nicht zu drastischem Suchlatenz-Overhead (Max. 50% Overhead statt >200% vor dem Lock-Hoisting Fix).
#[tokio::test]
async fn proof_search_throughput_not_degraded_by_deletes() {
    let dim = 32;

    // Index A: 500 vectors, 150 soft-deleted
    let config_a = HnswConfig {
        dimension: dim,
        m: 16,
        ef_construction: 64,
        ef_search: 64,
        rebuild_threshold: 0.0, // Disable auto rebuild to keep deleted nodes present
        ..Default::default()
    };
    let index_a = HnswIndex::try_new(config_a).expect("create index_a");
    let tx1 = TxId::new(1);

    for i in 1..=500u64 {
        let vec = vec![(i as f32) * 0.01; dim];
        index_a
            .insert(tx1, DocId::new(i), &vec)
            .await
            .expect("insert_a");
    }
    index_a.commit(tx1).await.expect("commit_a1");

    // Soft delete 150 vectors
    let tx2 = TxId::new(2);
    for i in 1..=150u64 {
        index_a.delete(tx2, DocId::new(i)).await.expect("delete_a");
    }
    index_a.commit(tx2).await.expect("commit_a2");

    // Index B: 500 fresh vectors (0 deleted)
    let config_b = HnswConfig {
        dimension: dim,
        m: 16,
        ef_construction: 64,
        ef_search: 64,
        rebuild_threshold: 0.0,
        ..Default::default()
    };
    let index_b = HnswIndex::try_new(config_b).expect("create index_b");
    let tx_b = TxId::new(1);

    for i in 1..=500u64 {
        let vec = vec![(i as f32) * 0.01; dim];
        index_b
            .insert(tx_b, DocId::new(i), &vec)
            .await
            .expect("insert_b");
    }
    index_b.commit(tx_b).await.expect("commit_b");

    let query = vec![2.5f32; dim];

    // Measure search latency on Index A (with deletes)
    let start_a = Instant::now();
    for _ in 0..100 {
        let _ = index_a.search(&query, 10).await.expect("search_a");
    }
    let duration_a = start_a.elapsed();

    // Measure search latency on Index B (without deletes)
    let start_b = Instant::now();
    for _ in 0..100 {
        let _ = index_b.search(&query, 10).await.expect("search_b");
    }
    let duration_b = start_b.elapsed();

    let nanos_a = duration_a.as_nanos().max(1) as f64;
    let nanos_b = duration_b.as_nanos().max(1) as f64;

    let overhead_ratio = nanos_a / nanos_b;

    assert!(
        overhead_ratio <= 1.5,
        "Search throughput degraded excessively by soft-deletes: ratio = {:.2} (expected <= 1.5)",
        overhead_ratio
    );
}
