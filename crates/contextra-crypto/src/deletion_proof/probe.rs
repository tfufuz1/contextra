//! Store-Probe-Schnittstelle und Generation-Token für Deletion Witnesses (C3).

#![forbid(unsafe_code)]

use crate::deletion_proof::types::DeletionLayer;
use serde::Serialize;

/// Monoton steigendes Generation-Token für eine Storage-Schicht.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub struct WitStoreGeneration(pub u64);

/// Store-Probe-Vertrag zur Abfrage des Generationenstands und Schreib-Sequenzen.
pub trait WitStoreProbe: Send + Sync {
    /// Gibt die aktuelle Generation der angegebenen Storage-Schicht zurück.
    fn current_generation(&self, layer: &DeletionLayer) -> WitStoreGeneration;

    /// Gibt die letzte Schreib-Sequenznummer für den gegebenen Scope-Hash zurück.
    fn scope_write_seq(&self, scope_hash: &[u8; 32]) -> u64;

    /// Gibt die Anzahl der Manifest-Dateien zurück, falls anwendbar.
    fn manifest_file_count(&self) -> Option<u32>;
}
