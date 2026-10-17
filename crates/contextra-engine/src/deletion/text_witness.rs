// FILE-CONTEXT
// STAND: 2026-10-13
// ZWECK: TextIndex LayerWitness Ausstellung gemäß Garantiemodell (INV-D1/D3).
// INVARIANTEN: Zero-Panic, Fail-Closed bei fehlendem Scan, Überlaufprüfung.

use contextra_crypto::deletion_proof::claim_text::{WitAssurance, WitClaimText};
use contextra_crypto::deletion_proof::evidence::WitLayerEvidence;
use contextra_crypto::deletion_proof::probe::WitStoreGeneration;
use contextra_crypto::deletion_proof::DeletionLayer;
use contextra_crypto::deletion_witness::{LayerWitness, WitIssueArgs};
use contextra_ports::StorageEngine;
use contextra_types::{ContextraError, DocId, Result, TxId};

/// Erzeugt einen [`LayerWitness`] für das TextIndex-Subsystem.
///
/// # Fail-Closed
/// Ohne tatsächlichen Scan-Wert (`term_doc_hits_override` ist `None`) wird kein
/// fiktiver Zero-Default verwendet, sondern ein Fehler zurückgegeben.
pub fn issue_text_index_witness(
    _doc_id: DocId,
    term_bytes: &[Vec<u8>],
    scope_hash: [u8; 32],
    as_of_tx: TxId,
    term_doc_hits_override: Option<u64>,
) -> Result<LayerWitness> {
    let term_doc_hits = term_doc_hits_override.ok_or_else(|| {
        ContextraError::Storage(
            "TextIndex witness creation failed: no actual scan value provided (term_doc_hits is None)".to_string(),
        )
    })?;

    let postings_lists_scanned = u32::try_from(term_bytes.len()).map_err(|_| {
        ContextraError::Storage("Overflow converting term_bytes length to u32".to_string())
    })?;

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

    LayerWitness::issue(args)
        .map_err(|e| ContextraError::Storage(format!("TextIndex witness creation failed: {e}")))
}

/// Führt einen realen Scan über das TextIndex-Subsystem (InvertedIndex) durch und erzeugt einen [`LayerWitness`].
pub async fn scan_and_issue_text_index_witness<S: StorageEngine>(
    index: &contextra_text::inverted::InvertedIndex<S>,
    doc_id: DocId,
    term_bytes: &[Vec<u8>],
    scope_hash: [u8; 32],
    as_of_tx: TxId,
) -> Result<LayerWitness> {
    let evidence = index.scan_residue(doc_id, term_bytes).await?;
    let total_hits = evidence
        .postings_found
        .checked_add(evidence.positions_found)
        .ok_or_else(|| {
            ContextraError::Storage(
                "Overflow computing term_doc_hits from residue evidence".to_string(),
            )
        })?;

    let term_doc_hits = u64::try_from(total_hits).map_err(|_| {
        ContextraError::Storage("Overflow converting term_doc_hits to u64".to_string())
    })?;

    issue_text_index_witness(
        doc_id,
        term_bytes,
        scope_hash,
        as_of_tx,
        Some(term_doc_hits),
    )
}
