//! Secret types with zeroize-on-drop and constant-time equality.

use subtle::ConstantTimeEq;
use zeroize::{Zeroize, ZeroizeOnDrop};

/// Fixed-size secret key that is wiped on drop and never printed.
#[derive(Zeroize, ZeroizeOnDrop)]
pub struct SecretKey<const N: usize> {
    bytes: [u8; N],
}

impl<const N: usize> SecretKey<N> {
    /// Creates a new secret key wrapping raw bytes.
    pub fn new(bytes: [u8; N]) -> Self {
        Self { bytes }
    }

    /// Exposes a reference to the secret bytes.
    pub fn as_bytes(&self) -> &[u8; N] {
        &self.bytes
    }
}

impl<const N: usize> std::fmt::Debug for SecretKey<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SecretKey<{}>([REDACTED])", N)
    }
}

impl<const N: usize> ConstantTimeEq for SecretKey<N> {
    fn ct_eq(&self, other: &Self) -> subtle::Choice {
        self.bytes.ct_eq(&other.bytes)
    }
}

/// Secret passphrase buffer.
#[derive(Zeroize, ZeroizeOnDrop)]
pub struct Secret {
    bytes: Vec<u8>,
}

impl Secret {
    /// Creates a new secret from a byte slice.
    pub fn from_slice(slice: &[u8]) -> Self {
        Self {
            bytes: slice.to_vec(),
        }
    }

    /// Exposes a slice to the secret bytes.
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }
}

impl std::fmt::Debug for Secret {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Secret([REDACTED])")
    }
}
