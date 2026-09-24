//! Random number generator port trait definition and SplitMix64 deterministic implementation.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

/// Golden ratio constant step for SplitMix64 (`2^64 / phi`).
const GOLDEN_RATIO_64: u64 = 0x9e37_79b9_7f4a_7c15;

/// Abstract random number generator trait for non-determinism injection.
///
/// Implementations must be thread-safe (`Send + Sync + 'static`) and use interior
/// mutability (`&self`) so that `Arc<dyn Rng>` can be shared without external locking.
pub trait Rng: Send + Sync + 'static {
    /// Generates the next pseudo-random 64-bit unsigned integer.
    fn next_u64(&self) -> u64;

    /// Fills the destination buffer `dest` with pseudo-random bytes.
    fn fill_bytes(&self, dest: &mut [u8]);

    /// Returns a pseudo-random `f64` in the half-open interval `[0.0, 1.0)`.
    ///
    /// The value is generated using a 53-bit mantissa to ensure uniform distribution.
    fn next_unit_f64(&self) -> f64;
}

impl<T: Rng + ?Sized> Rng for Arc<T> {
    fn next_u64(&self) -> u64 {
        (**self).next_u64()
    }

    fn fill_bytes(&self, dest: &mut [u8]) {
        (**self).fill_bytes(dest)
    }

    fn next_unit_f64(&self) -> f64 {
        (**self).next_unit_f64()
    }
}

/// A deterministic pseudo-random number generator based on the SplitMix64 algorithm.
///
/// Uses an [`AtomicU64`] state counter advanced via atomic [`Ordering::SeqCst`] fetch-add
/// with the golden ratio increment, making all generation methods lock-free and thread-safe.
///
/// # Note on Entropy Seeding
/// High-entropy random seeding (e.g., via `OsRng` or OS-provided randomness) is strictly
/// the responsibility of the Composition Root when initializing [`SeededRng`].
#[derive(Debug)]
pub struct SeededRng {
    state: AtomicU64,
}

impl SeededRng {
    /// Creates a new [`SeededRng`] initialized with the given `seed`.
    pub fn new(seed: u64) -> Self {
        Self {
            state: AtomicU64::new(seed),
        }
    }

    /// Internal helper to run the SplitMix64 bit-mixing function on a state value.
    #[inline]
    fn splitmix64(state: u64) -> u64 {
        let mut z = state;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
    }
}

impl Rng for SeededRng {
    fn next_u64(&self) -> u64 {
        let state = self.state.fetch_add(GOLDEN_RATIO_64, Ordering::SeqCst);
        Self::splitmix64(state.wrapping_add(GOLDEN_RATIO_64))
    }

    fn fill_bytes(&self, dest: &mut [u8]) {
        let mut chunk_start = 0;
        while chunk_start + 8 <= dest.len() {
            let val = self.next_u64();
            dest[chunk_start..chunk_start + 8].copy_from_slice(&val.to_le_bytes());
            chunk_start += 8;
        }
        if chunk_start < dest.len() {
            let val = self.next_u64();
            let bytes = val.to_le_bytes();
            let rem = dest.len() - chunk_start;
            dest[chunk_start..].copy_from_slice(&bytes[..rem]);
        }
    }

    fn next_unit_f64(&self) -> f64 {
        let val = self.next_u64() >> 11;
        (val as f64) * (1.0 / ((1u64 << 53) as f64))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_seeded_rng_reproducibility() {
        let rng1 = SeededRng::new(12345);
        let rng2 = SeededRng::new(12345);

        let seq1: Vec<u64> = (0..10).map(|_| rng1.next_u64()).collect();
        let seq2: Vec<u64> = (0..10).map(|_| rng2.next_u64()).collect();

        assert_eq!(seq1, seq2);
    }

    #[test]
    fn test_seeded_rng_different_seeds() {
        let rng1 = SeededRng::new(100);
        let rng2 = SeededRng::new(200);

        assert_ne!(rng1.next_u64(), rng2.next_u64());
    }

    #[test]
    fn test_next_unit_f64_bounds() {
        let rng = SeededRng::new(42);
        for _ in 0..1000 {
            let v = rng.next_unit_f64();
            assert!((0.0..1.0).contains(&v), "Value {} out of [0, 1)", v);
        }
    }

    #[test]
    fn test_fill_bytes_lengths() {
        let rng = SeededRng::new(999);
        for len in [0, 1, 7, 8, 9, 16, 23] {
            let mut buf = vec![0u8; len];
            rng.fill_bytes(&mut buf);
            if len > 0 {
                // Assert buffer was modified (statistically almost guaranteed for non-zero lengths)
                assert!(buf.iter().any(|&b| b != 0) || len == 0);
            }
        }
    }

    #[test]
    fn test_arc_dyn_rng_compiles() {
        let rng: Arc<dyn Rng> = Arc::new(SeededRng::new(1));
        let _ = rng.next_u64();
        let _ = rng.next_unit_f64();
        let mut buf = [0u8; 4];
        rng.fill_bytes(&mut buf);
    }
}
