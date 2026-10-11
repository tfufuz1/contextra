//! Key pair and verification key definitions for deletion proofs.

/// KeyPair for Ed25519 signing and verification of DeletionProofs (version 3).
pub struct DeletionProofKeyPair {
    pub(super) signing_key: ed25519_dalek::SigningKey,
    pub verifying_key: ed25519_dalek::VerifyingKey,
}

impl std::fmt::Debug for DeletionProofKeyPair {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DeletionProofKeyPair")
            .field("signing_key", &"***REDACTED***")
            .field("verifying_key", &self.verifying_key)
            .finish()
    }
}

impl DeletionProofKeyPair {
    /// Explicitly zeroizes the secret signing key bytes.
    pub fn zeroize(&mut self) {
        self.signing_key = ed25519_dalek::SigningKey::from_bytes(&[0u8; 32]);
    }
    /// Generates a new random Ed25519 keypair using OsRng.
    pub fn generate() -> Self {
        let mut rng = rand::rngs::OsRng;
        let signing_key = ed25519_dalek::SigningKey::generate(&mut rng);
        let verifying_key = signing_key.verifying_key();
        Self {
            signing_key,
            verifying_key,
        }
    }

    /// Returns the 32-byte representation of the verifying public key.
    pub fn verifying_key_bytes(&self) -> [u8; 32] {
        self.verifying_key.to_bytes()
    }

    /// Returns a reference to the signing key.
    pub fn signing_key(&self) -> &ed25519_dalek::SigningKey {
        &self.signing_key
    }
}

impl Drop for DeletionProofKeyPair {
    fn drop(&mut self) {
        self.zeroize();
    }
}

/// Key parameter for verification (either HMAC-SHA256 byte slice or Ed25519 VerifyingKey).
#[derive(Debug, Clone, Copy)]
pub enum VerificationKey<'a> {
    /// Key for HMAC-SHA256 signature verification (version 1 and 2).
    Hmac(&'a [u8]),
    /// Key for Ed25519 signature verification (version 3).
    Ed25519(&'a ed25519_dalek::VerifyingKey),
}

impl<'a> From<&'a [u8]> for VerificationKey<'a> {
    fn from(key: &'a [u8]) -> Self {
        VerificationKey::Hmac(key)
    }
}

impl<'a, const N: usize> From<&'a [u8; N]> for VerificationKey<'a> {
    fn from(key: &'a [u8; N]) -> Self {
        VerificationKey::Hmac(key.as_slice())
    }
}

impl<'a> From<&'a Vec<u8>> for VerificationKey<'a> {
    fn from(key: &'a Vec<u8>) -> Self {
        VerificationKey::Hmac(key.as_slice())
    }
}

impl<'a> From<&'a ed25519_dalek::VerifyingKey> for VerificationKey<'a> {
    fn from(key: &'a ed25519_dalek::VerifyingKey) -> Self {
        VerificationKey::Ed25519(key)
    }
}
