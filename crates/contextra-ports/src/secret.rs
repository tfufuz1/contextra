//! Cryptographic secret byte container with automatic zeroization and type-level protections.

use crate::rng::{CryptoRng, RngError};
use zeroize::Zeroizing;

/// Fixed-size container for sensitive cryptographic bytes (keys, salts, seeds).
///
/// Ensures memory zeroization upon drop, redacts `Debug` formatting, and forbids `Clone` or `Copy`.
pub struct SecretBytes<const N: usize>(Zeroizing<[u8; N]>);

impl<const N: usize> SecretBytes<N> {
    /// Constructs a `SecretBytes` container from an existing byte array.
    pub fn new(bytes: [u8; N]) -> Self {
        Self(Zeroizing::new(bytes))
    }

    /// Alias for `new` to construct from an array.
    pub fn from_bytes(bytes: [u8; N]) -> Self {
        Self::new(bytes)
    }

    /// Generates new secret bytes using the given CSPRNG source.
    ///
    /// Calls `rng.fill_secret(&mut buf)` exclusively to ensure non-CSPRNG sources are rejected.
    pub fn generate(rng: &dyn CryptoRng) -> Result<Self, RngError> {
        let mut buf = [0u8; N];
        rng.fill_secret(&mut buf)?;
        Ok(Self(Zeroizing::new(buf)))
    }

    /// Exposes a reference to the underlying secret bytes array.
    pub fn expose(&self) -> &[u8; N] {
        &self.0
    }

    /// Exposes a reference to the underlying secret bytes array.
    pub fn expose_secret(&self) -> &[u8; N] {
        &self.0
    }

    /// Returns a slice over the secret bytes.
    pub fn as_slice(&self) -> &[u8] {
        self.0.as_slice()
    }

    /// Consumes `Self` and returns the inner `Zeroizing<[u8; N]>`.
    pub fn into_inner(self) -> Zeroizing<[u8; N]> {
        self.0
    }

    /// Zeroizes the secret bytes in place.
    pub fn zeroize(&mut self) {
        use zeroize::Zeroize;
        self.0.zeroize();
    }

    /// Constant-time equality check between two `SecretBytes` instances.
    pub fn ct_eq(&self, other: &Self) -> bool {
        let a = self.as_slice();
        let b = other.as_slice();
        if a.len() != b.len() {
            return false;
        }
        let mut res = 0u8;
        for (x, y) in a.iter().zip(b.iter()) {
            res |= x ^ y;
        }
        res == 0
    }
}

impl<const N: usize> std::fmt::Debug for SecretBytes<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SecretBytes<{}>([REDACTED])", N)
    }
}

impl<const N: usize> std::fmt::Display for SecretBytes<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[REDACTED]")
    }
}
