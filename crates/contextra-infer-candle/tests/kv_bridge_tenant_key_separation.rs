#![cfg(feature = "kv-bridge")]
//! Verification test for KV-Bridge tenant separation: Data stored under Tenant A cannot be read under Tenant B.

use contextra_infer_candle::kv_bridge::{create_test_kv_bridge, KvCacheKey};
use contextra_types::{ModelFingerprint, TenantId};

fn test_fingerprint() -> ModelFingerprint {
    ModelFingerprint::new([0x33u8; 32], "llama-3.2-1b.gguf", "Q4_K_M")
}

#[test]
fn test_tenant_key_separation_isolation() {
    let adapter = create_test_kv_bridge();

    let tenant_a = TenantId::try_new(101).ok().unwrap();
    let tenant_b = TenantId::try_new(202).ok().unwrap();
    let fp = test_fingerprint();
    let chunk_id = 5005;

    let payload_a = b"Confidential KV Cache Payload for Tenant A".to_vec();
    let key = KvCacheKey::new(chunk_id, fp, None);

    adapter.store_segment(tenant_a, key.clone(), payload_a.clone());

    let retrieved_a = adapter.try_get_cached_segment(tenant_a, &key);
    assert_eq!(
        retrieved_a,
        Some(payload_a),
        "Tenant A must be able to read its own stored segment"
    );

    let retrieved_b = adapter.try_get_cached_segment(tenant_b, &key);
    assert!(
        retrieved_b.is_none(),
        "Tenant B MUST NOT be able to read Tenant A's cached segment"
    );
}
