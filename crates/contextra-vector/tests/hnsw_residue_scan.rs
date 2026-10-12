use contextra_core::{DocId, TxId, VectorIndex};
use contextra_vector::hnsw::{
    GhostFreeVectorIndex, HnswConfig, HnswIndex, ResDepth,
};

#[tokio::test]
async fn test_hnsw_residue_scan_clean_positive() -> Result<(), Box<dyn std::error::Error>> {
    let dim = 8;
    let config = HnswConfig {
        dimension: dim,
        m: 8,
        ef_construction: 32,
        ..Default::default()
    };

    let mut index = HnswIndex::try_new(config)?;

    // Insert 20 documents
    for i in 1..=20 {
        let doc_id = DocId::new(i as u64);
        let vector = vec![(i as f32) / 20.0; dim];
        index.insert(TxId::new(i as u64), doc_id, &vector).await?;
        index.commit(TxId::new(i as u64)).await?;
    }

    // Delete doc 10 via remove_with_graph_repair
    let target_doc = DocId::new(10);
    index.remove_with_graph_repair(target_doc)?;

    // Perform residue scan in Footprint and FullSweep mode
    let evidence_fp = index.scan_residue(&[target_doc], ResDepth::Footprint)?;
    let evidence_fs = index.scan_residue(&[target_doc], ResDepth::FullSweep)?;

    assert_eq!(evidence_fp.nodes_checked, 1);
    assert_eq!(evidence_fp.nonzero_vector_copies, 0);
    assert_eq!(evidence_fp.incoming_edges, 0);
    assert!(evidence_fp.entry_point_valid);
    assert!(!evidence_fp.pinned_by_reader);

    assert_eq!(evidence_fs.nodes_checked, 1);
    assert_eq!(evidence_fs.nonzero_vector_copies, 0);
    assert_eq!(evidence_fs.incoming_edges, 0);
    assert!(evidence_fs.entry_point_valid);
    assert!(!evidence_fs.pinned_by_reader);

    Ok(())
}

#[tokio::test]
async fn test_hnsw_residue_scan_after_rebuild_clean() -> Result<(), Box<dyn std::error::Error>> {
    let dim = 8;
    let config = HnswConfig {
        dimension: dim,
        m: 8,
        ef_construction: 32,
        ..Default::default()
    };

    let index = HnswIndex::try_new(config)?;

    // Insert 20 documents
    for i in 1..=20 {
        let doc_id = DocId::new(i as u64);
        let vector = vec![(i as f32) / 20.0; dim];
        index.insert(TxId::new(i as u64), doc_id, &vector).await?;
        index.commit(TxId::new(i as u64)).await?;
    }

    // Delete doc 5 via delete/commit and rebuild
    let del_tx = TxId::new(100);
    let target_doc = DocId::new(5);
    index.delete(del_tx, target_doc).await?;
    index.commit(del_tx).await?;
    index.rebuild().await?;

    let evidence = index.scan_residue(&[target_doc], ResDepth::FullSweep)?;
    assert_eq!(evidence.nonzero_vector_copies, 0);
    assert_eq!(evidence.incoming_edges, 0);
    assert!(evidence.entry_point_valid);
    assert!(!evidence.pinned_by_reader);

    Ok(())
}
