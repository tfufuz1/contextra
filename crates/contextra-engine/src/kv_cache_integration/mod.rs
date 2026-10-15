#![forbid(unsafe_code)]

//! KV Cache integration and composition root module for `contextra-engine`.
//!
//! Provides factory, lifecycle, crypto-shredding hooks, and metrics ranking functions
//! to integrate [`contextra_kvcache`] with `contextra-engine`.

pub mod wiring;

use contextra_infer_candle::CandleAttentionExporter;
use contextra_kvcache::{EvictionWorker, KvShreddingWiring, TenantIsolatedKvStore};
use contextra_ports::{AttentionExporter, KvLifecycleHooks, RequestId};
use contextra_types::{ContextraError, DocId, TenantId};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;

/// Source provenance classification for attention weight scores.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AttentionSource {
    /// Attention scores derived directly from ML model transformer weights.
    ModelWeights,
    /// Fallback attention scores calculated from request access counters.
    AccessCounter,
}

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

    /// Returns the source classification for this exporter.
    pub fn attention_source(&self) -> AttentionSource {
        AttentionSource::AccessCounter
    }
}

impl AttentionExporter for AccessCounterAttentionExporter {
    fn export_attention_weights(&self, request_id: RequestId) -> Option<Vec<f32>> {
        let mut count = self.get_access_count(request_id);
        if count == 0 {
            self.record_access(request_id);
            count = 1;
        }
        Some(vec![count as f32])
    }
}

/// Scope definition for crypto-shredding KV segment keys.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KvShredScope {
    /// Single document deletion (e.g., `forget_document`).
    Document(DocId),
    /// Entire collection deletion.
    Collection(String),
    /// Entire tenant deletion.
    Tenant,
}

/// Hook trait allowing deletion mechanisms in `deletion/` or `collection/`
/// to trigger crypto-shredding without modifying those modules (W1-09 / FIND-01).
pub trait KvShreddingHook: Send + Sync + 'static {
    /// Triggers crypto-shredding of KV segment keys for the given tenant and scope.
    fn shred(&self, tenant_id: TenantId, scope: KvShredScope) -> Result<usize, ContextraError>;
}

/// Composite hook adapter implementing both [`KvLifecycleHooks`] and [`KvShreddingHook`].
pub struct KvEngineShreddingAdapter {
    shredding_wiring: Arc<KvShreddingWiring>,
    kv_store: Option<Arc<TenantIsolatedKvStore>>,
}

impl KvEngineShreddingAdapter {
    /// Creates a new `KvEngineShreddingAdapter`.
    pub fn new(
        shredding_wiring: Arc<KvShreddingWiring>,
        kv_store: Option<Arc<TenantIsolatedKvStore>>,
    ) -> Self {
        Self {
            shredding_wiring,
            kv_store,
        }
    }

    /// Explicitly invokes crypto-shredding for a document, collection, or tenant.
    pub fn shred(&self, tenant_id: TenantId, scope: KvShredScope) -> Result<usize, ContextraError> {
        match scope {
            KvShredScope::Document(doc_id) => {
                let segment_id = match doc_id {
                    DocId(id) => id as u64,
                };
                let shredded = self.shredding_wiring.shred_group(tenant_id, segment_id)?;
                Ok(if shredded { 1 } else { 0 })
            }
            KvShredScope::Collection(ref col_name) => self
                .shredding_wiring
                .handle_collection_deletion(tenant_id, col_name),
            KvShredScope::Tenant => self.shredding_wiring.handle_tenant_deletion(tenant_id),
        }
    }
}

impl KvShreddingHook for KvEngineShreddingAdapter {
    fn shred(&self, tenant_id: TenantId, scope: KvShredScope) -> Result<usize, ContextraError> {
        self.shred(tenant_id, scope)
    }
}

impl KvLifecycleHooks for KvEngineShreddingAdapter {
    fn remove_doc_segments(&self, tenant_id: TenantId, doc_id: DocId) {
        if let Some(ref store) = self.kv_store {
            store.remove_doc_segments(tenant_id, doc_id);
        }
        let _ = self.shred(tenant_id, KvShredScope::Document(doc_id));
    }

