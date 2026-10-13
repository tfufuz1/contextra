// FILE-CONTEXT
// STAND: 2026-10-13
// ZWECK: Relations LayerWitness Ausstellung gemäß Garantiemodell (INV-D1/D3).

use contextra_crypto::deletion_proof::claim_text::{WitAssurance, WitClaimText};
use contextra_crypto::deletion_proof::evidence::WitLayerEvidence;
use contextra_crypto::deletion_proof::probe::WitStoreGeneration;
use contextra_crypto::deletion_proof::DeletionLayer;
use contextra_crypto::deletion_witness::{LayerWitness, WitIssueArgs};
use contextra_types::{ContextraError, DocId, Result, TxId};

/// Erzeugt einen [`LayerWitness`] für das Relational/Graph-Subsystem.
pub fn issue_relations_witness(
    _doc_id: DocId,
    edges_scanned: u64,
    scope_hash: [u8; 32],
    as_of_tx: TxId,
    matching_edges_found_override: Option<u64>,
) -> Result<LayerWitness> {
    let matching_edges_found = matching_edges_found_override.unwrap_or(0);

    let layer_evidence = WitLayerEvidence::Relations {
        edges_scanned,
        matching_edges_found,
    };

    let args = WitIssueArgs {
        layer: DeletionLayer::Relations,
        scope_hash,
        scanned_at_tx: as_of_tx,
        as_of_tx,
        assurance: WitAssurance::S1,
        claim: WitClaimText::S1,
        footprint_hash: scope_hash,
        scope_write_seq: 0,
        scanner_name: "relations_scanner",
        scanner_version: 1,
        store_generation: WitStoreGeneration(0),
        layer_evidence,
        key_identifiers_claimed: true,
    };

    LayerWitness::issue(args)
        .map_err(|e| ContextraError::Storage(format!("Relations witness creation failed: {e}")))
}
