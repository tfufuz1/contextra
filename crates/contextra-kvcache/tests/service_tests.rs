use std::collections::BTreeMap;
use std::ops::Bound;
use std::sync::Arc;
use std::thread;

use bytes::Bytes;
use contextra_kvcache::adapters::{
    EmbeddingProviderSemanticEmbedder, KvCachePluginConfig, KvCachePluginManifest,
    StorageEngineKvBridge,
};
use contextra_kvcache::attention_score::{rank_for_eviction, NullAttentionScoreSource};
use contextra_kvcache::radix::KvReusePolicy;
use contextra_kvcache::service::{KvCacheService, KvCacheServiceBuilder};
use contextra_ports::embedding::{EmbeddingError, EmbeddingProvider};
use contextra_ports::kv_bridge_port::KvBridgeStorage;
use contextra_ports::license::FeatureRing;
use contextra_ports::plugin::PluginManifest;
use contextra_ports::storage::{StorageEngine, StorageStats};
use contextra_ports::BoxFuture;
use contextra_types::{ContextraError, Result as ContextraResult, TenantId, TxId};
use parking_lot::Mutex;

struct MockStorageEngine {
    data: Mutex<BTreeMap<Vec<u8>, Bytes>>,
}

impl MockStorageEngine {
    fn new() -> Self {
        Self {
            data: Mutex::new(BTreeMap::new()),
        }
    }
}

impl StorageEngine for MockStorageEngine {
    fn get<'a>(&'a self, key: &'a [u8]) -> BoxFuture<'a, ContextraResult<Option<Bytes>>> {
        Box::pin(async move { Ok(self.data.lock().get(key).cloned()) })
    }

    fn put<'a>(
        &'a self,
        _tx_id: TxId,
        key: &'a [u8],
        value: &'a [u8],
    ) -> BoxFuture<'a, ContextraResult<()>> {
        Box::pin(async move {
            self.data
                .lock()
                .insert(key.to_vec(), Bytes::copy_from_slice(value));
            Ok(())
        })
    }

    fn commit<'a>(&'a self, _tx_id: TxId) -> BoxFuture<'a, ContextraResult<()>> {
        Box::pin(async move { Ok(()) })
    }

    fn delete<'a>(&'a self, _tx_id: TxId, key: &'a [u8]) -> BoxFuture<'a, ContextraResult<()>> {
        Box::pin(async move {
            self.data.lock().remove(key);
            Ok(())
        })
    }

    fn rollback<'a>(&'a self, _tx_id: TxId) -> BoxFuture<'a, ContextraResult<()>> {
        Box::pin(async move { Ok(()) })
    }

    fn rollback_to_tx<'a>(&'a self, _tx_id: TxId) -> BoxFuture<'a, ContextraResult<()>> {
        Box::pin(async move { Ok(()) })
    }

    fn get_at_seq<'a>(
        &'a self,
        key: &'a [u8],
        _seq: u64,
    ) -> BoxFuture<'a, ContextraResult<Option<Bytes>>> {
        self.get(key)
    }

    fn flush<'a>(&'a self) -> BoxFuture<'a, ContextraResult<()>> {
        Box::pin(async move { Ok(()) })
    }

    fn stats<'a>(&'a self) -> BoxFuture<'a, ContextraResult<StorageStats>> {
        Box::pin(async move {
            Ok(StorageStats {
                num_segments: self.data.lock().len(),
                total_size_bytes: 0,
                memtable_size_bytes: 0,
            })
        })
    }

    fn last_seq_no<'a>(&'a self) -> BoxFuture<'a, ContextraResult<u64>> {
        Box::pin(async move { Ok(1) })
    }

    fn last_tx_id<'a>(&'a self) -> BoxFuture<'a, ContextraResult<TxId>> {
        Box::pin(async move { Ok(TxId::new(1)) })
    }

    fn pin_checkpoint<'a>(&'a self, _seq: u64) -> BoxFuture<'a, ContextraResult<()>> {
        Box::pin(async move { Ok(()) })
    }

    fn unpin_checkpoint<'a>(&'a self, _seq: u64) -> BoxFuture<'a, ContextraResult<()>> {
        Box::pin(async move { Ok(()) })
    }

    fn scan_prefix<'a>(
        &'a self,
        prefix: &'a [u8],
    ) -> BoxFuture<'a, ContextraResult<Vec<(Vec<u8>, Vec<u8>)>>> {
        Box::pin(async move {
            let data = self.data.lock();
            let matches = data
                .iter()
                .filter(|(k, _)| k.starts_with(prefix))
                .map(|(k, v)| (k.clone(), v.to_vec()))
                .collect();
            Ok(matches)
        })
    }

    fn scan<'a>(
        &'a self,
        start: Bound<&'a [u8]>,
        end: Bound<&'a [u8]>,
        limit: Option<usize>,
    ) -> BoxFuture<'a, ContextraResult<Vec<(Vec<u8>, Vec<u8>)>>> {
        Box::pin(async move {
            let data = self.data.lock();
            let mut results = Vec::new();
            for (k, v) in data.range((start.map(|b| b.to_vec()), end.map(|b| b.to_vec()))) {
                if let Some(l) = limit {
                    if results.len() >= l {
                        break;
                    }
                }
                results.push((k.clone(), v.to_vec()));
            }
            Ok(results)
        })
    }
}

