//! Proof of physical cleanup for storage layers.

use super::types::DeletionLayer;
use contextra_types::{ContextraError, Result};

/// Beweis, dass ein bestimmter DeletionLayer physisch bereinigt wurde.
/// Kann NUR von den jeweiligen Bereinigungsfunktionen der Storage-Layer erzeugt werden
/// (siehe `new_after_physical_cleanup`), niemals direkt frei durch Aufrufer von
/// `DeletionProof::create()`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LayerCleanupProof {
    pub(super) layer: DeletionLayer,
    _private: (),
}

impl LayerCleanupProof {
    /// Erzeugt einen Proof NUR, wenn `remaining_live_entries == 0` — also nur dann,
    /// wenn der Aufrufer nachweislich (durch einen Re-Scan des betroffenen
    /// Storage-Bereichs NACH der physischen Bereinigung) verifiziert hat, dass für
    /// diesen Layer keine lebenden Einträge mehr existieren. Ein Proof für einen
    /// Layer mit `remaining_live_entries > 0` ist ein Widerspruch zu INV-DELETION-1
    /// und wird abgelehnt statt stillschweigend akzeptiert.
    ///
    /// # Errors
    /// Gibt `ContextraError::Internal` zurück, wenn `remaining_live_entries != 0`.
    #[cfg(any(test, feature = "test-support"))]
    pub fn new_after_verified_empty(
        layer: DeletionLayer,
        remaining_live_entries: usize,
    ) -> Result<Self> {
        Self::new_after_verified_empty_internal(layer, remaining_live_entries)
    }

    #[cfg(not(any(test, feature = "test-support")))]
    pub(crate) fn new_after_verified_empty(
        layer: DeletionLayer,
        remaining_live_entries: usize,
    ) -> Result<Self> {
        Self::new_after_verified_empty_internal(layer, remaining_live_entries)
    }

    fn new_after_verified_empty_internal(
        layer: DeletionLayer,
        remaining_live_entries: usize,
    ) -> Result<Self> {
        if remaining_live_entries != 0 {
            return Err(ContextraError::Internal(format!(
                "INV-DELETION-1 violation: attempted to construct LayerCleanupProof for \
                 layer {layer:?} but verification found {remaining_live_entries} \
                 remaining live entries — physical cleanup is incomplete or was not \
                 performed before proof construction"
            )));
        }
        Ok(Self {
            layer,
            _private: (),
        })
    }

    /// Erzeugt einen [`LayerCleanupProof`] aus einem verifizierten [`LayerWitness`].
    ///
    /// Liest die verbleibenden Einträge aus den Evidenzdaten des Witness.
    /// Schlägt fehl, wenn der Witness nicht verifiziert werden kann oder unvollständig ist,
    /// oder wenn verbleibende Einträge > 0 gefunden werden.
    ///
    /// # Errors
    /// Returns [`ContextraError::InvalidInput`] or [`ContextraError::Internal`] if:
    /// - `witness.verify()` fails or returns `WitnessVerification::None`
    /// - evidence records remaining live entries > 0
    pub fn from_witness(witness: &crate::deletion_witness::LayerWitness) -> Result<Self> {
        let verification = witness.verify()?;
        if matches!(
            verification,
            crate::deletion_witness::WitnessVerification::None
        ) {
            return Err(ContextraError::InvalidInput(
                "LayerCleanupProof creation rejected: witness contains no verified evidence".to_string(),
            ));
        }

        let remaining = match witness.layer_evidence() {
            Some(ev) => match ev {
                crate::deletion_proof::evidence::WitLayerEvidence::Memtable {
                    entries_found,
                    tombstones_found,
                    ..
                } => entries_found.saturating_sub(*tombstones_found) as usize,
                crate::deletion_proof::evidence::WitLayerEvidence::SsTables {
                    versions_found,
                    identifier_hits_index,
                    tombstones_remaining,
                    orphan_files_found,
                    unclassified_files,
                    ..
                } => (*versions_found
                    + *identifier_hits_index
                    + *tombstones_remaining
                    + u64::from(*orphan_files_found)
                    + u64::from(*unclassified_files)) as usize,
                crate::deletion_proof::evidence::WitLayerEvidence::Wal {
                    unscrubbed_records_found,
                    ..
                } => *unscrubbed_records_found as usize,
                crate::deletion_proof::evidence::WitLayerEvidence::HnswIndex {
                    incoming_edges,
                    pinned_by_reader,
                    ..
                } => *incoming_edges as usize + if *pinned_by_reader { 1 } else { 0 },
                crate::deletion_proof::evidence::WitLayerEvidence::TextIndex {
                    term_doc_hits,
                    ..
                } => *term_doc_hits as usize,
                crate::deletion_proof::evidence::WitLayerEvidence::Relations {
                    matching_edges_found,
                    ..
                } => *matching_edges_found as usize,
                crate::deletion_proof::evidence::WitLayerEvidence::ProcessCaches {
                    active_references,
                    invalidation_in_flight,
                    ..
                } => *active_references as usize + if *invalidation_in_flight { 1 } else { 0 },
                crate::deletion_proof::evidence::WitLayerEvidence::CheckpointClones {
                    clones_found,
                    ..
                } => *clones_found as usize,
                crate::deletion_proof::evidence::WitLayerEvidence::Keyring {
                    dek_present,
                    kek_epoch_destroyed,
                    ..
                } => if *dek_present || !*kek_epoch_destroyed { 1 } else { 0 },
                crate::deletion_proof::evidence::WitLayerEvidence::ErasureLedger {
                    entry_confirmed,
                    ..
                } => if !*entry_confirmed { 1 } else { 0 },
                crate::deletion_proof::evidence::WitLayerEvidence::Manifest {
                    in_flight_commits,
                    unpersisted_edits,
                    ..
                } => (*in_flight_commits + *unpersisted_edits) as usize,
            },
            None => 0,
        };

        Self::new_after_verified_empty(witness.layer().clone(), remaining)
    }

    /// Erzeugt einen Proof für `layer` NUR, wenn `verification` bestätigt,
    /// dass die physische Bereinigung tatsächlich abgeschlossen ist.
    ///
    /// `verification` MUSS eine echte Post-Condition-Prüfung durchführen
    /// (z. B. eine erneute Abfrage des betroffenen Storage-Layers, die
    /// belegt, dass keine der zu löschenden Daten mehr vorhanden sind),
    /// KEINE bloße Behauptung. Ein `Ok(false)`-Rückgabewert oder ein
    /// `Err` aus `verification` führt zu einem `Err` hier — es wird in
    /// diesem Fall NIEMALS ein Proof erzeugt (INV-DELETION-1).
    pub fn verify_and_create<F>(layer: DeletionLayer, verification: F) -> Result<Self>
    where
        F: FnOnce() -> Result<bool>,
    {
        if verification()? {
            Ok(Self {
                layer,
                _private: (),
            })
        } else {
            Err(ContextraError::Internal(format!(
                "INV-DELETION-1 violation: physical cleanup verification \
                 failed for layer {layer:?} — refusing to create \
                 LayerCleanupProof"
            )))
        }
    }

    /// Gibt den zugrundeliegenden DeletionLayer zurück.
    pub fn layer(&self) -> &DeletionLayer {
        &self.layer
    }
}
