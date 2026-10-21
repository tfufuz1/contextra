//! Cryptographic secret byte container with automatic zeroization and type-level protections.

use crate::rng::{EntropyRng, RngError};
use zeroize::Zeroizing;

/// Fixed-size container for sensitive cryptographic bytes (keys, salts, seeds).
///
/// Ensures memory zeroization upon drop, redacts `Debug` formatting, and forbids `Clone` or `Copy`.
pub struct SecretBytes<const N: usize>(Zeroizing<[u8; N]>);

impl<const N: usize> SecretBytes<N> {
    /// Generates new secret bytes using the given cryptographic entropy source.
    ///
    /// Calls `rng.fill_secret(&mut buf)` exclusively to ensure non-CSPRNG sources are rejected.
    pub fn generate(rng: &dyn EntropyRng) -> Result<Self, RngError> {
        let mut buf = [0u8; N];
        rng.fill_secret(&mut buf)?;
        Ok(Self(Zeroizing::new(buf)))
    }

    /// Exposes a reference to the underlying secret bytes array.
    pub fn expose(&self) -> &[u8; N] {
        &self.0
    }

    /// Constant-time equality check between two `SecretBytes` instances.
    pub fn ct_eq(&self, other: &Self) -> bool {
        let a = self.expose().as_slice();
        let b = other.expose().as_slice();
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
        write!(f, "SecretBytes<{}>((redacted))", N)
    }
}
