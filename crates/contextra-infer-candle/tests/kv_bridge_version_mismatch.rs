#![cfg(feature = "kv-bridge")]
//! Verification test for KV-Bridge mismatch and corruption scenarios: Model fingerprint mismatch, RoPE mismatch, truncated or corrupted bytes.

use contextra_infer_candle::kv_bridge::{create_test_kv_bridge, KvCacheKey};
use contextra_types::{ModelFingerprint, TenantId};

fn test_fingerprint_a() -> ModelFingerprint {
    ModelFingerprint::new([0x11u8; 32], "llama-3.2-1b.gguf", "Q4_K_M")
}

fn test_fingerprint_b() -> ModelFingerprint {
    ModelFingerprint::new([0x22u8; 32], "llama-3.2-3b.gguf", "Q8_0")
}

#[test]
fn test_version_and_model_mismatch_returns_none_without_panic() {
    let adapter = create_test_kv_bridge();

    let tenant = TenantId::try_new(10).ok().unwrap();
    let chunk_id = 2002;
    let payload = b"Synthetic KV Tensor Data Payload".to_vec();

    let key_a = KvCacheKey::new(chunk_id, test_fingerprint_a(), Some(128));
    adapter.store_segment(tenant, key_a.clone(), payload.clone());

    assert_eq!(
        adapter.try_get_cached_segment(tenant, &key_a),
        Some(payload)
    );

    let key_b = KvCacheKey::new(chunk_id, test_fingerprint_b(), Some(128));
    let res_mismatch = adapter.try_get_cached_segment(tenant, &key_b);
    assert!(
        res_mismatch.is_none(),
        "Lookup with different model fingerprint MUST return None"
    );

    let key_diff_rope = KvCacheKey::new(chunk_id, test_fingerprint_a(), Some(256));
    let res_rope_mismatch = adapter.try_get_cached_segment(tenant, &key_diff_rope);
    assert!(
        res_rope_mismatch.is_none(),
        "Lookup with different RoPE offset MUST return None"
    );
}

#[test]
fn test_corrupted_or_truncated_bytes_returns_none_without_panic() {
    let adapter = create_test_kv_bridge();

    let tenant = TenantId::try_new(15).ok().unwrap();
    let chunk_id = 3003;
    let key = KvCacheKey::new(chunk_id, test_fingerprint_a(), None);

    adapter.store.insert_segment(tenant, chunk_id, vec![0xCA, 0xFE, 0xBA, 0xBE, 0x00]);

    let res_corrupt = adapter.try_get_cached_segment(tenant, &key);
    assert!(
        res_corrupt.is_none(),
        "Corrupted payload MUST return None without panic"
    );

    adapter.store.insert_segment(tenant, chunk_id + 1, vec![0x01, 0x02]);

    let key_truncated = KvCacheKey::new(chunk_id + 1, test_fingerprint_a(), None);
    let res_truncated = adapter.try_get_cached_segment(tenant, &key_truncated);
    assert!(
        res_truncated.is_none(),
        "Truncated payload MUST return None without panic"
    );
}
