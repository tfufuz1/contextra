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

<<<<<<< HEAD
use rand::RngCore;

/// Cryptographically secure OS random number generator adapter.
=======
/// Cryptographically secure OS-backed random number generator backed by [`getrandom::getrandom`].
>>>>>>> 0000a4c4f (feat(ports,sys,engine,mcp): implement CryptoRng, SecretBytes, and secure proof signer key loading (F-01, F-10, F-22))
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

<<<<<<< HEAD
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
=======
    /// Generates the next pseudorandom 64-bit unsigned integer using OS CSPRNG.
    pub fn next_u64(&self) -> u64 {
        let mut buf = [0u8; 8];
        let _ = getrandom::getrandom(&mut buf);
        u64::from_le_bytes(buf)
    }

    /// Fills `dest` with cryptographically secure pseudorandom bytes using OS CSPRNG.
    pub fn fill_bytes(&self, dest: &mut [u8]) {
        let _ = getrandom::getrandom(dest);
    }

    /// Fills `dest` with cryptographically secure bytes, returning a Result if OS entropy is unavailable.
    pub fn fill_secret(&self, dest: &mut [u8]) -> Result<(), String> {
        getrandom::getrandom(dest)
            .map_err(|e| format!("getrandom OS CSPRNG call failed: {e}"))
    }
}

#[cfg(feature = "ports")]
impl contextra_ports::CryptoRng for OsCryptoRng {
    fn fill_secret(&self, dest: &mut [u8]) -> Result<(), contextra_ports::RngError> {
        getrandom::getrandom(dest).map_err(|e| {
            contextra_ports::RngError::Unavailable(format!("OS entropy source unavailable: {e}"))
        })
    }
}

#[cfg(feature = "ports")]
impl contextra_ports::EntropyRng for OsCryptoRng {
    fn fill_bytes(&self, dest: &mut [u8]) {
        OsCryptoRng::fill_bytes(self, dest);
    }

    fn next_u64(&self) -> u64 {
        OsCryptoRng::next_u64(self)
    }

    fn fill_secret(&self, dest: &mut [u8]) -> Result<(), contextra_ports::RngError> {
        contextra_ports::CryptoRng::fill_secret(self, dest)
>>>>>>> 0000a4c4f (feat(ports,sys,engine,mcp): implement CryptoRng, SecretBytes, and secure proof signer key loading (F-01, F-10, F-22))
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
