//! `contextra-rank`: 4-Signal Fusion, Isotonic & Platt Calibration, and Drift Detection.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod calibration;
pub mod drift;
pub mod explain;
pub mod fusion;

#[cfg(feature = "dibud")]
pub mod dibud;

pub use calibration::{IsotonicCalibrator, PlattScaler};
#[cfg(feature = "dibud")]
pub use dibud::{
    fuse_exact_prefix, fuse_exact_prefix_async, BudgetedChannel, DiBudFusionState, DiBudOutcome,
    DiBudStep, FusionBudget,
};
pub use drift::{DriftDetector, DriftStatus};
pub use explain::{explain, ExplanationEntry, RetrievalExplanation};
pub use fusion::{
    apply_resonance_bonus, fuse_search_results_with_strategy, reciprocal_rank_fusion,
    score_normalized_fusion_with_options, weighted_reciprocal_rank_fusion,
    weighted_reciprocal_rank_fusion_with_options, weighted_reciprocal_rank_fusion_with_priority,
    weights_to_signal_factors, BoundedTopK, MetadataMergePriority, ProvenanceBuilder,
    ResonanceConfig, SignalKind,
};
