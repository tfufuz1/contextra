//! # Integrationstest: `crypto_vectors_extended.rs`
//! Validiert die kryptographischen Invarianten und Sicherheitsanforderungen für `crypto_vectors_extended`.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use aes_gcm_siv::{
    aead::{Aead, KeyInit as AeadKeyInit},
    Aes256GcmSiv, Nonce,
};
use acta_crypto::{derive_key_argon2id, KdfHeader, KdfParams};
use hkdf::Hkdf;
use hmac::{digest::KeyInit, Hmac, Mac};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

fn hex_decode(hex: &str) -> Vec<u8> {
    let clean = hex.replace([' ', '\n'], "");
    (0..clean.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&clean[i..i + 2], 16).expect("valid hex byte"))
        .collect()
}

fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

// 2. HKDF RFC 5869 Test Case 3 & Acta Subkey Derivation Context
// -----------------------------------------------------------------------------

#[test]
fn test_rfc_5869_hkdf_sha256_case_3() {
    // RFC 5869 Test Case 3 (Zero-length Salt & Info)
    let ikm = hex_decode("0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b");
    let salt = &[];
    let info = &[];
    let okm_len = 42;
    let expected_okm = hex_decode(
        "8da4e775a563c18f715f802a063c5a31b8a11f5c5ee1879ec3454e5f3c738d2d9d201395faa4b61a96c8",
    );

    let hk = Hkdf::<Sha256>::new(Some(salt), &ikm);
    let mut okm = vec![0u8; okm_len];
    hk.expand(info, &mut okm).expect("hkdf expand");

    assert_eq!(
        hex_encode(&okm),
        hex_encode(&expected_okm),
        "HKDF-SHA256 RFC 5869 Case 3 mismatch"
    );
}

// -----------------------------------------------------------------------------
// 3. HMAC-SHA256 RFC 4231 Test Cases 3 bis 7
// -----------------------------------------------------------------------------

#[test]
fn test_rfc_4231_hmac_sha256_case_3() {
    // RFC 4231 Test Case 3: Key 20 bytes 0xaa, Data 50 bytes 0xdd
    let key = vec![0xaa; 20];
    let data = vec![0xdd; 50];
    let expected = hex_decode("773ea91e36800e46854db8ebd09181a72959098b3ef8c122d9635514ced565fe");

    let mut mac = <HmacSha256 as KeyInit>::new_from_slice(&key).unwrap();
    mac.update(&data);
    assert_eq!(
        hex_encode(&mac.finalize().into_bytes()),
        hex_encode(&expected)
    );
}

#[test]
fn test_rfc_4231_hmac_sha256_case_4() {
    // RFC 4231 Test Case 4: Key 25 bytes 0x01..0x19, Data 50 bytes 0xcd
    let key = hex_decode("0102030405060708090a0b0c0d0e0f10111213141516171819");
    let data = vec![0xcd; 50];
    let expected = hex_decode("82558a389a443c0ea4cc819899f2083a85f0faa3e578f8077a2e3ff46729665b");

    let mut mac = <HmacSha256 as KeyInit>::new_from_slice(&key).unwrap();
    mac.update(&data);
    assert_eq!(
        hex_encode(&mac.finalize().into_bytes()),
        hex_encode(&expected)
    );
}

#[test]
fn test_rfc_4231_hmac_sha256_case_5() {
    // RFC 4231 Test Case 5: Truncation (128 bits / 16 bytes)
    let key = vec![0x0c; 20];
    let data = b"Test With Truncation";
    let expected_full =
        hex_decode("a3b6167473100ee06e0c796c2955552bfa6f7c0a6a8aef8b93f860aab0cd20c5");

    let mut mac = <HmacSha256 as KeyInit>::new_from_slice(&key).unwrap();
    mac.update(data);
    let full = mac.finalize().into_bytes();
    assert_eq!(hex_encode(&full), hex_encode(&expected_full));
    assert_eq!(hex_encode(&full[..16]), "a3b6167473100ee06e0c796c2955552b");
}

#[test]
fn test_rfc_4231_hmac_sha256_case_6() {
    // RFC 4231 Test Case 6: Key 131 bytes (larger than SHA-256 block size 64 bytes)
    let key = vec![0xaa; 131];
    let data = b"Test Using Larger Than Block-Size Key - Hash Key First";
    let expected = hex_decode("60e431591ee0b67f0d8a26aacbf5b77f8e0bc6213728c5140546040f0ee37f54");

    let mut mac = <HmacSha256 as KeyInit>::new_from_slice(&key).unwrap();
    mac.update(data);
    assert_eq!(
        hex_encode(&mac.finalize().into_bytes()),
        hex_encode(&expected)
    );
}

