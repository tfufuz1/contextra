// FILE-CONTEXT
// ZWECK: MemFuse Cognition Engine (Layer 3 - Cognition).
// INVARIANTEN: No unsafe code; depends on memfuse-engine; zero cyclic dependencies.

#![forbid(unsafe_code)]

use std::sync::Arc;

pub mod consolidation_executor;
pub mod consolidation_locks;
pub mod context;
pub mod context_compaction;
pub mod maintenance_config;
pub mod maintenance_scheduler;
pub mod memory_consolidation;
pub mod synthesis_phase;

#[allow(deprecated)]
pub use consolidation_executor::{
    execute_background_consolidation, execute_consolidation_pass, execute_sleep_cycle,
    start_consolidation_reaper, start_consolidation_worker, ConsolidationEngine,
};
pub use consolidation_locks::ConsolidationNodesGuard;
pub use context::{ContextManager, SpatialFence};
pub use context_compaction::{
    cleanup_orphaned_consolidation_intents, CompactedContext, CompactionStrategy,
    ConsolidationSession, ContextCompactor, StatusToken,
};
pub use maintenance_config::MaintenanceConfig;
pub use maintenance_scheduler::MaintenanceScheduler;
pub use memory_consolidation::{
    compact_segment_via_context_compactor, compute_community_hash, detect_near_duplicates,
    group_turns_into_segments, run_consolidation_pass, CommunityStabilityTracker,
    ConsolidationConfig, ConsolidationPhaseResult, MetaChunk, SynthesisConfig,
    SynthesisPhaseResult, TurnSegment,
};
pub use synthesis_phase::run_synthesis_pass;

/// Registers consolidation engine launcher with `memfuse-engine`.
pub fn init() {
    memfuse_engine::register_consolidation_launcher(
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
