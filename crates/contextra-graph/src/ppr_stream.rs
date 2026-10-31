//! Candidate stream for Personalized PageRank (PPR) graph retrieval (DiBud / Spec §21.2 / AK-17).
//!
//! # Candidate Ordering & Determinism
//! Results are delivered in deterministic order: score descending using [`f32::total_cmp`], with primary score ties broken by [`EntityId`] ascending.
//!
//! # Lazy Evaluation & Performance
//! Personalized PageRank ([`PprConfig`]) does not accept a `top_k` parameter; PPR computes the entire ranking vector.
//! Consequently, [`PprCandidateStream`] calculates PPR **once lazily** on the first pull ([`PprCandidateStream::next_batch`], [`PprCandidateStream::next_entity`], or [`PprCandidateStream::next_batch_docs`]),
//! truncates the result list to [`MAX_SEARCH_K`][contextra_types::MAX_SEARCH_K] (1,000), and yields from the cached vector in batches.
//!
//! The performance gain (P24) stems from seed-local forward-push calculation in the underlying PPR engine, rather than incremental batching.

use contextra_ports::GraphIndex;
use contextra_types::{DocId, EntityId, PprConfig, Result, MAX_SEARCH_K};
use std::collections::{HashSet, VecDeque};

/// Default batch size for candidate retrieval (Spec §21.2).
pub const DEFAULT_PPR_STREAM_BATCH_SIZE: usize = 16;

/// On-demand loading async candidate stream for Personalized PageRank search.
pub struct PprCandidateStream<'a, G: GraphIndex + ?Sized> {
    graph: &'a G,
    seeds: Vec<EntityId>,
    config: PprConfig,
    batch_size: usize,
    exclude_seeds: bool,
    ranked: Option<VecDeque<(EntityId, f32)>>,
    yielded: usize,
}

impl<'a, G: GraphIndex + ?Sized> PprCandidateStream<'a, G> {
    /// Creates a new candidate stream for Personalized PageRank starting from `seeds`.
    pub fn new(graph: &'a G, seeds: impl Into<Vec<EntityId>>, config: PprConfig) -> Self {
        Self {
            graph,
            seeds: seeds.into(),
            config,
            batch_size: DEFAULT_PPR_STREAM_BATCH_SIZE,
            exclude_seeds: false,
            ranked: None,
            yielded: 0,
        }
    }

    /// Sets a custom batch size (minimum 1).
    pub fn with_batch_size(mut self, n: usize) -> Self {
        self.batch_size = n.max(1);
        self
    }

    /// Specifies whether seed entities themselves should be excluded from the yielded candidate stream.
    pub fn with_exclude_seeds(mut self, exclude: bool) -> Self {
        self.exclude_seeds = exclude;
        self
    }

    /// Fetches the next batch of candidate entity-score pairs (up to `batch_size`).
    /// Returns an empty `Vec` when exhausted.
    pub async fn next_batch(&mut self) -> Result<Vec<(EntityId, f32)>> {
        let batch_size = self.batch_size;
        let ranked = self.ensure_calculated().await?;
        let drain_count = batch_size.min(ranked.len());
        let mut batch = Vec::with_capacity(drain_count);
        for _ in 0..drain_count {
            if let Some(item) = ranked.pop_front() {
                batch.push(item);
            }
        }
        self.yielded = self.yielded.saturating_add(batch.len());
        Ok(batch)
    }

    /// Fetches the next single candidate entity ID.
    /// Returns `Ok(None)` when exhausted.
    pub async fn next_entity(&mut self) -> Result<Option<EntityId>> {
        let ranked = self.ensure_calculated().await?;
        if let Some((eid, _score)) = ranked.pop_front() {
            self.yielded = self.yielded.saturating_add(1);
            Ok(Some(eid))
        } else {
            Ok(None)
        }
    }

    /// Fetches the next batch of resolved [`DocId`]s (up to `batch_size`).
    /// Entities for which `resolve` returns `None` are skipped.
    pub async fn next_batch_docs<F>(&mut self, mut resolve: F) -> Result<Vec<DocId>>
    where
        F: FnMut(EntityId) -> Option<DocId>,
    {
        let batch_size = self.batch_size;
        let ranked = self.ensure_calculated().await?;
        let mut docs = Vec::new();
        let mut popped_count = 0usize;
        while docs.len() < batch_size && !ranked.is_empty() {
            if let Some((eid, _score)) = ranked.pop_front() {
                popped_count = popped_count.saturating_add(1);
                if let Some(doc_id) = resolve(eid) {
                    docs.push(doc_id);
                }
            }
        }
        self.yielded = self.yielded.saturating_add(popped_count);
        Ok(docs)
    }

    /// Returns the total number of items yielded so far.
    pub fn yielded(&self) -> usize {
        self.yielded
    }

    /// Returns whether the candidate stream is exhausted.
    pub fn is_exhausted(&self) -> bool {
        match &self.ranked {
            Some(ranked) => ranked.is_empty(),
            None => self.seeds.is_empty(),
        }
    }

    /// Returns the maximum depth limit for candidate retrieval (`MAX_SEARCH_K`).
    pub const fn max_depth() -> usize {
        MAX_SEARCH_K
    }

    async fn ensure_calculated(&mut self) -> Result<&mut VecDeque<(EntityId, f32)>> {
        if self.ranked.is_none() {
            if self.seeds.is_empty() {
                self.ranked = Some(VecDeque::new());
            } else {
                let mut raw = self
                    .graph
                    .personalized_page_rank(&self.seeds, &self.config)
                    .await?;
                raw.sort_by(|a, b| b.1.total_cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
                raw.truncate(MAX_SEARCH_K);
                if self.exclude_seeds {
                    let seed_set: HashSet<EntityId> = self.seeds.iter().copied().collect();
                    raw.retain(|(eid, _)| !seed_set.contains(eid));
                }
                self.ranked = Some(VecDeque::from(raw));
            }
        }

        match self.ranked.as_mut() {
            Some(ranked) => Ok(ranked),
            None => Err(contextra_types::ContextraError::Internal(
                "Failed to initialize PPR stream ranking buffer".to_string(),
            )),
        }
    }
}
