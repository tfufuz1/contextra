//! Thin delegation wrapper for Hybrid Search Signal Fusion (delegated to `memfuse-rank::fusion`).

use crate::{ProvenanceRecord, SearchResult, SignalContribution};
pub use memfuse_rank::fusion::{BoundedTopK, MetadataMergePriority, ResonanceConfig, SignalKind};

/// Fluent builder for constructing `ProvenanceRecord` instances.
#[derive(Debug, Clone, PartialEq)]
pub struct ProvenanceBuilder {
    inner: memfuse_rank::fusion::ProvenanceBuilder,
}

impl ProvenanceBuilder {
    /// Creates a new `ProvenanceBuilder` with specified RRF parameter `k`.
    pub fn new(rrf_k: f32) -> Self {
        Self {
            inner: memfuse_rank::fusion::ProvenanceBuilder::new(rrf_k),
        }
    }

    /// Attaches vector signal distance, rank, and optional weight.
    pub fn vector(mut self, dist: f32, rank: u32, weight: impl Into<Option<f32>>) -> Self {
        self.inner = self.inner.vector(dist, rank, weight);
        self
    }

    /// Attaches BM25 text signal score, rank, and optional weight.
    pub fn bm25(mut self, score: f32, rank: u32, weight: impl Into<Option<f32>>) -> Self {
        self.inner = self.inner.bm25(score, rank, weight);
        self
    }

    /// Attaches graph signal score, rank, and optional weight.
    pub fn graph(mut self, score: f32, rank: u32, weight: impl Into<Option<f32>>) -> Self {
        self.inner = self.inner.graph(score, rank, weight);
        self
    }

    /// Attaches optional cross-encoder rerank score.
    pub fn rerank_score(mut self, score: f32) -> Self {
        self.inner = self.inner.rerank_score(score);
        self
    }

    /// Attaches source collection name.
    pub fn source_collection(mut self, collection: impl Into<String>) -> Self {
        self.inner = self.inner.source_collection(collection);
        self
    }

    /// Attaches index type string.
    pub fn index_type(mut self, index_type: impl Into<String>) -> Self {
        self.inner = self.inner.index_type(index_type);
        self
    }

    /// Sets expected ground truth RRF score.
    pub fn expected_total(mut self, total: f32) -> Self {
        self.inner = self.inner.expected_total(total);
        self
    }

    /// Builds `crate::ProvenanceRecord`.
    pub fn build(self) -> ProvenanceRecord {
        from_rank_provenance(self.inner.build())
    }
}

fn to_rank_result(r: SearchResult) -> memfuse_rank::fusion::SearchResult {
    memfuse_rank::fusion::SearchResult {
        id: r.id,
        score: r.score,
        metadata: r.metadata,
        matched_signals: r.matched_signals,
        provenance: r.provenance.map(to_rank_provenance),
    }
}

fn from_rank_result(r: memfuse_rank::fusion::SearchResult) -> SearchResult {
    SearchResult {
        id: r.id,
        score: r.score,
        metadata: r.metadata,
        matched_signals: r.matched_signals,
        provenance: r.provenance.map(from_rank_provenance),
    }
}

fn to_rank_provenance(p: ProvenanceRecord) -> memfuse_rank::fusion::ProvenanceRecord {
    memfuse_rank::fusion::ProvenanceRecord {
        vector_distance: p.vector_distance,
        bm25_score: p.bm25_score,
        graph_score: p.graph_score,
        rerank_score: p.rerank_score,
        signal_ranks: p.signal_ranks,
        source_collection: p.source_collection,
        index_type: p.index_type,
        signal_contributions: p
            .signal_contributions
            .into_iter()
            .map(|(k, v)| (k, to_rank_contribution(v)))
            .collect(),
        coherence_bonus: p.coherence_bonus,
    }
}

fn from_rank_provenance(p: memfuse_rank::fusion::ProvenanceRecord) -> ProvenanceRecord {
    ProvenanceRecord {
        vector_distance: p.vector_distance,
        bm25_score: p.bm25_score,
        graph_score: p.graph_score,
        rerank_score: p.rerank_score,
        signal_ranks: p.signal_ranks,
        source_collection: p.source_collection,
        index_type: p.index_type,
        signal_contributions: p
            .signal_contributions
            .into_iter()
            .map(|(k, v)| (k, from_rank_contribution(v)))
            .collect(),
        coherence_bonus: p.coherence_bonus,
    }
}

fn to_rank_contribution(c: SignalContribution) -> memfuse_rank::fusion::SignalContribution {
    memfuse_rank::fusion::SignalContribution {
        raw_score: c.raw_score,
        rank: c.rank,
        rrf_contribution: c.rrf_contribution,
    }
}

fn from_rank_contribution(c: memfuse_rank::fusion::SignalContribution) -> SignalContribution {
    SignalContribution {
        raw_score: c.raw_score,
        rank: c.rank,
        rrf_contribution: c.rrf_contribution,
    }
}

