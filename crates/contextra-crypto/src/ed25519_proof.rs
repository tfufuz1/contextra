// FILE-CONTEXT
// ZWECK: Standalone Ed25519 asymmetric signature creation and external verification for deletion proofs (version 3).
// INVARIANTEN: Zero dependency on internal Contextra state in verification path — runnable by third parties with only public key, payload, and signature.
// HOTSPOTS: [sign_deletion_proof_v3, verify_deletion_proof_v3]

#![forbid(unsafe_code)]

//! Asymmetrische Ed25519-Löschbeweis-Signierung und externe Verifikation (Version 3).
//!
//! Ermöglicht Dritten (z. B. Datenschutz-Aufsichtsbehörden) die unabhängige Prüfung
//! von Contextra-Löschbeweisen ohne Zugriff auf interne Systemgeheimnisse.

use rand::RngCore;
use thiserror::Error;

/// Erzeugt den längenpräfixierten Blake3-Hash für eine Liste gelöschter Schlüssel.
pub fn hash_deleted_keys_length_prefixed(deleted_keys: &[Vec<u8>]) -> [u8; 32] {
    crate::deletion_proof::hash_deleted_keys_length_prefixed(deleted_keys)
}

/// Fehlerzustände bei der Verifikation von Ed25519-Löschbeweisen.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum DeletionProofError {
    /// Ungültiger Ed25519 VerifyingKey (32 Bytes).
    #[error("invalid verifying key: {0}")]
    InvalidVerifyingKey(String),

    /// Ungültige Ed25519 Signatur (64 Bytes) oder Fehlschlag bei der Verifikation.
    #[error("invalid signature: {0}")]
    InvalidSignature(String),

    /// Nicht unterstützte Version der Löschbeweis-Signatur.
    #[error("unsupported signature version: {0}")]
    UnsupportedVersion(u8),
}

/// Schlüsselpaar für Ed25519-Löschbeweise.
#[derive(Debug)]
pub struct DeletionProofKeyPair {
    /// Privater Signierschlüssel.
    pub signing_key: ed25519_dalek::SigningKey,
    /// Öffentlicher Verifikationsschlüssel.
    pub verifying_key: ed25519_dalek::VerifyingKey,
}

impl DeletionProofKeyPair {
    /// Generiert ein neues Ed25519-Schlüsselpaar unter Nutzung eines injizierten Zufallsgenerators.
    pub fn generate(rng: &mut impl RngCore) -> Self {
        let mut secret_bytes = [0u8; 32];
        rng.fill_bytes(&mut secret_bytes);
        let signing_key = ed25519_dalek::SigningKey::from_bytes(&secret_bytes);
        let verifying_key = signing_key.verifying_key();
        Self {
            signing_key,
            verifying_key,
        }
    }

    /// Gibt die 32-Byte-Repräsentation des öffentlichen Verifikationsschlüssels zurück.
    pub fn verifying_key_bytes(&self) -> [u8; 32] {
        self.verifying_key.to_bytes()
    }

    /// Gibt eine Referenz auf den Signierschlüssel zurück.
    pub fn signing_key(&self) -> &ed25519_dalek::SigningKey {
        &self.signing_key
    }

    /// Gibt eine Referenz auf den Verifikationsschlüssel zurück.
    pub fn verifying_key(&self) -> &ed25519_dalek::VerifyingKey {
        &self.verifying_key
    }
}

/// Signiert den kanonischen Byte-Payload eines Version-3-Löschbeweises mit Ed25519.
///
/// Gibt eine 64-Byte Ed25519-Signatur zurück.
pub fn sign_deletion_proof_v3(keypair: &DeletionProofKeyPair, proof_payload: &[u8]) -> [u8; 64] {
    use ed25519_dalek::Signer;
    let signature = keypair.signing_key.sign(proof_payload);
    signature.to_bytes()
}

