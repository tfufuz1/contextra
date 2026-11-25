// FILE-CONTEXT: On-demand candidate streaming for vector search.
// ZWECK: Liefert Vector-Kandidaten in Batches à 16 (oder konfigurierbar) via geometrisches Nachladen.
// INVARIANTEN: #![forbid(unsafe_code)], zero panic/unwrap, max_depth = MAX_SEARCH_K.
// STAND: TS:2026-09-15T00:00:00Z

use contextra_core::{DocId, Result, ScoredDocument, VectorIndex, MAX_SEARCH_K};
use std::collections::{HashSet, VecDeque};

/// Default batch size for vector candidate retrieval.
pub const DEFAULT_VECTOR_STREAM_BATCH_SIZE: usize = 16;

/// On-demand loading async pull-stream for vector search candidates.
///
/// # Candidate Ordering & Determinism
/// Vector search scores represent relevance (higher score = better candidate).
/// Newly discovered candidates in each reload iteration are delivered in deterministic
/// order: score descending, then [`DocId`] ascending. `f32::total_cmp` is used to
/// ensure safe and deterministic ordering even with special floating point values.
///
/// # Non-Prefix-Stability (Known Limitation)
/// Approximate Nearest Neighbor (ANN) search algorithms (such as HNSW or DiskANN) are
/// **not prefix-stable**: `search(q, 32)` is not guaranteed to return the exact top-16
/// of `search(q, 16)` in identical positions or order.
///
/// To handle this robustly:
/// - A `seen: HashSet<DocId>` tracks all previously yielded document IDs, ensuring no document
///   is ever returned twice.
/// - In each refill step with depth $k'$, any newly discovered candidates not previously seen
///   are sorted deterministically (score descending, `DocId` ascending) and buffered.
///
/// # Index Search Depth (`ef_search` Expansion)
/// In HNSW implementations (such as [`HnswIndex`][crate::hnsw::HnswIndex]), searching for $k$ candidates
/// dynamically expands the search scope `ef = ef_search.max(k)`. Requesting larger $k$ during geometric
/// reloading maintains or improves recall quality without quality degradation.
///
/// # Geometric Reloading
/// Candidates are loaded in geometric stages ($k' = \text{batch\_size}, \dots, 2 \times k, \dots, 1000$).
/// Total search work is bounded by $\le 2 \times$ the final retrieval depth, capped at `MAX_SEARCH_K` (1,000).
pub struct VectorCandidateStream<'a, V: VectorIndex> {
    index: &'a V,
    query: Vec<f32>,
    seq_no: Option<u64>,
    batch_size: usize,
    buffer: VecDeque<ScoredDocument>,
    seen: HashSet<DocId>,
    fetched_k: usize,
    yielded: usize,
    exhausted: bool,
}

impl<'a, V: VectorIndex> VectorCandidateStream<'a, V> {
    /// Creates a new vector candidate stream for a query and index snapshot.
    pub fn new(index: &'a V, query: impl Into<Vec<f32>>, seq_no: Option<u64>) -> Self {
        Self {
            index,
            query: query.into(),
            seq_no,
            batch_size: DEFAULT_VECTOR_STREAM_BATCH_SIZE,
            buffer: VecDeque::new(),
            seen: HashSet::new(),
            fetched_k: 0,
            yielded: 0,
            exhausted: false,
        }
    }

    /// Sets a custom batch size (minimum 1).
    pub fn with_batch_size(mut self, n: usize) -> Self {
        self.batch_size = n.max(1);
        self
    }

    /// Fetches the next batch of candidate documents (up to `batch_size`).
    /// Returns an empty `Vec` when exhausted.
    pub async fn next_batch(&mut self) -> Result<Vec<ScoredDocument>> {
        while self.buffer.len() < self.batch_size && !self.exhausted {
            self.refill().await?;
        }

        let drain_count = self.batch_size.min(self.buffer.len());
        let mut batch = Vec::with_capacity(drain_count);
        for _ in 0..drain_count {
            if let Some(item) = self.buffer.pop_front() {
                batch.push(item);
            }
        }
        self.yielded = self.yielded.saturating_add(batch.len());
        Ok(batch)
    }

    /// Fetches the next single candidate document ID.
    /// Returns `Ok(None)` when exhausted.
    pub async fn next_doc(&mut self) -> Result<Option<DocId>> {
        if self.buffer.is_empty() && !self.exhausted {
            self.refill().await?;
        }

        if let Some(item) = self.buffer.pop_front() {
            self.yielded = self.yielded.saturating_add(1);
            Ok(Some(item.doc_id))
        } else {
            Ok(None)
        }
    }

    /// Returns the total number of items yielded so far.
    pub fn yielded(&self) -> usize {
        self.yielded
    }

    /// Returns the last $k$ search depth requested from the vector index.
    pub fn fetched_depth(&self) -> usize {
        self.fetched_k
    }

    /// Returns whether the candidate stream is exhausted.
    pub fn is_exhausted(&self) -> bool {
        self.exhausted && self.buffer.is_empty()
    }

    /// Returns the maximum depth limit for candidate retrieval (`MAX_SEARCH_K`).
    pub const fn max_depth() -> usize {
        MAX_SEARCH_K
    }

    async fn refill(&mut self) -> Result<()> {
        if self.exhausted {
            return Ok(());
        }

        let target_k = self
            .batch_size
            .max(self.fetched_k.saturating_mul(2))
            .min(MAX_SEARCH_K);

        let candidates = match self.seq_no {
            Some(seq) => self.index.search_at(&self.query, target_k, seq).await?,
            None => self.index.search(&self.query, target_k).await?,
        };

        let res_len = candidates.len();
        let mut new_candidates = Vec::new();

        for doc in candidates {
            if self.seen.insert(doc.doc_id) {
                new_candidates.push(doc);
            }
        }

        let new_items = new_candidates.len();

        // Sort newly discovered candidates per refill step deterministically:
        // score descending (higher is better), ties broken by DocId ascending.
        new_candidates.sort_by(|a, b| {
            b.score
                .total_cmp(&a.score)
                .then_with(|| a.doc_id.cmp(&b.doc_id))
        });

        self.buffer.extend(new_candidates);
        self.fetched_k = target_k;

        if res_len < target_k || (target_k == MAX_SEARCH_K && new_items == 0) {
            self.exhausted = true;
        }

        Ok(())
    }
}
