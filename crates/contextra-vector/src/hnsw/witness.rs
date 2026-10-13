// FILE-CONTEXT
// ZWECK: Witness-Ausstellung für HNSW Vektorindex gemäß Spezifikation C4.4.
// INVARIANTEN: Zero-Panic. Unfälschbare Witness-Ausstellung über LayerWitness::issue.

use contextra_core::{ContextraError, DocId, Result};
use contextra_crypto::deletion_proof::{
    claim_text::{WitAssurance, WitClaimText},
    evidence::WitLayerEvidence,
    probe::WitStoreGeneration,
    DeletionLayer,
};
use contextra_crypto::deletion_witness::{LayerWitness, WitIssueArgs};

use super::residue_scan::ResDepth;
use super::types::HnswIndex;

/// Grund für das Nicht-Ausstellen eines Witness (Err-freier Pfad).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HnwNoClaim {
    /// Knoten ist durch einen aktiven Reader gepinnt.
    PinnedByReader,
    /// Eine persistierte HNSW-Mmap-Datei ist vorhanden (AP-07).
    PersistedSnapshotPresent,
    /// Keine doc_ids übergeben.
    EmptyScope,
}

impl std::fmt::Display for HnwNoClaim {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HnwNoClaim::PinnedByReader => write!(f, "Pinned by active reader"),
            HnwNoClaim::PersistedSnapshotPresent => write!(f, "Persisted mmap snapshot present"),
            HnwNoClaim::EmptyScope => write!(f, "Empty doc_ids scope"),
        }
    }
}

/// Argumente für die Ausstellung eines HNSW LayerWitness.
#[derive(Debug, Clone)]
pub struct HnwIssueArgs {
    pub doc_ids: Vec<DocId>,
    pub scope_hash: [u8; 32],
    pub footprint_hash: [u8; 32],
    pub as_of_tx: contextra_types::TxId,
    pub scope_write_seq: u64,
    pub depth: ResDepth,
    pub assurance: WitAssurance,
}

impl HnswIndex {
    /// Rufen `scan_residue` ab und stellt bei Erfolg einen [`LayerWitness`] aus.
    ///
    /// Gibt `Ok(None)` zurück, wenn der Witness aufgrund eines aktiven Reader-Pins,
    /// einer persistierten Mmap-Datei oder eines leeren Scopes nicht ausgestellt werden kann.
    pub fn issue_residue_witness(&self, args: &HnwIssueArgs) -> Result<Option<LayerWitness>> {
        match self.issue_residue_witness_explained(args)? {
            Ok(witness) => Ok(Some(witness)),
            Err(_) => Ok(None),
        }
    }

    /// Detaillierte Version von `issue_residue_witness`, die im Fall des Nicht-Ausstellens
    /// das spezifische `HnwNoClaim` zurückgibt.
    pub fn issue_residue_witness_explained(
        &self,
        args: &HnwIssueArgs,
    ) -> Result<std::result::Result<LayerWitness, HnwNoClaim>> {
        if args.doc_ids.is_empty() {
            return Ok(Err(HnwNoClaim::EmptyScope));
        }

        // 1. First scan pass (Footprint)
        let evidence1 = self.scan_residue(&args.doc_ids, ResDepth::Footprint)?;

        if evidence1.pinned_by_reader {
            return Ok(Err(HnwNoClaim::PinnedByReader));
        }

        if evidence1.persisted_snapshots_present {
            return Ok(Err(HnwNoClaim::PersistedSnapshotPresent));
        }

        // 2. Second scan pass if depth is FullSweep
        let final_evidence = if args.depth == ResDepth::FullSweep {
            let evidence2 = self.scan_residue(&args.doc_ids, ResDepth::FullSweep)?;
            if evidence2.pinned_by_reader {
                return Ok(Err(HnwNoClaim::PinnedByReader));
            }
            if evidence2.persisted_snapshots_present {
                return Ok(Err(HnwNoClaim::PersistedSnapshotPresent));
            }
            evidence2
        } else {
            evidence1
        };

        // 3. Invariant checks (INV-D2)
        if final_evidence.nonzero_vector_copies > 0 {
            return Err(ContextraError::InvalidInput(
                "INV-D2-HNSW: Nonzero vector copies found for target doc_ids".to_string(),
            ));
        }

        if final_evidence.incoming_edges > 0 {
            return Err(ContextraError::InvalidInput(format!(
                "INV-D2-HNSW: Incoming ghost edges remain in graph ({})",
                final_evidence.incoming_edges
            )));
        }

        if !final_evidence.entry_point_valid {
            return Err(ContextraError::InvalidInput(
                "INV-D2-HNSW: Graph entry point is invalid or points to deleted node".to_string(),
            ));
        }

        let scanned_at_tx = contextra_types::TxId::new(self.inner.hot.last_tx_id.load(std::sync::atomic::Ordering::Acquire));
        if scanned_at_tx < args.as_of_tx {
            return Err(ContextraError::InvalidInput(format!(
                "INV-D2-HNSW: scanned_at_tx ({:?}) < as_of_tx ({:?})",
                scanned_at_tx, args.as_of_tx
            )));
        }

        let store_gen = WitStoreGeneration(self.index_epoch());

        let node_id = args
            .doc_ids
            .first()
            .map(|d| d.inner())
            .unwrap_or(0);

        let layer_evidence = WitLayerEvidence::HnswIndex {
            node_id,
            incoming_edges: final_evidence.incoming_edges as u32,
            outgoing_edges: 0,
            pinned_by_reader: final_evidence.pinned_by_reader,
            marked_zeroed: final_evidence.vector_copies_zeroed > 0,
        };

        let witness_args = WitIssueArgs {
            layer: DeletionLayer::HnswIndex,
            scope_hash: args.scope_hash,
            scanned_at_tx,
            as_of_tx: args.as_of_tx,
            assurance: args.assurance,
            claim: WitClaimText::RamZeroized,
            footprint_hash: args.footprint_hash,
            scope_write_seq: args.scope_write_seq,
            scanner_name: "HnswResidueScanner",
            scanner_version: 1,
            store_generation: store_gen,
            layer_evidence,
            key_identifiers_claimed: false,
        };

        let witness = LayerWitness::issue(witness_args)?;
        Ok(Ok(witness))
    }
}
