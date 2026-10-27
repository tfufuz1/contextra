// FILE-CONTEXT
// ZWECK: Definition der Schichten, Scantechniken, Target-Deskriptoren und Purger-Traits (Vertrag V1, §12.7.4).
// INVARIANTEN: Schichten und Scantechniken definieren typsichere Parameter für Nachweis-Sessions.
// HOTSPOTS: [1-80]

#![forbid(unsafe_code)]

use contextra_types::Result;
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
    /// Key-Value Cache Segmente
    KvCacheSegments,
    /// In-Memory Embedding Cache
    EmbeddingCache,
    /// Benutzerdefinierte Schichtbezeichnung für Erweiterungen
    Custom(String),
}

impl WitnessLayer {
    /// Gibt den String-Namen der Schicht zurück.
    pub fn as_str(&self) -> &str {
        match self {
            Self::LsmMemtable => "LsmMemtable",
            Self::SsTableAllLevels => "SsTableAllLevels",
            Self::HnswIndex => "HnswIndex",
            Self::WalAllSegments => "WalAllSegments",
            Self::CsrGraph => "CsrGraph",
            Self::KvCacheSegments => "KvCacheSegments",
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
            "KvCacheSegments" => WitnessLayer::KvCacheSegments,
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
    /// Anzahl physisch gelöschter Einheiten.
    pub purged_units: u64,
    /// Anzahl freigegebener Bytes.
    pub bytes_freed: u64,
}

/// Trait für physische Bereinigungsaktionen (Purger) einer Schicht.
pub trait Purger: Send + Sync {
    /// Führt die physische Löschung für das gegebene Ziel aus.
    fn purge(
        &self,
        ctx: &contextra_types::RequestContext,
        target: &ScanTarget,
    ) -> Result<PurgeStats>;
}