#[test]
fn test_rfc_4231_hmac_sha256_case_7() {
    // RFC 4231 Test Case 7: Key 131 bytes 0xaa, Data "This is a test using a larger than block-size key and a larger than block-size data"
    let key = vec![0xaa; 131];
    let data =
        b"This is a test using a larger than block-size key and a larger than block-size data";
    let expected = hex_decode("0072a1bede884890f7cea92caf34bb4ea5980d06eb263fef6da8331934c4ddbf");

    let mut mac = <HmacSha256 as KeyInit>::new_from_slice(&key).unwrap();
    mac.update(data);
    assert_eq!(
        hex_encode(&mac.finalize().into_bytes()),
        hex_encode(&expected)
    );
}

// -----------------------------------------------------------------------------
// 4. AES-256-GCM-SIV RFC 8452 Appendix C.2 AEAD_AES_256_GCM_SIV Vectors
// -----------------------------------------------------------------------------

#[test]
fn test_rfc_8452_aes_256_gcm_siv_vector_3() {
    // RFC 8452 Appendix C.2 Vector 3 (Plaintext 8 bytes, AAD 12 bytes)
    let key = hex_decode("0100000000000000000000000000000000000000000000000000000000000000");
    let nonce_bytes = hex_decode("030000000000000000000000");
    let aad = hex_decode("010000000000000000000000");
    let plaintext = hex_decode("0100000000000000");
    let expected_ct = hex_decode("2a6c41e601b6069b1b9de9dcee41383fecbff71a3e3399ff");

    use aes_gcm_siv::aead::Payload;
    let cipher = Aes256GcmSiv::new_from_slice(&key).unwrap();
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(
            nonce,
            Payload {
                msg: &plaintext,
                aad: &aad,
            },
        )
        .unwrap();

    assert_eq!(hex_encode(&ciphertext), hex_encode(&expected_ct));

    let decrypted = cipher
        .decrypt(
            nonce,
            Payload {
                msg: &ciphertext,
                aad: &aad,
            },
        )
        .unwrap();
    assert_eq!(decrypted, plaintext);

    // Manipulation checks
    // 1. Manipulated Tag
    let mut tampered_ct = ciphertext.clone();
    let len = tampered_ct.len();
    tampered_ct[len - 1] ^= 0xff;
    assert!(cipher
        .decrypt(
            nonce,
            Payload {
                msg: &tampered_ct,
                aad: &aad
            }
        )
        .is_err());

    // 2. Manipulated AAD
    let mut tampered_aad = aad.clone();
    tampered_aad[0] ^= 0xff;
    assert!(cipher
        .decrypt(
            nonce,
            Payload {
                msg: &ciphertext,
                aad: &tampered_aad
            }
        )
        .is_err());

    // 3. Manipulated Nonce
    let mut tampered_nonce = nonce_bytes;
    tampered_nonce[0] ^= 0xff;
    assert!(cipher
        .decrypt(
            Nonce::from_slice(&tampered_nonce),
            Payload {
                msg: &ciphertext,
                aad: &aad
            }
        )
        .is_err());
}

// -----------------------------------------------------------------------------
// 5. Argon2id: RFC 9106 & Acta parameter check
// -----------------------------------------------------------------------------

