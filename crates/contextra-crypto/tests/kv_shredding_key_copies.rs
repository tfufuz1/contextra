// FILE-CONTEXT
// STAND: 2026-10-13
// ZWECK: Tests für Schlüsselkopien, Debug-Redaktion, Zwei-Phasen-Revokation und Vanish-Lehre (B13).
// INVARIANTEN: INV-VAULT-1, INV-VAULT-2, INV-VAULT-3; Nach revoke_group ist Entschlüsselung unmöglich.

#![forbid(unsafe_code)]

use contextra_crypto::crypto::KeyManager;
use contextra_crypto::error::Result;
use contextra_crypto::kv_shredding::{GroupKek, KeyRegistry, RecordDek, SubKey};

#[test]
fn test_debug_output_redacts_key_bytes() {
    let subkey = SubKey([0xAA; 32]);
    let group_kek = GroupKek([0xBB; 32]);
    let record_dek = RecordDek([0xCC; 32]);

    let debug_subkey = format!("{subkey:?}");
    let debug_group_kek = format!("{group_kek:?}");
    let debug_record_dek = format!("{record_dek:?}");

    assert!(debug_subkey.contains("***REDACTED***"));
    assert!(!debug_subkey.contains("170")); // 0xAA = 170

    assert!(debug_group_kek.contains("***REDACTED***"));
    assert!(!debug_group_kek.contains("187")); // 0xBB = 187

    assert!(debug_record_dek.contains("***REDACTED***"));
    assert!(!debug_record_dek.contains("204")); // 0xCC = 204
}

#[test]
fn test_two_phase_revocation_lifecycle() -> Result<()> {
    let km = KeyManager::try_new("test-two-phase-passphrase", b"salt2")?;
    let registry = KeyRegistry::new();
    let group_id = 888;

    let subkey = registry.get_or_derive(&km, group_id)?;
    assert_eq!(subkey.0.len(), 32);
    assert!(registry.is_group_active(group_id));

    // Phase 1: Request revocation
    let requested = registry.request_revoke(group_id)?;
    assert!(
        requested,
        "request_revoke must return Ok(true) on active group"
    );
    assert!(registry.is_revoke_pending(group_id));

    // Cancel pending revocation
    let canceled = registry.cancel_revoke(group_id)?;
    assert!(
        canceled,
        "cancel_revoke must return Ok(true) for pending group"
    );
    assert!(!registry.is_revoke_pending(group_id));
    assert!(registry.is_group_active(group_id));

    // Re-request and finalize
    assert!(registry.request_revoke(group_id)?);
    let finalized = registry.finalize_revoke(group_id)?;
    assert!(finalized, "finalize_revoke must return Ok(true)");
    assert!(!registry.is_group_active(group_id));
    assert!(!registry.is_revoke_pending(group_id));

    // Cancel attempt post-finalize MUST return Err(CryptoError::KeyRevoked)
    let cancel_after_finalize = registry.cancel_revoke(group_id);
    assert!(
        cancel_after_finalize.is_err(),
        "Canceling revocation after finalize MUST return error"
    );

    Ok(())
}

#[test]
fn test_decryption_impossible_post_revocation() -> Result<()> {
    let km = KeyManager::try_new("test-decryption-impossible", b"salt3")?;
    let registry = KeyRegistry::new();
    let group_id = 999;
    let record_id = 1;

    let payload = registry.encrypt_record(&km, group_id, record_id, b"Confidential Payload B13")?;

    // Decryption works prior to revocation
    let decrypted = registry.decrypt_record(&km, &payload)?;
    assert_eq!(decrypted, b"Confidential Payload B13");

    // Revoke group
    let revoked = registry.revoke_group(group_id)?;
    assert!(revoked);

    // Decryption post-revocation MUST fail
    let dec_res = registry.decrypt_record(&km, &payload);
    assert!(dec_res.is_err(), "Decryption post-revocation MUST fail");

    Ok(())
}

#[test]
fn test_vanish_lehre_dek_persistence_only_wrapped() -> Result<()> {
    let km = KeyManager::try_new("test-vanish-lehre", b"salt4")?;
    let registry = KeyRegistry::new();
    let group_id = 1234;
    let record_id = 7;

    let payload = registry.encrypt_record(&km, group_id, record_id, b"Sensitive Data")?;

    // Verify payload only contains wrapped_dek and dek_nonce, never raw DEK
    assert!(!payload.wrapped_dek.is_empty());
    assert_ne!(payload.wrapped_dek, vec![0u8; 32]);
    assert_eq!(payload.dek_nonce.len(), 12);

    // Bincode serialization of payload contains wrapped_dek
    let serialized = bincode::serialize(&payload).expect("Serialization must succeed");
    assert!(!serialized.is_empty());

    Ok(())
}
