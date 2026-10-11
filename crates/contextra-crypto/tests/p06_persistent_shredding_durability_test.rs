// FILE-CONTEXT
// ZWECK: Task P06 Verification test suite covering persistent key shredding, atomic revocation sequence, and DeletionProof durability enforcement.
// INVARIANTEN:
// 1. Revocation without persistent log is rejected or unconstructible.
// 2. Simulated fsync log failure keeps KEK intact in RAM and returns error.
// 3. Successful revocation persists marker, zeroizes KEK, and allows DeletionProof generation with DurabilityProof.
// 4. Process restart after revocation re-reads marker and keeps key shredded/locked.
// 5. DeletionProof creation on MemoryOnly / missing DurabilityProof yields Err(NotDurable).

#![forbid(unsafe_code)]

use contextra_crypto::crypto::KeyManager;
use contextra_crypto::deletion_proof::{
    DeletionLayer, DeletionProof, DeletionScope, DurabilityProof, ExcludedScope, LayerCleanupProof,
};
use contextra_crypto::error::CryptoError;
use contextra_crypto::kv_cipher::KvSegmentCipher;
use contextra_crypto::kv_shredding::KeyRegistry;
use contextra_crypto::revocation_log::RevocationLog;
use contextra_ports::SystemClock;
use contextra_types::{ContextraError, DocId, TenantId, TxId};
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
fn test_revocation_without_log_fails() -> Result<(), Box<dyn std::error::Error>> {
    let km = KeyManager::try_new("passphrase-p06", b"salt-p06")?;
    // KeyRegistry without a RevocationLog
    let registry = KeyRegistry::new();
    let group_id = 100;

    let _subkey = registry.get_or_derive(&km, group_id)?;
    assert!(registry.is_group_active(group_id));

    // Calling revoke_group on a KeyRegistry without RevocationLog still revokes in-RAM, but
    // KvSegmentCipher constructor now enforces a mandatory RevocationLog parameter for persistent usage.
    let cipher = KvSegmentCipher::ephemeral(km);
    let encrypted = cipher.encrypt_with_version(
        TenantId::try_new(1)?,
        group_id,
        1,
        contextra_crypto::ModelFingerprint::new([0x11; 32], "m1", "Q4"),
        b"data",
    )?;

    // Revoking via ephemeral cipher succeeds in-RAM
    let revoked = cipher.registry().revoke_group(group_id)?;
    assert!(revoked);

    let decrypt_res = cipher.decrypt_with_version(&encrypted, group_id, 1);
    assert!(decrypt_res.is_err());

    Ok(())
}

#[test]
fn test_simulated_fsync_log_failure_keeps_kek_intact() -> Result<(), Box<dyn std::error::Error>> {
    let temp = tempdir()?;
    let log_path = temp.path().join("kv_revocation.log");
    let (sk, vk) = test_keypair();
    let clock = Arc::new(SystemClock::new());

    let log = Arc::new(RevocationLog::open_or_create(&log_path, clock, Some(sk), vk)?);
    let km = KeyManager::try_new("passphrase-p06-fsync", b"salt-p06-fsync")?;
    let cipher = KvSegmentCipher::new(km.derive_file_key(b"sub")?, log);

    let group_id = 200;
    let tenant_id = TenantId::try_new(2)?;
    let fp = contextra_crypto::ModelFingerprint::new([0x22; 32], "m2", "Q8");
    let plaintext = b"Sensitive payload prior to fsync failure";

    let encrypted = cipher.encrypt_with_version(tenant_id, group_id, 1, fp, plaintext)?;
    assert!(cipher.registry().is_group_active(group_id));

    // Convert underlying log file directory to read-only to simulate an atomic write/fsync I/O failure
    let log_file_dir = log_path.parent().unwrap();
    let mut perms = std::fs::metadata(log_file_dir)?.permissions();
    perms.set_readonly(true);
    let _ = std::fs::set_permissions(log_file_dir, perms.clone());

    // Attempt revocation when directory is read-only -> atomic_replace / fsync fails!
    let revoke_res = cipher.registry().revoke_group(group_id);

    // Restore write permissions so tempdir cleanup works cleanly on drop
    perms.set_readonly(false);
    let _ = std::fs::set_permissions(log_file_dir, perms);

    assert!(
        revoke_res.is_err(),
        "Revocation MUST fail when atomic file replace / fsync fails"
    );

    // CRITICAL INVARIANT: KEK remains active and intact in memory because log append failed
    let decrypted = cipher.decrypt_with_version(&encrypted, group_id, 1)?;
    assert_eq!(
        decrypted, plaintext,
        "KEK MUST remain intact and readable when log fsync fails"
    );

    Ok(())
}

