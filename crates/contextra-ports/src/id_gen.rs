//! Unique ID generator port trait and atomic sequential implementation.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

/// Port trait for unique ID generation.
pub trait IdGen: Send + Sync + 'static {
    /// Generates and returns the next unique 64-bit unsigned identifier.
    fn next_id(&self) -> u64;
}

/// A thread-safe ID generator that produces monotonically increasing 64-bit IDs.
///
/// Wraps around safely on integer overflow (`u64::MAX` -> `0`) via atomic standard wrapping.
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

impl<T: IdGen + ?Sized> IdGen for Arc<T> {
    fn next_id(&self) -> u64 {
        (**self).next_id()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    fn _assert_dyn_id_gen(_: Option<&dyn IdGen>) {}

    #[test]
    fn test_sequential_id_gen_monotonic() {
        let gen = SequentialIdGen::new(10);
        assert_eq!(gen.next_id(), 10);
        assert_eq!(gen.next_id(), 11);
        assert_eq!(gen.next_id(), 12);
    }

    #[test]
    fn test_sequential_id_gen_concurrent_no_duplicates() {
        let gen = Arc::new(SequentialIdGen::new(1));
        let handles: Vec<_> = (0..8)
            .map(|_| {
                let g = gen.clone();
                std::thread::spawn(move || {
                    let mut ids = Vec::with_capacity(1000);
                    for _ in 0..1000 {
                        ids.push(g.next_id());
                    }
                    ids
                })
            })
            .collect();

        let mut all_ids = HashSet::with_capacity(8000);
        for handle in handles {
            let ids = handle.join().expect("Thread panicked");
            for id in ids {
                assert!(all_ids.insert(id), "Duplicate ID found: {id}");
            }
        }

        assert_eq!(all_ids.len(), 8000);
    }

    #[test]
    fn test_arc_dyn_id_gen() {
        let gen: Arc<dyn IdGen> = Arc::new(SequentialIdGen::new(100));
        _assert_dyn_id_gen(Some(gen.as_ref()));
        assert_eq!(gen.next_id(), 100);
        assert_eq!(gen.next_id(), 101);
    }
}
