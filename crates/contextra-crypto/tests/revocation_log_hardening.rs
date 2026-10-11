// FILE-CONTEXT
// ZWECK: Hardening- und Invarianten-Tests für RevocationLog (W1-02).
// TEIL-TESTS: Error-Atomizität, Inkrementelle Verifikation, Signierte Marker, Restart-Roundtrip, is_durable.

use contextra_crypto::error::{CryptoError, Result};
use contextra_crypto::revocation_log::{marker_path_for, RevocationLog, RevocationTarget};
use contextra_ports::SystemClock;
use ed25519_dalek::SigningKey;
use rand::rngs::OsRng;
use std::fs;
use std::sync::Arc;
use tempfile::TempDir;

fn test_keypair() -> (SigningKey, ed25519_dalek::VerifyingKey) {
    let sk = SigningKey::generate(&mut OsRng);
    let vk = sk.verifying_key();
    (sk, vk)
}

#[test]
fn test_is_durable() -> Result<()> {
    let (sk, vk) = test_keypair();
    let clock = Arc::new(SystemClock::new());

    let in_mem = RevocationLog::new_in_memory(clock.clone(), Some(sk.clone()), vk);
    assert!(!in_mem.is_durable());

    let temp_dir = TempDir::new().map_err(|e| CryptoError::Crypto(e.to_string()))?;
    let log_path = temp_dir.path().join("rev.log");
    let file_log = RevocationLog::open_or_create(&log_path, clock, Some(sk), vk)?;
    assert!(file_log.is_durable());

    Ok(())
}

#[test]
fn test_error_atomicity_persist_before_mutate() -> Result<()> {
    let temp_dir = TempDir::new().map_err(|e| CryptoError::Crypto(e.to_string()))?;
    let log_path = temp_dir.path().join("rev.log");
    let (sk, vk) = test_keypair();
    let clock = Arc::new(SystemClock::new());

    let log = RevocationLog::open_or_create(&log_path, clock, Some(sk), vk)?;
    let target = RevocationTarget::Group(42);

    // Initial state
    assert_eq!(log.len(), 0);
    assert!(!log.is_revoked(&target));

    // Make parent directory read-only (or remove permissions) to force persistence failure
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(temp_dir.path())
            .map_err(|e| CryptoError::Crypto(e.to_string()))?
            .permissions();
        perms.set_mode(0o555); // Read + Execute, no Write
        fs::set_permissions(temp_dir.path(), perms)
            .map_err(|e| CryptoError::Crypto(e.to_string()))?;
    }

    // Append should fail because writing temp file in parent dir fails
    let append_res = log.append(target.clone());
    assert!(
        append_res.is_err(),
        "append MUST return Err when persistence fails"
    );

    // Restore permissions for clean temp directory cleanup
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(temp_dir.path())
            .map_err(|e| CryptoError::Crypto(e.to_string()))?
            .permissions();
        perms.set_mode(0o755);
        fs::set_permissions(temp_dir.path(), perms)
            .map_err(|e| CryptoError::Crypto(e.to_string()))?;
    }

    // RAM state MUST remain completely untouched!
    assert_eq!(
        log.len(),
        0,
        "RAM entries count MUST remain 0 after persistence failure"
    );
    assert!(
        !log.is_revoked(&target),
        "RAM target MUST NOT be marked as revoked after persistence failure"
    );

    Ok(())
}

#[test]
fn test_incremental_equals_full_verification() -> Result<()> {
    let temp_dir = TempDir::new().map_err(|e| CryptoError::Crypto(e.to_string()))?;
    let log_path = temp_dir.path().join("rev.log");
    let (sk, vk) = test_keypair();
    let clock = Arc::new(SystemClock::new());

    let log = RevocationLog::open_or_create(&log_path, clock.clone(), Some(sk), vk)?;

    for i in 0..10 {
        let target = RevocationTarget::Group(i);
        log.append(target)?;
        // Both incremental and full verification should pass
        assert!(log.verify_integrity().is_ok());
        assert!(log.verify_integrity_full().is_ok());
    }

    // Corrupt an entry in the middle on disk
    let mut bytes = fs::read(&log_path).map_err(|e| CryptoError::Crypto(e.to_string()))?;
    assert!(bytes.len() > 50);
    bytes[40] ^= 0xFF;
    fs::write(&log_path, &bytes).map_err(|e| CryptoError::Crypto(e.to_string()))?;

    // Reopening the tampered log must fail
    let reopened = RevocationLog::open_or_create(&log_path, clock, None, vk);
    assert!(
        matches!(reopened, Err(CryptoError::IntegrityViolation)),
        "Reopening tampered log must return IntegrityViolation"
    );

    Ok(())
}

