#![cfg(feature = "kv-bridge")]
//! Verification test for KV-Bridge roundtrip: Export, serialization, and bit-identical restoration.

use contextra_infer_candle::kv_bridge::{create_test_kv_bridge, KvCacheKey};
use contextra_types::{ModelFingerprint, TenantId};

fn test_fingerprint() -> ModelFingerprint {
    ModelFingerprint::new([0xABu8; 32], "llama-3.2-1b.gguf", "Q4_K_M")
}

#[test]
fn test_kv_bridge_roundtrip_bit_identical() {
    let adapter = create_test_kv_bridge();

    let tenant = TenantId::try_new(42).ok().unwrap();
    let fp = test_fingerprint();
    let chunk_id = 1001;
    let rope_offset = Some(64);

    let synthetic_kv_bytes: Vec<u8> = (0..512).map(|i| (i % 256) as u8).collect();
    let key = KvCacheKey::new(chunk_id, fp, rope_offset);

    adapter.store_segment(tenant, key.clone(), synthetic_kv_bytes.clone());

    let retrieved = adapter.try_get_cached_segment(tenant, &key);
    assert!(
        retrieved.is_some(),
        "Stored segment must be successfully retrieved"
    );
    assert_eq!(
        retrieved,
        Some(synthetic_kv_bytes),
        "Retrieved payload must be bit-identical to stored synthetic KV bytes"
    );
}