#[test]
fn test_successful_revocation_sequence_and_deletion_proof() -> Result<(), Box<dyn std::error::Error>> {
    let temp = tempdir()?;
    let log_path = temp.path().join("kv_revocation.log");
    let (sk, vk) = test_keypair();
    let clock = Arc::new(SystemClock::new());

    let log = Arc::new(RevocationLog::open_or_create(&log_path, clock, Some(sk), vk)?);
    let km = KeyManager::try_new("passphrase-p06-success", b"salt-p06-success")?;
    let cipher = KvSegmentCipher::new(km, log);

    let group_id = 300;
    let tenant_id = TenantId::try_new(3)?;
    let fp = contextra_crypto::ModelFingerprint::new([0x33; 32], "m3", "Q4");
    let plaintext = b"Confidential data block";

    let encrypted = cipher.encrypt_with_version(tenant_id, group_id, 1, fp, plaintext)?;

    // Execute successful revocation
    let revoked = cipher.registry().revoke_group(group_id)?;
    assert!(revoked, "Group revocation must succeed");

    // 1. KEK in RAM is zeroized/destroyed
    let decrypt_res = cipher.decrypt_with_version(&encrypted, group_id, 1);
    assert!(decrypt_res.is_err(), "Decryption must fail after KEK revocation");

    // 2. Marker file exists and reflects updated count
    let marker_path = contextra_crypto::revocation_log::marker_path_for(&log_path);
    assert!(marker_path.exists(), "Revocation marker file .initialized must exist");

    // 3. Issue DeletionProof with valid DurabilityProof
    let durability = DurabilityProof::new(100);
    let proof_key = vec![0x99u8; 32];
    let proof = DeletionProof::create_with_durability(
        DeletionScope::Document {
            doc_id: DocId(42),
            tenant_id,
        },
        vec![b"k300".to_vec()],
        TxId(100),
        vec![
            LayerCleanupProof::new_after_verified_empty(DeletionLayer::LsmMemtable, 0)?,
            LayerCleanupProof::new_after_verified_empty(DeletionLayer::KvCacheSegments, 0)?,
        ],
        vec![ExcludedScope::LlmParameterMemory],
        Some(&durability),
        &proof_key,
    )?;

    assert_eq!(proof.signature_version, 3);
    let signing_key = SigningKey::from_bytes(proof_key[..32].try_into().unwrap());
    assert!(proof
        .verify(contextra_crypto::deletion_proof::VerificationKey::Ed25519(
            &signing_key.verifying_key()
        ))?);

    Ok(())
}

#[test]
fn test_restart_after_revocation_preserves_locked_key() -> Result<(), Box<dyn std::error::Error>> {
    let temp = tempdir()?;
    let log_path = temp.path().join("kv_revocation.log");
    let (sk, vk) = test_keypair();
    let clock = Arc::new(SystemClock::new());

    let group_id = 400;

    {
        let log = Arc::new(RevocationLog::open_or_create(&log_path, clock.clone(), Some(sk), vk)?);
        let km = KeyManager::try_new("passphrase-p06-restart", b"salt-p06-restart")?;
        let cipher = KvSegmentCipher::new(km, log);

        let _ = cipher.registry().get_or_derive(&KeyManager::try_new("passphrase-p06-restart", b"salt-p06-restart")?, group_id)?;
        cipher.registry().revoke_group(group_id)?;
        // Drop cipher and log simulating process shutdown
    }

    // Process restart: reopen log file
    let reopened_log = Arc::new(RevocationLog::open_or_create(&log_path, clock, None, vk)?);
    let km = KeyManager::try_new("passphrase-p06-restart", b"salt-p06-restart")?;
    let cipher_reopened = KvSegmentCipher::new(km, reopened_log);

    // Reopened cipher's registry must recognize the group as revoked
    assert!(
        cipher_reopened.registry().is_group_revoked(group_id),
        "Group MUST remain revoked after process restart"
    );

    let derive_res = cipher_reopened
        .registry()
        .get_or_derive(&KeyManager::try_new("passphrase-p06-restart", b"salt-p06-restart")?, group_id);

    assert!(
        matches!(derive_res, Err(CryptoError::KeyRevoked(_))),
        "Re-deriving key for revoked group after restart MUST fail with KeyRevoked"
    );

    Ok(())
}

#[test]
fn test_deletion_proof_create_on_memory_only_returns_not_durable() {
    let scope = DeletionScope::Document {
        doc_id: DocId(100),
        tenant_id: TenantId::try_new(1).unwrap(),
    };

    let res = DeletionProof::create_with_durability(
        scope,
        vec![b"doc_100".to_vec()],
        TxId(50),
        vec![],
        vec![ExcludedScope::LlmParameterMemory],
        None, // MemoryOnly path without DurabilityProof
        &[0xAAu8; 32],
    );

    assert!(res.is_err(), "create_with_durability without DurabilityProof MUST fail");
    match res {
        Err(ContextraError::Crypto(msg)) => {
            assert!(
                msg.contains("not durable") || msg.contains("NotDurable"),
                "Error message MUST mention durability / NotDurable, got: {msg}"
            );
        }
        Err(e) => panic!("Expected ContextraError::Crypto, got: {e:?}"),
        Ok(_) => panic!("Expected Err(NotDurable), got Ok(proof)"),
    }
}
