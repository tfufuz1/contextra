// FILE-CONTEXT
// ZWECK: Targeted integration test verifying closure and full functionality of the 7 core crypto symbols in contextra-crypto.
// INVARIANTEN: Explicit calls to cipher_for_scoped, derive_deletion_proof_key, derive_kv_key_scoped, derive_segment_key, generate_default, last_seq_no_snapshot, set_last_seq_no.

#![forbid(unsafe_code)]

use contextra_crypto::crypto::KeyManager;
use contextra_crypto::deletion_proof::DeletionScope;
use contextra_crypto::error::Result;
use contextra_crypto::kdf::KdfHeader;
use contextra_crypto::kv_cipher::ModelFingerprint;
use contextra_crypto::wal_crypto::IntegrityVerifier;
use contextra_types::{TenantId, TenantScoped};

#[test]
fn test_closure_symbol_generate_default() -> Result<()> {
    let header = KdfHeader::generate_default()?;
    assert_eq!(header.version, 1);
    assert_eq!(header.salt.len(), 32);

    let (km, derived_header) = KeyManager::try_new_argon2id("closure-passphrase-argon2id")?;
    assert_eq!(derived_header.version, 1);

    let payload = b"kdf default test payload";
    let (ct, nonce) = km.encrypt_auto_nonce(payload)?;
    let decrypted = km.decrypt_auto_nonce(&ct, &nonce)?;
    assert_eq!(decrypted, payload);
    Ok(())
}

#[test]
fn test_closure_symbol_derive_deletion_proof_key() -> Result<()> {
    let km = KeyManager::try_new("closure-passphrase-delproof", b"salt-delproof")?;
    let key1 = km.derive_deletion_proof_key()?;
    let key2 = km.derive_deletion_proof_key()?;
    assert_ne!(key1, [0u8; 32]);
    assert_eq!(key1, key2, "Deletion proof key derivation MUST be deterministic");

    let integrity_key = km.integrity_key()?;
    assert_ne!(key1, integrity_key, "Deletion proof key MUST be distinct from integrity key");

    let proof = km.create_deletion_proof(
        DeletionScope::Tenant { tenant_id: TenantId::try_new(99).unwrap() },
        vec![b"key1".to_vec()],
        contextra_types::TxId(10),
        vec![],
        vec![],
    )?;
    assert_eq!(proof.deleted_after_tx, contextra_types::TxId(10));
    Ok(())
}

#[test]
fn test_closure_symbol_derive_segment_key() -> Result<()> {
    let km = KeyManager::try_new("closure-passphrase-segment", b"salt-segment")?;
    let seg_km1 = km.derive_segment_key("contextra-kv-v1-segment-10-100")?;
    let seg_km2 = km.derive_segment_key("contextra-kv-v1-segment-10-100")?;
    let seg_km3 = km.derive_segment_key("contextra-kv-v1-segment-10-101")?;

    let payload = b"segment payload data";
    let (ct1, nonce1) = seg_km1.encrypt_auto_nonce(payload)?;
    let decrypted = seg_km2.decrypt_auto_nonce(&ct1, &nonce1)?;
    assert_eq!(decrypted, payload);

    assert!(seg_km3.decrypt_auto_nonce(&ct1, &nonce1).is_err());

    let seg_for_id = km.derive_segment_key_for_id(TenantId::try_new(10).unwrap(), 100)?;
    let (ct_id, nonce_id) = seg_for_id.encrypt_auto_nonce(payload)?;
    let decrypted_id = seg_km1.decrypt_auto_nonce(&ct_id, &nonce_id)?;
    assert_eq!(decrypted_id, payload);
    Ok(())
}

#[test]
fn test_closure_symbol_derive_kv_key_scoped() -> Result<()> {
    let km = KeyManager::try_new("closure-passphrase-scoped-kv", b"salt-scoped-kv")?;
    let tenant_a = TenantId::try_new(500).unwrap();
    let tenant_b = TenantId::try_new(600).unwrap();

    let fp = ModelFingerprint {
        hash: [0x55; 32],
        model_id: "closure-model-v1".to_string(),
        quantization: "Q4_K_M".to_string(),
    };

    let scoped_a = TenantScoped::new(tenant_a, &fp);

    let km_scoped_a = km.derive_kv_key_scoped(scoped_a.clone(), &tenant_a)?;
    let km_direct_a = km.derive_kv_key(tenant_a, &fp)?;

    let payload = b"kv scoped isolation payload";
    let (ct, nonce) = km_scoped_a.encrypt_auto_nonce(payload)?;
    let decrypted = km_direct_a.decrypt_auto_nonce(&ct, &nonce)?;
    assert_eq!(decrypted, payload);

    let mismatch_err = km.derive_kv_key_scoped(scoped_a, &tenant_b);
    assert!(mismatch_err.is_err());
    Ok(())
}

#[test]
fn test_closure_symbol_cipher_for_scoped() -> Result<()> {
    let km = KeyManager::try_new("closure-passphrase-scoped-cipher", b"salt-scoped-cipher")?;
    let tenant_a = TenantId::try_new(700).unwrap();
    let tenant_b = TenantId::try_new(800).unwrap();

    let scoped_key = TenantScoped::new(tenant_a, ());

    let cipher_scoped_a = km.cipher_for_scoped(scoped_key.clone(), &tenant_a)?;
    let cipher_direct_a = km.cipher_for(tenant_a)?;

    let payload = b"tenant cipher isolation payload";
    let (ct, nonce) = cipher_scoped_a.encrypt_auto_nonce(payload)?;
    let decrypted = cipher_direct_a.decrypt_auto_nonce(&ct, &nonce)?;
    assert_eq!(decrypted, payload);

    let mismatch_err = km.cipher_for_scoped(scoped_key, &tenant_b);
    assert!(mismatch_err.is_err());
    Ok(())
}

#[test]
fn test_closure_symbol_wal_verifier_last_seq_no_snapshot_and_set() {
    let key = b"integrity-key-32-bytes-wal-cl--";
    let mut verifier_source = IntegrityVerifier::new(key);
    assert_eq!(verifier_source.last_seq_no_snapshot(), None);

    verifier_source.set_last_seq_no(Some(42));
    assert_eq!(verifier_source.last_seq_no_snapshot(), Some(42));

    let mut verifier_target = IntegrityVerifier::new(key);
    verifier_target.handoff_from(&verifier_source);
    assert_eq!(verifier_target.last_seq_no_snapshot(), Some(42));
}
