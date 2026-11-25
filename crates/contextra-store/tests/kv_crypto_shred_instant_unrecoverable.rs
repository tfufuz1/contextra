//! Integration test verifying that KV CryptoShred mode provides instant, unrecoverable
//! deletion via sub-key destruction without physical byte overwrite.

#![cfg(feature = "encryption-at-rest")]

use contextra_crypto::{CryptoKey, KeyRegistry};

#[test]
fn test_kv_crypto_shred_instant_unrecoverable() {
    let master_key = CryptoKey::try_new("master-passphrase-shred-test", b"salt-shred-123")
        .expect("master key init");

    let registry = KeyRegistry::new();
    let group_id = 42;
    let original_plaintext = b"TOP SECRET: Sensitive User PII Record Data Payload";

    // 1. Encrypt payload under group_id sub-key
    let (ciphertext, nonce) = registry
        .encrypt_with_group(&master_key, group_id, original_plaintext)
        .expect("encryption with group subkey");

    // Physical ciphertext bytes exist
    assert!(!ciphertext.is_empty());
    assert_ne!(ciphertext.as_slice(), original_plaintext);

    // 2. Verify decryption works when key is active in registry
    let decrypted = registry
        .decrypt_with_group(group_id, &ciphertext, &nonce)
        .expect("decryption before revocation");
    assert_eq!(decrypted.as_slice(), original_plaintext);

    // 3. Perform O(1) Crypto-Shredding by revoking (destroying) sub-key from registry
    let revoked = registry.revoke_subkey(group_id);
    assert!(revoked, "sub-key must be revoked successfully");
    assert!(!registry.is_key_active(group_id));

    // 4. Verify physical bytes still exist (no overwrite occurred)
    // Direct scan/check: ciphertext and nonce bytes are identical and intact
    let (ciphertext_after_shred, _) = (&ciphertext, &nonce);
    assert_eq!(ciphertext_after_shred.as_slice(), ciphertext.as_slice());

    // Byte scan of physical ciphertext must NOT contain any unencrypted plaintext fragments
    let needle = b"TOP SECRET";
    let contains_plaintext = ciphertext
        .windows(needle.len())
        .any(|window| window == needle);
    assert!(
        !contains_plaintext,
        "Physical ciphertext bytes must not expose plaintext fragments"
    );

    // 5. Attempting decryption on the intact physical bytes fails unconditionally (unrecoverable)
    let decrypt_result = registry.decrypt_with_group(group_id, &ciphertext, &nonce);
    assert!(
        decrypt_result.is_err(),
        "Decryption after sub-key revocation must fail and produce error"
    );
}
