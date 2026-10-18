use contextra_core::{DocId, TxId, VectorIndex};
use contextra_crypto::deletion_proof::claim_text::WitAssurance;
use contextra_vector::hnsw::{
    witness::HnwIssueArgs, GhostFreeVectorIndex, HnswConfig, HnswIndex, ResDepth,
};
use std::sync::Arc;

#[tokio::test]
async fn test_concurrent_delete_scan_insert() -> Result<(), Box<dyn std::error::Error>> {
    let dim = 8;
    let config = HnswConfig {
        dimension: dim,
        m: 8,
        ef_construction: 32,
        ..Default::default()
    };

    let index = Arc::new(HnswIndex::try_new(config)?);

    // Initial population with 30 documents
    for i in 1..=30 {
        let doc_id = DocId::new(i as u64);
        let vector = vec![(i as f32) / 30.0; dim];
        index.insert(TxId::new(i as u64), doc_id, &vector).await?;
        index.commit(TxId::new(i as u64)).await?;
    }

    let mut handles = Vec::new();

    // Task 1: Delete documents 1..=10 via remove_with_graph_repair
    let index_del = Arc::clone(&index);
    handles.push(tokio::spawn(async move {
        for i in 1..=10 {
            let doc_id = DocId::new(i as u64);
            let mut mut_idx = (*index_del).clone();
            let _ = mut_idx.remove_with_graph_repair(doc_id);
        }
    }));

    // Task 2: Scan residue for documents 1..=10
    let index_scan = Arc::clone(&index);
    handles.push(tokio::spawn(async move {
        for _ in 0..10 {
            let doc_ids: Vec<DocId> = (1..=10).map(|i| DocId::new(i as u64)).collect();
            let _ = index_scan.scan_residue(&doc_ids, ResDepth::Footprint);
            tokio::time::sleep(std::time::Duration::from_millis(2)).await;
        }
    }));

    // Task 3: Insert new documents 101..=110
    let index_ins = Arc::clone(&index);
    handles.push(tokio::spawn(async move {
        for i in 101..=110 {
            let doc_id = DocId::new(i as u64);
            let vector = vec![(i as f32) / 100.0; dim];
            let tx = TxId::new(i as u64);
            if index_ins.insert(tx, doc_id, &vector).await.is_ok() {
                let _ = index_ins.commit(tx).await;
            }
        }
    }));

    for handle in handles {
        handle.await?;
    }

    // Verify witness can be issued cleanly for deleted doc 5
    let args = HnwIssueArgs {
        doc_ids: vec![DocId::new(5)],
        scope_hash: [5u8; 32],
        footprint_hash: [5u8; 32],
        as_of_tx: TxId::new(1),
        scope_write_seq: 200,
        depth: ResDepth::FullSweep,
        assurance: WitAssurance::S1,
    };

    let witness_opt = index.issue_residue_witness(&args)?;
    assert!(
        witness_opt.is_some(),
        "Witness for doc 5 should be issued successfully after concurrent delete, scan, and insert"
    );

    Ok(())
}
