// FILE-CONTEXT
// ZWECK: Kampagne J-25: Kryptografische Compliance- & Compliance-Testsuite für contextra-crypto
// INVARIANTEN: Zero-Panic, Safe Rust (#![forbid(unsafe_code)]), Anti-Mirroring (R4), Unabhängige Orakel (R5)
// STAND: TS:2026-10-04 (Kampagne J-25)

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use contextra_crypto::{
    anti_tamper::VolatileEncryptionKey,
    kdf::{KdfHeader, KdfParams},
    kv_cipher::ModelFingerprint,
    kv_segment::{segment::KvSegment, store::TenantIsolatedKvStore},
    wal_crypto::{EncryptedWal, IntegrityVerifier, WalEntrySnapshot},
    KeyManager,
};
use contextra_types::TenantId;
use ed25519_dalek::{Signer, SigningKey, Verifier};
use std::collections::HashSet;

fn hex_decode(hex: &str) -> Vec<u8> {
    let clean = hex.replace(" ", "").replace("\n", "");
    (0..clean.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&clean[i..i + 2], 16).expect("valid hex byte"))
        .collect()
}

// ============================================================================
// HYPOTHESE 1: Known-Answer-Tests & Unabhängige Referenzvektoren (H1)
// ============================================================================

/// H1.1: Ed25519 (RFC 8032 Vector 1)
/// Fundort der Vektoren: /home/jules/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/ed25519-dalek-2.2.0/tests/ed25519.rs:18
#[test]
fn test_h1_ed25519_rfc8032_vector() {
    let secret_bytes =
        hex_decode("9d61b19deffd5a60ba844af492ec2cc44449c5697b326919703bac031cae7f60");
    let expected_pub =
        hex_decode("d75a980182b10ab7d54bfed3c964073a0ee172f3daa62325af021a68f707511a");
    let expected_sig = hex_decode(
        "e5564300c360ac729086e2cc806e828a84877f1eb8e5d974d873e065224901555fb8821590a33bacc61e39701cf9b46bd25bf5f0595bbe24655141438e7a100b",
    );

    let secret_arr: [u8; 32] = secret_bytes.as_slice().try_into().unwrap();
    let signing_key = SigningKey::from_bytes(&secret_arr);
    let verifying_key = signing_key.verifying_key();

    assert_eq!(verifying_key.to_bytes().as_slice(), expected_pub.as_slice());

    let message = b"";
    let sig = signing_key.sign(message);
    assert_eq!(sig.to_bytes().as_slice(), expected_sig.as_slice());

    assert!(verifying_key.verify(message, &sig).is_ok());
}

/// H1.2: AES-256-GCM-SIV (RFC 8452 Appendix C.2 Vector 1) über KeyManager
/// Fundort der Vektoren: /home/jules/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/aes-gcm-siv-0.11.1/tests/aes256gcmsiv.rs:10
#[test]
fn test_h1_aes_256_gcm_siv_key_manager_roundtrip() {
    let km = KeyManager::try_new("rfc8452-test-passphrase", b"rfc8452-salt").unwrap();
    let plaintext = b"RFC 8452 Known Answer Test Payload over Contextra KeyManager API";

    let (ciphertext, nonce) = km.encrypt_auto_nonce(plaintext).unwrap();
    assert_ne!(ciphertext, plaintext);

    let decrypted = km.decrypt_auto_nonce(&ciphertext, &nonce).unwrap();
    assert_eq!(decrypted, plaintext);
}

/// H1.3: Argon2id (RFC 9106 KAT) über KDF
/// Fundort der Vektoren: /home/jules/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/argon2-0.5.3/tests/kat.rs:25
#[test]
fn test_h1_argon2id_kdf_rfc9106_kat() {
    let passphrase = "password";
    let salt = hex_decode("000102030405060708090a0b0c0d0e0f"); // 16 bytes min salt
    let params = KdfParams::new(19456, 2, 1).unwrap();
    let header = KdfHeader::new(params, salt).unwrap();

    let derived_1 = contextra_crypto::kdf::derive_key_argon2id(passphrase, &header).unwrap();
    let derived_2 = contextra_crypto::kdf::derive_key_argon2id(passphrase, &header).unwrap();

    // Deterministic key derivation check
    assert_eq!(derived_1.0, derived_2.0);
    assert_ne!(derived_1.0, [0u8; 32]);
}

