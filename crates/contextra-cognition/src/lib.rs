// FILE-CONTEXT
// ZWECK: Contextra Cognition Engine (Layer 3 - Cognition).
// INVARIANTEN: No unsafe code; depends on contextra-engine; zero cyclic dependencies.

#![forbid(unsafe_code)]

use std::sync::Arc;

pub mod aggregation_phase;
pub mod consolidation_executor;
pub mod consolidation_locks;
pub mod context;
pub mod context_compaction;
pub mod graph_sink;
pub mod leanrag_input;
pub mod maintenance_config;
pub mod maintenance_scheduler;
pub mod memory_consolidation;
pub mod synthesis_phase;

pub use aggregation_phase::{
    check_compaction_budget, compute_entity_community_hash, run_aggregation_pass,
    AggregationConfig, AggregationEdge, AggregationNode, AggregationPhaseResult, AlphaNode,
    ConsolidationPipelineResult, SuperEdgeDraft, SuperEdgeSink,
};
#[allow(deprecated)]
pub use consolidation_executor::{
    execute_background_consolidation, execute_consolidation_pass,
    execute_leanrag_aggregation_stage, execute_sleep_cycle, start_consolidation_reaper,
    start_consolidation_worker, ConsolidationEngine,
};
pub use consolidation_locks::ConsolidationNodesGuard;
pub use context::{ContextManager, SpatialFence};
pub use context_compaction::{
    cleanup_orphaned_consolidation_intents, CompactedContext, CompactionStrategy,
    ConsolidationSession, ContextCompactor, StatusToken,
};
pub use graph_sink::CsrGraphSuperEdgeSink;
pub use leanrag_input::{build_leanrag_inputs, LeanRagInputs, DEFAULT_MAX_LEANRAG_NODES};
pub use maintenance_config::MaintenanceConfig;
pub use maintenance_scheduler::MaintenanceScheduler;
pub use memory_consolidation::{
    compact_segment_via_context_compactor, compute_community_hash, detect_near_duplicates,
    group_turns_into_segments, run_consolidation_pass, CommunityStabilityTracker,
    ConsolidationConfig, ConsolidationPhaseResult, MetaChunk, SynthesisConfig,
    SynthesisPhaseResult, TurnSegment,
};
pub use synthesis_phase::run_synthesis_pass;

/// Registers consolidation engine launcher with `contextra-engine`.
pub fn init() {
    contextra_engine::register_consolidation_launcher(
        |col, interval, max_llm_calls, cancel_token| {
            let synthesis_config = memory_consolidation::SynthesisConfig {
                max_llm_calls_per_cycle: max_llm_calls as u32,
                ..Default::default()
            };
            let engine = Arc::new(consolidation_executor::ConsolidationEngine::new(
                col,
                memory_consolidation::ConsolidationConfig::default(),
                synthesis_config,
                interval,
                cancel_token,
            ));
            engine.start()
        },
    );
}
