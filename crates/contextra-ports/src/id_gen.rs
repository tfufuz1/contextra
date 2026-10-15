//! Unique ID generator port trait without SystemTime dependencies.

use std::sync::Arc;

/// Port trait for unique 64-bit unsigned identifier generation.
///
/// Implementations produce unique identifiers without relying on `SystemTime` in the trait interface.
pub trait IdGenerator: Send + Sync + 'static {
    /// Generates and returns the next unique 64-bit unsigned identifier.
    fn next_id(&self) -> u64;
}

/// Alias for [`IdGenerator`] trait for specification naming compatibility.
pub use IdGenerator as IdGen;

impl<T: IdGenerator + ?Sized> IdGenerator for Arc<T> {
    fn next_id(&self) -> u64 {
        (**self).next_id()
    }
}

/// A thread-safe, sequential 64-bit ID generator for deterministic testing.
#[derive(Debug)]
pub struct SequentialIdGen {
    next: std::sync::atomic::AtomicU64,
}

impl SequentialIdGen {
    /// Creates a new [`SequentialIdGen`] starting at `start_id`.
    pub fn new(start_id: u64) -> Self {
        Self {
            next: std::sync::atomic::AtomicU64::new(start_id),
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
        self.next.fetch_add(1, std::sync::atomic::Ordering::SeqCst)
    }
}

/// Standard production ID generator.
#[derive(Debug)]
pub struct SystemIdGen {
    next: std::sync::atomic::AtomicU64,
}

impl SystemIdGen {
    /// Creates a new [`SystemIdGen`].
    pub fn new() -> Self {
        let start = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| u64::try_from(d.as_nanos()).unwrap_or(u64::MAX))
            .unwrap_or(1);
        Self {
            next: std::sync::atomic::AtomicU64::new(start),
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
        self.next.fetch_add(1, std::sync::atomic::Ordering::SeqCst)
    }
}

/// Alias for [`SystemIdGen`].
pub use SystemIdGen as SystemIdGenerator;

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    struct TestIdGen {
        next: AtomicU64,
    }

    impl IdGenerator for TestIdGen {
        fn next_id(&self) -> u64 {
            self.next.fetch_add(1, Ordering::SeqCst)
        }
    }

    fn assert_dyn_id_generator(_: Option<&dyn IdGenerator>) {}

    #[test]
    fn test_id_generator_object_safety() {
        let gen = Arc::new(TestIdGen {
            next: AtomicU64::new(100),
        });

        assert_dyn_id_generator(Some(gen.as_ref()));

        let dyn_gen: Arc<dyn IdGenerator> = gen;
        assert_eq!(dyn_gen.next_id(), 100);
        assert_eq!(dyn_gen.next_id(), 101);
    }
}
