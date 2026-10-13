// FILE-CONTEXT
// STAND: 2026-10-13
// ZWECK: TextIndex LayerWitness Ausstellung gemäß Garantiemodell (INV-D1/D3).

use contextra_crypto::deletion_proof::claim_text::{WitAssurance, WitClaimText};
use contextra_crypto::deletion_proof::evidence::WitLayerEvidence;
use contextra_crypto::deletion_proof::probe::WitStoreGeneration;
use contextra_crypto::deletion_proof::DeletionLayer;
use contextra_crypto::deletion_witness::{LayerWitness, WitIssueArgs};
use contextra_types::{ContextraError, DocId, Result, TxId};

/// Erzeugt einen [`LayerWitness`] für das TextIndex-Subsystem.
pub fn issue_text_index_witness(
    _doc_id: DocId,
    term_bytes: &[Vec<u8>],
    scope_hash: [u8; 32],
    as_of_tx: TxId,
    term_doc_hits_override: Option<u64>,
) -> Result<LayerWitness> {
    let term_doc_hits = term_doc_hits_override.unwrap_or(0);
    let postings_lists_scanned = u32::try_from(term_bytes.len()).unwrap_or(u32::MAX);

    let layer_evidence = WitLayerEvidence::TextIndex {
        postings_lists_scanned,
        term_doc_hits,
    };

    let args = WitIssueArgs {
        layer: DeletionLayer::TextIndex,
        scope_hash,
        scanned_at_tx: as_of_tx,
        as_of_tx,
        assurance: WitAssurance::S1,
        claim: WitClaimText::S1,
        footprint_hash: [0u8; 32],
        scope_write_seq: 0,
        scanner_name: "text_index_scanner",
        scanner_version: 1,
        store_generation: WitStoreGeneration(0),
        layer_evidence,
        key_identifiers_claimed: true,
    };

    LayerWitness::issue(args).map_err(|e| ContextraError::Storage(format!("TextIndex witness creation failed: {e}")))
}