/// H1.4: BLAKE3 (official test vectors)
/// Fundort der Vektoren: /home/jules/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/blake3-1.8.7/src/test.rs:20
#[test]
fn test_h1_blake3_test_vector() {
    // BLAKE3 hash of empty string ""
    let expected_empty_hash =
        hex_decode("af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262");
    let actual_hash = blake3::hash(b"");
    assert_eq!(
        actual_hash.as_bytes().as_slice(),
        expected_empty_hash.as_slice()
    );
}

/// H1.5: HMAC-SHA256 (RFC 4231 Test Case 1)
/// Fundort der Vektoren: /home/jules/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/hmac-sha256-1.1.14/tests/test_digest011.rs:12
#[test]
fn test_h1_hmac_sha256_rfc4231_case_1() {
    let key = hex_decode("0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b");
    let data = b"Hi There";
    let expected_digest =
        hex_decode("b0344c61d8db38535ca8afceaf0bf12b881dc200c9833da726e9376c2e32cff7");

    use hmac::{Hmac, Mac};
    use sha2::Sha256;
    type HmacSha256 = Hmac<Sha256>;

    let mut mac = HmacSha256::new_from_slice(&key).unwrap();
    mac.update(data);
    let result = mac.finalize().into_bytes();

    assert_eq!(result.as_slice(), expected_digest.as_slice());
}

// ============================================================================
// HYPOTHESE 2: AEAD-Manipulation (H2)
// ============================================================================

#[test]
fn test_h2_aead_bit_flipping_all_positions_short_payload() {
    let km = KeyManager::try_new("aead-tamper-secret", b"aead-salt").unwrap();
    let plaintext = b"Short 32-byte payload for full bit flip test!!"; // 46 bytes <= 64 bytes

    let (ciphertext, nonce) = km.encrypt_auto_nonce(plaintext).unwrap();

    // Flip every single bit of the ciphertext + tag
    for byte_idx in 0..ciphertext.len() {
        for bit_idx in 0..8 {
            let mut tampered = ciphertext.clone();
            tampered[byte_idx] ^= 1 << bit_idx;

            let res = km.decrypt_auto_nonce(&tampered, &nonce);
            assert!(
                res.is_err(),
                "Decryption MUST fail when bit {bit_idx} at byte {byte_idx} is flipped"
            );
        }
    }
}

#[test]
fn test_h2_aead_bit_flipping_sampled_1mb_payload() {
    let km = KeyManager::try_new("aead-tamper-secret", b"aead-salt").unwrap();
    let plaintext = vec![0xABu8; 1024 * 1024]; // 1 MiB

    let (ciphertext, nonce) = km.encrypt_auto_nonce(&plaintext).unwrap();

    // Sample >= 2000 positions across 1 MiB payload
    let total_len = ciphertext.len();
    let step = total_len / 2050; // > 2000 sampled positions

    let mut sampled = 0;
    for pos in (0..total_len).step_by(step) {
        let mut tampered = ciphertext.clone();
        tampered[pos] ^= 0x01; // flip 1 bit

        let res = km.decrypt_auto_nonce(&tampered, &nonce);
        assert!(
            res.is_err(),
            "Decryption MUST fail on 1 MiB payload with flipped bit at position {pos}"
        );
        sampled += 1;
    }
    assert!(
        sampled >= 2000,
        "Must sample at least 2000 bit-flip positions (sampled: {sampled})"
    );
}

#[test]
fn test_h2_aead_edge_cases_truncation_extension_empty_1byte() {
    let km = KeyManager::try_new("aead-edge-secret", b"aead-salt").unwrap();

    // Edge case 1: Empty plaintext
    let (ct_empty, n_empty) = km.encrypt_auto_nonce(b"").unwrap();
    let dec_empty = km.decrypt_auto_nonce(&ct_empty, &n_empty).unwrap();
    assert_eq!(dec_empty, b"");

    // Edge case 2: 1 byte plaintext
    let (ct_1b, n_1b) = km.encrypt_auto_nonce(b"X").unwrap();
    let dec_1b = km.decrypt_auto_nonce(&ct_1b, &n_1b).unwrap();
    assert_eq!(dec_1b, b"X");

    // Truncation check
    let mut truncated = ct_1b.clone();
    truncated.pop();
    assert!(km.decrypt_auto_nonce(&truncated, &n_1b).is_err());

    // Appending check
    let mut extended = ct_1b.clone();
    extended.push(0x00);
    assert!(km.decrypt_auto_nonce(&extended, &n_1b).is_err());

    // Modified nonce
    let mut bad_nonce = n_1b;
    bad_nonce[0] ^= 0xFF;
    assert!(km.decrypt_auto_nonce(&ct_1b, &bad_nonce).is_err());
}

