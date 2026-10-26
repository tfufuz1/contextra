use contextra_kvcache::store::{
    CacheDirective, StepId, TenantIsolatedKvStore, MAX_PINNED_BYTES_PER_TENANT,
};
use contextra_kvcache::KvSegment;
use contextra_types::{ContextraError, TenantId};
use std::time::Duration;

#[test]
fn test_cache_directive_pin_survives_eviction() {
    let store = TenantIsolatedKvStore::with_capacity(5);
    let tenant = TenantId::try_new(100).unwrap();

    // 1. Insert 1 pinned segment (id: 1) and 4 auto segments (ids: 2, 3, 4, 5)
    let pinned_seg = KvSegment::new(tenant, 1, vec![0xAA; 1024]);
    store
        .insert_segment_with_directive(tenant, pinned_seg, CacheDirective::Pin { ttl: None })
        .expect("Pinned segment insert should succeed");

    for id in 2..=5 {
        let auto_seg = KvSegment::new(tenant, id, vec![0xBB; 1024]);
        store
            .insert_segment_with_directive(tenant, auto_seg, CacheDirective::Auto)
            .expect("Auto segment insert should succeed");
    }

    assert_eq!(store.get_tenant_segment_len(tenant), 5);

    // 2. Trigger eviction to free 2048 bytes (2 segments worth)
    let freed = store.evict_lru_fair(2048);
    assert!(freed >= 2048, "Eviction must free at least 2048 bytes");

    // Pinned segment 1 MUST survive eviction!
    let segments_left = store.get_segments(tenant);
    assert!(
        segments_left.contains(&1),
        "Pinned segment (id: 1) MUST survive eviction!"
    );
    assert!(
        store.get_segment_bytes(tenant, 1).is_some(),
        "Pinned segment data MUST remain accessible"
    );
}

#[test]
fn test_cache_directive_pin_ttl_expiration_eviction() {
    let store = TenantIsolatedKvStore::with_capacity(5);
    let tenant = TenantId::try_new(101).unwrap();

    // Insert segment with a very short TTL pin (10 ms)
    let short_ttl_seg = KvSegment::new(tenant, 1, vec![0xCC; 512]);
    store
        .insert_segment_with_directive(
            tenant,
            short_ttl_seg,
            CacheDirective::Pin {
                ttl: Some(Duration::from_millis(10)),
            },
        )
        .expect("Short TTL segment insert should succeed");

    // Immediately after insertion, eviction cannot evict it
    let freed_immediate = store.evict_lru_fair(512);
    assert_eq!(
        freed_immediate, 0,
        "Unexpired TTL pinned segment must NOT be evicted"
    );
    assert_eq!(store.get_tenant_segment_len(tenant), 1);

    // Sleep for 20 ms to allow TTL to expire
    std::thread::sleep(Duration::from_millis(20));

    // Now eviction should evict the expired pinned segment
    let freed_after_ttl = store.evict_lru_fair(512);
    assert!(
        freed_after_ttl >= 512,
        "Expired TTL pinned segment MUST be evicted"
    );
    assert_eq!(store.get_tenant_segment_len(tenant), 0);
}

#[test]
fn test_cache_directive_pin_budget_exceeded() {
    let store = TenantIsolatedKvStore::new();
    let tenant = TenantId::try_new(102).unwrap();

    // Create segment larger than MAX_PINNED_BYTES_PER_TENANT
    let oversized_len = (MAX_PINNED_BYTES_PER_TENANT + 1024) as usize;
    let oversized_data = vec![0xDD; oversized_len];
    let seg = KvSegment::new(tenant, 1, oversized_data);

    let res = store.insert_segment_with_directive(tenant, seg, CacheDirective::Pin { ttl: None });

    assert!(
        res.is_err(),
        "Inserting segment exceeding MAX_PINNED_BYTES_PER_TENANT must return Err(PinBudgetExceeded)"
    );

    match res {
        Err(ContextraError::PinBudgetExceeded(msg)) => {
            assert!(
                msg.contains("exceeding limit"),
                "Error message must indicate exceeding limit: {msg}"
            );
        }
        other => panic!(
            "Expected ContextraError::PinBudgetExceeded, got {:?}",
            other
        ),
    }

    assert_eq!(
        store.get_tenant_segment_len(tenant),
        0,
        "Rejected segment must not be placed in store"
    );
}

#[test]
fn test_cache_directive_never_cache() {
    let store = TenantIsolatedKvStore::new();
    let tenant = TenantId::try_new(103).unwrap();

    let seg = KvSegment::new(tenant, 1, vec![0xEE; 1024]);
    store
        .insert_segment_with_directive(tenant, seg, CacheDirective::NeverCache)
        .expect("NeverCache call should return Ok");

    assert_eq!(
        store.get_tenant_segment_len(tenant),
        0,
        "Segment with NeverCache directive must never be stored in cache"
    );
    assert!(store.get_segment_bytes(tenant, 1).is_none());
}

#[test]
fn test_cache_directive_release_after_step() {
    let store = TenantIsolatedKvStore::new();
    let tenant = TenantId::try_new(104).unwrap();
    let step_a = StepId::new(10);
    let step_b = StepId::new(20);

    // Insert 2 segments for step A and 1 segment for step B
    let seg1 = KvSegment::new(tenant, 1, vec![0x01; 256]);
    let seg2 = KvSegment::new(tenant, 2, vec![0x02; 256]);
    let seg3 = KvSegment::new(tenant, 3, vec![0x03; 256]);

    store
        .insert_segment_with_directive(
            tenant,
            seg1,
            CacheDirective::ReleaseAfterStep { step_id: step_a },
        )
        .unwrap();
    store
        .insert_segment_with_directive(
            tenant,
            seg2,
            CacheDirective::ReleaseAfterStep { step_id: step_a },
        )
        .unwrap();
    store
        .insert_segment_with_directive(
            tenant,
            seg3,
            CacheDirective::ReleaseAfterStep { step_id: step_b },
        )
        .unwrap();

    assert_eq!(store.get_tenant_segment_len(tenant), 3);

    // Release step A
    store.release_step(tenant, step_a);

    assert_eq!(
        store.get_tenant_segment_len(tenant),
        1,
        "Step A segments must be released, only Step B segment remains"
    );
    assert!(store.get_segment_bytes(tenant, 1).is_none());
    assert!(store.get_segment_bytes(tenant, 2).is_none());
    assert!(store.get_segment_bytes(tenant, 3).is_some());

    // Release step B
    store.release_step(tenant, step_b);
    assert_eq!(store.get_tenant_segment_len(tenant), 0);
}
