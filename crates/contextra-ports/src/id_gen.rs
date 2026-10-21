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

/// Atomic sequential 64-bit ID generator.
#[derive(Debug)]
pub struct SequentialIdGen {
    next: std::sync::atomic::AtomicU64,
}

impl SequentialIdGen {
    /// Creates a new `SequentialIdGen` initialized with the starting seed.
    pub fn new(seed: u64) -> Self {
        Self {
            next: std::sync::atomic::AtomicU64::new(seed),
        }
    }
}

impl IdGenerator for SequentialIdGen {
    fn next_id(&self) -> u64 {
        self.next.fetch_add(1, std::sync::atomic::Ordering::SeqCst)
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
