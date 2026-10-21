//! OS-backed cryptographic entropy source using `getrandom`.

use crate::rng::{EntropyRng, RngError};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

static POISONED: AtomicBool = AtomicBool::new(false);
static FALLBACK_COUNTER: AtomicU64 = AtomicU64::new(1);

/// Cryptographic entropy source backed directly by OS random syscalls (`getrandom`).
///
/// # Security Contract
/// - [`EntropyRng::fill_secret`] invokes the operating system CSPRNG directly.
/// - If the OS entropy source fails, `OsEntropyRng` enters a permanent poisoned state
///   and subsequent secret generation requests fail closed with [`RngError::Unavailable`].
/// - Infallible legacy methods ([`EntropyRng::fill_bytes`], [`EntropyRng::next_u64`]) are NOT
///   capable of generating secret material. On OS entropy failure, they use non-zero fallback
///   entropy without panicking or returning zeroed buffers.
#[derive(Debug, Default, Clone, Copy)]
pub struct OsEntropyRng;

impl OsEntropyRng {
    /// Creates a new [`OsEntropyRng`].
    pub fn new() -> Self {
        Self
    }

    /// Returns `true` if the OS entropy source encountered a fatal error and was poisoned.
    pub fn is_poisoned() -> bool {
        POISONED.load(Ordering::Relaxed)
    }
}

impl EntropyRng for OsEntropyRng {
    fn fill_secret(&self, dest: &mut [u8]) -> Result<(), RngError> {
        if POISONED.load(Ordering::SeqCst) {
            return Err(RngError::Unavailable(
                "OS entropy source is poisoned due to previous failure".to_string(),
            ));
        }

        if let Err(err) = getrandom::getrandom(dest) {
            POISONED.store(true, Ordering::SeqCst);
            tracing::error!("Fatal error in OS entropy source getrandom: {}", err);
            return Err(RngError::Unavailable(format!(
                "getrandom system call failed: {}",
                err
            )));
        }

        Ok(())
    }

    fn fill_bytes(&self, dest: &mut [u8]) {
        if !POISONED.load(Ordering::Relaxed) {
            if getrandom::getrandom(dest).is_ok() {
                return;
            }
            POISONED.store(true, Ordering::SeqCst);
            tracing::error!("getrandom failed in fill_bytes; falling back to non-secret PRNG");
        }

        // Fallback for legacy non-secret fill_bytes (ensures no panic, no zeroes)
        use std::collections::hash_map::RandomState;
        use std::hash::{BuildHasher, Hasher};
        let mut written = 0;
        while written < dest.len() {
            let ctr = FALLBACK_COUNTER.fetch_add(1, Ordering::Relaxed);
            let mut hasher = RandomState::new().build_hasher();
            hasher.write_u64(ctr);
            let val = hasher.finish() | 1; // Guarantee non-zero
            let bytes = val.to_le_bytes();
            let chunk = dest.len().saturating_sub(written).min(8);
            if let (Some(dst_slice), Some(src_slice)) = (
                dest.get_mut(written..written.saturating_add(chunk)),
                bytes.get(..chunk),
            ) {
                dst_slice.copy_from_slice(src_slice);
            }
            written = written.saturating_add(chunk);
        }
    }

    fn next_u64(&self) -> u64 {
        let mut buf = [0u8; 8];
        self.fill_bytes(&mut buf);
        u64::from_le_bytes(buf)
    }
}
