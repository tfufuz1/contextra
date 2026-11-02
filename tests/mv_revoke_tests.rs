//! Tests für Snapshot-Widerruf gemäß 07-contextra-mvcc.md
//! Spec-IDs: MV-03, MV-05, T-VER-1

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use contextra_mvcc::error::MvccError;
use contextra_mvcc::snapshot::{Matcher, SnapshotRegistry, TenantTag};
use std::sync::Arc;

#[test]
fn mv_03_revoke_covering_tenant_filtered() {
    let registry = Arc::new(SnapshotRegistry::new());
    let tenant_target: TenantTag = [10u8; 16];
    let tenant_other: TenantTag = [20u8; 16];
    let matcher: Matcher = ();

    let handle_t1 = registry.open(tenant_target, 100).expect("open t1");
    let handle_t2 = registry.open(tenant_target, 200).expect("open t2");
    let handle_other = registry.open(tenant_other, 100).expect("open other");

    // Revoke target tenant with seq <= 150
    let count = registry.revoke_covering(tenant_target, &matcher, 150);
    assert_eq!(count, 1);

    // handle_t1 (seq 100 <= 150) is revoked
    assert_eq!(handle_t1.seq(), Err(MvccError::SnapshotRevoked(100)));

    // handle_t2 (seq 200 > 150) is intact
    assert_eq!(handle_t2.seq(), Ok(200));

    // other tenant's snapshot at seq 100 remains completely unaffected
    assert_eq!(handle_other.seq(), Ok(100));
}

#[test]
fn mv_05_immediate_revocation_fail_closed() {
    let registry = Arc::new(SnapshotRegistry::new());
    let tenant: TenantTag = [11u8; 16];
    let matcher: Matcher = ();

    let handle = registry.open(tenant, 50).expect("open");
    assert_eq!(handle.seq(), Ok(50));

    // Immediate revocation
    let revoked = registry.revoke_covering(tenant, &matcher, 50);
    assert_eq!(revoked, 1);

    // Immediately returns SnapshotRevoked on subsequent access
    assert_eq!(handle.seq(), Err(MvccError::SnapshotRevoked(50)));
}

#[test]
fn t_ver_1_time_travel_erased() {
    let registry = Arc::new(SnapshotRegistry::new());
    let tenant: TenantTag = [12u8; 16];
    let matcher: Matcher = ();

    let h1 = registry.open(tenant, 10).expect("open 10");
    let h2 = registry.open(tenant, 20).expect("open 20");
    let h3 = registry.open(tenant, 30).expect("open 30");

    // Erase through seq 20
    let count = registry.revoke_covering(tenant, &matcher, 20);
    assert_eq!(count, 2);

    assert_eq!(h1.seq(), Err(MvccError::SnapshotRevoked(10)));
    assert_eq!(h2.seq(), Err(MvccError::SnapshotRevoked(20)));
    assert_eq!(h3.seq(), Ok(30));
}
