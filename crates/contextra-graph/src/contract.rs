// FILE-CONTEXT
// STAND: 2026-11-15T12:00:00Z
// ZWECK: Neutraler Crate-Vertrag (Clock, Rng, NodeId, Weight) für Klasse A Baustein.
// INVARIANTEN: Keine externen Abhängigkeiten, deterministische Traits, zero-panic.
// HOTSPOTS: Keine kritischen Bottlenecks identifiziert
// SIEHE AUCH: docs/SPEC.md, crates/contextra-graph/SPEC.md

// SECTION Contract Core Implementation

// ANCHOR[contextra-graph:SRC-CONTRACT-001]
/// Lokale Fehlertaxonomie für grundlegende Vertragsprüfungen.
#[allow(dead_code)]
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    /// Ungültige Eingabedaten oder fehlerhafte Argumente.
    #[error("ungültige Eingabedaten: {0}")]
    InvalidInput(&'static str),

    /// Geforderte Kapazität übersteigt die maximal allozierbare Grenze.
    #[error("Kapazitätsgrenze überschritten: gefordert {requested}, maximal {maximum}")]
    CapacityExceeded {
        /// Angefragte Element- oder Speichermenge
        requested: usize,
        /// Absolutes Maximum des Bausteins
        maximum: usize
    },

    /// Dateisystem- oder E/A-Fehler.
    #[error("E/A-Fehler: {0}")]
    Io(#[from] std::io::Error),

    /// Interner Zustandskonflikt bei nebenläufigen Mutationen.
    #[error("interner Zustandskonflikt: {0}")]
    StateConflict(&'static str),

    /// Beschädigte oder korrupte Serialisierungsdaten.
    #[error("beschädigte Daten: {0}")]
    Corrupt(&'static str),
}

// ANCHOR[contextra-graph:SRC-CONTRACT-002]
/// Typalias für vertragsbasierte Rückgabewerte.
#[allow(dead_code)]
pub type Result<T> = core::result::Result<T, Error>;

// ANCHOR[contextra-graph:SRC-CONTRACT-003]
/// Neutraler Uhren-Trait zur Entkopplung von Zeitmessungen.
/// Ermöglicht deterministische Zeitinjektion in Tests und Simulatoren.
#[allow(dead_code)]
pub trait Clock: Send + Sync {
    /// Gibt den aktuellen Unix-Zeitstempel in Millisekunden zurück.
    fn now_unix_ms(&self) -> u64;
}

// ANCHOR[contextra-graph:SRC-CONTRACT-004]
/// Neutraler Pseudozufallsgenerator-Trait.
/// Erlaubt reproduzierbare Zufallssequenzen für diffusionsbasierte Algorithmen.
#[allow(dead_code)]
pub trait Rng: Send {
    /// Generiert die nächste 64-Bit Pseudozufallszahl.
    fn next_u64(&mut self) -> u64;
}

// ANCHOR[contextra-graph:SRC-CONTRACT-005]
/// Eindeutiger Knoten-Identifikator im autonomen Graphen.
#[allow(dead_code)]
pub type NodeId = u64;

// ANCHOR[contextra-graph:SRC-CONTRACT-006]
/// Kantengewicht im CSR-Graphen.
/// NOTE: Muss stets endlich und nicht-negativ (>= 0.0) sein.
#[allow(dead_code)]
pub type Weight = f32;

// !SECTION Contract Core Implementation
