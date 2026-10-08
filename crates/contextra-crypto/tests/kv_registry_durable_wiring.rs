use contextra_crypto::{CryptoError, KeyManager, KeyRegistry, Result};
use contextra_ports::SystemClock;
use ed25519_dalek::SigningKey;
use rand::rngs::OsRng;
use std::sync::Arc;
use tempfile::tempdir;

fn test_keypair() -> (SigningKey, ed25519_dalek::VerifyingKey) {
    let sk = SigningKey::generate(&mut OsRng);
    let vk = sk.verifying_key();
    (sk, vk)
}

#[test]
fn test_restart_simulation_persists_group_revocation() -> Result<()> {
    let dir = tempdir().map_err(|e| CryptoError::Crypto(e.to_string()))?;
    let log_path = dir.path().join("kv-revocation.log");
    let (sk, vk) = test_keypair();
    let clock = Arc::new(SystemClock::new());

    let group_id = 42;

    {
        let registry =
            KeyRegistry::open_durable(&log_path, clock.clone(), Some(sk.clone()), vk, false)?;
        assert!(registry.is_durable());

        let revoked = registry.revoke_group(group_id)?;
        assert!(revoked, "First group revocation must succeed");
        assert!(registry.is_group_revoked(group_id));
        // drop registry to simulate shutdown
    }

    // Reopen durable registry on same log_path
    let reopened_registry = KeyRegistry::open_durable(&log_path, clock, Some(sk), vk, true)?;
    assert!(reopened_registry.is_durable());
    assert!(
        reopened_registry.is_group_revoked(group_id),
        "Group revocation must persist after reopening durable registry"
    );

    Ok(())
}

#[test]
fn test_deleted_log_or_marker_fails_open_durable() -> Result<()> {
    let dir = tempdir().map_err(|e| CryptoError::Crypto(e.to_string()))?;
    let log_path = dir.path().join("kv-revocation.log");
    let marker_path = dir.path().join("kv-revocation.log.initialized");
    let (sk, vk) = test_keypair();
    let clock = Arc::new(SystemClock::new());

    // 1. Create a valid log first
    {
        let registry =
            KeyRegistry::open_durable(&log_path, clock.clone(), Some(sk.clone()), vk, false)?;
        registry.revoke_group(100)?;
    }

    assert!(log_path.exists());
    assert!(marker_path.exists());

    // Case 1: Log file deleted, marker file still present -> open_durable MUST fail with Err
    std::fs::remove_file(&log_path).map_err(|e| CryptoError::Crypto(e.to_string()))?;
    let res1 = KeyRegistry::open_durable(&log_path, clock.clone(), Some(sk.clone()), vk, false);
    assert!(
        matches!(res1, Err(CryptoError::IntegrityViolation)),
        "Opening with missing log file but existing marker file MUST fail with IntegrityViolation"
    );

    // Case 2: Log file and marker file both deleted, but has_existing_keys = true -> open_durable MUST fail with Err
    if marker_path.exists() {
        std::fs::remove_file(&marker_path).map_err(|e| CryptoError::Crypto(e.to_string()))?;
    }
    let res2 = KeyRegistry::open_durable(&log_path, clock, Some(sk), vk, true);
    assert!(
        matches!(res2, Err(CryptoError::IntegrityViolation)),
        "Opening missing log when existing keys are present MUST fail with IntegrityViolation"
    );

    Ok(())
}

#[test]
fn test_poisoned_locks_return_error() -> Result<()> {
    let km = KeyManager::try_new("passphrase", b"salt1")?;
    let registry = Arc::new(KeyRegistry::new());

    // Populate group 1 (record 10) and group 2 (record 20) in registry
    let _payload1 = registry.encrypt_record(&km, 1, 10, b"rec1")?;
    let _payload2 = registry.encrypt_record(&km, 2, 20, b"rec2")?;

    // Poison the groups lock
    registry.poison_groups_lock_for_test();

    // Now attempting revoke_group MUST return Err (not Ok(false) or Ok(true) with skipped zeroizing)
    let res_group = registry.revoke_group(1);
    assert!(
        res_group.is_err(),
        "revoke_group on poisoned write lock MUST return Err"
    );

    // Revoking an existing active record on poisoned write lock MUST return Err
    let res_record = registry.revoke_record(2, 20);
    assert!(
        res_record.is_err(),
        "revoke_record on poisoned write lock MUST return Err"
    );

    Ok(())
}

#[test]
fn test_revoke_record_for_unknown_record_does_not_append_log() -> Result<()> {
    let (sk, vk) = test_keypair();
    let clock = Arc::new(SystemClock::new());
    let km = KeyManager::try_new("passphrase", b"salt1")?;

    let registry = KeyRegistry::new_in_memory(clock, Some(sk), vk);

    // Populate group 1 with record 10
    let _payload = registry.encrypt_record(&km, 1, 10, b"record 10 content")?;

    let initial_log_len = registry
        .revocation_log
        .as_ref()
        .map(|l| l.len())
        .unwrap_or(0);

    // Attempting to revoke an UNKNOWN record 999 in group 1
    let revoked = registry.revoke_record(1, 999)?;
    assert!(!revoked, "Revoking unknown record must return Ok(false)");

    let log_len_after = registry
        .revocation_log
        .as_ref()
        .map(|l| l.len())
        .unwrap_or(0);
    assert_eq!(
        initial_log_len, log_len_after,
        "Log length MUST remain unchanged when revoking an unknown record"
    );

    // Now revoking the EXISTING active record 10
    let revoked_active = registry.revoke_record(1, 10)?;
    assert!(
        revoked_active,
        "Revoking active record 10 must return Ok(true)"
    );

    let log_len_final = registry
        .revocation_log
        .as_ref()
        .map(|l| l.len())
        .unwrap_or(0);
    assert_eq!(
        initial_log_len + 1,
        log_len_final,
        "Log length MUST increment by 1 when revoking an active record"
    );

    Ok(())
}
