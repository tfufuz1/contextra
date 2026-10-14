// FILE-CONTEXT
// STAND: 2026-10-13
// ZWECK: Relations LayerWitness Ausstellung gemäß Garantiemodell (INV-D1/D3).
// INVARIANTEN: Zero-Panic, Fail-Closed bei fehlendem Scan, Überlaufprüfung.

use contextra_crypto::deletion_proof::claim_text::{WitAssurance, WitClaimText};
use contextra_crypto::deletion_proof::evidence::WitLayerEvidence;
use contextra_crypto::deletion_proof::probe::WitStoreGeneration;
use contextra_crypto::deletion_proof::DeletionLayer;
use contextra_crypto::deletion_witness::{LayerWitness, WitIssueArgs};
use contextra_ports::StorageEngine;
use contextra_types::{ContextraError, DocId, Result, TxId};

/// Erzeugt einen [`LayerWitness`] für das Relational/Graph-Subsystem.
///
/// # Fail-Closed
/// Ohne tatsächlichen Scan-Wert (`matching_edges_found_override` ist `None`) wird kein
/// fiktiver Zero-Default verwendet, sondern ein Fehler zurückgegeben.
pub fn issue_relations_witness(
    _doc_id: DocId,
    edges_scanned: u64,
    scope_hash: [u8; 32],
    as_of_tx: TxId,
    matching_edges_found_override: Option<u64>,
) -> Result<LayerWitness> {
    let matching_edges_found = matching_edges_found_override.ok_or_else(|| {
        ContextraError::Storage(
            "Relations witness creation failed: no actual scan value provided (matching_edges_found is None)".to_string(),
        )
    })?;

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
        footprint_hash: [0u8; 32],
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

/// Führt einen realen Scan über das Relational/Graph-Subsystem durch und erzeugt einen [`LayerWitness`].
pub async fn scan_and_issue_relations_witness<S: StorageEngine>(
    storage: &S,
    doc_id: DocId,
    user_id: &str,
    rel_prefix: &[u8],
    scope_hash: [u8; 32],
    as_of_tx: TxId,
) -> Result<LayerWitness> {
    let entries = storage.scan_prefix(rel_prefix).await?;
    let edges_scanned = u64::try_from(entries.len()).map_err(|_| {
        ContextraError::Storage(
            "Overflow converting scanned relation edges count to u64".to_string(),
        )
    })?;

    let user_bytes = user_id.as_bytes();
    let mut matching_edges_found = 0u64;
    for (k, _) in &entries {
        if k.windows(user_bytes.len()).any(|w| w == user_bytes) {
            matching_edges_found = matching_edges_found.checked_add(1).ok_or_else(|| {
                ContextraError::Storage("Overflow counting matching relation edges".to_string())
            })?;
        }
    }

    issue_relations_witness(
        doc_id,
        edges_scanned,
        scope_hash,
        as_of_tx,
        Some(matching_edges_found),
    )
}
