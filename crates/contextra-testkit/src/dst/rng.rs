// FILE-CONTEXT
// STAND: 2026-10-12
// ZWECK: Seedable, deterministic PRNG for Deterministic Simulation Testing (DST).
// INVARIANTEN: Bit-identical random streams for identical seeds across executions.

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
}
