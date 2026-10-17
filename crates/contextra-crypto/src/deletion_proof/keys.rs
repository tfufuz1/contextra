//! Key pair, key source traits, and verification key definitions for deletion proofs.

use crate::ed25519_proof::SignatureVersion;
use crate::error::Result;
use rand::RngCore;
use zeroize::Zeroizing;

/// Trait defining a secure source for deletion proof signing keys (CSPRNG or KMS/HSM handles).
pub trait ProofKeySource: Send + Sync {
    /// Generates or provisions a new signing keypair via this key source.
    fn generate_keypair(&self) -> Result<DeletionProofKeyPair>;

    /// Loads the active signing keypair from this key source.
    fn load_keypair(&self) -> Result<DeletionProofKeyPair>;

    /// Returns the 32-byte public verification key associated with the active key from this source.
    fn public_key(&self) -> Result<[u8; 32]>;

    /// Returns the 32-byte BLAKE3 key ID for the active key from this source.
    fn key_id(&self) -> Result<[u8; 32]>;
}

/// Local in-memory CSPRNG key source for generating and holding deletion proof keys.
pub struct InMemoryCsprngKeySource {
    keypair: DeletionProofKeyPair,
}

impl std::fmt::Debug for InMemoryCsprngKeySource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InMemoryCsprngKeySource")
            .field("keypair", &self.keypair)
            .finish()
    }
}

impl InMemoryCsprngKeySource {
    /// Generates a new key source backed by system CSPRNG (`OsRng`).
    pub fn new() -> Self {
        Self {
            keypair: DeletionProofKeyPair::generate(),
        }
    }

    /// Creates a key source from an explicit keypair.
    pub fn from_keypair(keypair: DeletionProofKeyPair) -> Self {
        Self { keypair }
    }
}

impl Default for InMemoryCsprngKeySource {
    fn default() -> Self {
        Self::new()
    }
}

impl ProofKeySource for InMemoryCsprngKeySource {
    fn generate_keypair(&self) -> Result<DeletionProofKeyPair> {
        let mut secret = Zeroizing::new([0u8; 32]);
        rand::rngs::OsRng.fill_bytes(secret.as_mut());
        let signing_key = ed25519_dalek::SigningKey::from_bytes(&*secret);
        let verifying_key = signing_key.verifying_key();
        Ok(DeletionProofKeyPair {
            signing_key,
            verifying_key,
        })
    }

    fn load_keypair(&self) -> Result<DeletionProofKeyPair> {
        let sk_bytes = self.keypair.signing_key.to_bytes();
        let signing_key = ed25519_dalek::SigningKey::from_bytes(&sk_bytes);
        let verifying_key = signing_key.verifying_key();
        Ok(DeletionProofKeyPair {
            signing_key,
            verifying_key,
        })
    }

    fn public_key(&self) -> Result<[u8; 32]> {
        Ok(self.keypair.verifying_key_bytes())
    }

    fn key_id(&self) -> Result<[u8; 32]> {
        let pub_key = self.keypair.verifying_key_bytes();
        Ok(*blake3::hash(&pub_key).as_bytes())
    }
}

/// KeyPair for Ed25519 signing and verification of DeletionProofs (version 3).
pub struct DeletionProofKeyPair {
    pub signing_key: ed25519_dalek::SigningKey,
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
        let zero_bytes = Zeroizing::new([0u8; 32]);
        self.signing_key = ed25519_dalek::SigningKey::from_bytes(&*zero_bytes);
    }

    /// Generates a new random Ed25519 keypair using OsRng.
    pub fn generate() -> Self {
        let mut rng = rand::rngs::OsRng;
        Self::generate_with_rng(&mut rng)
    }

    /// Generates a new Ed25519 keypair using an explicit random number generator.
    pub fn generate_with_rng(rng: &mut impl rand::RngCore) -> Self {
        let mut secret_bytes = Zeroizing::new([0u8; 32]);
        rng.fill_bytes(secret_bytes.as_mut());
        let signing_key = ed25519_dalek::SigningKey::from_bytes(&*secret_bytes);
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

    /// Returns a reference to the verifying key.
    pub fn verifying_key(&self) -> &ed25519_dalek::VerifyingKey {
        &self.verifying_key
    }
}

impl Drop for DeletionProofKeyPair {
    fn drop(&mut self) {
        self.zeroize();
    }
}

/// Key parameter for verification coupled with its signature version.
#[derive(Debug, Clone, Copy)]
pub enum VerificationKey<'a> {
    /// Key for HMAC-SHA256 signature verification (version 1).
    HmacV1(&'a [u8]),
    /// Key for HMAC-SHA256 signature verification (version 2).
    HmacV2(&'a [u8]),
    /// Key for HMAC-SHA256 signature verification.
    Hmac(&'a [u8]),
    /// Key for Ed25519 signature verification (version 3).
    Ed25519(&'a ed25519_dalek::VerifyingKey),
}

impl<'a> VerificationKey<'a> {
    /// Returns the associated signature version for this verification key.
    pub fn version(&self) -> SignatureVersion {
        match self {
            Self::HmacV1(_) => SignatureVersion::V1,
            Self::HmacV2(_) | Self::Hmac(_) => SignatureVersion::V2,
            Self::Ed25519(_) => SignatureVersion::V3,
        }
    }
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
