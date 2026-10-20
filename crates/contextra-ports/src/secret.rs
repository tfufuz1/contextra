//! Secret bytes port abstractions for secure key and seed handling.

use std::fmt;
use zeroize::Zeroize;

/// Stack-allocated fixed-size secret byte buffer that is zeroized upon drop.
///
/// Ensures cryptographic material is safely scrubbed from RAM when dropped,
/// and prevents accidental exposure via [`fmt::Debug`] or [`fmt::Display`].
pub struct SecretBytes<const N: usize> {
    inner: [u8; N],
}

impl<const N: usize> SecretBytes<N> {
    /// Constructs a new [`SecretBytes`] wrapping `bytes`.
    pub fn new(bytes: [u8; N]) -> Self {
        Self { inner: bytes }
    }

    /// Exposes a read-only reference to the underlying secret array.
    pub fn expose_secret(&self) -> &[u8; N] {
        &self.inner
    }

    /// Exposes a read-only slice to the underlying secret bytes.
    pub fn as_slice(&self) -> &[u8] {
        &self.inner
    }

    /// Explicitly zeroizes the secret byte array in memory.
    pub fn zeroize(&mut self) {
        self.inner.zeroize();
    }
}

impl<const N: usize> Drop for SecretBytes<N> {
    fn drop(&mut self) {
        self.inner.zeroize();
    }
}

impl<const N: usize> fmt::Debug for SecretBytes<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SecretBytes([REDACTED])")
    }
}

impl<const N: usize> fmt::Display for SecretBytes<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[REDACTED]")
    }
}

impl<const N: usize> Zeroize for SecretBytes<N> {
    fn zeroize(&mut self) {
        self.inner.zeroize();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_secret_bytes_redaction_and_zeroize() {
        let mut secret = SecretBytes::new([0xAA; 32]);
        assert_eq!(secret.expose_secret(), &[0xAA; 32]);
        assert_eq!(secret.as_slice(), &[0xAA; 32]);
        assert_eq!(format!("{:?}", secret), "SecretBytes([REDACTED])");
        assert_eq!(format!("{}", secret), "[REDACTED]");

        secret.zeroize();
        assert_eq!(secret.expose_secret(), &[0; 32]);
    }
}
