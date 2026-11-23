//! `Mac` als BLAKE3-keyed, auf 16 Byte gekürzt (SPEC §4, §10.1; algo_set 1).
//!
//! Die Teile werden ohne Längenangabe hintereinander gehasht; der Kern kodiert Längen
//! selbst (SPEC §4). Die Kürzung auf die ersten 16 Byte der 32-Byte-Ausgabe nennt die
//! Spec nicht; sie ist eine Annahme (siehe PR-Beschreibung).

use contextra_ports::persistence::Mac;
use subtle::ConstantTimeEq;

/// BLAKE3-keyed-MAC mit 16-Byte-Tag. Zustandslos.
#[derive(Debug, Clone, Copy, Default)]
pub struct Blake3Mac;

impl Blake3Mac {
    fn digest(key: &[u8; 32], parts: &[&[u8]]) -> [u8; 16] {
        let mut hasher = blake3::Hasher::new_keyed(key);
        for part in parts {
            hasher.update(part);
        }
        let full = hasher.finalize();
        match full.as_bytes().first_chunk::<16>() {
            Some(tag) => *tag,
            // Unerreichbar: BLAKE3 liefert stets 32 Byte.
            None => [0u8; 16],
        }
    }
}

impl Mac for Blake3Mac {
    fn mac16(&self, key: &[u8; 32], parts: &[&[u8]]) -> [u8; 16] {
        Self::digest(key, parts)
    }

    fn verify16(&self, key: &[u8; 32], parts: &[&[u8]], tag: &[u8; 16]) -> bool {
        let expected = Self::digest(key, parts);
        bool::from(expected.as_slice().ct_eq(tag.as_slice()))
    }
}
