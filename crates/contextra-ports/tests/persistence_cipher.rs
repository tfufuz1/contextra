//! Vertragstests für `persistence::Cipher` ohne konkrete Primitive (SPEC §4, SA-4, UE-10).
//!
//! Das Testdouble ist keine Kryptografie. Es prüft nur die Vertragsform: Längen,
//! Tag am Ende, Fehlerabbildung und die Standardmethoden `seal_into`/`open_into`.
//! Die echte Suite `cipher_suite` (WP-2.4) prüft XChaCha20-Poly1305 gegen die
//! Implementierung in `contextra-crypto`.

use std::error::Error;

use contextra_ports::persistence::{Cipher, CryptoError, Key32};

const TAG_LEN: usize = 16;

/// Testdouble ohne Sicherheitsanspruch: XOR-Strom aus Schlüssel und Nonce,
/// Tag aus einer Rotations-XOR-Kette über alle Eingaben. Ändert sich ein
/// einzelnes Byte von Schlüssel, Nonce, AAD oder Chiffrat, ändert sich der Tag.
/// Der Tag-Vergleich ist nicht konstant-zeitig; das ist für ein Testdouble hinnehmbar.
#[derive(Debug, Default)]
struct DoubleCipher;

fn keystream_byte(key: &Key32, nonce: &[u8; 24], i: usize) -> u8 {
    let k = key.get(i % 32).copied().unwrap_or(0);
    let n = nonce.get(i % 24).copied().unwrap_or(0);
    let pos = u8::try_from(i % 256).unwrap_or(0);
    k ^ n ^ pos
}

fn xor_stream(key: &Key32, nonce: &[u8; 24], data: &[u8]) -> Vec<u8> {
    data.iter()
        .enumerate()
        .map(|(i, b)| b ^ keystream_byte(key, nonce, i))
        .collect()
}

fn tag_over(key: &Key32, nonce: &[u8; 24], aad: &[u8], ct: &[u8]) -> [u8; TAG_LEN] {
    let mut tag = [0u8; TAG_LEN];
    let len_aad = (aad.len() as u64).to_le_bytes();
    let stream = key
        .iter()
        .chain(nonce.iter())
        .chain(len_aad.iter())
        .chain(aad.iter())
        .chain(ct.iter());
    for (pos, byte) in stream.enumerate() {
        if let Some(lane) = tag.get_mut(pos % TAG_LEN) {
            *lane = lane.rotate_left(3) ^ byte;
        }
    }
    tag
}

impl Cipher for DoubleCipher {
    fn seal(&self, key: &Key32, nonce: &[u8; 24], aad: &[u8], plain: &[u8]) -> Vec<u8> {
        let ct = xor_stream(key, nonce, plain);
        let tag = tag_over(key, nonce, aad, &ct);
        let mut out = ct;
        out.extend_from_slice(&tag);
        out
    }

    fn open(
        &self,
        key: &Key32,
        nonce: &[u8; 24],
        aad: &[u8],
        sealed: &[u8],
    ) -> Result<Vec<u8>, CryptoError> {
        if sealed.len() < TAG_LEN {
            return Err(CryptoError::Malformed);
        }
        let split = sealed.len() - TAG_LEN;
        let (ct, tag) = sealed.split_at(split);
        if tag_over(key, nonce, aad, ct).as_slice() != tag {
            return Err(CryptoError::AuthenticationFailed);
        }
        Ok(xor_stream(key, nonce, ct))
    }
}

const KEY: Key32 = [7u8; 32];
const NONCE: [u8; 24] = [3u8; 24];
const AAD: &[u8] = b"aad";
const PLAIN: &[u8] = b"hallo welt";

#[test]
fn port_is_dyn_compatible_with_key32_and_24_byte_nonce() {
    fn assert_dyn(_: &dyn Cipher) {}
    let cipher = DoubleCipher;
    assert_dyn(&cipher);
    // Typalias: Key32 ist [u8; 32] und passt ohne Umwandlung.
    let key: Key32 = KEY;
    let nonce: &[u8; 24] = &NONCE;
    let sealed = cipher.seal(&key, nonce, AAD, PLAIN);
    assert_eq!(sealed.len(), PLAIN.len() + TAG_LEN);
}

#[test]
fn seal_appends_tag_at_end_and_roundtrips() -> Result<(), Box<dyn Error>> {
    let cipher = DoubleCipher;
    let sealed = cipher.seal(&KEY, &NONCE, AAD, PLAIN);
    assert_eq!(sealed.len(), PLAIN.len() + TAG_LEN);
    let plain = cipher.open(&KEY, &NONCE, AAD, &sealed)?;
    assert_eq!(plain, PLAIN);
    Ok(())
}

#[test]
fn roundtrip_with_empty_plaintext() -> Result<(), Box<dyn Error>> {
    let cipher = DoubleCipher;
    let sealed = cipher.seal(&KEY, &NONCE, AAD, &[]);
    assert_eq!(sealed.len(), TAG_LEN);
    assert_eq!(cipher.open(&KEY, &NONCE, AAD, &sealed)?, Vec::<u8>::new());
    Ok(())
}

