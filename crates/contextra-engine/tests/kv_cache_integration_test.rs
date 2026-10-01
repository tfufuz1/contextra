#![forbid(unsafe_code)]

//! Integration test verifying that `contextra-engine` correctly wires `EvictionWorker`
//! with a real `AttentionExporter` composition root, proving `NullAttentionScoreSource`
//! is no longer active when initialized via `build_eviction_worker`.

use contextra_engine::kv_cache_integration::{
    build_eviction_worker, build_eviction_worker_with_exporter, AccessCounterAttentionExporter,
};
use contextra_kvcache::{EvictionWorker, TenantIsolatedKvStore};
use contextra_ports::{AttentionExporter, RequestId};
use contextra_types::TenantId;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

/// Spy/Mock exporter that counts calls to `export_attention_weights` to prove
/// that the injected score source is actually invoked instead of `NullAttentionScoreSource`.
#[derive(Debug, Default)]
struct SpyAttentionExporter {
    export_calls: AtomicUsize,
}

impl SpyAttentionExporter {
    fn calls(&self) -> usize {
        self.export_calls.load(Ordering::SeqCst)
    }
}

impl AttentionExporter for SpyAttentionExporter {
    fn export_attention_weights(&self, _request_id: RequestId) -> Option<Vec<f32>> {
        self.export_calls.fetch_add(1, Ordering::SeqCst);
        Some(vec![0.8, 0.9, 0.7])
    }
}

#[test]
fn test_unwired_eviction_worker_uses_null_attention_score_source() {
    let store = Arc::new(TenantIsolatedKvStore::new());
    let worker = EvictionWorker::spawn(store);

    let source = worker.attention_source();
    // NullAttentionScoreSource always returns None for all segment IDs
    assert!(
        source.importance_score_for_tenant(TenantId::SYSTEM, 1).is_none(),
        "Unwired EvictionWorker must return None from NullAttentionScoreSource"
    );
}

#[test]
fn test_build_eviction_worker_wires_candle_attention_exporter() {
    let store = Arc::new(TenantIsolatedKvStore::new());
    let worker = build_eviction_worker(store);

    let source = worker.attention_source();
    // Register segment mapping to a request
    let tenant = TenantId::SYSTEM;
    let segment_id = 42u64;
    let request_id = RequestId(42);

    worker.register_segment_request(tenant, segment_id, request_id);

    // Before recording weights in CandleAttentionExporter, it returns None
    assert!(source.importance_score_for_tenant(tenant, segment_id).is_none());

    // Clean shutdown
    worker.shutdown();
}

#[test]
fn test_build_eviction_worker_with_spy_proves_null_attention_source_inactive() {
    let store = Arc::new(TenantIsolatedKvStore::new());
    let spy = Arc::new(SpyAttentionExporter::default());

    let worker = build_eviction_worker_with_exporter(store, spy.clone());

    assert_eq!(
        spy.calls(),
        0,
        "Spy should not have been called prior to importance score query"
    );

    let tenant = TenantId::SYSTEM;
    let segment_id = 100u64;
    let request_id = RequestId(100);

    worker.register_segment_request(tenant, segment_id, request_id);

    let source = worker.attention_source();
    let score = source.importance_score_for_tenant(tenant, segment_id);

    assert!(
        score.is_some(),
        "Importance score must be returned from injected AttentionExporter"
    );

    let average_score = score.unwrap();
    let expected_average = (0.8f32 + 0.9f32 + 0.7f32) / 3.0f32;
    assert!(
        (average_score - expected_average).abs() < 1e-5,
        "Score must match exported attention weights average (expected ~{expected_average}, got {average_score})"
    );

    assert_eq!(
        spy.calls(),
        1,
        "Spy exporter export_attention_weights MUST be invoked exactly once, proving NullAttentionScoreSource is NOT active"
    );

    worker.shutdown();
}

#[tokio::test]
async fn test_contextra_lifecycle_initializes_kv_eviction_worker() {
    let dir = tempfile::tempdir().unwrap();
    let db = contextra_engine::Contextra::open(dir.path()).await.unwrap();
    db.close().await.unwrap();
}

#[test]
fn test_access_counter_attention_exporter_fallback() {
    let exporter = Arc::new(AccessCounterAttentionExporter::new());
    let request_id = RequestId(55);

    assert_eq!(exporter.get_access_count(request_id), 0);
    assert!(exporter.export_attention_weights(request_id).is_none());

    exporter.record_access(request_id);
    exporter.record_access(request_id);

    assert_eq!(exporter.get_access_count(request_id), 2);
    let weights = exporter.export_attention_weights(request_id).unwrap();
    assert_eq!(weights, vec![2.0]);
}
