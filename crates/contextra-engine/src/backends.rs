//! # Modul: backends.rs
//!
//! **Zweck:** Generisches Trait-Bündel für Speicher-, Vektor- und Index-Backends (WP-ENG-1, SPEC §4).
//!
//! **Architektur:** Gehört zur Crate `contextra-engine` (Ring 3).
//! Eingehaltene Invarianten: Fail-Closed bei Fehlern, Entkopplung von konkreten Implementierungen.

use contextra_ports::{GraphIndex, StorageEngine, TextIndex, VectorIndex};

/// Baukasten-Vertrag (M3, WP-ENG-1): Assoziiertes Trait-Bündel zur Entkopplung
/// der Engine von konkreten Backend-Implementierungen (`store`, `vector`, `text`, `graph`).
pub trait Backends: Send + Sync + 'static {
    /// Konkreter Speicher-Backend-Typ (implementiert `StorageEngine`).
    type Storage: StorageEngine;
    /// Konkreter Vektor-Index-Typ (implementiert `VectorIndex`).
    type Vector: VectorIndex;
    /// Optionaler Volltext-Index-Typ (implementiert `TextIndex`).
    type Text: TextIndex;
    /// Optionaler Graph-Index-Typ (implementiert `GraphIndex`).
    type Graph: GraphIndex;
}

#[cfg(not(loom))]
use contextra_store::LsmStorage;
#[cfg(not(loom))]
use contextra_vector::HnswIndex;
#[cfg(not(loom))]
use contextra_text::InvertedIndex;
#[cfg(not(loom))]
use contextra_graph::CsrGraph;

/// Standard-Backends unter Verwendung von LSM-Storage, HNSW-Index, Inverted-Text und CSR-Graph.
#[cfg(not(loom))]
#[derive(Debug, Clone, Copy, Default)]
pub struct DefaultBackends;

#[cfg(not(loom))]
impl Backends for DefaultBackends {
    type Storage = LsmStorage;
    type Vector = HnswIndex;
    type Text = InvertedIndex;
    type Graph = CsrGraph;
}
