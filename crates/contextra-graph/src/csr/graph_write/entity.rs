//! Entitätsverwaltung und Primärschlüssel-Mapping im CSR-Graphen.
//!
//! # Zweck
//! Ordnet externen `EntityId`-Werten interne, dichte Array-Indizes für schnellen Zugriff zu.

// SIEHE AUCH: docs/SPEC.md, crates/contextra-graph/SPEC.md

use super::super::inner::sentinel_entity;
use super::CsrGraph;
use crate::error::Result;
use crate::types::Entity;


// SECTION Entity Core Implementation
impl CsrGraph {
    pub fn insert_entity_direct(&self, entity: Entity) -> Result<()> {
        let mut inner = self.inner_write();
        let idx = inner.get_or_create_index(entity.id);
        if idx >= inner.entities.len() {
            inner.entities.resize(idx + 1, sentinel_entity());
        }
        inner.entities[idx] = entity;
        Ok(())
    }

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
