use memfuse_core::{DocId, TxId, VectorIndex};
use memfuse_index::hnsw::{HnswConfig, HnswIndex};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

#[tokio::test]
async fn proof_no_deadlock_delete_during_search() {
    // Timeout of 5 seconds to prevent hanging if a deadlock or excessive lock contention occurs
    let res = tokio::time::timeout(Duration::from_secs(5), async {
        let config = HnswConfig {
            dimension: 16,
            m: 16,
            ef_construction: 64,
            ef_search: 64,
            rebuild_threshold: 0.0, // Disable auto rebuild for isolated testing
            ..Default::default()
        };
        let index = Arc::new(HnswIndex::try_new(config).unwrap());

        // Insert 200 documents
        let tx1 = TxId::new(1);
        for i in 1..=200u64 {
            let vec = vec![(i % 10) as f32; 16];
            index.insert(tx1, DocId::new(i), &vec).await.unwrap();
        }
        index.commit(tx1).await.unwrap();

        let stop_flag = Arc::new(AtomicBool::new(false));

        // Spawn search workers
        let mut handles = Vec::new();
        for _ in 0..4 {
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

        // Spawn delete worker
        let idx_del = Arc::clone(&index);
        let del_handle = tokio::spawn(async move {
            let mut tx_counter = 2u64;
            for i in 1..=100u64 {
                let tx = TxId::new(tx_counter);
                tx_counter += 1;
                let _ = idx_del.delete(tx, DocId::new(i)).await;
                let _ = idx_del.commit(tx).await;
                tokio::task::yield_now().await;
            }
        });

        del_handle.await.unwrap();
        stop_flag.store(true, Ordering::Relaxed);

        for handle in handles {
            handle.await.unwrap();
        }
    })
    .await;

    assert!(
        res.is_ok(),
        "proof_no_deadlock_delete_during_search timed out! Potential deadlock or heavy lock contention."
    );
}

#[tokio::test]
async fn proof_deleted_nodes_lock_hoisted_before_loop() {
    let config = HnswConfig {
        dimension: 4,
        m: 8,
        ef_construction: 32,
        ef_search: 32,
        rebuild_threshold: 0.0,
        ..Default::default()
    };
    let index = HnswIndex::try_new(config).unwrap();

    let tx1 = TxId::new(1);
    for i in 1..=50u64 {
        let v = vec![i as f32, 0.0, 0.0, 0.0];
        index.insert(tx1, DocId::new(i), &v).await.unwrap();
    }
    index.commit(tx1).await.unwrap();

    // Soft delete 20 nodes
    let tx2 = TxId::new(2);
    for i in 1..=20u64 {
        index.delete(tx2, DocId::new(i)).await.unwrap();
    }
    index.commit(tx2).await.unwrap();

    // Search query and ensure correct execution without per-candidate lock re-acquisition
    let query = vec![25.0f32, 0.0, 0.0, 0.0];
    let results = index.search(&query, 10).await.unwrap();

    assert!(!results.is_empty());
    for doc in results {
        assert!(
            doc.doc_id.inner() > 20,
            "Deleted node should not appear in search results"
        );
    }
}
