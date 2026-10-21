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
}

#[cfg(feature = "ports")]
impl contextra_ports::CryptoRng for OsCryptoRng {}

#[cfg(feature = "ports")]
impl contextra_ports::EntropyRng for OsCryptoRng {
    fn fill_bytes(&self, dest: &mut [u8]) {
        OsCryptoRng::fill_bytes(self, dest);
    }

    fn next_u64(&self) -> u64 {
        OsCryptoRng::next_u64(self)
    }
}

#[cfg(feature = "ports")]
impl contextra_ports::SeededRng for OsCryptoRng {
    fn next_u64(&self) -> u64 {
        OsCryptoRng::next_u64(self)
    }

    fn fill_bytes(&self, dest: &mut [u8]) {
        OsCryptoRng::fill_bytes(self, dest);
    }

    fn next_unit_f64(&self) -> f64 {
        let mantissa = OsCryptoRng::next_u64(self) >> 11;
        (mantissa as f64) * (1.0 / (1u64 << 53) as f64)
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
