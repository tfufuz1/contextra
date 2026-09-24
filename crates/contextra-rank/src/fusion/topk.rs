pub(super) fn cmp_scores(a: f32, b: f32) -> std::cmp::Ordering {
    match (a.is_nan(), b.is_nan()) {
        (true, false) => std::cmp::Ordering::Less,
        (false, true) => std::cmp::Ordering::Greater,
        _ => a.total_cmp(&b),
    }
}

/// Size-bounded min-heap for Top-K candidate selection during Reciprocal Rank Fusion.
pub struct BoundedTopK<T> {
    heap: std::collections::BinaryHeap<T>,
    capacity: usize,
}

impl<T: Ord> BoundedTopK<T> {
    /// Creates a new `BoundedTopK` container with maximum capacity `k`.
    pub fn new(capacity: usize) -> Self {
        let capacity = capacity.min(contextra_types::MAX_SEARCH_K);
        Self {
            heap: std::collections::BinaryHeap::with_capacity(capacity.saturating_add(1)),
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

pub(super) struct TopKCandidate<'a> {
    pub(super) idx: u32,
    pub(super) score: f32,
    pub(super) id: &'a str,
}

impl<'a> PartialEq for TopKCandidate<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == std::cmp::Ordering::Equal
    }
}

impl<'a> Eq for TopKCandidate<'a> {}

impl<'a> Ord for TopKCandidate<'a> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        cmp_scores(other.score, self.score).then_with(|| self.id.cmp(other.id))
    }
}

impl<'a> PartialOrd for TopKCandidate<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
