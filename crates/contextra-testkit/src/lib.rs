//! # Crate `contextra-testkit`
//!
//! Deterministische Test-Utilities, Mocks, In-Memory-Referenzimplementierungen
//! und Konformitätsprüfsuiten für das Contextra-System.
//!
//! ## Ring & Architektur-Einordnung
//! - **Ring:** Werkzeug / Tooling (außerhalb der numerischen Ringe 0 bis 4).
//! - **Klasse:** Werkzeug (`—`).
//! - **Sicherheits-Invariante (INV-EMBED-1):** `contextra-testkit` darf ausschließlich
//!   als `[dev-dependencies]` in Produktions-Crates eingebunden werden und niemals
//!   in Release-Binaries oder produktiven Abhängigkeiten landen.
//! - **Unsafe-Code:** Ausdrücklich verboten über `#![forbid(unsafe_code)]`.
//!
//! ## Bereitgestellte Hauptkomponenten
//! - [`ManualClock`]: Deterministische, manuell steuerbare Uhr (implementiert `Clock`-Port).
//! - [`FaultVfs`]: Virtuelles Dateisystem mit kontrollierter Fehlerinjektion für I/O- und fsync-Fehler.
//! - [`InMemoryStorageEngine`]: Transaktionale In-Memory-Speicher-Engine mit Rollback-Unterstützung.
//! - [`InMemoryVectorIndex`]: In-Memory-Vektorindex mit Cosine-Distanzberechnung.
//! - [`InMemoryTextIndex`]: In-Memory-Volltextindex (BM25-ähnlich).
//! - [`InMemoryGraphIndex`]: In-Memory-Graphindex für Kanten- und Knotentraversierung.
//! - [`ReferenceModel`]: Referenzmodell zur Validierung von MVCC-Isolationsstufen (Multi-Version Concurrency Control).
//! - [`mocks`]: Mocks für Embedder und Metriken (`MockEmbedder`, `TestMetricsSink`).
//! - [`conformance`]: Standardisierte Konformitätstests für Ports (Clock, RNG, Storage, Vector, etc.).
//!
//! ## Abhängigkeiten
//! - **Normal:** `contextra-ports`, `contextra-types`, `bytes`, `parking_lot`, `tracing`.
//! - **Wird eingebunden von:** `contextra`, `contextra-agent`, `contextra-checkpoint`,
//!   `contextra-cognition`, `contextra-crypto`, `contextra-db`, `contextra-engine`,
//!   `contextra-graph`, `contextra-mvcc`, `contextra-privacy`, `contextra-store`.

// FILE-CONTEXT
// STAND: 2026-10-29
// ZWECK: Deterministic test utilities, ManualClock, InMemoryStorageEngine, InMemoryVectorIndex, InMemoryTextIndex, InMemoryGraphIndex, FaultVfs, ReferenceModel, Mocks, and Conformance suites.
// INVARIANTEN: No unsafe code allowed in testkit (#![forbid(unsafe_code)]).

#![forbid(unsafe_code)]

// ANCHOR[contextra-testkit:MOD-CONFORMANCE-001]
/// Modul mit standardisierten Konformitätsprüfsuiten für Contextra-Port-Traits.
pub mod conformance;

// ANCHOR[contextra-testkit:MOD-FAULT-VFS-001]
/// Virtuelles Dateisystem mit deterministischer Fehlerinjektion (I/O- und fsync-Failures).
pub mod fault_vfs;

// ANCHOR[contextra-testkit:MOD-IN-MEM-GRAPH-001]
/// Deterministische In-Memory-Referenzimplementierung des `GraphIndex`-Ports.
pub mod in_memory_graph;

// ANCHOR[contextra-testkit:MOD-IN-MEM-STORE-001]
/// Transaktionale In-Memory-Referenzimplementierung des `StorageEngine`-Ports.
pub mod in_memory_store;

// ANCHOR[contextra-testkit:MOD-IN-MEM-TEXT-001]
/// Deterministische In-Memory-Referenzimplementierung des `TextIndex`-Ports.
pub mod in_memory_text;

// ANCHOR[contextra-testkit:MOD-IN-MEM-VECTOR-001]
/// Deterministische In-Memory-Referenzimplementierung des `VectorIndex`-Ports.
pub mod in_memory_vector;

// ANCHOR[contextra-testkit:MOD-MANUAL-CLOCK-001]
/// Manuell steuerbare Uhr zur reproduzierbaren Zeitmessung im `Clock`-Port.
pub mod manual_clock;

// ANCHOR[contextra-testkit:MOD-MOCKS-001]
/// Mocks für externe Provider wie Embedding-Generatoren und Metrik-Sinks.
pub mod mocks;

// ANCHOR[contextra-testkit:MOD-REF-MODEL-001]
/// Formales Referenzmodell zur Prüfung von MVCC- und Transaktionsisolationsgarantien.
pub mod reference_model;

// Re-Exports der zentralen Typen für bequemen Zugriff in Testsuiten
pub use fault_vfs::{FaultConfig, FaultVfs};
pub use in_memory_graph::InMemoryGraphIndex;
pub use in_memory_store::InMemoryStorageEngine;
pub use in_memory_text::InMemoryTextIndex;
pub use in_memory_vector::InMemoryVectorIndex;
pub use manual_clock::ManualClock;
pub use mocks::*;
pub use reference_model::{RefOp, ReferenceModel};

// ANCHOR[contextra-testkit:MINT-WITNESS-001]
/// Hilfsfunktion zum Erzeugen eines synthetischen [`contextra_ports::scanner::LayerWitness`].
///
/// **Zweck:** Ermöglicht Testsuiten das Instanziieren von Bereinigungs- und Scan-Zeugen
/// (Witnesses), ohne echte Platten-Scans oder kryptografische Prüfungen durchführen zu müssen.
///
/// **Ablauf:**
/// 1. Überführt das übergebene Layer-Argument in ein `WitnessLayer`.
/// 2. Ruft die interne Port-Funktion `mint_witness_for_testkit` mit synthetischen Werten
///    (fixer Hash `[0xFE; 32]`, Sequenznummer `0`) auf.
///
/// **Vorbedingungen:** Nur aktiv unter dem Feature-Flag `testkit`.
/// **Rückgabe:** Ein fertig initialisiertes `LayerWitness`-Objekt.
///
/// # Beispiel
/// ```rust,ignore
/// use contextra_testkit::mint_witness_for_tests;
/// use contextra_ports::scanner::{ScanTechnique, WitnessLayer};
///
/// let witness = mint_witness_for_tests(WitnessLayer::Wal, 100, 0, ScanTechnique::FullScan);
/// assert_eq!(witness.scanned_keys(), 100);
/// ```
#[cfg(feature = "testkit")]
pub fn mint_witness_for_tests(
    layer: impl Into<contextra_ports::scanner::WitnessLayer>,
    scanned: u64,
    remaining: u64,
    technique: contextra_ports::scanner::ScanTechnique,
) -> contextra_ports::scanner::LayerWitness {
    contextra_ports::scanner::mint_witness_for_testkit(
        layer.into(),
        scanned,
        remaining,
        technique,
        [0xFE; 32],
        0,
    )
}
