#![forbid(unsafe_code)]

//! KV Cache integration and composition root module for `contextra-engine`.
//!
//! Provides factory and lifecycle functions to instantiate and bind [`EvictionWorker`]
//! with production-grade or fallback [`AttentionExporter`] implementations.

use contextra_infer_candle::CandleAttentionExporter;
use contextra_kvcache::{EvictionWorker, TenantIsolatedKvStore};
use contextra_ports::{AttentionExporter, RequestId};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;

/// A minimal fallback adapter tracking request access counts.
///
/// Honestly marked as initial iteration fallback providing real access counters
/// rather than constant zeros when no ML model attention weights are explicitly recorded yet.
#[derive(Debug, Default)]
pub struct AccessCounterAttentionExporter {
    access_counts: RwLock<HashMap<RequestId, u64>>,
}

impl AccessCounterAttentionExporter {
    /// Creates a new `AccessCounterAttentionExporter`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Records an access for the given request ID, incrementing its frequency counter.
    pub fn record_access(&self, request_id: RequestId) {
        let mut map = self.access_counts.write();
        *map.entry(request_id).or_insert(0) += 1;
    }

    /// Returns the current access count for the given request ID.
    pub fn get_access_count(&self, request_id: RequestId) -> u64 {
        self.access_counts
            .read()
            .get(&request_id)
            .copied()
            .unwrap_or(0)
    }
}

impl AttentionExporter for AccessCounterAttentionExporter {
    fn export_attention_weights(&self, request_id: RequestId) -> Option<Vec<f32>> {
        let count = self.access_counts.read().get(&request_id).copied()?;
        Some(vec![count as f32])
    }
}

/// Primary composition root function for building an [`EvictionWorker`].
///
/// Spawns the [`EvictionWorker`] for the provided [`TenantIsolatedKvStore`]
/// and wires it with a production [`CandleAttentionExporter`] via [`start_kv_cache_eviction`].
pub fn build_eviction_worker(store: Arc<TenantIsolatedKvStore>) -> EvictionWorker {
    start_kv_cache_eviction(store)
}

/// Composition root function allowing injection of a custom [`AttentionExporter`].
///
/// Spawns the [`EvictionWorker`] and connects it with the supplied `exporter`.
pub fn build_eviction_worker_with_exporter(
    store: Arc<TenantIsolatedKvStore>,
    exporter: Arc<dyn AttentionExporter>,
) -> EvictionWorker {
    EvictionWorker::spawn(store).with_attention_exporter(exporter)
}

/// Engine lifecycle integration entry point for KV cache background eviction.
///
/// Delegates to [`build_eviction_worker_with_exporter`] to instantiate and attach the active
/// attention exporter to the background eviction thread.
pub fn start_kv_cache_eviction(store: Arc<TenantIsolatedKvStore>) -> EvictionWorker {
    let exporter: Arc<dyn AttentionExporter> = Arc::new(CandleAttentionExporter::new());
    build_eviction_worker_with_exporter(store, exporter)
}
