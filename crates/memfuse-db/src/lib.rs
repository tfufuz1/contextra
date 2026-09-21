// FILE-CONTEXT
// ZWECK: MemFuse Database Strangler Shell / Re-export Facade (Layer 3).
// INVARIANTEN: Backward compatibility for all public types; re-exports from memfuse-engine and memfuse-cognition.

#![forbid(unsafe_code)]

pub use memfuse_cognition as cognition;
pub use memfuse_engine as engine;

// Re-exports from memfuse-engine
pub use memfuse_engine::{
    background_workers, chunker, collection, export, filter, import, temporal_filter, transaction,
};
pub use memfuse_engine::{
    CommunityDetectionConfig, DbStats, Document, EmbeddingBackend, ExportCollectionV1,
    ExportDocumentV1, ExportMemoryV1, ExportRelationV1, HybridQueryBuilder, ImportSummary,
    Language, MemFuse, MemFuseConfig, MemFuseStats, MetadataFilter,
    SearchStrategy, SignalWeights, MAX_SCAN_RESULTS, SCHEMA_VERSION_V1,
};

// Re-exports from memfuse-rank (Ring 0)
#[cfg(feature = "coherence-bonus-fusion")]
pub use memfuse_rank::apply_resonance_bonus;
pub use memfuse_rank::{
    fuse_search_results_with_strategy, reciprocal_rank_fusion,
    score_normalized_fusion_with_options, weighted_reciprocal_rank_fusion,
    weighted_reciprocal_rank_fusion_with_options, weighted_reciprocal_rank_fusion_with_priority,
    weights_to_signal_factors, BoundedTopK, MetadataMergePriority, ProvenanceBuilder,
    ProvenanceRecord, ResonanceConfig, SearchResult, SignalContribution, SignalKind,
};

#[cfg(feature = "graph-connectivity-health")]
pub use memfuse_engine::collection::maintenance::PercolationResult;
#[cfg(feature = "sandbox")]
pub use memfuse_engine::SandboxBridge;

// Re-exports from memfuse-cognition
pub use memfuse_cognition::{
    cleanup_orphaned_consolidation_intents, compact_segment_via_context_compactor,
    compute_community_hash, detect_near_duplicates, execute_background_consolidation,
    execute_consolidation_pass, execute_sleep_cycle, group_turns_into_segments,
    run_consolidation_pass, run_synthesis_pass, CommunityStabilityTracker, CompactedContext,
    CompactionStrategy, ConsolidationConfig, ConsolidationEngine, ConsolidationNodesGuard,
    ConsolidationPhaseResult, ConsolidationSession, ContextCompactor, ContextManager,
    MaintenanceConfig, MaintenanceScheduler, MetaChunk, SpatialFence, StatusToken, SynthesisConfig,
    SynthesisPhaseResult, TurnSegment,
};

pub mod context {
    pub use memfuse_cognition::context::*;
}
pub mod context_compaction {
    pub use memfuse_cognition::context_compaction::*;
}
pub mod memory_consolidation {
    pub use memfuse_cognition::memory_consolidation::*;
}
pub mod synthesis_phase {
    pub use memfuse_cognition::synthesis_phase::*;
}
pub mod consolidation_executor {
    pub use memfuse_cognition::consolidation_executor::*;
}
pub mod consolidation_locks {
    pub use memfuse_cognition::consolidation_locks::*;
}
pub mod maintenance_scheduler {
    pub use memfuse_cognition::maintenance_scheduler::*;
}
pub mod maintenance_config {
    pub use memfuse_cognition::maintenance_config::*;
}

#[deprecated(note = "use background_workers instead")]
pub mod reaper {
    pub use memfuse_engine::background_workers::*;
}
#[deprecated(note = "use start_consolidation_worker instead")]
#[allow(deprecated)]
pub use memfuse_cognition::start_consolidation_reaper;

pub mod decay_controller {
    pub use memfuse_adapt::decay_controller::*;
}
pub mod fusion;
pub mod homeostat {
    pub use memfuse_adapt::homeostat::*;
}
pub mod multistep;
pub mod pid_latency_controller {
    pub use memfuse_adapt::pid_latency_controller::*;
}

#[cfg(feature = "volatile-vault")]
pub mod volatile_vault;
#[cfg(feature = "volatile-vault")]
pub use volatile_vault::{
    CommitReceipt, PurgeReceipt, SignalModality, VaultChunk, VaultChunkMetadata, VaultConfig,
    VaultError, VolatileContextVault,
};

pub use decay_controller::{AdaptiveDecayController, DecayControllerConfig, DecaySignalInputs};
#[allow(deprecated)]
pub use homeostat::{pid_regulated_candidate_pool, RerankDeadline, RerankPidController};
pub use multistep::{MultiStepConfig, MultiStepEngine, MultiStepResult, QueryRewriter};
pub use pid_latency_controller::{
    LatencyBudgetGuard, PidLatencyController, DEFAULT_TARGET_LATENCY_MS, MAX_SCALING_FACTOR,
    MIN_SCALING_FACTOR,
};

pub use collection::{Collection, CollectionConfig};
pub use memfuse_checkpoint;
#[cfg(feature = "graph-connectivity-health")]
pub use memfuse_graph::percolation::PercolationConfig;

pub use memfuse_core::DistanceMetric;
pub use memfuse_core::DriftStatusProvider;
pub use memfuse_core::SegmentSynthesizer;
pub use memfuse_core::TextEmbeddingEngine;
pub use serde_json::json;
