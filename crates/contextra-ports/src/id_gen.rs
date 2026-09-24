//! ID generator port trait definition and sequential implementation.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

/// Abstract ID generator trait for non-determinism injection.
pub trait IdGen: Send + Sync + 'static {
    /// Generates and returns the next unique 64-bit identifier.
    fn next_id(&self) -> u64;
}

impl<T: IdGen + ?Sized> IdGen for Arc<T> {
    fn next_id(&self) -> u64 {
        (**self).next_id()
    }
}

/// A thread-safe, lock-free sequential ID generator backed by an [`AtomicU64`].
///
/// Increments monotonically on each invocation using atomic fetch-add semantics.
/// Overflow wraps around safely via `wrapping_add`.
#[derive(Debug)]
pub struct SequentialIdGen {
    next: AtomicU64,
}

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

impl IdGen for SequentialIdGen {
    fn next_id(&self) -> u64 {
        self.next.fetch_add(1, Ordering::SeqCst)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;
    use std::sync::Arc;
    use std::thread;

    #[test]
    fn test_sequential_id_gen_monotonic() {
        let gen = SequentialIdGen::new(10);
        assert_eq!(gen.next_id(), 10);
        assert_eq!(gen.next_id(), 11);
        assert_eq!(gen.next_id(), 12);
    }

    #[test]
    fn test_sequential_id_gen_wrap_around() {
        let gen = SequentialIdGen::new(u64::MAX);
        assert_eq!(gen.next_id(), u64::MAX);
        assert_eq!(gen.next_id(), 0);
        assert_eq!(gen.next_id(), 1);
    }

    #[test]
    fn test_sequential_id_gen_multithreaded() {
        let gen = Arc::new(SequentialIdGen::new(1));
        let num_threads = 8;
        let ids_per_thread = 1000;

        let handles: Vec<_> = (0..num_threads)
            .map(|_| {
                let g = gen.clone();
                thread::spawn(move || {
                    let mut ids = Vec::with_capacity(ids_per_thread);
                    for _ in 0..ids_per_thread {
                        ids.push(g.next_id());
                    }
                    ids
                })
            })
            .collect();

        let mut all_ids = HashSet::new();
        for handle in handles {
            let ids = handle.join().unwrap();
            for id in ids {
                assert!(all_ids.insert(id), "Duplicate ID generated: {}", id);
            }
        }

        assert_eq!(all_ids.len(), num_threads * ids_per_thread);
    }

    #[test]
    fn test_arc_dyn_id_gen_compiles() {
        let gen: Arc<dyn IdGen> = Arc::new(SequentialIdGen::default());
        let _ = gen.next_id();
    }
}