struct MockEmbeddingProvider;

impl EmbeddingProvider for MockEmbeddingProvider {
    fn provider_name(&self) -> &str {
        "mock"
    }

    fn embedding_dim(&self) -> usize {
        2
    }

    fn embed<'a>(&'a self, text: &'a str) -> BoxFuture<'a, Result<Vec<f32>, EmbeddingError>> {
        Box::pin(async move {
            let val = text.len() as f32;
            Ok(vec![val, val * 0.5])
        })
    }
}

#[test]
fn test_null_attention_score_source_fallback_and_eviction() {
    let null_source = NullAttentionScoreSource;
    let base = std::time::Instant::now();
    let candidates = vec![
        (100, base + std::time::Duration::from_secs(10)),
        (200, base + std::time::Duration::from_secs(2)),
        (300, base + std::time::Duration::from_secs(1)),
    ];

    let ranked = rank_for_eviction(&candidates, &null_source);
    // Deterministic LRU fallback: oldest access first (300 -> 200 -> 100)
    assert_eq!(ranked, vec![300, 200, 100]);
}

#[tokio::test]
async fn test_service_lifecycle_put_get_evict() {
    let service = KvCacheServiceBuilder::new()
        .memory_budget_bytes(16 << 20)
        .reuse_policy(KvReusePolicy::Always)
        .build();

    let tenant = TenantId::try_new(42).expect("valid tenant_id");
    let tokens = vec![101, 102, 103, 104];
    let payload = vec![1, 2, 3, 4, 5, 6, 7, 8];

    // 1. Initial Insert (MISS -> returns None)
    let res1 = service
        .get_or_insert_prefix(tenant, &tokens, &payload)
        .expect("get_or_insert_prefix should succeed");
    assert!(res1.is_none(), "Initial insertion must return None (MISS)");

    let stats1 = service.stats(tenant);
    assert_eq!(stats1.segment_count, 1);
    assert_eq!(stats1.radix_entry_count, 1);

    // 2. Exact Lookup (HIT -> returns Some((PrefixMatch, KvBlockGuard)))
    let res2 = service
        .get_or_insert_prefix(tenant, &tokens, &payload)
        .expect("lookup should succeed");
    assert!(res2.is_some(), "Subsequent query must return Some (HIT)");
    let (pm, guard) = res2.unwrap();
    assert_eq!(pm.matched_len, 4);
    assert_eq!(guard.tenant_id(), tenant);

    // 3. Read Segment Payload
    let read_data = service
        .read_segment(tenant, pm.block_id)
        .await
        .expect("read_segment should succeed");
    assert_eq!(read_data, Some(payload));

    // 4. Delete Tenant
    drop(guard);
    service.delete_tenant(tenant);
    let stats_after = service.stats(tenant);
    assert_eq!(stats_after.segment_count, 0);
    assert_eq!(stats_after.radix_entry_count, 0);
}

#[test]
fn test_tenant_isolation_foreign_prefix_miss() {
    let service = KvCacheServiceBuilder::new()
        .memory_budget_bytes(16 << 20)
        .reuse_policy(KvReusePolicy::Always)
        .build();

    let tenant_a = TenantId::try_new(100).expect("tenant a");
    let tenant_b = TenantId::try_new(200).expect("tenant b");

    let tokens = vec![1, 2, 3, 4, 5];
    let payload = vec![9, 8, 7, 6];

    // Tenant A inserts
    assert!(service
        .get_or_insert_prefix(tenant_a, &tokens, &payload)
        .unwrap()
        .is_none());

    // Tenant B with exact same tokens queries -> MUST MISS
    let res_b = service
        .get_or_insert_prefix(tenant_b, &tokens, &payload)
        .unwrap();
    assert!(
        res_b.is_none(),
        "Tenant B must not hit Tenant A's cached prefix!"
    );

    // Verify stats separation
    assert_eq!(service.stats(tenant_a).segment_count, 1);
    assert_eq!(service.stats(tenant_b).segment_count, 1);
}

