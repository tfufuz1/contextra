//! Tests für MVCC Snapshots gemäß 07-contextra-mvcc.md
//! Spec-IDs: MV-01, MV-02, T-SNAP-1, T-LIMIT-1

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use contextra_mvcc::error::MvccError;
use contextra_mvcc::snapshot::{SnapshotRegistry, TenantTag};
use proptest::prelude::*;
use std::sync::Arc;

#[test]
fn mv_01_min_read_snapshot_none_when_empty() {
    let registry = SnapshotRegistry::new();
    assert_eq!(registry.min_read_snapshot(), None);
}

#[test]
fn mv_02_min_for_compaction_returns_current_when_empty() {
    let registry = SnapshotRegistry::new();
    assert_eq!(registry.min_for_compaction(42), 42);
    assert_eq!(registry.min_for_compaction(1000), 1000);
}

#[test]
fn t_snap_1_two_tenants_isolation() {
    let registry = Arc::new(SnapshotRegistry::new());
    let tenant_a: TenantTag = [1u8; 16];
    let tenant_b: TenantTag = [2u8; 16];

    let handle_a = registry.open(tenant_a, 100).expect("open tenant a");
    assert_eq!(handle_a.seq().expect("seq"), 100);
    assert_eq!(registry.min_read_snapshot(), Some(100));
    assert_eq!(registry.min_for_compaction(150), 100);

    let handle_b = registry.open(tenant_b, 50).expect("open tenant b");
    assert_eq!(handle_b.seq().expect("seq"), 50);
    assert_eq!(registry.min_read_snapshot(), Some(50));
    assert_eq!(registry.min_for_compaction(150), 50);

    drop(handle_b);
    assert_eq!(registry.min_read_snapshot(), Some(100));
    assert_eq!(registry.min_for_compaction(150), 100);

    drop(handle_a);
    assert_eq!(registry.min_read_snapshot(), None);
    assert_eq!(registry.min_for_compaction(150), 150);
}

#[test]
fn t_limit_1_per_tenant_limit_1024() {
    let registry = Arc::new(SnapshotRegistry::new());
    let tenant_a: TenantTag = [3u8; 16];
    let tenant_b: TenantTag = [4u8; 16];

    let mut handles_a = Vec::with_capacity(1024);
    for i in 1..=1024 {
        handles_a.push(registry.open(tenant_a, i).expect("under limit"));
    }

    // 1025th snapshot for tenant_a must fail with LimitSnapshots
    let err = registry.open(tenant_a, 1025).unwrap_err();
    assert_eq!(err, MvccError::LimitSnapshots);

    // Other tenant should still be able to open snapshots up to limit
    let handle_b = registry.open(tenant_b, 2000).expect("tenant b under limit");
    assert_eq!(handle_b.seq().expect("seq"), 2000);

    // Dropping one snapshot allows opening another one for tenant_a
    handles_a.pop();
    let new_handle_a = registry.open(tenant_a, 1025).expect("slot freed");
    assert_eq!(new_handle_a.seq().expect("seq"), 1025);
}

proptest! {
    #[test]
    fn prop_mv_01_mv_02_lifecycle_invariants(
        seqs in proptest::collection::vec(1..10_000u64, 1..30),
        current_seq in 10_001..20_000u64,
    ) {
        let registry = Arc::new(SnapshotRegistry::new());
        let tenant: TenantTag = [5u8; 16];

        prop_assert_eq!(registry.min_read_snapshot(), None);
        prop_assert_eq!(registry.min_for_compaction(current_seq), current_seq);

        let mut handles = Vec::new();
        for &s in &seqs {
            let handle = registry.open(tenant, s).expect("within limit");
            handles.push(handle);
        }

        let expected_min = *seqs.iter().min().unwrap();
        prop_assert_eq!(registry.min_read_snapshot(), Some(expected_min));
        prop_assert_eq!(registry.min_for_compaction(current_seq), expected_min.min(current_seq));

        drop(handles);

        prop_assert_eq!(registry.min_read_snapshot(), None);
        prop_assert_eq!(registry.min_for_compaction(current_seq), current_seq);
    }
}
