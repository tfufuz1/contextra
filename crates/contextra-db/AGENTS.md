# AGENTS.md — `contextra-db`

> Werkzeug/Crate-Kontext für Agenten. Der obere Teil ist generiert; manuelle Notizen unterhalb ergänzen.

<!-- BEGIN GENERATED: crate_inventory.py -->

## Inventar (generiert)

- **Pfad:** `crates/contextra-db` · **Ring:** 3 · **Version:** 0.1.0
- **Beschreibung:** Contextra — Embedded hybrid-search for AI agents
- **src:** 94 Dateien, 18742 LOC · **unsafe:** 0 Vorkommen, Policy `forbid`
- **Interne Abhängigkeiten (normal, 14):** `contextra-adapt`, `contextra-checkpoint`, `contextra-cognition`, `contextra-graph`, `contextra-infer-onnx`, `contextra-mvcc`, `contextra-ports`, `contextra-rank`, `contextra-sandbox`, `contextra-store`, `contextra-text`, `contextra-types`, `contextra-vault`, `contextra-vector`
- **Interne Dev/Build-Abhängigkeiten (8):** `contextra-adapt`, `contextra-checkpoint`, `contextra-graph`, `contextra-ingest`, `contextra-store`, `contextra-testkit`, `contextra-text`, `contextra-vector`
- **Externe Abhängigkeiten (normal, 11):** `ahash`, `blake3`, `bytes`, `parking_lot`, `serde`, `serde_json`, `thiserror`, `tokio`, `tokio-util`, `tracing`, `zeroize`
- **Reverse-Abhängigkeiten (4):** `contextra`, `contextra-agent`, `contextra-bench`, `contextra-engine`
- **Kopplung an Hub-Crates:** ausgehend `contextra-ports`, `contextra-types`; eingehend `contextra`, `contextra-engine`
- **Features (17):** `adaptive-candidate-pool-sizing`, `auto-extraction-opt-out`, `background-maintenance`, `bench`, `cognition`, `coherence-bonus-fusion`, `default`, `edge-reinforcement-learning`, `entity-extraction`, `exp-extraction`, `experimental-diskann`, `graph-connectivity-health`, `license`, `onnx`, `reranking`, `sandbox`, `volatile-vault`

### Öffentliche API (grob, aus src/)
- struct (57): `AdaptiveDecayController`, `AutoExtractionConfig`, `CheckpointPinGuard`, `ChunkerConfig`, `Collection`, `CollectionConfig`, `CommitLedger`, `CommunityDetectionConfig`, `CompensateGraphAction`, `CompensateHnswAction`, `CompensateLsmAction`, `CompensateTextAction`, `Completed`, `Contextra`, `ContextraConfig` … (+42)
- enum (7): `CommitIntent`, `EmbeddingBackend`, `ExtractorMode`, `FilterOp`, `MetadataFilter`, `SearchStrategy`, `TenantPolicy`
- trait (7): `Backends`, `CompensatingAction`, `EntityExtractor`, `OllamaBridge`, `OrphanCleanupIndex`, `SandboxBridge`, `WorkflowStep`
- fn (266): `add_step`, `allocate_tx`, `anchor_entities`, `anchors`, `apply_temporal_validity_filter`, `apply_temporal_validity_filter_at`, `apply_temporal_validity_filter_with_clock`, `as_of`, `auto_extraction_config`, `auto_extraction_config_with_extractor`, `begin_transaction`, `budget_ms`, `calculate_rebuild_cooldown`, `chunk`, `chunk_text` … (+251)
- type (4): `ConsolidationLauncher`, `DbStats`, `FusionResult`, `TenantCollectionMap`
- const (36): `BACKPRESSURE_LEVEL`, `CHECKPOINT_DURATION`, `COMMIT_LATENCY`, `DECAY_DELETION_THRESHOLD`, `DEFAULT_COMMUNITY_BOOST`, `DEFAULT_MAX_SUB_QUERIES`, `DEFAULT_MIN_RERANK_CANDIDATES`, `DEFAULT_RERANK_POOL_MAX`, `DEFAULT_RERANK_POOL_MULTIPLIER`, `DEFAULT_SCAN_LIMIT`, `DEFAULT_TARGET_LATENCY_MS`, `HARD_SCAN_CEILING`, `INSERT_LATENCY`, `MAX_DEFERRED_HYPEREDGES_PER_TICK`, `MAX_EXPIRED_PER_TICK` … (+21)
- mod (39): `adapters`, `advanced`, `backends`, `background_workers`, `bridge`, `chunker`, `cognition`, `collection`, `contextra_impl`, `crud`, `decay_controller`, `export`, `extraction`, `filter`, `fusion` … (+24)
- pub use: `advanced::*`, `auto_extraction::{AutoExtractionConfig, AutoExtractionMode, EntityExtractionConfig}`, `backends::Backends`, `backends::DefaultBackends`, `background_workers::start_decay_cleanup_worker`, `background_workers::{start_expiry_cleanup_worker, start_orphan_cleanup_worker}`, `bridge::*`, `builder::{HybridQueryBuilder, DEFAULT_RERANK_POOL_MAX, DEFAULT_RERANK_POOL_MULTIPLIER}`, `checkpoint::{ with_pinned_checkpoint, with_pinned_checkpoint_at_latest, CheckpointPinGuard, }`, `cleanup::{cleanup_orphaned_consolidation_intents, recover_pending_intents}` … (+67)

Regeneriert mit `python3 tools/crate_inventory.py agents`. Manuelle Notizen stehen außerhalb dieses Blocks.

<!-- END GENERATED: crate_inventory.py -->