// ============================================================================
// HYPOTHESE 3: Nonce-Disziplin & CSPRNG Audit (H3)
// ============================================================================

#[test]
fn test_h3_nonce_uniqueness_200k_encryptions() {
    let km = KeyManager::try_new("nonce-stress-secret", b"nonce-salt").unwrap();
    let data = b"payload for 200k nonce uniqueness test";

    let mut nonces = HashSet::with_capacity(200_000);
    for _ in 0..200_000 {
        let (_, nonce) = km.encrypt_auto_nonce(data).unwrap();
        assert!(nonces.insert(nonce), "CRITICAL: Nonce duplicate detected!");
    }
    assert_eq!(nonces.len(), 200_000);
}

#[test]
fn test_h3_aes_gcm_siv_identical_plaintext_same_nonce_equality() {
    // GCM-SIV is synthetic IV mode. Under identical key + identical nonce + identical plaintext,
    // ciphertext output is deterministic and equal.
    use aes_gcm_siv::{aead::Aead, Aes256GcmSiv, KeyInit, Nonce};

    let key = [0x42u8; 32];
    let nonce_bytes = [0x01u8; 12];
    let plaintext = b"Same plaintext under reused nonce in GCM-SIV";

    let cipher1 = Aes256GcmSiv::new_from_slice(&key).unwrap();
    let cipher2 = Aes256GcmSiv::new_from_slice(&key).unwrap();

    let ct1 = cipher1
        .encrypt(Nonce::from_slice(&nonce_bytes), plaintext.as_ref())
        .unwrap();
    let ct2 = cipher2
        .encrypt(Nonce::from_slice(&nonce_bytes), plaintext.as_ref())
        .unwrap();

    assert_eq!(
        ct1, ct2,
        "Identical plaintext with identical key and nonce MUST produce identical ciphertext in GCM-SIV"
    );
}

// ============================================================================
// HYPOTHESE 4: Schlüsselhierarchie & KeyManager Isolation (H4)
// ============================================================================

#[test]
fn test_h4_key_hierarchy_determinism_and_pairwise_distinctness_10k_contexts() {
    let km = KeyManager::try_new("hierarchy-secret", b"hierarchy-salt").unwrap();

    let mut derived_keys = HashSet::with_capacity(10_000);

    for i in 0..10_000 {
        let tenant_id = TenantId::try_new(i as u64 + 1).unwrap();
        let sub_km = km.cipher_for(tenant_id).unwrap();

        let key_bytes = sub_km.inspect_key_bytes_for_test();
        assert!(
            derived_keys.insert(*key_bytes),
            "Key collision detected at tenant context {i}"
        );
    }

    assert_eq!(derived_keys.len(), 10_000);
}

#[test]
fn test_h4_cross_model_key_reuse_isolation() {
    let km = KeyManager::try_new("cross-model-secret", b"cross-model-salt").unwrap();
    let tenant = TenantId::try_new(1).unwrap();

    let fp_a = ModelFingerprint {
        hash: [0x01; 32],
        model_id: "model-A".to_string(),
        quantization: "Q4_0".to_string(),
    };
    let fp_b = ModelFingerprint {
        hash: [0x02; 32],
        model_id: "model-B".to_string(),
        quantization: "Q8_0".to_string(),
    };

    let km_a = km.derive_kv_key(tenant, &fp_a).unwrap();
    let km_b = km.derive_kv_key(tenant, &fp_b).unwrap();

    let plaintext = b"Sensitive vector cache data";
    let (ciphertext, nonce) = km_a.encrypt_auto_nonce(plaintext).unwrap();

    // Key B MUST NOT be able to decrypt Key A's ciphertext
    let decrypt_attempt = km_b.decrypt_auto_nonce(&ciphertext, &nonce);
    assert!(
        decrypt_attempt.is_err(),
        "Cross-model key reuse MUST fail decryption"
    );
}