#[test]
fn open_maps_input_shorter_than_tag_to_malformed() {
    let cipher = DoubleCipher;
    assert_eq!(
        cipher.open(&KEY, &NONCE, AAD, &[]),
        Err(CryptoError::Malformed)
    );
    assert_eq!(
        cipher.open(&KEY, &NONCE, AAD, &[0u8; TAG_LEN - 1]),
        Err(CryptoError::Malformed)
    );
}

#[test]
fn open_maps_any_tampered_byte_to_authentication_failed() {
    let cipher = DoubleCipher;
    let sealed = cipher.seal(&KEY, &NONCE, AAD, PLAIN);
    for pos in 0..sealed.len() {
        let mut bad = sealed.clone();
        if let Some(b) = bad.get_mut(pos) {
            *b ^= 0x01;
        }
        assert_eq!(
            cipher.open(&KEY, &NONCE, AAD, &bad),
            Err(CryptoError::AuthenticationFailed),
            "Manipulation an Position {pos} muss scheitern"
        );
    }
}

#[test]
fn open_binds_key_nonce_and_aad() {
    let cipher = DoubleCipher;
    let sealed = cipher.seal(&KEY, &NONCE, AAD, PLAIN);

    let mut other_key = KEY;
    if let Some(b) = other_key.get_mut(0) {
        *b ^= 1;
    }
    assert_eq!(
        cipher.open(&other_key, &NONCE, AAD, &sealed),
        Err(CryptoError::AuthenticationFailed),
        "falscher Schlüssel"
    );

    let mut nonce_head = NONCE;
    if let Some(b) = nonce_head.get_mut(0) {
        *b ^= 1;
    }
    assert_eq!(
        cipher.open(&KEY, &nonce_head, AAD, &sealed),
        Err(CryptoError::AuthenticationFailed),
        "Nonce: erstes Byte"
    );

    // Die 24-Byte-Nonce wird vollständig gebunden, nicht nur die ersten 12 Byte.
    let mut nonce_tail = NONCE;
    if let Some(b) = nonce_tail.get_mut(23) {
        *b ^= 1;
    }
    assert_eq!(
        cipher.open(&KEY, &nonce_tail, AAD, &sealed),
        Err(CryptoError::AuthenticationFailed),
        "Nonce: letztes Byte (Byte 13 bis 24 gehören zur Nonce)"
    );

    assert_eq!(
        cipher.open(&KEY, &NONCE, b"aaX", &sealed),
        Err(CryptoError::AuthenticationFailed),
        "falsche AAD"
    );
}

#[test]
fn seal_into_appends_and_matches_seal() {
    let cipher = DoubleCipher;
    let mut out = vec![1u8, 2, 3];
    cipher.seal_into(&KEY, &NONCE, AAD, PLAIN, &mut out);
    let mut expected = vec![1u8, 2, 3];
    expected.extend_from_slice(&cipher.seal(&KEY, &NONCE, AAD, PLAIN));
    assert_eq!(out, expected);
}

#[test]
fn open_into_appends_plaintext() -> Result<(), CryptoError> {
    let cipher = DoubleCipher;
    let sealed = cipher.seal(&KEY, &NONCE, AAD, PLAIN);
    let mut out = vec![9u8];
    cipher.open_into(&KEY, &NONCE, AAD, &sealed, &mut out)?;
    let mut expected = vec![9u8];
    expected.extend_from_slice(PLAIN);
    assert_eq!(out, expected);
    Ok(())
}

#[test]
fn open_into_leaves_out_unchanged_on_error() {
    let cipher = DoubleCipher;
    let mut sealed = cipher.seal(&KEY, &NONCE, AAD, PLAIN);
    if let Some(b) = sealed.first_mut() {
        *b ^= 0xff;
    }
    let mut out = vec![9u8, 9];
    let r = cipher.open_into(&KEY, &NONCE, AAD, &sealed, &mut out);
    assert_eq!(r, Err(CryptoError::AuthenticationFailed));
    assert_eq!(
        out,
        vec![9u8, 9],
        "bei Fehler darf out nicht verändert werden"
    );
}

#[test]
fn open_into_maps_errors_like_open() {
    let cipher = DoubleCipher;
    let mut out = Vec::new();
    assert_eq!(
        cipher.open_into(&KEY, &NONCE, AAD, &[0u8; 3], &mut out),
        cipher.open(&KEY, &NONCE, AAD, &[0u8; 3]).map(|_| ())
    );
    assert_eq!(
        cipher.open_into(&KEY, &NONCE, AAD, &[0u8; 3], &mut out),
        Err(CryptoError::Malformed)
    );
    assert!(out.is_empty());
}

#[test]
fn crypto_error_display_is_fixed_text_and_is_std_error() {
    let auth: Box<dyn Error> = Box::new(CryptoError::AuthenticationFailed);
    let short: Box<dyn Error> = Box::new(CryptoError::Malformed);
    assert_eq!(auth.to_string(), "Authentisierung fehlgeschlagen");
    assert_eq!(short.to_string(), "Chiffrat zu kurz");
}
