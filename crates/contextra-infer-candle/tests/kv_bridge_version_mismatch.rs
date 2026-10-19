#![cfg(feature = "kv-bridge")]

//! Verification test for KV-Bridge mismatch and corruption scenarios: Model fingerprint mismatch, RoPE mismatch, truncated or corrupted bytes.

use contextra_crypto::{CryptoKey, KvSegment, KvSegmentCipher, TenantIsolatedKvStore};
use contextra_infer_candle::kv_bridge::{KvBridgeAdapter, KvCacheKey};
use contextra_types::{ModelFingerprint, TenantId};
use std::sync::Arc;

/// Helper function setting up cryptographic material per P28 rule exception.
/// Uses fixed salt and passphrase for deterministic test execution.
fn setup_test_cipher() -> Arc<KvSegmentCipher> {
    let master_km =
        CryptoKey::try_new("test-passphrase-mismatch", b"test-salt-mismatch123").unwrap();
    Arc::new(KvSegmentCipher::ephemeral(master_km))
}

fn test_fingerprint_a() -> ModelFingerprint {
    ModelFingerprint::new([0x11u8; 32], "llama-3.2-1b.gguf", "Q4_K_M")
}

fn test_fingerprint_b() -> ModelFingerprint {
    ModelFingerprint::new([0x22u8; 32], "llama-3.2-3b.gguf", "Q8_0")
}

#[test]
fn test_version_and_model_mismatch_returns_none_without_panic() {
    let cipher = setup_test_cipher();
    let store = Arc::new(TenantIsolatedKvStore::new());
    let adapter = KvBridgeAdapter::new(store, cipher);

    let tenant = TenantId::try_new(10).unwrap();
    let chunk_id = 2002;
    let payload = b"Synthetic KV Tensor Data Payload".to_vec();

    // 1. Store with Model Fingerprint A
    let key_a = KvCacheKey::new(chunk_id, test_fingerprint_a(), Some(128));
    adapter.store_segment(tenant, key_a.clone(), payload.clone());

    // Exact match succeeds
    assert_eq!(
        adapter.try_get_cached_segment(tenant, &key_a),
        Some(payload.clone())
    );

    // 2. Lookup with Model Fingerprint B (Model Mismatch) -> must yield None without panic
    let key_b = KvCacheKey::new(chunk_id, test_fingerprint_b(), Some(128));
    let res_mismatch = adapter.try_get_cached_segment(tenant, &key_b);
    assert!(
        res_mismatch.is_none(),
        "Lookup with different model fingerprint MUST return None"
    );

    // 3. Lookup with RoPE Offset Mismatch -> must yield None without panic
    let key_diff_rope = KvCacheKey::new(chunk_id, test_fingerprint_a(), Some(256));
    let res_rope_mismatch = adapter.try_get_cached_segment(tenant, &key_diff_rope);
    assert!(
        res_rope_mismatch.is_none(),
        "Lookup with different RoPE offset MUST return None"
    );
}

#[test]
fn test_corrupted_or_truncated_bytes_returns_none_without_panic() {
    let cipher = setup_test_cipher();
    let store = Arc::new(TenantIsolatedKvStore::new());
    let adapter = KvBridgeAdapter::new(store, Arc::clone(&cipher));

    let tenant = TenantId::try_new(15).unwrap();
    let chunk_id = 3003;
    let key = KvCacheKey::new(chunk_id, test_fingerprint_a(), None);

    // Insert raw garbage bytes into store directly
    let corrupt_segment = KvSegment::new(tenant, chunk_id, vec![0xCA, 0xFE, 0xBA, 0xBE, 0x00]);
    adapter.store.insert_segment(tenant, corrupt_segment);

    // Lookup on corrupt raw bytes -> decrypt or deserialization fail -> returns None without panic
    let res_corrupt = adapter.try_get_cached_segment(tenant, &key);
    assert!(
        res_corrupt.is_none(),
        "Corrupted payload MUST return None without panic"
    );

    // Insert truncated bincode payload
    let truncated_bytes = vec![0x01, 0x02];
    let truncated_segment = KvSegment::new(tenant, chunk_id + 1, truncated_bytes);
    adapter.store.insert_segment(tenant, truncated_segment);

    let key_truncated = KvCacheKey::new(chunk_id + 1, test_fingerprint_a(), None);
    let res_truncated = adapter.try_get_cached_segment(tenant, &key_truncated);
    assert!(
        res_truncated.is_none(),
        "Truncated payload MUST return None without panic"
    );
}
