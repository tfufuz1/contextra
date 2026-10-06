// FILE-CONTEXT
// ZWECK: Cross-Tenant Isolation Tests for KvLifecycleHooks port implementation in TenantIsolatedKvStore.

use contextra_kvcache::{KvSegment, TenantIsolatedKvStore};
use contextra_ports::KvLifecycleHooks;
use contextra_types::{DocId, TenantId};

#[test]
fn test_kv_lifecycle_hooks_on_rollback_tenant_isolation() {
    let store = TenantIsolatedKvStore::new();
    let tenant1 = TenantId::try_new(100).unwrap();
    let tenant2 = TenantId::try_new(200).unwrap();

    // Insert segments for tenant 1 and tenant 2
    store.insert_segment(tenant1, KvSegment::new(tenant1, 10, vec![0x11; 32]));
    store.insert_segment(tenant1, KvSegment::new(tenant1, 11, vec![0x12; 32]));
    store.insert_segment(tenant2, KvSegment::new(tenant2, 10, vec![0x22; 32]));

    assert_eq!(store.get_tenant_segment_len(tenant1), 2);
    assert_eq!(store.get_tenant_segment_len(tenant2), 1);

    // Call on_rollback on tenant1 via KvLifecycleHooks
    let hooks: &dyn KvLifecycleHooks = &store;
    hooks.on_rollback(tenant1, &[10]);

    // Assert segment 10 removed for tenant1, tenant1 segment 11 preserved, tenant2 completely untouched
    assert_eq!(store.get_tenant_segment_len(tenant1), 1);
    assert!(store.get_segment_bytes(tenant1, 10).is_none());
    assert!(store.get_segment_bytes(tenant1, 11).is_some());

    assert_eq!(store.get_tenant_segment_len(tenant2), 1);
    assert!(store.get_segment_bytes(tenant2, 10).is_some());
}

#[test]
fn test_kv_lifecycle_hooks_remove_doc_segments_tenant_isolation() {
    let store = TenantIsolatedKvStore::new();
    let tenant1 = TenantId::try_new(101).unwrap();
    let tenant2 = TenantId::try_new(202).unwrap();
    let doc_id = DocId::new(55);

    store.insert_segment(
        tenant1,
        KvSegment::new(tenant1, doc_id.inner() as u64, vec![0xA1; 16]),
    );
    store.insert_segment(
        tenant2,
        KvSegment::new(tenant2, doc_id.inner() as u64, vec![0xB2; 16]),
    );

    let hooks: &dyn KvLifecycleHooks = &store;
    hooks.remove_doc_segments(tenant1, doc_id);

    // Assert doc segment removed for tenant1, tenant2 remains unaffected
    assert_eq!(store.get_tenant_segment_len(tenant1), 0);
    assert_eq!(store.get_tenant_segment_len(tenant2), 1);
    assert!(store
        .get_segment_bytes(tenant2, doc_id.inner() as u64)
        .is_some());
}

#[test]
fn test_kv_lifecycle_hooks_purge_tenant_isolation() {
    let store = TenantIsolatedKvStore::new();
    let tenant1 = TenantId::try_new(301).unwrap();
    let tenant2 = TenantId::try_new(302).unwrap();

    for i in 1..=5 {
        store.insert_segment(tenant1, KvSegment::new(tenant1, i, vec![0x10; 64]));
        store.insert_segment(tenant2, KvSegment::new(tenant2, i + 10, vec![0x20; 64]));
    }

    assert_eq!(store.get_tenant_segment_len(tenant1), 5);
    assert_eq!(store.get_tenant_segment_len(tenant2), 5);

    let hooks: &dyn KvLifecycleHooks = &store;
    hooks.purge_tenant(tenant1);

    // Assert tenant1 fully purged, tenant2 completely preserved
    assert_eq!(store.get_tenant_segment_len(tenant1), 0);
    assert_eq!(store.get_tenant_segment_len(tenant2), 5);
}
