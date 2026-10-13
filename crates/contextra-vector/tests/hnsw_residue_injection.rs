use contextra_core::{DocId, TxId, VectorIndex};
use contextra_vector::hnsw::{GhostFreeVectorIndex, HnswConfig, HnswIndex, ResDepth, VectorData};

#[tokio::test]
async fn test_injected_f32_nonzero_vector_detected() -> Result<(), Box<dyn std::error::Error>> {
    let dim = 8;
    let config = HnswConfig {
        dimension: dim,
        m: 8,
        ef_construction: 32,
        ..Default::default()
    };

    let mut index = HnswIndex::try_new(config)?;

    for i in 1..=10 {
        let doc_id = DocId::new(i as u64);
        let vector = vec![(i as f32) / 10.0; dim];
        index.insert(TxId::new(i as u64), doc_id, &vector).await?;
        index.commit(TxId::new(i as u64)).await?;
    }

    let target_doc = DocId::new(3);
    let target_slot = 2usize; // doc 3 is RAM index 2

    index.remove_with_graph_repair(target_doc)?;

    // Clean check
    let clean_evidence = index.scan_residue(&[target_doc], ResDepth::Footprint)?;
    assert_eq!(clean_evidence.nonzero_vector_copies, 0);

    // Fault injection: Mutate vector back to non-zero using test helper
    index.set_ram_vector_at_for_test(target_slot, VectorData::F32(vec![0.5f32; dim]));

    let evidence = index.scan_residue(&[target_doc], ResDepth::Footprint)?;
    assert!(
        evidence.nonzero_vector_copies > 0,
        "Scanner MUST detect non-zero F32 vector residue"
    );

    Ok(())
}

#[tokio::test]
async fn test_injected_u8_nonzero_vector_detected() -> Result<(), Box<dyn std::error::Error>> {
    let dim = 8;
    let config = HnswConfig {
        dimension: dim,
        m: 8,
        ef_construction: 32,
        quantize: true,
        ..Default::default()
    };

    let mut index = HnswIndex::try_new(config)?;

    for i in 1..=10 {
        let doc_id = DocId::new(i as u64);
        let vector = vec![(i as f32) / 10.0; dim];
        index.insert(TxId::new(i as u64), doc_id, &vector).await?;
        index.commit(TxId::new(i as u64)).await?;
    }

    let target_doc = DocId::new(4);
    let target_slot = 3usize;

    index.remove_with_graph_repair(target_doc)?;

    // Fault injection: Mutate U8 vector to non-zero bytes using test helper
    index.set_ram_vector_at_for_test(target_slot, VectorData::U8(vec![128u8; dim]));

    let evidence = index.scan_residue(&[target_doc], ResDepth::Footprint)?;
    assert!(
        evidence.nonzero_vector_copies > 0,
        "Scanner MUST detect non-zero U8 vector residue"
    );

    Ok(())
}

#[tokio::test]
async fn test_injected_incoming_edge_detected() -> Result<(), Box<dyn std::error::Error>> {
    let dim = 8;
    let m = 8;
    let config = HnswConfig {
        dimension: dim,
        m,
        ef_construction: 32,
        ..Default::default()
    };

    let mut index = HnswIndex::try_new(config)?;

    for i in 1..=10 {
        let doc_id = DocId::new(i as u64);
        let vector = vec![(i as f32) / 10.0; dim];
        index.insert(TxId::new(i as u64), doc_id, &vector).await?;
        index.commit(TxId::new(i as u64)).await?;
    }

    let target_doc = DocId::new(1);
    let target_slot_u32 = 0u32;

    index.remove_with_graph_repair(target_doc)?;

    // Clean check
    let clean = index.scan_residue(&[target_doc], ResDepth::FullSweep)?;
    assert_eq!(clean.incoming_edges, 0);

    // Fault injection: Inject incoming backlink at node index 5 pointing to target_slot 0
    let mut conns = index
        .inner_core()
        .hot
        .arena
        .get_ram_node_connections(5, 0, m);
    if !conns.contains(&target_slot_u32) {
        conns.push(target_slot_u32);
    }
    index
        .inner_core()
        .hot
        .arena
        .update_backlink(5, 0, m, &conns)?;

    let evidence_fp = index.scan_residue(&[target_doc], ResDepth::Footprint)?;
    let evidence_fs = index.scan_residue(&[target_doc], ResDepth::FullSweep)?;

    assert!(
        evidence_fp.incoming_edges > 0 || evidence_fs.incoming_edges > 0,
        "Scanner MUST detect residual incoming edge"
    );

    Ok(())
}

