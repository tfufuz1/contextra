# Kanten-Inventar Archiv-Crates — WP 2.1

<!-- doc-stamp: 014ee79f0 | Generiert: tools/generate_migration_edges.py -->

> **Zweck.** Dieses Dokument listet alle Kanten von aktiven Crates zu Archiv-Crates.
> Es ist die Abarbeitungsliste für WP 2.2, WP 2.3 und WP 2.4.
> Erstellt und aktualisiert in WP 2.1 gemäß PLAN.md §7 (WP 2.1, Schritt 2 & 3).

## Status

| WP | Kanten-Ziel | Status |
|---|---|---|
| WP 2.2 | `engine` entkoppeln | 🔴 offen |
| WP 2.3 | `db`, `mcp`, `contextra`, `core` entkoppeln | 🔴 offen |
| WP 2.4 | Rand-Crates als optional markieren | 🔴 offen |
| WP 2.1b | `bench` & Archiv-Crates (physisches Entfernen) | 🔴 offen |

---

## Ratchet-Baseline: Übersicht nach Dependent

> **Ratchet-Regel:** Die Summen in dieser Tabelle dürfen in WP 2.2, 2.3 und 2.4 nur sinken.

| Dependent | Manifest-Kanten | Quellcode-Dateien (src / tests / gesamt) | Archiv-Kanten | Ziel-WP |
|---|---|---|---|---|
| contextra | 4 | 8 / 9 / **17** | 4 | WP 2.3 |
| contextra-agent | 2 | 3 / 1 / **4** | 2 | WP 2.1b |
| contextra-bench | 3 | 6 / 2 / **8** | 3 | WP 2.1b |
| contextra-checkpoint | 1 | 2 / 1 / **3** | 1 | WP 2.3 |
| contextra-cognition | 1 | 8 / 7 / **15** | 1 | WP 2.1b |
| contextra-db | 3 | 6 / 15 / **21** | 3 | WP 2.3 |
| contextra-engine | 1 | 23 / 10 / **33** | 5 | WP 2.2 |
| contextra-graph | 1 | 1 / 0 / **1** | 1 | WP 2.1b |
| contextra-infer-ollama | 1 | 1 / 0 / **1** | 1 | WP 2.1b |
| contextra-mcp | 4 | 3 / 0 / **3** | 4 | WP 2.3 |
| contextra-py | 3 | 4 / 0 / **4** | 3 | WP 2.3 |
| contextra-rank | 1 | 0 / 2 / **2** | 1 | WP 2.3 |
| contextra-router | 2 | 5 / 3 / **8** | 2 | WP 2.1b |
| contextra-simd | 1 | 2 / 0 / **2** | 1 | WP 2.3 |
| contextra-store | 2 | 51 / 109 / **160** | 2 | WP 2.3 |
| contextra-types | 0 | 2 / 0 / **2** | 2 | WP 2.3 |
| contextra-vector | 1 | 34 / 58 / **92** | 1 | WP 2.3 |

---

## contextra-adapt

| Dependent | Manifest | Art | Optional | Quellcode-Dateien (src / tests) | Top-10 Symbole | Zu erledigen in | Hinweis |
|---|---|---|---|---|---|---|---|
| contextra-db | ja | normal | nein | 3 / 1 (4) | PidController, RerankDeadline, RerankPidController, decay_controller, pid_latency_controller, new | WP 2.3 | - |
| contextra-engine | nein | nein | nein | 3 / 0 (3) | PidController, default | WP 2.2 | **nur Quellcode** (keine Manifest-Kante) |
| contextra-graph | ja | normal | nein | 1 / 0 (1) | ShadowDiscrepancy, ShadowSink | WP 2.1b | - |
| contextra-mcp | ja | normal | nein | 1 / 0 (1) | PidController, default | WP 2.3 | - |
| contextra-py | ja | normal | nein | 0 / 0 (0) | - | WP 2.3 | - |
| contextra-rank | ja | dev | nein | 0 / 2 (2) | PidController | WP 2.3 | - |
| contextra-router | ja | normal | nein | 4 / 3 (7) | BanditProfileState, FcTsArmSet, bandit, lyapunov, offpolicy, BanditError, RandomizedLoggingPolicy, FcTsError, FcTsRng, SplitMix64 | WP 2.1b | - |

