#![cfg(feature = "kv-bridge")]

//! Verification test for KV-Bridge roundtrip: Export, serialization, encryption, decryption, and bit-identical restoration.

use contextra_crypto::{CryptoKey, KvSegmentCipher, TenantIsolatedKvStore};
use contextra_infer_candle::kv_bridge::{KvBridgeAdapter, KvCacheKey};
use contextra_types::{ModelFingerprint, TenantId};
use std::sync::Arc;

/// Helper function setting up cryptographic material per P28 rule exception.
/// Uses fixed salt and passphrase for deterministic test execution.
fn setup_test_cipher() -> Arc<KvSegmentCipher> {
    let master_km =
        CryptoKey::try_new("test-passphrase-roundtrip", b"test-salt-1234567890").unwrap();
    Arc::new(KvSegmentCipher::ephemeral(master_km))
}

fn test_fingerprint() -> ModelFingerprint {
    ModelFingerprint::new([0xABu8; 32], "llama-3.2-1b.gguf", "Q4_K_M")
}

#[test]
fn test_kv_bridge_roundtrip_bit_identical() {
    let cipher = setup_test_cipher();
    let store = Arc::new(TenantIsolatedKvStore::new());
    let adapter = KvBridgeAdapter::new(store, cipher);

    let tenant = TenantId::try_new(42).unwrap();
    let fp = test_fingerprint();
    let chunk_id = 1001;
    let rope_offset = Some(64);

    // Synthetic fixed tensor bytes representing layer-wise KV cache payload
    let synthetic_kv_bytes: Vec<u8> = (0..512).map(|i| (i % 256) as u8).collect();

    let key = KvCacheKey::new(chunk_id, fp, rope_offset);

    // Store segment
    adapter.store_segment(tenant, key.clone(), synthetic_kv_bytes.clone());

    // Retrieve segment
    let retrieved = adapter.try_get_cached_segment(tenant, &key);

    assert!(
        retrieved.is_some(),
        "Stored segment must be successfully retrieved"
    );
    assert_eq!(
        retrieved.unwrap(),
        synthetic_kv_bytes,
        "Retrieved payload must be bit-identical to stored synthetic KV bytes"
    );
}
