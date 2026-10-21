//! Cryptographically secure OS random number generator interface (§7.4).
//!
//! This module provides [`OsCryptoRng`], which accesses the operating system's
//! entropy source directly via [`rand::rngs::OsRng`].
//!
//! # OS Entropy Guarantee
//! Randomness provided by this module originates directly from OS system calls
//! (e.g., `getrandom` on Linux, ` SecRandomCopyBytes` on macOS, `BCryptGenRandom` on Windows).
//!
//! # Non-Deterministic / Security Scope
//! Test PRNGs, deterministic seeded generators, and pseudo-random generators
//! DO NOT belong in this module. Non-cryptographic or seeded generators must be placed
//! in [`crate::determinism`].

use rand::RngCore;

/// Cryptographically secure OS random number generator adapter.
///
/// Wraps system entropy calls without panic paths, adhering to Contextra's
/// zero-panic principle (P7).
#[derive(Debug, Default, Clone, Copy)]
pub struct OsCryptoRng;

impl OsCryptoRng {
    /// Creates a new instance of [`OsCryptoRng`].
    #[inline]
    pub fn new() -> Self {
        Self
    }

    /// Fills `dest` with random bytes from the OS entropy source.
    ///
    /// Returns `Ok(())` on success or `Err(rand::Error)` if the OS entropy source fails.
    #[inline]
    pub fn try_fill_bytes(&self, dest: &mut [u8]) -> Result<(), rand::Error> {
        rand::rngs::OsRng.try_fill_bytes(dest)
    }

    /// Generates a random `u64` from the OS entropy source.
    ///
    /// Returns `Ok(u64)` on success or `Err(rand::Error)` if the OS entropy source fails.
    #[inline]
    pub fn try_next_u64(&self) -> Result<u64, rand::Error> {
        let mut buf = [0u8; 8];
        self.try_fill_bytes(&mut buf)?;
        Ok(u64::from_ne_bytes(buf))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_os_crypto_rng_draw_distinct_bytes() {
        let rng = OsCryptoRng::new();
        let mut buf1 = [0u8; 32];
        let mut buf2 = [0u8; 32];

        rng.try_fill_bytes(&mut buf1).expect("Entropy draw 1 failed");
        rng.try_fill_bytes(&mut buf2).expect("Entropy draw 2 failed");

        assert_ne!(buf1, buf2, "Two 32-byte OS entropy draws must not be identical");
    }

    #[test]
    fn test_os_crypto_rng_try_next_u64() {
        let rng = OsCryptoRng;
        let val1 = rng.try_next_u64().expect("u64 entropy draw 1 failed");
        let val2 = rng.try_next_u64().expect("u64 entropy draw 2 failed");

        // Statistically, two random u64s will practically never be equal.
        assert_ne!(val1, val2);
    }
}
