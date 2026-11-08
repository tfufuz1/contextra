//! KV-Bridge Adapter connecting Candle inference to tenant-isolated KV cache store.

use bytes::Bytes;
use contextra_ports::{ContextSegment, KvBridgeStorage};
use contextra_types::{ModelFingerprint, TenantId, TxId};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock};

/// Gekapselter KV-Payload zur Persistierung im Store mit Metadaten.
#[derive(Debug, Serialize, Deserialize)]
struct CachedKvPayload {
    fingerprint: ModelFingerprint,
    rope_offset: Option<usize>,
    data: Vec<u8>,
}

/// Cache-Lookup-Schlüssel: eindeutige Kombination aus Chunk-ID, Modell und optionalem RoPE-Offset.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KvCacheKey {
    pub chunk_id: u64,
    pub fingerprint: ModelFingerprint,
    pub rope_offset: Option<usize>,
}

impl KvCacheKey {
    pub fn new(chunk_id: u64, fingerprint: ModelFingerprint, rope_offset: Option<usize>) -> Self {
        Self {
            chunk_id,
            fingerprint,
            rope_offset,
        }
    }
}

/// Mandantenisolierter In-Memory KV-Cache-Speicher für Segment-Payloads (Tier 1).
pub struct InMemoryKvSegmentStore {
    segments: RwLock<HashMap<(TenantId, u64), Vec<u8>>>,
    capacity_per_tenant: usize,
    spill_handler: RwLock<Option<Arc<dyn Fn(TenantId, u64, Vec<u8>) + Send + Sync>>>,
}

impl std::fmt::Debug for InMemoryKvSegmentStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InMemoryKvSegmentStore")
            .field("capacity_per_tenant", &self.capacity_per_tenant)
            .finish()
    }
}

impl Default for InMemoryKvSegmentStore {
    fn default() -> Self {
        Self::new()
    }
}

impl InMemoryKvSegmentStore {
    /// Erstellt einen neuen Speicher mit unbegrenzter Kapazität.
    pub fn new() -> Self {
        Self {
            segments: RwLock::new(HashMap::new()),
            capacity_per_tenant: usize::MAX,
            spill_handler: RwLock::new(None),
        }
    }

    /// Erstellt einen neuen Speicher mit Kapazitätsgrenze pro Mandant für Eviction/Spill.
    pub fn with_capacity(capacity_per_tenant: usize) -> Self {
        Self {
            segments: RwLock::new(HashMap::new()),
            capacity_per_tenant,
            spill_handler: RwLock::new(None),
        }
    }

    /// Registriert einen Spill-Handler für eviktierte Segmente.
    pub fn set_spill_handler(
        &self,
        handler: Arc<dyn Fn(TenantId, u64, Vec<u8>) + Send + Sync>,
    ) {
        if let Ok(mut lock) = self.spill_handler.write() {
            *lock = Some(handler);
        }
    }

    /// Fügt ein Segment für einen Mandanten ein.
    pub fn insert_segment(&self, tenant: TenantId, chunk_id: u64, payload: Vec<u8>) {
        let mut evicted = None;
        if let Ok(mut lock) = self.segments.write() {
            let tenant_count = lock.keys().filter(|(t, _)| *t == tenant).count();
            if tenant_count >= self.capacity_per_tenant {
                if let Some(first_key) = lock.keys().find(|(t, _)| *t == tenant).cloned() {
                    if let Some(removed_val) = lock.remove(&first_key) {
                        evicted = Some((first_key.0, first_key.1, removed_val));
                    }
                }
            }
            lock.insert((tenant, chunk_id), payload);
        }

        if let Some((t, id, data)) = evicted {
            if let Ok(guard) = self.spill_handler.read() {
                if let Some(ref handler) = *guard {
                    handler(t, id, data);
                }
            }
        }
    }

    /// Ruft ein Segment für einen Mandanten ab.
    pub fn get_segment(&self, tenant: TenantId, chunk_id: u64) -> Option<Vec<u8>> {
        let lock = self.segments.read().ok()?;
        lock.get(&(tenant, chunk_id)).cloned()
    }
}