/// Verifiziert eine Ed25519-Signatur eines Version-3-Löschbeweises rein funktional.
///
/// Diese Funktion benötigt keine Contextra-internen Zustände oder Geheimnisse.
/// Dritte (z. B. Datenschutzbeauftragte) können einen Beweis mit nur `verifying_key_bytes`,
/// `proof_payload` und `signature` verifizieren.
///
/// # Errors
/// - [`DeletionProofError::InvalidVerifyingKey`]: Wenn der Verifikationsschlüssel ungültig ist.
/// - [`DeletionProofError::InvalidSignature`]: Wenn die Signatur nicht zum Payload oder Schlüssel passt.
pub fn verify_deletion_proof_v3(
    verifying_key_bytes: &[u8; 32],
    proof_payload: &[u8],
    signature: &[u8; 64],
) -> Result<(), DeletionProofError> {
    use ed25519_dalek::{Signature, Verifier, VerifyingKey};

    let verifying_key = VerifyingKey::from_bytes(verifying_key_bytes)
        .map_err(|e| DeletionProofError::InvalidVerifyingKey(e.to_string()))?;

    let sig = Signature::from_bytes(signature);

    verifying_key
        .verify(proof_payload, &sig)
        .map_err(|e| DeletionProofError::InvalidSignature(e.to_string()))
}

/// Vorgeschlagenes `SignatureVersion`-Enum für spätere Integration in `deletion_proof.rs`.
///
/// TODO (Separate PR): In `deletion_proof.rs` das `pub signature_version: u8` Feld
/// durch dieses Enum ersetzen oder konvertieren.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignatureVersion {
    /// HMAC-SHA256 legacy scope-only signature (version 1)
    V1 = 1,
    /// HMAC-SHA256 scope + layer coverage signature (version 2)
    V2 = 2,
    /// Ed25519 asymmetric signature with length-prefixed key hashes (version 3)
    V3 = 3,
}

impl SignatureVersion {
    /// Gibt den `u8`-Wert der Version zurück.
    pub fn as_u8(&self) -> u8 {
        *self as u8
    }
}

impl From<SignatureVersion> for u8 {
    fn from(v: SignatureVersion) -> Self {
        v.as_u8()
    }
}

impl TryFrom<u8> for SignatureVersion {
    type Error = DeletionProofError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(SignatureVersion::V1),
            2 => Ok(SignatureVersion::V2),
            3 => Ok(SignatureVersion::V3),
            v => Err(DeletionProofError::UnsupportedVersion(v)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::StdRng;
    use rand::SeedableRng;

    #[test]
    fn test_keypair_generation_and_bytes() {
        let mut rng = StdRng::seed_from_u64(42);
        let keypair = DeletionProofKeyPair::generate(&mut rng);
        let vk_bytes = keypair.verifying_key_bytes();

        assert_eq!(vk_bytes, keypair.verifying_key.to_bytes());
        assert_eq!(keypair.signing_key().verifying_key(), *keypair.verifying_key());
    }

    #[test]
    fn test_v3_sign_and_verify_roundtrip() {
        let mut rng = StdRng::seed_from_u64(100);
        let keypair = DeletionProofKeyPair::generate(&mut rng);
        let payload = b"canonical_deletion_proof_payload_v3";

        let signature = sign_deletion_proof_v3(&keypair, payload);
        let vk_bytes = keypair.verifying_key_bytes();

        let res = verify_deletion_proof_v3(&vk_bytes, payload, &signature);
        assert!(res.is_ok());
    }

    #[test]
    fn test_v3_tampered_payload_fails_verification() {
        let mut rng = StdRng::seed_from_u64(101);
        let keypair = DeletionProofKeyPair::generate(&mut rng);
        let payload = b"canonical_deletion_proof_payload_v3";

        let signature = sign_deletion_proof_v3(&keypair, payload);
        let vk_bytes = keypair.verifying_key_bytes();

        let tampered_payload = b"canonical_deletion_proof_payload_v3_tampered";
        let res = verify_deletion_proof_v3(&vk_bytes, tampered_payload, &signature);
        assert!(matches!(res, Err(DeletionProofError::InvalidSignature(_))));
    }

    #[test]
    fn test_signature_version_conversions() {
        assert_eq!(SignatureVersion::V1.as_u8(), 1);
        assert_eq!(SignatureVersion::V2.as_u8(), 2);
        assert_eq!(SignatureVersion::V3.as_u8(), 3);

        assert_eq!(SignatureVersion::try_from(1).unwrap(), SignatureVersion::V1);
        assert_eq!(SignatureVersion::try_from(2).unwrap(), SignatureVersion::V2);
        assert_eq!(SignatureVersion::try_from(3).unwrap(), SignatureVersion::V3);

        assert!(matches!(
            SignatureVersion::try_from(99),
            Err(DeletionProofError::UnsupportedVersion(99))
        ));
    }
}
