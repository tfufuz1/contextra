//! # Signal-Fusions-Subsystem (`fusion`)
//!
//! **Ring:** Ring 0 (Ranking & Kalibrierung)
//!
//! Dieses Modul bündelt alle Algorithmen zur Zusammenführung (Fusion) mehrdimensionaler
//! Suchergebnisse aus verschiedenen Retrieval-Kanälen mittels Reciprocal Rank Fusion (RRF).

// ANCHOR[contextra-rank:FUS-MOD-001]
mod normalized;
mod provenance;
mod resonance;
mod rrf;
mod signal;
mod topk;
mod types;

pub use contextra_types::{FusionStrategy, SignalFusionStrategies};
pub use normalized::{score_normalized_fusion_with_options, weights_to_signal_factors};
pub use provenance::ProvenanceBuilder;
pub use resonance::{apply_resonance_bonus, fuse_signals, ResonanceConfig};
// Begründung: Veraltete Wrapper bleiben für Abwärtskompatibilität zu contextra-engine erhalten
// bis diese umgestellt ist (siehe ÜBERGABE).
#[allow(deprecated)]
pub use rrf::{
    fuse_search_results_with_signal_strategies, fuse_search_results_with_strategy,
    reciprocal_rank_fusion, weighted_reciprocal_rank_fusion, weighted_reciprocal_rank_fusion_mrrf,
    weighted_reciprocal_rank_fusion_with_options, weighted_reciprocal_rank_fusion_with_priority,
};
pub use rrf::{g, modulate_and_renormalize_weights, rrf_fuse, DEFAULT_RRF_K};
pub use signal::{MetadataMergePriority, SignalKind};
pub use topk::BoundedTopK;
pub use types::{
    FusedScore, Hit, ProvenanceRecord, SearchReport, SearchResult, Signal, SignalContribution,
    SignalFailurePolicy,
};
