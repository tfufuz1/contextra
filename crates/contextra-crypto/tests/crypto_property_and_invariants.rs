//! Comprehensive Property and Invariant Tests for contextra-crypto (Class A, Ring 0).
//!
//! Validates:
//! - INV-CRYPTO-ZEROIZE-1
//! - INV-CRYPTO-CONSTANT-TIME-1
//! - open(seal(x)) == x roundtrip
//! - Bit-mutation rejection across ciphertext, tag, AAD, and nonce.

use contextra_crypto::aead::{open_record, seal_record_with_nonce};
use contextra_crypto::anti_tamper::{constant_time_compare, verify_tag_constant_time, VolatileEncryptionKey};
use contextra_crypto::dek::{unwrap_dek, wrap_dek, Dek, UnitKey};
use contextra_crypto::secret::{Secret, SecretKey};
use proptest::prelude::*;
use zeroize::Zeroize;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    #[test]
    fn prop_aead_seal_open_roundtrip(
        key_bytes in prop::collection::vec(any::<u8>(), 32),
        doc in prop::array::uniform16(any::<u8>()),
        key_meta in prop::collection::vec(any::<u8>(), 1..64),
        plaintext in prop::collection::vec(any::<u8>(), 0..256),
        nonce in prop::array::uniform12(any::<u8>())
    ) {
        let mut k = [0u8; 32];
        k.copy_from_slice(&key_bytes);
        let dek = Dek::new(k);

        let env = seal_record_with_nonce(&dek, doc, &key_meta, &plaintext, nonce).expect("seal");
        let opened = open_record(&dek, &key_meta, &env).expect("open");
        prop_assert_eq!(opened.as_slice(), plaintext.as_slice());
    }

    #[test]
    fn prop_bit_mutation_rejects(
        key_bytes in prop::collection::vec(any::<u8>(), 32),
        doc in prop::array::uniform16(any::<u8>()),
        key_meta in prop::collection::vec(any::<u8>(), 1..64),
        plaintext in prop::collection::vec(any::<u8>(), 1..128),
        nonce in prop::array::uniform12(any::<u8>()),
        mutation_idx in 0..150usize,
        bit_idx in 0..8u8
    ) {
        let mut k = [0u8; 32];
        k.copy_from_slice(&key_bytes);
        let dek = Dek::new(k);

        let mut env = seal_record_with_nonce(&dek, doc, &key_meta, &plaintext, nonce).expect("seal");
        if !env.is_empty() {
            let idx = mutation_idx % env.len();
            env[idx] ^= 1 << (bit_idx % 8);
            let res = open_record(&dek, &key_meta, &env);
            prop_assert!(res.is_err());
        }
    }
}

#[test]
fn test_inv_crypto_zeroize_1_contract() {
    let key_arr = [0xAAu8; 32];
    let mut secret = SecretKey::new(key_arr);
    assert_eq!(secret.expose_secret(), &[0xAAu8; 32]);

    secret.zeroize();
    assert_eq!(secret.expose_secret(), &[0x00u8; 32]);

    let mut volatile = VolatileEncryptionKey::new(key_arr);
    assert_eq!(volatile.as_bytes(), &[0xAAu8; 32]);
    volatile.emergency_wipe();
    assert_eq!(volatile.as_bytes(), &[0x00u8; 32]);

    let mut secret_dyn = Secret::new([0xFFu8; 64]);
    assert_eq!(secret_dyn.expose_secret(), &[0xFFu8; 64]);
    secret_dyn.zeroize();
    assert_eq!(secret_dyn.expose_secret(), &[0x00u8; 64]);
}

#[test]
fn test_inv_crypto_constant_time_1_contract() {
    let tag_a = [0x33u8; 32];
    let mut tag_b = [0x33u8; 32];
    assert!(constant_time_compare(&tag_a, &tag_b));
    assert!(verify_tag_constant_time(&tag_a, &tag_b).is_ok());

    tag_b[15] ^= 0x80;
    assert!(!constant_time_compare(&tag_a, &tag_b));
    assert!(verify_tag_constant_time(&tag_a, &tag_b).is_err());
}

#[test]
fn test_dek_key_wrapping_roundtrip() {
    let kek = UnitKey::new([0x42u8; 32]);
    let dek = Dek::new([0x77u8; 32]);
    let nonce = [0x12u8; 12];

    let wrapped = wrap_dek(&kek, &dek, nonce).expect("wrap");
    let unwrapped = unwrap_dek(&kek, &wrapped).expect("unwrap");

    assert_eq!(unwrapped.expose_secret(), dek.expose_secret());
}
