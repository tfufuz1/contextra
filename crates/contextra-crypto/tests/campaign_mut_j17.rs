//! # Integrationstest: `campaign_mut_j17.rs`
//!
//! Validiert die kryptographischen Invarianten und Sicherheitsanforderungen für `campaign_mut_j17`.
//! Prüft Zero-Panic, deterministische Ausführung und Fail-Closed-Verhalten.

#![forbid(unsafe_code)]

use contextra_crypto::kdf::{KDF_HEADER_MAGIC, KDF_HEADER_VERSION_1, KDF_ID_ARGON2ID};
use contextra_crypto::{KdfHeader, KdfParams, KeyManager, KvCipher, KvSegmentCipher};

#[test]
fn killer_test_kdf_params_new_for_test() {
    // Kills mutant: KdfParams::new_for_test -> Default::default()
    // Oracle: Parameters passed to new_for_test(1024, 1, 1) must be preserved in returned struct
    let params = KdfParams::new_for_test(1024, 1, 1);
    assert_eq!(params.m_cost_kib, 1024);
    assert_eq!(params.t_cost, 1);
    assert_eq!(params.p_cost, 1);
}

#[test]
fn killer_test_kdf_header_to_bytes_exact_length_and_magic() {
    // Kills mutants: KdfHeader::to_bytes -> vec![], vec![0], vec![1]
    // Oracle: Layout 4B magic + 1B ver + 1B kdf_id + 4B m + 4B t + 4B p + 4B salt_len + salt.len() B
    let params = KdfParams::new_for_test(20000, 2, 1);
    let salt = vec![0xAA; 16];
    let header = KdfHeader::new(params, salt.clone()).unwrap();
    let bytes = header.to_bytes();

    let expected_len = 4 + 1 + 1 + 4 + 4 + 4 + 4 + 16; // 38 bytes
    assert_eq!(
        bytes.len(),
        expected_len,
        "Serialized KdfHeader length must match header layout"
    );
    assert_eq!(
        &bytes[0..4],
        KDF_HEADER_MAGIC,
        "Serialized header must start with magic MFKD"
    );
    assert_eq!(bytes[4], KDF_HEADER_VERSION_1);
    assert_eq!(bytes[5], KDF_ID_ARGON2ID);
}

#[test]
fn killer_test_kdf_header_from_bytes_boundary_validations() {
    // Kills mutants in KdfHeader::from_bytes comparison & length calculations
    let params = KdfParams::new_for_test(20000, 2, 1);
    let salt = vec![0xBB; 16];
    let valid_header = KdfHeader::new(params, salt).unwrap();
    let valid_bytes = valid_header.to_bytes();

    // 1. Truncated header (< 22 bytes)
    let truncated = &valid_bytes[..21];
    assert!(
        KdfHeader::from_bytes(truncated).is_err(),
        "Header < 22 bytes must fail"
    );

    // 2. Exact 22 bytes header with salt_len = 16 (incomplete payload, 22 < 22 + 16)
    let incomplete_salt = &valid_bytes[..22];
    assert!(
        KdfHeader::from_bytes(incomplete_salt).is_err(),
        "Incomplete salt payload must fail"
    );

    // 3. Header with salt_len below MIN_SALT_LEN (e.g. 15 bytes)
    let mut bad_salt_len_bytes = valid_bytes.clone();
    bad_salt_len_bytes[18..22].copy_from_slice(&15u32.to_be_bytes());
    assert!(
        KdfHeader::from_bytes(&bad_salt_len_bytes).is_err(),
        "salt_len < 16 must fail"
    );

    // 4. Header with salt_len > 10,000 (e.g. 10,001 bytes)
    let mut huge_salt_len_bytes = valid_bytes.clone();
    huge_salt_len_bytes[18..22].copy_from_slice(&10_001u32.to_be_bytes());
    assert!(
        KdfHeader::from_bytes(&huge_salt_len_bytes).is_err(),
        "salt_len > 10000 must fail"
    );
}

#[test]
fn killer_test_key_manager_kv_cipher_seal_open_roundtrip() {
    // Kills mutants in <impl KvCipher for KeyManager>::seal & open
    // Oracle: Decrypted plaintext MUST equal original plaintext
    let km = KeyManager::try_new("test-passphrase-kv-cipher", b"test-salt-kv-cipher").unwrap();
    let plaintext = b"exact plaintext payload for KeyManager KvCipher trait";

    let sealed = km.seal(plaintext).expect("KeyManager seal must succeed");
    assert!(
        sealed.len() >= 12 + plaintext.len(),
        "Sealed payload must contain 12B nonce"
    );

    let opened = km.open(&sealed).expect("KeyManager open must succeed");
    assert_eq!(
        opened, plaintext,
        "Opened payload must equal original plaintext"
    );

    // Short ciphertext (< 12 bytes) must fail open
    let short_ct = vec![0u8; 11];
    assert!(
        km.open(&short_ct).is_err(),
        "Ciphertext < 12 bytes must fail open"
    );
}

#[test]
fn killer_test_kv_segment_cipher_kv_cipher_seal_open() {
    // Kills mutants in <impl KvCipher for KvSegmentCipher>::seal & open
    // Oracle: Decrypted payload MUST equal original plaintext
    let km = KeyManager::try_new("test-passphrase-kv-cipher", b"test-salt-kv-cipher").unwrap();
    let cipher = KvSegmentCipher::new(km);
    let plaintext = b"exact plaintext payload for KvSegmentCipher seal open";

    let sealed = cipher
        .seal(plaintext)
        .expect("KvSegmentCipher seal must succeed");
    let opened = cipher
        .open(&sealed)
        .expect("KvSegmentCipher open must succeed");
    assert_eq!(opened, plaintext);
}
