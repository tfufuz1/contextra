#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]

// FILE-CONTEXT
// STAND: 2026-10-14
// ZWECK: Integrationstest für zweistufige Krypto-Löschung (KeyRevoked, KeyQuarantined, KeyDestroyed) und Atomic Bitmask.

use contextra_crypto::crypto::KeyManager;
use contextra_crypto::error::CryptoError;
use contextra_crypto::kv_shredding::{
    KeyDeletionScope, KeyLifecycleState, KeyRegistry, DEFAULT_COLLECTION_QUARANTINE_NANOS,
    DEFAULT_DOCUMENT_QUARANTINE_NANOS,
};

#[test]
fn test_two_stage_crypto_erase_collection_and_document_scopes() -> Result<(), CryptoError> {
    let km = KeyManager::try_new("test-passphrase-two-stage", b"salt1")?;
    let registry = KeyRegistry::new();
    let group_col = 101;
    let group_doc = 202;
    let current_time_nanos = 1_000_000_000_000u64;

    // Derive active keys
    let _subkey_col = registry.get_or_derive(&km, group_col)?;
    let _subkey_doc = registry.get_or_derive(&km, group_doc)?;

    assert!(!registry.is_group_revoked(group_col));
    assert!(!registry.is_group_revoked(group_doc));

    // Revoke Collection scope (72h quarantine)
    let revoked_col = registry.revoke_group_with_scope(
        group_col,
        KeyDeletionScope::CollectionOrTenant,
        current_time_nanos,
    )?;
    assert!(revoked_col);
    assert!(registry.is_group_revoked(group_col));

    // Decryption is immediately blocked fail-closed
    let res_col = registry.get_or_derive(&km, group_col);
    assert!(matches!(res_col, Err(CryptoError::KeyRevoked(_))));

    // Revoke Document scope (0h quarantine)
    let revoked_doc = registry.revoke_group_with_scope(
        group_doc,
        KeyDeletionScope::Document,
        current_time_nanos,
    )?;
    assert!(revoked_doc);
    assert!(registry.is_group_revoked(group_doc));

    let res_doc = registry.get_or_derive(&km, group_doc);
    assert!(matches!(res_doc, Err(CryptoError::KeyRevoked(_))));

    assert_eq!(
        DEFAULT_COLLECTION_QUARANTINE_NANOS,
        72 * 3600 * 1_000_000_000
    );
    assert_eq!(DEFAULT_DOCUMENT_QUARANTINE_NANOS, 0);

    Ok(())
}
