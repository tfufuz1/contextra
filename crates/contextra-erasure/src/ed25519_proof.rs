//! # Ed25519-Signaturpfade für Beweise (`ed25519_proof.rs`)
//!
//! Asymmetrische Signaturerstellung und -prüfung mittels Ed25519 nach RFC 8032.
//!
//! ## Invarianten
//! - Verwendet `ed25519_dalek::VerifyingKey::verify_strict`, um Signatur-Malleability auszuschließen.
//! - Private Schlüssel verbleiben ausschließlich im Speicher der signierenden Instanz.

// FILE-CONTEXT
// ZWECK: Standalone Ed25519 asymmetric signature creation and external verification for deletion proofs (version 3).
// INVARIANTEN: Zero dependency on internal Contextra state in verification path — runnable by third parties with only public key, payload, and signature.
// HOTSPOTS: [sign_deletion_proof_v3, verify_deletion_proof_v3]

#![forbid(unsafe_code)]

//!
//! Asymmetrische Ed25519-Attestationsbeweis-Signierung und externe Verifikation (Version 3).
//!
//! Ermöglicht Dritten (z. B. Datenschutz-Aufsichtsbehörden) die unabhängige Prüfung
//! von Contextra-Attestationsbeweisen ohne Zugriff auf interne Systemgeheimnisse.

use rand::{CryptoRng, RngCore};
use thiserror::Error;
use zeroize::ZeroizeOnDrop;

/// Berechnet den BLAKE3-Hash über längenpräfixierte gelöschte Schlüssel.
pub fn hash_deleted_keys_length_prefixed(keys: &[Vec<u8>]) -> [u8; 32] {
    let mut hasher = blake3::Hasher::new();
    for key in keys {
        let len_bytes = u32::try_from(key.len()).unwrap_or(u32::MAX).to_le_bytes();
        hasher.update(&len_bytes);
        hasher.update(key);
    }
    *hasher.finalize().as_bytes()
}

/// Fehlerzustände bei der Verifikation von Ed25519-Attestationsbeweisen.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum DeletionProofError {
    /// Ungültiger Ed25519 VerifyingKey (32 Bytes).
    #[error("invalid verifying key: {0}")]
    InvalidVerifyingKey(String),

    /// Ungültige Ed25519 Signatur (64 Bytes) oder Fehlschlag bei der Verifikation.
    #[error("invalid signature: {0}")]
    InvalidSignature(String),

    /// Nicht unterstützte Version der Attestationsbeweis-Signatur.
    #[error("Unsupported DeletionProof signature_version: {0}")]
    UnsupportedVersion(u8),
}

/// Schlüsselpaar für Ed25519-Attestationsbeweise.
///
/// Signierschlüssel wird bei Drop automatisch zeroisiert (`ZeroizeOnDrop`).
#[derive(Debug, ZeroizeOnDrop)]
pub struct DeletionProofKeyPair {
    /// Privater Signierschlüssel.
    pub signing_key: ed25519_dalek::SigningKey,
    /// Öffentlicher Verifikationsschlüssel.
    #[zeroize(skip)]
    pub verifying_key: ed25519_dalek::VerifyingKey,
}

impl DeletionProofKeyPair {
    /// Generiert ein neues Ed25519-Schlüsselpaar unter Nutzung des System-RNGs (`OsRng`).
    pub fn generate() -> Self {
        let mut rng = rand::rngs::OsRng;
        Self::generate_with_rng(&mut rng)
    }

    /// Generiert ein neues Ed25519-Schlüsselpaar unter Nutzung eines beliebigen RNGs.
    pub fn generate_with_rng<R: RngCore + CryptoRng>(rng: &mut R) -> Self {
        let mut secret_bytes = [0u8; 32];
        rng.fill_bytes(&mut secret_bytes);
        let signing_key = ed25519_dalek::SigningKey::from_bytes(&secret_bytes);
        let verifying_key = signing_key.verifying_key();
        Self {
            signing_key,
            verifying_key,
        }
    }

    /// Erzeugt ein `DeletionProofKeyPair` aus einem bestehenden Signierschlüssel.
    pub fn from_signing_key(signing_key: ed25519_dalek::SigningKey) -> Self {
        let verifying_key = signing_key.verifying_key();
        Self {
            signing_key,
            verifying_key,
        }
    }

    /// Gibt die Bytes des öffentlichen Verifikationsschlüssels (32 Bytes) zurück.
    pub fn verifying_key_bytes(&self) -> [u8; 32] {
        self.verifying_key.to_bytes()
    }

    /// Gibt eine Referenz auf den Signierschlüssel zurück.
    pub fn signing_key(&self) -> &ed25519_dalek::SigningKey {
        &self.signing_key
    }
}

/// Signiert den Payload eines DeletionProofs unter Nutzung von Ed25519 (Version 3).
pub fn sign_deletion_proof_v3(keypair: &DeletionProofKeyPair, proof_payload: &[u8]) -> [u8; 64] {
    use ed25519_dalek::Signer;
    let signature = keypair.signing_key.sign(proof_payload);
    signature.to_bytes()
}

/// Verifiziert die Ed25519-Signatur eines DeletionProofs unabhängig von internem Contextra-State (Version 3).
pub fn verify_deletion_proof_v3(
    verifying_key_bytes: &[u8; 32],
    proof_payload: &[u8],
    signature_bytes: &[u8; 64],
) -> Result<(), DeletionProofError> {


    let verifying_key = ed25519_dalek::VerifyingKey::from_bytes(verifying_key_bytes)
        .map_err(|e| DeletionProofError::InvalidVerifyingKey(e.to_string()))?;

    let signature = ed25519_dalek::Signature::from_bytes(signature_bytes);

    verifying_key
        .verify_strict(proof_payload, &signature)
        .map_err(|e| DeletionProofError::InvalidSignature(e.to_string()))
}

/// `SignatureVersion`-Enum für typsichere Versionierung von DeletionProof-Signaturen.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum SignatureVersion {
    /// Version 1: HMAC-SHA256 über Scope, Key-Hash und TxId.
    V1 = 1,
    /// Version 2: HMAC-SHA256 mit Bindung an Covered Layers, Excluded Scopes und WAL-Receipt.
    V2 = 2,
    /// Version 3: Ed25519 asymmetrische Signatur mit längenpräfixierten Key-Hashes.
    V3 = 3,
    /// Version 5: Canonical Report Format.
    V5 = 5,
}

impl SignatureVersion {
    /// Liefert den numerischen Wert als `u8`.
    pub const fn as_u8(self) -> u8 {
        self as u8
    }
}

impl From<SignatureVersion> for u8 {
    fn from(v: SignatureVersion) -> Self {
        v.as_u8()
    }
}

impl TryFrom<u8> for SignatureVersion {
    type Error = DeletionProofError;

    fn try_from(v: u8) -> Result<Self, Self::Error> {
        match v {
            1 => Ok(Self::V1),
            2 => Ok(Self::V2),
            3 => Ok(Self::V3),
            5 => Ok(Self::V5),
            v => Err(DeletionProofError::UnsupportedVersion(v)),
        }
    }
}
