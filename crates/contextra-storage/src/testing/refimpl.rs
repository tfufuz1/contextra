//! # Referenzimplementierungen der Crypto-Ports (nur Tests)
//!
//! Ersetzt das im Repository fehlende Crate `contextra-crypto-ref`, ohne ein
//! neues Crate anzulegen. Der Kern selbst bleibt frei von Kryptografie (SPEC
//! §3.3, SD-36). Diese Typen sind Testdoubles und dürfen nicht in Produktion
//! verwendet werden. Sie hängen am Feature `testing`.
//!
//! Verfahren: `Blake3Mac` und `Blake3Kdf` nutzen BLAKE3 im Keyed-Modus.
//! `AesGcmSivCipher` nutzt AES-256-GCM-SIV (32 Byte Schlüssel, 12 Byte Nonce,
//! 16 Byte Tag), passend zum AEAD-Vertrag von `Cipher` (SPEC §4).

use contextra_ports::persistence::{Cipher, CryptoError, Kdf, Key32, Mac};

use aes_gcm_siv::aead::{Aead, KeyInit, Payload};
use aes_gcm_siv::{Aes256GcmSiv, Nonce};

/// Keyed-BLAKE3-MAC, auf 16 Byte gekürzt (Testdouble für `Mac`).
#[derive(Debug, Clone, Copy, Default)]
pub struct Blake3Mac;

impl Mac for Blake3Mac {
    fn mac16(&self, key: &[u8; 32], parts: &[&[u8]]) -> [u8; 16] {
        let mut hasher = blake3::Hasher::new_keyed(key);
        for part in parts {
            hasher.update(part);
        }
        let digest = hasher.finalize();
        let mut tag = [0u8; 16];
        for (dst, src) in tag.iter_mut().zip(digest.as_bytes().iter()) {
            *dst = *src;
        }
        tag
    }

    fn verify16(&self, key: &[u8; 32], parts: &[&[u8]], tag: &[u8; 16]) -> bool {
        let expected = self.mac16(key, parts);
        // Konstante Laufzeit: alle 16 Bytes werden stets verglichen (SPEC §4).
        let diff = expected
            .iter()
            .zip(tag.iter())
            .fold(0u8, |acc, (a, b)| acc | (a ^ b));
        diff == 0
    }
}

/// Schlüsselableitung über BLAKE3 im Keyed-Modus (Testdouble für `Kdf`).
#[derive(Debug, Clone, Copy, Default)]
pub struct Blake3Kdf;

impl Kdf for Blake3Kdf {
    fn derive(&self, key: &[u8; 32], context: &[&[u8]]) -> [u8; 32] {
        let mut hasher = blake3::Hasher::new_keyed(key);
        for part in context {
            hasher.update(part);
        }
        *hasher.finalize().as_bytes()
    }
}

/// Verdichtet die 24-Byte-Nonce des Ports auf die 12 Byte, die AES-GCM-SIV braucht.
///
/// PLATZHALTER bis WP-2.4, kein Produktivpfad. Die Verdichtung läuft über BLAKE3,
/// damit jedes der 24 Nonce-Bytes in die Prüfung eingeht. Eine Verdichtung durch
/// Abschneiden würde Nonces, die sich nur in Byte 13 bis 24 unterscheiden,
/// gleich behandeln. Annahme: Für Testzwecke ist die Kollisionswahrscheinlichkeit
/// der 96-Bit-Verdichtung vernachlässigbar.
fn nonce12(nonce: &[u8; 24]) -> [u8; 12] {
    let digest = blake3::hash(nonce);
    let mut out = [0u8; 12];
    for (dst, src) in out.iter_mut().zip(digest.as_bytes().iter()) {
        *dst = *src;
    }
    out
}

/// AES-256-GCM-SIV als AEAD (Testdouble für `Cipher`).
///
/// PLATZHALTER bis WP-2.4, kein Produktivpfad: Der Port verlangt 24-Byte-Nonces
/// (UE-10, XChaCha20-Poly1305), dieses Testdouble arbeitet mit AES-GCM-SIV und
/// verdichtet die Nonce über `nonce12`.
#[derive(Debug, Clone, Copy, Default)]
pub struct AesGcmSivCipher;

impl Cipher for AesGcmSivCipher {
    /// Verschlüsselt. Ein Fehler ist nur bei unzulässiger Länge möglich und
    /// ergibt dann eine leere Ausgabe. Das Öffnen schlägt dann fehl, daher
    /// fällt der Fehler im Test auf.
    fn seal(&self, key: &Key32, nonce: &[u8; 24], aad: &[u8], plain: &[u8]) -> Vec<u8> {
        let cipher = Aes256GcmSiv::new(key.into());
        let n12 = nonce12(nonce);
        cipher
            .encrypt(Nonce::from_slice(&n12), Payload { msg: plain, aad })
            .unwrap_or_default()
    }

    fn open(
        &self,
        key: &Key32,
        nonce: &[u8; 24],
        aad: &[u8],
        sealed: &[u8],
    ) -> Result<Vec<u8>, CryptoError> {
        if sealed.len() < 16 {
            return Err(CryptoError::Malformed);
        }
        let cipher = Aes256GcmSiv::new(key.into());
        let n12 = nonce12(nonce);
        cipher
            .decrypt(Nonce::from_slice(&n12), Payload { msg: sealed, aad })
            .map_err(|_| CryptoError::AuthenticationFailed)
    }
}