**Maßnahme:** `AdaptiveController` durch feste Startwerte ersetzen: `efSearch = 64`, `exact_scan_max = 20_000`, Overfetch `3 / 6 / 12` (PLAN.md §7, WP 2.2 Schritt 3).

---

## contextra-agent

| Dependent | Manifest | Art | Optional | Quellcode-Dateien (src / tests) | Top-10 Symbole | Zu erledigen in | Hinweis |
|---|---|---|---|---|---|---|---|
| contextra-mcp | ja | normal | ja | 0 / 0 (0) | - | WP 2.3 | - |

**Maßnahme:** Feature `agent-workflows` entfernen; optionale Kante aus `mcp/Cargo.toml` löschen.

---

## contextra-cognition

| Dependent | Manifest | Art | Optional | Quellcode-Dateien (src / tests) | Top-10 Symbole | Zu erledigen in | Hinweis |
|---|---|---|---|---|---|---|---|
| contextra-db | ja | normal | nein | 1 / 0 (1) | execute_sleep_cycle, context, context_compaction, memory_consolidation, synthesis_phase, consolidation_executor, consolidation_locks, maintenance_scheduler, maintenance_config, start_consolidation_reaper | WP 2.3 | - |

**Maßnahme:** Kante aus `db/Cargo.toml` löschen; betroffene Module in `db/src/` entfernen oder stubben.

---

## contextra-core

| Dependent | Manifest | Art | Optional | Quellcode-Dateien (src / tests) | Top-10 Symbole | Zu erledigen in | Hinweis |
|---|---|---|---|---|---|---|---|
| contextra | ja | normal | nein | 5 / 9 (14) | ContextraError, Result, DistanceMetric, TenantId, BoxFuture, AutoExtractionMode, domain, TxId | WP 2.3 | - |
| contextra-bench | ja | normal | nein | 5 / 2 (7) | Result, VectorIndex, Internal, SnapshotUnsupportedForSignal, types, ContextraError, StorageEngine, EmbeddingProvider, CapabilityUnsupported, TxId | WP 2.1b | - |
| contextra-checkpoint | ja | normal | nein | 2 / 1 (3) | SnapshotRegistry | WP 2.3 | - |
| contextra-engine | ja | dev | nein | 0 / 1 (1) | ResourceBudget, ResourceTracker, SnapshotRegistry | WP 2.2 | - |
| contextra-py | ja | normal | nein | 4 / 0 (4) | create, ContextraError, domain, ScoredDocumentArgs, SearchResponseArgs, new, from | WP 2.3 | - |
| contextra-simd | ja | normal | nein | 2 / 0 (2) | Result, ContextraError, DistanceMetric | WP 2.3 | - |
| contextra-store | ja | normal | nein | 51 / 109 (160) | TxId, StorageEngine, ContextraError, Result, ResourceBudget, new, TOMBSTONE_BIT, ResourceTracker, SnapshotRegistry, Storage | WP 2.3 | - |
| contextra-vector | ja | normal | nein | 34 / 58 (92) | DocId, VectorIndex, DistanceMetric, TxId, Result, ContextraError, invalid_input, types, MAX_SEARCH_K, ScoredDocument | WP 2.3 | - |

**Maßnahme:** Mechanische Ersetzung `use contextra_core::` → `use contextra_types::` (K-11: ~286 Dateien). Nach `cargo check` Crate und Eintrag in `Cargo.toml` löschen (D-05).

---

## contextra-graph

| Dependent | Manifest | Art | Optional | Quellcode-Dateien (src / tests) | Top-10 Symbole | Zu erledigen in | Hinweis |
|---|---|---|---|---|---|---|---|
| contextra-agent | ja | normal | nein | 1 / 1 (2) | CsrGraph | WP 2.1b | - |
| contextra-bench | ja | normal | nein | 0 / 0 (0) | - | WP 2.1b | - |
| contextra-cognition | ja | normal | nein | 8 / 7 (15) | CsrGraph, EdgeType, hyperedge, HyperEdgeId, EdgeReinforcementBuffer, PercolationConfig, CascadeInvalidationReport, NodesGuard, EdgeReinforcementConfig, default | WP 2.1b | - |
| contextra-db | ja | normal | nein | 2 / 14 (16) | CsrGraph, PathRAGEngine, PercolationConfig | WP 2.3 | - |
| contextra-engine | nein | nein | nein | 18 / 8 (26) | CsrGraph, HyperEdgeId, new, PercolationConfig, csr, HyperEdge, load_from_storage, detect_communities, CommunityAssignment, CommunityDetectionConfig | WP 2.2 | **nur Quellcode** (keine Manifest-Kante) |
| contextra-router | ja | dev | nein | 1 / 0 (1) | new | WP 2.1b | - |
| contextra-store | ja | dev | nein | 0 / 0 (0) | - | WP 2.3 | - |
| contextra-types | nein | nein | nein | 1 / 0 (1) | TlHfdParams | WP 2.3 | **nur Quellcode** (keine Manifest-Kante) |

