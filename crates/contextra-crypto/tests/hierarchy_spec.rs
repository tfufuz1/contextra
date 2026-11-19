//! Spec-Verifikation `contextra-crypto` v2.0 (§3.2 Argon2id, §3.3 HKDF-Hierarchie, §3.4 AEAD).
//!
//! Known-Answer-Vektoren wurden unabhängig berechnet: HKDF-SHA-256 nach RFC 5869 in Python
//! (`hmac`/`hashlib`) und Argon2id über `argon2-cffi` (m=256 MiB, t=4, p=1, Version 0x13).

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]

use contextra_crypto::{
    build_aad, decrypt_aead, derive_collection_kek, derive_document_dek, derive_master_key,
    derive_subject_doc_dek, derive_subject_key, derive_tenant_kek, encrypt_aead, CryptoError,
    MasterKey, SecretKey32,
};
use contextra_ids::{CollectionId, DocId, SubjectRef, TenantId};

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// Test-MasterKey mit bekannten Bytes 0x00..0x1f (nur für KAT der HKDF-Kette).
fn known_master() -> MasterKey {
    let bytes: [u8; 32] = core::array::from_fn(|i| i as u8);
    MasterKey::from_secret(SecretKey32::from_bytes(bytes))
}

// ---------------------------------------------------------------- Argon2id (§3.2)

#[test]
fn argon2id_master_key_matches_reference() {
    let mk = derive_master_key(b"correct horse", &[1u8; 32]).expect("derive");
    assert_eq!(
        hex(mk.as_bytes()),
        "09c54e1107f68a7568ad2ca3290c09390861f95bdb17a6438aa2f5e2caf16d03"
    );
}

#[test]
fn argon2id_rejects_empty_passphrase_fail_closed() {
    let err = derive_master_key(b"", &[1u8; 32]).err().expect("must fail");
    assert!(matches!(err, CryptoError::EmptyPassphrase));
}

// ---------------------------------------------------------------- HKDF-Hierarchie (§3.3)

#[test]
fn hkdf_hierarchy_matches_independent_reference() {
    let master = known_master();
    let tenant = derive_tenant_kek(&master, TenantId(42)).unwrap();
    assert_eq!(
        hex(tenant.as_bytes()),
        "882a42df80cedb3adfda720b7d20651036d0bea349a3e579bfc3d085a559ddc1"
    );

    let coll = derive_collection_kek(&tenant, CollectionId(7), 1).unwrap();
    assert_eq!(
        hex(coll.as_bytes()),
        "c4da3eb26ac38f1b42116aa3b4e85defd39963ce859a51681e61c074eaff7eca"
    );

    let doc = derive_document_dek(&coll, DocId(99), 3).unwrap();
    assert_eq!(
        hex(doc.as_bytes()),
        "496e33ffbecded3488ef7fb32b6f439118e3f1f6bfb713b9473fa21e1a55dc74"
    );

    let subject = derive_subject_key(&coll, &SubjectRef([0xAB; 16])).unwrap();
    assert_eq!(
        hex(subject.as_bytes()),
        "756b058eb6d954b35d3cfe60bb989f94f4fbf1c3e99a64c9ef780617164f484b"
    );

    let subject_doc = derive_subject_doc_dek(&subject, DocId(99), 3).unwrap();
    assert_eq!(
        hex(subject_doc.as_bytes()),
        "0480fe6473ed84eb5b7216d146309d2b2c25a0dd8edfc5efd5c851dfb5036245"
    );
}

#[test]
fn hkdf_domain_separation_and_context_binding() {
    let master = known_master();
    let t1 = derive_tenant_kek(&master, TenantId(1)).unwrap();
    let t2 = derive_tenant_kek(&master, TenantId(2)).unwrap();
    assert_ne!(t1.as_bytes(), t2.as_bytes(), "tenant binding");

    let c_e1 = derive_collection_kek(&t1, CollectionId(5), 1).unwrap();
    let c_e2 = derive_collection_kek(&t1, CollectionId(5), 2).unwrap();
    assert_ne!(
        c_e1.as_bytes(),
        c_e2.as_bytes(),
        "epoch rotation changes key"
    );

    let c_other_tenant = derive_collection_kek(&t2, CollectionId(5), 1).unwrap();
    assert_ne!(
        c_e1.as_bytes(),
        c_other_tenant.as_bytes(),
        "tenant chain binding"
    );

    // Klasse K und Klasse P dürfen für denselben Doc/Version nie denselben Schlüssel ergeben.
    let dek = derive_document_dek(&c_e1, DocId(9), 1).unwrap();
    let sk = derive_subject_key(&c_e1, &SubjectRef([9; 16])).unwrap();
    let sdek = derive_subject_doc_dek(&sk, DocId(9), 1).unwrap();
    assert_ne!(dek.as_bytes(), sdek.as_bytes(), "K/P domain separation");

    // Version ist Teil des Kontexts.
    let dek_v2 = derive_document_dek(&c_e1, DocId(9), 2).unwrap();
    assert_ne!(dek.as_bytes(), dek_v2.as_bytes(), "version binding");
}

