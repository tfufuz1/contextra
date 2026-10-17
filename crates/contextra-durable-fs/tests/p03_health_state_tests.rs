use contextra_durable_fs::{AtomicHealthState, HealthEvent, HealthState};

#[test]
fn test_p03_health_state_initial_state_is_healthy() {
    let state = HealthState::default();
    assert_eq!(state, HealthState::Healthy);
    assert!(state.allows_writes());
    assert!(!state.is_poisoned());

    let atomic = AtomicHealthState::new();
    assert_eq!(atomic.load(), HealthState::Healthy);
}

#[test]
fn test_p03_health_state_fsync_error_transitions_to_poisoned() {
    let state = HealthState::Healthy;
    let next = state.transition(HealthEvent::FsyncFailure);
    assert_eq!(next, HealthState::Poisoned);
    assert!(!next.allows_writes());
    assert!(next.is_poisoned());
}

#[test]
fn test_p03_health_state_enospc_transitions_to_readonly() {
    let state = HealthState::Healthy;
    let next = state.transition(HealthEvent::Enospc);
    assert_eq!(next, HealthState::ReadOnly);
    assert!(!next.allows_writes());
    assert!(!next.is_poisoned());
}

#[test]
fn test_p03_health_state_read_checksum_warning_transitions_to_degraded() {
    let state = HealthState::Healthy;
    let next = state.transition(HealthEvent::ReadChecksumWarning);
    assert_eq!(next, HealthState::Degraded);
    assert!(next.allows_writes());
    assert!(!next.is_poisoned());
}

#[test]
fn test_p03_health_state_poisoned_never_returns_automatically() {
    let state = HealthState::Poisoned;
    let next1 = state.transition(HealthEvent::ReadChecksumWarning);
    assert_eq!(next1, HealthState::Poisoned);

    let next2 = state.transition(HealthEvent::Enospc);
    assert_eq!(next2, HealthState::Poisoned);

    let next3 = state.transition(HealthEvent::FsyncFailure);
    assert_eq!(next3, HealthState::Poisoned);
}

#[test]
fn test_p03_atomic_health_state_concurrent_transitions() {
    let atomic = AtomicHealthState::new();
    assert_eq!(atomic.load(), HealthState::Healthy);

    atomic.record_event(HealthEvent::ReadChecksumWarning);
    assert_eq!(atomic.load(), HealthState::Degraded);

    atomic.record_event(HealthEvent::Enospc);
    assert_eq!(atomic.load(), HealthState::ReadOnly);

    atomic.record_event(HealthEvent::FsyncFailure);
    assert_eq!(atomic.load(), HealthState::Poisoned);
}
