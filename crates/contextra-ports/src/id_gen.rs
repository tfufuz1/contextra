//! Unique ID generator port trait and atomic sequential implementation.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

/// Port trait for unique ID generation without SystemTime in its interface.
pub trait IdGenerator: Send + Sync + 'static {
    /// Generates and returns the next unique 64-bit unsigned identifier.
    fn next_id(&self) -> u64;
}

/// Alias for [`IdGenerator`] trait for specification naming compatibility.
pub use IdGenerator as IdGen;

/// Standard production implementation of [`IdGenerator`] initialized from system wall-clock nanoseconds.
#[derive(Debug)]
pub struct SystemIdGen {
    next: AtomicU64,
}

/// Alias for [`SystemIdGen`] for specification naming compatibility.
pub type SystemIdGenerator = SystemIdGen;

impl SystemIdGen {
    /// Creates a new [`SystemIdGen`] initialized from current system wall-clock nanoseconds.
    #[allow(clippy::disallowed_methods)]
    pub fn new() -> Self {
        let start = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| u64::try_from(d.as_nanos()).unwrap_or(u64::MAX))
            .unwrap_or(1);
        Self {
            next: AtomicU64::new(start),
        }
    }
}

impl Default for SystemIdGen {
    fn default() -> Self {
        Self::new()
    }
}

impl IdGenerator for SystemIdGen {
    fn next_id(&self) -> u64 {
        self.next.fetch_add(1, Ordering::SeqCst)
    }
}

/// A thread-safe ID generator that produces monotonically increasing 64-bit IDs.
#[derive(Debug)]
pub struct SequentialIdGen {
    next: AtomicU64,
}

/// Test implementation alias for [`SequentialIdGen`].
pub type SequentialIds = SequentialIdGen;

impl SequentialIdGen {
    /// Creates a new [`SequentialIdGen`] starting at `start`.
    pub fn new(start: u64) -> Self {
        Self {
            next: AtomicU64::new(start),
        }
    }
}

impl Default for SequentialIdGen {
    fn default() -> Self {
        Self::new(1)
    }
}

impl IdGenerator for SequentialIdGen {
    fn next_id(&self) -> u64 {
        self.next.fetch_add(1, Ordering::SeqCst)
    }
}

impl<T: IdGenerator + ?Sized> IdGenerator for Arc<T> {
    fn next_id(&self) -> u64 {
        (**self).next_id()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_dyn_id_generator(_: Option<&dyn IdGenerator>) {}

    #[test]
    fn test_id_generator_object_safety() {
        let gen = Arc::new(SequentialIdGen::new(100));
        assert_dyn_id_generator(Some(gen.as_ref()));

        let dyn_gen: Arc<dyn IdGenerator> = gen;
        assert_eq!(dyn_gen.next_id(), 100);
        assert_eq!(dyn_gen.next_id(), 101);
    }
}