#[test]
fn test_signed_marker_tampering_and_legacy_upgrade() -> Result<()> {
    let temp_dir = TempDir::new().map_err(|e| CryptoError::Crypto(e.to_string()))?;
    let log_path = temp_dir.path().join("rev.log");
    let m_path = marker_path_for(&log_path);
    let (sk, vk) = test_keypair();
    let clock = Arc::new(SystemClock::new());

    {
        let log = RevocationLog::open_or_create(&log_path, clock.clone(), Some(sk.clone()), vk)?;
        log.append(RevocationTarget::Kek("kek-1".into()))?;
    }

    // Verify marker file format starts with magic prefix b"RVMK"
    let marker_bytes = fs::read(&m_path).map_err(|e| CryptoError::Crypto(e.to_string()))?;
    assert!(
        marker_bytes.starts_with(b"RVMK"),
        "New marker MUST start with magic prefix b\"RVMK\""
    );

    // Corrupt signature bytes in signed marker
    let mut tampered_marker = marker_bytes.clone();
    let last_idx = tampered_marker.len() - 1;
    tampered_marker[last_idx] ^= 0xFF;
    fs::write(&m_path, &tampered_marker).map_err(|e| CryptoError::Crypto(e.to_string()))?;

    // Open MUST fail due to invalid marker signature
    let open_res = RevocationLog::open_or_create(&log_path, clock.clone(), None, vk);
    assert!(
        matches!(open_res, Err(CryptoError::IntegrityViolation)),
        "Opening with tampered marker signature MUST return IntegrityViolation"
    );

    // Now test Legacy Marker upgrade
    // Re-create log with legacy marker
    {
        // Get head_hash from log entry 0
        let log_bytes = fs::read(&log_path).map_err(|e| CryptoError::Crypto(e.to_string()))?;
        let entries: Vec<contextra_crypto::revocation_log::RevocationEntry> =
            bincode::deserialize(&log_bytes).map_err(|e| CryptoError::Crypto(e.to_string()))?;
        let legacy_marker = contextra_crypto::revocation_log::RevocationMarker {
            count: 1,
            head_hash: entries[0].entry_hash,
        };
        let legacy_bytes =
            bincode::serialize(&legacy_marker).map_err(|e| CryptoError::Crypto(e.to_string()))?;
        fs::write(&m_path, &legacy_bytes).map_err(|e| CryptoError::Crypto(e.to_string()))?;
    }

    // Reopen log with legacy marker MUST succeed
    let log_reopened =
        RevocationLog::open_or_create(&log_path, clock.clone(), Some(sk.clone()), vk)?;
    assert_eq!(log_reopened.len(), 1);

    // Next append MUST upgrade marker format to signed marker
    log_reopened.append(RevocationTarget::Kek("kek-2".into()))?;

    let upgraded_marker_bytes =
        fs::read(&m_path).map_err(|e| CryptoError::Crypto(e.to_string()))?;
    assert!(
        upgraded_marker_bytes.starts_with(b"RVMK"),
        "Marker MUST be upgraded to signed format after append"
    );

    Ok(())
}

#[test]
fn test_restart_roundtrip() -> Result<()> {
    let temp_dir = TempDir::new().map_err(|e| CryptoError::Crypto(e.to_string()))?;
    let log_path = temp_dir.path().join("rev.log");
    let (sk, vk) = test_keypair();
    let clock = Arc::new(SystemClock::new());

    let target = RevocationTarget::Group(999);

    {
        let log = RevocationLog::open_or_create(&log_path, clock.clone(), Some(sk), vk)?;
        log.append(target.clone())?;
        assert!(log.is_revoked(&target));
    }

    // Restart process, open existing log
    let reopened = RevocationLog::open_or_create(&log_path, clock, None, vk)?;
    assert_eq!(reopened.len(), 1);
    assert!(reopened.is_revoked(&target));

    Ok(())
}
