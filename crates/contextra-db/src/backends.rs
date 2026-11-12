//! # Modul: backends.rs
//!
//! **Zweck:** Generisches Trait-Bündel für Speicher-, Vektor- und Index-Backends (WP-ENG-1, SPEC §4).
//!
//! **Architektur:** Gehört zur Crate `contextra-engine` (Ring 3, Klasse C).
//! Eingehaltene Invarianten: Fail-Closed bei Fehlern, Entkopplung von konkreten Implementierungen.

// FILE-CONTEXT
// ZWECK: Generische Abstraktion und Bündelung von Storage-, Vektor-, Text- und Graph-Backends.
// INVARIANTEN: Keine harten Kopplungen an konkrete Typen in Trait-Signaturen; Erweiterbarkeit für Mocks/Alternative Engines.
// STAND: 2026-11-11

use contextra_ports::{GraphIndex, StorageEngine, TextIndex, VectorIndex};

// SECTION Backends-Architektur
// ANCHOR[id=engine-backends-trait,epic=decoupling,seq=1]
// NOTE: Backends definiert den Kern-Vertrag zur flexiblen Zusammensetzung von Speicher- und Index-Bausteinen.
// LINK crates/contextra-engine/src/lib.rs#engine-config

/// Baukasten-Vertrag (M3, WP-ENG-1): Assoziiertes Trait-Bündel zur Entkopplung
/// der Engine von konkreten Backend-Implementierungen (`store`, `vector`, `text`, `graph`).
///
/// Nachfolgende Entwickler können diesen Vertrag implementieren, um alternative
/// Speicher- oder Index-Engines (z. B. In-Memory-Mocks für Unittests oder verteilte Speichersysteme)
/// einzuhängen.
pub trait Backends: Send + Sync + 'static {
    /// Konkreter Speicher-Backend-Typ (implementiert [`StorageEngine`]).
    type Storage: StorageEngine;
    /// Konkreter Vektor-Index-Typ (implementiert [`VectorIndex`]).
    type Vector: VectorIndex;
    /// Optionaler Volltext-Index-Typ (implementiert [`TextIndex`]).
    type Text: TextIndex;
    /// Optionaler Graph-Index-Typ (implementiert [`GraphIndex`]).
    type Graph: GraphIndex;
}

#[cfg(not(loom))]
use contextra_graph::CsrGraph;
#[cfg(not(loom))]
use contextra_store::LsmStorage;
#[cfg(not(loom))]
use contextra_text::InvertedIndex;
#[cfg(not(loom))]
use contextra_vector::HnswIndex;

// ANCHOR[id=engine-default-backends,epic=decoupling,seq=2]
// NOTE: DefaultBackends bindet die Standard-Produktionskerne (LsmStorage, HnswIndex, InvertedIndex, CsrGraph).
// LINK crates/contextra-store/src/lsm/engine.rs
// LINK crates/contextra-vector/src/hnsw/mod.rs

/// Standard-Backends unter Verwendung von LSM-Storage, HNSW-Index, Inverted-Text und CSR-Graph.
///
/// Wird standardmäßig von der Engine genutzt, wenn keine abweichenden Implementierungen konfiguriert werden.
#[cfg(not(loom))]
#[derive(Debug, Clone, Copy, Default)]
pub struct DefaultBackends;

#[cfg(not(loom))]
impl Backends for DefaultBackends {
    type Storage = LsmStorage;
    type Vector = HnswIndex;
    type Text = InvertedIndex<LsmStorage>;
    type Graph = CsrGraph;
}
// !SECTION Backends-Architektur
