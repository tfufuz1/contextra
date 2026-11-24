//! The one system-wide definition of `Clock` and `Entropy`.

/// Seconds since the Unix epoch (UTC).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UnixSeconds(pub u64);

/// Injectable time source (deterministic in simulation tests).
pub trait Clock: Send + Sync {
    /// Current wall-clock time.
    fn now(&self) -> UnixSeconds;
}

/// Injectable entropy source.
pub trait Entropy: Send + Sync {
    /// Fills `dest` with unpredictable bytes or reports failure.
    fn fill(&self, dest: &mut [u8]) -> Result<(), EntropyError>;
}

/// Entropy source failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EntropyError;

impl core::fmt::Display for EntropyError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("entropy source failed")
    }
}
impl std::error::Error for EntropyError {}
