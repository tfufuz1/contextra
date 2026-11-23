//! `Cipher` als XChaCha20-Poly1305 (SPEC §4, §10.1; UE-10).
//!
//! Schlüssel 32 Byte, Nonce 24 Byte, Tag 16 Byte, angehängt an den Chiffretext.
//! Die Nonce übergibt der Aufrufer (SPEC §10.1: frische Zufallsnonce je Versiegelung).
//! Dieses Modul erzeugt keine Nonce und zählt keine Versiegelungen (SD-14, D-28).
//! Längen kodiert der Kern selbst; hier wird keine Länge angehängt.

use chacha20poly1305::aead::AeadInPlace;
use chacha20poly1305::{Key, KeyInit, XChaCha20Poly1305, XNonce};
use zeroize::Zeroize;

use crate::error::CryptoError;

/// Länge des Authentisierungs-Tags in Byte (SPEC §4).
pub const TAG_LEN: usize = 16;

/// XChaCha20-Poly1305 ohne Zustand. Schlüssel werden je Aufruf übergeben.
#[derive(Debug, Clone, Copy, Default)]
pub struct XChaCha20Cipher;

impl XChaCha20Cipher {
    /// Verschlüsselt und authentisiert. Allokiert das Ergebnis.
    ///
    /// Liefert einen leeren Vektor, wenn der Klartext die Grenze des Stromchiffres
    /// überschreitet (2^32 Blöcke, 256 GiB je Nonce). `open` weist leere Eingaben zurück,
    /// weil jede gültige Versiegelung mindestens den Tag trägt. Wer den Fehler sehen
    /// will, nutzt [`Self::seal_into`]; auf dem Heißpfad gilt das ohnehin (SPEC §20.5).
    pub fn seal(&self, key: &[u8; 32], nonce: &[u8; 24], aad: &[u8], plain: &[u8]) -> Vec<u8> {
        let mut out = Vec::with_capacity(plain.len().saturating_add(TAG_LEN));
        match self.seal_into(key, nonce, aad, plain, &mut out) {
            Ok(()) => out,
            Err(_) => Vec::new(),
        }
    }

    /// Prüft und entschlüsselt. Bei Fehlschlag kommt `IntegrityViolation` ohne Klartext.
    pub fn open(
        &self,
        key: &[u8; 32],
        nonce: &[u8; 24],
        aad: &[u8],
        sealed: &[u8],
    ) -> Result<Vec<u8>, CryptoError> {
        let mut out = Vec::with_capacity(sealed.len());
        self.open_into(key, nonce, aad, sealed, &mut out)?;
        Ok(out)
    }

    /// Schreibt Chiffretext und Tag nach `out`; `out` wird vorher geleert.
    ///
    /// Allokiert nur, wenn die Kapazität von `out` nicht reicht. Der Heißpfad legt `out`
    /// vorab mit `plain.len() + TAG_LEN` an (SPEC §20.5, geprüft in
    /// `tests/persistence_alloc_budget.rs`).
    pub fn seal_into(
        &self,
        key: &[u8; 32],
        nonce: &[u8; 24],
        aad: &[u8],
        plain: &[u8],
        out: &mut Vec<u8>,
    ) -> Result<(), CryptoError> {
        let cipher = cipher_for(key);
        out.clear();
        out.extend_from_slice(plain);
        match cipher.encrypt_in_place(&XNonce::from(*nonce), aad, out) {
            Ok(()) => Ok(()),
            Err(_) => {
                out.zeroize();
                Err(CryptoError::Encryption(
                    "XChaCha20-Poly1305: Klartext zu lang".into(),
                ))
            }
        }
    }

    /// Prüft den Tag und schreibt den Klartext nach `out`; `out` wird vorher geleert.
    ///
    /// Bei Fehlschlag ist `out` leer und enthält keinen unauthentisierten Klartext.
    /// Allokiert nur, wenn die Kapazität von `out` nicht reicht.
    pub fn open_into(
        &self,
        key: &[u8; 32],
        nonce: &[u8; 24],
        aad: &[u8],
        sealed: &[u8],
        out: &mut Vec<u8>,
    ) -> Result<(), CryptoError> {
        out.clear();
        if sealed.len() < TAG_LEN {
            return Err(CryptoError::IntegrityViolation);
        }
        let cipher = cipher_for(key);
        out.extend_from_slice(sealed);
        match cipher.decrypt_in_place(&XNonce::from(*nonce), aad, out) {
            Ok(()) => Ok(()),
            Err(_) => {
                out.zeroize();
                Err(CryptoError::IntegrityViolation)
            }
        }
    }
}

/// Baut den AEAD-Zustand aus dem Schlüssel. Die lokale Kopie wird danach genullt.
fn cipher_for(key: &[u8; 32]) -> XChaCha20Poly1305 {
    let mut k = *key;
    let cipher = XChaCha20Poly1305::new(&Key::from(k));
    k.zeroize();
    cipher
}
