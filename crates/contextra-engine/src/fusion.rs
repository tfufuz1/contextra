//! Hybrid Search Signal Fusion implementations (Reciprocal Rank Fusion & Score Normalization).
//!
//! Re-exports and redirects all fusion types and operations to `contextra-rank`.

pub use contextra_rank::fusion::{
    apply_resonance_bonus, reciprocal_rank_fusion, score_normalized_fusion_with_options,
    weighted_reciprocal_rank_fusion, weighted_reciprocal_rank_fusion_with_options,
    weighted_reciprocal_rank_fusion_with_priority, BoundedTopK,
    MetadataMergePriority, ProvenanceBuilder, ProvenanceRecord, ResonanceConfig, SearchReport,
    SearchResult, Signal, SignalContribution, SignalFailurePolicy, SignalKind,
};

pub use contextra_types::{FusionStrategy, SignalFusionStrategies};

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
        strategy.into(),
    )
}

/// Fuses search result sets using per-signal fusion strategies (`SignalFusionStrategies`).
pub fn fuse_search_results_with_signal_strategies(
    result_sets: Vec<(String, Vec<SearchResult>, f32)>,
    max_results: usize,
    priority: MetadataMergePriority,
    include_provenance: bool,
    resonance_config: Option<&ResonanceConfig>,
    strategies: contextra_types::SignalFusionStrategies,
) -> Vec<SearchResult> {
    contextra_rank::fusion::fuse_search_results_with_signal_strategies(
        result_sets,
        max_results,
        priority,
        include_provenance,
        resonance_config,
        strategies.into(),
    )
}

/// Convenience function for fusing multi-signal search results.
pub fn fuse_signals(
    result_sets: Vec<(String, Vec<SearchResult>, f32)>,
    max_results: usize,
) -> Vec<SearchResult> {
    contextra_rank::fusion::fuse_signals(result_sets, max_results)
}


/// Converts optional types::FusionWeights into signal factors.
pub fn weights_to_signal_factors(weights: Option<&contextra_types::FusionWeights>) -> (f32, f32, f32) {
    let rank_weights = weights.map(|w| contextra_rank::FusionWeights::from(w.clone()));
    contextra_rank::fusion::weights_to_signal_factors(rank_weights.as_ref())
}
