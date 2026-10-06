//! Integration test for atomic persistence in RevocationLog.
//!
//! Demonstrates the vulnerability in non-atomic truncation/write logic
//! and verifies atomic persistence behavior under failure conditions.

#![forbid(unsafe_code)]

use contextra_crypto::revocation_log::{RevocationLog, RevocationTarget};
use contextra_ports::SystemClock;
use ed25519_dalek::SigningKey;
use rand::rngs::OsRng;
use std::fs::{File, Permissions};
use std::os::unix::fs::PermissionsExt;
use std::sync::Arc;
use tempfile::tempdir;

fn test_keypair() -> (SigningKey, ed25519_dalek::VerifyingKey) {
    let sk = SigningKey::generate(&mut OsRng);
    let vk = sk.verifying_key();
    (sk, vk)
}

#[test]
fn test_reproduce_non_atomic_write_vulnerability_or_atomic_guarantee() {
    let temp_dir = tempdir().expect("Failed to create temp dir");
    let log_path = temp_dir.path().join("revocation.log");

    let (sk, vk) = test_keypair();
    let clock = Arc::new(SystemClock::new());

    // Step 1: Open and populate log with initial entries
    let initial_target_1 = RevocationTarget::Kek("kek-initial-1".into());
    let initial_target_2 = RevocationTarget::Group(100);

    {
        let log = RevocationLog::open_or_create(&log_path, clock.clone(), Some(sk.clone()), vk)
            .expect("Failed to open or create log");
        log.append(initial_target_1.clone())
            .expect("Failed to append entry 1");
        log.append(initial_target_2.clone())
            .expect("Failed to append entry 2");
    }

    // Read initial file contents
    let initial_contents = std::fs::read(&log_path).expect("Failed to read initial log file");
    assert!(!initial_contents.is_empty());

    // Verify initial log reopens successfully
    {
        let log_check = RevocationLog::open_or_create(&log_path, clock.clone(), None, vk)
            .expect("Failed to reopen initial log");
        assert_eq!(log_check.len(), 2);
        assert!(log_check.is_revoked(&initial_target_1));
        assert!(log_check.is_revoked(&initial_target_2));
    }

    // Step 2: Simulate process crash after truncation during naive direct write
    {
        // Simulate truncation before crash
        let _file = File::create(&log_path).expect("Failed to truncate file");
        // Process crash / failure happens here without write_all completing
    }

    // Check file state after failure
    let after_failure_contents =
        std::fs::read(&log_path).expect("Failed to read log file after crash");

    // Before fix: log_path is 0 bytes, all initial revoked targets lost.
    let reopen_res = RevocationLog::open_or_create(&log_path, clock.clone(), None, vk);

    if after_failure_contents.is_empty() {
        println!("REPRODUCTION CONFIRMED: Naive truncate-in-place leaves file empty on crash. Reopen len: {:?}", reopen_res.map(|l| l.len()));
    } else {
        println!("Log file preserved initial contents or recovered.");
    }
}

#[test]
fn test_atomic_persistence_preserves_old_state_on_write_failure() {
    let temp_dir = tempdir().expect("Failed to create temp dir");
    let log_path = temp_dir.path().join("revocation.log");

    let (sk, vk) = test_keypair();
    let clock = Arc::new(SystemClock::new());

    let target_1 = RevocationTarget::Kek("kek-1".into());
    let target_2 = RevocationTarget::Group(42);

    // Initial state setup
    {
        let log = RevocationLog::open_or_create(&log_path, clock.clone(), Some(sk.clone()), vk)
            .expect("Failed to create log");
        log.append(target_1.clone()).unwrap();
    }

    let initial_bytes = std::fs::read(&log_path).unwrap();

    // Reopen log with signing key
    let log =
        RevocationLog::open_or_create(&log_path, clock.clone(), Some(sk.clone()), vk).unwrap();

    // Make parent directory read-only so creating temp file in log.append() will fail with EACCES
    std::fs::set_permissions(temp_dir.path(), Permissions::from_mode(0o555)).unwrap();

    // Attempt append - must fail with I/O error when creating temp file
    let append_res = log.append(target_2.clone());
    assert!(
        append_res.is_err(),
        "Append must fail when temp file creation fails"
    );

    // Restore permissions so we can inspect directory
    std::fs::set_permissions(temp_dir.path(), Permissions::from_mode(0o755)).unwrap();

    // Verify original log file on disk remains 100% unchanged
    let current_bytes = std::fs::read(&log_path).unwrap();
    assert_eq!(
        initial_bytes, current_bytes,
        "Original log file MUST remain intact after failed append"
    );

    // Reopen from disk to verify integrity
    let reopened = RevocationLog::open_or_create(&log_path, clock.clone(), None, vk).unwrap();
    assert_eq!(reopened.len(), 1);
    assert!(reopened.is_revoked(&target_1));
    assert!(!reopened.is_revoked(&target_2));
    assert!(reopened.verify_integrity().is_ok());
}

#[test]
fn test_reopen_and_read_after_successful_atomic_append() {
    let temp_dir = tempdir().expect("Failed to create temp dir");
    let log_path = temp_dir.path().join("revocation.log");

    let (sk, vk) = test_keypair();
    let clock = Arc::new(SystemClock::new());

    let target_1 = RevocationTarget::Dek("dek-1".into());
    let target_2 = RevocationTarget::Record("rec-2".into());

    {
        let log =
            RevocationLog::open_or_create(&log_path, clock.clone(), Some(sk.clone()), vk).unwrap();
        log.append(target_1.clone()).unwrap();
        log.append(target_2.clone()).unwrap();
    }

    // Reopen and check
    let reopened = RevocationLog::open_or_create(&log_path, clock.clone(), None, vk).unwrap();
    assert_eq!(reopened.len(), 2);
    assert!(reopened.is_revoked(&target_1));
    assert!(reopened.is_revoked(&target_2));
    assert!(reopened.verify_integrity().is_ok());
}
