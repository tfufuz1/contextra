// FILE-CONTEXT
// ZWECK: Vertragsdatei für contextra-vector (Eingefrorener Crate-Vertrag Klasse A).
// INVARIANTEN: Zero-Panic, autarker Baustein ohne interne Workspace-Kopplung (SPEC §4.3).
// HOTSPOTS: contract.rs:90-95 (DistanceFn), contract.rs:97-110 (Index), contract.rs:112-123 (Maintain), contract.rs:125-132 (BlockTransform)
// STAND: 2026-11-11

//! Vertragsdatei für `contextra-vector` (Eingefrorener Crate-Vertrag Klasse A).
//!
//! Definiert die kanonischen Typen und Schnittstellen für die autarke Nutzung
//! des Vektorindex-Bausteins gemäß SPEC §4.3 (B1–B4).
//!
//! # Kanonische Einordnung (SPEC §2, §4.3)
//! - **Kanonische Kern-Umwelt:** `contextra-env` ([`crate::types::Clock`], [`crate::types::Rng`])
//! - **Kanonische Kern-Identifikatoren:** `contextra-ids` ([`crate::types::DocId`], [`crate::types::TxId`])
//! - **Kanonische Verträge Klasse A:** Die hier deklarierten Traits ([`Index`], [`Maintain`],
//!   [`DistanceFn`], [`BlockTransform`]) sowie Hilfstypen ([`ExternalId`], [`Metric`], [`ScoredPoint`])
//!   definieren den minimalen, portablen Vertrag für Einbettungsumgebungen ohne Workspace-Kopplung.

// SECTION Vertragsfehler

// ANCHOR[contextra-vector:CONTRACT-ERROR]
/// Lokales Fehler-Enum für Vertragsoperationen.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    /// Ungültige Eingabedaten.
    #[error("ungültige Eingabedaten: {0}")]
    InvalidInput(&'static str),

    /// Kapazitätsgrenze überschritten.
    #[error("Kapazitätsgrenze überschritten: gefordert {requested}, maximal {maximum}")]
    CapacityExceeded {
        /// Geforderte Kapazität.
        requested: usize,
        /// Maximal zulässige Kapazität.
        maximum: usize,
    },

    /// E/A-Fehler.
    #[error("E/A-Fehler: {0}")]
    Io(#[from] std::io::Error),

    /// Interner Zustandskonflikt.
    #[error("interner Zustandskonflikt: {0}")]
    StateConflict(&'static str),

    /// Beschädigte Daten.
    #[error("beschädigte Daten: {0}")]
    Corrupt(&'static str),
}

/// Standard-Result-Typ für Vertragsoperationen.
pub type Result<T> = core::result::Result<T, Error>;
// !SECTION

// SECTION Umwelt-Traits (Clock, Rng)

// ANCHOR[contextra-vector:CONTRACT-CLOCK]
/// Trait zur Zeitabfrage.
pub trait Clock: Send + Sync {
    /// Liefert die aktuelle Zeit in Unix-Millisekunden.
    fn now_unix_ms(&self) -> u64;
}

// ANCHOR[contextra-vector:CONTRACT-RNG]
/// Trait für deterministischen Zufall.
pub trait Rng: Send {
    /// Liefert das nächste Pseudozufalls-`u64`.
    fn next_u64(&mut self) -> u64;
}
// !SECTION

// SECTION Datentypen (ExternalId, Metric, ScoredPoint)

// ANCHOR[contextra-vector:CONTRACT-EXTERNAL-ID]
/// Externe Identifikationsnummer (128-Bit für breite Kompatibilität).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ExternalId(pub u128); // 128 Bit erhält vorhandene Fähigkeit (E-05); From<u64> vorhanden

impl From<u64> for ExternalId {
    fn from(v: u64) -> Self {
        Self(v as u128)
    }
}

// ANCHOR[contextra-vector:CONTRACT-METRIC]
/// Unterstützte Distanzmetriken.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Metric {
    /// Kosinus-Distanz.
    Cosine,
    /// Euklidische Distanz.
    Euclidean,
    /// Skalarprodukt.
    DotProduct,
}

// ANCHOR[contextra-vector:CONTRACT-SCORED-POINT]
/// Trefferpunkt mit Distanz-/Ähnlichkeitsbewertung.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ScoredPoint {
    /// Externe Identifikationsnummer des Punkts.
    pub id: ExternalId,
    /// Ähnlichkeitswert (höher = besser, total geordnet).
    pub score: f32,
}
// !SECTION

// SECTION Service-Schnittstellen (DistanceFn, Index, Maintain, BlockTransform)

// ANCHOR[contextra-vector:CONTRACT-DISTANCE-FN]
/// Distanzfunktions-Schnittstelle zur externen Injektion von Berechnungen (z.B. SIMD).
pub trait DistanceFn: Send + Sync {
    /// Berechnet die Distanz zwischen zwei gleich langen Vektoren.
    fn distance(&self, a: &[f32], b: &[f32]) -> f32;
}

// ANCHOR[contextra-vector:CONTRACT-INDEX]
/// Standardisierter Index-Trait für Klasse A Bausteine (SPEC §4.3).
pub trait Index<Id, Query> {
    /// Typ eines einzelnen Suchtreffers.
    type Hit;

    /// Fügt einen Vektordatenpunkt ein.
    fn insert(&mut self, id: Id, payload: &[u8]) -> Result<()>;

    /// Entfernt einen Vektordatenpunkt anhand seiner ID.
    fn remove(&mut self, id: &Id) -> Result<bool>;

    /// Sucht die nächsten Nachbarn für eine gegebene Abfrage.
    fn search(&self, query: &Query, limit: usize) -> Result<Vec<Self::Hit>>;
}

// ANCHOR[contextra-vector:CONTRACT-MAINTAIN]
/// Neutraler Wartungs-Trait für Klasse A Bausteine (SPEC §4.3).
pub trait Maintain {
    /// Typ der Wartungsstatistiken.
    type Stats;

    /// Kompaktiert den Index und gibt freigegebene Ressourcen zurück.
    fn compact(&mut self) -> Result<Self::Stats>;

    /// Führt physische Bereinigungen durch.
    fn vacuum(&mut self) -> Result<()>;
}

// ANCHOR[contextra-vector:CONTRACT-BLOCK-TRANSFORM]
/// Generischer Transformations-Hook (z. B. für optionale Verschlüsselung/Transformation).
pub trait BlockTransform: Send + Sync {
    /// Transformiert einen Datenblock vor der Persistierung.
    fn transform(&self, block: &mut [u8]) -> Result<()>;
    /// Rekonstruiert einen transformierten Datenblock beim Lesen.
    fn inverse_transform(&self, block: &mut [u8]) -> Result<()>;
}
// !SECTION
