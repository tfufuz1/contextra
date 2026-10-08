// FILE-CONTEXT
// ZWECK: Targeted killer test suite for contextra-crypto mutants (shard 1/4 campaign J-17).
// ORAKEL: Independent math/spec assertions and explicit boundary inputs (R4).

#![forbid(unsafe_code)]

use contextra_crypto::kdf::{KDF_HEADER_MAGIC, KDF_HEADER_VERSION_1, KDF_ID_ARGON2ID};
use contextra_crypto::kv_shredding::{GroupKek, RecordDek};
use contextra_crypto::{
    KdfHeader, KdfParams, KeyManager, KeyRegistry, KvCipher, KvSegmentCipher, SignatureVersion,
    SubKey,
};

#[test]
fn killer_test_signature_version_into_u8() {
    // Kills mutant: <impl From<SignatureVersion> for u8>::from -> Default::default()
    // Oracle: SignatureVersion enum discriminant values V1=1, V2=2, V3=3 defined in spec/code
    let v1_u8: u8 = SignatureVersion::V1.into();
    let v2_u8: u8 = SignatureVersion::V2.into();
    let v3_u8: u8 = SignatureVersion::V3.into();

    assert_eq!(v1_u8, 1, "SignatureVersion::V1 must convert to u8 value 1");
    assert_eq!(v2_u8, 2, "SignatureVersion::V2 must convert to u8 value 2");
    assert_eq!(v3_u8, 3, "SignatureVersion::V3 must convert to u8 value 3");
}

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
    let cipher = KvSegmentCipher::ephemeral(km);
    let plaintext = b"exact plaintext payload for KvSegmentCipher seal open";

    let sealed = cipher
        .seal(plaintext)
        .expect("KvSegmentCipher seal must succeed");
    let opened = cipher
        .open(&sealed)
        .expect("KvSegmentCipher open must succeed");
    assert_eq!(opened, plaintext);
}

#[test]
fn killer_test_key_registry_get_wrapped_kek_and_dek() {
    // Kills mutants in KeyRegistry::get_wrapped_kek & get_wrapped_dek
    let km = KeyManager::try_new("registry-passphrase", b"registry-salt").unwrap();
    let registry = KeyRegistry::new();
    let group_id = 777;
    let record_id = 888;

    // Before group/record creation, get_wrapped_kek/dek return None
    assert!(registry.get_wrapped_kek(group_id).is_none());
    assert!(registry.get_wrapped_dek(group_id, record_id).is_none());

    // Encrypt a record to populate registry
    let payload = registry
        .encrypt_record(&km, group_id, record_id, b"sample record")
        .unwrap();

    // Now get_wrapped_kek and get_wrapped_dek return Some((wrapped, nonce))
    let kek_opt = registry.get_wrapped_kek(group_id);
    assert!(kek_opt.is_some(), "Active group must return wrapped KEK");
    let (wrapped_kek, _kek_nonce) = kek_opt.unwrap();
    assert!(!wrapped_kek.is_empty());

    let dek_opt = registry.get_wrapped_dek(group_id, record_id);
    assert!(dek_opt.is_some(), "Active record must return wrapped DEK");
    let (wrapped_dek, dek_nonce) = dek_opt.unwrap();
    assert_eq!(wrapped_dek, payload.wrapped_dek);
    assert_eq!(dek_nonce, payload.dek_nonce);

    // After revoking record, get_wrapped_dek returns None, but get_wrapped_kek returns Some
    assert!(registry.revoke_record(group_id, record_id).unwrap());
    assert!(
        registry.get_wrapped_dek(group_id, record_id).is_none(),
        "Revoked record wrapped DEK must be None"
    );
    assert!(
        registry.get_wrapped_kek(group_id).is_some(),
        "Unrevoked group wrapped KEK must remain Some"
    );

    // After revoking group, get_wrapped_kek returns None
    assert!(registry.revoke_group(group_id).unwrap());
    assert!(
        registry.get_wrapped_kek(group_id).is_none(),
        "Revoked group wrapped KEK must be None"
    );
}

#[test]
fn killer_test_key_registry_is_group_active_logic() {
    // Kills mutant: replace && with || in KeyRegistry::is_group_active
    let km = KeyManager::try_new("registry-passphrase", b"registry-salt").unwrap();
    let registry = KeyRegistry::new();
    let group_id = 999;

    // 1. Group not in registry and not revoked -> active must be FALSE
    assert!(
        !registry.is_group_active(group_id),
        "Unregistered group must not be active"
    );

    // 2. Group registered and active -> active must be TRUE
    let _ = registry.get_or_derive(&km, group_id).unwrap();
    assert!(
        registry.is_group_active(group_id),
        "Registered active group must be active"
    );

    // 3. Group revoked -> active must be FALSE
    registry.revoke_group(group_id).unwrap();
    assert!(
        !registry.is_group_active(group_id),
        "Revoked group must not be active"
    );
}

#[test]
fn killer_test_debug_redaction_for_subkeys() {
    // Kills mutants replacing Debug fmt for SubKey, GroupKek, RecordDek with Ok(Default::default())
    let subkey = SubKey([0x11; 32]);
    let kek = GroupKek([0x22; 32]);
    let dek = RecordDek([0x33; 32]);

    let sk_debug = format!("{subkey:?}");
    let kek_debug = format!("{kek:?}");
    let dek_debug = format!("{dek:?}");

    assert!(
        sk_debug.contains("***REDACTED***"),
        "SubKey Debug must contain ***REDACTED***"
    );
    assert!(
        kek_debug.contains("***REDACTED***"),
        "GroupKek Debug must contain ***REDACTED***"
    );
    assert!(
        dek_debug.contains("***REDACTED***"),
        "RecordDek Debug must contain ***REDACTED***"
    );
}
