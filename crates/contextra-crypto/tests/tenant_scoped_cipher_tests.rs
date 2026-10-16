// FILE-CONTEXT
// ZWECK: Targeted security tests for tenant-scoped KeyManager derivation (KeyManager::cipher_for_scoped).
// INVARIANTEN: Multi-tenant cryptographic isolation. Rejects cross-tenant scope mismatches fail-closed.

#![forbid(unsafe_code)]

use contextra_crypto::{CryptoError, KeyManager};
use contextra_types::{TenantId, TenantScoped};

#[test]
fn test_cipher_for_scoped_happy_path() -> Result<(), CryptoError> {
    let km = KeyManager::try_new("master-passphrase-tenant-scoped", b"salt-tenant-123")?;
    let tenant_10 = TenantId::try_new(10).map_err(|e| CryptoError::InvalidInput(e.to_string()))?;

    // Wrap dummy payload in TenantScoped for tenant 10
    let scoped_payload = TenantScoped::new(tenant_10, "tenant-10-secret-data");

    // Derivation with matching expected tenant ID MUST succeed
    let cipher_scoped = km.cipher_for_scoped(scoped_payload, &tenant_10)?;
    let cipher_direct = km.cipher_for(tenant_10)?;

    assert_eq!(
        cipher_scoped.inspect_key_bytes_for_test(),
        cipher_direct.inspect_key_bytes_for_test(),
        "cipher_for_scoped MUST produce identical subkey as cipher_for when tenant match is verified"
    );

    // Verify encryption/decryption roundtrip works with derived cipher
    let data = b"multi-tenant isolated payload";
    let (encrypted, nonce) = cipher_scoped.encrypt_auto_nonce(data)?;
    let decrypted = cipher_scoped.decrypt_auto_nonce(&encrypted, &nonce)?;
    assert_eq!(decrypted, data);

    Ok(())
}

#[test]
fn test_cipher_for_scoped_mismatch_attack_rejected() -> Result<(), CryptoError> {
    let km = KeyManager::try_new("master-passphrase-tenant-scoped", b"salt-tenant-123")?;
    let tenant_a = TenantId::try_new(100).map_err(|e| CryptoError::InvalidInput(e.to_string()))?;
    let tenant_b = TenantId::try_new(200).map_err(|e| CryptoError::InvalidInput(e.to_string()))?;

    // Wrap payload bound to tenant A
    let scoped_a = TenantScoped::new(tenant_a, "tenant-a-confidential-payload");

    // Attempt to derive cipher for tenant B using tenant A's payload MUST fail fail-closed
    let res = km.cipher_for_scoped(scoped_a, &tenant_b);
    assert!(
        matches!(res, Err(CryptoError::InvalidInput(ref msg)) if msg.contains("tenant scope mismatch")),
        "Expected tenant scope mismatch error, got: {:?}",
        res
    );

    Ok(())
}