/// Verbindet Retrieval-Chunks mit dem mandantenisolierten KV-Cache.
///
/// # Fail-Safe-Garantie
/// Jede Methode fällt bei Fehler oder Cache-Miss transparent auf den normalen Prefill zurück.
/// Kein Fehler aus dem KV-Store darf eine Inferenz-Anfrage abbrechen.
#[derive(Clone)]
pub struct KvBridgeAdapter {
    pub store: Arc<InMemoryKvSegmentStore>,
    pub lsm_store: Option<Arc<dyn KvBridgeStorage>>,
    pub consultations: Arc<AtomicU64>,
}

impl KvBridgeAdapter {
    /// Erstellt einen neuen Adapter mit gegebenem In-Memory Store.
    pub fn new(store: Arc<InMemoryKvSegmentStore>) -> Self {
        Self {
            store,
            lsm_store: None,
            consultations: Arc::new(AtomicU64::new(0)),
        }
    }

    /// Erstellt einen neuen Adapter mit Zwei-Tier-Hierarchie (RAM Fast-Path + LSM-Spill Slow-Path via KvBridgeStorage).
    pub fn with_lsm_fallback(
        store: Arc<InMemoryKvSegmentStore>,
        lsm_store: Arc<dyn KvBridgeStorage>,
    ) -> Self {
        let lsm = Arc::clone(&lsm_store);
        let handler = Arc::new(
            move |tenant: TenantId, segment_id: u64, payload_bytes: Vec<u8>| {
                let lsm_inner = Arc::clone(&lsm);
                tokio::spawn(async move {
                    let spill_key =
                        format!("__kv_spill:{}:{:#x}", tenant.inner(), segment_id).into_bytes();
                    let tx_id = TxId(segment_id);
                    let bytes = Bytes::from(payload_bytes);
                    if let Err(e) = lsm_inner.put(tx_id, &spill_key, &bytes).await {
                        tracing::warn!(
                            tenant_id = tenant.inner(),
                            segment_id,
                            error = %e,
                            "KvBridgeAdapter: Failed to put spilled segment into LSM"
                        );
                        return;
                    }
                    if let Err(e) = lsm_inner.commit(tx_id).await {
                        tracing::warn!(
                            tenant_id = tenant.inner(),
                            segment_id,
                            error = %e,
                            "KvBridgeAdapter: Failed to commit spilled segment into LSM"
                        );
                    }
                });
            },
        );

        store.set_spill_handler(handler);

        Self {
            store,
            lsm_store: Some(lsm_store),
            consultations: Arc::new(AtomicU64::new(0)),
        }
    }

