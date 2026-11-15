//! RFC-Testvektoren als unabhängiges Orakel (Spec 16.2 „Golden“, 16.1).
//!
//! HKDF (RFC 5869) und HMAC (RFC 4231) stammen aus dem Archiv `rfc_vectors.rs`. Der dortige
//! AES-GCM-SIV-Teil (RFC 8452) ist entfernt, weil das Verfahren nicht Teil des Umschlags v1 ist
//! und `aes-gcm-siv` nicht auf der Whitelist steht. Ed25519 (RFC 8032, Test 1) ist neu hinzugekommen.

use acta_crypto::hash::sha256;
use acta_crypto::kdf::hkdf_sha256;
use acta_crypto::mac::hmac_sha256;
use acta_crypto::sign::NotarySigner;
use std::io;

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn unhex(s: &str) -> Vec<u8> {
    let clean: String = s.chars().filter(|c| !c.is_whitespace()).collect();
    (0..clean.len())
        .step_by(2)
        .filter_map(|i| {
            clean
                .get(i..i + 2)
                .and_then(|h| u8::from_str_radix(h, 16).ok())
        })
        .collect()
}

#[test]
fn rfc5869_case1_first_32_bytes() -> io::Result<()> {
    let okm = hkdf_sha256(
        &[&unhex("0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b")],
        &unhex("000102030405060708090a0b0c"),
        &unhex("f0f1f2f3f4f5f6f7f8f9"),
    )
    .map_err(io::Error::other)?;
    // Die RFC-Ausgabe hat 42 Byte. HKDF ist präfixstabil, also genügen die ersten 32.
    assert_eq!(
        hex(okm.as_slice()),
        "3cb25f25faacd57a90434f64d0362f2a2d2d0a90cf1a5a4c5db02d56ecc4c5bf"
    );
    Ok(())
}

#[test]
fn rfc5869_case2_first_32_bytes() -> io::Result<()> {
    let ikm = unhex(
        "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f\
         202122232425262728292a2b2c2d2e2f303132333435363738393a3b3c3d3e3f\
         404142434445464748494a4b4c4d4e4f",
    );
    let salt = unhex(
        "606162636465666768696a6b6c6d6e6f707172737475767778797a7b7c7d7e7f\
         808182838485868788898a8b8c8d8e8f909192939495969798999a9b9c9d9e9f\
         a0a1a2a3a4a5a6a7a8a9aaabacadaeaf",
    );
    let info = unhex(
        "b0b1b2b3b4b5b6b7b8b9babbbcbdbebfc0c1c2c3c4c5c6c7c8c9cacbcccdcecf\
         d0d1d2d3d4d5d6d7d8d9dadbdcdddedfe0e1e2e3e4e5e6e7e8e9eaebecedeeef\
         f0f1f2f3f4f5f6f7f8f9fafbfcfdfeff",
    );
    let okm = hkdf_sha256(&[&ikm], &salt, &info).map_err(io::Error::other)?;
    assert_eq!(
        hex(okm.as_slice()),
        "b11e398dc80327a1c8e7f78c596a49344f012eda2d4efad8a050cc4c19afa97c"
    );
    Ok(())
}

#[test]
fn rfc4231_case1_matches() -> io::Result<()> {
    let tag = hmac_sha256(&[0x0b; 20], &[b"Hi There"]).map_err(io::Error::other)?;
    assert_eq!(
        hex(&tag),
        "b0344c61d8db38535ca8afceaf0bf12b881dc200c9833da726e9376c2e32cff7"
    );
    Ok(())
}

#[test]
fn rfc4231_case2_matches() -> io::Result<()> {
    let tag = hmac_sha256(b"Jefe", &[b"what do ya want for nothing?"]).map_err(io::Error::other)?;
    assert_eq!(
        hex(&tag),
        "5bdcc146bf60754e6a042426089575c75a003f089d2739839dec58b964ec3843"
    );
    Ok(())
}

#[test]
fn sha256_empty_matches_nist() {
    assert_eq!(
        hex(&sha256(&[])),
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
    );
}

#[test]
fn rfc8032_ed25519_test1_empty_message() -> io::Result<()> {
    // RFC 8032, Abschnitt 7.1, Test 1. Die Domäne ist leer, die Nachricht ist leer.
    let seed: [u8; 32] = unhex("9d61b19deffd5a60ba844af492ec2cc44449c5697b326919703bac031cae7f60")
        .try_into()
        .map_err(|_| io::Error::other("seed length"))?;
    let signer = NotarySigner::from_seed(seed);
    assert_eq!(
        hex(&signer.public_key()),
        "d75a980182b10ab7d54bfed3c964073a0ee172f3daa62325af021a68f707511a"
    );
    assert_eq!(
        hex(&signer.sign(&[], &[])),
        "e5564300c360ac729086e2cc806e828a84877f1eb8e5d974d873e065224901555fb8821590a33bacc61e39701cf9b46bd25bf5f0595bbe24655141438e7a100b"
    );
    Ok(())
}
