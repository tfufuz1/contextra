//! Entitätsverwaltung und Primärschlüssel-Mapping im CSR-Graphen.
//!
//! # Zweck
//! Ordnet externen `EntityId`-Werten interne, dichte Array-Indizes für schnellen Zugriff zu.

// FILE-CONTEXT
// STAND: 2026-11-15T12:00:00Z
// ZWECK: Compressed Sparse Row (CSR) Graph-Engine Kern.
// INVARIANTEN: Bitemporale Sichtbarkeit, lock-freie Lesepfade.
// HOTSPOTS: Keine kritischen Bottlenecks identifiziert
// SIEHE AUCH: docs/SPEC.md, crates/contextra-graph/SPEC.md

use super::super::inner::sentinel_entity;
use super::CsrGraph;
use contextra_types::{Entity, Result};


// SECTION Entity Core Implementation
impl CsrGraph {
    // ANCHOR[contextra-graph:SRC-CSR-GRAPH-WRITE-ENTITY-001]
    pub fn insert_entity_direct(&self, entity: Entity) -> Result<()> {
        let mut inner = self.inner_write();
        let idx = inner.get_or_create_index(entity.id);
        if idx >= inner.entities.len() {
            inner.entities.resize(idx + 1, sentinel_entity());
        }
        inner.entities[idx] = entity;
        Ok(())
    }

    // ANCHOR[contextra-graph:SRC-CSR-GRAPH-WRITE-ENTITY-002]
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

// !SECTION Entity Core Implementation
