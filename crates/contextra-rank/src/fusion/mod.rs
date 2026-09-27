//! Hybrid Search Signal Fusion implementations (Reciprocal Rank Fusion & Score Normalization).

mod global;
mod normalized;
mod provenance;
mod resonance;
mod rrf;
mod signal;
mod topk;
mod types;

pub use contextra_types::FusionStrategy;
pub use global::{GlobalFusionConfig, GlobalFusionStrategy};
pub use normalized::{score_normalized_fusion_with_options, weights_to_signal_factors};
pub use provenance::ProvenanceBuilder;
pub use resonance::{apply_resonance_bonus, fuse_signals, ResonanceConfig};
pub use rrf::{
    fuse_search_results_with_strategy, reciprocal_rank_fusion, weighted_reciprocal_rank_fusion,
    weighted_reciprocal_rank_fusion_with_options, weighted_reciprocal_rank_fusion_with_priority,
};
pub use signal::{MetadataMergePriority, SignalKind};
pub use topk::BoundedTopK;
pub use types::{FusedScore, ProvenanceRecord, SearchResult, SignalContribution};
