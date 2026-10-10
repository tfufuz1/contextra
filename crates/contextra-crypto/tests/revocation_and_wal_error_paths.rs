// FILE-CONTEXT
// ZWECK: Targeted security error path tests for RevocationLog, IntegrityVerifier, and KeyRegistry.
// INVARIANTEN: Unsigned appends fail; tampered signatures reject; invalid op_type (>2) fail; double revocations are idempotent.

#![forbid(unsafe_code)]

use contextra_crypto::wal_crypto::{IntegrityVerifier, WalEntrySnapshot};
use contextra_crypto::{CryptoError, KeyManager, KeyRegistry, RevocationLog, RevocationTarget};
use contextra_ports::SystemClock;
use ed25519_dalek::SigningKey;
use rand::rngs::OsRng;
use std::sync::Arc;
use tempfile::NamedTempFile;

#[test]
fn test_revocation_log_append_without_signing_key_fails() -> Result<(), CryptoError> {
    let sk = SigningKey::generate(&mut OsRng);
    let vk = sk.verifying_key();
    let clock = Arc::new(SystemClock::new());

    // Create log with signing_key = None
    let log = RevocationLog::new_in_memory(clock, None, vk);

    // Attempting to append without a configured signing key MUST fail
    let res = log.append(RevocationTarget::Group(12345));
    assert!(
        matches!(res, Err(CryptoError::Crypto(ref msg)) if msg.contains("Signing key not configured")),
        "Append without signing key MUST fail with Crypto error, got: {:?}",
        res
    );

    Ok(())
}

#[test]
fn test_revocation_log_open_with_wrong_verifying_key_fails() -> Result<(), CryptoError> {
    let tmp_file = NamedTempFile::new().map_err(|e| CryptoError::Crypto(e.to_string()))?;
    let log_path = tmp_file.path().to_path_buf();

    let sk_a = SigningKey::generate(&mut OsRng);
    let vk_a = sk_a.verifying_key();

    let sk_b = SigningKey::generate(&mut OsRng);
    let vk_b = sk_b.verifying_key();

    let clock = Arc::new(SystemClock::new());

    // Create and sign entries with Key A
    {
        let log_a = RevocationLog::open_or_create(&log_path, clock.clone(), Some(sk_a), vk_a)?;
        log_a.append(RevocationTarget::Group(1))?;
        log_a.append(RevocationTarget::Kek("kek-key-a".into()))?;
    }

    // Attempting to open log using unauthorized Key B MUST fail with IntegrityViolation
    let open_res = RevocationLog::open_or_create(&log_path, clock, None, vk_b);
    assert!(
        matches!(open_res, Err(CryptoError::IntegrityViolation)),
        "Opening log signed with Key A using Key B MUST return IntegrityViolation, got: {:?}",
        open_res
    );

    Ok(())
}

#[test]
fn test_integrity_verifier_unsupported_op_type_v3_and_v2() -> Result<(), CryptoError> {
    let key = b"integrity-key-32-bytes-op-check";
    let mut verifier = IntegrityVerifier::new(key);

    let invalid_entry_v3 = WalEntrySnapshot {
        tx_id: 1,
        seq_no: 1,
        op_type: 3, // Invalid op_type (supported: 0, 1, 2)
        key: b"invalid_key".to_vec(),
        value: b"invalid_val".to_vec(),
        checksum: [0u8; 32],
        prev_hmac: [0u8; 32],
    };

    // V3 verifier MUST reject op_type = 3 with WalCorruption
    let res_v3 = verifier.verify_and_update_v3(&invalid_entry_v3, 100);
    assert!(
        matches!(res_v3, Err(CryptoError::WalCorruption { offset: 100, ref reason, .. }) if reason.contains("Unsupported op_type 3")),
        "V3 verifier MUST reject invalid op_type with WalCorruption error, got: {:?}",
        res_v3
    );

    #[allow(deprecated)]
    {
        let invalid_entry_v2 = WalEntrySnapshot {
            tx_id: 2,
            seq_no: 1,
            op_type: 255, // Invalid op_type
            key: b"invalid_key".to_vec(),
            value: b"invalid_val".to_vec(),
            checksum: [0u8; 32],
            prev_hmac: [0u8; 32],
        };

        // V2 verifier MUST reject op_type = 255 with WalCorruption
        let res_v2 = verifier.verify_and_update_v2(&invalid_entry_v2, 200);
        assert!(
            matches!(res_v2, Err(CryptoError::WalCorruption { offset: 200, ref reason, .. }) if reason.contains("Unsupported op_type 255")),
            "V2 verifier MUST reject invalid op_type with WalCorruption error, got: {:?}",
            res_v2
        );
    }

    Ok(())
}

#[test]
fn test_key_registry_double_revocation_idempotency() -> Result<(), CryptoError> {
    let clock = Arc::new(contextra_ports::SystemClock::new());
    let sk = ed25519_dalek::SigningKey::generate(&mut rand::rngs::OsRng);
    let vk = sk.verifying_key();
    let registry = KeyRegistry::new_in_memory(clock, Some(sk), vk);
    let km = KeyManager::try_new("passphrase-double-revoke", b"salt-double-revoke")?;

    let group_id = 99911;
    let record_id = 4422;

    // Encrypt record to initialize group and record in registry
    let payload = registry.encrypt_record(&km, group_id, record_id, b"sensitive payload")?;

    // First record revocation MUST return true
    let first_rec_revoke = registry.revoke_record(group_id, record_id)?;
    assert!(first_rec_revoke, "First record revocation must return true");

    // Second record revocation MUST return false (idempotent, already revoked)
    let second_rec_revoke = registry.revoke_record(group_id, record_id)?;
    assert!(
        !second_rec_revoke,
        "Second record revocation on already revoked record MUST return false"
    );

    // First group revocation MUST return true
    let first_grp_revoke = registry.revoke_group(group_id)?;
    assert!(first_grp_revoke, "First group revocation must return true");

    // Second group revocation MUST return false (already revoked)
    let second_grp_revoke = registry.revoke_group(group_id)?;
    assert!(
        !second_grp_revoke,
        "Second group revocation on already revoked group MUST return false"
    );

    // Revoking non-existent or already revoked record after group revocation MUST return false
    let rec_after_grp_revoke = registry.revoke_record(group_id, record_id)?;
    assert!(
        !rec_after_grp_revoke,
        "Revoking record in revoked group MUST return false"
    );

    // Verify record payload decryption fails
    assert!(registry.decrypt_record(&km, &payload).is_err());

    Ok(())
}