/// Fuses multiple sets of ranked search results into a single ranked list using Reciprocal Rank Fusion (RRF).
pub fn reciprocal_rank_fusion(
    result_sets: Vec<Vec<SearchResult>>,
    max_results: usize,
) -> Vec<SearchResult> {
    let rank_sets = result_sets
        .into_iter()
        .map(|set| set.into_iter().map(to_rank_result).collect())
        .collect();
    memfuse_rank::fusion::reciprocal_rank_fusion(rank_sets, max_results)
        .into_iter()
        .map(from_rank_result)
        .collect()
}

/// Weighted Reciprocal Rank Fusion with default signal metadata priority.
pub fn weighted_reciprocal_rank_fusion(
    result_sets: Vec<(String, Vec<SearchResult>, f32)>,
    max_results: usize,
) -> Vec<SearchResult> {
    let rank_sets = result_sets
        .into_iter()
        .map(|(name, set, weight)| {
            (
                name,
                set.into_iter().map(to_rank_result).collect(),
                weight,
            )
        })
        .collect();
    memfuse_rank::fusion::weighted_reciprocal_rank_fusion(rank_sets, max_results)
        .into_iter()
        .map(from_rank_result)
        .collect()
}

/// Weighted Reciprocal Rank Fusion with explicit metadata merge priority.
pub fn weighted_reciprocal_rank_fusion_with_priority(
    result_sets: Vec<(String, Vec<SearchResult>, f32)>,
    max_results: usize,
    priority: MetadataMergePriority,
) -> Vec<SearchResult> {
    let rank_sets = result_sets
        .into_iter()
        .map(|(name, set, weight)| {
            (
                name,
                set.into_iter().map(to_rank_result).collect(),
                weight,
            )
        })
        .collect();
    memfuse_rank::fusion::weighted_reciprocal_rank_fusion_with_priority(
        rank_sets,
        max_results,
        priority,
    )
    .into_iter()
    .map(from_rank_result)
    .collect()
}

/// Weighted Reciprocal Rank Fusion with options.
pub fn weighted_reciprocal_rank_fusion_with_options(
    result_sets: Vec<(String, Vec<SearchResult>, f32)>,
    max_results: usize,
    priority: MetadataMergePriority,
    include_provenance: bool,
    resonance_config: Option<&ResonanceConfig>,
) -> Vec<SearchResult> {
    let rank_sets = result_sets
        .into_iter()
        .map(|(name, set, weight)| {
            (
                name,
                set.into_iter().map(to_rank_result).collect(),
                weight,
            )
        })
        .collect();
    memfuse_rank::fusion::weighted_reciprocal_rank_fusion_with_options(
        rank_sets,
        max_results,
        priority,
        include_provenance,
        resonance_config,
    )
    .into_iter()
    .map(from_rank_result)
    .collect()
}

/// Fuses search result sets using the specified FusionStrategy.
pub fn fuse_search_results_with_strategy(
    result_sets: Vec<(String, Vec<SearchResult>, f32)>,
    max_results: usize,
    priority: MetadataMergePriority,
    include_provenance: bool,
    resonance_config: Option<&ResonanceConfig>,
    strategy: memfuse_core::FusionStrategy,
) -> Vec<SearchResult> {
    let rank_sets = result_sets
        .into_iter()
        .map(|(name, set, weight)| {
            (
                name,
                set.into_iter().map(to_rank_result).collect(),
                weight,
            )
        })
        .collect();
    memfuse_rank::fusion::fuse_search_results_with_strategy(
        rank_sets,
        max_results,
        priority,
        include_provenance,
        resonance_config,
        strategy,
    )
    .into_iter()
    .map(from_rank_result)
    .collect()
}

/// Score-normalized fusion with options.
pub fn score_normalized_fusion_with_options(
    result_sets: Vec<(String, Vec<SearchResult>, f32)>,
    max_results: usize,
    priority: MetadataMergePriority,
    include_provenance: bool,
    resonance_config: Option<&ResonanceConfig>,
) -> Vec<SearchResult> {
    let rank_sets = result_sets
        .into_iter()
        .map(|(name, set, weight)| {
            (
                name,
                set.into_iter().map(to_rank_result).collect(),
                weight,
            )
        })
        .collect();
    memfuse_rank::fusion::score_normalized_fusion_with_options(
        rank_sets,
        max_results,
        priority,
        include_provenance,
        resonance_config,
    )
    .into_iter()
    .map(from_rank_result)
    .collect()
}

/// Applies Resonance Coherence Bonus to fused search results.
pub fn apply_resonance_bonus(
    results: Vec<SearchResult>,
    valid_signal_count: usize,
    config: &ResonanceConfig,
) -> Vec<SearchResult> {
    let rank_results = results.into_iter().map(to_rank_result).collect();
    memfuse_rank::fusion::apply_resonance_bonus(rank_results, valid_signal_count, config)
        .into_iter()
        .map(from_rank_result)
        .collect()
}

/// Converts optional FusionWeights into (vector, text, graph) weight tuple.
pub fn weights_to_signal_factors(
    weights: Option<&memfuse_core::FusionWeights>,
) -> (f32, f32, f32) {
    memfuse_rank::fusion::weights_to_signal_factors(weights)
}
