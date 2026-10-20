// FILE-CONTEXT
// ZWECK: Edge-Case- und Concurrency-Tests für TenantIsolatedKvStore (Sharding, Eviction, Pinning, Panics).
// STAND: TS:2026-10-21T00:00:00Z

use std::sync::{Arc, Barrier};
use std::thread;

use contextra_kvcache::{KvReusePolicy, KvSegment, TenantIsolatedKvStore};
use contextra_types::error::{CacheDirective, StepId};
use contextra_types::TenantId;

#[test]
fn test_eviction_vs_insert_vs_pin() {
    let store = TenantIsolatedKvStore::with_capacity(3);
    let tenant = TenantId::try_new(10).unwrap();

    // Insert 1 unpinned, 1 pinned (TTL: None), and 1 pinned with TTL (long)
    let seg1 = KvSegment::new(tenant, 1, vec![0x11; 128]);
    let seg2 = KvSegment::new(tenant, 2, vec![0x22; 128]);
    let seg3 = KvSegment::new(tenant, 3, vec![0x33; 128]);

    store
        .insert_segment_with_directive(tenant, seg1, CacheDirective::Auto)
        .unwrap();
    store
        .insert_segment_with_directive(tenant, seg2, CacheDirective::Pin { ttl: None })
        .unwrap();
    store
        .insert_segment_with_directive(
            tenant,
            seg3,
            CacheDirective::Pin {
                ttl: Some(std::time::Duration::from_secs(3600)),
            },
        )
        .unwrap();

    assert_eq!(store.get_tenant_segment_len(tenant), 3);

    // Eviction attempt: unpinned seg1 should be evicted first, while pinned seg2 and seg3 survive.
    let freed = store.evict_lru_fair(128);
    assert!(freed >= 128);
    let remaining = store.get_segments(tenant);
    assert!(!remaining.contains(&1), "Unpinned seg1 must be evicted");
    assert!(
        remaining.contains(&2),
        "Pinned seg2 (TTL None) must NOT be evicted"
    );
    assert!(
        remaining.contains(&3),
        "Pinned seg3 (TTL unexpired) must NOT be evicted"
    );

    // Further eviction cannot evict pinned items
    let freed_pinned = store.evict_lru_fair(128);
    assert_eq!(freed_pinned, 0, "Pinned segments cannot be evicted");

    // Test release_step with unmatched/duplicate step_id is harmless
    store.release_step(tenant, StepId(9999));
    store.release_step(tenant, StepId(9999)); // Duplicate
    assert_eq!(
        store.get_tenant_segment_len(tenant),
        2,
        "Duplicate release_step must be harmless"
    );

    // Insert transient segment bound to step 1
    let seg4 = KvSegment::new(tenant, 4, vec![0x44; 128]);
    store
        .insert_segment_with_directive(
            tenant,
            seg4,
            CacheDirective::ReleaseAfterStep { step_id: StepId(1) },
        )
        .unwrap();
    assert_eq!(store.get_tenant_segment_len(tenant), 3);

    store.release_step(tenant, StepId(1));
    assert_eq!(store.get_tenant_segment_len(tenant), 2);
    assert!(!store.get_segments(tenant).contains(&4));
}

#[test]
fn test_capacity_and_shard_variations() {
    // 1. Capacity = 1
    let store_cap1 = TenantIsolatedKvStore::with_capacity(1);
    let tenant1 = TenantId::try_new(100).unwrap();

    store_cap1.insert_segment(tenant1, KvSegment::new(tenant1, 1, vec![10; 64]));
    assert_eq!(store_cap1.get_tenant_segment_len(tenant1), 1);

    store_cap1.insert_segment(tenant1, KvSegment::new(tenant1, 2, vec![20; 64]));
    assert_eq!(store_cap1.get_tenant_segment_len(tenant1), 1);
    assert_eq!(store_cap1.get_segments(tenant1), vec![2]);

    // 2. Large capacity (NonZeroUsize boundary check)
    let store_large = TenantIsolatedKvStore::with_capacity(1_000_000);
    store_large.insert_segment(tenant1, KvSegment::new(tenant1, 1, vec![10; 64]));
    assert_eq!(store_large.get_tenant_segment_len(tenant1), 1);

    // 3. Shard count = 1
    let store_shard1 = TenantIsolatedKvStore::with_shard_count(1);
    store_shard1.insert_segment(tenant1, KvSegment::new(tenant1, 1, vec![10; 64]));
    assert_eq!(store_shard1.get_tenant_segment_len(tenant1), 1);

    // 4. Shard count non-power-of-two (should be rounded up to power of two, e.g., 3 -> 4, 5 -> 8)
    let store_shard3 = TenantIsolatedKvStore::with_shard_count(3);
    let store_shard5 = TenantIsolatedKvStore::with_shard_count(5);

    for t in 1..=10 {
        let tid = TenantId::try_new(t).unwrap();
        store_shard3.insert_segment(tid, KvSegment::new(tid, 100, vec![1; 32]));
        store_shard5.insert_segment(tid, KvSegment::new(tid, 100, vec![1; 32]));
    }

    for t in 1..=10 {
        let tid = TenantId::try_new(t).unwrap();
        assert_eq!(store_shard3.get_tenant_segment_len(tid), 1);
        assert_eq!(store_shard5.get_tenant_segment_len(tid), 1);
    }
}

