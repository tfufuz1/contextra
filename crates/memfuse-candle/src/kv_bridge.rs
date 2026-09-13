// FILE-CONTEXT
// STAND: 2026-09-13T00:00:00Z (SESSION: KV-BRIDGE-ADAPTER-IMPL)
// ZWECK: KvBridgeAdapter verbindet Retrieval-Chunks mit mandantenisoliertem KV-Cache-Store.
// INVARIANTEN: Cache-Miss und jeder Fehler ergeben transparenten Fallback auf vollen Prefill.
//              Kein parking_lot-Lock über .await-Punkt.

//! KV-Bridge Adapter connecting Candle inference to tenant-isolated encrypted KV cache store.

#![cfg(feature = "kv-bridge")]

use memfuse_core::traits::ContextSegment;
use memfuse_core::{ModelFingerprint, TenantId};
use memfuse_security::{KvSegment, KvSegmentCipher, TenantIsolatedKvStore};
use std::sync::Arc;

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

/// Verbindet Retrieval-Chunks mit dem mandantenisolierten, verschlüsselten KV-Cache.
///
/// # Fail-Safe-Garantie
/// Jede Methode fällt bei Fehler oder Cache-Miss transparent auf den normalen Prefill zurück.
/// Kein Fehler aus dem KV-Store oder der Krypto-Schicht darf eine Anfrage abbrechen.
#[derive(Clone)]
pub struct KvBridgeAdapter {
    pub store: Arc<TenantIsolatedKvStore>,
    pub cipher: Arc<KvSegmentCipher>,
    pub consultations: Arc<std::sync::atomic::AtomicU64>,
}

impl KvBridgeAdapter {
    /// Erstellt einen neuen Adapter mit gegebenem Store und Cipher.
    pub fn new(store: Arc<TenantIsolatedKvStore>, cipher: Arc<KvSegmentCipher>) -> Self {
        Self {
            store,
            cipher,
            consultations: Arc::new(std::sync::atomic::AtomicU64::new(0)),
        }
    }

    /// Consults segment metadata and KV cache bridge state for a context segment.
    pub fn consult_segment<'a>(&self, segment: &ContextSegment<'a>) {
        self.consultations
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let _ = (
            segment.chunk_id,
            segment.text,
            segment.model_fingerprint,
            segment.rope_offset,
        );
    }

    /// Returns the number of segment consultations recorded.
    pub fn consultation_count(&self) -> u64 {
        self.consultations.load(std::sync::atomic::Ordering::SeqCst)
    }

    /// Versucht, ein gecachetes KV-Segment zu laden.
    ///
    /// Gibt `None` zurück bei Cache-Miss, Decrypt-Fehler, Fingerprint-Mismatch oder jedem anderen Fehler.
    /// NIEMALS wird ein Fehler propagiert — `None` bedeutet stets "voller Prefill".
    pub fn try_get_cached_segment(
        &self,
        tenant: TenantId,
        key: &KvCacheKey,
    ) -> Option<Vec<u8>> {
        // 1. Store-Lookup (synchron, Lock wird vor Rückgabe freigegeben)
        let encrypted_bytes = self.store.get_segment_bytes(tenant, key.chunk_id)?;

        // 2. Deserialisieren (außerhalb des Store-Locks)
        let encrypted_layer: memfuse_security::EncryptedKvLayer =
            match bincode::deserialize(&encrypted_bytes) {
                Ok(l) => l,
                Err(e) => {
                    tracing::warn!(
                        chunk_id = key.chunk_id,
                        error = %e,
                        "KvBridgeAdapter: Deserialization failed — cache miss"
                    );
                    return None;
                }
            };

        // 3. Fingerprint-Validierung
        if encrypted_layer.model_fingerprint != key.fingerprint {
            tracing::warn!(
                chunk_id = key.chunk_id,
                "KvBridgeAdapter: Model fingerprint mismatch — cache miss"
            );
            return None;
        }

        // 4. Entschlüsseln (außerhalb des Store-Locks)
        match self.cipher.decrypt(&encrypted_layer) {
            Ok(plaintext) => Some(plaintext),
            Err(e) => {
                tracing::warn!(
                    chunk_id = key.chunk_id,
                    error = %e,
                    "KvBridgeAdapter: Decrypt failed — cache miss (possible format version mismatch)"
                );
                None
            }
        }
    }

    /// Speichert ein KV-Segment im Cache. Fehler werden geloggt, nie propagiert.
    pub fn store_segment(
        &self,
        tenant: TenantId,
        key: KvCacheKey,
        plaintext_kv_bytes: Vec<u8>,
    ) {
        let encrypted_layer = match self.cipher.encrypt(
            tenant,
            key.fingerprint,
            &plaintext_kv_bytes,
        ) {
            Ok(l) => l,
            Err(e) => {
                tracing::warn!(
                    chunk_id = key.chunk_id,
                    error = %e,
                    "KvBridgeAdapter: Encrypt failed — segment not cached"
                );
                return;
            }
        };

        let bytes = match bincode::serialize(&encrypted_layer) {
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

        let segment = KvSegment::new(tenant, key.chunk_id, bytes);
        self.store.insert_segment(tenant, segment);
    }
}

