// FILE-CONTEXT
// ZWECK: Integrationstests zur Verifikation der Verdrahtung aller J06-Symbole in contextra-kvcache.
// STAND: TS:2026-10-01T00:00:00Z

#![forbid(unsafe_code)]

use bytes::Bytes;
use contextra_kvcache::eviction_worker::EvictionWorker;
use contextra_kvcache::prefix_store::TenantPrefixKvStore;
use contextra_kvcache::quantize_kivi::{KiviQuantizeConfig, KvTensorView};
use contextra_kvcache::radix::KvReusePolicy;
use contextra_kvcache::scratchpad_invalidation::{
    PrefixStoreScratchpadInvalidator, ScratchpadCacheScope, ScratchpadInvalidator,
};
use contextra_kvcache::segment::{KvSegment, Tier2EncryptedSegment};
use contextra_kvcache::store::TenantIsolatedKvStore;
use contextra_ports::kv::{KvBlock, KvLayout, PrefixKey, RopeConfig};
use contextra_ports::RequestId;
use contextra_types::model_fingerprint::ModelFingerprint;
use contextra_types::TenantId;
use std::sync::Arc;

fn create_test_prefix_key(name: &str) -> PrefixKey {
    PrefixKey {
        model: ModelFingerprint::new([0xcd; 32], name, "F16"),
        tokenizer_hash: [0x11; 32],
        layout: KvLayout {
            n_layer: 16,
            n_kv_head: 4,
            head_dim: 64,
            dtype: "f16".to_string(),
        },
        rope: RopeConfig {
            base: 10000.0,
            scaling: None,
        },
    }
}

#[test]
fn j06_test_segment_request_registration_wiring() {
    let store = Arc::new(TenantIsolatedKvStore::new());
    let worker = EvictionWorker::spawn(Arc::clone(&store));

    let tenant = TenantId::try_new(100).unwrap();
    let request_id = RequestId(999);
    let segment = KvSegment::new(tenant, 101, vec![0xAB; 256]);

    // Insert segment via request registration path
    store.insert_segment_for_request(tenant, segment, request_id);

    assert_eq!(store.get_tenant_segment_len(tenant), 1);
    assert_eq!(store.get_segments(tenant), vec![101]);

    // Unregister segment request
    store.unregister_segment_request(tenant, 101);

    worker.shutdown();
}

#[test]
fn j06_test_prefix_store_builder_lever_wiring() {
    let tenant = TenantId::try_new(200).unwrap();
    let key = create_test_prefix_key("j06-model");

    // Configure TenantPrefixKvStore using builder setters
    let store = TenantPrefixKvStore::new()
        .with_byte_budget_per_tenant(1024)
        .with_reuse_policy(KvReusePolicy::Always);

    let tokens = vec![1, 2, 3, 4, 5];
    let block = KvBlock {
        block_id: 1,
        data: Bytes::from_static(b"prefix_block_payload"),
    };

    store.insert(tenant, &key, &tokens, vec![block]).unwrap();

    let hit = store
        .lookup(tenant, &key, &tokens)
        .expect("Prefix lookup must succeed");
    assert_eq!(hit.matched_tokens, 5);
}

#[test]
fn j06_test_scratchpad_invalidator_with_keys_wiring() {
    let store = TenantPrefixKvStore::new().with_reuse_policy(KvReusePolicy::Always);
    let tenant = TenantId::try_new(300).unwrap();

    let key_target = create_test_prefix_key("target-model");
    let key_other = create_test_prefix_key("other-model");

    let scratchpad_tokens = vec![10, 20, 30];
    let block1 = KvBlock {
        block_id: 10,
        data: Bytes::from_static(b"scratchpad_10"),
    };
    let block2 = KvBlock {
        block_id: 20,
        data: Bytes::from_static(b"scratchpad_20"),
    };

    store
        .insert(tenant, &key_target, &scratchpad_tokens, vec![block1])
        .unwrap();
    store
        .insert(tenant, &key_other, &scratchpad_tokens, vec![block2])
        .unwrap();

    // Use with_keys to target only key_target
    let invalidator =
        PrefixStoreScratchpadInvalidator::new(&store).with_keys(vec![key_target.clone()]);

    let scope = ScratchpadCacheScope::new(tenant, scratchpad_tokens.clone());
    let count = invalidator.invalidate_scope(&scope).unwrap();
    assert_eq!(count, 1);

    // key_target is invalidated, key_other remains retrievable
    assert!(store.lookup(tenant, &key_target, &scratchpad_tokens).is_none());
    assert!(store.lookup(tenant, &key_other, &scratchpad_tokens).is_some());
}

