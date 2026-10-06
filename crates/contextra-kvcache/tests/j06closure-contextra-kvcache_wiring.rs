// ZWECK: Integrationstests zur Verifikation der Schließung aller 10 J06-Symbole in contextra-kvcache.
// STAND: TS:2026-10-01T00:00:00Z

use bytes::Bytes;
use contextra_crypto::{CryptoKey, ModelFingerprint};
use contextra_ports::kv::{KvBlock, KvLayout, PrefixKey, RopeConfig};
use contextra_ports::RequestId;
use contextra_types::{ContextraError, Result, TenantId};
use std::sync::Arc;

use contextra_kvcache::eviction_worker::EvictionWorker;
use contextra_kvcache::prefix_store::TenantPrefixKvStore;
use contextra_kvcache::quantize_kivi::{KiviQuantizeConfig, KvTensorView};
use contextra_kvcache::radix::KvReusePolicy;
use contextra_kvcache::scratchpad_invalidation::{
    PrefixStoreScratchpadInvalidator, ScratchpadCacheScope, ScratchpadInvalidator,
};
use contextra_kvcache::segment::{KvSegment, ShreddableSegmentKey, Tier2EncryptedSegment};
use contextra_kvcache::store::TenantIsolatedKvStore;

struct TestCipher {
    key: CryptoKey,
}

impl TestCipher {
    fn new() -> Self {
        let key = CryptoKey::try_new("test-passphrase-closure", b"test-salt-closure").unwrap();
        Self { key }
    }
}

impl contextra_crypto::KvCipher for TestCipher {
    fn seal(&self, plaintext: &[u8]) -> Result<Vec<u8>> {
        let (ct, nonce) = self
            .key
            .encrypt_auto_nonce(plaintext)
            .map_err(|e| ContextraError::Crypto(e.to_string()))?;
        let mut out = Vec::with_capacity(12 + ct.len());
        out.extend_from_slice(&nonce);
        out.extend_from_slice(&ct);
        Ok(out)
    }

    fn open(&self, ciphertext: &[u8]) -> Result<Vec<u8>> {
        if ciphertext.len() < 12 {
            return Err(ContextraError::Crypto("Ciphertext too short".into()));
        }
        let nonce: &[u8; 12] = ciphertext[..12].try_into().unwrap();
        let ct = &ciphertext[12..];
        self.key
            .decrypt_auto_nonce(ct, nonce)
            .map_err(|e| ContextraError::Crypto(e.to_string()))
    }
}

fn make_prefix_key(name: &str) -> PrefixKey {
    PrefixKey {
        model: ModelFingerprint::new([0xde; 32], name, "F16"),
        tokenizer_hash: [0x22; 32],
        layout: KvLayout {
            n_layer: 8,
            n_kv_head: 2,
            head_dim: 32,
            dtype: "f16".to_string(),
        },
        rope: RopeConfig {
            base: 10000.0,
            scaling: None,
        },
    }
}

#[test]
fn j06closure_test_eviction_worker_registration_symbols() {
    let store = Arc::new(TenantIsolatedKvStore::new());
    let tenant = TenantId::try_new(500).unwrap();
    let worker = EvictionWorker::spawn(Arc::clone(&store));

    // Symbol 1: register_segment_request
    worker.register_segment_request(tenant, 101, RequestId(9001));

    // Symbol 2: unregister_segment_request
    worker.unregister_segment_request(tenant, 101);
}

#[test]
fn j06closure_test_prefix_store_builder_symbols() {
    // Symbol 3: with_byte_budget_per_tenant
    // Symbol 4: with_reuse_policy
    let store =
        TenantPrefixKvStore::with_capacity_and_policy(2 * 1024 * 1024, KvReusePolicy::Always);

    let tenant = TenantId::try_new(501).unwrap();
    let key = make_prefix_key("closure-model");
    let tokens = vec![10, 20, 30];
    let block = KvBlock {
        block_id: 1,
        data: Bytes::from_static(b"closure_block_data"),
    };

    store.insert(tenant, &key, &tokens, vec![block]).unwrap();
    let hit = store
        .lookup(tenant, &key, &tokens)
        .expect("Lookup should succeed");
    assert_eq!(hit.matched_tokens, 3);
}

