//! # Crate `acta-testkit`
//!
//! Deterministische Test-Utilities, Mocks, In-Memory-Referenzimplementierungen
//! und Konformitätsprüfsuiten für das Acta-System.
//!
//! ## Ring & Architektur-Einordnung
//! - **Ring:** Werkzeug / Tooling (außerhalb der numerischen Ringe 0 bis 4).
//! - **Klasse:** Werkzeug (`—`).
//! - **Sicherheits-Invariante:** `acta-testkit` darf ausschließlich
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
//! - **Normal:** `acta-base`, `acta-base`, `bytes`, `parking_lot`, `tracing`.
//! - **Wird eingebunden von:** `acta`, `acta-engine`, `acta-history`,
//!   `acta-engine`, `acta-crypto`, `acta-engine`, `acta-engine`,
//!   `acta-engine`, `acta-history`, `acta-pii`, `acta-seg`.

// FILE-CONTEXT
// STAND: 2026-10-29
// ZWECK: Deterministic test utilities, ManualClock, InMemoryStorageEngine, InMemoryVectorIndex, InMemoryTextIndex, InMemoryGraphIndex, FaultVfs, ReferenceModel, Mocks, and Conformance suites.
// INVARIANTEN: No unsafe code allowed in testkit (#![forbid(unsafe_code)]).

#![forbid(unsafe_code)]

// ANCHOR[acta-testkit:MOD-CONFORMANCE-001]
/// Modul mit standardisierten Konformitätsprüfsuiten für Acta-Port-Traits.
pub mod conformance;

// ANCHOR[acta-testkit:MOD-FAULT-VFS-001]
/// Virtuelles Dateisystem mit deterministischer Fehlerinjektion (I/O- und fsync-Failures).
pub mod fault_vfs;

// ANCHOR[acta-testkit:MOD-IN-MEM-GRAPH-001]
/// Deterministische In-Memory-Referenzimplementierung des `GraphIndex`-Ports.
pub mod in_memory_graph;

// ANCHOR[acta-testkit:MOD-IN-MEM-]
/// Transaktionale In-Memory-Referenzimplementierung des `StorageEngine`-Ports.
pub mod in_memory_store;

// ANCHOR[acta-testkit:MOD-IN-MEM-TEXT-001]
/// Deterministische In-Memory-Referenzimplementierung des `TextIndex`-Ports.
pub mod in_memory_text;

// ANCHOR[acta-testkit:MOD-IN-MEM-VECTOR-001]
/// Deterministische In-Memory-Referenzimplementierung des `VectorIndex`-Ports.
pub mod in_memory_vector;

// ANCHOR[acta-testkit:MOD-MANUAL-CLOCK-001]
/// Manuell steuerbare Uhr zur reproduzierbaren Zeitmessung im `Clock`-Port.
pub mod manual_clock;

// ANCHOR[acta-testkit:MOD-MOCKS-001]
/// Mocks für externe Provider wie Embedding-Generatoren und Metrik-Sinks.
pub mod mocks;

// ANCHOR[acta-testkit:MOD-REF-MODEL-001]
/// Formales Referenzmodell zur Prüfung von MVCC- und Transaktionsisolationsgarantien.
pub mod reference_model;

/// Baukasten-Zielmodul `doubles`: Test-Doubles, In-Memory-Referenzen und steuerbare Uhren.
pub mod doubles {
    pub use crate::in_memory_graph::InMemoryGraphIndex;
    pub use crate::in_memory_store::InMemoryStorageEngine;
    pub use crate::in_memory_text::InMemoryTextIndex;
    pub use crate::in_memory_vector::InMemoryVectorIndex;
    pub use crate::manual_clock::ManualClock;
    pub use crate::mocks::{
        BudgetTruncatingContextPreparer, InMemoryKvBridgeStorage, InMemoryKvPrefixStore,
        MockEmbedder, ScriptedAttentionExporter, ScriptedEgressPolicy, ScriptedHybridSearchProvider,
        StaticCommunityResolver, StaticDriftStatusProvider, TestMetricsSink,
    };
}

/// Baukasten-Zielmodul `fault`: Fehlerinjektion und Fehlersimulation (VFS, I/O, Crashes).
pub mod fault {
    pub use crate::fault_vfs::{FaultConfig, FaultVfs};
}

/// Baukasten-Zielmodul `model`: Formale Orakel und Referenzmodelle für MVCC und KV-Semantik.
pub mod model {
    pub use crate::reference_model::{RefOp, ReferenceModel};
}

// Re-Exports der zentralen Typen für bequemen Zugriff in Testsuiten
pub use fault_vfs::{FaultConfig, FaultVfs};
pub use in_memory_graph::InMemoryGraphIndex;
pub use in_memory_store::InMemoryStorageEngine;
pub use in_memory_text::InMemoryTextIndex;
pub use in_memory_vector::InMemoryVectorIndex;
pub use manual_clock::ManualClock;
pub use mocks::*;
pub use reference_model::{RefOp, ReferenceModel};

// ANCHOR[acta-testkit:MINT-WITNESS-001]
/// Hilfsfunktion zum Erzeugen eines synthetischen [`acta_base::scanner::LayerWitness`].
///
/// **Zweck:** Ermöglicht Testsuiten das Instanziieren von Bereinigungs- und Scan-Zeugen
/// (Witnesses), ohne echte Platten-Scans oder kryptografische Prüfungen durchführen zu müssen.
///
/// **Ablauf:**
/// 1. Überführt das übergebene Layer-Argument in ein `WitnessLayer`.
/// 2. Ruft die interne Port-Funktion `mint_witness_for_testkit` mit synthetischen Werten
///    (fixer Hash `[0xFE; 32]`, Sequenznummer `0`) auf.
///
/// **Vorbedingungen:** Nur aktiv unter dem Feature-Flag `erasure-conformance`.
/// **Rückgabe:** Ein fertig initialisiertes `LayerWitness`-Objekt.
///
/// # Beispiel
/// ```rust,ignore
/// use acta_testkit::mint_witness_for_tests;
/// use acta_base::scanner::{ScanTechnique, WitnessLayer};
///
/// let witness = mint_witness_for_tests(WitnessLayer::Wal, 100, 0, ScanTechnique::FullScan);
/// assert_eq!(witness.scanned_keys(), 100);
/// ```
#[cfg(feature = "erasure-conformance")]
pub fn mint_witness_for_tests(
    layer: impl Into<acta_base::scanner::WitnessLayer>, // TODO(acta-hr3): Hub-Import (ex types/ports) nach HR-2/HR-3 aufloesen
    scanned: u64,
    remaining: u64,
    technique: acta_base::scanner::ScanTechnique, // TODO(acta-hr3): Hub-Import (ex types/ports) nach HR-2/HR-3 aufloesen
) -> acta_base::scanner::LayerWitness { // TODO(acta-hr3): Hub-Import (ex types/ports) nach HR-2/HR-3 aufloesen
    acta_base::scanner::mint_witness_for_testkit( // TODO(acta-hr3): Hub-Import (ex types/ports) nach HR-2/HR-3 aufloesen
        layer.into(),
        scanned,
        remaining,
        technique,
        [0xFE; 32],
        0,
    )
}