#[test]
fn j06_test_tier2_encrypted_segment_wiring() {
    let tenant = TenantId::try_new(400).unwrap();
    let segment_id = 5001;
    let plaintext = b"Tier-2 persistent disk segment payload";
    let passphrase = "j06-passphrase-secret";

    // Create Tier-2 segment with random key
    let tier2 = Tier2EncryptedSegment::new_with_random_key(
        tenant,
        segment_id,
        plaintext,
        passphrase,
    )
    .unwrap();

    assert_eq!(tier2.tenant_id, tenant);
    assert_eq!(tier2.segment_id, segment_id);

    // Unspill tier2 segment back to KvSegment using read_and_decrypt
    let unspilled_segment = tier2.to_kv_segment().unwrap();
    assert_eq!(unspilled_segment.tenant_id, tenant);
    assert_eq!(unspilled_segment.segment_id, segment_id);
    assert_eq!(unspilled_segment.as_bytes(), plaintext);
}

#[test]
fn j06_test_quantized_segment_wiring() {
    let store = TenantIsolatedKvStore::new();
    let tenant = TenantId::try_new(500).unwrap();
    let segment_id = 6001;

    let km = contextra_crypto::CryptoKey::try_new("passphrase-q1", b"salt-q1").unwrap();

    let num_tokens = 2;
    let num_channels = 32;
    let num_elements = num_tokens * num_channels;
    let key_data: Vec<f32> = (0..num_elements).map(|i| i as f32 * 0.1).collect();
    let val_data: Vec<f32> = (0..num_elements).map(|i| i as f32 * 0.2).collect();
    let raw_kv = KvTensorView::new(key_data, val_data, num_tokens, num_channels).unwrap();
    let config = KiviQuantizeConfig::default();

    // Insert quantized segment (uses write_quantized via KvSegment::new_quantized)
    store
        .insert_quantized_segment(tenant, segment_id, &raw_kv, config, &km)
        .unwrap();

    assert_eq!(store.get_tenant_segment_len(tenant), 1);

    // Retrieve dequantized segment (uses read_dequantized)
    let decompressed_view = store
        .get_dequantized_segment(tenant, segment_id, &km)
        .unwrap()
        .expect("Dequantized segment must be present");

    assert_eq!(decompressed_view.num_tokens, num_tokens);
    assert_eq!(decompressed_view.num_channels, num_channels);
}

#[test]
fn j06_test_pin_block_guard_wiring() {
    let store = TenantIsolatedKvStore::with_capacity(1);
    let tenant = TenantId::try_new(600).unwrap();

    let seg = KvSegment::new(tenant, 7001, vec![0x11; 512]);
    store.insert_segment(tenant, seg);

    // Pin block using pin_block (invokes acquire_block_guard)
    let guard = store
        .pin_block(tenant, 7001)
        .expect("Block guard must be acquired");
    assert_eq!(guard.active_refs(), 1);

    // Insert another segment into capacity-1 store; pinned segment must be protected
    let seg2 = KvSegment::new(tenant, 7002, vec![0x22; 512]);
    store.insert_segment(tenant, seg2);

    let segments = store.get_segments(tenant);
    assert!(segments.contains(&7001));
    assert!(segments.contains(&7002));

    drop(guard);
}
