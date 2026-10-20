// FILE-CONTEXT
// STAND: 2026-10-12
// ZWECK: Seedable, deterministic PRNG for Deterministic Simulation Testing (DST).
// INVARIANTEN: Bit-identical random streams for identical seeds across executions.

use contextra_ports::EntropyRng as EntropyRngPort;
use contextra_sys::SystemRng;

/// Type-separated entropy RNG source for non-deterministic runtime entropy.
///
/// Unlike [`SeededRng`], `EntropyRng` CANNOT be initialized with a fixed seed value.
/// This enforces type-level separation between deterministic test generators ([`SeededRng`])
/// and real non-deterministic entropy sources ([`EntropyRng`]).
#[derive(Debug, Default)]
pub struct EntropyRng {
    inner: SystemRng,
}

impl EntropyRng {
    /// Creates a new `EntropyRng` instance seeded from system / runtime entropy.
    pub fn new() -> Self {
        Self {
            inner: SystemRng::new(),
        }
    }
}

impl EntropyRngPort for EntropyRng {
    fn next_u64(&self) -> u64 {
        EntropyRngPort::next_u64(&self.inner)
    }

    fn fill_bytes(&self, dest: &mut [u8]) {
        EntropyRngPort::fill_bytes(&self.inner, dest);
    }
}

pub use contextra_ports::SeededRng as PortSeededRng;

/// Seedable pseudo-random number generator (SplitMix64 algorithm).
#[derive(Debug, Clone)]
pub struct DstRng {
    state: u64,
}

impl DstRng {
    /// Creates a new `DstRng` from a 64-bit seed.
    pub fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    /// Generates the next pseudo-random 64-bit integer.
    pub fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
    }

    /// Returns a pseudo-random integer in `[min, max]`.
    pub fn next_range(&mut self, min: u64, max: u64) -> u64 {
        if min >= max {
            return min;
        }
        let span = max - min + 1;
        min + (self.next_u64() % span)
    }

    /// Returns a pseudo-random boolean.
    pub fn next_bool(&mut self) -> bool {
        (self.next_u64() & 1) == 1
    }

    /// Fills or creates a `Vec<u8>` with pseudo-random bytes.
    pub fn next_bytes(&mut self, len: usize) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(len);
        while bytes.len() < len {
            let val = self.next_u64().to_le_bytes();
            let take = (len - bytes.len()).min(8);
            bytes.extend_from_slice(&val[..take]);
        }
        bytes
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::seeded_rng::SeededRng;
    use contextra_ports::SeededRng as SeededRngTrait;

    #[test]
    fn test_rng_determinism() {
        let mut rng1 = DstRng::new(12345);
        let mut rng2 = DstRng::new(12345);

        for _ in 0..100 {
            assert_eq!(rng1.next_u64(), rng2.next_u64());
            assert_eq!(rng1.next_range(10, 50), rng2.next_range(10, 50));
            assert_eq!(rng1.next_bytes(16), rng2.next_bytes(16));
        }
    }

    #[test]
    fn test_entropy_rng_and_seeded_rng_separation() {
        let seeded = SeededRng::new(42);
        let seeded_same = SeededRng::new(42);
        assert_eq!(seeded.next_u64(), seeded_same.next_u64());

        let entropy1 = EntropyRng::new();
        let entropy2 = EntropyRng::new();
        let v1 = entropy1.next_u64();
        let v2 = entropy2.next_u64();
        // Non-deterministic streams from process/system entropy should differ
        assert_ne!(v1, v2);
    }
}
