#![cfg(feature = "kv-bridge")]

//! Verification test for KV-Bridge tenant key separation: Data encrypted under Tenant A key cannot be read under Tenant B.

use contextra_crypto::{CryptoKey, KvSegmentCipher, TenantIsolatedKvStore};
use contextra_infer_candle::kv_bridge::{KvBridgeAdapter, KvCacheKey};
use contextra_types::{ModelFingerprint, TenantId};
use std::sync::Arc;

/// Helper function setting up cryptographic material per P28 rule exception.
/// Uses fixed salt and passphrase for deterministic test execution.
fn setup_test_cipher() -> Arc<KvSegmentCipher> {
    let master_km =
        CryptoKey::try_new("test-passphrase-tenant-sep", b"test-salt-tenantsep").unwrap();
    Arc::new(KvSegmentCipher::ephemeral(master_km))
}

fn test_fingerprint() -> ModelFingerprint {
    ModelFingerprint::new([0x33u8; 32], "llama-3.2-1b.gguf", "Q4_K_M")
}

#[test]
fn test_tenant_key_separation_isolation() {
    let cipher = setup_test_cipher();
    let store = Arc::new(TenantIsolatedKvStore::new());
    let adapter = KvBridgeAdapter::new(store, cipher);

    let tenant_a = TenantId::try_new(101).unwrap();
    let tenant_b = TenantId::try_new(202).unwrap();
    let fp = test_fingerprint();
    let chunk_id = 5005;

    let payload_a = b"Confidential KV Cache Payload for Tenant A".to_vec();
    let key = KvCacheKey::new(chunk_id, fp, None);

    // Store payload under Tenant A
    adapter.store_segment(tenant_a, key.clone(), payload_a.clone());

    // Tenant A can successfully retrieve its own payload
    let retrieved_a = adapter.try_get_cached_segment(tenant_a, &key);
    assert_eq!(
        retrieved_a,
        Some(payload_a),
        "Tenant A must be able to read its own stored segment"
    );

    // Tenant B attempting to access the same chunk_id under tenant_b gets None (isolated)
    let retrieved_b = adapter.try_get_cached_segment(tenant_b, &key);
    assert!(
        retrieved_b.is_none(),
        "Tenant B MUST NOT be able to read Tenant A's encrypted segment"
    );
}
