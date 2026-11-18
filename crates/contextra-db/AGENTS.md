# AGENTS.md — contextra-db — Integrationsmotor

**Status:** Normativ  
**Crate-Spezifikation:** [`docs/crates/contextra-db.md`](file:///home/freddy/Projekte/contextra/docs/crates/contextra-db.md)  
**Gesamtspezifikation:** [`docs/SPEC.md`](file:///home/freddy/Projekte/contextra/docs/SPEC.md) — Bindung: docs/SPEC.md §2 (Architektur), §3 (Datenklassifikation), §4 (Schlüsselhierarchie), §5 (Löschkette), §6 (ReadGate), §7 (ErasureLayer), §8 (Lineage), §9 (Zeitreise), §10 (Barrieren), §11 (Fehlerverhalten), §13 (Invarianten), §14 (Konfiguration)  
**Workspace-Richtlinie:** [`AGENTS.md`](file:///home/freddy/Projekte/contextra/AGENTS.md) (Wurzel)  
**Ebene:** E2 (Integrationsmotor & Orchestrierung)  
**Kern-Closure:** Bündelt alle E1-Bausteine (Store, Vector, Text, Graph, Rank, Checkpoint, MVCC) und E0/E2-Typen  
**Unsafe-Status:** Verboten (`#![forbid(unsafe_code)]`)  

---

## 1. Zweck & Systemrolle

Integrationsmotor: Multi-Index-Orchestrierung, 4-Signal Hybrid-Search, 2PC Transaktionen, ReadGate-Durchsetzung und bitemporale Zeitreise.

---

## 2. Kern-Richtlinien für Agenten

- **Multi-Index-Orchestrierung:** Koordiniert LSM-Store, HNSW-Vektorindex, BM25-Volltext und Wissensgraph.
- **Transaktionen (2PC):** Atomare Updates über alle Indizes mit `TxBuffer`, `KvLockTable`, SSI-Validierung und `CommitBarrier::shared()`.
- **Durchsetzung des ReadGate (SPEC §6.2):** An ausnahmslos jedem Lesepfad (`get`, `scan`, `search`, `hybrid_search`, `graph_query`, `export`) MUSS vor Entschlüsselung und Rückgabe synchron das `ReadGate` geprüft werden (Fail-Closed).
- **Hybrid-Search:** 4 Signale (Vektor, Volltext, Graph, Rerank) mit RRF-Fusion und Auffüllen auf $k$.
- **Bitemporale Zeitreise:** Unabhängige Trennung von Transaktionszeit (`as_of`) und Gültigkeitszeit (`valid_at`). Gate-Prüfung erfolgt stets im **aktuellen Stand** (Z-T5).
- **Hintergrund-Worker:** `ExpiryCleanupWorker`, `OrphanCleanupWorker`, `DecayCleanupWorker`, `PidLatencyController`.

## 3. Test-Pflichten
- T-DB1 bis T-DB10 (Roundtrip, Hybrid-Search RRF, ReadGate Ok(None), Auffüllen auf k, Zeitreise Z-T5, Gültigkeitsfilter, 2PC Atomarität, Mandantenisolation, Drop Collection, Crash-Recovery).

---

<!-- BEGIN GENERATED: crate_inventory.py -->

## Inventar (generiert)

- **Pfad:** `crates/contextra-db` · **Ring:** 3 · **Version:** 0.1.0
- **Beschreibung:** Contextra — Embedded hybrid-search for AI agents
- **src:** 94 Dateien, 18505 LOC · **unsafe:** 0 Vorkommen, Policy `forbid`
- **Interne Abhängigkeiten (normal, 9):** `contextra-checkpoint`, `contextra-graph`, `contextra-mvcc`, `contextra-ports`, `contextra-rank`, `contextra-store`, `contextra-text`, `contextra-types`, `contextra-vector`
- **Interne Dev/Build-Abhängigkeiten (6):** `contextra-checkpoint`, `contextra-graph`, `contextra-store`, `contextra-testkit`, `contextra-text`, `contextra-vector`
- **Externe Abhängigkeiten (normal, 11):** `ahash`, `blake3`, `bytes`, `parking_lot`, `serde`, `serde_json`, `thiserror`, `tokio`, `tokio-util`, `tracing`, `zeroize`
- **Reverse-Abhängigkeiten (3):** `contextra`, `contextra-bench`, `contextra-engine`
- **Kopplung an Hub-Crates:** ausgehend `contextra-ports`, `contextra-types`; eingehend `contextra`, `contextra-engine`
- **Features (13):** `auto-extraction-opt-out`, `background-maintenance`, `bench`, `coherence-bonus-fusion`, `default`, `edge-reinforcement-learning`, `entity-extraction`, `exp-extraction`, `experimental-diskann`, `graph-connectivity-health`, `license`, `reranking`, `volatile-vault`

### Öffentliche API (grob, aus src/)
- struct (57): `AdaptiveDecayController`, `AutoExtractionConfig`, `CheckpointPinGuard`, `ChunkerConfig`, `Collection`, `CollectionConfig`, `CommitLedger`, `CommunityDetectionConfig`, `CompensateGraphAction`, `CompensateHnswAction`, `CompensateLsmAction`, `CompensateTextAction`, `Completed`, `Contextra`, `ContextraConfig` … (+42)
- enum (7): `CommitIntent`, `EmbeddingBackend`, `ExtractorMode`, `FilterOp`, `MetadataFilter`, `SearchStrategy`, `TenantPolicy`
- trait (7): `Backends`, `CompensatingAction`, `EntityExtractor`, `OllamaBridge`, `OrphanCleanupIndex`, `SandboxBridge`, `WorkflowStep`
- fn (265): `add_step`, `allocate_tx`, `anchor_entities`, `anchors`, `apply_temporal_validity_filter`, `apply_temporal_validity_filter_at`, `apply_temporal_validity_filter_with_clock`, `as_of`, `auto_extraction_config`, `auto_extraction_config_with_extractor`, `begin_transaction`, `budget_ms`, `calculate_rebuild_cooldown`, `chunk`, `chunk_text` … (+250)
- type (4): `ConsolidationLauncher`, `DbStats`, `FusionResult`, `TenantCollectionMap`
- const (36): `BACKPRESSURE_LEVEL`, `CHECKPOINT_DURATION`, `COMMIT_LATENCY`, `DECAY_DELETION_THRESHOLD`, `DEFAULT_COMMUNITY_BOOST`, `DEFAULT_MAX_SUB_QUERIES`, `DEFAULT_MIN_RERANK_CANDIDATES`, `DEFAULT_RERANK_POOL_MAX`, `DEFAULT_RERANK_POOL_MULTIPLIER`, `DEFAULT_SCAN_LIMIT`, `DEFAULT_TARGET_LATENCY_MS`, `HARD_SCAN_CEILING`, `INSERT_LATENCY`, `MAX_DEFERRED_HYPEREDGES_PER_TICK`, `MAX_EXPIRED_PER_TICK` … (+21)
- mod (39): `adapters`, `advanced`, `backends`, `background_workers`, `bridge`, `chunker`, `cognition`, `collection`, `contextra_impl`, `crud`, `decay_controller`, `export`, `extraction`, `filter`, `fusion` … (+24)
- pub use: `advanced::*`, `auto_extraction::{AutoExtractionConfig, AutoExtractionMode, EntityExtractionConfig}`, `backends::Backends`, `backends::DefaultBackends`, `background_workers::start_decay_cleanup_worker`, `background_workers::{start_expiry_cleanup_worker, start_orphan_cleanup_worker}`, `bridge::*`, `builder::{HybridQueryBuilder, DEFAULT_RERANK_POOL_MAX, DEFAULT_RERANK_POOL_MULTIPLIER}`, `checkpoint::{ with_pinned_checkpoint, with_pinned_checkpoint_at_latest, CheckpointPinGuard, }`, `cleanup::{cleanup_orphaned_consolidation_intents, recover_pending_intents}` … (+66)

Regeneriert mit `python3 tools/crate_inventory.py agents`. Manuelle Notizen stehen außerhalb dieses Blocks.

<!-- END GENERATED: crate_inventory.py -->