**Maßnahme:** `engine/src/collection/crud/links.rs` löschen; graph-bezogene Funktionen (`stage_graph_entity_delete`, `check_and_trigger_community_detection`, `CommunityDetectionConfig`, PPR-Pfade im Hybrid-Planner) entfernen; Modalität als `modalities_unavailable` markieren.

---

## contextra-infer-ollama

| Dependent | Manifest | Art | Optional | Quellcode-Dateien (src / tests) | Top-10 Symbole | Zu erledigen in | Hinweis |
|---|---|---|---|---|---|---|---|
| contextra | ja | normal | ja | 1 / 0 (1) | - | WP 2.3 | - |
| contextra-mcp | ja | normal | ja | 1 / 0 (1) | new, OllamaConfig, with_config | WP 2.3 | - |

**Maßnahme:** Feature `ollama` bleibt in Fassade, aber Kante nur noch über Feature-Gate.

---

## contextra-infer-onnx

| Dependent | Manifest | Art | Optional | Quellcode-Dateien (src / tests) | Top-10 Symbole | Zu erledigen in | Hinweis |
|---|---|---|---|---|---|---|---|
| contextra | ja | normal | ja | 1 / 0 (1) | - | WP 2.3 | - |
| contextra-bench | ja | normal | nein | 1 / 0 (1) | CrossEncoderReranker, RerankConfig | WP 2.1b | - |
| contextra-engine | nein | nein | nein | 1 / 0 (1) | ensure_onnx_model_download, load | WP 2.2 | **nur Quellcode** (keine Manifest-Kante) |
| contextra-infer-ollama | ja | dev | nein | 1 / 0 (1) | MAX_EMBED_BATCH_SIZE | WP 2.1b | - |
| contextra-mcp | ja | normal | ja | 1 / 0 (1) | from_path | WP 2.3 | - |

**Maßnahme:** Feature `onnx` bleibt in Fassade, aber Kante nur noch über Feature-Gate.

---

## contextra-kvcache

| Dependent | Manifest | Art | Optional | Quellcode-Dateien (src / tests) | Top-10 Symbole | Zu erledigen in | Hinweis |
|---|---|---|---|---|---|---|---|
| contextra-engine | nein | nein | nein | 1 / 1 (2) | new, EvictionWorker, TenantIsolatedKvStore | WP 2.2 | **nur Quellcode** (keine Manifest-Kante) |

**Maßnahme:** `engine/src/kv_cache_integration/` löschen; `Collection::with_kv_store`, `set_kv_store`, `kv_store`, `start_kv_cache_eviction`, `TenantIsolatedKvStore`-Reste entfernen; Layer `KvCache` → `FeatureDisabled`.

---

## contextra-router

| Dependent | Manifest | Art | Optional | Quellcode-Dateien (src / tests) | Top-10 Symbole | Zu erledigen in | Hinweis |
|---|---|---|---|---|---|---|---|
| contextra | ja | normal | ja | 1 / 0 (1) | - | WP 2.3 | - |
| contextra-agent | ja | normal | nein | 2 / 0 (2) | Success, Rejected, DefaultRouterEngine, DecisionId | WP 2.1b | - |
| contextra-py | ja | normal | nein | 0 / 0 (0) | - | WP 2.3 | - |
| contextra-types | nein | nein | nein | 1 / 0 (1) | RoutingStrategy | WP 2.3 | **nur Quellcode** (keine Manifest-Kante) |

**Maßnahme:** Feature `router` in Fassade entfernen; `contextra-py` anpassen.

---

## Zusammenfassung

- **Manifest-Kanten zu Archiv-Crates:** 31
- **Kanten gesamt (inkl. nur Quellcode):** 37
- **Betroffene Quellcode-Dateien gesamt:** 376