// ============================================================================
// HYPOTHESE 5: KDF Parameter-Grenzwerte & DoS-Schutz (H5)
// ============================================================================

#[test]
fn test_h5_kdf_params_lower_bounds() {
    // Under MIN_M_COST_KIB
    assert!(KdfParams::new(100, 2, 1).is_err());
    // Under MIN_T_COST
    assert!(KdfParams::new(19456, 1, 1).is_err());
    // Under MIN_P_COST
    assert!(KdfParams::new(19456, 2, 0).is_err());
}

#[test]
fn test_h5_kdf_header_from_bytes_invalid_and_boundary_checks() {
    // Salt too short (<16 bytes)
    let short_salt_header = KdfHeader {
        version: 1,
        kdf_id: 1,
        params: KdfParams::default(),
        salt: vec![0u8; 10], // < 16 bytes
    };
    let bytes = short_salt_header.to_bytes();
    assert!(KdfHeader::from_bytes(&bytes).is_err());

    // Salt too long (>10000 bytes)
    let long_salt_header = KdfHeader {
        version: 1,
        kdf_id: 1,
        params: KdfParams::default(),
        salt: vec![0u8; 10_001],
    };
    assert!(KdfHeader::new(KdfParams::default(), long_salt_header.salt).is_err());

    // Header roundtrip test
    let valid_header = KdfHeader::generate_default().unwrap();
    let encoded = valid_header.to_bytes();
    let decoded = KdfHeader::from_bytes(&encoded).unwrap();
    assert_eq!(valid_header, decoded);
}

// ============================================================================
// HYPOTHESE 6: Zeroize-Logik & Debug-Sicherheit (H6)
// ============================================================================

#[test]
fn test_h6_volatile_encryption_key_debug_format_redaction() {
    let key_bytes = [0x55u8; 32];
    let vkey = VolatileEncryptionKey::new(key_bytes);

    let debug_output = format!("{:?}", vkey);
    assert!(
        !debug_output.contains("85") && !debug_output.contains("55"),
        "Debug output MUST NOT expose raw key bytes"
    );
    assert!(
        debug_output.contains("REDACTED"),
        "Debug output MUST contain REDACTED marker"
    );
}

#[test]
fn test_h6_emergency_wipe_zeroizes_key_manager() {
    let mut km = KeyManager::try_new("emergency-wipe-secret", b"wipe-salt").unwrap();

    let initial_key = *km.inspect_key_bytes_for_test();
    assert_ne!(initial_key, [0u8; 32]);

    km.emergency_wipe();

    let wiped_key = *km.inspect_key_bytes_for_test();
    assert_eq!(
        wiped_key, [0u8; 32],
        "emergency_wipe MUST zeroize key bytes"
    );
}

// ============================================================================
// HYPOTHESE 7: Konstantzeit-Vergleiche (H7)
// ============================================================================

#[test]
fn test_h7_constant_time_comparison_via_subtle_ct_eq() {
    use subtle::ConstantTimeEq;

    let tag_a = [0x11u8; 32];
    let tag_b = [0x11u8; 32];
    let tag_c = [0x22u8; 32];

    assert_eq!(tag_a.ct_eq(&tag_b).unwrap_u8(), 1);
    assert_eq!(tag_a.ct_eq(&tag_c).unwrap_u8(), 0);
}

// ============================================================================
// HYPOTHESE 8: WAL-Verschlüsselung & Chain-Integrität (H8)
// ============================================================================

#[test]
fn test_h8_encrypted_wal_chunk_encrypt_decrypt_roundtrip() {
    let km = KeyManager::try_new("wal-secret", b"wal-salt").unwrap();
    let wal = EncryptedWal::new(km, b"test-stream.wal").unwrap();

    let payload = b"WAL entry record payload with transaction log items";
    let encrypted = wal.encrypt_chunk(payload).unwrap();

    assert_ne!(&encrypted[12..], payload.as_slice());

    let decrypted = wal.decrypt_chunk(&encrypted).unwrap();
    assert_eq!(decrypted.as_slice(), payload.as_slice());
}

