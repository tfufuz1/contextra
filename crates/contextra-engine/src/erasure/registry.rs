//! Deterministic registry of secondary erasure layers.

use contextra_ports::erasure::{DocScope, ResolvedScope};
use contextra_ports::{ErasureLayer, WitnessLayer};
use contextra_types::TenantId;
use std::collections::BTreeMap;
use std::sync::Arc;

/// Stable purge order required by the engine's cross-layer dependencies.
///
/// Text needs the privileged `fw:` reads, graph needs the provenance index in the store; both run
/// before the store purge. Checkpoints run last because their clones hold data at file level.
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
    /// Canonical registry key of a tenant collection (the registry is keyed per tenant).
    pub fn collection_key(tenant: TenantId, collection: &str) -> String {
        format!("t{}:{}", tenant.0, collection)
    }

    /// Registers (or atomically replaces) a layer.
    pub fn register(
        &mut self,
        order: LayerOrder,
        collection: impl Into<String>,
        layer: Arc<dyn ErasureLayer>,
    ) {
        self.layers.insert((order, collection.into()), layer);
    }

    /// Removes every layer registered for `collection` (registry key, see [`Self::collection_key`]).
    pub fn remove_collection(&mut self, collection: &str) {
        self.layers.retain(|(_, name), _| name != collection);
    }

    /// All layers in deterministic `(order, collection)` order.
    pub fn ordered(&self) -> Vec<Arc<dyn ErasureLayer>> {
        self.layers.values().cloned().collect()
    }

    /// Layers participating in `scope`, in purge order. A tenant scope selects every layer
    /// of that tenant; collection and document scopes select that collection's layers.
    pub fn layers_for(&self, scope: &ResolvedScope) -> Vec<(LayerOrder, Arc<dyn ErasureLayer>)> {
        let mut wanted_exact: Vec<String> = Vec::new();
        let mut wanted_tenants: Vec<TenantId> = Vec::new();
        for d in &scope.docs {
            match d {
                DocScope::Set { tenant, collection, .. }
                | DocScope::Collection { tenant, collection } => {
                    wanted_exact.push(Self::collection_key(*tenant, collection));
                }
                DocScope::Tenant { tenant } => wanted_tenants.push(*tenant),
            }
        }
        self.layers
            .iter()
            .filter(|((_, key), _)| {
                wanted_exact.iter().any(|w| w == key)
                    || wanted_tenants
                        .iter()
                        .any(|t| key.starts_with(&format!("t{}:", t.0)))
            })
            .map(|((order, _), layer)| (*order, Arc::clone(layer)))
            .collect()
    }

    /// Witness layers of the registered layers (order preserved, duplicates removed).
    pub fn expected_layers(&self) -> Vec<WitnessLayer> {
        let mut out: Vec<WitnessLayer> = Vec::new();
        for layer in self.layers.values() {
            let l = layer.layer();
            if !out.contains(&l) {
                out.push(l);
            }
        }
        out
    }

    /// Number of registered layers.
    pub fn len(&self) -> usize {
        self.layers.len()
    }

    /// Whether no layer is registered.
    pub fn is_empty(&self) -> bool {
        self.layers.is_empty()
    }
}
