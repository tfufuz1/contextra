//! Stable, domain-separated key derivation for the erasure journal and ledger.
//!
//! The root is supplied by the storage composition root. This module performs no I/O,
//! time reads, or random generation.

use crate::{error::Result, kdf::hkdf_expand, secret::SecretKey};

/// Domain separating labels for keys derived from an erasure root.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[non_exhaustive]
pub enum ErasureLabel {
    /// Authentication key for the erasure ledger.
    LedgerMac,
    /// Encryption key for the erasure journal.
    ErasureLog,
    /// Key used to wrap a salt vault generation.
    VaultEpoch,
    /// Per-job key, with the erasure ID as context.
    JobKey,
}

impl ErasureLabel {
    /// Frozen HKDF labels. Changing these values breaks key compatibility.
    pub const fn info(self) -> &'static [u8] {
        match self {
            Self::LedgerMac => b"ctx-ledger-mac-v1",
            Self::ErasureLog => b"ctx-erasure-log-v1",
            Self::VaultEpoch => b"ctx-vault-epoch-v1",
            Self::JobKey => b"ctx-erasure-job-v1",
        }
    }
}

/// Zeroizing root key material. It cannot be cloned, formatted with its bytes, or serialized.
pub struct ErasureRoot(SecretKey<32>);

impl ErasureRoot {
    /// Wraps root material supplied by the caller.
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(SecretKey::new(bytes))
    }

    /// Derives a domain-separated key using HKDF-SHA256.
    ///
    /// # Errors
    /// Returns the KDF error if the root is invalid or HKDF expansion fails.
    ///
    /// # Crash safety
    /// This method is deterministic and does not persist anything. Callers must persist
    /// the root independently and keep it stable across restarts.
    pub fn derive(&self, label: ErasureLabel, context: &[u8]) -> Result<SecretKey<32>> {
        let mut info = Vec::with_capacity(label.info().len() + context.len());
        info.extend_from_slice(label.info());
        info.extend_from_slice(context);
        let mut out = [0u8; 32];
        hkdf_expand(self.0.as_bytes(), &info, &mut out)?;
        Ok(SecretKey::new(out))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derivation_is_stable_and_labels_are_separated() {
        let root = ErasureRoot::from_bytes([0x42; 32]);
        let a = root.derive(ErasureLabel::LedgerMac, b"ctx").expect("valid HKDF input");
        let b = root.derive(ErasureLabel::LedgerMac, b"ctx").expect("valid HKDF input");
        let c = root.derive(ErasureLabel::ErasureLog, b"ctx").expect("valid HKDF input");
        assert_eq!(a, b);
        assert_ne!(a, c);
    }
}