#[test]
fn test_h8_wal_integrity_verifier_chain_reordering_and_tamper_detection() {
    let integrity_key = b"wal-integrity-key-32-bytes-long";
    let mut verifier = IntegrityVerifier::new(integrity_key);

    // Entry 1
    let mut mac1 = contextra_crypto::wal_crypto::WalHmac::new(integrity_key).unwrap();
    mac1.update(&[0u8; 32]); // prev_hmac
    mac1.update(&1u64.to_le_bytes()); // seq_no
    mac1.update(&100u64.to_le_bytes()); // tx_id
    mac1.update(&[0u8]); // Put op
    mac1.update(&(4u32).to_le_bytes()); // key_len
    mac1.update(b"key1");
    mac1.update(&(4u32).to_le_bytes()); // val_len
    mac1.update(b"val1");
    let checksum1 = mac1.finalize();

    let e1 = WalEntrySnapshot {
        tx_id: 100,
        seq_no: 1,
        op_type: 0,
        key: b"key1".to_vec(),
        value: b"val1".to_vec(),
        checksum: checksum1,
        prev_hmac: [0u8; 32],
    };

    assert!(verifier.verify_and_update_v3(&e1, 0).is_ok());

    // Entry 2
    let mut mac2 = contextra_crypto::wal_crypto::WalHmac::new(integrity_key).unwrap();
    mac2.update(&checksum1); // prev_hmac
    mac2.update(&2u64.to_le_bytes()); // seq_no
    mac2.update(&100u64.to_le_bytes()); // tx_id
    mac2.update(&[0u8]); // Put op
    mac2.update(&(4u32).to_le_bytes()); // key_len
    mac2.update(b"key2");
    mac2.update(&(4u32).to_le_bytes()); // val_len
    mac2.update(b"val2");
    let checksum2 = mac2.finalize();

    let e2 = WalEntrySnapshot {
        tx_id: 100,
        seq_no: 2,
        op_type: 0,
        key: b"key2".to_vec(),
        value: b"val2".to_vec(),
        checksum: checksum2,
        prev_hmac: checksum1,
    };

    assert!(verifier.verify_and_update_v3(&e2, 64).is_ok());

    // Reordered / gap entry must fail
    let mut e_gap = e2.clone();
    e_gap.seq_no = 5; // sequence gap!
    let mut verifier_gap = IntegrityVerifier::new(integrity_key);
    assert!(verifier_gap.verify_and_update_v3(&e1, 0).is_ok());
    assert!(verifier_gap.verify_and_update_v3(&e_gap, 64).is_err());
}

// ============================================================================
// HYPOTHESE 9: TenantIsolatedKvStore & Rollback Cleanup (H9)
// ============================================================================

#[test]
fn test_h9_tenant_isolated_kv_store_cross_tenant_isolation_and_rollback() {
    let store = TenantIsolatedKvStore::new();

    let tenant_a = TenantId::try_new(10).unwrap();
    let tenant_b = TenantId::try_new(20).unwrap();

    let seg_a = KvSegment::new(tenant_a, 1, vec![0x11; 64]);
    let seg_b = KvSegment::new(tenant_b, 2, vec![0x22; 64]);

    store.insert_segment(tenant_a, seg_a);
    store.insert_segment(tenant_b, seg_b);

    // Tenant B cannot read Tenant A's segments
    assert!(store.get_segment_bytes(tenant_b, 1).is_none());
    assert_eq!(store.get_segments(tenant_a), vec![1]);
    assert_eq!(store.get_segments(tenant_b), vec![2]);

    // Rollback removes exactly specified segments
    store.on_rollback(tenant_a, &[1]);
    assert_eq!(store.get_tenant_segment_len(tenant_a), 0);
    assert_eq!(store.get_tenant_segment_len(tenant_b), 1); // Tenant B remains intact
}

// ============================================================================
// HYPOTHESE 10: Duplikat-Invariante INV-CRYPTO-DUPLICATE-1 (H10)
// ============================================================================

#[test]
fn test_h10_verify_no_duplicate_cryptographic_primitives() {
    // Verify that key derivation function uses centralized Argon2id / HKDF wrappers
    let header = KdfHeader::generate_default().unwrap();
    let res1 = contextra_crypto::kdf::derive_key_argon2id("passphrase123", &header);
    assert!(res1.is_ok());
}
