//! Hybrid Search Signal Fusion implementations (Reciprocal Rank Fusion & Score Normalization).
//!
//! Re-exports and redirects all fusion types and operations to `contextra-rank`.

pub use contextra_rank::fusion::{
    apply_resonance_bonus, reciprocal_rank_fusion, score_normalized_fusion_with_options,
    weighted_reciprocal_rank_fusion, weighted_reciprocal_rank_fusion_with_options,
    weighted_reciprocal_rank_fusion_with_priority, weights_to_signal_factors, BoundedTopK,
    MetadataMergePriority, ProvenanceBuilder, ProvenanceRecord, ResonanceConfig, SearchResult,
    SignalContribution, SignalKind,
};

pub use contextra_types::FusionStrategy;

/// Fuses search result sets using the specified FusionStrategy.
pub fn fuse_search_results_with_strategy(
    result_sets: Vec<(String, Vec<SearchResult>, f32)>,
    max_results: usize,
    priority: MetadataMergePriority,
    include_provenance: bool,
    resonance_config: Option<&ResonanceConfig>,
    strategy: contextra_types::FusionStrategy,
) -> Vec<SearchResult> {
    contextra_rank::fusion::fuse_search_results_with_strategy(
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
    contextra_rank::fusion::fuse_signals(result_sets, max_results)
}
