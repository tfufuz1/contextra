//! Arc-retirement queue. Physical release is returned only after the last view drops.

use std::sync::Arc;

pub struct Graveyard<T> { retired: Vec<Arc<T>> }

impl<T> Default for Graveyard<T> {
    fn default() -> Self { Self { retired: Vec::new() } }
}

impl<T> Graveyard<T> {
    pub fn new() -> Self { Self::default() }

    pub fn retire(&mut self, object: Arc<T>) {
        self.retired.push(object);
    }

    pub fn pending(&self) -> usize { self.retired.len() }

    /// Extracts objects whose sole remaining strong reference belongs to this
    /// graveyard. The caller may now unlink files or reclaim partition memory.
    pub fn reclaim(&mut self) -> Vec<T> {
        let mut ready = Vec::new();
        let mut waiting = Vec::with_capacity(self.retired.len());
        for object in self.retired.drain(..) {
            if Arc::strong_count(&object) == 1 {
                match Arc::try_unwrap(object) {
                    Ok(value) => ready.push(value),
                    Err(still_shared) => waiting.push(still_shared),
                }
            } else {
                waiting.push(object);
            }
        }
        self.retired = waiting;
        ready
    }
}