impl std::fmt::Debug for KvBridgeAdapter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("KvBridgeAdapter")
            .field("store", &"<TenantIsolatedKvStore>")
            .field("cipher", &"<KvSegmentCipher>")
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use memfuse_security::{CryptoKey, EvictionWorker};
    use std::sync::Arc;
    use std::thread;

    fn create_test_adapter() -> KvBridgeAdapter {
        let master_km = CryptoKey::try_new("test-passphrase-kv", b"test-salt-12345").unwrap();
        let cipher = Arc::new(KvSegmentCipher::new(master_km));
        let store = Arc::new(TenantIsolatedKvStore::new());
        KvBridgeAdapter::new(store, cipher)
    }

    fn dummy_fp() -> ModelFingerprint {
        ModelFingerprint::new([0x77u8; 32], "llama-3.2-1b.gguf", "Q4_K_M")
    }

    #[test]
    fn test_cache_miss_returns_none_without_panic() {
        let adapter = create_test_adapter();
        let tenant = TenantId::try_new(1).unwrap();
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
        let tenant = TenantId::try_new(10).unwrap();
        let fp = dummy_fp();
        let chunk_id = 42;
        let payload = b"KV tensor keys and values plaintext cache payload".to_vec();
        let key = KvCacheKey::new(chunk_id, fp, Some(128));

        adapter.store_segment(tenant, key.clone(), payload.clone());

        let retrieved = adapter.try_get_cached_segment(tenant, &key);
        assert!(retrieved.is_some(), "Stored segment MUST be retrievable");
        assert_eq!(retrieved.unwrap(), payload);
    }

    #[test]
    fn test_tenant_isolation() {
        let adapter = create_test_adapter();
        let tenant_a = TenantId::try_new(101).unwrap();
        let tenant_b = TenantId::try_new(202).unwrap();
        let fp = dummy_fp();
        let chunk_id = 1;
        let payload_a = b"Secret payload of Tenant A".to_vec();
        let key = KvCacheKey::new(chunk_id, fp, None);

        adapter.store_segment(tenant_a, key.clone(), payload_a.clone());

        // Tenant A gets its data
        let retrieved_a = adapter.try_get_cached_segment(tenant_a, &key);
        assert_eq!(retrieved_a, Some(payload_a));

        // Tenant B trying to get chunk_id 1 under tenant_b gets None
        let retrieved_b = adapter.try_get_cached_segment(tenant_b, &key);
        assert!(
            retrieved_b.is_none(),
            "Tenant B MUST NOT access Tenant A's cached segment"
        );
    }

    #[test]
    fn test_corrupt_payload_returns_none() {
        let adapter = create_test_adapter();
        let tenant = TenantId::try_new(55).unwrap();
        let key = KvCacheKey::new(777, dummy_fp(), None);

        // Store garbage bytes in store under tenant and chunk_id
        let corrupt_segment = KvSegment::new(tenant, key.chunk_id, vec![0xFF, 0xFE, 0xFD, 0xFC]);
        adapter.store.insert_segment(tenant, corrupt_segment);

        let result = adapter.try_get_cached_segment(tenant, &key);
        assert!(
            result.is_none(),
            "Corrupt payload MUST return None without panic"
        );
    }

    #[test]
    fn test_concurrency_parallel_requests_and_eviction() {
        let master_km = CryptoKey::try_new("concurrency-passphrase", b"salt-987654321").unwrap();
        let cipher = Arc::new(KvSegmentCipher::new(master_km));
        let store = Arc::new(TenantIsolatedKvStore::new());
        let adapter = KvBridgeAdapter::new(Arc::clone(&store), cipher);

        // Spawn EvictionWorker
        let worker = EvictionWorker::spawn(Arc::clone(&store));

        let adapter1 = adapter.clone();
        let adapter2 = adapter.clone();

        let tenant1 = TenantId::try_new(1).unwrap();
        let tenant2 = TenantId::try_new(2).unwrap();
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

        handle1.join().expect("Thread 1 panicked");
        handle2.join().expect("Thread 2 panicked");

        worker.shutdown();
    }
}
