#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]

// FILE-CONTEXT
// STAND: 2026-10-13
// ZWECK: Crash- und Rollback-Tests für RevocationLog (B14).
// INVARIANTEN: Atomares Ersetzen via FaultInjector, Monoton-Generationen, Fail-Closed Rollback-Erkennung.

#![forbid(unsafe_code)]

use contextra_crypto::error::{CryptoError, Result};
use contextra_crypto::revocation_log::{marker_path_for, RevocationLog, RevocationTarget};
use contextra_durable_fs::{atomic_replace_with_injector, FaultInjector};
use contextra_ports::SystemClock;
use ed25519_dalek::SigningKey;
use rand::rngs::OsRng;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tempfile::TempDir;

fn test_keypair() -> (SigningKey, ed25519_dalek::VerifyingKey) {
    let sk = SigningKey::generate(&mut OsRng);
    let vk = sk.verifying_key();
    (sk, vk)
}

struct FailRenameInjector {
    fail: AtomicBool,
}

impl FaultInjector for FailRenameInjector {
    fn on_rename(&self, _from: &Path, _to: &Path) -> std::io::Result<()> {
        if self.fail.load(Ordering::SeqCst) {
            Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Simulated crash/failure during rename",
            ))
        } else {
            Ok(())
        }
    }
}

#[test]
fn test_simulated_crash_during_atomic_replace_preserves_previous_state() -> Result<()> {
    let temp_dir = TempDir::new().map_err(|e| CryptoError::Crypto(e.to_string()))?;
    let log_file = temp_dir.path().join("revocation.log");
    let (sk, vk) = test_keypair();
    let clock = Arc::new(SystemClock::new());

    // 1. Initial log creation with 1 entry
    let log = RevocationLog::open_or_create(&log_file, clock.clone(), Some(sk.clone()), vk)?;
    log.append(RevocationTarget::Group(1))?;
    let (initial_gen, initial_head) = log.export_head();
    assert_eq!(initial_gen, 1);
    assert_ne!(initial_head, [0u8; 32]);

    // 2. Simulate atomic replace write failure via FaultInjector
    let injector = FailRenameInjector {
        fail: AtomicBool::new(true),
    };
    let test_path = temp_dir.path().join("fault_test.tmp");
    let res = atomic_replace_with_injector(&test_path, b"data", &injector);
    assert!(res.is_err(), "atomic_replace MUST fail when rename fails");
    assert!(
        !test_path.exists(),
        "Target file MUST NOT exist on rename failure"
    );

    // 3. Reopen log to verify previous state remains completely uncorrupted
    let reopened = RevocationLog::open_or_create(&log_file, clock, None, vk)?;
    assert_eq!(reopened.len(), 1);
    assert_eq!(reopened.generation(), 1);
    assert!(reopened.is_revoked(&RevocationTarget::Group(1)));

    Ok(())
}

#[test]
fn test_rollback_to_older_log_state_triggers_fail_closed() -> Result<()> {
    let temp_dir = TempDir::new().map_err(|e| CryptoError::Crypto(e.to_string()))?;
    let log_file = temp_dir.path().join("revocation.log");
    let marker_file = marker_path_for(&log_file);
    let (sk, vk) = test_keypair();
    let clock = Arc::new(SystemClock::new());

    // 1. Write state at Gen 1
    let log1 = RevocationLog::open_or_create(&log_file, clock.clone(), Some(sk.clone()), vk)?;
    log1.append(RevocationTarget::Group(100))?;
    let (anchor_gen_1, _anchor_head_1) = log1.export_head();
    assert_eq!(anchor_gen_1, 1);

    // Backup Gen 1 files
    let log_bak_1 = std::fs::read(&log_file).map_err(|e| CryptoError::Crypto(e.to_string()))?;
    let marker_bak_1 =
        std::fs::read(&marker_file).map_err(|e| CryptoError::Crypto(e.to_string()))?;

    // 2. Write state at Gen 2 & Gen 3
    log1.append(RevocationTarget::Group(200))?;
    log1.append(RevocationTarget::Group(300))?;
    let (anchor_gen_3, anchor_head_3) = log1.export_head();
    assert_eq!(anchor_gen_3, 3);

    // Verify anchor 3 against active log passes
    assert!(log1
        .verify_against_anchor(anchor_gen_3, Some(anchor_head_3))
        .is_ok());

    // 3. Simulate Rollback Attack: Restore Gen 1 files over active log
    std::fs::write(&log_file, &log_bak_1).map_err(|e| CryptoError::Crypto(e.to_string()))?;
    std::fs::write(&marker_file, &marker_bak_1).map_err(|e| CryptoError::Crypto(e.to_string()))?;

    // 4. Reopen restored log
    let rolled_back_log = RevocationLog::open_or_create(&log_file, clock, None, vk)?;
    assert_eq!(rolled_back_log.generation(), 1);

    // Verify against external anchor (Gen 3) MUST detect rollback and trigger fail-closed state
    let verify_res = rolled_back_log.verify_against_anchor(anchor_gen_3, Some(anchor_head_3));
    assert!(
        matches!(verify_res, Err(CryptoError::IntegrityViolation)),
        "verify_against_anchor MUST fail with IntegrityViolation when log is rolled back"
    );

    // Fail-Closed Rule: ALL queries return true (revoked) for ANY group after rollback detection
    assert!(
        rolled_back_log.is_revoked(&RevocationTarget::Group(100)),
        "Revoked group 100 must remain revoked"
    );
    assert!(
        rolled_back_log.is_revoked(&RevocationTarget::Group(200)),
        "Rolled back group 200 MUST be reported as revoked (fail-closed)"
    );
    assert!(
        rolled_back_log.is_revoked(&RevocationTarget::Group(9999)),
        "Unrelated group 9999 MUST be reported as revoked (fail-closed)"
    );

    Ok(())
}

