//! OS-backed cryptographically secure random number generator (Ring 0).

/// Cryptographically secure OS-backed random number generator backed by [`getrandom::getrandom`].
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
