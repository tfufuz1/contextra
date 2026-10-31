//! Modul `entity` im Ring 0 (Graph-Subsystem) von Contextra.
//!
//! Stellt Kernkomponenten für Graphtraversierung, CSR-Repräsentationen und
//! Analysemethoden bereit. Wird von Integrationsschichten und Algorithmenmodulen verwendet.

use super::super::inner::sentinel_entity;
use super::CsrGraph;
use contextra_types::{Entity, Result};

impl CsrGraph {
    // ANCHOR[contextra-graph:INSERT_ENTIT-0120]
    /// Funktion `insert_entity_direct` für die Graphverarbeitung.
    ///
    /// # Zweck
    /// Kapselt Funktionalität im Ring 0 (Graph) ohne externe I/O-Seiteneffekte.
    ///
    /// # Ablauf
    /// 1. Parameter und Vorbedingungen im MVCC-/CSR-Graphkontext validieren.
    /// 2. Speichereffiziente Berechnung oder Datenzugriff durchführen.
    pub fn insert_entity_direct(&self, entity: Entity) -> Result<()> {
        let mut inner = self.inner_write();
        let idx = inner.get_or_create_index(entity.id);
        if idx >= inner.entities.len() {
            inner.entities.resize(idx + 1, sentinel_entity());
        }
        inner.entities[idx] = entity;
        Ok(())
    }

    // ANCHOR[contextra-graph:LOAD_ENTITY_-0121]
    /// Funktion `load_entity_direct` für die Graphverarbeitung.
    ///
    /// # Zweck
    /// Kapselt Funktionalität im Ring 0 (Graph) ohne externe I/O-Seiteneffekte.
    ///
    /// # Ablauf
    /// 1. Parameter und Vorbedingungen im MVCC-/CSR-Graphkontext validieren.
    /// 2. Speichereffiziente Berechnung oder Datenzugriff durchführen.
    pub fn load_entity_direct(&self, entity: Entity) -> Result<()> {
        let mut inner = self.inner_write();
        let idx = inner.get_or_create_index(entity.id);
        while inner.entities.len() <= idx {
            inner.entities.push(sentinel_entity());
        }
        inner.entities[idx] = entity;
        Ok(())
    }
}