#[test]
fn test_export_head_and_generation_consistency() -> Result<()> {
    let temp_dir = TempDir::new().map_err(|e| CryptoError::Crypto(e.to_string()))?;
    let log_file = temp_dir.path().join("revocation.log");
    let (sk, vk) = test_keypair();
    let clock = Arc::new(SystemClock::new());

    let log = RevocationLog::open_or_create(&log_file, clock.clone(), Some(sk.clone()), vk)?;

    // Initial empty log export
    let (gen0, head0) = log.export_head();
    assert_eq!(gen0, 0);
    assert_eq!(head0, [0u8; 32]);

    // Append 3 entries and verify head updates monotonically
    let entry1 = log.append(RevocationTarget::Kek("kek-1".into()))?;
    let (gen1, head1) = log.export_head();
    assert_eq!(gen1, 1);
    assert_eq!(head1, entry1.entry_hash);

    let entry2 = log.append(RevocationTarget::Dek("dek-2".into()))?;
    let (gen2, head2) = log.export_head();
    assert_eq!(gen2, 2);
    assert_eq!(head2, entry2.entry_hash);

    let entry3 = log.append(RevocationTarget::Record("rec-3".into()))?;
    let (gen3, head3) = log.export_head();
    assert_eq!(gen3, 3);
    assert_eq!(head3, entry3.entry_hash);

    // Reopen and check exported head matches
    let reopened = RevocationLog::open_or_create(&log_file, clock, None, vk)?;
    let (r_gen, r_head) = reopened.export_head();
    assert_eq!(r_gen, 3);
    assert_eq!(r_head, entry3.entry_hash);

    Ok(())
}

#[test]
fn test_reopen_after_crash_preserves_all_revocations() -> Result<()> {
    let temp_dir = TempDir::new().map_err(|e| CryptoError::Crypto(e.to_string()))?;
    let log_file = temp_dir.path().join("revocation.log");
    let (sk, vk) = test_keypair();
    let clock = Arc::new(SystemClock::new());

    let targets = vec![
        RevocationTarget::Group(10),
        RevocationTarget::Group(20),
        RevocationTarget::Kek("kek-key-55".into()),
    ];

    {
        let log = RevocationLog::open_or_create(&log_file, clock.clone(), Some(sk), vk)?;
        for t in &targets {
            log.append(t.clone())?;
        }
        assert_eq!(log.len(), 3);
    }

    // Simulate process crash/restart by opening existing log file
    let reopened = RevocationLog::open_or_create(&log_file, clock, None, vk)?;
    assert_eq!(reopened.len(), 3);

    for t in &targets {
        assert!(
            reopened.is_revoked(t),
            "Every committed revocation MUST be preserved after restart"
        );
    }

    assert!(
        !reopened.is_revoked(&RevocationTarget::Group(999)),
        "Non-revoked target MUST return false in healthy log"
    );

    Ok(())
}
