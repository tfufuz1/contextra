// FILE-CONTEXT: On-demand candidate streaming for BM25 search.
// ZWECK: Liefert BM25-Kandidaten in Batches à 16 (oder konfigurierbar) via geometrisches Nachladen.
// INVARIANTEN: #![forbid(unsafe_code)], zero panic/unwrap, Snapshot-Isolation via max_seq, max_depth = MAX_SEARCH_K.
// STAND: TS:2026-09-14T00:00:00Z

use contextra_ports::StorageEngine;
use contextra_types::{DocId, Result, MAX_SEARCH_K};
use std::collections::{HashSet, VecDeque};

use crate::inverted::InvertedIndex;

/// Default batch size for candidate retrieval.
pub const DEFAULT_STREAM_BATCH_SIZE: usize = 16;

/// On-demand loading async pull-stream for BM25 search candidates.
///
/// # Candidate Ordering & Determinism
/// Results are delivered in deterministic order: score descending, then `DocId` ascending.
///
/// # Geometric Reloading
/// Candidates are loaded in geometric stages ($k' = 16, 32, 64, \dots, 1000$).
/// Total search work is bounded by $\le 2\times$ the final retrieval depth.
///
/// # Snapshot Isolation & Depth Limit
/// To guarantee consistent candidates across reload stages, specify `max_seq = Some(snapshot_seq)`.
/// Retrieval depth is hard-capped at `MAX_SEARCH_K` (1,000).
pub struct Bm25CandidateStream<'a, S: StorageEngine> {
    index: &'a InvertedIndex<S>,
    query: String,
    max_seq: Option<u64>,
    batch_size: usize,
    buffer: VecDeque<(DocId, f32)>,
    seen: HashSet<DocId>,
    fetched_k: usize,
    yielded: usize,
    exhausted: bool,
}

impl<'a, S: StorageEngine> Bm25CandidateStream<'a, S> {
    /// Creates a new candidate stream for a query and index snapshot.
    pub fn new(
        index: &'a InvertedIndex<S>,
        query: impl Into<String>,
        max_seq: Option<u64>,
    ) -> Self {
        Self {
            index,
            query: query.into(),
            max_seq,
            batch_size: DEFAULT_STREAM_BATCH_SIZE,
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
    pub async fn next_batch(&mut self) -> Result<Vec<(DocId, f32)>> {
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

        if let Some((doc_id, _score)) = self.buffer.pop_front() {
            self.yielded = self.yielded.saturating_add(1);
            Ok(Some(doc_id))
        } else {
            Ok(None)
        }
    }

    /// Returns the total number of items yielded so far.
    pub fn yielded(&self) -> usize {
        self.yielded
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

        let candidates = self
            .index
            .search_bm25_at(&self.query, target_k, self.max_seq)
            .await?;

        let res_len = candidates.len();
        let mut new_items = 0usize;

        for (doc_id, score) in candidates {
            if self.seen.insert(doc_id) {
                self.buffer.push_back((doc_id, score));
                new_items = new_items.saturating_add(1);
            }
        }

        self.fetched_k = target_k;

        if res_len < target_k || (target_k == MAX_SEARCH_K && new_items == 0) {
            self.exhausted = true;
        }

        Ok(())
    }
}