#[test]
fn test_tenant_isolation_edge_cases() {
    let store = TenantIsolatedKvStore::new();

    let tenant_a = TenantId::try_new(10).unwrap();
    let tenant_b = TenantId::try_new(20).unwrap();

    // Same segment ID across different tenants
    let seg_a = KvSegment::new(tenant_a, 500, vec![0xAA; 128]);
    let seg_b = KvSegment::new(tenant_b, 500, vec![0xBB; 256]);

    store.insert_segment(tenant_a, seg_a);
    store.insert_segment(tenant_b, seg_b);

    assert_eq!(
        store.get_segment_bytes(tenant_a, 500),
        Some(vec![0xAA; 128])
    );
    assert_eq!(
        store.get_segment_bytes(tenant_b, 500),
        Some(vec![0xBB; 256])
    );

    // Prefix Radix isolated by tenant
    let tokens = vec![1, 2, 3, 4, 5];
    store.insert_token_sequence(tenant_a, &tokens, 500).unwrap();

    let match_a = store.find_prefix_match(tenant_a, &tokens, KvReusePolicy::Always, None);
    assert!(match_a.is_some());

    let match_b = store.find_prefix_match(tenant_b, &tokens, KvReusePolicy::Always, None);
    assert!(
        match_b.is_none(),
        "Tenant B must not match Tenant A prefix sequence"
    );

    // Eviction on tenant A does not affect tenant B
    let freed = store.evict_lru_fair(128);
    assert!(freed >= 128);
    let remaining_a = store.get_tenant_segment_len(tenant_a);
    let remaining_b = store.get_tenant_segment_len(tenant_b);
    assert_eq!(remaining_a + remaining_b, 1);
}

#[test]
fn test_concurrent_same_segment_id_insert() {
    let store = Arc::new(TenantIsolatedKvStore::new());
    let tenant = TenantId::try_new(42).unwrap();
    let barrier = Arc::new(Barrier::new(4));

    let mut handles = vec![];
    for thread_idx in 0..4 {
        let store_cl = Arc::clone(&store);
        let bar_cl = Arc::clone(&barrier);
        handles.push(thread::spawn(move || {
            bar_cl.wait();
            let data = vec![(thread_idx + 1) as u8; 64];
            let seg = KvSegment::new(tenant, 100, data);
            store_cl.insert_segment(tenant, seg);
        }));
    }

    for h in handles {
        h.join().unwrap();
    }

    assert_eq!(store.get_tenant_segment_len(tenant), 1);
    let bytes = store.get_segment_bytes(tenant, 100).unwrap();
    assert_eq!(bytes.len(), 64);
}

#[test]
fn test_insert_during_find_prefix_match() {
    let store = Arc::new(TenantIsolatedKvStore::new());
    let tenant = TenantId::try_new(1).unwrap();

    let tokens = vec![10, 20, 30, 40, 50];
    store.insert_segment(tenant, KvSegment::new(tenant, 1, vec![0xFF; 128]));
    store.insert_token_sequence(tenant, &tokens, 1).unwrap();

    let store_cl1 = Arc::clone(&store);
    let store_cl2 = Arc::clone(&store);
    let barrier = Arc::new(Barrier::new(2));

    let b1 = Arc::clone(&barrier);
    let h1 = thread::spawn(move || {
        b1.wait();
        store_cl1.find_prefix_match(
            tenant,
            &[10, 20, 30, 40, 50, 60],
            KvReusePolicy::Always,
            None,
        )
    });

    let b2 = Arc::clone(&barrier);
    let h2 = thread::spawn(move || {
        b2.wait();
        let tokens2 = vec![10, 20, 30, 70, 80];
        store_cl2.insert_segment(tenant, KvSegment::new(tenant, 2, vec![0xEE; 128]));
        store_cl2.insert_token_sequence(tenant, &tokens2, 2)
    });

    let match_res = h1.join().unwrap();
    let insert_res = h2.join().unwrap();

    assert!(insert_res.is_ok());
    assert!(match_res.is_some());
    let (pm, guard) = match_res.unwrap();
    assert_eq!(pm.block_id, 1);
    assert_eq!(guard.active_refs(), 1);
}

#[test]
fn test_spill_handler_panic_handled_gracefully() {
    let store = Arc::new(TenantIsolatedKvStore::new());
    let tenant = TenantId::try_new(7).unwrap();

    // Set a panicking spill handler
    store.set_spill_handler(Arc::new(|_t, _id, _bytes| {
        panic!("Simulated SpillHandler failure!");
    }));

    for i in 1..=5 {
        store.insert_segment(tenant, KvSegment::new(tenant, i, vec![i as u8; 256]));
    }

    // Trigger eviction on background thread catching unwinds to verify store state integrity
    let store_cl = Arc::clone(&store);
    let handle = thread::spawn(move || {
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            store_cl.evict_lru_fair(256);
        }))
    });

    let res = handle.join().unwrap();
    assert!(
        res.is_err(),
        "SpillHandler panic should occur inside catch_unwind"
    );

    // Store locks must NOT be poisoned (parking_lot RwLock is poison-free) and store must remain fully functional
    store.insert_segment(tenant, KvSegment::new(tenant, 99, vec![99; 128]));
    assert!(store.get_segment_bytes(tenant, 99).is_some());
}