    /// Consults segment metadata and KV cache bridge state for a context segment.
    pub fn consult_segment<'a>(&self, segment: &ContextSegment<'a>) {
        self.consultations.fetch_add(1, Ordering::SeqCst);
        let _ = (
            segment.chunk_id,
            segment.text,
            segment.model_fingerprint,
            segment.rope_offset,
        );
    }

    /// Returns the number of segment consultations recorded.
    pub fn consultation_count(&self) -> u64 {
        self.consultations.load(Ordering::SeqCst)
    }

    /// Deserialisiert und validiert Payload-Bytes gegen den angegebenen Key.
    fn validate_payload(raw_bytes: &[u8], key: &KvCacheKey) -> Option<Vec<u8>> {
        let payload: CachedKvPayload = match bincode::deserialize(raw_bytes) {
            Ok(p) => p,
            Err(e) => {
                tracing::warn!(
                    chunk_id = key.chunk_id,
                    error = %e,
                    "KvBridgeAdapter: Deserialization failed — cache miss"
                );
                return None;
            }
        };

        if payload.fingerprint != key.fingerprint {
            tracing::warn!(
                chunk_id = key.chunk_id,
                "KvBridgeAdapter: Model fingerprint mismatch — cache miss"
            );
            return None;
        }

        if payload.rope_offset != key.rope_offset {
            tracing::warn!(
                chunk_id = key.chunk_id,
                expected_rope = ?key.rope_offset,
                cached_rope = ?payload.rope_offset,
                "KvBridgeAdapter: RoPE offset mismatch — cache miss"
            );
            return None;
        }

        Some(payload.data)
    }

    /// Versucht, ein gecachetes KV-Segment zu laden.
    ///
    /// Gibt `None` zurück bei Cache-Miss, Fingerprint-Mismatch oder Deserialisierungsfehler.
    /// NIEMALS wird ein Fehler propagiert — `None` bedeutet stets "voller Prefill".
    pub fn try_get_cached_segment(&self, tenant: TenantId, key: &KvCacheKey) -> Option<Vec<u8>> {
        let raw_bytes = self.store.get_segment(tenant, key.chunk_id)?;
        Self::validate_payload(&raw_bytes, key)
    }

    /// Versucht asynchron, ein gecachetes KV-Segment erst im RAM (Tier 1) und bei Miss im LSM-Store (Tier 2) zu laden.
    pub async fn try_get_cached_segment_async(
        &self,
        tenant: TenantId,
        key: &KvCacheKey,
    ) -> Option<Vec<u8>> {
        if let Some(bytes) = self.try_get_cached_segment(tenant, key) {
            return Some(bytes);
        }

        let lsm = self.lsm_store.as_ref()?;
        let spill_key = format!("__kv_spill:{}:{:#x}", tenant.inner(), key.chunk_id).into_bytes();

        let doc_bytes = match lsm.get(&spill_key).await {
            Ok(Some(bytes)) => bytes,
            _ => return None,
        };

        Self::validate_payload(&doc_bytes, key)
    }

    /// Versucht asynchron, ein gecachetes KV-Segment zu laden und als `Bytes` Slice (Zero-Copy) zurückzugeben.
    pub async fn try_get_cached_segment_async_bytes(
        &self,
        tenant: TenantId,
        key: &KvCacheKey,
    ) -> Option<Bytes> {
        self.try_get_cached_segment_async(tenant, key)
            .await
            .map(Bytes::from)
    }

    /// Speichert ein KV-Segment im Cache. Fehler werden geloggt, nie propagiert.
    pub fn store_segment(&self, tenant: TenantId, key: KvCacheKey, plaintext_kv_bytes: Vec<u8>) {
        let payload = CachedKvPayload {
            fingerprint: key.fingerprint.clone(),
            rope_offset: key.rope_offset,
            data: plaintext_kv_bytes,
        };

        let serialized_bytes = match bincode::serialize(&payload) {
            Ok(b) => b,
            Err(e) => {
                tracing::warn!(
                    chunk_id = key.chunk_id,
                    error = %e,
                    "KvBridgeAdapter: Serialization failed — segment not cached"
                );
                return;
            }
        };

        self.store
            .insert_segment(tenant, key.chunk_id, serialized_bytes);
    }
}

impl std::fmt::Debug for KvBridgeAdapter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("KvBridgeAdapter")
            .field("store", &"<InMemoryKvSegmentStore>")
            .field("lsm_store", &self.lsm_store.is_some())
            .finish()
    }
}

/// Erstellt einen Standard-Test-Adapter für Modultests.
pub fn create_test_kv_bridge() -> KvBridgeAdapter {
    let store = Arc::new(InMemoryKvSegmentStore::new());
    KvBridgeAdapter::new(store)
}

#[cfg(test)]
mod tests {
    use super::*;
    use contextra_ports::BoxFuture;
    use contextra_types::Result;
    use std::sync::Mutex;
    use std::thread;

    #[derive(Default)]
    struct MockKvStorage {
        data: Mutex<HashMap<Vec<u8>, Bytes>>,
    }

    impl KvBridgeStorage for MockKvStorage {
        fn get<'a>(&'a self, key: &'a [u8]) -> BoxFuture<'a, Result<Option<Bytes>>> {
            Box::pin(async move {
                let guard = match self.data.lock() {
                    Ok(g) => g,
                    Err(poisoned) => poisoned.into_inner(),
                };
                Ok(guard.get(key).cloned())
            })
        }

