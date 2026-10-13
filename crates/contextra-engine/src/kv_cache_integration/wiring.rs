// FILE-CONTEXT
// ZWECK: KvCacheIntegrationWiring — Facade wiring entry point for contextra-engine KV cache integration.
// STAND: TS:2026-10-13T00:00:00Z

#![forbid(unsafe_code)]

use crate::kv_cache_integration::{
    build_eviction_worker, AccessCounterAttentionExporter, KvEngineShreddingAdapter,
};
use contextra_kvcache::{EvictionWorker, KvShreddingWiring, TenantIsolatedKvStore};
use contextra_ports::{AttentionExporter, KvLifecycleHooks};
use contextra_types::ContextraError;
use std::sync::Arc;

/// Dependencies required to construct [`KvCacheIntegrationWiring`].
pub struct KvCacheIntegrationWiringDeps {
    /// Active KV store instance (if configured).
    pub kv_store: Option<Arc<TenantIsolatedKvStore>>,
    /// Crypto-shredding wiring instance.
    pub shredding_wiring: Arc<KvShreddingWiring>,
    /// Optional custom attention exporter.
    pub attention_exporter: Option<Arc<dyn AttentionExporter>>,
}

/// Facade entry point for KV cache integration in `contextra-engine`.
pub struct KvCacheIntegrationWiring {
    /// Active composite lifecycle & crypto-shredding hook adapter.
    pub hooks: Arc<KvEngineShreddingAdapter>,
    /// Background eviction worker (if a KV store was provided).
    pub eviction_worker: Option<EvictionWorker>,
    /// Active attention exporter.
    pub attention_exporter: Arc<dyn AttentionExporter>,
}

impl KvCacheIntegrationWiring {
    /// Constructs and wires the KV cache integration components per Directive 7.
    pub fn build(deps: KvCacheIntegrationWiringDeps) -> Result<Self, ContextraError> {
        let hooks = Arc::new(KvEngineShreddingAdapter::new(
            deps.shredding_wiring,
            deps.kv_store.clone(),
        ));

        let attention_exporter: Arc<dyn AttentionExporter> = deps
            .attention_exporter
            .unwrap_or_else(|| Arc::new(AccessCounterAttentionExporter::new()));

        let eviction_worker = deps.kv_store.map(build_eviction_worker);

        Ok(Self {
            hooks,
            eviction_worker,
            attention_exporter,
        })
    }

    /// Returns a cloned reference to the [`KvLifecycleHooks`] implementation.
    pub fn kv_hooks(&self) -> Arc<dyn KvLifecycleHooks> {
        self.hooks.clone()
    }
}
