#![cfg(feature = "content-addressed-kv-cache")]

use contextra_kvcache::{
    ContentAddressedKvStore, KvLookupResult, KvReusePolicy, SemanticCacheConfig, SemanticEmbedder,
};
use contextra_types::{ContextraError, TenantId};
use std::sync::Arc;

struct DummyEmbedder;

impl SemanticEmbedder for DummyEmbedder {
    fn embed(&self, _tokens: &[u32]) -> Result<Vec<f32>, ContextraError> {
        Ok(vec![1.0, 0.0, 0.0, 0.0])
    }
}

#[test]
fn test_cross_tenant_content_addressed_isolation_exact_and_content_hash() {
    let tenant_a = TenantId::try_new(101).expect("valid tenant_a");
    let tenant_b = TenantId::try_new(202).expect("valid tenant_b");

    let mut store = ContentAddressedKvStore::new(tenant_a);

    let tokens = vec![1000, 2000, 3000, 4000, 5000];

    // Tenant A inserts segment 1
    store
        .insert(tenant_a, &tokens, 1)
        .expect("Tenant A insert success");

    // Tenant A lookup should hit ExactPrefixHit for segment 1
    let lookup_a = store.lookup(tenant_a, &tokens);
    match lookup_a {
        KvLookupResult::ExactPrefixHit(ref seg) => {
            assert_eq!(seg.segment_id, 1);
            assert_eq!(seg.tenant_id, tenant_a);
        }
        other => panic!("Tenant A should get ExactPrefixHit, got {:?}", other),
    }

    // Tenant B looks up identical tokens before inserting anything for Tenant B.
    // SECURITY CRITICAL: MUST BE A MISS! MUST NOT LEAK TENANT A DATA TO TENANT B!
    let lookup_b_before = store.lookup(tenant_b, &tokens);
    assert_eq!(
        lookup_b_before,
        KvLookupResult::Miss,
        "SECURITY VIOLATION: Tenant B received a cache hit for Tenant A's content!"
    );

    // Tenant B inserts segment 2 with identical token content
    store
        .insert(tenant_b, &tokens, 2)
        .expect("Tenant B insert success");

    // Tenant A lookup must still return segment 1 for Tenant A
    let lookup_a_after = store.lookup(tenant_a, &tokens);
    match lookup_a_after {
        KvLookupResult::ExactPrefixHit(ref seg) => {
            assert_eq!(seg.segment_id, 1);
            assert_eq!(seg.tenant_id, tenant_a);
        }
        other => panic!("Tenant A should get segment 1, got {:?}", other),
    }

    // Tenant B lookup must return segment 2 for Tenant B
    let lookup_b_after = store.lookup(tenant_b, &tokens);
    match lookup_b_after {
        KvLookupResult::ExactPrefixHit(ref seg) => {
            assert_eq!(seg.segment_id, 2);
            assert_eq!(seg.tenant_id, tenant_b);
        }
        other => panic!("Tenant B should get segment 2, got {:?}", other),
    }
}

#[test]
fn test_cross_tenant_content_hash_hit_isolation() {
    let tenant_a = TenantId::try_new(10).expect("valid tenant_a");
    let tenant_b = TenantId::try_new(20).expect("valid tenant_b");

    let mut store = ContentAddressedKvStore::new(tenant_a);

    // Tokens shorter than default min_prefix_len (3 < 4) to force content_index lookup
    let short_tokens = vec![111, 222, 333];

    // Tenant A inserts short_tokens -> goes to content_index
    store
        .insert(tenant_a, &short_tokens, 100)
        .expect("insert short_tokens for tenant A");

    // Tenant A lookup gets ContentHashHit
    let res_a = store.lookup(tenant_a, &short_tokens);
    match res_a {
        KvLookupResult::ContentHashHit(ref seg) => {
            assert_eq!(seg.segment_id, 100);
            assert_eq!(seg.tenant_id, tenant_a);
        }
        other => panic!("Expected ContentHashHit for Tenant A, got {:?}", other),
    }

    // Tenant B lookup MUST BE MISS
    let res_b = store.lookup(tenant_b, &short_tokens);
    assert_eq!(
        res_b,
        KvLookupResult::Miss,
        "SECURITY VIOLATION: Tenant B received ContentHashHit for Tenant A!"
    );
}

#[test]
fn test_cross_tenant_semantic_similarity_hit_isolation() {
    let tenant_a = TenantId::try_new(1).expect("valid tenant_a");
    let tenant_b = TenantId::try_new(2).expect("valid tenant_b");

    let embedder = Arc::new(DummyEmbedder);
    let config = SemanticCacheConfig::new(embedder, 0.8);

    let store = ContentAddressedKvStore::new(tenant_a).with_semantic_config(config);

    // Use Never reuse policy to bypass position tree and test semantic lookup
    let mut store = store.with_reuse_policy(KvReusePolicy::Never);

    let tokens = vec![9, 8, 7, 6];

    store
        .insert(tenant_a, &tokens, 500)
        .expect("Tenant A insert");

    // Lookup for Tenant B must return Miss even with semantic similarity match
    let res_b = store.lookup(tenant_b, &tokens);
    assert_eq!(
        res_b,
        KvLookupResult::Miss,
        "SECURITY VIOLATION: Tenant B received SemanticSimilarityHit for Tenant A!"
    );
}