        fn put<'a>(
            &'a self,
            _tx_id: TxId,
            key: &'a [u8],
            value: &'a [u8],
        ) -> BoxFuture<'a, Result<()>> {
            Box::pin(async move {
                let mut guard = match self.data.lock() {
                    Ok(g) => g,
                    Err(poisoned) => poisoned.into_inner(),
                };
                guard.insert(key.to_vec(), Bytes::copy_from_slice(value));
                Ok(())
            })
        }

        fn commit<'a>(&'a self, _tx_id: TxId) -> BoxFuture<'a, Result<()>> {
            Box::pin(async move { Ok(()) })
        }
    }

    fn create_test_adapter() -> KvBridgeAdapter {
        create_test_kv_bridge()
    }

    fn dummy_fp() -> ModelFingerprint {
        ModelFingerprint::new([0x77u8; 32], "llama-3.2-1b.gguf", "Q4_K_M")
    }

    #[test]
    fn test_cache_miss_returns_none_without_panic() {
        let adapter = create_test_adapter();
        let tenant = TenantId::try_new(1).ok().unwrap();
        let key = KvCacheKey::new(999, dummy_fp(), None);

        let cached = adapter.try_get_cached_segment(tenant, &key);
        assert!(
            cached.is_none(),
            "Cache miss MUST return None without panic"
        );
    }

    #[test]
    fn test_roundtrip_store_then_get() {
        let adapter = create_test_adapter();
        let tenant = TenantId::try_new(10).ok().unwrap();
        let fp = dummy_fp();
        let chunk_id = 42;
        let payload = b"KV tensor keys and values plaintext cache payload".to_vec();
        let key = KvCacheKey::new(chunk_id, fp, Some(128));

        adapter.store_segment(tenant, key.clone(), payload.clone());

        let retrieved = adapter.try_get_cached_segment(tenant, &key);
        assert!(retrieved.is_some(), "Stored segment MUST be retrievable");
        assert_eq!(retrieved, Some(payload));
    }

    #[test]
    fn test_tenant_isolation() {
        let adapter = create_test_adapter();
        let tenant_a = TenantId::try_new(101).ok().unwrap();
        let tenant_b = TenantId::try_new(202).ok().unwrap();
        let fp = dummy_fp();
        let chunk_id = 1;
        let payload_a = b"Secret payload of Tenant A".to_vec();
        let key = KvCacheKey::new(chunk_id, fp, None);

        adapter.store_segment(tenant_a, key.clone(), payload_a.clone());

        let retrieved_a = adapter.try_get_cached_segment(tenant_a, &key);
        assert_eq!(retrieved_a, Some(payload_a));

        let retrieved_b = adapter.try_get_cached_segment(tenant_b, &key);
        assert!(
            retrieved_b.is_none(),
            "Tenant B MUST NOT access Tenant A's cached segment"
        );
    }

    #[test]
    fn test_rope_offset_mismatch_returns_cache_miss() {
        let adapter = create_test_adapter();
        let tenant = TenantId::try_new(10).ok().unwrap();
        let fp = dummy_fp();
        let chunk_id = 123;
        let payload = b"KV tensor keys and values plaintext cache payload".to_vec();

        let key_stored = KvCacheKey::new(chunk_id, fp.clone(), Some(128));
        adapter.store_segment(tenant, key_stored.clone(), payload.clone());

        let retrieved = adapter.try_get_cached_segment(tenant, &key_stored);
        assert_eq!(retrieved, Some(payload.clone()));

        let key_diff_rope = KvCacheKey::new(chunk_id, fp.clone(), Some(256));
        let res_diff = adapter.try_get_cached_segment(tenant, &key_diff_rope);
        assert!(
            res_diff.is_none(),
            "Lookup with different rope_offset MUST return None"
        );

        let key_no_rope = KvCacheKey::new(chunk_id, fp, None);
        let res_none = adapter.try_get_cached_segment(tenant, &key_no_rope);
        assert!(
            res_none.is_none(),
            "Lookup with None rope_offset MUST return None when stored with Some(128)"
        );
    }

    #[test]
    fn test_corrupt_payload_returns_none() {
        let adapter = create_test_adapter();
        let tenant = TenantId::try_new(55).ok().unwrap();
        let key = KvCacheKey::new(777, dummy_fp(), None);

        adapter.store.insert_segment(tenant, key.chunk_id, vec![0xFF, 0xFE, 0xFD, 0xFC]);

        let result = adapter.try_get_cached_segment(tenant, &key);
        assert!(
            result.is_none(),
            "Corrupt payload MUST return None without panic"
        );
    }

    #[test]
    fn test_with_lsm_fallback_without_lsm_store() {
        let store = Arc::new(InMemoryKvSegmentStore::new());
        let adapter = KvBridgeAdapter::new(store);

        assert!(adapter.lsm_store.is_none());
        let tenant = TenantId::try_new(1).ok().unwrap();
        let key = KvCacheKey::new(999, dummy_fp(), None);
        assert!(adapter.try_get_cached_segment(tenant, &key).is_none());
    }

    #[tokio::test]
    async fn test_lsm_fallback_ram_hit() {
        let store = Arc::new(InMemoryKvSegmentStore::new());
        let mock_store = Arc::new(MockKvStorage::default());

        let adapter = KvBridgeAdapter::with_lsm_fallback(store, mock_store);
        let tenant = TenantId::try_new(10).ok().unwrap();
        let fp = dummy_fp();
        let chunk_id = 42;
        let payload = b"RAM hit payload".to_vec();
        let key = KvCacheKey::new(chunk_id, fp, None);

        adapter.store_segment(tenant, key.clone(), payload.clone());

        let retrieved = adapter.try_get_cached_segment_async(tenant, &key).await;
        assert_eq!(retrieved, Some(payload));
    }

    #[tokio::test]
    async fn test_lsm_fallback_ram_miss_lsm_hit() {
        let store = Arc::new(InMemoryKvSegmentStore::new());
        let mock_store = Arc::new(MockKvStorage::default());

        let adapter =
            KvBridgeAdapter::with_lsm_fallback(store, Arc::clone(&mock_store) as _);
        let tenant = TenantId::try_new(10).ok().unwrap();
        let fp = dummy_fp();
        let chunk_id = 100;
        let payload = b"LSM hit payload".to_vec();
        let key = KvCacheKey::new(chunk_id, fp.clone(), None);

        let inner_payload = CachedKvPayload {
            fingerprint: fp.clone(),
            rope_offset: None,
            data: payload.clone(),
        };
        let serialized_bytes = bincode::serialize(&inner_payload).ok().unwrap();

        let spill_key = format!("__kv_spill:{}:{:#x}", tenant.inner(), chunk_id).into_bytes();
        let tx_id = TxId(chunk_id);
        mock_store.put(tx_id, &spill_key, &serialized_bytes).await.ok().unwrap();
        mock_store.commit(tx_id).await.ok().unwrap();

        assert!(adapter.try_get_cached_segment(tenant, &key).is_none());

        let retrieved = adapter.try_get_cached_segment_async(tenant, &key).await;
        assert_eq!(retrieved, Some(payload));
    }

    #[tokio::test]
    async fn test_lsm_fallback_ram_miss_lsm_miss() {
        let store = Arc::new(InMemoryKvSegmentStore::new());
        let mock_store = Arc::new(MockKvStorage::default());

        let adapter = KvBridgeAdapter::with_lsm_fallback(store, mock_store);
        let tenant = TenantId::try_new(10).ok().unwrap();
        let key = KvCacheKey::new(999, dummy_fp(), None);

        let retrieved = adapter.try_get_cached_segment_async(tenant, &key).await;
        assert!(retrieved.is_none());
    }

    #[tokio::test]
    async fn test_lsm_fallback_fingerprint_mismatch() {
        let store = Arc::new(InMemoryKvSegmentStore::new());
        let mock_store = Arc::new(MockKvStorage::default());

        let adapter =
            KvBridgeAdapter::with_lsm_fallback(store, Arc::clone(&mock_store) as _);
        let tenant = TenantId::try_new(10).ok().unwrap();
        let fp_stored = dummy_fp();
        let fp_lookup = ModelFingerprint::new([0x88u8; 32], "other-model.gguf", "Q8_0");
        let chunk_id = 200;
        let payload = b"LSM payload with mismatched fingerprint".to_vec();

        let inner_payload = CachedKvPayload {
            fingerprint: fp_stored.clone(),
            rope_offset: None,
            data: payload,
        };
        let serialized_bytes = bincode::serialize(&inner_payload).ok().unwrap();

        let spill_key = format!("__kv_spill:{}:{:#x}", tenant.inner(), chunk_id).into_bytes();
        let tx_id = TxId(chunk_id);
        mock_store.put(tx_id, &spill_key, &serialized_bytes).await.ok().unwrap();
        mock_store.commit(tx_id).await.ok().unwrap();

        let key_mismatch = KvCacheKey::new(chunk_id, fp_lookup, None);
        let retrieved = adapter
            .try_get_cached_segment_async(tenant, &key_mismatch)
            .await;
        assert!(
            retrieved.is_none(),
            "Fingerprint mismatch MUST return None without panic"
        );
    }

    #[tokio::test]
    async fn test_lsm_fallback_golden_zero_copy_async_bytes() {
        let store = Arc::new(InMemoryKvSegmentStore::with_capacity(1));
        let mock_store = Arc::new(MockKvStorage::default());

        let adapter = KvBridgeAdapter::with_lsm_fallback(store, mock_store);
        let tenant = TenantId::try_new(777).ok().unwrap();
        let fp = dummy_fp();

        let key1 = KvCacheKey::new(1, fp.clone(), Some(32));
        let payload1 = b"Golden zero-copy paged KV payload block".to_vec();
        adapter.store_segment(tenant, key1.clone(), payload1.clone());

        let key2 = KvCacheKey::new(2, fp.clone(), Some(64));
        let payload2 = b"Eviction trigger payload block".to_vec();
        adapter.store_segment(tenant, key2.clone(), payload2);

        let mut retrieved_bytes = None;
        for _ in 0..20 {
            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
            if let Some(b) = adapter
                .try_get_cached_segment_async_bytes(tenant, &key1)
                .await
            {
                retrieved_bytes = Some(b);
                break;
            }
        }
        assert!(retrieved_bytes.is_some());
        assert_eq!(retrieved_bytes.unwrap().as_ref(), payload1.as_slice());
    }

    #[tokio::test]
    async fn test_lsm_fallback_eviction_triggers_spill() {
        let store = Arc::new(InMemoryKvSegmentStore::with_capacity(1));
        let mock_store = Arc::new(MockKvStorage::default());

        let adapter = KvBridgeAdapter::with_lsm_fallback(store, mock_store);
        let tenant = TenantId::try_new(100).ok().unwrap();
        let fp = dummy_fp();

        let key1 = KvCacheKey::new(1, fp.clone(), None);
        let payload1 = b"Payload segment 1".to_vec();
        adapter.store_segment(tenant, key1.clone(), payload1.clone());

        let key2 = KvCacheKey::new(2, fp.clone(), None);
        let payload2 = b"Payload segment 2".to_vec();
        adapter.store_segment(tenant, key2.clone(), payload2.clone());

        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        assert!(adapter.try_get_cached_segment(tenant, &key1).is_none());

        let retrieved1 = adapter.try_get_cached_segment_async(tenant, &key1).await;
        assert_eq!(retrieved1, Some(payload1));
    }

    #[test]
    fn test_concurrency_parallel_requests_and_eviction() {
        let store = Arc::new(InMemoryKvSegmentStore::new());
        let adapter = KvBridgeAdapter::new(Arc::clone(&store));

        let adapter1 = adapter.clone();
        let adapter2 = adapter.clone();

        let tenant1 = TenantId::try_new(1).ok().unwrap();
        let tenant2 = TenantId::try_new(2).ok().unwrap();
        let fp = dummy_fp();

        let fp1 = fp.clone();
        let handle1 = thread::spawn(move || {
            for chunk_id in 0..50 {
                let data = vec![(chunk_id % 256) as u8; 128];
                let key = KvCacheKey::new(chunk_id, fp1.clone(), None);
                adapter1.store_segment(tenant1, key.clone(), data);
                let _ = adapter1.try_get_cached_segment(tenant1, &key);
            }
        });

        let fp2 = fp;
        let handle2 = thread::spawn(move || {
            for chunk_id in 100..150 {
                let data = vec![(chunk_id % 256) as u8; 128];
                let key = KvCacheKey::new(chunk_id, fp2.clone(), None);
                adapter2.store_segment(tenant2, key.clone(), data);
                let _ = adapter2.try_get_cached_segment(tenant2, &key);
            }
        });

        if let Ok(_) = handle1.join() {}
        if let Ok(_) = handle2.join() {}
    }
}
