//! `cipher_suite`: XChaCha20-Poly1305 als `Cipher` (SPEC §4, §10.1; UE-10; IMPL WP-2.4).
//!
//! Testvektor: draft-irtf-cfrg-xchacha-01, Anhang A.3.1, übernommen aus
//! `golang/crypto` (`chacha20poly1305/chacha20poly1305_vectors_test.go`, BSD-Lizenz),
//! wo der Eintrag ausdrücklich auf den Anhang verweist. Felder in der Reihenfolge
//! Klartext, AAD, Schlüssel, Nonce, Ausgabe (Chiffretext ‖ Tag).

use std::error::Error;

use contextra_crypto::error::CryptoError;
use contextra_crypto::persistence::{XChaCha20Cipher, TAG_LEN};

type TestResult = Result<(), Box<dyn Error>>;

const DRAFT_A31_PLAIN: &str = "4c616469657320616e642047656e746c656d656e206f662074686520636c617373206f66202739393a204966204920636f756c64206f6666657220796f75206f6e6c79206f6e652074697020666f7220746865206675747572652c2073756e73637265656e20776f756c642062652069742e";
const DRAFT_A31_AAD: &str = "50515253c0c1c2c3c4c5c6c7";
const DRAFT_A31_KEY: &str = "808182838485868788898a8b8c8d8e8f909192939495969798999a9b9c9d9e9f";
const DRAFT_A31_NONCE: &str = "404142434445464748494a4b4c4d4e4f5051525354555657";
const DRAFT_A31_OUT: &str = "bd6d179d3e83d43b9576579493c0e939572a1700252bfaccbed2902c21396cbb731c7f1b0b4aa6440bf3a82f4eda7e39ae64c6708c54c216cb96b72e1213b4522f8c9ba40db5d945b11b69b982c1bb9e3f3fac2bc369488f76b2383565d3fff921f9664c97637da9768812f615c68b13b52ec0875924c1c7987947deafd8780acf49";

fn hex(s: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    let bytes = s.as_bytes();
    if !bytes.len().is_multiple_of(2) {
        return Err("ungerade Hex-Länge".into());
    }
    bytes
        .chunks(2)
        .map(|pair| -> Result<u8, Box<dyn Error>> {
            let txt = std::str::from_utf8(pair)?;
            Ok(u8::from_str_radix(txt, 16)?)
        })
        .collect()
}

fn key32(v: &[u8]) -> Result<[u8; 32], Box<dyn Error>> {
    Ok(<[u8; 32]>::try_from(v)?)
}

fn nonce24(v: &[u8]) -> Result<[u8; 24], Box<dyn Error>> {
    Ok(<[u8; 24]>::try_from(v)?)
}

struct Vector {
    plain: Vec<u8>,
    aad: Vec<u8>,
    key: [u8; 32],
    nonce: [u8; 24],
    out: Vec<u8>,
}

fn draft_a31() -> Result<Vector, Box<dyn Error>> {
    Ok(Vector {
        plain: hex(DRAFT_A31_PLAIN)?,
        aad: hex(DRAFT_A31_AAD)?,
        key: key32(&hex(DRAFT_A31_KEY)?)?,
        nonce: nonce24(&hex(DRAFT_A31_NONCE)?)?,
        out: hex(DRAFT_A31_OUT)?,
    })
}

#[test]
fn draft_a31_seal_matches_public_vector() -> TestResult {
    let v = draft_a31()?;
    let cipher = XChaCha20Cipher;
    assert_eq!(v.out.len(), v.plain.len() + TAG_LEN);
    assert_eq!(cipher.seal(&v.key, &v.nonce, &v.aad, &v.plain), v.out);
    let mut out = Vec::new();
    cipher.seal_into(&v.key, &v.nonce, &v.aad, &v.plain, &mut out)?;
    assert_eq!(out, v.out);
    Ok(())
}

#[test]
fn draft_a31_open_recovers_plaintext() -> TestResult {
    let v = draft_a31()?;
    let cipher = XChaCha20Cipher;
    assert_eq!(cipher.open(&v.key, &v.nonce, &v.aad, &v.out)?, v.plain);
    Ok(())
}

#[test]
fn roundtrip_over_lengths_including_empty() -> TestResult {
    let cipher = XChaCha20Cipher;
    let key = [0x11u8; 32];
    let nonce = [0x22u8; 24];
    for len in [0usize, 1, 15, 16, 17, 63, 64, 65, 255, 4096] {
        let plain: Vec<u8> = (0..len)
            .map(|i| u8::try_from(i % 256))
            .collect::<Result<Vec<u8>, _>>()?;
        let sealed = cipher.seal(&key, &nonce, b"aad", &plain);
        assert_eq!(sealed.len(), plain.len() + TAG_LEN, "len {len}");
        assert_eq!(
            cipher.open(&key, &nonce, b"aad", &sealed)?,
            plain,
            "len {len}"
        );
    }
    Ok(())
}

#[test]
fn tampered_ciphertext_or_tag_is_rejected() -> TestResult {
    let cipher = XChaCha20Cipher;
    let (key, nonce) = ([0x33u8; 32], [0x44u8; 24]);
    let sealed = cipher.seal(&key, &nonce, b"aad", b"geheimer Text");

    let mut body = sealed.clone();
    if let Some(b) = body.first_mut() {
        *b ^= 0x01;
    }
    let mut tag = sealed.clone();
    if let Some(b) = tag.last_mut() {
        *b ^= 0x80;
    }
    for bad in [body, tag] {
        let mut out = Vec::new();
        let res = cipher.open_into(&key, &nonce, b"aad", &bad, &mut out);
        assert!(matches!(res, Err(CryptoError::IntegrityViolation)));
        assert!(out.is_empty(), "kein Klartext bei Fehlschlag");
    }
    Ok(())
}

#[test]
fn wrong_key_nonce_or_aad_is_rejected() -> TestResult {
    let cipher = XChaCha20Cipher;
    let (key, nonce) = ([0x55u8; 32], [0x66u8; 24]);
    let sealed = cipher.seal(&key, &nonce, b"bindung", b"payload");

    let other_key = [0x56u8; 32];
    let other_nonce = [0x67u8; 24];
    assert!(matches!(
        cipher.open(&other_key, &nonce, b"bindung", &sealed),
        Err(CryptoError::IntegrityViolation)
    ));
    assert!(matches!(
        cipher.open(&key, &other_nonce, b"bindung", &sealed),
        Err(CryptoError::IntegrityViolation)
    ));
    assert!(matches!(
        cipher.open(&key, &nonce, b"anders", &sealed),
        Err(CryptoError::IntegrityViolation)
    ));
    Ok(())
}

#[test]
fn input_shorter_than_tag_is_rejected() -> TestResult {
    let cipher = XChaCha20Cipher;
    let (key, nonce) = ([0x77u8; 32], [0x88u8; 24]);
    for len in [0usize, 1, TAG_LEN - 1] {
        let short = vec![0u8; len];
        assert!(
            matches!(
                cipher.open(&key, &nonce, b"", &short),
                Err(CryptoError::IntegrityViolation)
            ),
            "len {len}"
        );
    }
    Ok(())
}

#[test]
fn seal_into_overwrites_previous_buffer_content() -> TestResult {
    let cipher = XChaCha20Cipher;
    let (key, nonce) = ([0x99u8; 32], [0xAAu8; 24]);
    let mut out = vec![0xFFu8; 500];
    cipher.seal_into(&key, &nonce, b"", b"abc", &mut out)?;
    assert_eq!(out, cipher.seal(&key, &nonce, b"", b"abc"));
    Ok(())
}
