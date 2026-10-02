// FILE-CONTEXT
// ZWECK: Integrationstest für INV-CLM-SCRATCHPAD-1 (Gepinnter Basiskontext bleibt bei Scratchpad-Reset unberührt).
// INVARIANTEN: Ein Scratchpad-Reset invalidiert ausschließlich den zugehörigen KV-Cache-Branch,
//              niemals den gepinnten Basiskontext.
//              Wiederholte Invalidation ist idempotent (zweiter Aufruf liefert 0).

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
        model: ModelFingerprint::new([0x33; 32], model_name, "F16"),
        tokenizer_hash: [0x44; 32],
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
fn test_clm_pinned_base_preserved_and_invalidation_idempotency() {
    let store = TenantPrefixKvStore::new().with_reuse_policy(KvReusePolicy::Always);
    let tenant = TenantId::try_new(777).expect("Valid tenant");
    let key = make_prefix_key("qwen-2.5-72b");

    // 1. Base Pinned Context tokens: [1, 2, 3, 4, 5, 6, 7, 8]
    let base_tokens = vec![1, 2, 3, 4, 5, 6, 7, 8];
    let base_block = KvBlock {
        block_id: 10,
        data: Bytes::from_static(b"PINNED_BASE_CONTEXT_KV_DATA"),
    };
    store
        .insert(tenant, &key, &base_tokens, vec![base_block.clone()])
        .expect("Insert base context");

    // 2. Scratchpad Branch 1 tokens: [100, 101, 102, 103, 104]
    let scratchpad_branch_1 = vec![100, 101, 102, 103, 104];
    let scratchpad_block_1 = KvBlock {
        block_id: 20,
        data: Bytes::from_static(b"SCRATCHPAD_BRANCH_1_KV_DATA"),
    };
    store
        .insert(tenant, &key, &scratchpad_branch_1, vec![scratchpad_block_1])
        .expect("Insert scratchpad branch 1");

    // 3. Scratchpad Branch 2 tokens: [200, 201, 202]
    let scratchpad_branch_2 = vec![200, 201, 202];
    let scratchpad_block_2 = KvBlock {
        block_id: 30,
        data: Bytes::from_static(b"SCRATCHPAD_BRANCH_2_KV_DATA"),
    };
    store
        .insert(tenant, &key, &scratchpad_branch_2, vec![scratchpad_block_2])
        .expect("Insert scratchpad branch 2");

    let invalidator = PrefixStoreScratchpadInvalidator::new(&store).with_keys(vec![key.clone()]);
    let scope_branch_1 = ScratchpadCacheScope::new(tenant, scratchpad_branch_1.clone());

    // Execute first invalidation for Branch 1
    let count1 = invalidator
        .invalidate_scope(&scope_branch_1)
        .expect("Invalidation 1");
    assert_eq!(count1, 1, "First invalidation must remove Branch 1");

    // Verify Branch 1 is gone
    assert!(
        store
            .lookup(tenant, &key, &scratchpad_branch_1)
            .is_none(),
        "Branch 1 must be removed from cache"
    );

    // Verify Pinned Base Context remains intact & bit-identical
    let base_hit = store
        .lookup(tenant, &key, &base_tokens)
        .expect("Base context must remain retrievable");
    assert_eq!(base_hit.matched_tokens, 8);
    assert_eq!(
        base_hit.blocks[0].data, base_block.data,
        "Pinned base context data must remain bit-identical"
    );

    // Verify Branch 2 remains intact
    let branch2_hit = store
        .lookup(tenant, &key, &scratchpad_branch_2)
        .expect("Branch 2 must remain retrievable");
    assert_eq!(branch2_hit.matched_tokens, 3);

    // 4. Test Idempotency: Repeating the invalidation on Branch 1 returns 0
    let count2 = invalidator
        .invalidate_scope(&scope_branch_1)
        .expect("Invalidation 2 (repeat)");
    assert_eq!(
        count2, 0,
        "Repeated invalidation must be idempotent and return 0"
    );
}
