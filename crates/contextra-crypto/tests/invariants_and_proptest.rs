//! Invariant and Property verification test suite for `contextra-crypto`.

use contextra_crypto::aead::{open_record, seal_record_with_nonce};
use contextra_crypto::anti_tamper::{compute_keyed_mac, verify_keyed_mac, VolatileEncryptionKey};
use contextra_crypto::dek::Dek;
use contextra_crypto::secret::SecretKey;
use proptest::prelude::*;
use subtle::ConstantTimeEq;
use zeroize::Zeroize;

#[test]
fn test_inv_crypto_zeroize_1_secret_key() {
    let raw = [0x5Au8; 32];
    let mut secret = SecretKey::new(raw);
    assert_eq!(secret.expose_secret(), &[0x5Au8; 32]);

    secret.zeroize();
    assert_eq!(
        secret.expose_secret(),
        &[0u8; 32],
        "INV-CRYPTO-ZEROIZE-1 violation: SecretKey memory was not zeroed on zeroize"
    );
}

#[test]
fn test_inv_crypto_zeroize_1_volatile_key() {
    let raw = [0x7Eu8; 32];
    let mut vkey = VolatileEncryptionKey::new(raw);
    assert_eq!(vkey.as_bytes(), &[0x7Eu8; 32]);

    vkey.emergency_wipe();
    assert_eq!(
        vkey.as_bytes(),
        &[0u8; 32],
        "INV-CRYPTO-ZEROIZE-1 violation: VolatileEncryptionKey not zeroed on wipe"
    );
}

#[test]
fn test_inv_crypto_constant_time_1_tag_comparison() {
    let key = [0x33u8; 32];
    let msg = b"sensitive-data-payload";
    let valid_tag = compute_keyed_mac(&key, msg);

    assert!(verify_keyed_mac(&key, msg, &valid_tag));

    let mut bad_tag = valid_tag;
    bad_tag[0] ^= 0x01;
    assert!(!verify_keyed_mac(&key, msg, &bad_tag));

    let s1 = SecretKey::new([0x12u8; 32]);
    let s2 = SecretKey::new([0x12u8; 32]);
    let s3 = SecretKey::new([0x13u8; 32]);

    assert!(bool::from(s1.ct_eq(&s2)));
    assert!(!bool::from(s1.ct_eq(&s3)));
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    #[test]
    fn prop_seal_open_roundtrip(
        key_bytes in prop::collection::vec(any::<u8>(), 1..64),
        plaintext in prop::collection::vec(any::<u8>(), 0..512),
        doc in any::<[u8; 16]>(),
        nonce in any::<[u8; 12]>(),
    ) {
        let secret = SecretKey::new([0x24u8; 32]);
        let dek = Dek::new(secret);

        let env = seal_record_with_nonce(&dek, doc, &key_bytes, &plaintext, nonce);
        let opened = open_record(&dek, &key_bytes, &env).expect("open must succeed for valid seal");
        prop_assert_eq!(opened.as_slice(), plaintext.as_slice());
    }

    #[test]
    fn prop_tampered_ciphertext_rejected(
        key_bytes in prop::collection::vec(any::<u8>(), 1..64),
        plaintext in prop::collection::vec(any::<u8>(), 1..256),
        doc in any::<[u8; 16]>(),
        nonce in any::<[u8; 12]>(),
        flip_idx in 0usize..256,
    ) {
        let secret = SecretKey::new([0x35u8; 32]);
        let dek = Dek::new(secret);

        let mut env = seal_record_with_nonce(&dek, doc, &key_bytes, &plaintext, nonce);
        let ct_offset = 30;
        if env.len() > ct_offset {
            let actual_flip_pos = ct_offset + (flip_idx % (env.len() - ct_offset));
            env[actual_flip_pos] ^= 0x01;
            let res = open_record(&dek, &key_bytes, &env);
            prop_assert!(res.is_err(), "Corrupted ciphertext byte must fail AEAD verification");
        }
    }

    #[test]
    fn prop_tampered_nonce_rejected(
        key_bytes in prop::collection::vec(any::<u8>(), 1..64),
        plaintext in prop::collection::vec(any::<u8>(), 1..256),
        doc in any::<[u8; 16]>(),
        nonce in any::<[u8; 12]>(),
        nonce_byte_idx in 0usize..12,
    ) {
        let secret = SecretKey::new([0x46u8; 32]);
        let dek = Dek::new(secret);

        let mut env = seal_record_with_nonce(&dek, doc, &key_bytes, &plaintext, nonce);
        let nonce_pos = 18 + (nonce_byte_idx % 12);
        env[nonce_pos] ^= 0x01;
        let res = open_record(&dek, &key_bytes, &env);
        prop_assert!(res.is_err(), "Corrupted nonce must fail AEAD verification");
    }

    #[test]
    fn prop_tampered_aad_rejected(
        key_bytes in prop::collection::vec(any::<u8>(), 1..64),
        plaintext in prop::collection::vec(any::<u8>(), 1..256),
        doc in any::<[u8; 16]>(),
        nonce in any::<[u8; 12]>(),
        tampered_byte in any::<u8>(),
    ) {
        let secret = SecretKey::new([0x57u8; 32]);
        let dek = Dek::new(secret);

        let env = seal_record_with_nonce(&dek, doc, &key_bytes, &plaintext, nonce);
        let mut corrupted_key = key_bytes.clone();
        corrupted_key.push(tampered_byte);

        let res = open_record(&dek, &corrupted_key, &env);
        prop_assert!(res.is_err(), "Mismatching AAD must fail AEAD verification");
    }
}