    fn purge_tenant(&self, tenant_id: TenantId) {
        if let Some(ref store) = self.kv_store {
            store.purge_tenant(tenant_id);
        }
        let _ = self.shred(tenant_id, KvShredScope::Tenant);
    }

    fn on_rollback(&self, tenant_id: TenantId, chunk_ids: &[u64]) {
        if let Some(ref store) = self.kv_store {
            store.on_rollback(tenant_id, chunk_ids);
        }
    }
}

/// Evaluates and rescores items during top-k re-ranking using user metrics.
///
/// **Safety & Architecture Requirement**: User metrics (such as access counts, CTR, or engagement scores)
/// MUST strictly be evaluated only during top-k re-ranking and NEVER during graph/vector index traversal.
pub fn apply_user_metrics_topk_rerank<T>(
    candidates: &mut [(T, f32)],
    metric_scores: &HashMap<RequestId, f32>,
    request_map: impl Fn(&T) -> Option<RequestId>,
    weight: f32,
) {
    for (candidate, score) in candidates.iter_mut() {
        if let Some(req_id) = request_map(candidate) {
            if let Some(&user_metric) = metric_scores.get(&req_id) {
                *score += user_metric * weight;
            }
        }
    }
    candidates.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
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

#[cfg(test)]
mod tests {
    use super::*;
    use contextra_crypto::kv_shredding::KeyRegistry;
    use contextra_kvcache::segment::{ShreddableSegmentKey, Tier2EncryptedSegment};
    use contextra_kvcache::KvShreddingWiringDeps;

    #[test]
    fn test_forget_document_shreds_kv_segment() {
        let key_registry = Arc::new(KeyRegistry::new());
        let shred_deps = KvShreddingWiringDeps {
            key_registry: Arc::clone(&key_registry),
        };
        let shredding_wiring = Arc::new(KvShreddingWiring::build(shred_deps).unwrap());

        let tenant = TenantId::try_new(42).unwrap();
        let doc_id = DocId(1001);
        let segment_id = match doc_id {
            DocId(id) => id as u64,
        };

        let key = ShreddableSegmentKey::try_new_random("secret-passphrase").unwrap();
        let segment =
            Tier2EncryptedSegment::new(tenant, segment_id, b"sensitive KV payload", key.clone())
                .unwrap();

        shredding_wiring.register_segment_key(tenant, segment_id, key);

        // Before shredding, segment payload is readable
        assert_eq!(segment.read_and_decrypt().unwrap(), b"sensitive KV payload");

        let adapter = KvEngineShreddingAdapter::new(shredding_wiring.clone(), None);

        // Execute forget_document simulation via KvShreddingHook / KvLifecycleHooks
        adapter.remove_doc_segments(tenant, doc_id);

        // After forget_document, segment key is zeroized and unreadable
        assert!(segment.read_and_decrypt().is_err());
        assert!(shredding_wiring.is_shredded(tenant, segment_id));
    }

    #[test]
    fn test_access_counter_attention_source_flag() {
        let exporter = AccessCounterAttentionExporter::new();
        assert_eq!(exporter.attention_source(), AttentionSource::AccessCounter);
    }

    #[test]
    fn test_user_metrics_topk_rerank() {
        let req1 = RequestId(1);
        let req2 = RequestId(2);

        let mut candidates = vec![("doc_a", 0.8f32), ("doc_b", 0.9f32)];

        let mut metrics = HashMap::new();
        metrics.insert(req1, 0.5f32); // boosts doc_a by 0.5 * 0.4 = 0.2 -> 1.0 total

        apply_user_metrics_topk_rerank(
            &mut candidates,
            &metrics,
            |doc| match *doc {
                "doc_a" => Some(req1),
                "doc_b" => Some(req2),
                _ => None,
            },
            0.4,
        );

        assert_eq!(candidates[0].0, "doc_a");
        assert!((candidates[0].1 - 1.0).abs() < 1e-4);
        assert_eq!(candidates[1].0, "doc_b");
    }
}
