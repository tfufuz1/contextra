//! Deterministic SeededRng implementation for testing and simulations (§7.4).

use contextra_ports::SeededRng as SeededRngPort;
use std::sync::atomic::{AtomicU64, Ordering};

const GOLDEN_RATIO_64: u64 = 0x9e37_79b9_7f4a_7c15;

/// Deterministic, thread-safe SplitMix64 PRNG implementing [`SeededRngPort`].
///
/// Guaranteed to produce identical streams for identical seeds across executions.
#[derive(Debug)]
pub struct SeededRng {
    state: AtomicU64,
}

impl SeededRng {
    /// Creates a new [`SeededRng`] initialized with the given 64-bit seed.
    pub fn new(seed: u64) -> Self {
        Self {
            state: AtomicU64::new(seed),
        }
    }
}

impl SeededRngPort for SeededRng {
    fn next_u64(&self) -> u64 {
        let z = self
            .state
            .fetch_add(GOLDEN_RATIO_64, Ordering::SeqCst)
            .wrapping_add(GOLDEN_RATIO_64);
        let mut z = z;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
    }

    fn fill_bytes(&self, mut dest: &mut [u8]) {
        while !dest.is_empty() {
            let val = self.next_u64();
            let bytes = val.to_le_bytes();
            let len = dest.len().min(8);
            dest[..len].copy_from_slice(&bytes[..len]);
            dest = &mut dest[len..];
        }
    }

    fn next_unit_f64(&self) -> f64 {
        let mantissa = self.next_u64() >> 11;
        (mantissa as f64) * (1.0 / (1u64 << 53) as f64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_seeded_rng_determinism() {
        let rng1 = SeededRng::new(42);
        let rng2 = SeededRng::new(42);

        let seq1: Vec<u64> = (0..10).map(|_| rng1.next_u64()).collect();
        let seq2: Vec<u64> = (0..10).map(|_| rng2.next_u64()).collect();

        assert_eq!(seq1, seq2, "SeededRng with same seed produced different streams");
    }
}
