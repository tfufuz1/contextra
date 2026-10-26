// FILE-CONTEXT
// ZWECK: Integrationstest für Mandantenisolation bei der Invalidation von Scratchpad-Cache-Scopes.
// INVARIANTEN: Mandantenisolation (VETO-F10): Invalidation von Mandant A lässt Mandant B bitgleich unberührt.

#![forbid(unsafe_code)]
#![allow(clippy::panic, clippy::unwrap_used, clippy::expect_used)]

use bytes::Bytes;
use contextra_ports::kv::{KvBlock, KvLayout, PrefixKey, RopeConfig};
use contextra_types::model_fingerprint::ModelFingerprint;
use contextra_types::TenantId;

use contextra_kvcache::prefix_store::TenantPrefixKvStore;
use contextra_kvcache::radix::KvReusePolicy;
use contextra_kvcache::scratchpad_invalidation::{
    PrefixStoreScratchpadInvalidator, ScratchpadCacheScope, ScratchpadInvalidator,
};

fn make_prefix_key(model_name: &str) -> PrefixKey {
    PrefixKey {
        model: ModelFingerprint::new([0xee; 32], model_name, "F16"),
        tokenizer_hash: [0x22; 32],
        layout: KvLayout {
            n_layer: 32,
            n_kv_head: 8,
            head_dim: 128,
            dtype: "f16".to_string(),
        },
        rope: RopeConfig {
            base: 10000.0,
            scaling: None,
        },
    }
}

#[test]
fn test_scratchpad_invalidation_multi_tenant_isolation() {
    let store = TenantPrefixKvStore::new().with_reuse_policy(KvReusePolicy::Always);

    let tenant_a = TenantId::try_new(1001).expect("Valid tenant_a");
    let tenant_b = TenantId::try_new(2002).expect("Valid tenant_b");

    let prefix_key = make_prefix_key("llama-3-8b");

    let scratchpad_tokens = vec![500, 501, 502, 503];

    // Populate Tenant A
    let block_a = KvBlock {
        block_id: 101,
        data: Bytes::from_static(b"tenant_a_scratchpad_data_payload_bytes"),
    };
    store
        .insert(tenant_a, &prefix_key, &scratchpad_tokens, vec![block_a])
        .expect("Insert Tenant A");

    // Populate Tenant B with identical tokens and payload
    let block_b = KvBlock {
        block_id: 202,
        data: Bytes::from_static(b"tenant_b_scratchpad_data_payload_bytes"),
    };
    store
        .insert(tenant_b, &prefix_key, &scratchpad_tokens, vec![block_b])
        .expect("Insert Tenant B");

    // Confirm initial state
    let hit_a_before = store
        .lookup(tenant_a, &prefix_key, &scratchpad_tokens)
        .expect("Tenant A initial hit");
    let hit_b_before = store
        .lookup(tenant_b, &prefix_key, &scratchpad_tokens)
        .expect("Tenant B initial hit");

    assert_eq!(hit_a_before.matched_tokens, 4);
    assert_eq!(hit_b_before.matched_tokens, 4);

    // Invalidate Scratchpad Scope for Tenant A ONLY
    let invalidator =
        PrefixStoreScratchpadInvalidator::new(&store).with_keys(vec![prefix_key.clone()]);
    let scope_a = ScratchpadCacheScope::new(tenant_a, scratchpad_tokens.clone());

    let count = invalidator
        .invalidate_scope(&scope_a)
        .expect("Invalidation execution");

    assert_eq!(count, 1, "Exactly one group for Tenant A should be removed");

    // Verify Tenant A is invalidated
    assert!(
        store
            .lookup(tenant_a, &prefix_key, &scratchpad_tokens)
            .is_none(),
        "Tenant A's scratchpad scope must be completely removed"
    );

    // Verify Tenant B remains completely untouched & bit-identical
    let hit_b_after = store
        .lookup(tenant_b, &prefix_key, &scratchpad_tokens)
        .expect("Tenant B hit must remain retrievable");

    assert_eq!(hit_b_after.matched_tokens, 4);
    assert_eq!(
        hit_b_after.blocks, hit_b_before.blocks,
        "Tenant B's blocks must remain bit-identical"
    );
}
