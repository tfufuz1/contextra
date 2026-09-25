// FILE-CONTEXT
// ZWECK: Tenant-Isolations-Tests für den KV-Cache (Task KV-05 / INV-TENANT).
// STAND: TS:2026-09-15T00:00:00Z

use contextra_kvcache::{emergency_wipe, KvReusePolicy, KvSegment, TenantIsolatedKvStore};
use contextra_types::{ContextraError, TenantId};

#[test]
fn test_tenant_isolation_direct_lookup_and_segments() -> Result<(), ContextraError> {
    let store = TenantIsolatedKvStore::new();

    let tenant_a = TenantId::try_new(10)?;
    let tenant_b = TenantId::try_new(20)?;

    let seg_a1 = KvSegment::new(tenant_a, 101, vec![0xAA; 256]);
    let seg_a2 = KvSegment::new(tenant_a, 102, vec![0xBB; 256]);

    store.insert_segment(tenant_a, seg_a1);
    store.insert_segment(tenant_a, seg_a2);

    // Tenant A's data must be present for Tenant A
    assert_eq!(store.get_tenant_segment_len(tenant_a), 2);
    let mut segs_a = store.get_segments(tenant_a);
    segs_a.sort();
    assert_eq!(segs_a, vec![101, 102]);
    assert_eq!(
        store.get_segment_bytes(tenant_a, 101),
        Some(vec![0xAA; 256])
    );

    // Tenant B MUST NOT see Tenant A's segments or length
    assert_eq!(store.get_tenant_segment_len(tenant_b), 0);
    assert!(store.get_segments(tenant_b).is_empty());
    assert!(store.get_segment_bytes(tenant_b, 101).is_none());
    assert!(store.get_segment_bytes(tenant_b, 102).is_none());

    Ok(())
}

#[test]
fn test_tenant_isolation_prefix_radix_tree() -> Result<(), ContextraError> {
    let store = TenantIsolatedKvStore::new();

    let tenant_a = TenantId::try_new(100)?;
    let tenant_b = TenantId::try_new(200)?;

    let shared_tokens = vec![1, 2, 3, 4, 5, 6, 7, 8];
    store.insert_token_sequence(tenant_a, &shared_tokens, 500)?;
    store.insert_segment(tenant_a, KvSegment::new(tenant_a, 500, vec![0x11; 128]));

    // Tenant A finds the prefix match
    let match_a = store.find_prefix_match(tenant_a, &shared_tokens, KvReusePolicy::Always, None);
    assert!(match_a.is_some());
    let (pm_a, _guard_a) = match_a.unwrap();
    assert_eq!(pm_a.block_id, 500);

    // Tenant B searching the same sequence MUST NOT match Tenant A's prefix
    let match_b = store.find_prefix_match(tenant_b, &shared_tokens, KvReusePolicy::Always, None);
    assert!(
        match_b.is_none(),
        "Tenant B must NOT match Tenant A's prefix in PrefixRadixTree"
    );

    // Tenant B cannot acquire block guard for Tenant A's block
    let guard_b = store.acquire_block_guard(tenant_b, 500, None);
    assert!(
        guard_b.is_none(),
        "Tenant B must NOT acquire block guard for Tenant A's block ID"
    );

    Ok(())
}

#[test]
fn test_tenant_isolation_eviction_and_fair_recycling() -> Result<(), ContextraError> {
    let store = TenantIsolatedKvStore::new();

    let tenant_a = TenantId::try_new(1)?;
    let tenant_b = TenantId::try_new(2)?;

    // Populate Tenant A and Tenant B segments
    for i in 1..=5 {
        store.insert_segment(tenant_a, KvSegment::new(tenant_a, i, vec![0xAA; 100]));
        store.insert_segment(tenant_b, KvSegment::new(tenant_b, i + 10, vec![0xBB; 100]));
    }

    assert_eq!(store.get_tenant_segment_len(tenant_a), 5);
    assert_eq!(store.get_tenant_segment_len(tenant_b), 5);

    // Evict 200 bytes (2 segments total across tenants)
    let freed = store.evict_lru_fair(200);
    assert!(freed >= 200);

    // Verify both tenants are active and neither was completely wiped or starved
    let len_a = store.get_tenant_segment_len(tenant_a);
    let len_b = store.get_tenant_segment_len(tenant_b);

    assert!(
        len_a < 5 || len_b < 5,
        "At least one tenant segment must be evicted"
    );
    assert!(
        len_a > 0 && len_b > 0,
        "Neither tenant should be completely starved"
    );

    // Check that Tenant A still only sees Tenant A segments
    for id in store.get_segments(tenant_a) {
        assert!(id <= 5, "Tenant A segment IDs must be <= 5");
    }
    for id in store.get_segments(tenant_b) {
        assert!(id >= 11, "Tenant B segment IDs must be >= 11");
    }

    Ok(())
}

#[test]
fn test_tenant_isolation_rollback_no_cross_tenant_impact() -> Result<(), ContextraError> {
    let store = TenantIsolatedKvStore::new();

    let tenant_a = TenantId::try_new(301)?;
    let tenant_b = TenantId::try_new(302)?;

    for id in 1..=3 {
        store.insert_segment(tenant_a, KvSegment::new(tenant_a, id, vec![0x11; 64]));
        store.insert_segment(tenant_b, KvSegment::new(tenant_b, id + 10, vec![0x22; 64]));
    }

    // Rollback Tenant A's transaction with segment IDs [1, 2]
    store.on_rollback(tenant_a, &[1, 2]);

    // Tenant A has 1 remaining segment (3)
    assert_eq!(store.get_tenant_segment_len(tenant_a), 1);
    assert_eq!(store.get_segments(tenant_a), vec![3]);

    // Tenant B is completely unaffected
    assert_eq!(store.get_tenant_segment_len(tenant_b), 3);
    let mut segs_b = store.get_segments(tenant_b);
    segs_b.sort();
    assert_eq!(segs_b, vec![11, 12, 13]);

    Ok(())
}

#[test]
fn test_tenant_isolation_emergency_wipe() -> Result<(), ContextraError> {
    let store = TenantIsolatedKvStore::new();

    let tenant_a = TenantId::try_new(401)?;
    let tenant_b = TenantId::try_new(402)?;

    store.insert_segment(tenant_a, KvSegment::new(tenant_a, 1, vec![0xFF; 512]));
    store.insert_segment(tenant_b, KvSegment::new(tenant_b, 2, vec![0xEE; 512]));

    assert_eq!(store.get_tenant_segment_len(tenant_a), 1);
    assert_eq!(store.get_tenant_segment_len(tenant_b), 1);

    emergency_wipe(&store);

    assert_eq!(store.get_tenant_segment_len(tenant_a), 0);
    assert_eq!(store.get_tenant_segment_len(tenant_b), 0);
    assert!(store.get_segments(tenant_a).is_empty());
    assert!(store.get_segments(tenant_b).is_empty());

    Ok(())
}
