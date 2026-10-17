use contextra_core::{DocId, TxId, VectorIndex};
use contextra_crypto::deletion_proof::{claim_text::WitAssurance, DeletionLayer};
use contextra_vector::hnsw::{
    witness::{HnwIssueArgs, HnwNoClaim},
    GhostFreeVectorIndex, HnswConfig, HnswIndex, ResDepth, VectorData,
};

#[tokio::test]
async fn test_witness_matrix_injection_and_positive() -> Result<(), Box<dyn std::error::Error>> {
    let dim = 8;
    let config = HnswConfig {
        dimension: dim,
        m: 8,
        ef_construction: 32,
        ..Default::default()
    };

    let index = HnswIndex::try_new(config.clone())?;

    // Insert 10 documents
    for i in 1..=10 {
        let doc_id = DocId::new(i as u64);
        let vector = vec![(i as f32) / 10.0; dim];
        index.insert(TxId::new(i as u64), doc_id, &vector).await?;
        index.commit(TxId::new(i as u64)).await?;
    }

    let args = HnwIssueArgs {
        doc_ids: vec![DocId::new(3)],
        scope_hash: [1u8; 32],
        footprint_hash: [2u8; 32],
        as_of_tx: TxId::new(1),
        scope_write_seq: 100,
        depth: ResDepth::FullSweep,
        assurance: WitAssurance::S1,
    };

    // 1. Injektion T2 Zeile 17: F32 Vektor nicht genullt
    // Simuliere Soft-Delete ohne Nullung durch manuelles Setzen in deleted_nodes
    index.inner_core().cold.deleted_nodes.write().insert(3);
    let witness_res = index.issue_residue_witness(&args);
    assert!(
        witness_res.is_err(),
        "Erwartet: Fehler bei F32 Vektor nicht genullt"
    );

    // 2. Injektion T2 Zeile 18: U8/Code-Kopie nicht genullt
    let sq8_config = HnswConfig {
        dimension: dim,
        quantize: true,
        m: 8,
        ef_construction: 32,
        ..Default::default()
    };
    let sq8_index = HnswIndex::try_new(sq8_config)?;
    for i in 1..=10 {
        let doc_id = DocId::new(i as u64);
        let vector = vec![(i as f32) / 10.0; dim];
        sq8_index.insert(TxId::new(i as u64), doc_id, &vector).await?;
        sq8_index.commit(TxId::new(i as u64)).await?;
    }
    sq8_index.inner_core().cold.deleted_nodes.write().insert(3);
    let witness_sq8_res = sq8_index.issue_residue_witness(&args);
    assert!(
        witness_sq8_res.is_err(),
        "Erwartet: Fehler bei U8 Vektor nicht genullt"
    );

    // 3. Injektion T2 Zeile 19: Eingangskante bleibt
    let clean_index = HnswIndex::try_new(config.clone())?;
    for i in 1..=10 {
        let doc_id = DocId::new(i as u64);
        let vector = vec![(i as f32) / 10.0; dim];
        clean_index.insert(TxId::new(i as u64), doc_id, &vector).await?;
        clean_index.commit(TxId::new(i as u64)).await?;
    }
    clean_index.set_ram_vector_at_for_test(2, VectorData::F32(vec![0.0; dim]));
    clean_index.inner_core().cold.deleted_nodes.write().insert(2); // node idx 2 = doc 3
    let witness_ghost_res = clean_index.issue_residue_witness(&args);
    assert!(
        witness_ghost_res.is_err(),
        "Erwartet: Fehler bei verbleibender Eingangskante"
    );

    // 4. Injektion T2 Zeile 20: Entry point = gelöschter Knoten
    clean_index.inner_core().hot.set_entry_point(Some(2));
    let witness_ep_res = clean_index.issue_residue_witness(&args);
    assert!(
        witness_ep_res.is_err(),
        "Erwartet: Fehler bei Entry Point = gelöschter Knoten"
    );

    // 5. Injektion T2 Zeile 21/22: Reader pin
    clean_index.pin_snapshot(50);
    let witness_pin_res = clean_index.issue_residue_witness_explained(&args)?;
    assert_eq!(
        witness_pin_res,
        Err(HnwNoClaim::PinnedByReader),
        "Erwartet: Ok(Err(HnwNoClaim::PinnedByReader))"
    );

    // 6. Sauber-Positivtest nach regulärer Löschung mit FullSweep
    let mut pos_index = HnswIndex::try_new(config)?;
    for i in 1..=10 {
        let doc_id = DocId::new(i as u64);
        let vector = vec![(i as f32) / 10.0; dim];
        pos_index.insert(TxId::new(i as u64), doc_id, &vector).await?;
        pos_index.commit(TxId::new(i as u64)).await?;
    }

    let del_target = DocId::new(3);
    pos_index.remove_with_graph_repair(del_target)?;

    let pos_args = HnwIssueArgs {
        doc_ids: vec![del_target],
        scope_hash: [1u8; 32],
        footprint_hash: [2u8; 32],
        as_of_tx: TxId::new(1),
        scope_write_seq: 100,
        depth: ResDepth::FullSweep,
        assurance: WitAssurance::S1,
    };

    let witness_pos = pos_index.issue_residue_witness(&pos_args)?;
    assert!(
        witness_pos.is_some(),
        "Erwartet: Erfolgreich ausgestellter Witness nach regulärer Löschung mit FullSweep"
    );

    let witness = witness_pos.unwrap();
    assert_eq!(witness.layer(), &DeletionLayer::HnswIndex);
    assert_eq!(witness.scope_hash(), &[1u8; 32]);

    Ok(())
}
