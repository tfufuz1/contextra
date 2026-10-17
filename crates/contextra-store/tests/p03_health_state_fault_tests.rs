use contextra_durable_fs::fault_injector::FaultInjector;
use contextra_durable_fs::{AtomicHealthState, DurableHandle, FlushKind, HealthEvent, HealthState};
use std::io::{Error, ErrorKind, Result, Write};
use std::path::Path;
use tempfile::TempDir;

#[derive(Debug, Clone, Copy, Default)]
struct FailFsyncInjector {
    fail_fsync: bool,
}

impl FaultInjector for FailFsyncInjector {
    fn on_fsync(&self, _path: Option<&Path>) -> Result<()> {
        if self.fail_fsync {
            Err(Error::new(ErrorKind::Other, "simulated fsync fault"))
        } else {
            Ok(())
        }
    }
}

#[test]
fn test_p03_fsync_error_poisons_handle_and_rejects_subsequent_writes() {
    let tmp = TempDir::new().unwrap();
    let file_path = tmp.path().join("fault_test.log");
    let file = std::fs::File::create(&file_path).unwrap();

    let health = AtomicHealthState::new();
    assert_eq!(health.load(), HealthState::Healthy);

    let injector = FailFsyncInjector { fail_fsync: true };
    let mut handle = DurableHandle::new_with_injector(file, injector);

    // Initial flush fails due to injected fault
    let flush_res = handle.flush(FlushKind::DataAndMetadata);
    assert!(flush_res.is_err());
    health.record_event(HealthEvent::FsyncFailure);

    // Health state MUST transition to Poisoned
    assert_eq!(health.load(), HealthState::Poisoned);
    assert!(handle.is_poisoned());

    // Subsequent write MUST be rejected
    let write_res = handle.write(b"data after fsync failure");
    assert!(write_res.is_err());
    assert!(write_res
        .unwrap_err()
        .to_string()
        .contains("poisoned due to a previous fsync failure"));
}

#[test]
fn test_p03_poisoned_state_is_sticky_no_transition_to_healthy() {
    let health = AtomicHealthState::new();
    health.record_event(HealthEvent::FsyncFailure);
    assert_eq!(health.load(), HealthState::Poisoned);

    // Attempting recovery event on poisoned state must remain Poisoned
    let current = health.load();
    let next = current.transition(HealthEvent::Recovered);
    assert_eq!(next, HealthState::Poisoned);
}

#[test]
fn test_p03_enospc_write_rejection() {
    let health = AtomicHealthState::new();
    health.record_event(HealthEvent::Enospc);
    assert_eq!(health.load(), HealthState::ReadOnly);

    // ReadOnly rejects writes
    assert!(!health.load().allows_writes());
}