#[tokio::test]
async fn test_injected_invalid_entry_point_detected() -> Result<(), Box<dyn std::error::Error>> {
    let dim = 8;
    let config = HnswConfig {
        dimension: dim,
        m: 8,
        ef_construction: 32,
        ..Default::default()
    };

    let mut index = HnswIndex::try_new(config)?;

    for i in 1..=10 {
        let doc_id = DocId::new(i as u64);
        let vector = vec![(i as f32) / 10.0; dim];
        index.insert(TxId::new(i as u64), doc_id, &vector).await?;
        index.commit(TxId::new(i as u64)).await?;
    }

    let target_doc = DocId::new(2);
    let target_slot = 1usize;

    index.remove_with_graph_repair(target_doc)?;

    // Fault injection: Set global entry point to deleted node index (1)
    index.inner_core().hot.set_entry_point(Some(target_slot));

    let evidence = index.scan_residue(&[target_doc], ResDepth::Footprint)?;
    assert!(
        !evidence.entry_point_valid,
        "Scanner MUST detect invalid entry point pointing to deleted node"
    );

    Ok(())
}

#[tokio::test]
async fn test_pinned_reader_snapshot_detected() -> Result<(), Box<dyn std::error::Error>> {
    let dim = 8;
    let config = HnswConfig {
        dimension: dim,
        m: 8,
        ef_construction: 32,
        ..Default::default()
    };

    let index = HnswIndex::try_new(config)?;

    let doc1 = DocId::new(1);
    index.insert(TxId::new(1), doc1, &vec![0.1f32; dim]).await?;
    index.commit(TxId::new(1)).await?;

    // Pin snapshot
    index.pin_snapshot(1);

    let evidence = index.scan_residue(&[doc1], ResDepth::Footprint)?;
    assert!(
        evidence.pinned_by_reader,
        "Scanner MUST detect pinned reader snapshot"
    );

    index.unpin_snapshot(1);

    Ok(())
}

#[tokio::test]
async fn test_injected_slot_reuse_ghost_edge_detected() -> Result<(), Box<dyn std::error::Error>> {
    let dim = 8;
    let m = 8;
    let config = HnswConfig {
        dimension: dim,
        m,
        ef_construction: 32,
        ..Default::default()
    };

    let mut index = HnswIndex::try_new(config)?;

    for i in 1..=10 {
        let doc_id = DocId::new(i as u64);
        let vector = vec![(i as f32) / 10.0; dim];
        index.insert(TxId::new(i as u64), doc_id, &vector).await?;
        index.commit(TxId::new(i as u64)).await?;
    }

    let target_doc = DocId::new(5);
    let target_slot = 4usize;

    index.remove_with_graph_repair(target_doc)?;

    // Reuse slot for doc 99
    let new_doc = DocId::new(99);
    index
        .insert(TxId::new(99), new_doc, &vec![0.5f32; dim])
        .await?;
    index.commit(TxId::new(99)).await?;

    // Inject ghost edge pointing to slot 4 from node 2
    let mut conns = index
        .inner_core()
        .hot
        .arena
        .get_ram_node_connections(2, 0, m);
    if !conns.contains(&(target_slot as u32)) {
        conns.push(target_slot as u32);
    }
    index
        .inner_core()
        .hot
        .arena
        .update_backlink(2, 0, m, &conns)?;

    let evidence = index.scan_residue(&[new_doc], ResDepth::FullSweep)?;
    assert!(
        evidence.incoming_edges > 0,
        "Scanner MUST detect ghost edge on reused slot"
    );

    Ok(())
}
