// FILE-CONTEXT
// STAND: 2026-10-06
// ZWECK: Complete verification tests for P03/F-02 HNSW ghost pointer verification.
// INVARIANTEN: INV-DELETION-2: After Ok(..) of remove_with_graph_repair(doc_id), no neighborhood pointer points to doc_id.

use contextra_core::{DocId, Result, TxId, VectorIndex};
use contextra_vector::hnsw::{GhostFreeVectorIndex, HnswConfig, HnswIndex};

#[tokio::test]
async fn test_normal_deletion_complete_verification() -> Result<()> {
    let dimension = 8;
    let config = HnswConfig {
        dimension,
        m: 8,
        ef_construction: 32,
        ..Default::default()
    };

    let index = HnswIndex::try_new(config)?;

    // Insert 100 documents
    for i in 1..=100 {
        let doc_id = DocId::new(i as u64);
        let vector = vec![(i as f32) / 100.0; dimension];
        index.insert(TxId::new(i as u64), doc_id, &vector).await?;
        index.commit(TxId::new(i as u64)).await?;
    }

    let mut mut_index = index;
    let target_doc = DocId::new(1);
    let stats = mut_index.remove_with_graph_repair(target_doc)?;

    assert_eq!(stats.doc_id, target_doc);
    assert!(
        stats.verified_no_ghost_pointers,
        "verified_no_ghost_pointers MUST be true after successful repair"
    );

    // Verify search across index never returns target_doc
    let query_vector = vec![0.01f32; dimension];
    let search_results = mut_index.search(&query_vector, 50).await?;
    for res in search_results {
        assert_ne!(
            res.doc_id, target_doc,
            "Deleted document MUST NOT be returned in search results"
        );
    }

    Ok(())
}

#[tokio::test]
async fn test_verify_no_ghost_pointers_standalone_complete_scan() -> Result<()> {
    let dimension = 8;
    let m = 8;
    let config = HnswConfig {
        dimension,
        m,
        ef_construction: 32,
        ..Default::default()
    };

    let index = HnswIndex::try_new(config)?;

    for i in 1..=50 {
        let doc_id = DocId::new(i as u64);
        let vector = vec![(i as f32) / 50.0; dimension];
        index.insert(TxId::new(i as u64), doc_id, &vector).await?;
        index.commit(TxId::new(i as u64)).await?;
    }

    // Call verify_no_ghost_pointers with budget = 5 and targeted_nodes = None
    // This triggers full fallback scan over all 50 nodes.
    let verif_res = index.verify_no_ghost_pointers(0, 5, None);

    assert!(
        verif_res.is_complete,
        "Fallback scan over all nodes MUST be complete"
    );
    assert_eq!(
        verif_res.remaining_ghost_pointers, 0,
        "Fresh index must have 0 residual ghost pointers to node 0"
    );

    Ok(())
}

#[tokio::test]
async fn test_multiple_deletions_verification_integrity() -> Result<()> {
    let dimension = 8;
    let config = HnswConfig {
        dimension,
        m: 8,
        ef_construction: 32,
        ..Default::default()
    };

    let index = HnswIndex::try_new(config)?;

    for i in 1..=30 {
        let doc_id = DocId::new(i as u64);
        let vector = vec![(i as f32) / 30.0; dimension];
        index.insert(TxId::new(i as u64), doc_id, &vector).await?;
        index.commit(TxId::new(i as u64)).await?;
    }

    let mut mut_index = index;

    // Delete multiple documents sequentially
    for del_id in [5u64, 12u64, 25u64] {
        let doc_id = DocId::new(del_id);
        let stats = mut_index.remove_with_graph_repair(doc_id)?;
        assert_eq!(stats.doc_id, doc_id);
        assert!(stats.verified_no_ghost_pointers);
    }

    // Direct standalone verification check over remaining graph
    let verif = mut_index.verify_no_ghost_pointers(0, 100, None);
    assert!(verif.is_complete);
    assert_eq!(verif.remaining_ghost_pointers, 0);

    Ok(())
}
