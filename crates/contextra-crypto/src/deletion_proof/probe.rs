//! Store-Probe-Schnittstelle und Generation-Token für Deletion Witnesses (C3).

#![forbid(unsafe_code)]

use crate::deletion_proof::types::DeletionLayer;
use serde::Serialize;

/// Monoton steigendes Generation-Token für eine Storage-Schicht.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub struct WitStoreGeneration(pub u64);

/// Fehler beim Abfragen der Store Probe.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ProbeError {
    /// Operation wird nicht unterstützt.
    #[error("Probe operation unsupported")]
    Unsupported,
    /// Schicht nicht gefunden.
    #[error("Layer not found")]
    LayerNotFound,
    /// Interner Fehler.
    #[error("Internal probe error: {0}")]
    Internal(String),
}

/// Store-Probe-Vertrag zur Abfrage des Generationenstands und Schreib-Sequenzen.
pub trait ZeuStoreProbe: Send + Sync {
    /// Gibt die aktuelle Generation der angegebenen Storage-Schicht zurück (deprecated).
    #[deprecated(note = "current_generation is constant/deprecated. Use try_current_generation instead.")]
    fn current_generation(&self, _layer: &DeletionLayer) -> WitStoreGeneration {
        WitStoreGeneration(0)
    }

    /// Versucht, die aktuelle Generation abzufragen. Standardmäßig Err(ProbeError::Unsupported).
    fn try_current_generation(&self, _layer: DeletionLayer) -> Result<u64, ProbeError> {
        Err(ProbeError::Unsupported)
    }

    /// Gibt die letzte Schreib-Sequenznummer für den gegebenen Scope-Hash zurück.
    fn scope_write_seq(&self, _scope_hash: &[u8; 32]) -> u64 {
        0
    }

    /// Gibt die Anzahl der Manifest-Dateien zurück, falls anwendbar.
    fn manifest_file_count(&self) -> Option<u32> {
        None
    }
}

pub use ZeuStoreProbe as WitStoreProbe;