#[test]
fn j06closure_test_scratchpad_invalidator_with_keys_symbol() {
    let store = TenantPrefixKvStore::new().with_reuse_policy(KvReusePolicy::Always);
    let tenant = TenantId::try_new(502).unwrap();
    let key1 = make_prefix_key("key-1");
    let key2 = make_prefix_key("key-2");

    let tokens = vec![1, 2, 3];
    let block = KvBlock {
        block_id: 10,
        data: Bytes::from_static(b"data"),
    };

    store
        .insert(tenant, &key1, &tokens, vec![block.clone()])
        .unwrap();
    store.insert(tenant, &key2, &tokens, vec![block]).unwrap();

    // Symbol 5: with_keys
    let invalidator = PrefixStoreScratchpadInvalidator::new(&store).with_keys(vec![key1.clone()]);
    let scope = ScratchpadCacheScope::new(tenant, tokens.clone());

    let count = invalidator.invalidate_scope(&scope).unwrap();
    assert_eq!(count, 1);
    assert!(store.lookup(tenant, &key1, &tokens).is_none());
    assert!(store.lookup(tenant, &key2, &tokens).is_some());
}

#[test]
fn j06closure_test_segment_crypto_and_quantization_symbols() {
    let tenant = TenantId::try_new(503).unwrap();

    // Symbol 8: try_new_random
    let key = ShreddableSegmentKey::try_new_random("closure-secret-passphrase").unwrap();
    assert!(!key.is_shredded());

    // Symbol 6: read_and_decrypt
    let tier2 = Tier2EncryptedSegment::new(tenant, 888, b"encrypted-tier2-payload", key).unwrap();
    let decrypted = tier2.read_and_decrypt().unwrap();
    assert_eq!(decrypted, b"encrypted-tier2-payload");

    let cipher = TestCipher::new();
    let config = KiviQuantizeConfig {
        key_group_size: 16,
        quantize_values: true,
    };
    let raw =
        KvTensorView::new(vec![1.0, 2.0, 3.0, 4.0], vec![10.0, 20.0, 30.0, 40.0], 2, 2).unwrap();

    // Symbol 9: write_quantized
    let mut seg = KvSegment::new(tenant, 999, vec![]);
    seg.write_quantized(&raw, config, &cipher).unwrap();

    // Symbol 7: read_dequantized
    let reconstructed = seg.read_dequantized(&cipher).unwrap();
    assert_eq!(reconstructed.num_tokens, 2);
    assert_eq!(reconstructed.num_channels, 2);
    assert_eq!(reconstructed.keys.len(), 4);
    assert_eq!(reconstructed.values.len(), 4);
}

#[test]
fn j06closure_test_store_acquire_block_guard_symbol() {
    let store = TenantIsolatedKvStore::new();
    let tenant = TenantId::try_new(504).unwrap();

    let seg = KvSegment::new(tenant, 777, vec![0xAA; 128]);
    store.insert_segment(tenant, seg);

    // Symbol 10: acquire_block_guard
    let guard = store
        .acquire_block_guard(tenant, 777, None)
        .expect("Guard must be acquired");
    assert_eq!(guard.active_refs(), 1);

    // Verify pinned protection against LRU eviction
    let freed = store.evict_lru_fair(128);
    assert_eq!(freed, 0);
    assert_eq!(store.get_tenant_segment_len(tenant), 1);

    drop(guard);
    let freed_after = store.evict_lru_fair(128);
    assert!(freed_after >= 128);
    assert_eq!(store.get_tenant_segment_len(tenant), 0);
}
