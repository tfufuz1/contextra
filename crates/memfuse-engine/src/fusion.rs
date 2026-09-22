//! Hybrid Search Signal Fusion implementations (Reciprocal Rank Fusion & Score Normalization).
//!
//! Re-exports and redirects all fusion types and operations to `memfuse-rank`.

pub use memfuse_rank::fusion::{
    apply_resonance_bonus, reciprocal_rank_fusion, score_normalized_fusion_with_options,
    weighted_reciprocal_rank_fusion, weighted_reciprocal_rank_fusion_with_options,
    weighted_reciprocal_rank_fusion_with_priority, weights_to_signal_factors, BoundedTopK,
    MetadataMergePriority, ProvenanceBuilder, ProvenanceRecord, ResonanceConfig, SearchResult,
    SignalContribution, SignalKind,
};

pub use memfuse_core::FusionStrategy;

/// Fuses search result sets using the specified `FusionStrategy` (`Rrf` or `ScoreNormalized`).
pub fn fuse_search_results_with_strategy(
    result_sets: Vec<(String, Vec<SearchResult>, f32)>,
    max_results: usize,
    priority: MetadataMergePriority,
    include_provenance: bool,
    resonance_config: Option<&ResonanceConfig>,
    strategy: FusionStrategy,
) -> Vec<SearchResult> {
    memfuse_rank::fusion::fuse_search_results_with_strategy(
        result_sets,
        max_results,
        priority,
        include_provenance,
        resonance_config,
        strategy,
    )
}

/// Convenience function for fusing multi-signal search results.
pub fn fuse_signals(
    result_sets: Vec<(String, Vec<SearchResult>, f32)>,
    max_results: usize,
) -> Vec<SearchResult> {
    memfuse_rank::fusion::fuse_signals(result_sets, max_results)
}
