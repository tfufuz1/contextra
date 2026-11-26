//! # Modul: backends.rs
//!
//! **Zweck:** Generisches Trait-Bündel für Speicher-, Vektor- und Index-Backends (WP-ENG-1, SPEC §4).
//!
//! **Architektur:** Gehört zur Crate `contextra-engine` (Ring 3).
//! Eingehaltene Invarianten: Fail-Closed bei Fehlern, Entkopplung von konkreten Implementierungen.

use contextra_ports::{StorageEngine, VectorIndex};

/// Baukasten-Vertrag (M3, WP-ENG-1): Assoziiertes Trait-Bündel zur Entkopplung
/// der Engine von konkreten Backend-Implementierungen (`store`, `vector`, `text`).
pub trait Backends: Send + Sync + 'static {
    /// Konkreter Speicher-Backend-Typ (implementiert `StorageEngine`).
    type Storage: StorageEngine;
    /// Konkreter Vektor-Index-Typ (implementiert `VectorIndex`).
    type Vector: VectorIndex;
}

#[cfg(not(loom))]
use contextra_store::LsmStorage;
#[cfg(not(loom))]
use contextra_vector::HnswIndex;

/// Standard-Backends unter Verwendung von LSM-Storage und HNSW-Index.
#[cfg(not(loom))]
#[derive(Debug, Clone, Copy, Default)]
pub struct DefaultBackends;

#[cfg(not(loom))]
impl Backends for DefaultBackends {
    type Storage = LsmStorage;
    type Vector = HnswIndex;
}
