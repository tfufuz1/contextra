// FILE-CONTEXT
// ZWECK: Zeroize-on-drop secure byte buffer wrapper CchSecureBuf (Spec C4.7 / VAULT-Invarianten).
// STAND: TS:2026-10-12T00:00:00Z

#![forbid(unsafe_code)]

use zeroize::{Zeroize, Zeroizing};

/// Secure byte buffer wrapper around `Zeroizing<Vec<u8>>`.
///
/// Guarantees that buffer memory is zeroed out on drop or explicit zeroize.
/// Does NOT implement `Clone` or `Default` to prevent accidental memory copies or uninitialized states.
pub struct CchSecureBuf {
    inner: Zeroizing<Vec<u8>>,
}

impl CchSecureBuf {
    /// Creates a new `CchSecureBuf` from a `Vec<u8>`.
    pub fn from_vec(vec: Vec<u8>) -> Self {
        Self {
            inner: Zeroizing::new(vec),
        }
    }

    /// Returns a slice referencing the underlying buffer bytes.
    pub fn as_slice(&self) -> &[u8] {
        self.inner.as_slice()
    }

    /// Returns the length of the buffer in bytes.
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Returns `true` if the buffer contains no bytes.
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Consumes `self` and returns the inner `Zeroizing<Vec<u8>>`.
    pub fn into_zeroizing(self) -> Zeroizing<Vec<u8>> {
        self.inner
    }
}

impl Zeroize for CchSecureBuf {
    fn zeroize(&mut self) {
        self.inner.as_mut_slice().zeroize();
    }
}

impl std::fmt::Debug for CchSecureBuf {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CchSecureBuf")
            .field("len", &self.len())
            .finish()
    }
}
