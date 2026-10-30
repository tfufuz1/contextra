//! Size-bounded min-heap for Top-K candidate selection across search and ranking operations.

#![forbid(unsafe_code)]

use crate::MAX_SEARCH_K;
use std::collections::BinaryHeap;

/// Size-bounded min-heap for Top-K candidate selection.
///
/// # Invarianten
/// - `capacity` wird immer auf maximal [`MAX_SEARCH_K`] (1.000) beschränkt.
/// - Hält maximal `capacity` Elemente; beim Überschreiten wird der schlechteste Kandidat verdraengt.
#[derive(Debug, Clone)]
pub struct BoundedTopK<T> {
    heap: BinaryHeap<T>,
    capacity: usize,
}

impl<T: Ord> BoundedTopK<T> {
    /// Creates a new `BoundedTopK` container with maximum capacity `k`.
    /// `capacity` is clamped to [`MAX_SEARCH_K`].
    pub fn new(capacity: usize) -> Self {
        let capacity = capacity.min(MAX_SEARCH_K);
        Self {
            heap: BinaryHeap::with_capacity(capacity.saturating_add(1)),
            capacity,
        }
    }

    /// Returns maximum allowed capacity `k`.
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Returns current number of items held in the container.
    pub fn len(&self) -> usize {
        self.heap.len()
    }

    /// Returns `true` if the container holds zero items.
    pub fn is_empty(&self) -> bool {
        self.heap.is_empty()
    }

    /// Pushes an item into top-K container, evicting the worst candidate if size exceeds capacity `k`.
    pub fn push(&mut self, item: T) {
        if self.capacity == 0 {
            return;
        }
        if self.heap.len() < self.capacity {
            self.heap.push(item);
        } else if let Some(worst) = self.heap.peek() {
            if item < *worst {
                self.heap.pop();
                self.heap.push(item);
            }
        }
    }

    /// Consumes the container and returns items sorted from best to worst.
    pub fn into_sorted_vec(self) -> Vec<T> {
        self.heap.into_sorted_vec()
    }
}
