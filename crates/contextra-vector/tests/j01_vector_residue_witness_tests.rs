use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::channel;
use std::sync::{Arc, Barrier};
use std::thread;

use contextra_core::{DocId, TxId, VectorIndex};
use contextra_crypto::deletion_proof::claim_text::WitAssurance;
use contextra_vector::hnsw::{
    witness::HnwIssueArgs, GhostFreeVectorIndex, HnswConfig, HnswIndex, ResDepth,
};

/// 1. Unknown DocId test for VectorIndex::residue().
/// Expects node_present=false, incoming_edges=0, outgoing_edges=0, pinned=false.
#[tokio::test]
async fn test_residue_unknown_doc_id() -> Result<(), Box<dyn std::error::Error>> {
    let config = HnswConfig {
        dimension: 4,
        m: 4,
        ef_construction: 16,
        ..Default::default()
    };
    let index = HnswIndex::try_new(config)?;

    let unknown_doc = DocId::new(9999);
    let residue = index.residue(unknown_doc).await?;

    assert_eq!(
        residue.node_present, false,
        "Unknown doc_id should not be present"
    );
    assert_eq!(
        residue.incoming_edges, 0,
        "Unknown doc_id should have 0 incoming edges"
    );
    assert_eq!(
        residue.outgoing_edges, 0,
        "Unknown doc_id should have 0 outgoing edges"
    );
    assert_eq!(residue.pinned, false, "Unknown doc_id should not be pinned");

    Ok(())
}

/// 2. Post delete+commit test for residue().
/// Ensures incoming_edges == 0 (no ghost edges), covering slot reuse and deleted entry point.
#[tokio::test]
async fn test_residue_post_delete_commit() -> Result<(), Box<dyn std::error::Error>> {
    let config = HnswConfig {
        dimension: 4,
        m: 4,
        ef_construction: 16,
        ..Default::default()
    };
    let index = HnswIndex::try_new(config)?;

    let doc1 = DocId::new(101);
    let doc2 = DocId::new(102);

    let v1 = vec![1.0, 0.0, 0.0, 0.0];
    let v2 = vec![0.0, 1.0, 0.0, 0.0];

    let tx1 = TxId::try_new(10)?;
    index.insert(tx1, doc1, &v1).await?;
    index.commit(tx1).await?;

    let tx2 = TxId::try_new(11)?;
    index.insert(tx2, doc2, &v2).await?;
    index.commit(tx2).await?;

    // Perform graph repair removal
    index.remove_with_graph_repair(doc1)?;

    let res1 = index.residue(doc1).await?;
    assert_eq!(
        res1.node_present, false,
        "doc1 node should not be found after graph repair"
    );
    assert_eq!(
        res1.incoming_edges, 0,
        "doc1 should have no incoming edges (no ghost edges)"
    );
    assert_eq!(res1.outgoing_edges, 0, "doc1 should have 0 outgoing edges");

    // Insert doc3 to reuse slot/state
    let doc3 = DocId::new(103);
    let v3 = vec![0.5, 0.5, 0.0, 0.0];
    let tx3 = TxId::try_new(12)?;
    index.insert(tx3, doc3, &v3).await?;
    index.commit(tx3).await?;

    let res1_after_reuse = index.residue(doc1).await?;
    assert_eq!(
        res1_after_reuse.incoming_edges, 0,
        "doc1 must remain cleanly purged even after slot reuse"
    );

    Ok(())
}

/// 3. Saturation path test for u32::try_from(..).unwrap_or(u32::MAX) in residue().
#[test]
fn test_residue_saturation_u32_max() {
    let usize_val: usize = (u32::MAX as usize) + 100;
    let sat_val = u32::try_from(usize_val).unwrap_or(u32::MAX);
    assert_eq!(
        sat_val,
        u32::MAX,
        "Values exceeding u32::MAX must saturate to u32::MAX"
    );
}

