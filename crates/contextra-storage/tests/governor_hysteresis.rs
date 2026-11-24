use contextra_storage::env::governor::{Governor, GovernorConfig, WorkClass};

#[test]
fn disk_full_hysteresis_preserves_urgent_work() {
    // INV-DISK-1: reserve pressure rejects writes but keeps erase progress.
    let governor = Governor::new(GovernorConfig {
        reserve_bytes: 100,
        recovery_headroom_bytes: 25,
        maintenance_io_basis_points: 5_000,
    })
    .expect("valid governor config");

    assert!(!governor.observe_free_space(100));
    assert!(governor.observe_free_space(99));
    assert!(!governor.admits(WorkClass::UserCommit));
    assert!(!governor.admits(WorkClass::Flush));
    assert!(governor.admits(WorkClass::Read));
    assert!(governor.admits(WorkClass::UrgentErase));
    assert!(governor.observe_free_space(124));
    assert!(!governor.observe_free_space(125));
    assert!(governor.admits(WorkClass::UserCommit));
}

#[test]
fn governor_rejects_invalid_configuration() {
    assert!(Governor::new(GovernorConfig {
        reserve_bytes: 100,
        recovery_headroom_bytes: 0,
        maintenance_io_basis_points: 5_000,
    })
    .is_none());
    assert!(Governor::new(GovernorConfig {
        reserve_bytes: 100,
        recovery_headroom_bytes: 1,
        maintenance_io_basis_points: 10_001,
    })
    .is_none());
}
