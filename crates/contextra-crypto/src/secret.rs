//! Secret key and payload containers with zeroize-on-drop and constant-time equality.

use subtle::ConstantTimeEq;
use zeroize::{Zeroize, ZeroizeOnDrop};

/// Fixed-size secret key that is wiped on drop and never printed.
#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub struct SecretKey<const N: usize> {
    bytes: [u8; N],
}

impl<const N: usize> SecretKey<N> {
    /// Creates a new secret key wrapping raw bytes.
    pub fn new(bytes: [u8; N]) -> Self {
        Self { bytes }
    }

    /// Exposes a reference to the secret bytes.
    #[inline]
    pub fn expose_secret(&self) -> &[u8; N] {
        &self.bytes
    }

    /// Exposes a reference to the secret bytes (compatibility alias).
    #[inline]
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
        write!(f, "SecretKey<{}>([REDACTED])", N)
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

/// Dynamically-sized secret container that is wiped on drop and never printed.
#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub struct Secret<T: Zeroize> {
    inner: T,
}

impl<T: Zeroize> Secret<T> {
    /// Creates a new `Secret` wrapping sensitive data.
    pub fn new(inner: T) -> Self {
        Self { inner }
    }

    /// Exposes a reference to the inner secret.
    #[inline]
    pub fn expose_secret(&self) -> &T {
        &self.inner
    }

    /// Exposes a mutable reference to the inner secret.
    #[inline]
    pub fn expose_secret_mut(&mut self) -> &mut T {
        &mut self.inner
    }
}

impl<T: Zeroize> std::fmt::Debug for Secret<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Secret<T>([REDACTED])")
    }
}

impl<T: Zeroize> std::fmt::Display for Secret<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Secret<T>([REDACTED])")
    }
}

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

#[cfg(feature = "serde")]
impl<T: Zeroize> serde::Serialize for Secret<T> {
    fn serialize<S>(&self, _serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::Error;
        Err(S::Error::custom("Secret cannot be serialized"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_secret_key_expose_and_redaction() {
        let raw = [0x42u8; 32];
        let secret = SecretKey::new(raw);
        assert_eq!(secret.expose_secret(), &raw);
        assert_eq!(secret.as_bytes(), &raw);

        let debug_str = format!("{secret:?}");
        assert!(!debug_str.contains("42"));
        assert!(debug_str.contains("[REDACTED]"));

        let display_str = format!("{secret}");
        assert!(!display_str.contains("42"));
        assert!(display_str.contains("[REDACTED]"));
    }

    #[test]
    fn test_secret_container_expose_and_redaction() {
        let payload = vec![1, 2, 3, 4, 5];
        let secret = Secret::new(payload.clone());
        assert_eq!(secret.expose_secret(), &payload);

        let debug_str = format!("{secret:?}");
        assert!(!debug_str.contains("1"));
        assert!(debug_str.contains("[REDACTED]"));
    }
}
