// FILE-CONTEXT
// ZWECK: Integration tests for KV cache scope count scan and VETO-F10 tenant isolation.
// STAND: TS:2026-10-12T00:00:00Z

#![forbid(unsafe_code)]

use bytes::Bytes;
use contextra_kvcache::{
    CchRamClaim, KvReusePolicy, PrefixStoreScratchpadInvalidator, ScratchpadCacheScope,
    ScratchpadInvalidator, TenantPrefixKvStore,
};
use contextra_ports::kv::{KvBlock, KvLayout, PrefixKey, RopeConfig};
use contextra_types::model_fingerprint::ModelFingerprint;
use contextra_types::TenantId;

fn create_prefix_key(model_name: &str) -> PrefixKey {
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
fn test_kvcache_scope_count_scan_and_veto_f10_isolation() {
    let store = TenantPrefixKvStore::new().with_reuse_policy(KvReusePolicy::Always);
    let tenant_a = TenantId::try_new(1000).unwrap();
    let tenant_b = TenantId::try_new(2000).unwrap();

    let key = create_prefix_key("gpt-model-v1");

    let base_tokens = vec![1, 2, 3, 4, 5];
    let scratchpad_tokens = vec![500, 501, 502];

    let block_a_base = KvBlock {
        block_id: 1,
        data: Bytes::from_static(b"tenant_a_base"),
    };
    let block_a_scratch = KvBlock {
        block_id: 2,
        data: Bytes::from_static(b"tenant_a_scratchpad"),
    };
    let block_b_scratch = KvBlock {
        block_id: 3,
        data: Bytes::from_static(b"tenant_b_scratchpad"),
    };

    store
        .insert(tenant_a, &key, &base_tokens, vec![block_a_base])
        .unwrap();
    store
        .insert(tenant_a, &key, &scratchpad_tokens, vec![block_a_scratch])
        .unwrap();
    store
        .insert(tenant_b, &key, &scratchpad_tokens, vec![block_b_scratch])
        .unwrap();

    let invalidator = PrefixStoreScratchpadInvalidator::new(&store).with_keys(vec![key.clone()]);

    let scope_a = ScratchpadCacheScope::new(tenant_a, scratchpad_tokens.clone());
    let scope_b = ScratchpadCacheScope::new(tenant_b, scratchpad_tokens.clone());

    // Negative control: scanner MUST detect scratchpad entry prior to invalidation
    let count_a_before = invalidator.count_entries_in_scope(&scope_a).unwrap();
    assert_eq!(
        count_a_before, 1,
        "Scanner MUST find Tenant A scratchpad entry before invalidation"
    );

    let report_a = invalidator.cache_report(&scope_a).unwrap();
    assert_eq!(report_a.caches_checked, 1);
    assert_eq!(report_a.entries_found, 1);
    assert_eq!(report_a.ram_claim, CchRamClaim::Unreferenced);

    // Action: Invalidate Tenant A's scratchpad scope ONLY
    let removed = invalidator.invalidate_scope(&scope_a).unwrap();
    assert_eq!(removed, 1, "Invalidator MUST remove exactly 1 entry for Tenant A");

    // Postcondition: Tenant A's scratchpad count MUST be 0
    let count_a_after = invalidator.count_entries_in_scope(&scope_a).unwrap();
    assert_eq!(
        count_a_after, 0,
        "Tenant A scratchpad count MUST be 0 after invalidation"
    );

    // VETO-F10 Tenant Isolation Assertion: Tenant B's scratchpad MUST remain untouched
    let count_b_after = invalidator.count_entries_in_scope(&scope_b).unwrap();
    assert_eq!(
        count_b_after, 1,
        "Tenant B scratchpad MUST NOT be affected (VETO-F10)"
    );

    let hit_b = store
        .lookup(tenant_b, &key, &scratchpad_tokens)
        .expect("Tenant B scratchpad must remain retrievable");
    assert_eq!(hit_b.matched_tokens, 3);

    // Tenant A's base context MUST remain retrievable
    let hit_a_base = store
        .lookup(tenant_a, &key, &base_tokens)
        .expect("Tenant A base context must remain retrievable");
    assert_eq!(hit_a_base.matched_tokens, 5);
}
