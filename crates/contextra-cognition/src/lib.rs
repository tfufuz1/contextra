//! # Kognitions-Engine (`contextra-cognition`)
//!
//! Die Crate `contextra-cognition` bildet die Kognitions- und Konsolidierungsschicht
//! im Contextra-Workspace (historisch Ring 3, adaptiert nach Spezifikation §4/§5.2).
//! Sie stellt Hintergrundprozesse für Gedächtniskonsolidierung (Memory Consolidation),
//! Kontext-Kompaktierung (Context Compaction), semantische Aggregation (LeanRAG / Hyperedge Consolidation)
//! und Wartungsplanung (Maintenance Scheduling) bereit.
//!
//! ## Ring-Zugehörigkeit und Abhängigkeiten
//! - **Abhängigkeiten (eingehend):** Wird von höheren Orchestrierungsschichten wie `contextra-db` eingebunden.
//! - **Abhängigkeiten (ausgehend):** Nutzt Basis- und Speicher-Crates:
//!   - `contextra-types`: Domänentypen, Fehlermodelle, Mandanten-IDs.
//!   - `contextra-ports`: Abstrakte Port-Schnittstellen (Storage, Clock, Rng, Metrics).
//!   - `contextra-engine`: Ausführungs- und Storage-Subsysteme.
//!   - `contextra-store`: Schlüssel-Wert- und LSM-Speicher (Key-Value-Store).
//!   - `contextra-vector`: Vektorindizes und Embeddings.
//!   - `contextra-graph`: CSR-Graphdarstellung und Hyperedges.
//!
//! ## Kernkomponenten
//! - [`ContextManager`]: Verwaltet Kontexte, Raumgrenzen (SpatialFence) und Token-Budgets.
//! - [`ContextCompactor`]: Führt Transaktions- und Dialogkompaktierung durch.
//! - [`ConsolidationEngine`]: Hintergrund-Worker für Schlafzyklen (Sleep Cycles) und semantische Verschmelzung.
//! - [`MaintenanceScheduler`]: Steuert periodische Wartungsläufe und Speicherbereinigungen.
//!
//! ## Invarianten
//! - **Kein Unsafe-Code:** `#![forbid(unsafe_code)]` wird strikt auf Crate-Ebene erzwungen.
//! - **Mandantentrennung:** Strikte Isolierung nach `tenant_id` (VETO-F10).
//! - **Deterministische Sortierung:** Alle aggregierten und vorverdichteten Strukturen werden deterministisch sortiert.

#![deny(dead_code)]
// FILE-CONTEXT
// ZWECK: Contextra Cognition Engine (Layer 3 - Cognition).
// INVARIANTEN: No unsafe code; depends on contextra-engine; zero cyclic dependencies.
// VORVERDICHTETER EINGANG: Agentenunabhängige Schnittstelle (`PreCondensedInput`, `intake_pre_condensed_inputs`)
//                          für vorverdichtete Eingabeknoten (z. B. ScratchpadCheckpoints aus Agenten-Pipelines).
//                          Strikte Mandantentrennung (VETO-F10), Provenienzerhalt, Längenbeschränkung und
//                          deterministische Sortierung nach tenant_id und text_digest.
#![forbid(unsafe_code)]

// ANCHOR[contextra-cognition:ROOT-001]
/// Modul für Aggregationsphase (LeanRAG Stage 3).
pub mod aggregation_phase;
/// Modul für die Ausführung der Konsolidierung.
pub mod consolidation_executor;
/// Modul für Konsolidierungs-Locks und Kaskaden-Invalidierung.
pub mod consolidation_locks;
/// Modul für autonome Kontextverwaltung und Token-Budgetierung.
pub mod context;
/// Modul für Kontext-Kompaktierung.
pub mod context_compaction;
/// Modul für Graph-Senken (Super-Hyperkanten).
pub mod graph_sink;
/// Modul zur Vorbereitung von LeanRAG-Eingabedaten.
pub mod leanrag_input;
/// Modul für Wartungskonfigurationen.
pub mod maintenance_config;
/// Modul für periodische Wartungsplanung.
pub mod maintenance_scheduler;
/// Modul für Gedächtniskonsolidierung (Structural Pass).
pub mod memory_consolidation;
/// Modul für die Fassadenschnittstelle zur semantischen Aggregation.
pub mod semantic_aggregation_facade;
/// Modul für generative Wissenssynthese (LLM-Pass).
pub mod synthesis_phase;
/// Modul für Transitivitäts-Veto-Prüfungen.
pub mod transitivity_veto;

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
    group_turns_into_segments, intake_pre_condensed_inputs, run_consolidation_pass,
    CommunityStabilityTracker, ConsolidationConfig, ConsolidationPhaseResult, MetaChunk,
    PreCondensedInput, SynthesisConfig, SynthesisPhaseResult, TurnSegment,
    MAX_PRE_CONDENSED_DIGEST_LEN, MAX_PRE_CONDENSED_LABEL_LEN, MAX_PRE_CONDENSED_PROVENANCE_COUNT,
};
pub use semantic_aggregation_facade::consolidate_semantic_hyperedges;
pub use synthesis_phase::run_synthesis_pass;
pub use transitivity_veto::{filter_candidates_with_transitivity_veto, validate_transitivity_veto};