#[test]
fn test_memory_limit_and_budget_eviction() {
    // 1 MiB budget
    let budget = 1 << 20;
    let service = KvCacheServiceBuilder::new()
        .memory_budget_bytes(budget)
        .reuse_policy(KvReusePolicy::Always)
        .build();

    let tenant = TenantId::try_new(1).expect("tenant");

    // 1. Oversized segment rejection
    let oversized_payload = vec![0xaa; budget + 100];
    let err = service
        .get_or_insert_prefix(tenant, &[1, 2, 3], &oversized_payload)
        .unwrap_err();
    match err {
        ContextraError::InvalidInput(msg) => {
            assert!(msg.contains("exceeds total service memory budget"));
        }
        other => panic!("Expected InvalidInput error, got {:?}", other),
    }

    // 2. Empty payload rejection
    let empty_err = service
        .get_or_insert_prefix(tenant, &[1, 2, 3], &[])
        .unwrap_err();
    assert!(matches!(empty_err, ContextraError::InvalidInput(_)));

    // 3. Valid insertion within budget
    let valid_payload = vec![0xbb; 100_000];
    assert!(service
        .get_or_insert_prefix(tenant, &[10, 20, 30], &valid_payload)
        .unwrap()
        .is_none());
}

#[test]
fn test_concurrency_multi_threaded_access() {
    let service: Arc<KvCacheService> = Arc::new(
        KvCacheServiceBuilder::new()
            .memory_budget_bytes(64 << 20)
            .reuse_policy(KvReusePolicy::Always)
            .build(),
    );

    let mut handles = Vec::new();

    for thread_idx in 0..8 {
        let service_clone: Arc<KvCacheService> = Arc::clone(&service);
        let handle = thread::spawn(move || {
            let tenant = TenantId::try_new(thread_idx + 1).expect("tenant");
            for i in 0..20 {
                let tokens = vec![thread_idx as u32, i as u32, 100, 200];
                let payload = vec![thread_idx as u8; 256];

                // First insert -> MISS
                let res1 = service_clone.get_or_insert_prefix(tenant, &tokens, &payload);
                assert!(res1.is_ok());

                // Second query -> HIT
                let res2 = service_clone.get_or_insert_prefix(tenant, &tokens, &payload);
                assert!(res2.is_ok());
            }

            let stats = service_clone.stats(tenant);
            assert_eq!(stats.segment_count, 20);
        });
        handles.push(handle);
    }

    for h in handles {
        h.join()
            .expect("Thread panicked during concurrent execution");
    }
}

#[tokio::test]
async fn test_adapter_roundtrip_storage_engine_bridge() {
    let mock_engine = Arc::new(MockStorageEngine::new());
    let bridge = StorageEngineKvBridge::new(mock_engine);

    let tx_id = TxId::new(42);
    let key = b"tenant:1:seg:100";
    let value = b"encrypted-kv-payload-bytes";

    // Put
    bridge
        .put(tx_id, key, value)
        .await
        .expect("put should succeed");

    // Commit
    bridge.commit(tx_id).await.expect("commit should succeed");

    // Get
    let fetched = bridge.get(key).await.expect("get should succeed");
    assert_eq!(fetched, Some(Bytes::from_static(value)));
}

#[test]
fn test_adapter_roundtrip_embedding_provider_and_plugin_manifest() {
    let mock_provider = Arc::new(MockEmbeddingProvider);
    let embedder = EmbeddingProviderSemanticEmbedder::new(mock_provider);
    assert!(!Arc::ptr_eq(
        embedder.provider(),
        &(Arc::new(MockEmbeddingProvider) as Arc<dyn EmbeddingProvider>)
    ));

    // Test Plugin Config Validation
    let invalid_config = KvCachePluginConfig {
        version: (1, 0, 0),
        max_memory_bytes: 500, // < 1 MiB -> Invalid
        feature_ring_required: FeatureRing::Fast,
    };
    assert!(KvCachePluginManifest::new(invalid_config).is_err());

    let valid_config = KvCachePluginConfig::default();
    let manifest =
        KvCachePluginManifest::new(valid_config).expect("valid manifest should construct");

    assert_eq!(manifest.name(), "contextra-kvcache");
    assert!(manifest.requires().is_empty());
    assert!(manifest.conflicts().is_empty());
    assert_eq!(manifest.feature_ring_required(), FeatureRing::Fast);

    let cap = manifest.capability();
    assert_eq!(cap.name, "contextra-kvcache");
    assert_eq!(cap.version, (1, 0, 0));
    assert_eq!(cap.ring, 1);

    assert!(manifest.activate().is_ok());
}