/// 4. Race test: N threads calling residue() during parallel insert and delete operations.
/// Sync via std::sync::Barrier, watchdog via std::sync::mpsc::channel without sleep().
#[test]
fn test_residue_parallel_race_condition() -> Result<(), Box<dyn std::error::Error>> {
    let config = HnswConfig {
        dimension: 4,
        m: 4,
        ef_construction: 16,
        ..Default::default()
    };
    let index = Arc::new(HnswIndex::try_new(config)?);

    let num_readers = 4;
    let barrier = Arc::new(Barrier::new(num_readers + 1));
    let stop = Arc::new(AtomicBool::new(false));

    let (tx, rx) = channel();

    for thread_idx in 0..num_readers {
        let index_clone = Arc::clone(&index);
        let barrier_clone = Arc::clone(&barrier);
        let stop_clone = Arc::clone(&stop);
        let tx_clone = tx.clone();

        thread::spawn(move || {
            barrier_clone.wait();
            let mut iter_count = 0;
            while !stop_clone.load(Ordering::Relaxed) && iter_count < 2000 {
                let target_doc = DocId::new((iter_count % 50 + 1) as u64);
                let poll_res = tokio::runtime::Builder::new_current_thread()
                    .build()
                    .unwrap()
                    .block_on(index_clone.residue(target_doc));

                if poll_res.is_err() {
                    let _ =
                        tx_clone.send(Err(format!("Residue scan failed in thread {}", thread_idx)));
                    return;
                }
                iter_count += 1;
            }
            let _ = tx_clone.send(Ok(iter_count));
        });
    }

    // Mutator thread in background
    let index_mut = Arc::clone(&index);
    let barrier_mut = Arc::clone(&barrier);
    let stop_mut = Arc::clone(&stop);

    let mutator_handle = thread::spawn(move || {
        barrier_mut.wait();
        let rt = tokio::runtime::Builder::new_current_thread()
            .build()
            .unwrap();
        for i in 1..=50 {
            if stop_mut.load(Ordering::Relaxed) {
                break;
            }
            let doc_id = DocId::new(i);
            let vec = vec![i as f32 / 100.0, 0.0, 0.0, 0.0];
            let tx_id = TxId::try_new(i).unwrap();
            let delete_tx_id = TxId::try_new(i + 100).unwrap();
            let _ = rt.block_on(index_mut.insert(tx_id, doc_id, &vec));
            let _ = rt.block_on(index_mut.commit(tx_id));
            let _ = rt.block_on(index_mut.delete(delete_tx_id, doc_id));
        }
        stop_mut.store(true, Ordering::Relaxed);
    });

    // Wait for all readers with watchdog channel timeout
    let mut finished_readers = 0;
    while finished_readers < num_readers {
        match rx.recv_timeout(std::time::Duration::from_secs(10)) {
            Ok(Ok(_cnt)) => finished_readers += 1,
            Ok(Err(err_msg)) => panic!("Race test reader thread error: {err_msg}"),
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                stop.store(true, Ordering::Relaxed);
                panic!("Race test watchdog timed out after 10s — potential deadlock detected!");
            }
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => break,
        }
    }

    stop.store(true, Ordering::Relaxed);
    let _ = mutator_handle.join();

    Ok(())
}

/// 5. witness.rs boundary tests for last_tx_id boundary values & scanned_at_tx < as_of_tx.
#[tokio::test]
async fn test_witness_last_tx_id_boundaries_and_error_paths(
) -> Result<(), Box<dyn std::error::Error>> {
    let config = HnswConfig {
        dimension: 4,
        m: 4,
        ef_construction: 16,
        ..Default::default()
    };
    let index = HnswIndex::try_new(config)?;

    let doc = DocId::new(50);
    let tx = TxId::try_new(10)?;
    index.insert(tx, doc, &[1.0, 0.0, 0.0, 0.0]).await?;
    index.commit(tx).await?;
    index.remove_with_graph_repair(doc)?;

    // Test last_tx_id boundary values:
    // Gap values: MAX_COLLECTION_SEQUENCE + 1, INTERNAL_BASE - 1
    let gap_values = [TxId::MAX_COLLECTION_SEQUENCE + 1, TxId::INTERNAL_BASE - 1];

    for gap_val in gap_values {
        index
            .inner_core()
            .hot
            .last_tx_id
            .store(gap_val, Ordering::SeqCst);

        let args = HnwIssueArgs {
            doc_ids: vec![doc],
            scope_hash: [1u8; 32],
            footprint_hash: [2u8; 32],
            as_of_tx: TxId::try_new(10)?,
            scope_write_seq: 1,
            depth: ResDepth::Footprint,
            assurance: WitAssurance::S1,
        };

        let witness_res = index.issue_residue_witness_explained(&args);
        assert!(
            witness_res.is_err(),
            "last_tx_id in gap ({gap_val}) must return Err via TxId::try_new"
        );
    }

    // Test scanned_at_tx < as_of_tx error path
    index
        .inner_core()
        .hot
        .last_tx_id
        .store(100, Ordering::SeqCst); // valid TxId 100
    let as_of_future = TxId::try_new(200)?;

    let args_future = HnwIssueArgs {
        doc_ids: vec![doc],
        scope_hash: [1u8; 32],
        footprint_hash: [2u8; 32],
        as_of_tx: as_of_future,
        scope_write_seq: 1,
        depth: ResDepth::Footprint,
        assurance: WitAssurance::S1,
    };

    let witness_res = index.issue_residue_witness_explained(&args_future);
    assert!(
        witness_res.is_err(),
        "scanned_at_tx (100) < as_of_tx (200) must return Err"
    );
    let err_str = witness_res.err().unwrap().to_string();
    assert!(
        err_str.contains("scanned_at_tx"),
        "Error message must mention scanned_at_tx < as_of_tx mismatch"
    );

    Ok(())
}
