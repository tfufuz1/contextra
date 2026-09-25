// FILE-CONTEXT
// ZWECK: Contextra Cognition Engine (Layer 3 - Cognition).
// INVARIANTEN: No unsafe code; depends on contextra-engine; zero cyclic dependencies.
//!
//! # Contextra Cognition Engine
//!
//! Dieses Crate bietet Algorithmen für Speicher-Konsolidierung (Memory Consolidation),
//! Kontext-Kompaktierung, LeanRAG-Aggregation und Synthese-Phasen.
//!
//! ## Einbindung des Consolidation Workers
//!
//! Da `contextra-engine` (Layer 3 Engine / Ring 2) nicht von `contextra-cognition`
//! (Layer 3 Cognition / Ring 3) abhängen darf und kein globaler veränderlicher Zustand
//! erlaubt ist (Prinzip P29 / ADR-N10), wird der Consolidation-Worker instanzgebunden
//! über `ContextraConfig::with_consolidation_launcher` konfiguriert:
//!
//! ```rust,ignore
//! use std::sync::Arc;
//! use contextra_engine::{Contextra, ContextraConfig, ConsolidationLauncher};
//! use contextra_cognition::consolidation_executor::ConsolidationEngine;
//! use contextra_cognition::memory_consolidation::{ConsolidationConfig, SynthesisConfig};
//!
//! let launcher: ConsolidationLauncher = Arc::new(|col, interval, max_llm_calls, cancel_token| {
//!     let engine = Arc::new(ConsolidationEngine::new(
//!         col,
//!         ConsolidationConfig::default(),
//!         SynthesisConfig {
//!             max_llm_calls_per_cycle: max_llm_calls as u32,
//!             ..Default::default()
//!         },
//!         interval,
//!         cancel_token,
//!     ));
//!     engine.start()
//! });
//!
//! let config = ContextraConfig::default().with_consolidation_launcher(launcher);
//! let db = Contextra::open_with_config("/path/to/db", config).await?;
//! ```

#![forbid(unsafe_code)]

pub mod consolidation_executor;
pub mod consolidation_locks;
pub mod context;
pub mod context_compaction;
pub mod maintenance_config;
pub mod maintenance_scheduler;
pub mod aggregation_phase;
pub mod graph_sink;
pub mod leanrag_input;
pub mod memory_consolidation;
pub mod synthesis_phase;

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
pub use maintenance_config::MaintenanceConfig;
pub use maintenance_scheduler::MaintenanceScheduler;
pub use memory_consolidation::{
    compact_segment_via_context_compactor, compute_community_hash, detect_near_duplicates,
    group_turns_into_segments, run_consolidation_pass, CommunityStabilityTracker,
    ConsolidationConfig, ConsolidationPhaseResult, MetaChunk, SynthesisConfig,
    SynthesisPhaseResult, TurnSegment,
};
pub use aggregation_phase::{
    check_compaction_budget, compute_entity_community_hash, run_aggregation_pass,
    AggregationConfig, AggregationEdge, AggregationNode, AggregationPhaseResult, AlphaNode,
    ConsolidationPipelineResult, SuperEdgeDraft, SuperEdgeSink,
};
pub use graph_sink::CsrGraphSuperEdgeSink;
pub use leanrag_input::{build_leanrag_inputs, LeanRagInputs, DEFAULT_MAX_LEANRAG_NODES};
pub use synthesis_phase::run_synthesis_pass;
