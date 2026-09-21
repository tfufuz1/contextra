//! MemFuse Ring 0 — Signal fusion, multi-step retrieval primitives, and score calibration.

// FILE-CONTEXT
// STAND: 2026-09-20T00:00:00Z
// ZWECK: MemFuse Ring 0 — Signal fusion, multi-step retrieval primitives, and score calibration.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod fusion;
pub mod isotonic;
/// Multi-step iterative retrieval primitives.
pub mod multistep;
pub mod platt;

#[cfg(feature = "coherence-bonus-fusion")]
pub use fusion::apply_resonance_bonus;
pub use fusion::{
    fuse_search_results_with_strategy, reciprocal_rank_fusion,
    score_normalized_fusion_with_options, weighted_reciprocal_rank_fusion,
    weighted_reciprocal_rank_fusion_with_options, weighted_reciprocal_rank_fusion_with_priority,
    weights_to_signal_factors, BoundedTopK, MetadataMergePriority, ProvenanceBuilder,
    ProvenanceRecord, ResonanceConfig, SearchResult, SignalContribution, SignalKind,
};
pub use isotonic::IsotonicCalibrator;
pub use multistep::{MultiStepConfig, MultiStepResult, QueryRewriter, DEFAULT_TARGET_LATENCY_MS};
pub use platt::PlattScaler;
