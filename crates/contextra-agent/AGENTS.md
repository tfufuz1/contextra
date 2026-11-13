# AGENTS.md — `contextra-agent`

> Werkzeug/Crate-Kontext für Agenten. Der obere Teil ist generiert; manuelle Notizen unterhalb ergänzen.

<!-- BEGIN GENERATED: crate_inventory.py -->

## Inventar (generiert)

- **Pfad:** `crates/contextra-agent` · **Ring:** 3 · **Version:** 0.1.0
- **Beschreibung:** Persistent agent workflow engine for Contextra — checkpoint/execute/audit loop
- **src:** 22 Dateien, 4426 LOC · **unsafe:** 0 Vorkommen, Policy `forbid`
- **Interne Abhängigkeiten (normal, 5):** `contextra-checkpoint`, `contextra-db`, `contextra-ports`, `contextra-router`, `contextra-types`
- **Interne Dev/Build-Abhängigkeiten (5):** `contextra-graph`, `contextra-store`, `contextra-testkit`, `contextra-text`, `contextra-vector`
- **Externe Abhängigkeiten (normal, 7):** `bytes`, `serde`, `serde_json`, `thiserror`, `tokio`, `tokio-util`, `tracing`
- **Reverse-Abhängigkeiten (0):** –
- **Kopplung an Hub-Crates:** ausgehend `contextra-db`, `contextra-ports`, `contextra-types`; eingehend –
- **Features (2):** `default`, `test-utils`

### Öffentliche API (grob, aus src/)
- struct (29): `AgentContext`, `AgentEngine`, `AgentNode`, `AuditEntry`, `AuditLog`, `BackgroundEvent`, `ClmScratchpad`, `ContextEditAuditRecord`, `CountingContextEditAuditSink`, `CountingScratchpadCacheInvalidator`, `DbCollectionStore`, `DeadLetterQueue`, `InMemoryScratchpadStore`, `MigrationStats`, `NoopContextEditAuditSink` … (+14)
- enum (7): `AgentError`, `AgentStatus`, `DeadLetterReason`, `EventLoopExitReason`, `GoalCondition`, `NodeType`, `ScratchpadEditOp`
- trait (8): `AgentCheckpointStore`, `AgentTool`, `ContextEditAuditSink`, `ContextRetriever`, `ContextStore`, `EventSource`, `ScratchpadCacheInvalidator`, `ScratchpadStore`
- fn (81): `add_edge`, `add_node`, `allocate_tx`, `append`, `append_to`, `apply_edit`, `as_str`, `attach_event`, `authorize`, `checkpoint`, `checkpoint_and_reset`, `contains`, `count`, `created_at_tx`, `current_subgoal_index` … (+66)
- type (1): `NodeId`
- const (8): `MAX_EVENT_SOURCE_CAPACITY`, `MAX_ID_LEN`, `MAX_MANIFEST_DESC_LEN`, `MAX_PENDING_EVENTS_CAPACITY`, `MAX_TELEMETRY_EVENTS`, `MAX_TEXT_LEN`, `MAX_WORKFLOW_STEPS`, `PREFIX`
- mod (18): `audit`, `budget`, `clm_scratchpad`, `compat`, `context`, `dlq`, `engine`, `error`, `event_source`, `goal_condition`, `graph`, `manifest`, `ports`, `registry`, `retrieve` … (+3)
- pub use: `budget::{BudgetStrategy, Reservation, TokenBudget}`, `clm_scratchpad::{ ClmScratchpad, ContextEditAuditSink, CountingContextEditAuditSink, CountingScratchpadCacheInvalidator, NoopContextEditAuditSink, NoopScratchpadCacheInvalidator, PinnedRegionId, ScratchpadCacheInvalidator, ScratchpadCheckpoint, ScratchpadEditOp, }`, `condition::evaluate_condition_expr`, `context::{AgentContext, AgentStatus}`, `contextra_types::types::budget::{ BudgetStrategy, Reservation, ResourceBudget, ResourceTracker, TokenBudget, }`, `dlq::DeadLetterQueue`, `engine::{EventLoopExitReason, OrchestratorEngine, MAX_WORKFLOW_STEPS}`, `error::AgentError`, `event_source::{BackgroundEvent, EventSource, PollingDocumentEventSource, VecEventSource}`, `goal_condition::{parse_legacy_condition_string, GoalCondition}` … (+7)

Regeneriert mit `python3 tools/crate_inventory.py agents`. Manuelle Notizen stehen außerhalb dieses Blocks.

<!-- END GENERATED: crate_inventory.py -->
