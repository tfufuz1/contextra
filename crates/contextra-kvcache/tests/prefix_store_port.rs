// FILE-CONTEXT
// ZWECK: Port- und Isolationstests für TenantPrefixKvStore (§9.2).
// STAND: TS:2026-09-15T00:00:00Z

#![allow(clippy::unwrap_used, clippy::expect_used)]

use contextra_core::kv::{KvBlock, KvLayout, KvPrefixStore, PrefixKey, RopeConfig};
use contextra_core::model_fingerprint::ModelFingerprint;
use contextra_core::{ContextraError, TenantId};
use contextra_kvcache::{KvReusePolicy, TenantPrefixKvStore};
use std::sync::Arc;

fn make_key(model_id: &str) -> PrefixKey {
    PrefixKey {
        model: ModelFingerprint::new([0xAA; 32], model_id, "f16"),
        tokenizer_hash: [0x55; 32],
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

fn make_block(id: u64, size: usize) -> KvBlock {
    KvBlock {
        block_id: id,
        data: vec![0x42; size].into(),
    }
}

#[test]
fn test_round_trip() -> Result<(), ContextraError> {
    let store = TenantPrefixKvStore::new();
    let tenant = TenantId::try_new(1)?;
    let key = make_key("llama-3-8b");
    let tokens = vec![101, 102, 103, 104, 105];
    let blocks = vec![make_block(1, 100), make_block(2, 100)];

    store.insert(tenant, &key, &tokens, blocks.clone())?;

    let hit = store.lookup(tenant, &key, &tokens).expect("exact match hit");
    assert_eq!(hit.matched_tokens, 5);
    assert_eq!(hit.blocks, blocks);

    Ok(())
}

#[test]
fn test_longest_prefix_wins() -> Result<(), ContextraError> {
    let store = TenantPrefixKvStore::new().with_reuse_policy(KvReusePolicy::Always);
    let tenant = TenantId::try_new(1)?;
    let key = make_key("llama-3-8b");

    let short_seq = vec![1, 2, 3, 4];
    let long_seq = vec![1, 2, 3, 4, 5, 6, 7];

    let blocks_short = vec![make_block(10, 50)];
    let blocks_long = vec![make_block(20, 50)];

    store.insert(tenant, &key, &short_seq, blocks_short.clone())?;
    store.insert(tenant, &key, &long_seq, blocks_long.clone())?;

    let query = vec![1, 2, 3, 4, 5, 6, 7, 8, 9];
    let hit = store.lookup(tenant, &key, &query).expect("hit");
    assert_eq!(hit.matched_tokens, 7);
    assert_eq!(hit.blocks, blocks_long);

    Ok(())
}

#[test]
fn test_cost_based_policy_suppresses_short_matches() -> Result<(), ContextraError> {
    let store = TenantPrefixKvStore::new().with_reuse_policy(KvReusePolicy::CostBased {
        min_prefix_len: 4,
    });
    let tenant = TenantId::try_new(1)?;
    let key = make_key("llama-3-8b");

    let short_seq = vec![10, 20, 30];
    store.insert(tenant, &key, &short_seq, vec![make_block(1, 100)])?;

    let query = vec![10, 20, 30, 40, 50];
    assert!(store.lookup(tenant, &key, &query).is_none());

    let long_seq = vec![10, 20, 30, 40];
    store.insert(tenant, &key, &long_seq, vec![make_block(2, 100)])?;
    let hit = store.lookup(tenant, &key, &query).expect("hit");
    assert_eq!(hit.matched_tokens, 4);

    Ok(())
}

#[test]
fn test_tenant_isolation() -> Result<(), ContextraError> {
    let store = TenantPrefixKvStore::new();
    let tenant_a = TenantId::try_new(100)?;
    let tenant_b = TenantId::try_new(200)?;
    let key = make_key("shared-model");
    let tokens = vec![1, 2, 3, 4, 5];
    let blocks_a = vec![make_block(1, 128)];

    store.insert(tenant_a, &key, &tokens, blocks_a.clone())?;

    assert!(store.lookup(tenant_b, &key, &tokens).is_none());

    let evicted_b = store.evict(tenant_b, &key)?;
    assert_eq!(evicted_b, 0);

    let hit_a = store.lookup(tenant_a, &key, &tokens).expect("tenant A hit");
    assert_eq!(hit_a.blocks, blocks_a);

    Ok(())
}

#[test]
fn test_different_prefix_key_miss() -> Result<(), ContextraError> {
    let store = TenantPrefixKvStore::new();
    let tenant = TenantId::try_new(1)?;
    let key1 = make_key("model-v1");
    let key2 = make_key("model-v2");
    let tokens = vec![1, 2, 3, 4, 5];

    store.insert(tenant, &key1, &tokens, vec![make_block(1, 100)])?;

    assert!(store.lookup(tenant, &key2, &tokens).is_none());

    Ok(())
}

#[test]
fn test_budget_eviction_and_oversized_rejection() -> Result<(), ContextraError> {
    let store = TenantPrefixKvStore::new().with_byte_budget_per_tenant(300);
    let tenant = TenantId::try_new(5)?;
    let key1 = make_key("model-a");
    let key2 = make_key("model-b");

    let res_oversized = store.insert(tenant, &key1, &[1, 2, 3, 4], vec![make_block(1, 350)]);
    assert!(
        matches!(res_oversized, Err(ContextraError::PolicyViolation(_))),
        "Oversized group must trigger PolicyViolation"
    );

    store.insert(tenant, &key1, &[10, 20, 30, 40], vec![make_block(10, 100)])?;
    store.insert(tenant, &key2, &[10, 20, 30, 40], vec![make_block(20, 100)])?;
    store.insert(tenant, &key1, &[50, 60, 70, 80], vec![make_block(30, 100)])?;

    assert!(store.lookup(tenant, &key1, &[10, 20, 30, 40]).is_some());
    assert!(store.lookup(tenant, &key2, &[10, 20, 30, 40]).is_some());
    assert!(store.lookup(tenant, &key1, &[50, 60, 70, 80]).is_some());

    let _ = store.lookup(tenant, &key2, &[10, 20, 30, 40]);

    store.insert(tenant, &key1, &[90, 91, 92, 93], vec![make_block(40, 100)])?;

    assert!(
        store.lookup(tenant, &key1, &[10, 20, 30, 40]).is_none(),
        "Oldest group must be evicted"
    );
    assert!(store.lookup(tenant, &key2, &[10, 20, 30, 40]).is_some());

    Ok(())
}

#[test]
fn test_evict_returns_block_count() -> Result<(), ContextraError> {
    let store = TenantPrefixKvStore::new();
    let tenant = TenantId::try_new(10)?;
    let key = make_key("model-x");

    store.insert(
        tenant,
        &key,
        &[1, 2, 3, 4],
        vec![make_block(1, 50), make_block(2, 50)],
    )?;
    store.insert(
        tenant,
        &key,
        &[5, 6, 7, 8],
        vec![make_block(3, 50), make_block(4, 50), make_block(5, 50)],
    )?;

    let evicted_count = store.evict(tenant, &key)?;
    assert_eq!(evicted_count, 5);

    assert!(store.lookup(tenant, &key, &[1, 2, 3, 4]).is_none());

    Ok(())
}

#[test]
fn test_concurrency_8_threads() -> Result<(), ContextraError> {
    let store = Arc::new(TenantPrefixKvStore::new().with_byte_budget_per_tenant(1024 * 1024));
    let key = make_key("concurrent-model");

    let mut handles = vec![];
    for thread_idx in 0..8 {
        let store_clone = Arc::clone(&store);
        let key_clone = key.clone();
        let handle = std::thread::spawn(move || {
            let tenant = TenantId::try_new((thread_idx % 2 + 1) as u64).unwrap();
            for i in 1..=50 {
                let seq = vec![thread_idx as u32 + 1, i as u32, i as u32 + 100, 200, 300];
                let blocks = vec![make_block(i, 256)];
                store_clone
                    .insert(tenant, &key_clone, &seq, blocks)
                    .unwrap();

                let res = store_clone.lookup(tenant, &key_clone, &seq);
                assert!(res.is_some());
            }
        });
        handles.push(handle);
    }

    for h in handles {
        h.join().expect("thread join");
    }

    Ok(())
}

#[test]
fn test_arc_dyn_trait_object() -> Result<(), ContextraError> {
    let store: Arc<dyn KvPrefixStore> = Arc::new(TenantPrefixKvStore::new());
    let tenant = TenantId::try_new(99)?;
    let key = make_key("dyn-model");
    let tokens = vec![1, 2, 3, 4, 5];

    store.insert(tenant, &key, &tokens, vec![make_block(1, 64)])?;
    let hit = store.lookup(tenant, &key, &tokens).expect("dyn hit");
    assert_eq!(hit.matched_tokens, 5);

    let evicted = store.evict(tenant, &key)?;
    assert_eq!(evicted, 1);

    Ok(())
}
