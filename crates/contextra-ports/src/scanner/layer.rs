// FILE-CONTEXT
// ZWECK: Definition der Schichten, Scantechniken, Target-Deskriptoren und Purger-Traits (Vertrag V1, §12.7.4).
// INVARIANTEN: Schichten und Scantechniken definieren typsichere Parameter für Nachweis-Sessions.
// HOTSPOTS: [1-80]

#![forbid(unsafe_code)]

use crate::Result;
use serde::Serialize;

/// Identifikator für die physische Storage-Schicht.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
pub enum WitnessLayer {
    /// LSM Memtable
    LsmMemtable,
    /// SsTable Dateien über alle Level
    SsTableAllLevels,
    /// HNSW Vektorgraph-Index
    HnswIndex,
    /// Write-Ahead-Log Segmente
    WalAllSegments,
    /// Compressed Sparse Row Graph
    CsrGraph,
    /// In-Memory Embedding Cache
    EmbeddingCache,
    /// Benutzerdefinierte Schichtbezeichnung für Erweiterungen
    Custom(String),
}

/// Wire-Name der Text-Index-Schicht (Erasure A-3). Entspricht dem bisherigen `Custom("TextIndex")`.
pub const LAYER_TEXT: &str = "TextIndex";
/// Wire-Name der Checkpoint-Schicht (Erasure A-3). Journal-Format unverändert.
pub const LAYER_CHECKPOINTS: &str = "Checkpoints";

impl WitnessLayer {
    /// Text-Index-Schicht (`LAYER_TEXT`).
    pub fn text() -> Self {
        Self::from(LAYER_TEXT)
    }

    /// Checkpoint-Schicht (`LAYER_CHECKPOINTS`).
    pub fn checkpoints() -> Self {
        Self::from(LAYER_CHECKPOINTS)
    }

    /// Gibt den String-Namen der Schicht zurück.
    pub fn as_str(&self) -> &str {
        match self {
            Self::LsmMemtable => "LsmMemtable",
            Self::SsTableAllLevels => "SsTableAllLevels",
            Self::HnswIndex => "HnswIndex",
            Self::WalAllSegments => "WalAllSegments",
            Self::CsrGraph => "CsrGraph",
            Self::EmbeddingCache => "EmbeddingCache",
            Self::Custom(s) => s.as_str(),
        }
    }
}

impl From<&str> for WitnessLayer {
    fn from(s: &str) -> Self {
        match s {
            "LsmMemtable" => WitnessLayer::LsmMemtable,
            "SsTableAllLevels" => WitnessLayer::SsTableAllLevels,
            "HnswIndex" => WitnessLayer::HnswIndex,
            "WalAllSegments" => WitnessLayer::WalAllSegments,
            "CsrGraph" => WitnessLayer::CsrGraph,
            "EmbeddingCache" => WitnessLayer::EmbeddingCache,
            other => WitnessLayer::Custom(other.to_string()),
        }
    }
}

impl From<String> for WitnessLayer {
    fn from(s: String) -> Self {
        WitnessLayer::from(s.as_str())
    }
}

/// Angewandte Methode/Technik während der Verifikation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum ScanTechnique {
    /// Direkter struktureller Scan.
    Structural,
    /// Scan über entschlüsselte Ansicht.
    DecryptedView,
    /// Index-Introspektion.
    IndexIntrospection,
    /// Nadel-Scan (Needle Scan).
    NeedleScan,
    /// Schicht wurde in der aktuellen Ausführung nicht gemessen mit Begründung.
    NotMeasured(String),
    /// Schichtmessung ist aufgrund der Feature-Konfiguration deaktiviert.
    FeatureDisabled,
}

/// Zielbeschreibung des Scans inklusive Hash.
#[deprecated(note = "Ersetzt durch ResolvedScope (WP-02); nach WP-6..8 entfernen")]
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ScanTarget {
    /// Bezeichner des Scan-Ziels (z. B. Tenant, Collection, Path).
    pub target_id: String,
    /// BLAKE3 Hash des Scan-Ziels.
    pub target_hash: [u8; 32],
}

impl ScanTarget {
    /// Erzeugt ein neues ScanTarget.
    pub fn new(target_id: impl Into<String>, target_hash: [u8; 32]) -> Self {
        Self {
            target_id: target_id.into(),
            target_hash,
        }
    }
}

/// Statistiken über bereinigte Einheiten.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize)]
pub struct PurgeStats {
    /// Anzahl physisch bereinigter Einheiten.
    pub purged_units: u64,
    /// Anzahl freigegebener Bytes.
    pub bytes_freed: u64,
}

/// Trait für physische Bereinigungsaktionen (Purger) einer Schicht.
#[deprecated(note = "Ersetzt durch ErasureLayer (WP-02); nach WP-6..8 entfernen")]
pub trait Purger: Send + Sync {
    /// Führt die physische Bereinigung für das gegebene Ziel aus.
    ///
    /// # Fehler
    /// Liefert `ContextraError` bei I/O- oder Freigabefehlern während der physischen Bereinigung.
    /// # Crash
    /// Idempotent; Teilausfälle hinterlassen konsistente Zwischenzustände ohne Datenverlust lebender Einheiten.
    fn purge(&self, ctx: &crate::RequestContext, target: &ScanTarget) -> Result<PurgeStats>;
}

#[cfg(test)]
mod layer_constants_tests {
    use super::*;

    #[test]
    fn text_and_checkpoint_layers_use_stable_wire_names() {
        assert_eq!(WitnessLayer::text(), WitnessLayer::Custom("TextIndex".to_string()));
        assert_eq!(WitnessLayer::checkpoints(), WitnessLayer::Custom("Checkpoints".to_string()));
        assert_eq!(WitnessLayer::text().as_str(), LAYER_TEXT);
        assert_eq!(WitnessLayer::checkpoints().as_str(), LAYER_CHECKPOINTS);
    }
}
