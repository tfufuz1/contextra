use contextra_storage::ports::{Error, KeyState};
use contextra_storage::recovery::integrity::{reconcile_cluster_key, KeyRecoveryState};

#[test]
fn provider_absence_requires_durable_erase_evidence() {
    assert!(matches!(reconcile_cluster_key(KeyState::Absent, false), Err(Error::KeyLost)));
    assert!(matches!(reconcile_cluster_key(KeyState::Absent, true), Ok(KeyRecoveryState::Shredded)));
}

#[test]
fn disabled_key_is_retryable_and_never_proves_shredding() {
    assert!(matches!(
        reconcile_cluster_key(KeyState::Disabled, false),
        Err(Error::KeyUnavailable { .. }),
    ));
    assert!(matches!(
        reconcile_cluster_key(KeyState::Disabled, true),
        Err(Error::KeyUnavailable { .. }),
    ));
}
