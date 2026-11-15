//! # Signal-Fusions-Subsystem (`fusion`)
//!
//! **Ring:** Ring 0 (Ranking & Kalibrierung)
//!
//! Dieses Modul bündelt alle Algorithmen zur Zusammenführung (Fusion) mehrdimensionaler
//! Suchergebnisse aus verschiedenen Retrieval-Kanälen (Dense Vector, BM25-Volltext,
//! Graph-Traversierung, Reranking).
pub mod ext;
mod global;
pub mod mrrf;
mod normalized;
mod provenance;
#[cfg(feature = "exp-resonance")]
pub mod resonance;
mod rrf;
mod signal;
mod topk;
mod types;

pub use crate::local_types::{FusionStrategy, SignalFusionStrategies};
pub use ext::ResonanceModifier;
#[cfg(not(feature = "exp-resonance"))]
pub use ext::{apply_resonance_bonus, ResonanceConfig};
pub use global::{GlobalFusionConfig, GlobalFusionStrategy};
#[cfg(feature = "workspace")]
pub use crate::compat::types::weights_to_signal_factors_types_compat;
pub use normalized::{score_normalized_fusion_with_options, weights_to_signal_factors};
pub use provenance::ProvenanceBuilder;
#[cfg(feature = "exp-resonance")]
pub use resonance::{apply_resonance_bonus, fuse_signals, ResonanceConfig};
pub use rrf::{
    fuse_search_results_with_signal_strategies, fuse_search_results_with_strategy, g,
    modulate_and_renormalize_weights, reciprocal_rank_fusion, rrf_fuse,
    weighted_reciprocal_rank_fusion, weighted_reciprocal_rank_fusion_mrrf,
    weighted_reciprocal_rank_fusion_with_options, weighted_reciprocal_rank_fusion_with_priority,
};
pub use signal::{MetadataMergePriority, SignalCalibrationContext, SignalKind};
pub use topk::BoundedTopK;
pub use types::{
    FusedScore, ProvenanceRecord, ScoredItem, SearchReport, SearchResult, Signal,
    SignalContribution, SignalFailurePolicy,
};
#[cfg(feature = "workspace")]
pub use crate::compat::types::signal_contribution_synthesized_from_types;
