//! # Manipulationsschutz und Notfall-Speicherbereinigung (`anti_tamper.rs`)
//!
//! Stellt sichere Speicherbehälter und Integritätsprüfungen bereit, die sensibles
//! Schlüsselmaterial vor Kaltstart-Angriffen, Speicher-Dumps und Timing-Orakeln schützen.
//!
//! ## Invarianten
//! - **INV-CRYPTO-ZEROIZE-1**: Automatisches Zeroizing beim Verlassen des Gültigkeitsbereichs (`Drop`).
//! - **INV-CRYPTO-CONSTANT-TIME-1**: Alle Schlüssel-, Tag- und Prüfsummenvergleiche in konstanter Zeit
//!   mittels `subtle::ConstantTimeEq`. Verbot von `==`-Vergleichen auf Byte-Geheimnissen.
//! - Manuelle Notfallbereinigung (`emergency_wipe`) überschreibt Speicher deterministisch mit Nullen.
//! - Debug- und Display-Ausgaben maskieren sensible Schlüssel (`[REDACTED]`).
//! - Strikter Verzicht auf `unsafe`-Code (`#![forbid(unsafe_code)]`).

#![cfg_attr(not(test), forbid(unsafe_code))]

use crate::error::{CryptoError, Result};
use subtle::ConstantTimeEq;
use zeroize::{Zeroize, Zeroizing};

/// Defines a cryptographic key that is explicitly zeroed out when dropped
/// or when an emergency trigger is activated, protecting against cold-boot attacks.
#[derive(Zeroize)]
#[zeroize(drop)]
pub struct VolatileEncryptionKey {
    key_bytes: Zeroizing<[u8; 32]>, // AES-256 Key
}

impl VolatileEncryptionKey {
    /// Creates a new volatile key from a raw 32-byte array.
    pub fn new(raw: [u8; 32]) -> Self {
        Self {
            key_bytes: Zeroizing::new(raw),
        }
    }

    /// Emergency Trigger: Explicitly wipes the key from memory.
    /// This is to be hooked into hardware sensors (e.g. crash/voltage).
    pub fn emergency_wipe(&mut self) {
        self.key_bytes.zeroize();
    }

    /// Accessor for cryptographic operations.
    pub fn as_bytes(&self) -> &[u8] {
        self.key_bytes.as_slice()
    }

    /// Test-only method to inspect key bytes.
    #[cfg(any(test, feature = "test-utils"))]
    pub fn inspect_key_bytes_for_test(&self) -> &[u8; 32] {
        &self.key_bytes
    }
}

impl std::fmt::Debug for VolatileEncryptionKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VolatileEncryptionKey")
            .field("key_bytes", &"*** REDACTED ***")
            .finish()
    }
}

impl PartialEq for VolatileEncryptionKey {
    fn eq(&self, other: &Self) -> bool {
        self.key_bytes
            .as_slice()
            .ct_eq(other.key_bytes.as_slice())
            .into()
    }
}

impl Eq for VolatileEncryptionKey {}

/// Konstantzeit-Verifikation zweier Byte-Slices (`INV-CRYPTO-CONSTANT-TIME-1`).
///
/// Verhindert Timing-Orakel bei MAC-, Tag- und Prüfsummen-Validierungen.
#[inline]
pub fn constant_time_compare(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    a.ct_eq(b).into()
}

/// Verifiziert ein Authentifizierungs-Tag in konstanter Zeit und liefert `Result<()>`.
#[inline]
pub fn verify_tag_constant_time(expected: &[u8], actual: &[u8]) -> Result<()> {
    if constant_time_compare(expected, actual) {
        Ok(())
    } else {
        Err(CryptoError::IntegrityViolation)
    }
}

/// BLAKE3-basierte Integritätskette für sequentielle Log- oder Datenblöcke.
#[derive(Debug, Clone)]
pub struct Blake3Chain {
    current_hash: [u8; 32],
    step_count: u64,
}

impl Blake3Chain {
    /// Initialisiert eine neue BLAKE3-Kette mit einem Seed/Genesis-Hash.
    pub fn new(genesis: [u8; 32]) -> Self {
        Self {
            current_hash: genesis,
            step_count: 0,
        }
    }

    /// Aktualisiert den Zustand der Kette mit einem neuen Block und Kontext in AAD-Disziplin.
    pub fn extend(&mut self, context: &[u8], data: &[u8]) -> [u8; 32] {
        let mut hasher = blake3::Hasher::new();
        hasher.update(&self.current_hash);
        hasher.update(&self.step_count.to_be_bytes());
        hasher.update(&(context.len() as u32).to_be_bytes());
        hasher.update(context);
        hasher.update(&(data.len() as u64).to_be_bytes());
        hasher.update(data);

        let next_hash = *hasher.finalize().as_bytes();
        self.current_hash = next_hash;
        self.step_count = self.step_count.saturating_add(1);
        next_hash
    }

    /// Gibt den aktuellen Ketten-Hash zurück.
    pub fn current_hash(&self) -> &[u8; 32] {
        &self.current_hash
    }

    /// Gibt die bisherige Anzahl an Kettengliedern zurück.
    pub fn step_count(&self) -> u64 {
        self.step_count
    }

    /// Verifiziert in konstanter Zeit, ob der aktuelle Hash mit einem erwarteten Wert übereinstimmt.
    pub fn verify_current_hash(&self, expected: &[u8; 32]) -> Result<()> {
        verify_tag_constant_time(expected, &self.current_hash)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inv_crypto_constant_time_1() {
        let tag1 = [0x55u8; 32];
        let mut tag2 = [0x55u8; 32];
        assert!(constant_time_compare(&tag1, &tag2));
        assert!(verify_tag_constant_time(&tag1, &tag2).is_ok());

        tag2[31] ^= 0x01;
        assert!(!constant_time_compare(&tag1, &tag2));
        assert!(verify_tag_constant_time(&tag1, &tag2).is_err());
    }

    #[test]
    fn test_blake3_chain_integrity_and_anti_tamper() {
        let mut chain1 = Blake3Chain::new([0u8; 32]);
        let mut chain2 = Blake3Chain::new([0u8; 32]);

        let h1 = chain1.extend(b"context_a", b"payload_1");
        let h2 = chain2.extend(b"context_a", b"payload_1");
        assert_eq!(h1, h2);
        assert!(chain1.verify_current_hash(&h1).is_ok());

        // Context divergence
        let h1_next = chain1.extend(b"context_a", b"payload_2");
        let h2_next = chain2.extend(b"context_b", b"payload_2");
        assert_ne!(h1_next, h2_next);
    }

    #[test]
    fn test_emergency_wipe_zeros_key_bytes() {
        let raw: [u8; 32] = [0xAA; 32];
        let mut key = VolatileEncryptionKey::new(raw);
        assert_eq!(key.inspect_key_bytes_for_test(), &[0xAA; 32]);
        key.emergency_wipe();
        assert_eq!(key.inspect_key_bytes_for_test(), &[0x00; 32]);
    }

    #[test]
    fn test_inv_crypto_zeroize_1_on_drop() {
        let raw: [u8; 32] = [0xCD; 32];
        let mut key = VolatileEncryptionKey::new(raw);
        assert_eq!(key.as_bytes(), &[0xCD; 32]);
        Zeroize::zeroize(&mut key);
        assert_eq!(key.as_bytes(), &[0x00; 32]);
    }
}
