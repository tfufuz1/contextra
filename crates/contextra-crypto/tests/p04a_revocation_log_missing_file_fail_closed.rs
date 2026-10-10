// FILE-CONTEXT
// ZWECK: Integrationstest für P04 / F-3 (Missing RevocationLog File Fail-Closed & Truncation Protection).
// INVARIANTEN: Reopen eines gelöschten oder gekürzten RevocationLogs MUSS mit CryptoError::IntegrityViolation fehlschlagen.

#![forbid(unsafe_code)]

use contextra_crypto::revocation_log::{RevocationLog, RevocationTarget};
use contextra_crypto::{CryptoError, KeyManager, KeyRegistry, Result};
use contextra_ports::SystemClock;
use ed25519_dalek::SigningKey;
use rand::rngs::OsRng;
use std::sync::Arc;
use tempfile::NamedTempFile;

fn test_keypair() -> (SigningKey, ed25519_dalek::VerifyingKey) {
    let sk = SigningKey::generate(&mut OsRng);
    let vk = sk.verifying_key();
    (sk, vk)
}

#[test]
fn test_missing_revocation_log_fails_closed_on_reopen() -> Result<()> {
    let tmp_file = NamedTempFile::new().map_err(|e| CryptoError::Crypto(e.to_string()))?;
    let log_path = tmp_file.path().to_path_buf();
    let (sk, vk) = test_keypair();
    let clock = Arc::new(SystemClock::new());

    let km = KeyManager::try_new("test-passphrase-p04a", b"salt1")?;
    let group_id = 999;

    {
        let log = Arc::new(RevocationLog::create_new(
            &log_path,
            clock.clone(),
            Some(sk.clone()),
            vk,
        )?);
        let registry = KeyRegistry::new_for_test(log.clone());

        let (ct, nonce) = registry.encrypt_with_group(&km, group_id, b"sensitive data")?;
        let decrypted = registry.decrypt_with_group(group_id, &ct, &nonce)?;
        assert_eq!(decrypted, b"sensitive data");

        // Revoke group
        registry.revoke_group(group_id)?;
        assert!(registry.is_group_revoked(group_id));
    }

    // Now delete the revocation.log file directly from disk, leaving the .initialized marker file behind
    std::fs::remove_file(&log_path).map_err(|e| CryptoError::Crypto(e.to_string()))?;

    // Attempting to reopen the log MUST fail with CryptoError::IntegrityViolation
    let reopen_res = RevocationLog::open_or_create(&log_path, clock, None, vk);
    assert!(
        matches!(reopen_res, Err(CryptoError::IntegrityViolation)),
        "Reopening when log file is missing MUST return CryptoError::IntegrityViolation"
    );

    Ok(())
}

#[test]
fn test_truncation_of_revocation_log_rejected() -> Result<()> {
    let tmp_file = NamedTempFile::new().map_err(|e| CryptoError::Crypto(e.to_string()))?;
    let log_path = tmp_file.path().to_path_buf();
    let (sk, vk) = test_keypair();
    let clock = Arc::new(SystemClock::new());

    {
        let log = RevocationLog::create_new(&log_path, clock.clone(), Some(sk.clone()), vk)?;
        log.append(RevocationTarget::Group(1))?;
        log.append(RevocationTarget::Group(2))?;
        log.append(RevocationTarget::Group(3))?;
        assert_eq!(log.len(), 3);
    }

    // Now truncate the log file down to 1 entry by rewriting serialized empty/partial entries
    // (Simulate tail truncation by serializing only the first entry)
    let log_bytes = std::fs::read(&log_path).map_err(|e| CryptoError::Crypto(e.to_string()))?;
    let mut parsed: Vec<contextra_crypto::RevocationEntry> =
        bincode::deserialize(&log_bytes).map_err(|_| CryptoError::IntegrityViolation)?;
    assert_eq!(parsed.len(), 3);

    parsed.truncate(1); // Truncate from 3 entries to 1
    let truncated_bytes =
        bincode::serialize(&parsed).map_err(|e| CryptoError::Crypto(e.to_string()))?;
    std::fs::write(&log_path, &truncated_bytes).map_err(|e| CryptoError::Crypto(e.to_string()))?;

    // Reopening the truncated log MUST be rejected because marker expects 3 entries
    let reopen_res = RevocationLog::open_or_create(&log_path, clock, None, vk);
    assert!(
        matches!(reopen_res, Err(CryptoError::IntegrityViolation)),
        "Reopening truncated log file MUST return CryptoError::IntegrityViolation"
    );

    Ok(())
}

#[test]
fn test_fresh_initialization_succeeds() -> Result<()> {
    let tmp_file = NamedTempFile::new().map_err(|e| CryptoError::Crypto(e.to_string()))?;
    let log_path = tmp_file.path().to_path_buf();
    // Remove temp file so path is clean
    std::fs::remove_file(&log_path).ok();

    let (sk, vk) = test_keypair();
    let clock = Arc::new(SystemClock::new());

    let log = RevocationLog::open_or_create(&log_path, clock, Some(sk), vk)?;
    assert_eq!(log.len(), 0);

    Ok(())
}
