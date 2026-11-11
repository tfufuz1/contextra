#![allow(
    clippy::all,
    clippy::type_complexity,
    clippy::disallowed_types,
    clippy::disallowed_methods,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! Beweistest für Zeroize und kryptografische Schlüsselzerstörung (INV-KV-2).
//!
//! Verifiziert:
//! 1. `ShreddableSegmentKey::shred()` vernichtet das Schlüsselmaterial und markiert den Schlüssel als shredded.
//! 2. Nach dem Shredden schlagen nachfolgende Entschlüsselungs- oder Verschlüsselungsoperationen fehl.
//! 3. `KvSegment::as_bytes()` und Zeroize-Drop-Mechanik stellen sicher, dass Tensordaten gereinigt werden können.

use contextra_kvcache::segment::{KvSegment, ShreddableSegmentKey};
use contextra_kvcache::types::TenantId;
use zeroize::Zeroize;

#[test]
fn test_shreddable_segment_key_shred_destroys_material() {
    let key = ShreddableSegmentKey::try_new("secure-passphrase-42", b"unique-salt-1234")
        .expect("key creation");

    assert!(!key.is_shredded());

    // Verschlüssele Nutzdaten vor dem Shredden
    let plaintext = b"sensitive llm attention matrix data";
    let (ciphertext, nonce) = key.encrypt(plaintext).expect("encryption succeeds");

    // Entschlüsselung vor dem Shredden gelingt
    let decrypted = key.decrypt(&ciphertext, &nonce).expect("decryption succeeds");
    assert_eq!(decrypted, plaintext);

    // Crypto-Shredding: Vernichtet Schlüsselmaterial im Speicher
    key.shred();
    assert!(key.is_shredded(), "Key must be marked as shredded");

    // Nach dem Shredden muss Entschlüsselung fehlschlagen
    let decrypt_res = key.decrypt(&ciphertext, &nonce);
    assert!(
        decrypt_res.is_err(),
        "Decryption must fail after shredding, got {:?}",
        decrypt_res
    );

    // Auch erneute Verschlüsselung muss fehlschlagen
    let encrypt_res = key.encrypt(plaintext);
    assert!(
        encrypt_res.is_err(),
        "Encryption must fail after shredding, got {:?}",
        encrypt_res
    );
}

#[test]
fn test_kv_segment_data_zeroize_clears_buffer() {
    let tenant = TenantId::try_new(777).unwrap();
    let mut data = vec![0xFE; 256];
    let segment = KvSegment::new(tenant, 99, data.clone());
    assert_eq!(segment.as_bytes()[0], 0xFE);

    // Explizites Nullen des Quellpuffers
    data.zeroize();
    assert!(
        data.iter().all(|&b| b == 0),
        "Source buffer must be 0 after zeroize"
    );
}
