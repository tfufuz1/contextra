//! Smoke-Test für den entkoppelten Kern von `contextra-kvcache` ohne Default-Features und ohne Workspace.

use contextra_ids::TenantId;
use contextra_kvcache::{KvSegment, TenantIsolatedKvStore};

#[test]
fn test_kvcache_core_smoke_without_workspace() {
    let store = TenantIsolatedKvStore::new();
    let tenant = TenantId::try_new(42).expect("valid tenant");

    let seg_id = 1001u64;
    let seg_data = vec![0xABu8; 128];
    let seg = KvSegment::new(tenant, seg_id, seg_data.clone());

    store.insert_segment(tenant, seg);
    assert_eq!(store.get_tenant_segment_len(tenant), 1);

    let fetched = store.get_segment_bytes(tenant, seg_id);
    assert_eq!(fetched, Some(seg_data));

    store.remove_segment(tenant, seg_id);
    assert_eq!(store.get_tenant_segment_len(tenant), 0);
}
