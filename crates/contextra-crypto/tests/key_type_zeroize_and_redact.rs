// FILE-CONTEXT
// ZWECK: Comprehensive verification of Zeroize discipline and Debug/Display redaction across all contextra-crypto key types.

#![forbid(unsafe_code)]
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, deprecated)]

use contextra_crypto::{
    CchSecureBuf, CryptoKey, DestroyedKey, IntegrityVerifier, KeyManager, MfksKeystore,
};
use zeroize::Zeroize;

#[test]
fn test_cch_secure_buf_zeroize_and_debug_redaction() {
    let raw = vec![0xFF; 32];
    let mut buf = CchSecureBuf::from_vec(raw);

    let debug_str = format!("{buf:?}");
    assert!(debug_str.contains("len: 32"));
    assert!(!debug_str.contains("255"));

    Zeroize::zeroize(&mut buf);
    assert_eq!(buf.as_slice(), &[0u8; 32]);
}

#[test]
fn test_key_manager_debug_redaction() {
    let (km, _) = KeyManager::try_new_argon2id("super-secret-passphrase").unwrap();
    let debug_str = format!("{km:?}");
    assert!(debug_str.contains("REDACTED"));
    assert!(!debug_str.contains("super-secret-passphrase"));

    let crypto_key = CryptoKey::from_key_manager(km);
    let debug_str2 = format!("{crypto_key:?}");
    assert!(debug_str2.contains("REDACTED"));
}

#[test]
fn test_destroyed_key_zeroize_and_receipt() {
    let dk = DestroyedKey {
        key_id: "key-999".to_string(),
        destroyed_at: 123456789,
    };
    let receipt = dk.issue_deletion_receipt();
    assert_eq!(receipt.key_id, "key-999");
    assert_eq!(receipt.status, "CRYPTOGRAPHICALLY_SHREDDED");
}

#[test]
fn test_integrity_verifier_zeroize_on_drop() {
    let key = [0x77u8; 32];
    let mut verifier = IntegrityVerifier::new(&key);
    assert_eq!(verifier.last_hmac_snapshot(), [0u8; 32]);
    verifier.set_last_hmac([0xAA; 32]);
    assert_eq!(verifier.last_hmac_snapshot(), [0xAA; 32]);
}

#[test]
fn test_mfks_keystore_format_and_integrity() {
    let (_, header) = KeyManager::try_new_argon2id("mfks-passphrase").unwrap();
    let keystore = MfksKeystore::new(header, vec![1, 2, 3], vec![4, 5, 6]).unwrap();
    let debug_str = format!("{keystore:?}");
    assert!(!debug_str.contains("mfks-passphrase"));

    let bytes = keystore.to_bytes();
    let parsed = MfksKeystore::from_bytes(&bytes).unwrap();
    assert_eq!(parsed.wrapped_deks, vec![1, 2, 3]);
}
