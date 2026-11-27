//! Deterministic registry of secondary erasure layers.

use contextra_ports::{ErasureLayer, WitnessLayer};
use std::collections::BTreeMap;
use std::sync::Arc;

/// Stable purge order required by the engine's cross-layer dependencies.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum LayerOrder {
    Text = 10,
    Graph = 20,
    Vector = 30,
    Checkpoint = 40,
}

/// Ordered, collection-scoped registry. Replacing a registration is atomic.
#[derive(Default, Clone)]
pub struct LayerRegistry {
    layers: BTreeMap<(LayerOrder, String), Arc<dyn ErasureLayer>>,
}

impl LayerRegistry {
    pub fn register(
        &mut self,
        order: LayerOrder,
        collection: impl Into<String>,
        layer: Arc<dyn ErasureLayer>,
    ) {
        self.layers.insert((order, collection.into()), layer);
    }

    pub fn remove_collection(&mut self, collection: &str) {
        self.layers.retain(|(_, name), _| name != collection);
    }

    pub fn ordered(&self) -> Vec<Arc<dyn ErasureLayer>> {
        self.layers.values().cloned().collect()
    }

    pub fn expected_layers(&self) -> Vec<WitnessLayer> {
        self.layers.values().map(|layer| layer.layer()).collect()
    }

    pub fn len(&self) -> usize { self.layers.len() }
    pub fn is_empty(&self) -> bool { self.layers.is_empty() }
}
