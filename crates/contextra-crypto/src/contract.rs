
//! # Umwelt-Vertrag für `contextra-crypto`
//!
//! Definiert die minimalen Schnittstellen (Traits) zur Entropiegewinnung und Schlüsselverwaltung (KMS),
//! die von Konsumenten injiziert werden können, sowie eine I/O-freie Referenzimplementierung.

use crate::error::{CryptoError, Result};
use zeroize::Zeroizing;

/// Trait zur Bereitstellung kryptographischer Entropie (Zufallsbytes).
///
/// Ermöglicht die Injektion deterministischer Test-Zufallsquellen oder sicherer CSPRNGs.
pub trait CryptoEntropy: Send + Sync {
    /// Füllt den übergebenen Puffer mit pseudozufälligen Bytes.
    fn fill_bytes(&mut self, dest: &mut [u8]);
}

/// Deterministische Test-Entropiequelle (xorshift) für reproduzierbare Tests.
///
/// **Nicht kryptographisch sicher** und deshalb ausschließlich unter `test`/`test-utils`
/// verfügbar: Produktiver Code darf nie auf vorhersagbare Zufallswerte zurückfallen
/// (Fail-Closed, SPEC §4.3).
#[cfg(any(test, feature = "test-utils"))]
#[derive(Debug, Clone)]
pub struct DeterministicEntropy {
    state: u64,
}

#[cfg(any(test, feature = "test-utils"))]
impl DeterministicEntropy {
    /// Erzeugt eine neue deterministische Entropiequelle mit festem Startwert (Seed).
    pub fn new(seed: u64) -> Self {
        Self {
            state: if seed == 0 { 0x853c49e6748fea9b } else { seed },
        }
    }
}

#[cfg(any(test, feature = "test-utils"))]
impl CryptoEntropy for DeterministicEntropy {
    fn fill_bytes(&mut self, dest: &mut [u8]) {
        for chunk in dest.chunks_mut(8) {
            self.state ^= self.state << 13;
            self.state ^= self.state >> 7;
            self.state ^= self.state << 17;
            let bytes = self.state.to_le_bytes();
            for (dst, src) in chunk.iter_mut().zip(bytes) {
                *dst = src;
            }
        }
    }
}

/// Schnittstelle für Key Management Services (KMS) zur sicheren Schlüsselableitung und Hüllkurven-Verschlüsselung.
pub trait KmsContract: Send + Sync {
    /// Liefert den aktiven Datenverschlüsselungsschlüssel (DEK), genullt bei Freigabe.
    fn get_key(&self) -> Result<Zeroizing<Vec<u8>>>;

    /// Liefert die Kennung (ID) des aktiven Schlüssels.
    fn active_key_id(&self) -> &str;

    /// Leitet einen abgeleiteten Schlüssel anhand einer `key_id` ab.
    fn derive_data_key(&self, key_id: &[u8]) -> Result<Zeroizing<Vec<u8>>>;

    /// Wickelt (verschlüsselt) Daten mit dem angegebenen Schlüssel ein.
    fn wrap_key(&self, key_id: &[u8], plaintext: &[u8]) -> Result<Vec<u8>>;

    /// Entpackt (entschlüsselt) Daten mit dem angegebenen Schlüssel.
    fn unwrap_key(&self, key_id: &[u8], wrapped: &[u8]) -> Result<Zeroizing<Vec<u8>>>;
}

/// I/O-freie In-Memory Referenzimplementierung von [`KmsContract`].
pub struct InMemoryKms {
    active_id: String,
    master_key: Zeroizing<[u8; 32]>,
}

impl InMemoryKms {
    /// Erzeugt eine neue `InMemoryKms`-Instanz mit einem 32-Byte Master-Schlüssel.
    pub fn new(active_id: impl Into<String>, master_key: [u8; 32]) -> Self {
        Self {
            active_id: active_id.into(),
            master_key: Zeroizing::new(master_key),
        }
    }
}

impl KmsContract for InMemoryKms {
    fn get_key(&self) -> Result<Zeroizing<Vec<u8>>> {
        Ok(Zeroizing::new(self.master_key.to_vec()))
    }

    fn active_key_id(&self) -> &str {
        &self.active_id
    }

    fn derive_data_key(&self, key_id: &[u8]) -> Result<Zeroizing<Vec<u8>>> {
        let mut out = Zeroizing::new([0u8; 32]);
        crate::kdf::hkdf_expand(&*self.master_key, key_id, out.as_mut_slice())?;
        Ok(Zeroizing::new(out.to_vec()))
    }

    fn wrap_key(&self, key_id: &[u8], plaintext: &[u8]) -> Result<Vec<u8>> {
        let derived = self.derive_data_key(key_id)?;
        let key_arr =
            Zeroizing::new(<[u8; 32]>::try_from(derived.as_slice()).map_err(|_| {
                CryptoError::InvalidLength("derived data key must be 32 bytes".into())
            })?);
        let dek = crate::dek::Dek::new(*key_arr);
        let doc_tag = [0u8; 16];
        // Zufällige 96-Bit-Nonce aus dem OS-CSPRNG (fehlbar); nie eine feste Nonce.
        crate::aead::seal_record(&dek, doc_tag, key_id, plaintext)
    }

    fn unwrap_key(&self, key_id: &[u8], wrapped: &[u8]) -> Result<Zeroizing<Vec<u8>>> {
        let derived = self.derive_data_key(key_id)?;
        let key_arr =
            Zeroizing::new(<[u8; 32]>::try_from(derived.as_slice()).map_err(|_| {
                CryptoError::InvalidLength("derived data key must be 32 bytes".into())
            })?);
        let dek = crate::dek::Dek::new(*key_arr);
        let opened = crate::aead::open_record(&dek, key_id, wrapped)?;
        Ok(Zeroizing::new(opened.to_vec()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deterministic_entropy() {
        let mut rng1 = DeterministicEntropy::new(42);
        let mut buf1 = [0u8; 32];
        rng1.fill_bytes(&mut buf1);

        let mut rng2 = DeterministicEntropy::new(42);
        let mut buf2 = [0u8; 32];
        rng2.fill_bytes(&mut buf2);

        assert_eq!(buf1, buf2);
        assert_ne!(buf1, [0u8; 32]);
    }

    #[test]
    fn test_in_memory_kms_roundtrip() {
        let kms = InMemoryKms::new("test-key-v1", [0x42u8; 32]);
        assert_eq!(kms.active_key_id(), "test-key-v1");

        let key_id = b"tenant_doc_key";
        let plaintext = b"Sensitive in-memory payload";

        let wrapped = kms.wrap_key(key_id, plaintext).expect("wrap key");
        let unwrapped = kms.unwrap_key(key_id, &wrapped).expect("unwrap key");
        assert_eq!(unwrapped.as_slice(), plaintext);

        let derived_a = kms.derive_data_key(b"key_a").expect("derive a");
        let derived_b = kms.derive_data_key(b"key_b").expect("derive b");
        assert_ne!(derived_a.as_slice(), derived_b.as_slice());
    }
}
