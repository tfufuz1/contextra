//! Ed25519-Signatur (RFC 8032) für Berichte und Checkpoints (Spec 4.1, 4.7).
//!
//! Die signierte Nachricht ist `domain ‖ msg`. Das Domänenpräfix trennt Berichte von
//! Checkpoints. Die Präfixe sind fest und nicht Präfix voneinander.

use crate::error::CryptoError;
use crate::key::KEY_LEN;
use acta_base::CryptoRng;
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use zeroize::Zeroizing;

/// Länge eines öffentlichen Schlüssels in Byte.
pub const PUBLIC_KEY_LEN: usize = 32;
/// Länge einer Signatur in Byte.
pub const SIGNATURE_LEN: usize = 64;

/// Notar-Signaturschlüssel. Kein `Clone`, kein `Debug`, kein `Serialize`.
pub struct NotarySigner {
    key: SigningKey,
}

impl NotarySigner {
    /// Erzeugt einen Schlüssel aus dem Zufall des Ports.
    ///
    /// # Errors
    /// [`CryptoError::Entropy`], wenn die Quelle versagt.
    pub fn generate(rng: &dyn CryptoRng) -> Result<Self, CryptoError> {
        let mut seed = Zeroizing::new([0_u8; KEY_LEN]);
        rng.try_fill(seed.as_mut_slice())?;
        Ok(Self::from_seed(*seed))
    }

    /// Schlüssel aus einem 32-Byte-Seed (etwa aus dem Anker).
    #[must_use]
    pub fn from_seed(seed: [u8; KEY_LEN]) -> Self {
        Self {
            key: SigningKey::from_bytes(&seed),
        }
    }

    /// Öffentlicher Schlüssel.
    #[must_use]
    pub fn public_key(&self) -> [u8; PUBLIC_KEY_LEN] {
        self.key.verifying_key().to_bytes()
    }

    /// Signiert `domain ‖ msg`.
    #[must_use]
    pub fn sign(&self, domain: &[u8], msg: &[u8]) -> [u8; SIGNATURE_LEN] {
        let message = [domain, msg].concat();
        self.key.sign(&message).to_bytes()
    }
}

/// Prüft eine Signatur über `domain ‖ msg`.
///
/// # Errors
/// [`CryptoError::Malformed`] bei ungültigem öffentlichem Schlüssel,
/// [`CryptoError::SignatureInvalid`] bei nicht passender Signatur.
pub fn verify(
    public: &[u8; PUBLIC_KEY_LEN],
    domain: &[u8],
    msg: &[u8],
    sig: &[u8; SIGNATURE_LEN],
) -> Result<(), CryptoError> {
    let key = VerifyingKey::from_bytes(public).map_err(|_| CryptoError::Malformed)?;
    let signature = Signature::from_bytes(sig);
    let message = [domain, msg].concat();
    key.verify(&message, &signature)
        .map_err(|_| CryptoError::SignatureInvalid)
}