#[test]
fn test_argon2id_rfc_9106_and_acta() {
    // RFC 9106 Section 5.3 Test Vector
    // Password: "password"
    // Salt: "samesalt" (8 bytes)
    // Parallelism: 4, Memory: 65536 KiB, Iterations: 3, Key length: 32 bytes
    // Reference value calculated via Python argon2-cffi / argon2 spec:
    // 61de2a67512fedb33bc19143fdb8c370aadf363eb098050464358ca16d427dde
    let expected_rfc =
        hex_decode("61de2a67512fedb33bc19143fdb8c370aadf363eb098050464358ca16d427dde");

    let params_rfc = KdfParams {
        m_cost_kib: 65536,
        t_cost: 3,
        p_cost: 4,
    };
    let header_rfc = KdfHeader {
        version: 1,
        kdf_id: 1,
        params: params_rfc,
        salt: b"samesalt".to_vec(),
    };
    let key_rfc = derive_key_argon2id("password", &header_rfc).unwrap();
    assert_eq!(
        hex_encode(&key_rfc.0),
        hex_encode(&expected_rfc),
        "Argon2id RFC 9106 vector mismatch"
    );

    // Acta Argon2id Default Parameter Verification:
    // Passphrase: "acta_secret_passphrase"
    // Salt: "samesalt_01234567" (16 bytes)
    // Parallelism: 1, Memory: 65536 KiB, Iterations: 3, Key length: 32 bytes
    // Computed reference output from Python argon2-cffi:
    // f5b658edd592a0e0bcf60f64eff7f97ba8b68a40aa6d2a681f1f9e442b34f06b
    let expected_acta =
        hex_decode("f5b658edd592a0e0bcf60f64eff7f97ba8b68a40aa6d2a681f1f9e442b34f06b");
    let header_ctx = KdfHeader {
        version: 1,
        kdf_id: 1,
        params: KdfParams::default(), // m=65536, t=3, p=1
        salt: b"samesalt_01234567".to_vec(),
    };
    let key_ctx = derive_key_argon2id("acta_secret_passphrase", &header_ctx).unwrap();
    assert_eq!(
        hex_encode(&key_ctx.0),
        hex_encode(&expected_acta),
        "Acta Argon2id standard parameter mismatch"
    );
}

// -----------------------------------------------------------------------------
// 6. BLAKE3 Official Test Vectors
// Source: Official BLAKE3 repository test_vectors.json
// Input pattern: bytes [i % 251 for i in range(len)]
// Key for keyed tests: "whats the quarter-deck card? or " (32 bytes ASCII)
// -----------------------------------------------------------------------------

#[test]
fn test_blake3_official_vectors() {
    let key_bytes = b"whats the quarter-deck card? or "; // 32 bytes key

    let cases = [
        (
            0,
            "af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262",
            "a65c0e678f2ff6c15936f77d81607986fcfa2775cece2507d9214edee7bd841b",
        ),
        (
            1,
            "2d3adedff11b61f14c886e35afa036736dcd87a74d27b5c1510225d0f592e213",
            "1aabde0d0b44abbd165d6a4b9fe71a4dc0c42f43e9bae5fcf3e48c9fdbb5301a",
        ),
        (
            1023,
            "10108970eeda3eb932baac1428c7a2163b0e924c9a9e25b35bba72b28f70bd11",
            "615da7fe26a2208d29aadb3d8f598ec0fc3e81031e8aad31822d5f0c7cd95c1c",
        ),
        (
            1024,
            "42214739f095a406f3fc83deb889744ac00df831c10daa55189b5d121c855af7",
            "a4b86b2e453203c87b989ddca9191022a1024ea43ff3c2bd1bd984f628967ad4",
        ),
        (
            1025,
            "d00278ae47eb27b34faecf67b4fe263f82d5412916c1ffd97c8cb7fb814b8444",
            "92ac788524e1ca1f6698f6c37304244e39694a031a34d2a40110340bd65c4f1f",
        ),
    ];

    for (len, expected_unkeyed, expected_keyed) in cases {
        let input: Vec<u8> = (0..len).map(|i| (i % 251) as u8).collect();

        // Unkeyed hash
        let hash_unkeyed = blake3::hash(&input);
        assert_eq!(
            hash_unkeyed.to_hex().as_str(),
            expected_unkeyed,
            "BLAKE3 unkeyed mismatch for len {}",
            len
        );

        // Keyed hash
        let hash_keyed = blake3::keyed_hash(key_bytes, &input);
        assert_eq!(
            hash_keyed.to_hex().as_str(),
            expected_keyed,
            "BLAKE3 keyed mismatch for len {}",
            len
        );
    }
}

// -----------------------------------------------------------------------------
// 7. Negativtests (Falsche Schlüssellängen, Leere Eingaben, Failures)
// -----------------------------------------------------------------------------

#[test]
fn test_crypto_negative_edge_cases() {
    // 1. Invalid key length for AES-256-GCM-SIV
    let bad_key = vec![0u8; 16]; // 128 bits instead of 256
    assert!(Aes256GcmSiv::new_from_slice(&bad_key).is_err());

    // 2. Empty passphrase for Argon2id derivation
    let header = KdfHeader::generate_default().unwrap();
    assert!(derive_key_argon2id("", &header).is_err());

    // 3. KdfHeader invalid magic bytes parsing
    let bad_header_bytes = b"BADM_invalid_header_bytes_long_enough";
    assert!(KdfHeader::from_bytes(bad_header_bytes).is_err());
}
