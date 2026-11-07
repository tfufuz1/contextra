//! Definitionen der Storage-Schichten, Scantechniken und Bereinigungsstatistiken.

#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

/// Identifikator für die physische Storage-Schicht.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WitnessLayer {
    /// LSM Memtable (flüchtiger In-Memory-Puffer).
    LsmMemtable,
    /// SsTable Dateien über alle Level.
    SsTableAllLevels,
    /// HNSW Vektorgraph-Index.
    HnswIndex,
    /// Write-Ahead-Log Segmente.
    WalAllSegments,
    /// Compressed Sparse Row Graph.
    CsrGraph,
    /// Key-Value Cache Segmente.
    KvCacheSegments,
    /// Inverted BM25 Volltext-Index.
    TextIndex,
    /// In-Memory Embedding Cache.
    EmbeddingCache,
    /// Point-in-Time Checkpoints.
    Checkpoints,
    /// Benutzerdefinierte Schichtbezeichnung für Erweiterungen.
    Custom(String),
}

impl WitnessLayer {
    /// Gibt den kanonischen String-Namen der Schicht zurück.
    pub fn as_str(&self) -> &str {
        match self {
            Self::LsmMemtable => "LsmMemtable",
            Self::SsTableAllLevels => "SsTableAllLevels",
            Self::HnswIndex => "HnswIndex",
            Self::WalAllSegments => "WalAllSegments",
            Self::CsrGraph => "CsrGraph",
            Self::KvCacheSegments => "KvCacheSegments",
            Self::TextIndex => "TextIndex",
            Self::EmbeddingCache => "EmbeddingCache",
            Self::Checkpoints => "Checkpoints",
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
            "KvCacheSegments" => WitnessLayer::KvCacheSegments,
            "TextIndex" => WitnessLayer::TextIndex,
            "EmbeddingCache" => WitnessLayer::EmbeddingCache,
            "Checkpoints" => WitnessLayer::Checkpoints,
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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScanTechnique {
    /// Direkter struktureller Scan.
    Structural,
    /// Scan über entschlüsselte Ansicht.
    DecryptedView,
    /// Index-Introspektion.
    IndexIntrospection,
    /// Nadel-Scan (Needle Scan nach Klartextmustern).
    NeedleScan,
    /// Schicht wurde in der aktuellen Ausführung nicht gemessen mit Begründung.
    NotMeasured(String),
    /// Schichtmessung ist aufgrund der Feature-Konfiguration deaktiviert.
    FeatureDisabled,
}

/// Statistiken über bereinigte Einheiten einer physischen Schicht.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct PurgeStats {
    /// Anzahl physisch bereinigter Einheiten (Dokumente, Slots, Einträge).
    pub purged_units: u64,
    /// Anzahl freigegebener bzw. überschriebener Bytes.
    pub bytes_freed: u64,
}

impl PurgeStats {
    /// Erzeugt eine neue Statistik.
    pub const fn new(purged_units: u64, bytes_freed: u64) -> Self {
        Self {
            purged_units,
            bytes_freed,
        }
    }
}
