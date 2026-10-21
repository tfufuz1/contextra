//! OS-backed cryptographically secure random number generator (Ring 0).

use rand::RngCore;

/// Cryptographically secure OS-backed random number generator backed by [`rand::rngs::OsRng`].
///
/// Guaranteed to draw entropy directly from OS CSPRNG syscalls (e.g., `getrandom` / `getentropy` / `CryptGenRandom`).
#[derive(Debug, Default, Clone, Copy)]
pub struct OsCryptoRng;

impl OsCryptoRng {
    /// Creates a new [`OsCryptoRng`].
    pub fn new() -> Self {
        Self
    }

    /// Generates the next pseudorandom 64-bit unsigned integer using OS CSPRNG.
    pub fn next_u64(&self) -> u64 {
        rand::rngs::OsRng.next_u64()
    }

    /// Fills `dest` with cryptographically secure pseudorandom bytes using OS CSPRNG.
    pub fn fill_bytes(&self, dest: &mut [u8]) {
        rand::rngs::OsRng.fill_bytes(dest);
    }

    /// Fills `dest` with cryptographically secure bytes using OS CSPRNG.
    ///
    /// # Errors
    ///
    /// Returns an error message if OS CSPRNG fill fails.
    pub fn try_fill_bytes(&self, dest: &mut [u8]) -> Result<(), String> {
        rand::rngs::OsRng
            .try_fill_bytes(dest)
            .map_err(|e| format!("OS CSPRNG fill_bytes failed: {e}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_os_crypto_rng_generates_entropy() {
        let rng = OsCryptoRng::new();
        let mut buf1 = [0u8; 32];
        let mut buf2 = [0u8; 32];

        rng.fill_bytes(&mut buf1);
        rng.fill_bytes(&mut buf2);

        assert_ne!(buf1, [0u8; 32]);
        assert_ne!(buf2, [0u8; 32]);
        assert_ne!(buf1, buf2, "CSPRNG produced identical 32-byte outputs");
    }
}
