// FILE-CONTEXT
// STAND: 2026-10-13
// ZWECK: Integrationstest für HNSW-Zeroisierung, Canary-Analyse, deferred-State bei Reader-Pin und Rückstandsfreiheit nach Rebuild.
// INVARIANTEN: Zero-Panic, VETO-F02 Einhaltung, INV-DELETION-2.

use contextra_types::{DocId, Result};
use contextra_crypto::deletion_proof::claim_text::WitAssurance;
use contextra_ports::VectorIndex;
use contextra_types::TxId;
use contextra_vector::hnsw::residue_scan::ResDepth;
use contextra_vector::hnsw::witness::{HnwIssueArgs, HnwNoClaim};
use contextra_vector::{HnswConfig, HnswIndex};

#[tokio::test]
async fn test_hnsw_zeroize_canary_and_deferred_pin_behavior() -> Result<()> {
    let dim = 32;
    let config = HnswConfig {
        dimension: dim,
        m: 16,
        ef_construction: 32,
        ef_search: 16,
        rebuild_threshold: 0.1,
        quantize: false,
        ..Default::default()
    };

    let index = HnswIndex::try_new(config)?;

    // 1. Insert vector canary
    let canary_val = 13.37f32;
    let canary_vec = vec![canary_val; dim];
    let doc_id1 = DocId::new(101);
    let doc_id2 = DocId::new(102);

    let tx1 = TxId::try_new(1)
        .map_err(|e| contextra_types::ContextraError::InvalidInput(e.to_string()))?;
    let tx2 = TxId::try_new(2)
        .map_err(|e| contextra_types::ContextraError::InvalidInput(e.to_string()))?;
    let tx3 = TxId::try_new(3)
        .map_err(|e| contextra_types::ContextraError::InvalidInput(e.to_string()))?;

    index.insert(tx1, doc_id1, &canary_vec).await?;
    index.insert(tx2, doc_id2, &vec![1.0; dim]).await?;
    index.commit(tx2).await?;

    // 2. Pin snapshot (simulate active snapshot reader pin at seqno 1)
    index.pin_snapshot(1);

    // 3. Delete doc_id1 while snapshot is pinned
    index.delete(tx3, doc_id1).await?;
    index.commit(tx3).await?;

    // 4. Verify that residue witness issuance fails with PinnedByReader (deferred zeroization)
    let issue_args = HnwIssueArgs {
        doc_ids: vec![doc_id1],
        scope_hash: [1u8; 32],
        footprint_hash: [2u8; 32],
        as_of_tx: tx3,
        scope_write_seq: 3,
        depth: ResDepth::FullSweep,
        assurance: WitAssurance::S1,
    };

    let witness_res = index.issue_residue_witness_explained(&issue_args)?;
    assert_eq!(
        witness_res.err(),
        Some(HnwNoClaim::PinnedByReader),
        "Deletion must be marked as deferred while reader snapshot is pinned"
    );

    // 5. Unpin snapshot and perform index rebuild
    index.unpin_snapshot(1);
    index.rebuild().await?;

    // 6. Verify that canary is completely removed and zeroed post-rebuild
    let evidence = index.scan_residue(&[doc_id1], ResDepth::FullSweep)?;
    assert_eq!(
        evidence.nonzero_vector_copies, 0,
        "No non-zero vector canary copies should remain after rebuild"
    );

    // 7. Witness issuance must now succeed after unpin and rebuild
    let witness_post = index.issue_residue_witness_explained(&issue_args)?;
    assert!(
        witness_post.is_ok(),
        "Residue witness issuance must succeed once reader unpins and rebuild completes"
    );

    Ok(())
}