// ---------------------------------------------------------------- AAD (§3.4)

#[test]
fn build_aad_layout_is_little_endian_tenant_collection_doc_version() {
    let aad = build_aad(
        TenantId(0x0102_0304_0506_0708),
        CollectionId(0x1112_1314_1516_1718),
        DocId(0x2122_2324_2526_2728),
        0x3132_3334_3536_3738,
    );
    assert_eq!(&aad[0..8], &0x0102_0304_0506_0708u64.to_le_bytes());
    assert_eq!(&aad[8..16], &0x1112_1314_1516_1718u64.to_le_bytes());
    assert_eq!(&aad[16..24], &0x2122_2324_2526_2728u64.to_le_bytes());
    assert_eq!(&aad[24..32], &0x3132_3334_3536_3738u64.to_le_bytes());
}

// ---------------------------------------------------------------- AEAD (§3.4)

fn test_key() -> SecretKey32 {
    SecretKey32::from_bytes([7u8; 32])
}

#[test]
fn aead_roundtrip_and_wire_format() {
    let key = test_key();
    let aad = build_aad(TenantId(1), CollectionId(2), DocId(3), 4);
    let blob = encrypt_aead(&key, &aad, b"hello").unwrap();
    assert_eq!(blob.len(), 12 + 5 + 16, "nonce || ciphertext || tag");
    let pt = decrypt_aead(&key, &aad, &blob).unwrap();
    assert_eq!(pt, b"hello");
}

#[test]
fn aead_nonce_is_fresh_per_encryption() {
    let key = test_key();
    let aad = build_aad(TenantId(1), CollectionId(2), DocId(3), 4);
    let a = encrypt_aead(&key, &aad, b"same").unwrap();
    let b = encrypt_aead(&key, &aad, b"same").unwrap();
    assert_ne!(&a[..12], &b[..12], "OS-CSPRNG nonce must differ");
    assert_ne!(a, b);
}

#[test]
fn aead_rejects_wrong_aad() {
    let key = test_key();
    let aad = build_aad(TenantId(1), CollectionId(2), DocId(3), 4);
    let other = build_aad(TenantId(1), CollectionId(2), DocId(3), 5);
    let blob = encrypt_aead(&key, &aad, b"payload").unwrap();
    assert!(matches!(
        decrypt_aead(&key, &other, &blob),
        Err(CryptoError::DecryptionFailed)
    ));
}

#[test]
fn aead_rejects_tampered_ciphertext() {
    let key = test_key();
    let aad = build_aad(TenantId(1), CollectionId(2), DocId(3), 4);
    let mut blob = encrypt_aead(&key, &aad, b"payload").unwrap();
    let last = blob.len() - 1;
    blob[last] ^= 0x01;
    assert!(matches!(
        decrypt_aead(&key, &aad, &blob),
        Err(CryptoError::DecryptionFailed)
    ));
}

#[test]
fn aead_rejects_wrong_key() {
    let aad = build_aad(TenantId(1), CollectionId(2), DocId(3), 4);
    let blob = encrypt_aead(&test_key(), &aad, b"payload").unwrap();
    let other = SecretKey32::from_bytes([8u8; 32]);
    assert!(matches!(
        decrypt_aead(&other, &aad, &blob),
        Err(CryptoError::DecryptionFailed)
    ));
}

#[test]
fn aead_rejects_blob_shorter_than_nonce_plus_tag() {
    let key = test_key();
    let aad = [0u8; 32];
    assert!(matches!(
        decrypt_aead(&key, &aad, &[0u8; 27]),
        Err(CryptoError::CiphertextTooShort)
    ));
    // 28 Byte: Länge reicht, aber kein gültiger Tag -> Authentifizierung schlägt fehl.
    assert!(matches!(
        decrypt_aead(&key, &aad, &[0u8; 28]),
        Err(CryptoError::DecryptionFailed)
    ));
}
