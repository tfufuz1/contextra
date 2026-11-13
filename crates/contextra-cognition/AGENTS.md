# AGENTS.md — `contextra-cognition`

> Werkzeug/Crate-Kontext für Agenten. Der obere Teil ist generiert; manuelle Notizen unterhalb ergänzen.

<!-- BEGIN GENERATED: crate_inventory.py -->

## Inventar (generiert)

- **Pfad:** `crates/contextra-cognition` · **Ring:** 3 · **Version:** 0.1.0
- **Beschreibung:** Contextra — Memory consolidation, compaction, and context management
- **src:** 29 Dateien, 5943 LOC · **unsafe:** 0 Vorkommen, Policy `forbid`
- **Interne Abhängigkeiten (normal, 5):** `contextra-graph`, `contextra-ports`, `contextra-rank`, `contextra-text`, `contextra-types`
- **Interne Dev/Build-Abhängigkeiten (2):** `contextra-db`, `contextra-testkit`
- **Externe Abhängigkeiten (normal, 11):** `ahash`, `async-trait`, `blake3`, `bytes`, `parking_lot`, `serde`, `serde_json`, `thiserror`, `tokio`, `tokio-util`, `tracing`
- **Reverse-Abhängigkeiten (1):** `contextra-db`
- **Kopplung an Hub-Crates:** ausgehend `contextra-ports`, `contextra-types`; eingehend `contextra-db`
- **Features (3):** `default`, `edge-reinforcement-learning`, `graph-connectivity-health`

### Öffentliche API (grob, aus src/)
- struct (42): `AdaptiveDecayController`, `AggregationConfig`, `AggregationEdge`, `AggregationNode`, `AggregationPhaseResult`, `AlphaNode`, `Collection`, `CommunityStabilityTracker`, `CompactedContext`, `ConsolidationConfig`, `ConsolidationEngine`, `ConsolidationLockGuard`, `ConsolidationNodesGuard`, `ConsolidationPhaseResult`, `ConsolidationPipelineResult` … (+27)
- enum (3): `CognitionError`, `CommitIntent`, `CompactionStrategy`
- trait (8): `CognitiveInputChannel`, `CognitiveOutputSink`, `CollectionOps`, `ConsolidationSink`, `DecayPolicy`, `DocumentSource`, `HyperedgeSink`, `SuperEdgeSink`
- fn (97): `abort`, `active_agent_sessions`, `allocate_tx`, `begin_transaction`, `build_leanrag_inputs`, `cascade_invalidate_edges_ordered`, `check_compaction_budget`, `cleanup_orphaned_consolidation_intents`, `collected`, `collection`, `commit`, `commit_ref`, `compact`, `compact_segment_via_context_compactor`, `compute_community_hash` … (+82)
- type (1): `SynthesisPhaseResult`
- const (4): `DEFAULT_MAX_LEANRAG_NODES`, `MAX_PRE_CONDENSED_DIGEST_LEN`, `MAX_PRE_CONDENSED_LABEL_LEN`, `MAX_PRE_CONDENSED_PROVENANCE_COUNT`
- mod (21): `aggregation_phase`, `channel`, `collection`, `collection_types`, `consolidation_executor`, `consolidation_locks`, `context`, `context_compaction`, `decay`, `decay_controller`, `error`, `graph_sink`, `leanrag_input`, `maintenance_config`, `maintenance_scheduler` … (+6)
- pub use: `aggregation_phase::{ check_compaction_budget, compute_entity_community_hash, run_aggregation_pass, AggregationConfig, AggregationEdge, AggregationNode, AggregationPhaseResult, AlphaNode, ConsolidationPipelineResult, SuperEdgeDraft, SuperEdgeSink, }`, `channel::{ CognitiveInputChannel, CognitiveOutputSink, DirectCognitiveChannel, InMemoryCognitiveChannel, InMemoryOutputSink, }`, `cleanup::cleanup_orphaned_consolidation_intents`, `compactor::ContextCompactor`, `consolidation_executor::{ execute_background_consolidation, execute_consolidation_pass, execute_leanrag_aggregation_stage, execute_sleep_cycle, start_consolidation_reaper, start_consolidation_worker, ConsolidationEngine, }`, `consolidation_locks::ConsolidationNodesGuard`, `context::{ContextManager, SpatialFence}`, `context_compaction::{ cleanup_orphaned_consolidation_intents, CompactedContext, CompactionStrategy, ConsolidationSession, ContextCompactor, StatusToken, }`, `contextra_graph as graph`, `contextra_ports as ports` … (+28)

Regeneriert mit `python3 tools/crate_inventory.py agents`. Manuelle Notizen stehen außerhalb dieses Blocks.

<!-- END GENERATED: crate_inventory.py -->
