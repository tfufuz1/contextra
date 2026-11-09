//! Secret key containers with zeroize-on-drop, constant-time equality, and no plaintext formatting.

use subtle::ConstantTimeEq;
use zeroize::{Zeroize, ZeroizeOnDrop};

/// Fixed-size secret key that is wiped on drop and never printed.
///
/// Implements `INV-CRYPTO-ZEROIZE-1` and `INV-CRYPTO-CONSTANT-TIME-1`.
#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub struct SecretKey<const N: usize> {
    bytes: [u8; N],
}

impl<const N: usize> SecretKey<N> {
    pub fn new(bytes: [u8; N]) -> Self {
        Self { bytes }
    }

    pub fn expose_secret(&self) -> &[u8; N] {
        &self.bytes
    }

    pub fn as_bytes(&self) -> &[u8; N] {
        &self.bytes
    }
}

impl<const N: usize> std::fmt::Debug for SecretKey<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SecretKey<{}>([REDACTED])", N)
    }
}

impl<const N: usize> std::fmt::Display for SecretKey<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[REDACTED SECRET]")
    }
}

impl<const N: usize> ConstantTimeEq for SecretKey<N> {
    fn ct_eq(&self, other: &Self) -> subtle::Choice {
        self.bytes.ct_eq(&other.bytes)
    }
}

impl<const N: usize> PartialEq for SecretKey<N> {
    fn eq(&self, other: &Self) -> bool {
        self.ct_eq(other).into()
    }
}

impl<const N: usize> Eq for SecretKey<N> {}

#[cfg(feature = "serde")]
impl<const N: usize> serde::Serialize for SecretKey<N> {
    fn serialize<S>(&self, _serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::Error;
        Err(S::Error::custom("SecretKey cannot be serialized"))
    }
}
