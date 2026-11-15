// FILE-CONTEXT
// ZWECK: Lokale Typ- und Traitabstraktionen für Vektorsuche und Indexierung (Klasse A).
// INVARIANTEN: Zero-Panic, DocId-Breite konfigurierbar (64/128-Bit), deterministischer RNG.
// HOTSPOTS: types.rs:41-59 (ScoredDocument), types.rs:69-75 (Maintain)
// STAND: 2026-11-11

//! `types` — Lokale Typ- und Traitabstraktionen für Vektorsuche und Indexierung (Klasse A).
//!
//! # Zweck
//! Stellt sicher, dass `acta-vec` als autarker Baustein ohne interne Abhängigkeiten
//! zu `acta-base`, `acta-base` oder `acta-history` funktioniert.
//! Bei aktiviertem transitionalen Feature `ports-compat` stehen verlustfreie Konvertierungen
//! zu den Workspace-Typen bereit.

// SECTION Identifikatoren (DocId, TxId)

pub use acta_base::{DocId, TxId};
// !SECTION

// SECTION Metriken & Suchergebnisse

// ANCHOR[acta-vec:TYPES-METRIC]
/// Unterstützte Distanzmetriken für Vektorräume.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize, Default,
)]
pub enum DistanceMetric {
    /// Kosinus-Distanz ($1 - \text{sim}$).
    #[default]
    Cosine,
    /// Euklidische Distanz ($L_2$).
    Euclidean,
    /// Skalarprodukt (Dot-Produkt, invertiert für Minimierung).
    DotProduct,
}

// ANCHOR[acta-vec:TYPES-SCORED-DOC]
/// Suchergebnis mit Dokument-ID und Relevanz-Score / Distanz.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ScoredDocument {
    /// Dokument-ID des Treffers.
    pub doc_id: DocId,
    /// Ähnlichkeitswert (Score, höher ist ähnlicher).
    pub score: f32,
    /// Distanzwert (niedriger ist näher).
    pub distance: f32,
}

impl ScoredDocument {
    /// Erzeugt ein ScoredDocument mit abgeleiteter Distanz (`1.0 - score`).
    pub fn from_score(doc_id: DocId, score: f32) -> Self {
        Self {
            doc_id,
            score,
            distance: 1.0 - score,
        }
    }

    /// Erzeugt ein neues ScoredDocument mit explizitem Score und Distanzwert.
    pub fn new(doc_id: DocId, score: f32, distance: f32) -> Self {
        Self {
            doc_id,
            score,
            distance,
        }
    }
}

/// Maximal zulässiges `k` für Nachbarschaftssuchen.
pub const MAX_SEARCH_K: usize = 10_000;

/// Bitmaske für Tombstone-Markierung in Sequenznummern.
/// Moved to acta_base::TOMBSTONE_BIT per ADR-0120.
/// This re-export remains until S-70 (INV-2).
pub use acta_base::TOMBSTONE_BIT;
// !SECTION

// SECTION Umwelt-Abstraktionen (Clock, Rng, Metrics, Maintain)

pub use acta_base::{
    unix_ms, Clock, MetricsSink, NoopMetricsSink, Rng, SeededRng, SystemClock,
};

// ANCHOR[acta-vec:TYPES-MAINTAIN]
/// Neutraler Wartungs-Trait für Speicherfreigaben und Kompaktierung.
pub trait Maintain: Send + Sync {
    /// Führt physische Bereinigung und Slot-Freigaben durch.
    fn compact(&self) -> crate::error::Result<()>;
    /// Prüft den Rebuild- bzw. Reorganisationsbedarf.
    fn is_rebuild_required(&self) -> bool;
}
// !SECTION
