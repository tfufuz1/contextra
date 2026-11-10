# Crate-Kopplungsbericht

- Erzeugt: 2026-11-10 00:41
- Workspace-Root: `/app`
- Analysierte Crates: 36 · Komponenten-Tiefe: 1 · Zeilen (ohne Leerzeilen): 373.368
- Legende: **prod** = src/bin/build ohne `cfg(test)`; **dev** = tests/benches/examples/fuzz und `cfg(test)`-Code.
- Hinweis: Textuelle Analyse ohne Compiler. Glob-Importe (`*`) sind nicht auflösbar. Siehe Kopf des Skripts für Grenzen.

## 1. Überblick

| Crate | Ring | Zeilen | Fan-out prod | Fan-in prod | Cargo-Deps intern (n/d/b) | Pub-Items | API-Lecks | Zyklus |
|---|---|---:|---:|---:|---|---:|---:|---|
| [contextra](#contextra) | 4 | 3372 | 4 | 2 | 4/0/0 | 25 | 15 | – |
| [contextra-adapt](#contextra-adapt) | 0 | 6872 | 0 | 2 | 0/0/0 | 68 | 0 | – |
| [contextra-agent](#contextra-agent) | 3 | 9539 | 4 | 0 | 6/4/0 | 53 | 8 | – |
| [contextra-audit-export](#contextra-audit-export) | 0 | 1645 | 1 | 0 | 1/0/0 | 18 | 2 | – |
| [contextra-avv-generator](#contextra-avv-generator) | 0 | 1538 | 0 | 0 | 0/0/0 | 17 | 0 | – |
| [contextra-bench](#contextra-bench) | tooling | 4774 | 4 | 0 | 4/1/0 | 53 | 3 | – |
| [contextra-checkpoint](#contextra-checkpoint) | 1 | 8557 | 2 | 1 | 2/1/0 | 1 | 0 | – |
| [contextra-cognition](#contextra-cognition) | 3 | 9556 | 4 | 0 | 4/1/0 | 64 | 12 | – |
| [contextra-crypto](#contextra-crypto) | 0 | 16267 | 2 | 4 | 2/1/0 | 110 | 10 | – |
| [contextra-db](#contextra-db) | 3 | 12245 | 3 | 2 | 3/7/0 | 20 | 7 | – |
| [contextra-durable-fs](#contextra-durable-fs) | 0 | 2750 | 0 | 0 | 0/0/0 | 49 | 0 | – |
| [contextra-engine](#contextra-engine) | 3 | 26359 | 11 | 2 | 12/2/0 | 100 | 33 | – |
| [contextra-erasure](#contextra-erasure) | 3 | 20048 | 0 | 0 | 0/0/0 | 339 | 0 | – |
| [contextra-graph](#contextra-graph) | 0 | 28673 | 2 | 2 | 2/0/0 | 138 | 22 | – |
| [contextra-infer-candle](#contextra-infer-candle) | 2 | 7269 | 4 | 3 | 4/1/0 | 48 | 2 | – |
| [contextra-infer-ollama](#contextra-infer-ollama) | 2 | 6311 | 2 | 1 | 2/0/0 | 56 | 1 | – |
| [contextra-infer-onnx](#contextra-infer-onnx) | 2 | 966 | 0 | 2 | 0/0/0 | 15 | 0 | – |
| [contextra-ingest](#contextra-ingest) | 4 | 26371 | 0 | 0 | 0/0/0 | 143 | 0 | – |
| [contextra-kvcache](#contextra-kvcache) | 1 | 8578 | 3 | 0 | 3/0/0 | 66 | 5 | – |
| [contextra-license](#contextra-license) | 4 | 2997 | 2 | 0 | 2/0/0 | 26 | 1 | – |
| [contextra-mcp](#contextra-mcp) | 4 | 8089 | 10 | 0 | 11/0/0 | 33 | 12 | – |
| [contextra-mvcc](#contextra-mvcc) | 0 | 6532 | 1 | 4 | 2/1/0 | 22 | 0 | – |
| [contextra-ports](#contextra-ports) | 0 | 6843 | 1 | 21 | 1/0/0 | 145 | 1 | – |
| [contextra-privacy](#contextra-privacy) | 3 | 7821 | 0 | 1 | 0/0/0 | 83 | 0 | – |
| [contextra-py](#contextra-py) | 4 | 3253 | 3 | 0 | 3/0/0 | 37 | 3 | – |
| [contextra-rank](#contextra-rank) | 0 | 7660 | 2 | 4 | 2/0/0 | 39 | 7 | – |
| [contextra-router](#contextra-router) | 3 | 8958 | 3 | 2 | 4/0/0 | 23 | 3 | – |
| [contextra-sandbox](#contextra-sandbox) | 2 | 3980 | 0 | 0 | 0/0/0 | 14 | 0 | – |
| [contextra-simd](#contextra-simd) | 0 | 5137 | 1 | 0 | 1/0/0 | 35 | 1 | – |
| [contextra-store](#contextra-store) | 1 | 48330 | 5 | 1 | 5/1/0 | 142 | 5 | – |
| [contextra-sys](#contextra-sys) | 0 | 2321 | 0 | 2 | 0/0/0 | 29 | 0 | – |
| [contextra-testkit](#contextra-testkit) | tooling | 4857 | 2 | 0 | 2/0/0 | 23 | 9 | – |
| [contextra-text](#contextra-text) | 0 | 15245 | 2 | 1 | 2/0/0 | 78 | 8 | – |
| [contextra-types](#contextra-types) | 0 | 8885 | 0 | 23 | 0/0/0 | 98 | 0 | – |
| [contextra-vector](#contextra-vector) | 0 | 25229 | 4 | 1 | 4/0/0 | 100 | 9 | – |
| [contextra-wire](#contextra-wire) | 0 | 5541 | 0 | 1 | 0/0/0 | 24 | 0 | – |

## 2. Workspace-weite Auffälligkeiten

### 2.1 Zyklen

- Kein Zyklus im prod-Code-Graphen gefunden.
- Kein Zyklus in den normalen Cargo-Abhängigkeiten.

### 2.2 Abweichungen Cargo.toml ↔ Code

| Crate | Ziel | Deklariert als | Befund |
|---|---|---|---|
| `contextra-adapt` | `contextra-types` | – | genutzt, aber nicht in Cargo.toml deklariert (cfg-/Feature-Pfad oder Fehler) |
| `contextra-agent` | `contextra-checkpoint` | normal | nur in Tests/Benches genutzt, aber als normale Abhängigkeit deklariert → dev-dependency? |
| `contextra-agent` | `contextra-graph` | normal | nur in Tests/Benches genutzt, aber als normale Abhängigkeit deklariert → dev-dependency? |
| `contextra-agent` | `contextra-store` | dev | kein Code-Zugriff gefunden (prüfen: Makro, Feature, cfg, oder entfernen) |
| `contextra-agent` | `contextra-vector` | dev | kein Code-Zugriff gefunden (prüfen: Makro, Feature, cfg, oder entfernen) |
| `contextra-bench` | `contextra-db` | normal | kein Code-Zugriff gefunden (prüfen: Makro, Feature, cfg, oder entfernen) |
| `contextra-bench` | `contextra-store` | normal | kein Code-Zugriff gefunden (prüfen: Makro, Feature, cfg, oder entfernen) |
| `contextra-bench` | `contextra-text` | normal | kein Code-Zugriff gefunden (prüfen: Makro, Feature, cfg, oder entfernen) |
| `contextra-bench` | `contextra-vector` | normal | kein Code-Zugriff gefunden (prüfen: Makro, Feature, cfg, oder entfernen) |
| `contextra-bench` | `contextra-infer-candle` | – | genutzt, aber nicht in Cargo.toml deklariert (cfg-/Feature-Pfad oder Fehler) |
| `contextra-bench` | `contextra-ports` | – | genutzt, aber nicht in Cargo.toml deklariert (cfg-/Feature-Pfad oder Fehler) |
| `contextra-bench` | `contextra-rank` | – | genutzt, aber nicht in Cargo.toml deklariert (cfg-/Feature-Pfad oder Fehler) |
| `contextra-bench` | `contextra-types` | – | genutzt, aber nicht in Cargo.toml deklariert (cfg-/Feature-Pfad oder Fehler) |
| `contextra-checkpoint` | `contextra-types` | normal | nur in Tests/Benches genutzt, aber als normale Abhängigkeit deklariert → dev-dependency? |
| `contextra-checkpoint` | `contextra-mvcc` | – | genutzt, aber nicht in Cargo.toml deklariert (cfg-/Feature-Pfad oder Fehler) |
| `contextra-db` | `contextra-testkit` | dev | kein Code-Zugriff gefunden (prüfen: Makro, Feature, cfg, oder entfernen) |
| `contextra-db` | `contextra-rank` | – | genutzt, aber nicht in Cargo.toml deklariert (cfg-/Feature-Pfad oder Fehler) |
| `contextra-engine` | `contextra-sandbox` | normal? | kein Code-Zugriff gefunden (prüfen: Makro, Feature, cfg, oder entfernen) |
| `contextra-mcp` | `contextra-wire` | normal | kein Code-Zugriff gefunden (prüfen: Makro, Feature, cfg, oder entfernen) |
| `contextra-mvcc` | `contextra-types` | normal | nur in Tests/Benches genutzt, aber als normale Abhängigkeit deklariert → dev-dependency? |
| `contextra-router` | `contextra-wire` | normal | kein Code-Zugriff gefunden (prüfen: Makro, Feature, cfg, oder entfernen) |

### 2.3 Schwache Kanten (billig zu trennen)

Kanten im prod-Code mit höchstens 3 verschiedenen Symbolen. Reihenfolge: wenigste Symbole, dann wenigste Fundstellen.

| Von | Nach | Symbole | Fundstellen | Dateien | Symbole (prod) |
|---|---|---:|---:|---:|---|
| `contextra-db` | `contextra-ports` | 1 | 1 | 1 | `QueryRewriter` |
| `contextra-checkpoint` | `contextra-mvcc` | 1 | 1 | 1 | `SnapshotRegistry` |
| `contextra-bench` | `contextra-rank` | 1 | 1 | 1 | `RankError` |
| `contextra-infer-candle` | `contextra-rank` | 1 | 1 | 1 | `IsotonicCalibrator` |
| `contextra-infer-candle` | `contextra-crypto` | 1 | 1 | 1 | `EncryptedKvLayer` |
| `contextra` | `contextra-infer-candle` | 1 | 1 | 1 | `(crate)` |
| `contextra-engine` | `contextra-mvcc` | 1 | 1 | 1 | `tx_buffer::TxBuffer` |
| `contextra-cognition` | `contextra-engine` | 1 | 1 | 1 | `(crate)` |
| `contextra-cognition` | `contextra-graph` | 1 | 1 | 1 | `(crate)` |
| `contextra-vector` | `contextra-sys` | 1 | 2 | 2 | `mmap_readonly`×2 |
| `contextra-engine` | `contextra-adapt` | 1 | 2 | 1 | `PidController`×2 |
| `contextra-rank` | `contextra-ports` | 1 | 2 | 1 | `Clock`×2 |
| `contextra-mvcc` | `contextra-ports` | 1 | 3 | 1 | `MetricsSink`×3 |
| `contextra-audit-export` | `contextra-types` | 1 | 4 | 1 | `TenantId`×4 |
| `contextra-license` | `contextra-types` | 1 | 4 | 1 | `TenantId`×4 |
| `contextra-router` | `contextra-types` | 2 | 2 | 2 | `RetrievalStrategy`, `TokenBudget::default` |
| `contextra-router` | `contextra-ports` | 2 | 2 | 1 | `vfs::Vfs`, `clock` |
| `contextra-bench` | `contextra-ports` | 2 | 2 | 2 | `VectorIndex`, `EmbeddingProvider` |
| `contextra-bench` | `contextra-infer-candle` | 2 | 2 | 1 | `CandleEmbedClient::from_dir`, `model_registry::CandleQuantization::Q4KM` |
| `contextra-engine` | `contextra-infer-onnx` | 2 | 2 | 1 | `ensure_onnx_model_download`, `OnnxEmbedder::load` |
| `contextra-cognition` | `contextra-ports` | 2 | 2 | 2 | `SeededRng::from_entropy`, `(crate)` |
| `contextra-store` | `contextra-sys` | 2 | 3 | 2 | `mmap_readonly`×2, `set_restrictive_file_acl` |
| `contextra-ports` | `contextra-types` | 2 | 3 | 3 | `RequestContext`×2, `ids::ErasureId` |
| `contextra-mcp` | `contextra-crypto` | 2 | 4 | 1 | `CryptoKey`×3, `CryptoKey::try_new` |
| `contextra-engine` | `contextra-text` | 2 | 4 | 3 | `inverted::InvertedIndex`×2, `Language`×2 |
| `contextra-text` | `contextra-ports` | 2 | 8 | 5 | `StorageEngine`×6, `scanner`×2 |
| `contextra-mcp` | `contextra-infer-ollama` | 3 | 3 | 1 | `OllamaEmbedder::new`, `OllamaConfig`, `OllamaClient::with_config` |
| `contextra-mcp` | `contextra-infer-onnx` | 3 | 3 | 1 | `OnnxEmbedder::from_path`, `default_model_cache_dir`, `OnnxEmbedder::load` |
| `contextra-mcp` | `contextra-rank` | 3 | 3 | 2 | `fusion::ProvenanceRecord`, `IsotonicCalibrator`, `IsotonicCalibrator::with_defaults` |
| `contextra-mcp` | `contextra-infer-candle` | 3 | 4 | 1 | `model_registry::CandleQuantization::Q4KM`×2, `CandleEmbedClient::from_dir`, `CandleLlmClient::from_dir` |
| `contextra-engine` | `contextra-checkpoint` | 3 | 4 | 3 | `InstanceOrphanRegistry`×2, `InstanceOrphanRegistry::new`, `(crate)` |
| `contextra-agent` | `contextra-db` | 3 | 5 | 4 | `Collection`×2, `Contextra`×2, `volatile_vault` |
| `contextra-mcp` | `contextra-types` | 3 | 6 | 3 | `ContextraError`×4, `ContextraError::InvalidInput`, `ContextraError::NotFound` |
| `contextra-infer-candle` | `contextra-types` | 3 | 6 | 5 | `ContextraError`×4, `DocId::from`, `ModelFingerprint` |

**Nur in Tests/Benches/Beispielen** (für das Produkt irrelevant, aber sie koppeln den Test-Build):

| Von | Nach | Fundstellen | Symbole |
|---|---|---:|---|
| `contextra-adapt` | `contextra-types` | 1 | `TenantId` |
| `contextra-agent` | `contextra-checkpoint` | 1 | `StateCheckpoint` |
| `contextra-agent` | `contextra-graph` | 2 | `CsrGraph`×2 |
| `contextra-agent` | `contextra-testkit` | 3 | `InMemoryStorageEngine`×2, `TestMetricsSink` |
| `contextra-agent` | `contextra-text` | 5 | `Language::English`×5 |
| `contextra-bench` | `contextra-sys` | 1 | `ensure_fd_limit` |
| `contextra-checkpoint` | `contextra-testkit` | 3 | `InMemoryStorageEngine`×2, `FaultVfs` |
| `contextra-checkpoint` | `contextra-types` | 13 | `TxId`×4, `Result`×3, `ContextraError`×2, `ContextraError::Internal`×2 … (+2) |
| `contextra-cognition` | `contextra-testkit` | 2 | `ManualClock`, `MockEmbedder` |
| `contextra-crypto` | `contextra-testkit` | 5 | `mint_witness_for_tests`×4, `FaultVfs` |
| `contextra-db` | `contextra-adapt` | 1 | `PidController` |
| `contextra-db` | `contextra-checkpoint` | 4 | `PersistentCheckpointStore`×2, `clear_all_orphaned_checkpoints`, `get_orphaned_checkpoints` |
| `contextra-db` | `contextra-graph` | 15 | `CsrGraph`×10, `path_rag::PathRAGEngine`×2, `csr::CsrGraph`×2, `PathRAGEngine` |
| `contextra-db` | `contextra-rank` | 1 | `IsotonicCalibrator::new` |
| `contextra-db` | `contextra-store` | 12 | `LsmConfig`×5, `LsmStorage::new`×4, `LsmStorage`×2, `lsm` |
| `contextra-db` | `contextra-text` | 11 | `Language::English`×7, `Language`×2, `bm25::score_term`, `inverted::InvertedIndex` |
| `contextra-db` | `contextra-vector` | 6 | `HnswConfig`×3, `HnswIndex`×2, `HnswIndex::try_new` |
| `contextra-engine` | `contextra-testkit` | 1 | `InMemoryStorageEngine` |
| `contextra-mvcc` | `contextra-testkit` | 6 | `ManualClock`×6 |
| `contextra-mvcc` | `contextra-types` | 7 | `TxId`×5, `DocId`, `types` |
| `contextra-store` | `contextra-testkit` | 8 | `ManualClock`×4, `ReferenceModel`×4 |

### 2.4 Hub-Symbole (Kandidaten für types/ports)

Symbole, die aus einem Nicht-Basis-Crate von mindestens 3 verschiedenen Crates im prod-Code importiert werden. Wer sie in ein schmales Basis-Crate verschiebt, kann viele Kanten kappen.

| Quelle | Symbol | Nutzer | Crates |
|---|---|---:|---|
| `contextra-rank` | `IsotonicCalibrator` | 3 | `contextra-engine`, `contextra-infer-candle`, `contextra-mcp` |

### 2.5 API-Lecks

Öffentliche Items, deren Signatur oder öffentliche Felder Typen anderer Workspace-Crates enthalten, sowie `pub use` fremder Crates. Diese Stellen machen eine Entkopplung am schwersten, weil Nutzer des Crates die fremde Abhängigkeit mitsehen.

| Crate | Komponente | Item | Fremde Crates | Fundstelle |
|---|---|---|---|---|
| `contextra` | `boot` | `fn execute_boot_sequence` | `contextra-types` | `src/boot.rs:39` |
| `contextra` | `collection_profile` | `const DEFAULT_AUTO_EXTRACTION_MODE` | `contextra-types` | `src/collection_profile.rs:55` |
| `contextra` | `collection_profile` | `const ENTERPRISE_REGULATED_AUTO_EXTRACTION_DEFAULT` | `contextra-types` | `src/collection_profile.rs:63` |
| `contextra` | `collection_profile` | `struct CollectionProfile` | `contextra-types` | `src/collection_profile.rs:82` |
| `contextra` | `collection_profile` | `pub use contextra_types::types::domain::AutoExtractionMode` | `contextra-types` | `src/collection_profile.rs:34` |
| `contextra` | `(crate-root)` | `fn open` | `contextra-types` | `src/lib.rs:130` |
| `contextra` | `(crate-root)` | `fn open_with_config` | `contextra-types` | `src/lib.rs:162` |
| `contextra` | `(crate-root)` | `pub use contextra_types::error::ContextraError` | `contextra-types` | `src/lib.rs:53` |
| `contextra` | `(crate-root)` | `pub use contextra_types::DistanceMetric` | `contextra-types` | `src/lib.rs:55` |
| `contextra` | `(crate-root)` | `pub use contextra_infer_candle` | `contextra-infer-candle` | `src/lib.rs:59` |
| `contextra` | `performance_profile` | `enum DurabilityMode` | `contextra-db` | `src/performance_profile.rs:37` |
| `contextra` | `performance_profile` | `enum DurabilityConfigError` | `contextra-db` | `src/performance_profile.rs:52` |
| `contextra` | `performance_profile` | `struct ResolvedProfileConfig` | `contextra-db`, `contextra-ports` | `src/performance_profile.rs:117` |
| `contextra` | `performance_profile` | `enum PerformanceProfileError` | `contextra-ports` | `src/performance_profile.rs:127` |
| `contextra` | `performance_profile` | `pub use contextra_ports::license::VectorDeleteMode` | `contextra-ports` | `src/performance_profile.rs:33` |
| `contextra-agent` | `audit` | `struct AuditEntry` | `contextra-types` | `src/audit.rs:21` |
| `contextra-agent` | `audit` | `struct AuditLog` | `contextra-ports` | `src/audit.rs:51` |
| `contextra-agent` | `audit` | `fn validate_audit_payload_and_error` | `contextra-types` | `src/audit.rs:56` |
| `contextra-agent` | `audit` | `fn migrate_legacy_audit_entries` | `contextra-db`, `contextra-ports`, `contextra-types` | `src/audit.rs:202` |
| `contextra-agent` | `budget` | `fn reserve_tokens` | `contextra-types` | `src/budget.rs:17` |
| `contextra-agent` | `context` | `fn validate_task_id` | `contextra-types` | `src/context.rs:26` |
| `contextra-agent` | `context` | `fn validate_node_id` | `contextra-types` | `src/context.rs:48` |
| `contextra-agent` | `context` | `struct AgentContext` | `contextra-db`, `contextra-router` | `src/context.rs:83` |
| `contextra-audit-export` | `(crate-root)` | `struct TenantId` | `contextra-types` | `src/lib.rs:29` |
| `contextra-audit-export` | `(crate-root)` | `struct ProcessingRegisterEntry` | `contextra-types` | `src/lib.rs:231` |
| `contextra-bench` | `matrix` | `fn fuse_via_rank_rrf` | `contextra-rank` | `src/matrix.rs:102` |
| `contextra-bench` | `path_rag_sweep` | `fn run_pathrag_sweep_long_mem_eval` | `contextra-types` | `src/path_rag_sweep.rs:39` |
| `contextra-bench` | `path_rag_sweep` | `fn run_pathrag_sweep_locomo` | `contextra-types` | `src/path_rag_sweep.rs:198` |
| `contextra-cognition` | `consolidation_executor` | `fn execute_consolidation_pass` | `contextra-types` | `src/consolidation_executor.rs:125` |
| `contextra-cognition` | `consolidation_executor` | `fn execute_background_consolidation` | `contextra-types` | `src/consolidation_executor.rs:245` |
| `contextra-cognition` | `consolidation_executor` | `fn execute_background_consolidation_with_rich_validator` | `contextra-types` | `src/consolidation_executor.rs:369` |
| `contextra-cognition` | `consolidation_executor` | `fn execute_leanrag_aggregation_stage` | `contextra-types` | `src/consolidation_executor.rs:434` |
| `contextra-cognition` | `consolidation_executor` | `fn execute_sleep_cycle` | `contextra-types` | `src/consolidation_executor.rs:449` |
| `contextra-cognition` | `engine_local` | `pub use contextra_engine` | `contextra-engine` | `src/engine_local.rs:3` |
| `contextra-cognition` | `graph_local` | `pub use contextra_graph` | `contextra-graph` | `src/graph_local.rs:3` |
| `contextra-cognition` | `memory_consolidation` | `fn run_structural_synthesis_pass` | `contextra-types` | `src/memory_consolidation.rs:591` |
| `contextra-cognition` | `memory_consolidation` | `struct PreCondensedInput` | `contextra-types` | `src/memory_consolidation.rs:655` |
| `contextra-cognition` | `memory_consolidation` | `fn intake_pre_condensed_inputs` | `contextra-types` | `src/memory_consolidation.rs:790` |
| `contextra-cognition` | `ports_local` | `pub use contextra_ports` | `contextra-ports` | `src/ports_local.rs:3` |
| `contextra-cognition` | `types_local` | `pub use contextra_types` | `contextra-types` | `src/types_local.rs:3` |
| `contextra-crypto` | `deletion_proof::chain` | `fn verify_proof_sequence` | `contextra-types` | `src/deletion_proof/chain.rs:12` |
| `contextra-crypto` | `deletion_proof::types` | `fn serialize` | `contextra-ports` | `src/deletion_proof/types.rs:235` |
| `contextra-crypto` | `deletion_proof::types` | `fn deserialize` | `contextra-ports` | `src/deletion_proof/types.rs:250` |
| `contextra-crypto` | `deletion_proof::types` | `struct LayerWitnessMeasurement` | `contextra-ports` | `src/deletion_proof/types.rs:267` |
| `contextra-crypto` | `deletion_proof::verifier` | `enum VerificationErrorReason` | `contextra-types` | `src/deletion_proof/verifier.rs:13` |
| `contextra-crypto` | `error` | `enum CryptoError` | `contextra-types` | `src/error.rs:19` |
| `contextra-crypto` | `kv_cipher` | `struct EncryptedKvLayer` | `contextra-types` | `src/kv_cipher.rs:52` |
| `contextra-crypto` | `kv_cipher` | `pub use contextra_types::ModelFingerprint` | `contextra-types` | `src/kv_cipher.rs:42` |
| `contextra-crypto` | `kv_segment::segment` | `struct KvSegment` | `contextra-types` | `src/kv_segment/segment.rs:42` |
| `contextra-crypto` | `kv_segment::store` | `type SpillHandler` | `contextra-types` | `src/kv_segment/store.rs:26` |
| `contextra-db` | `(crate-root)` | `pub use contextra_engine` | `contextra-engine` | `src/lib.rs:72` |
| `contextra-db` | `(crate-root)` | `pub use contextra_engine::DurabilityMode` | `contextra-engine` | `src/lib.rs:73` |
| `contextra-db` | `(crate-root)` | `pub use contextra_engine::SandboxBridge` | `contextra-engine` | `src/lib.rs:81` |
| `contextra-db` | `(crate-root)` | `pub use contextra_engine::decay_controller::*` | `contextra-engine` | `src/lib.rs:92` |
| `contextra-db` | `(crate-root)` | `pub use contextra_engine::fusion` | `contextra-engine` | `src/lib.rs:94` |
| `contextra-db` | `(crate-root)` | `pub use contextra_types::DistanceMetric` | `contextra-types` | `src/lib.rs:123` |
| `contextra-db` | `multistep` | `pub use contextra_ports::QueryRewriter` | `contextra-ports` | `src/multistep.rs:28` |
| `contextra-engine` | `adapters::privacy` | `fn to_canonical_tenant_id` | `contextra-types` | `src/adapters/privacy.rs:6` |
| `contextra-engine` | `background_workers::expiry_workers` | `fn start_decay_cleanup_worker` | `contextra-ports` | `src/background_workers/expiry_workers.rs:85` |
| `contextra-engine` | `collection::crud::auto_extraction` | `pub use contextra_types::AutoExtractionMode` | `contextra-types` | `src/collection/crud/auto_extraction.rs:18` |
| `contextra-engine` | `collection::maintenance` | `pub use contextra_graph::percolation::PercolationConfig` | `contextra-graph` | `src/collection/maintenance.rs:878` |
| `contextra-engine` | `collection` | `fn compute_default_importance` | `contextra-types` | `src/collection/mod.rs:124` |
| `contextra-engine` | `collection` | `struct Collection` | `contextra-store`, `contextra-vector` | `src/collection/mod.rs:272` |
| `contextra-engine` | `collection::query_builder::scope` | `struct ScopeConstraint` | `contextra-types` | `src/collection/query_builder/scope.rs:19` |
| `contextra-engine` | `collection::query_builder::strategy` | `enum SearchStrategy` | `contextra-types` | `src/collection/query_builder/strategy.rs:11` |
| `contextra-engine` | `extraction::open_ie` | `fn extract_triples` | `contextra-ports` | `src/extraction/open_ie.rs:80` |
| `contextra-engine` | `fusion` | `fn fuse_search_results_with_strategy` | `contextra-types` | `src/fusion.rs:16` |
| `contextra-engine` | `fusion` | `fn fuse_search_results_with_signal_strategies` | `contextra-types` | `src/fusion.rs:35` |
| `contextra-engine` | `(crate-root)` | `type ConsolidationLauncher` | `contextra-store` | `src/lib.rs:102` |
| `contextra-engine` | `(crate-root)` | `fn register_consolidation_launcher` | `contextra-store` | `src/lib.rs:119` |
| `contextra-engine` | `(crate-root)` | `struct ContextraStats` | `contextra-ports` | `src/lib.rs:152` |
| `contextra-engine` | `(crate-root)` | `struct ContextraConfig` | `contextra-ports`, `contextra-store`, `contextra-types` | `src/lib.rs:218` |
| `contextra-engine` | `(crate-root)` | `type TenantCollectionMap` | `contextra-store`, `contextra-types` | `src/lib.rs:357` |
| `contextra-engine` | `(crate-root)` | `pub use contextra_ports::TextEmbeddingEngine` | `contextra-ports` | `src/lib.rs:21` |
| `contextra-engine` | `(crate-root)` | `pub use contextra_ports::VectorDeleteMode` | `contextra-ports` | `src/lib.rs:22` |
| `contextra-engine` | `(crate-root)` | `pub use contextra_store::lsm::DurabilityMode` | `contextra-store` | `src/lib.rs:23` |
| `contextra-engine` | `(crate-root)` | `pub use contextra_checkpoint` | `contextra-checkpoint` | `src/lib.rs:96` |
| `contextra-engine` | `(crate-root)` | `pub use contextra_text::Language` | `contextra-text` | `src/lib.rs:97` |
| `contextra-engine` | `(crate-root)` | `pub use contextra_ports::DriftStatusProvider` | `contextra-ports` | `src/lib.rs:168` |
| `contextra-engine` | `(crate-root)` | `pub use contextra_types::DistanceMetric` | `contextra-types` | `src/lib.rs:392` |
| `contextra-engine` | `metrics_names` | `fn record_histogram_safe` | `contextra-ports` | `src/metrics_names.rs:33` |
| `contextra-engine` | `metrics_names` | `fn record_gauge_safe` | `contextra-ports` | `src/metrics_names.rs:45` |
| `contextra-engine` | `metrics_names` | `fn record_counter_safe` | `contextra-ports` | `src/metrics_names.rs:57` |
| `contextra-engine` | `temporal_filter` | `type FusionResult` | `contextra-rank` | `src/temporal_filter.rs:22` |
| `contextra-engine` | `temporal_filter` | `struct ValidityWindow` | `contextra-types` | `src/temporal_filter.rs:25` |
| `contextra-engine` | `temporal_filter` | `fn sort_candidates_deterministic` | `contextra-rank` | `src/temporal_filter.rs:162` |
| `contextra-engine` | `temporal_filter` | `fn apply_temporal_validity_filter_with_clock` | `contextra-ports`, `contextra-rank`, `contextra-types` | `src/temporal_filter.rs:173` |
| `contextra-engine` | `temporal_filter` | `fn apply_temporal_validity_filter` | `contextra-rank`, `contextra-types` | `src/temporal_filter.rs:187` |
| `contextra-engine` | `temporal_filter` | `fn apply_temporal_validity_filter_at` | `contextra-rank` | `src/temporal_filter.rs:207` |
| `contextra-engine` | `transaction::db_transaction` | `struct DbTransaction` | `contextra-vector` | `src/transaction/db_transaction.rs:18` |
| `contextra-graph` | `apprh::diffusion` | `fn run_apprh_push` | `contextra-types` | `src/apprh/diffusion.rs:23` |
| `contextra-graph` | `apprh` | `fn apprh_local` | `contextra-types` | `src/apprh/mod.rs:35` |
| `contextra-graph` | `apprh` | `fn forward_push_apprh` | `contextra-types` | `src/apprh/mod.rs:51` |
| `contextra-graph` | `apprh::shadow` | `struct ApprhShadowComparison` | `contextra-types` | `src/apprh/shadow.rs:30` |
| `contextra-graph` | `apprh::shadow` | `fn shadow_compare_forward_push_vs_apprh` | `contextra-types` | `src/apprh/shadow.rs:54` |
| `contextra-graph` | `edge_reinforcement_buffer` | `struct CooccurrenceSignal` | `contextra-types` | `src/edge_reinforcement_buffer.rs:26` |
| `contextra-graph` | `edge_reinforcement_buffer` | `struct TraversalSignal` | `contextra-types` | `src/edge_reinforcement_buffer.rs:36` |
| `contextra-graph` | `hyperedge_suggest` | `fn validate_candidates_with_llm` | `contextra-ports` | `src/hyperedge_suggest.rs:118` |
| `contextra-graph` | `(crate-root)` | `trait GraphIndexExt` | `contextra-ports` | `src/lib.rs:132` |
| `contextra-graph` | `path_rag::k_path` | `struct KPathResult` | `contextra-types` | `src/path_rag/k_path.rs:35` |
| `contextra-graph` | `path_rag` | `struct GraphPath` | `contextra-types` | `src/path_rag/mod.rs:31` |
| `contextra-graph` | `path_rag` | `fn forward_push_ppr` | `contextra-types` | `src/path_rag/mod.rs:96` |
| `contextra-graph` | `path_rag` | `pub use contextra_types::EntityId` | `contextra-types` | `src/path_rag/mod.rs:25` |
| `contextra-graph` | `percolation` | `fn find_rebonding_candidates` | `contextra-ports`, `contextra-types` | `src/percolation.rs:79` |
| `contextra-graph` | `ppr::shadow_hook` | `fn log_tl_hfd_vs_baseline_discrepancy` | `contextra-types` | `src/ppr/shadow_hook.rs:26` |
| `contextra-graph` | `ppr_stream` | `struct PprCandidateStream` | `contextra-ports` | `src/ppr_stream.rs:23` |
| `contextra-graph` | `tl_hfd::diffusion` | `fn run_diffusion` | `contextra-types` | `src/tl_hfd/diffusion.rs:28` |
| `contextra-graph` | `tl_hfd::lovasz` | `fn truncate_participants` | `contextra-types` | `src/tl_hfd/lovasz.rs:14` |
| `contextra-graph` | `tl_hfd::lovasz` | `fn compute_lovasz_extension` | `contextra-types` | `src/tl_hfd/lovasz.rs:56` |
| `contextra-graph` | `tl_hfd` | `fn tl_hfd_local` | `contextra-types` | `src/tl_hfd/mod.rs:31` |
| `contextra-graph` | `tl_hfd::shadow` | `struct ShadowComparison` | `contextra-types` | `src/tl_hfd/shadow.rs:55` |
| `contextra-graph` | `tl_hfd::shadow` | `fn shadow_compare_forward_push_vs_tl_hfd` | `contextra-types` | `src/tl_hfd/shadow.rs:79` |
| `contextra-infer-candle` | `model_registry` | `fn compute_fingerprint` | `contextra-types` | `src/model_registry.rs:56` |
| `contextra-infer-candle` | `model_registry` | `pub use contextra_types::ModelFingerprint` | `contextra-types` | `src/model_registry.rs:50` |
| `contextra-infer-ollama` | `error` | `struct SystemClock` | `contextra-ports` | `src/error.rs:124` |
| `contextra-kvcache` | `adapters` | `struct KvCachePluginConfig` | `contextra-ports` | `src/adapters.rs:128` |
| `contextra-kvcache` | `attention_score` | `fn rank_for_eviction_weighted_for_tenant` | `contextra-types` | `src/attention_score.rs:115` |
| `contextra-kvcache` | `scratchpad_invalidation` | `struct ScratchpadCacheScope` | `contextra-types` | `src/scratchpad_invalidation.rs:29` |
| `contextra-kvcache` | `scratchpad_invalidation` | `struct PrefixStoreScratchpadInvalidator` | `contextra-ports` | `src/scratchpad_invalidation.rs:59` |
| `contextra-kvcache` | `scratchpad_invalidation` | `struct BoundedScratchpadInvalidator` | `contextra-ports` | `src/scratchpad_invalidation.rs:267` |
| `contextra-license` | `types` | `struct TenantId` | `contextra-types` | `src/types.rs:60` |
| `contextra-mcp` | `bulk_exfiltration_detector` | `pub use contextra_privacy::bulk_exfiltration_detector::*` | `contextra-privacy` | `src/bulk_exfiltration_detector.rs:4` |
| `contextra-mcp` | `config` | `fn verify_embedding_provider` | `contextra-types` | `src/config.rs:78` |
| `contextra-mcp` | `config` | `fn create_embedding_provider` | `contextra-types` | `src/config.rs:272` |
| `contextra-mcp` | `config` | `fn create_llm_text_generator` | `contextra-types` | `src/config.rs:422` |
| `contextra-mcp` | `config` | `struct RouterConfig` | `contextra-router` | `src/config.rs:484` |
| `contextra-mcp` | `egress_gateway` | `pub use contextra_privacy::egress_gateway::*` | `contextra-privacy` | `src/egress_gateway.rs:4` |
| `contextra-mcp` | `egress_guard` | `struct CollectionSearchEngine` | `contextra` | `src/egress_guard.rs:11` |
| `contextra-mcp` | `egress_guard` | `pub use contextra_privacy::egress_guard::*` | `contextra-privacy` | `src/egress_guard.rs:4` |
| `contextra-mcp` | `explain` | `struct ExplainResponse` | `contextra-rank` | `src/explain.rs:23` |
| `contextra-mcp` | `routing` | `struct RoutingHandle` | `contextra-ports`, `contextra-rank`, `contextra-router` | `src/routing.rs:76` |
| `contextra-mcp` | `routing` | `fn setup_routing` | `contextra` | `src/routing.rs:82` |
| `contextra-mcp` | `server` | `struct McpServer` | `contextra`, `contextra-ports` | `src/server.rs:15` |
| `contextra-ports` | `observer` | `struct ErasureEvent` | `contextra-types` | `src/observer.rs:48` |
| `contextra-py` | `bindings::common` | `fn doc_to_py` | `contextra` | `src/bindings/common.rs:386` |
| `contextra-py` | `bindings::common` | `fn results_to_py` | `contextra` | `src/bindings/common.rs:403` |
| `contextra-py` | `bindings::common` | `fn contextra_err` | `contextra-types` | `src/bindings/common.rs:436` |
| `contextra-rank` | `calibration::isotonic` | `trait Clock` | `contextra-ports` | `src/calibration/isotonic.rs:14` |
| `contextra-rank` | `fusion::normalized` | `fn weights_to_signal_factors_types_compat` | `contextra-types` | `src/fusion/normalized.rs:197` |
| `contextra-rank` | `local_types` | `struct DocId` | `contextra-types` | `src/local_types.rs:9` |
| `contextra-rank` | `local_types` | `struct ConfigFingerprint` | `contextra-types` | `src/local_types.rs:67` |
| `contextra-rank` | `local_types` | `enum FusionStrategy` | `contextra-types` | `src/local_types.rs:120` |
| `contextra-rank` | `local_types` | `struct SignalFusionStrategies` | `contextra-types` | `src/local_types.rs:182` |
| `contextra-rank` | `local_types` | `struct FusionWeights` | `contextra-types` | `src/local_types.rs:237` |
| `contextra-router` | `(crate-root)` | `pub use contextra_adapt::bandit` | `contextra-adapt` | `src/lib.rs:31` |
| `contextra-router` | `(crate-root)` | `pub use contextra_adapt::lyapunov` | `contextra-adapt` | `src/lib.rs:36` |
| `contextra-router` | `(crate-root)` | `pub use contextra_adapt::offpolicy` | `contextra-adapt` | `src/lib.rs:41` |
| `contextra-simd` | `metric` | `enum DistanceMetric` | `contextra-types` | `src/metric.rs:3` |
| `contextra-store` | `lsm::engine` | `struct LsmStorage` | `contextra-mvcc` | `src/lsm/engine.rs:33` |
| `contextra-store` | `lsm::observer` | `struct CommitContext` | `contextra-types` | `src/lsm/observer.rs:66` |

… 29 weitere (siehe `--json`).

## 3. Kantenliste

| Von | Nach | prod Fundstellen | prod Symbole | dev Fundstellen | Cargo-Art |
|---|---|---:|---:|---:|---|
| `contextra-engine` | `contextra-types` | 236 | 50 | 179 | normal |
| `contextra-vector` | `contextra-types` | 97 | 21 | 40 | normal? |
| `contextra-checkpoint` | `contextra-ports` | 55 | 8 | 8 | normal |
| `contextra-store` | `contextra-ports` | 53 | 17 | 149 | normal |
| `contextra-graph` | `contextra-types` | 46 | 13 | 60 | normal? |
| `contextra-rank` | `contextra-types` | 38 | 16 | 0 | normal? |
| `contextra-store` | `contextra-types` | 37 | 13 | 123 | normal |
| `contextra-crypto` | `contextra-types` | 35 | 14 | 18 | normal |
| `contextra-engine` | `contextra-ports` | 35 | 17 | 19 | normal |
| `contextra-infer-ollama` | `contextra-types` | 34 | 13 | 1 | normal? |
| `contextra-vector` | `contextra-ports` | 34 | 7 | 78 | normal? |
| `contextra-graph` | `contextra-ports` | 29 | 20 | 44 | normal? |
| `contextra-agent` | `contextra-types` | 26 | 9 | 61 | normal |
| `contextra-license` | `contextra-ports` | 26 | 11 | 0 | normal? |
| `contextra-engine` | `contextra-store` | 22 | 10 | 104 | normal |
| `contextra-text` | `contextra-types` | 22 | 9 | 137 | normal? |
| `contextra-kvcache` | `contextra-ports` | 21 | 18 | 16 | normal? |
| `contextra-engine` | `contextra-graph` | 20 | 16 | 75 | normal |
| `contextra-testkit` | `contextra-types` | 20 | 4 | 10 | normal |
| `contextra-cognition` | `contextra-types` | 19 | 8 | 29 | normal |
| `contextra-kvcache` | `contextra-types` | 18 | 11 | 17 | normal? |
| `contextra-infer-candle` | `contextra-ports` | 16 | 6 | 15 | normal |
| `contextra-infer-ollama` | `contextra-ports` | 16 | 14 | 2 | normal? |
| `contextra-agent` | `contextra-ports` | 15 | 5 | 20 | normal |
| `contextra-mcp` | `contextra` | 15 | 12 | 9 | normal |
| `contextra-engine` | `contextra-rank` | 14 | 6 | 2 | normal |
| `contextra-testkit` | `contextra-ports` | 14 | 12 | 4 | normal |
| `contextra-simd` | `contextra-types` | 13 | 6 | 0 | normal? |
| `contextra` | `contextra-types` | 12 | 8 | 20 | normal |
| `contextra-vector` | `contextra-mvcc` | 11 | 8 | 0 | normal? |
| `contextra` | `contextra-db` | 10 | 5 | 1 | normal |
| `contextra-mcp` | `contextra-router` | 10 | 6 | 1 | normal |
| `contextra-store` | `contextra-mvcc` | 10 | 6 | 7 | normal |
| `contextra` | `contextra-ports` | 9 | 7 | 9 | normal |
| `contextra-engine` | `contextra-vector` | 9 | 4 | 117 | dev/normal |
| `contextra-bench` | `contextra-types` | 8 | 4 | 1 | – |
| `contextra-kvcache` | `contextra-crypto` | 8 | 5 | 3 | normal? |
| `contextra-mcp` | `contextra-ports` | 8 | 4 | 38 | normal |
| `contextra-py` | `contextra-wire` | 8 | 4 | 0 | normal |
| `contextra-text` | `contextra-ports` | 8 | 2 | 87 | normal? |
| `contextra-mcp` | `contextra-privacy` | 7 | 7 | 1 | normal |
| `contextra-py` | `contextra` | 7 | 7 | 3 | normal |
| `contextra-crypto` | `contextra-ports` | 6 | 6 | 34 | normal |
| `contextra-db` | `contextra-engine` | 6 | 6 | 0 | normal |
| `contextra-infer-candle` | `contextra-types` | 6 | 3 | 9 | normal |
| `contextra-mcp` | `contextra-types` | 6 | 3 | 5 | normal |
| `contextra-agent` | `contextra-db` | 5 | 3 | 42 | normal |
| `contextra-py` | `contextra-types` | 5 | 4 | 0 | normal |
| `contextra-store` | `contextra-crypto` | 5 | 4 | 2 | normal |
| `contextra-agent` | `contextra-router` | 4 | 4 | 0 | normal |
| `contextra-audit-export` | `contextra-types` | 4 | 1 | 0 | normal? |
| `contextra-db` | `contextra-types` | 4 | 4 | 46 | normal |
| `contextra-engine` | `contextra-checkpoint` | 4 | 3 | 0 | normal |
| `contextra-engine` | `contextra-text` | 4 | 2 | 68 | normal |
| `contextra-license` | `contextra-types` | 4 | 1 | 0 | normal? |
| `contextra-mcp` | `contextra-crypto` | 4 | 2 | 3 | normal |
| `contextra-mcp` | `contextra-infer-candle` | 4 | 3 | 0 | normal? |
| `contextra-router` | `contextra-adapt` | 4 | 4 | 4 | normal |
| `contextra-mcp` | `contextra-infer-ollama` | 3 | 3 | 0 | normal? |
| `contextra-mcp` | `contextra-infer-onnx` | 3 | 3 | 0 | normal? |
| `contextra-mcp` | `contextra-rank` | 3 | 3 | 0 | normal |
| `contextra-mvcc` | `contextra-ports` | 3 | 1 | 1 | normal |
| `contextra-ports` | `contextra-types` | 3 | 2 | 0 | normal |
| `contextra-store` | `contextra-sys` | 3 | 2 | 4 | normal |
| `contextra-bench` | `contextra-infer-candle` | 2 | 2 | 0 | – |
| `contextra-bench` | `contextra-ports` | 2 | 2 | 1 | – |
| `contextra-cognition` | `contextra-ports` | 2 | 2 | 6 | normal |
| `contextra-engine` | `contextra-adapt` | 2 | 1 | 1 | normal? |
| `contextra-engine` | `contextra-infer-onnx` | 2 | 2 | 0 | normal? |
| `contextra-rank` | `contextra-ports` | 2 | 1 | 0 | normal? |
| `contextra-router` | `contextra-ports` | 2 | 2 | 7 | normal |
| `contextra-router` | `contextra-types` | 2 | 2 | 21 | normal |
| `contextra-vector` | `contextra-sys` | 2 | 1 | 0 | normal? |
| `contextra` | `contextra-infer-candle` | 1 | 1 | 0 | normal? |
| `contextra-bench` | `contextra-rank` | 1 | 1 | 0 | – |
| `contextra-checkpoint` | `contextra-mvcc` | 1 | 1 | 0 | – |
| `contextra-cognition` | `contextra-engine` | 1 | 1 | 12 | normal |
| `contextra-cognition` | `contextra-graph` | 1 | 1 | 15 | normal |
| `contextra-db` | `contextra-ports` | 1 | 1 | 12 | normal |
| `contextra-engine` | `contextra-mvcc` | 1 | 1 | 5 | normal |
| `contextra-infer-candle` | `contextra-crypto` | 1 | 1 | 1 | dev/normal? |
| `contextra-infer-candle` | `contextra-rank` | 1 | 1 | 0 | normal |
| `contextra-adapt` | `contextra-types` | 0 | 0 | 1 | – |
| `contextra-agent` | `contextra-checkpoint` | 0 | 0 | 1 | normal |
| `contextra-agent` | `contextra-graph` | 0 | 0 | 2 | normal |
| `contextra-agent` | `contextra-testkit` | 0 | 0 | 3 | dev |
| `contextra-agent` | `contextra-text` | 0 | 0 | 5 | dev |
| `contextra-bench` | `contextra-sys` | 0 | 0 | 1 | dev |
| `contextra-checkpoint` | `contextra-testkit` | 0 | 0 | 3 | dev |
| `contextra-checkpoint` | `contextra-types` | 0 | 0 | 13 | normal |
| `contextra-cognition` | `contextra-testkit` | 0 | 0 | 2 | dev |
| `contextra-crypto` | `contextra-testkit` | 0 | 0 | 5 | dev |
| `contextra-db` | `contextra-adapt` | 0 | 0 | 1 | dev |
| `contextra-db` | `contextra-checkpoint` | 0 | 0 | 4 | dev |
| `contextra-db` | `contextra-graph` | 0 | 0 | 15 | dev |
| `contextra-db` | `contextra-rank` | 0 | 0 | 1 | – |
| `contextra-db` | `contextra-store` | 0 | 0 | 12 | dev |
| `contextra-db` | `contextra-text` | 0 | 0 | 11 | dev |
| `contextra-db` | `contextra-vector` | 0 | 0 | 6 | dev |
| `contextra-engine` | `contextra-testkit` | 0 | 0 | 1 | dev |
| `contextra-mvcc` | `contextra-testkit` | 0 | 0 | 6 | dev |
| `contextra-mvcc` | `contextra-types` | 0 | 0 | 7 | normal |
| `contextra-store` | `contextra-testkit` | 0 | 0 | 8 | dev |

## 4. Crates im Detail

### contextra

- Pfad: `crates/contextra` · Zeilen: 3372 · Rust-Dateien: 22 · Ring: 4
- Interne Cargo-Abhängigkeiten: `contextra-db` (normal), `contextra-infer-candle` (normal/optional via candle), `contextra-ports` (normal), `contextra-types` (normal)

**Ausgehend (Importe aus anderen Workspace-Crates)**

| Ziel | prod | dev | Dateien (prod) | Symbole (prod) | Symbole (nur dev) |
|---|---:|---:|---:|---|---|
| `contextra-types` | 12 | 20 | 7 | `error::ContextraError`×4, `DistanceMetric`×2, `types::domain::AutoExtractionMode`, `types::domain`, `ContextraError`, `ContextraError::DurabilityConfig`, `error::Result`, `ScoredEntry` | `DocId`×2, `DocId::from_u64`×2, `ContextraError::InvalidInput`, `ScoredDocument`, `DistanceMetric::Cosine` |
| `contextra-db` | 10 | 1 | 2 | `DurabilityMode`×3, `DurabilityMode::Full`×2, `DurabilityMode::WalNoHmac`×2, `DurabilityMode::MemoryOnly`×2, `QueryRewriter` | – |
| `contextra-ports` | 9 | 9 | 4 | `license::FeatureRing`×2, `license::LicenseGate`×2, `IdGen`, `license`, `license::GateToken::for_test`, `license::VectorDeleteMode`, `license::LicenseError` | `BoxFuture`×3, `license::GateToken`, `license::LicenseError::InsufficientRing`, `SequentialIdGen` |
| `contextra-infer-candle` | 1 | 0 | 1 | `(crate)` | – |

**Pro Komponente**

- `(crate-root)` (190 Zeilen)
  - → `contextra-infer-candle`: prod: `(crate)`
  - → `contextra-types`: prod: `error::ContextraError`, `DistanceMetric`, `types::domain`
- `agent_memory` (374 Zeilen)
  - → `contextra-ports`: prod: `IdGen`
  - → `contextra-types`: prod: `error::ContextraError`
- `boot` (90 Zeilen)
  - → `contextra-types`: prod: `error::ContextraError`
- `builder` (456 Zeilen)
  - → `contextra-ports`: prod: `license::FeatureRing`, `license`, `license::GateToken::for_test`
  - → `contextra-types`: prod: `error::ContextraError`, `DistanceMetric`
- `collection_profile` (186 Zeilen)
  - → `contextra-ports`: prod: `license::LicenseGate`
  - → `contextra-types`: prod: `types::domain::AutoExtractionMode`
- `performance_profile` (294 Zeilen)
  - → `contextra-db`: prod: `DurabilityMode`×3, `DurabilityMode::Full`×2, `DurabilityMode::WalNoHmac`×2, `DurabilityMode::MemoryOnly`×2
  - → `contextra-ports`: prod: `license::FeatureRing`, `license::LicenseGate`, `license::VectorDeleteMode`, `license::LicenseError` · dev: `license::LicenseError`×2, `license::GateToken`, `license::GateToken::for_test`, `license::LicenseError::InsufficientRing`
  - → `contextra-types`: prod: `ContextraError`, `ContextraError::DurabilityConfig`
- `query_rewriter` (250 Zeilen)
  - → `contextra-db`: prod: `QueryRewriter`
  - → `contextra-types`: prod: `error::Result`, `ScoredEntry`
- `examples/embedded_facade.rs` (47 Zeilen)
  - → `contextra-types`: dev: `DistanceMetric`
- `tests/agent_memory_facade.rs` (84 Zeilen)
  - → `contextra-ports`: dev: `BoxFuture`
  - → `contextra-types`: dev: `error::Result`
- `tests/agent_memory_hyperedges.rs` (149 Zeilen)
  - → `contextra-ports`: dev: `BoxFuture`
  - → `contextra-types`: dev: `error::Result`
- `tests/agent_memory_idgen_test.rs` (83 Zeilen)
  - → `contextra-ports`: dev: `BoxFuture`, `SequentialIdGen`
  - → `contextra-types`: dev: `error::Result`
- `tests/llm_query_rewriter.rs` (206 Zeilen)
  - → `contextra-db`: dev: `QueryRewriter`
  - → `contextra-types`: dev: `error::ContextraError`, `ScoredEntry`, `error::Result`
- `tests/type_identity_test.rs` (53 Zeilen)
  - → `contextra-types`: dev: `ContextraError`×2, `DocId`×2, `DocId::from_u64`×2, `DistanceMetric`×2 … (+4)

**Öffentliche Schnittstelle** (erreichbar über `pub mod`-Kette oder `pub use`)

- `(crate-root)`
  - 3 fn
    - `fn builder`
    - `fn open` ⚠ fremd: `contextra-types`
    - `fn open_with_config` ⚠ fremd: `contextra-types`
- `agent_memory`
  - 4 struct
    - `struct MemoryId` · 2 Methoden
    - `struct RelationId`
    - `struct Memory`
    - `struct AgentMemory` · 10 Methoden
- `boot`
  - 1 enum, 1 fn
    - `enum BootPhase`
    - `fn execute_boot_sequence` ⚠ fremd: `contextra-types`
- `builder`
  - 1 enum, 1 struct, 1 trait
    - `enum EncryptionMode`
    - `trait ContextraConfigExt` · Methoden: is_encrypted
    - `struct ContextraBuilder` · 14 Methoden
- `collection_profile`
  - 2 const, 3 enum, 2 struct
    - `enum KvDeleteMode`
    - `const DEFAULT_AUTO_EXTRACTION_MODE` ⚠ fremd: `contextra-types`
    - `const ENTERPRISE_REGULATED_AUTO_EXTRACTION_DEFAULT` ⚠ fremd: `contextra-types`
    - `struct LsmTuning`
    - `struct CollectionProfile` · 2 Methoden ⚠ fremd: `contextra-types`
    - `enum CollectionProfileError`
    - `enum DeploymentTier` · 1 Methoden
- `performance_profile`
  - 4 enum, 1 struct
    - `enum DurabilityMode` · 1 Methoden ⚠ fremd: `contextra-db`
    - `enum DurabilityConfigError` ⚠ fremd: `contextra-db`
    - `enum PerformanceProfile` · 1 Methoden
    - `struct ResolvedProfileConfig` · 2 Methoden ⚠ fremd: `contextra-db`, `contextra-ports`
    - `enum PerformanceProfileError` ⚠ fremd: `contextra-ports`
- `query_rewriter`
  - 1 struct
    - `struct LlmQueryRewriter` · 5 Methoden
- Zusätzlich 3 `pub`-Items in privaten Modulen ohne Re-Export (nicht Teil der echten API).
- ⚠ Re-Exporte **fremder Crates**: `contextra_types::types::domain::AutoExtractionMode`, `contextra_types::error::ContextraError`, `contextra_types::DistanceMetric`, `contextra_infer_candle`, `contextra_ports::license::VectorDeleteMode`

**Eingehend (wer nutzt dieses Crate)**

| Nutzer | Symbole prod | Symbole dev |
|---|---|---|
| `contextra-mcp` | `Contextra`×3, `Collection`×2, `chunker`, `chunker::MarkdownChunker::new`, `chunker::ChunkerConfig::default`, `collection_profile`, `collection_profile::DeploymentTier::EdgeMinimal`, `collection_profile::DeploymentTier::PowerUserLocal`, `collection_profile::DeploymentTier::EnterpriseShared`, `collection_profile::DeploymentTier::EnterpriseRegulated` … (+2) | `collection_profile::DeploymentTier` |
| `contextra-py` | `Collection`, `Document`, `SearchResult`, `Contextra`, `DistanceMetric::Cosine`, `DistanceMetric::Euclidean`, `DistanceMetric::DotProduct` | – |

**Nach außen ungenutzte öffentliche Items (Heuristik, 24 von 25)**: `MemoryId`, `RelationId`, `Memory`, `AgentMemory`, `BootPhase`, `execute_boot_sequence`, `EncryptionMode`, `ContextraConfigExt`, `ContextraBuilder`, `KvDeleteMode`, `DEFAULT_AUTO_EXTRACTION_MODE`, `ENTERPRISE_REGULATED_AUTO_EXTRACTION_DEFAULT`, `LsmTuning`, `CollectionProfile`, `CollectionProfileError`, `builder`, `open`, `open_with_config`, `DurabilityMode`, `DurabilityConfigError`, `PerformanceProfile`, `ResolvedProfileConfig`, `PerformanceProfileError`, `LlmQueryRewriter`

### contextra-adapt

- Pfad: `crates/contextra-adapt` · Zeilen: 6872 · Rust-Dateien: 34 · Ring: 0
- Interne Cargo-Abhängigkeiten: keine

**Ausgehend (Importe aus anderen Workspace-Crates)**

| Ziel | prod | dev | Dateien (prod) | Symbole (prod) | Symbole (nur dev) |
|---|---:|---:|---:|---|---|
| `contextra-types` | 0 | 1 | 0 | – | `TenantId` |

**Pro Komponente**

- `tests/sketched_projection_determinism.rs` (103 Zeilen)
  - → `contextra-types`: dev: `TenantId`

**Öffentliche Schnittstelle** (erreichbar über `pub mod`-Kette oder `pub use`)

- `bandit`
  - 1 const, 2 enum, 3 struct, 1 trait
    - `const SHERMAN_MORRISON_REFACTORIZATION_INTERVAL`
    - `enum BanditError`
    - `struct AlignedF32Vec` · 2 Methoden
    - `enum BanditImplementation`
    - `struct SketchMatrix` · 4 Methoden
    - `trait BanditPolicy` · Methoden: apply_drift_penalty
    - `struct BanditProfileState` · 6 Methoden
- `clock`
  - 1 struct, 1 trait
    - `trait Clock` · Methoden: now_unix_nanos, monotonic_nanos
    - `struct SystemClock` · 1 Methoden
- `contract`
  - 1 fn, 5 trait
    - `trait Clock` · Methoden: now_unix_nanos, monotonic_nanos
    - `trait Rng` · Methoden: next_u64, next_f64
    - `trait AdaptiveController` · Methoden: update, reset
    - `trait BanditArmSelector` · Methoden: select_arm, update
    - `trait StateSnapshot` · Methoden: to_snapshot_bytes, from_snapshot_bytes
    - `fn _assert_contract_conformance`
- `decay_controller`
  - 3 struct
    - `struct DecaySignalInputs`
    - `struct DecayControllerConfig`
    - `struct AdaptiveDecayController` · 7 Methoden
- `drift`
  - 1 enum, 3 struct, 1 trait
    - `enum DriftSignal`
    - `trait DriftDetector` · Methoden: observe, is_drifting
    - `struct CatoniDriftDetector` · 2 Methoden
    - `struct EnsembleDriftWatcher` · 3 Methoden
    - `struct DriftPolicyBridge` · 2 Methoden
- `error`
  - 1 enum, 1 type
    - `enum AdaptError`
    - `type Result`
- `flow_thompson`
  - 2 const, 2 enum, 1 fn, 9 struct, 3 trait
    - `enum FcTsError`
    - `trait FcTsRng` · Methoden: next_u64, next_f32, next_standard_normal
    - `struct SplitMix64` · 1 Methoden
    - `struct FcTsConfig` · 1 Methoden
    - `struct Ring3Token`
    - `fn ring3_background_task_token`
    - `struct FlowCorrectedThompsonBandit` · 12 Methoden
    - `struct DiagonalApproximationBandit` · 3 Methoden
    - `trait OffPolicyCompatibility` · Methoden: verify_positivity
    - `enum OffPolicyError`
    - `struct FcTsSamplingDistribution` · 3 Methoden
    - `struct FcTsArmSet` · 2 Methoden
    - `const MIN_BANDIT_SHADOW_SAMPLES`
    - `const MAX_CUMULATIVE_REGRET_THRESHOLD`
    - `struct BanditShadowReport`
    - `trait BanditDefaultFlipGate` · Methoden: should_flip
    - `struct DefaultBanditFlipGate`
- `homeostat`
  - 2 struct
    - `struct RerankPidController` · 9 Methoden
    - `struct RerankDeadline` · 5 Methoden
- `lyapunov`
  - 1 enum, 2 struct
    - `struct DriftReason`
    - `enum LyapunovResult` · 2 Methoden
    - `struct LyapunovDriftWatcher` · 7 Methoden
- `offpolicy`
  - 3 struct
    - `struct RandomizedLoggingPolicy` · 3 Methoden
    - `struct OffPolicyStats`
    - `struct OffPolicyEvaluator` · 7 Methoden
- `pid`
  - 2 const, 1 struct, 1 trait
    - `const PID_MIN_POOL_SIZE_DEFAULT`
    - `const PID_MAX_POOL_SIZE_DEFAULT`
    - `trait AntiWindupController` · Methoden: update_with_anti_windup
    - `struct PidController` · 4 Methoden
- `pid_latency_controller`
  - 4 const, 2 struct
    - `const DEFAULT_TARGET_LATENCY_MS`
    - `const MIN_SCALING_FACTOR`
    - `const MAX_SCALING_FACTOR`
    - `const MAX_INTEGRAL`
    - `struct PidLatencyController` · 6 Methoden
    - `struct LatencyBudgetGuard` · 5 Methoden
- `rie_greedy`
  - 1 enum, 1 struct
    - `enum RieGreedyError`
    - `struct RieGreedyProfile` · 5 Methoden
- `rng`
  - 1 struct, 1 trait
    - `trait Rng` · Methoden: next_u64, next_f32
    - `struct SeededRng` · 1 Methoden
- `shadow_mode`
  - 2 struct, 1 trait
    - `struct ShadowDiscrepancy`
    - `trait ShadowSink` · Methoden: record
    - `struct TracingShadowSink`
- `strategy`
  - 1 enum
    - `enum RetrievalStrategy`
- Interne Re-Exporte (`pub use`): `*`, `*`, `*`, `*`, `*`, `*`, `*`, `*`, `*`, `*`, `*`, `*`

**Eingehend (wer nutzt dieses Crate)**

| Nutzer | Symbole prod | Symbole dev |
|---|---|---|
| `contextra-router` | `bandit`, `lyapunov`, `offpolicy`, `offpolicy::RandomizedLoggingPolicy` | `bandit::BanditProfileState`×2, `BanditError`, `FcTsError::InvalidConfig` |
| `contextra-engine` | `PidController`×2 | `PidController::default` |
| `contextra-db` | – | `PidController` |

**Nach außen ungenutzte öffentliche Items (Heuristik, 64 von 68)**: `SHERMAN_MORRISON_REFACTORIZATION_INTERVAL`, `AlignedF32Vec`, `BanditImplementation`, `SketchMatrix`, `BanditPolicy`, `Clock`, `SystemClock`, `Clock`, `Rng`, `AdaptiveController`, `BanditArmSelector`, `StateSnapshot`, `_assert_contract_conformance`, `DecaySignalInputs`, `DecayControllerConfig`, `AdaptiveDecayController`, `DriftSignal`, `DriftDetector`, `CatoniDriftDetector`, `EnsembleDriftWatcher`, `DriftPolicyBridge`, `AdaptError`, `Result`, `FcTsError`, `FcTsRng`, `SplitMix64`, `FcTsConfig`, `Ring3Token`, `ring3_background_task_token`, `FlowCorrectedThompsonBandit` …

**Auffälligkeiten**

- contextra-types: genutzt, aber nicht in Cargo.toml deklariert (cfg-/Feature-Pfad oder Fehler)

### contextra-agent

- Pfad: `crates/contextra-agent` · Zeilen: 9539 · Rust-Dateien: 43 · Ring: 3
- Interne Cargo-Abhängigkeiten: `contextra-store` (dev), `contextra-testkit` (dev), `contextra-text` (dev), `contextra-vector` (dev), `contextra-checkpoint` (normal), `contextra-db` (normal), `contextra-graph` (normal), `contextra-ports` (normal), `contextra-router` (normal), `contextra-types` (normal)

**Ausgehend (Importe aus anderen Workspace-Crates)**

| Ziel | prod | dev | Dateien (prod) | Symbole (prod) | Symbole (nur dev) |
|---|---:|---:|---:|---|---|
| `contextra-types` | 26 | 61 | 7 | `ContextraError::InvalidInput`×10, `Result`×4, `TxId`×3, `WorkflowState`×3, `TokenBudget::new`×2, `ContextraError::Internal`, `DocId::from_key`, `types::budget`, `error` | `TokenBudget`×13, `TxId::new`×5, `ContextraError::Conflict`, `ContextraError::Serialization`, `ContextraError` |
| `contextra-ports` | 15 | 20 | 5 | `BoxFuture`×7, `MetricsSink`×3, `StorageEngine`×2, `VectorIndex`×2, `Checkpoint` | `SystemClock` |
| `contextra-db` | 5 | 42 | 4 | `Collection`×2, `Contextra`×2, `volatile_vault` | `Contextra::open_with_config`×19, `ContextraConfig::default`×16, `volatile_vault::VaultConfig`×3, `ContextraConfig`×2, `DistanceMetric::Cosine` |
| `contextra-router` | 4 | 0 | 2 | `DefaultRouterEngine`, `DecisionId`, `RoutingOutcome::Success`, `RoutingOutcome::Rejected` | – |
| `contextra-checkpoint` | 0 | 1 | 0 | – | `StateCheckpoint` |
| `contextra-graph` | 0 | 2 | 0 | – | `CsrGraph`×2 |
| `contextra-testkit` | 0 | 3 | 0 | – | `InMemoryStorageEngine`×2, `TestMetricsSink` |
| `contextra-text` | 0 | 5 | 0 | – | `Language::English`×5 |

**Pro Komponente**

- `audit` (448 Zeilen)
  - → `contextra-db`: prod: `Collection`
  - → `contextra-graph`: dev: `CsrGraph`
  - → `contextra-ports`: prod: `VectorIndex`×2, `StorageEngine`
  - → `contextra-testkit`: dev: `InMemoryStorageEngine`
  - → `contextra-text`: dev: `Language::English`×4
  - → `contextra-types`: prod: `ContextraError::InvalidInput`×4, `Result`, `TxId`, `ContextraError::Internal`, `DocId::from_key` · dev: `ContextraError::InvalidInput`×5, `ContextraError::Conflict`, `Result`, `ContextraError::Internal` … (+3)
- `budget` (73 Zeilen)
  - → `contextra-types`: prod: `types::budget`, `Result`
- `clm_scratchpad` (683 Zeilen)
  - → `contextra-db`: prod: `volatile_vault`
- `context` (519 Zeilen)
  - → `contextra-db`: dev: `ContextraConfig::default`×4, `Contextra::open_with_config`×4, `volatile_vault::VaultConfig`
  - → `contextra-router`: prod: `DefaultRouterEngine`, `DecisionId`
  - → `contextra-types`: prod: `ContextraError::InvalidInput`×2, `error`, `Result` · dev: `Result`×4
- `dlq` (359 Zeilen)
  - → `contextra-ports`: prod: `MetricsSink`×3, `StorageEngine` · dev: `MetricsSink`
  - → `contextra-testkit`: dev: `InMemoryStorageEngine`, `TestMetricsSink`
- `engine` (1153 Zeilen)
  - → `contextra-db`: prod: `Contextra`×2 · dev: `volatile_vault::VaultConfig`
  - → `contextra-ports`: prod: `BoxFuture`×5, `Checkpoint` · dev: `BoxFuture`×2
  - → `contextra-router`: prod: `RoutingOutcome::Success`, `RoutingOutcome::Rejected`
  - → `contextra-types`: prod: `WorkflowState`×3, `TxId`×2, `TokenBudget::new`×2 · dev: `Result`×2, `TokenBudget`, `TxId`, `TxId::new`
- `event_source` (261 Zeilen)
  - → `contextra-db`: prod: `Collection`
  - → `contextra-types`: prod: `ContextraError::InvalidInput`×3
- `graph` (290 Zeilen)
  - → `contextra-types`: prod: `ContextraError::InvalidInput`
- `step` (104 Zeilen)
  - → `contextra-ports`: prod: `BoxFuture`
- `tools` (510 Zeilen)
  - → `contextra-ports`: prod: `BoxFuture` · dev: `BoxFuture`
  - → `contextra-types`: prod: `Result`
- `examples/minimal_pipeline.rs` (70 Zeilen)
  - → `contextra-ports`: dev: `BoxFuture`
  - → `contextra-types`: dev: `Result`
- `tests/agent_recovery.rs` (383 Zeilen)
  - → `contextra-checkpoint`: dev: `StateCheckpoint`
  - → `contextra-ports`: dev: `BoxFuture`
  - → `contextra-types`: dev: `TxId::new`×3, `Result`×2, `TokenBudget`, `ContextraError::Internal`
- `tests/audit_isolation_test.rs` (371 Zeilen)
  - → `contextra-graph`: dev: `CsrGraph`
  - → `contextra-text`: dev: `Language::English`
- `tests/boundary_validation_tests.rs` (133 Zeilen)
  - → `contextra-ports`: dev: `BoxFuture`
  - → `contextra-types`: dev: `ContextraError`, `Result`
- `tests/budget_race_test.rs` (301 Zeilen)
  - → `contextra-ports`: dev: `BoxFuture`
  - → `contextra-types`: dev: `ContextraError::Internal`
- `tests/clm_scratchpad_reset_flow.rs` (96 Zeilen)
  - → `contextra-db`: dev: `volatile_vault::VaultConfig`
  - → `contextra-ports`: dev: `SystemClock`
- `tests/clock_tests.rs` (154 Zeilen)
  - → `contextra-db`: dev: `Contextra::open_with_config`×4, `ContextraConfig::default`×2
- `tests/contract_tests.rs` (208 Zeilen)
  - → `contextra-ports`: dev: `BoxFuture`, `StorageEngine`
  - → `contextra-types`: dev: `TokenBudget`, `Result`
- `tests/crash_recovery_injection.rs` (181 Zeilen)
  - → `contextra-ports`: dev: `BoxFuture`
  - → `contextra-types`: dev: `Result`
- `tests/dlq_clock_determinism_test.rs` (104 Zeilen)
  - → `contextra-db`: dev: `ContextraConfig::default`, `Contextra::open_with_config`
- `tests/dlq_tests.rs` (300 Zeilen)
  - → `contextra-db`: dev: `ContextraConfig::default`×5, `Contextra::open_with_config`×5
  - → `contextra-ports`: dev: `BoxFuture`
  - → `contextra-types`: dev: `TxId`×2
- `tests/e2e_integration.rs` (184 Zeilen)
  - → `contextra-ports`: dev: `BoxFuture`
  - → `contextra-types`: dev: `TokenBudget`, `Result`
- `tests/event_loop_integration.rs` (316 Zeilen)
  - → `contextra-ports`: dev: `BoxFuture`
  - → `contextra-types`: dev: `Result`×3, `TokenBudget`
- `tests/final_state_test.rs` (59 Zeilen)
  - → `contextra-db`: dev: `Contextra`, `ContextraConfig`
  - → `contextra-types`: dev: `TokenBudget`, `Result`
- `tests/graph_integration.rs` (106 Zeilen)
  - → `contextra-db`: dev: `ContextraConfig`, `DistanceMetric::Cosine`, `Contextra::open_with_config`
  - → `contextra-types`: dev: `TokenBudget::new`
- `tests/loop_protection_test.rs` (181 Zeilen)
  - → `contextra-ports`: dev: `BoxFuture`
  - → `contextra-types`: dev: `Result`
- `tests/persistence_test.rs` (142 Zeilen)
  - → `contextra-ports`: dev: `BoxFuture`
  - → `contextra-types`: dev: `TokenBudget`, `Result`
- `tests/regression_agt_a.rs` (168 Zeilen)
  - → `contextra-ports`: dev: `BoxFuture`
  - → `contextra-types`: dev: `Result`
- `tests/session_concurrency_test.rs` (204 Zeilen)
  - → `contextra-db`: dev: `ContextraConfig::default`×2, `Contextra::open_with_config`×2
  - → `contextra-ports`: dev: `BoxFuture`
- `tests/system_prompt_pinning_latency.rs` (77 Zeilen)
  - → `contextra-db`: dev: `ContextraConfig::default`×2, `Contextra::open_with_config`×2
  - → `contextra-types`: dev: `Result`×2, `TokenBudget`, `error`
- `tests/workflow_tests.rs` (510 Zeilen)
  - → `contextra-ports`: dev: `BoxFuture`
  - → `contextra-types`: dev: `TokenBudget`×5, `ContextraError::InvalidInput`×2, `Result`

**Öffentliche Schnittstelle** (erreichbar über `pub mod`-Kette oder `pub use`)

- `audit`
  - 2 fn, 3 struct
    - `struct AuditEntry` ⚠ fremd: `contextra-types`
    - `struct MigrationStats`
    - `struct AuditLog` · 4 Methoden ⚠ fremd: `contextra-ports`
    - `fn validate_audit_payload_and_error` ⚠ fremd: `contextra-types`
    - `fn migrate_legacy_audit_entries` ⚠ fremd: `contextra-db`, `contextra-ports`, `contextra-types`
- `budget`
  - 1 fn
    - `fn reserve_tokens` ⚠ fremd: `contextra-types`
- `clm_scratchpad`
  - 1 enum, 9 struct, 2 trait
    - `struct ContextEditAuditRecord`
    - `trait ContextEditAuditSink` · Methoden: record_edit
    - `struct NoopContextEditAuditSink`
    - `struct CountingContextEditAuditSink` · 2 Methoden
    - `trait ScratchpadCacheInvalidator` · Methoden: invalidate_scratchpad_scope
    - `struct NoopScratchpadCacheInvalidator`
    - `struct CountingScratchpadCacheInvalidator` · 2 Methoden
    - `struct PinnedRegionId` · 2 Methoden
    - `enum ScratchpadEditOp`
    - `struct VaultResetReceipt`
    - `struct ScratchpadCheckpoint`
    - `struct ClmScratchpad` · 15 Methoden
- `context`
  - 2 const, 1 enum, 2 fn, 2 struct
    - `const MAX_ID_LEN`
    - `const MAX_TELEMETRY_EVENTS`
    - `fn validate_task_id` ⚠ fremd: `contextra-types`
    - `fn validate_node_id` ⚠ fremd: `contextra-types`
    - `enum AgentStatus`
    - `struct AgentContext` · 9 Methoden ⚠ fremd: `contextra-db`, `contextra-router`
    - `struct AgentEngine` · 3 Methoden
- `dlq`
  - 1 struct
    - `struct DeadLetterQueue` · 9 Methoden
- `engine`
  - 1 const, 1 enum, 1 fn, 1 struct
    - `fn evaluate_condition_expr` *(re-exportiert)*
    - `enum EventLoopExitReason`
    - `const MAX_WORKFLOW_STEPS`
    - `struct OrchestratorEngine` · 12 Methoden
- `event_source`
  - 2 const, 3 struct, 1 trait
    - `const MAX_EVENT_SOURCE_CAPACITY`
    - `struct BackgroundEvent` · 2 Methoden
    - `trait EventSource` · Methoden: next_event, is_exhausted, wait_for_event
    - `const MAX_PENDING_EVENTS_CAPACITY`
    - `struct PollingDocumentEventSource` · 4 Methoden
    - `struct VecEventSource` · 1 Methoden
- `goal_condition`
  - 1 enum, 1 fn
    - `enum GoalCondition` · 2 Methoden
    - `fn parse_legacy_condition_string`
- `graph`
  - 1 const, 1 enum, 3 struct, 1 type
    - `type NodeId`
    - `const MAX_TEXT_LEN`
    - `enum NodeType`
    - `struct AgentNode`
    - `struct WorkflowEdge` · 2 Methoden
    - `struct StateGraph` · 7 Methoden
- `step`
  - 1 enum, 2 struct, 1 trait
    - `struct StepDeadLetter`
    - `enum DeadLetterReason`
    - `struct StepResult`
    - `trait AgentTool` · Methoden: name, estimated_cost, execute, timeout_ms, is_retriable, max_retries
- `tools`
  - 1 const, 3 struct, 1 trait
    - `const MAX_MANIFEST_DESC_LEN`
    - `struct ToolManifest` · 5 Methoden
    - `struct ToolRegistry` · 9 Methoden
    - `trait ContextRetriever` · Methoden: scan_prefix
    - `struct RetrieveTool` · 3 Methoden

**Eingehend (wer nutzt dieses Crate)**

Wird von keinem anderen analysierten Workspace-Crate importiert.

**Nach außen ungenutzte öffentliche Items (Heuristik, 53 von 53)**: `AuditEntry`, `MigrationStats`, `AuditLog`, `validate_audit_payload_and_error`, `migrate_legacy_audit_entries`, `reserve_tokens`, `ContextEditAuditRecord`, `ContextEditAuditSink`, `NoopContextEditAuditSink`, `CountingContextEditAuditSink`, `ScratchpadCacheInvalidator`, `NoopScratchpadCacheInvalidator`, `CountingScratchpadCacheInvalidator`, `PinnedRegionId`, `ScratchpadEditOp`, `VaultResetReceipt`, `ScratchpadCheckpoint`, `ClmScratchpad`, `MAX_ID_LEN`, `MAX_TELEMETRY_EVENTS`, `validate_task_id`, `validate_node_id`, `AgentStatus`, `AgentContext`, `AgentEngine`, `DeadLetterQueue`, `evaluate_condition_expr`, `EventLoopExitReason`, `MAX_WORKFLOW_STEPS`, `OrchestratorEngine` …

**Auffälligkeiten**

- contextra-checkpoint: nur in Tests/Benches genutzt, aber als normale Abhängigkeit deklariert → dev-dependency?
- contextra-graph: nur in Tests/Benches genutzt, aber als normale Abhängigkeit deklariert → dev-dependency?
- contextra-store: kein Code-Zugriff gefunden (prüfen: Makro, Feature, cfg, oder entfernen)
- contextra-vector: kein Code-Zugriff gefunden (prüfen: Makro, Feature, cfg, oder entfernen)

### contextra-audit-export

- Pfad: `crates/contextra-audit-export` · Zeilen: 1645 · Rust-Dateien: 15 · Ring: 0
- Interne Cargo-Abhängigkeiten: `contextra-types` (normal/optional via types-compat)

**Ausgehend (Importe aus anderen Workspace-Crates)**

| Ziel | prod | dev | Dateien (prod) | Symbole (prod) | Symbole (nur dev) |
|---|---:|---:|---:|---|---|
| `contextra-types` | 4 | 0 | 1 | `TenantId`×4 | – |

**Pro Komponente**

- `(crate-root)` (350 Zeilen)
  - → `contextra-types`: prod: `TenantId`×4

**Öffentliche Schnittstelle** (erreichbar über `pub mod`-Kette oder `pub use`)

- `(crate-root)`
  - 1 fn, 7 struct, 2 trait
    - `struct TenantId` · 5 Methoden ⚠ fremd: `contextra-types`
    - `struct ControllerContact` · 1 Methoden
    - `struct ThirdCountryTransfer` · 1 Methoden
    - `struct RetentionPeriod` · 1 Methoden
    - `struct EgressEventSummary`
    - `struct DeletionProofSummary`
    - `struct ProcessingRegisterEntry` · 2 Methoden ⚠ fremd: `contextra-types`
    - `trait ProcessingRegisterSource` · Methoden: collect_entries
    - `trait RecordSource` · Methoden: collect_records
    - `fn render_register_json`
- `bsi_mapping`
  - 2 fn, 1 struct
    - `struct BsiMappingEntry`
    - `fn bsi_mapping_table`
    - `fn render_bsi_mapping_markdown`
- `csv_template`
  - 1 fn
    - `fn render_register_csv`
- `error`
  - 1 enum
    - `enum AuditExportError`
- `markdown_template`
  - 2 fn
    - `fn sanitize_markdown_cell`
    - `fn render_register_markdown`
- `testkit`
  - 1 struct
    - `struct InMemoryProcessingRegisterSource` · 4 Methoden

**Eingehend (wer nutzt dieses Crate)**

Wird von keinem anderen analysierten Workspace-Crate importiert.

**Nach außen ungenutzte öffentliche Items (Heuristik, 18 von 18)**: `BsiMappingEntry`, `bsi_mapping_table`, `render_bsi_mapping_markdown`, `render_register_csv`, `AuditExportError`, `TenantId`, `ControllerContact`, `ThirdCountryTransfer`, `RetentionPeriod`, `EgressEventSummary`, `DeletionProofSummary`, `ProcessingRegisterEntry`, `ProcessingRegisterSource`, `RecordSource`, `render_register_json`, `sanitize_markdown_cell`, `render_register_markdown`, `InMemoryProcessingRegisterSource`

### contextra-avv-generator

- Pfad: `crates/contextra-avv-generator` · Zeilen: 1538 · Rust-Dateien: 13 · Ring: 0
- Interne Cargo-Abhängigkeiten: keine

**Ausgehend (Importe aus anderen Workspace-Crates)**

Keine.

**Pro Komponente**

Keine Importe aus anderen Workspace-Crates.

**Öffentliche Schnittstelle** (erreichbar über `pub mod`-Kette oder `pub use`)

- `(crate-root)`
  - 3 fn, 3 struct
    - `struct TenantId` · 4 Methoden
    - `struct AvvContext` · 4 Methoden
    - `struct TechnicalMeasure`
    - `fn render_avv_markdown`
    - `fn render_avv_markdown_with_date`
    - `fn default_technical_measures`
- `assistance`
  - 1 fn
    - `fn render_section`
- `audit`
  - 1 fn
    - `fn render_section`
- `confidentiality`
  - 1 fn
    - `fn render_section`
- `data_subject_rights`
  - 1 fn
    - `fn render_section`
- `error`
  - 1 enum, 1 type
    - `enum AvvError`
    - `type AvvGeneratorError`
- `instructions`
  - 1 fn
    - `fn render_section`
- `template`
  - 3 fn
    - `fn escape_markdown`
    - `fn render`
    - `fn render_with_date`
- `termination`
  - 1 fn
    - `fn render_section`

**Eingehend (wer nutzt dieses Crate)**

Wird von keinem anderen analysierten Workspace-Crate importiert.

**Nach außen ungenutzte öffentliche Items (Heuristik, 17 von 17)**: `render_section`, `render_section`, `render_section`, `render_section`, `AvvError`, `AvvGeneratorError`, `render_section`, `TenantId`, `AvvContext`, `TechnicalMeasure`, `render_avv_markdown`, `render_avv_markdown_with_date`, `default_technical_measures`, `escape_markdown`, `render`, `render_with_date`, `render_section`

### contextra-bench

- Pfad: `benchmarks/contextra-bench` · Zeilen: 4774 · Rust-Dateien: 15 · Ring: tooling
- Interne Cargo-Abhängigkeiten: `contextra-sys` (dev), `contextra-db` (normal), `contextra-store` (normal), `contextra-text` (normal), `contextra-vector` (normal)

**Ausgehend (Importe aus anderen Workspace-Crates)**

| Ziel | prod | dev | Dateien (prod) | Symbole (prod) | Symbole (nur dev) |
|---|---:|---:|---:|---|---|
| `contextra-types` | 8 | 1 | 2 | `ContextraError::Internal`×3, `Result`×2, `ContextraError::SnapshotUnsupportedForSignal`×2, `ContextraError::CapabilityUnsupported` | `TxId` |
| `contextra-infer-candle` | 2 | 0 | 1 | `CandleEmbedClient::from_dir`, `model_registry::CandleQuantization::Q4KM` | – |
| `contextra-ports` | 2 | 1 | 2 | `VectorIndex`, `EmbeddingProvider` | `StorageEngine` |
| `contextra-rank` | 1 | 0 | 1 | `RankError` | – |
| `contextra-sys` | 0 | 1 | 0 | – | `ensure_fd_limit` |

**Pro Komponente**

- `(crate-root)` (1388 Zeilen)
  - → `contextra-infer-candle`: prod: `CandleEmbedClient::from_dir`, `model_registry::CandleQuantization::Q4KM`
  - → `contextra-ports`: prod: `EmbeddingProvider`
  - → `contextra-types`: prod: `ContextraError::Internal`×3, `Result`, `ContextraError::CapabilityUnsupported`
- `ann_benchmarks` (240 Zeilen)
  - → `contextra-ports`: prod: `VectorIndex`
- `matrix` (185 Zeilen)
  - → `contextra-rank`: prod: `RankError`
- `path_rag_sweep` (318 Zeilen)
  - → `contextra-types`: prod: `ContextraError::SnapshotUnsupportedForSignal`×2, `Result`
- `benches/competitive_kv.rs` (554 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-sys`: dev: `ensure_fd_limit`
  - → `contextra-types`: dev: `TxId`

**Öffentliche Schnittstelle** (erreichbar über `pub mod`-Kette oder `pub use`)

- `adapters`
  - 5 struct, 2 trait
    - `struct BenchmarkDocument`
    - `struct EngineHit`
    - `struct QueryLatencyStats`
    - `trait FullTextEngineAdapter` · Methoden: name, index_batch, search, commit
    - `trait VectorEngineAdapter` · Methoden: name, dimension, insert_vectors, search_knn
    - `struct TantivyAdapterSketch` · 1 Methoden
    - `struct LanceDbAdapterSketch` · 1 Methoden
- `ann_benchmarks`
  - 5 fn, 1 struct
    - `struct AnnBenchResult`
    - `fn generate_synthetic_vectors`
    - `fn compute_ground_truth`
    - `fn recall_at_k`
    - `fn run_ann_benchmark`
    - `fn run_ann_benchmark_synthetic_ci`
- `beir_eval`
  - 6 fn, 3 struct
    - `struct BeirDocument`
    - `struct BeirQuery`
    - `struct BeirMetrics`
    - `fn load_beir_corpus`
    - `fn load_beir_queries`
    - `fn ndcg_at_k`
    - `fn map_at_k`
    - `fn recall_at_k`
    - `fn run_beir_eval`
- `compare`
  - 2 fn, 4 struct
    - `struct LongMemEvalMetricsSummary`
    - `struct LocomoMetricsSummary`
    - `struct CombinedMetrics`
    - `struct MetricComparisonResult`
    - `fn compare_metrics`
    - `fn compare_metrics_files`
- `matrix`
  - 1 enum, 6 fn, 2 struct
    - `enum EvaluatedStrategy` · 1 Methoden
    - `struct StrategyMatrixRow`
    - `struct ComparisonMatrixReport`
    - `fn to_rank_items`
    - `fn from_rank_items`
    - `fn to_search_results`
    - `fn fuse_via_rank_rrf` ⚠ fremd: `contextra-rank`
    - `fn fuse_via_rank_weighted_rrf`
    - `fn fuse_via_rank_score_normalized`
- `metrics`
  - 5 fn, 4 struct
    - `struct RelevanceGrade`
    - `struct RankedResult`
    - `struct QueryEvaluation`
    - `struct AggregateMetrics`
    - `fn calculate_recall_at_k`
    - `fn calculate_mrr_at_k`
    - `fn calculate_ndcg_at_k`
    - `fn calculate_latency_percentiles`
    - `fn read_process_rss_bytes`
- `path_rag_sweep`
  - 2 fn, 2 struct
    - `struct PathRagSweepMetric`
    - `struct PathRagSweepReport`
    - `fn run_pathrag_sweep_long_mem_eval` ⚠ fremd: `contextra-types`
    - `fn run_pathrag_sweep_locomo` ⚠ fremd: `contextra-types`
- `regression_gate`
  - 1 const, 1 fn, 1 struct
    - `const DEFAULT_TOLERANCE_THRESHOLD`
    - `struct RegressionGateResult`
    - `fn run_regression_gate`
- Zusätzlich 7 `pub`-Items in privaten Modulen ohne Re-Export (nicht Teil der echten API).

**Eingehend (wer nutzt dieses Crate)**

Wird von keinem anderen analysierten Workspace-Crate importiert.

**Nach außen ungenutzte öffentliche Items (Heuristik, 53 von 53)**: `BenchmarkDocument`, `EngineHit`, `QueryLatencyStats`, `FullTextEngineAdapter`, `VectorEngineAdapter`, `TantivyAdapterSketch`, `LanceDbAdapterSketch`, `AnnBenchResult`, `generate_synthetic_vectors`, `compute_ground_truth`, `recall_at_k`, `run_ann_benchmark`, `run_ann_benchmark_synthetic_ci`, `BeirDocument`, `BeirQuery`, `BeirMetrics`, `load_beir_corpus`, `load_beir_queries`, `ndcg_at_k`, `map_at_k`, `recall_at_k`, `run_beir_eval`, `LongMemEvalMetricsSummary`, `LocomoMetricsSummary`, `CombinedMetrics`, `MetricComparisonResult`, `compare_metrics`, `compare_metrics_files`, `EvaluatedStrategy`, `StrategyMatrixRow` …

**Auffälligkeiten**

- contextra-db: kein Code-Zugriff gefunden (prüfen: Makro, Feature, cfg, oder entfernen)
- contextra-store: kein Code-Zugriff gefunden (prüfen: Makro, Feature, cfg, oder entfernen)
- contextra-text: kein Code-Zugriff gefunden (prüfen: Makro, Feature, cfg, oder entfernen)
- contextra-vector: kein Code-Zugriff gefunden (prüfen: Makro, Feature, cfg, oder entfernen)
- contextra-infer-candle: genutzt, aber nicht in Cargo.toml deklariert (cfg-/Feature-Pfad oder Fehler)
- contextra-ports: genutzt, aber nicht in Cargo.toml deklariert (cfg-/Feature-Pfad oder Fehler)
- contextra-rank: genutzt, aber nicht in Cargo.toml deklariert (cfg-/Feature-Pfad oder Fehler)
- contextra-types: genutzt, aber nicht in Cargo.toml deklariert (cfg-/Feature-Pfad oder Fehler)

### contextra-checkpoint

- Pfad: `crates/contextra-checkpoint` · Zeilen: 8557 · Rust-Dateien: 27 · Ring: 1
- Interne Cargo-Abhängigkeiten: `contextra-testkit` (dev), `contextra-ports` (normal), `contextra-types` (normal)

**Ausgehend (Importe aus anderen Workspace-Crates)**

| Ziel | prod | dev | Dateien (prod) | Symbole (prod) | Symbole (nur dev) |
|---|---:|---:|---:|---|---|
| `contextra-ports` | 55 | 8 | 4 | `Clock`×20, `StorageEngine`×14, `Vfs`×7, `MetricsSink`×6, `BoxFuture`×3, `SystemClock`×2, `Checkpoint`×2, `CheckpointCoordinator` | – |
| `contextra-mvcc` | 1 | 0 | 1 | `SnapshotRegistry` | – |
| `contextra-testkit` | 0 | 3 | 0 | – | `InMemoryStorageEngine`×2, `FaultVfs` |
| `contextra-types` | 0 | 13 | 0 | – | `TxId`×4, `Result`×3, `ContextraError`×2, `ContextraError::Internal`×2, `TxId::new` … (+1) |

**Pro Komponente**

- `guard` (1100 Zeilen)
  - → `contextra-ports`: prod: `Clock`×9, `StorageEngine`×6, `MetricsSink`×3, `SystemClock` · dev: `MetricsSink`×2, `Clock`
- `hardlink_cloner` (259 Zeilen)
  - → `contextra-mvcc`: prod: `SnapshotRegistry`
  - → `contextra-ports`: prod: `BoxFuture`×2
- `manifest` (210 Zeilen)
  - → `contextra-types`: dev: `TxId`
- `orphan` (845 Zeilen)
  - → `contextra-ports`: prod: `Clock`×8, `Vfs`×7, `SystemClock`, `StorageEngine`
  - → `contextra-testkit`: dev: `FaultVfs`
- `store` (1674 Zeilen)
  - → `contextra-ports`: prod: `StorageEngine`×7, `Clock`×3, `MetricsSink`×3, `Checkpoint`×2, `BoxFuture`, `CheckpointCoordinator` · dev: `CheckpointCoordinator`×5
- `examples/checkpoint_memvfs.rs` (34 Zeilen)
  - → `contextra-testkit`: dev: `InMemoryStorageEngine`
  - → `contextra-types`: dev: `TxId::new`, `TxId::INTERNAL_BASE`
- `tests/checkpoint_clock_determinism.rs` (178 Zeilen)
  - → `contextra-types`: dev: `TxId`
- `tests/checkpoint_systematic_crash.rs` (225 Zeilen)
  - → `contextra-testkit`: dev: `InMemoryStorageEngine`
- `tests/clock_determinism.rs` (114 Zeilen)
  - → `contextra-types`: dev: `TxId`
- `tests/hardlink_cloner_tests.rs` (151 Zeilen)
  - → `contextra-types`: dev: `Result`×3, `ContextraError`
- `tests/manifest_fault_injection.rs` (82 Zeilen)
  - → `contextra-types`: dev: `TxId`
- `tests/time_travel_correctness.rs` (737 Zeilen)
  - → `contextra-types`: dev: `ContextraError::Internal`×2, `ContextraError`

**Öffentliche Schnittstelle** (erreichbar über `pub mod`-Kette oder `pub use`)

- `manifest`
  - 1 struct
    - `struct CheckpointManifest` *(re-exportiert)* · 2 Methoden
- Zusätzlich 37 `pub`-Items in privaten Modulen ohne Re-Export (nicht Teil der echten API).

**Eingehend (wer nutzt dieses Crate)**

| Nutzer | Symbole prod | Symbole dev |
|---|---|---|
| `contextra-engine` | `InstanceOrphanRegistry`×2, `InstanceOrphanRegistry::new`, `(crate)` | – |
| `contextra-agent` | – | `StateCheckpoint` |
| `contextra-db` | – | `PersistentCheckpointStore`×2, `clear_all_orphaned_checkpoints`, `get_orphaned_checkpoints` |

**Nach außen ungenutzte öffentliche Items (Heuristik, 1 von 1)**: `CheckpointManifest`

**Auffälligkeiten**

- contextra-types: nur in Tests/Benches genutzt, aber als normale Abhängigkeit deklariert → dev-dependency?
- contextra-mvcc: genutzt, aber nicht in Cargo.toml deklariert (cfg-/Feature-Pfad oder Fehler)

### contextra-cognition

- Pfad: `crates/contextra-cognition` · Zeilen: 9556 · Rust-Dateien: 38 · Ring: 3
- Interne Cargo-Abhängigkeiten: `contextra-testkit` (dev), `contextra-engine` (normal), `contextra-graph` (normal), `contextra-ports` (normal), `contextra-types` (normal)

**Ausgehend (Importe aus anderen Workspace-Crates)**

| Ziel | prod | dev | Dateien (prod) | Symbole (prod) | Symbole (nur dev) |
|---|---:|---:|---:|---|---|
| `contextra-types` | 19 | 29 | 4 | `ContextraError::Internal`×3, `Result`×3, `TenantId`×3, `ContextraError::InvalidInput`×3, `ContextChunk`×2, `ContextraError::PolicyViolation`×2, `TxId`×2, `(crate)` | `MemoryType::Episodic`×6, `DocId::new`×5, `DocId` |
| `contextra-ports` | 2 | 6 | 2 | `SeededRng::from_entropy`, `(crate)` | `GroundingAssessment`×2, `BoxFuture`, `GroundingVerdict::Pass`, `StorageEngine`, `ResponseGroundingValidator` |
| `contextra-engine` | 1 | 12 | 1 | `(crate)` | `Language::English`×5, `collection::Collection`×4, `collection::Collection::new`×2, `Language::German` |
| `contextra-graph` | 1 | 15 | 1 | `(crate)` | `CsrGraph`×6, `HyperEdgeId`×3, `csr::EdgeType`×3, `hyperedge`×3 |
| `contextra-testkit` | 0 | 2 | 0 | – | `ManualClock`, `MockEmbedder` |

**Pro Komponente**

- `channel` (115 Zeilen)
  - → `contextra-types`: prod: `ContextraError::Internal`
- `consolidation_executor` (1033 Zeilen)
  - → `contextra-types`: prod: `ContextraError::Internal`×2, `ContextChunk`×2, `TxId`×2, `ContextraError::PolicyViolation` · dev: `MemoryType::Episodic`×2, `DocId::new`×2, `Result`, `ContextraError::Internal` … (+1)
- `context` (583 Zeilen)
  - → `contextra-types`: dev: `ContextraError::InvalidInput`
- `context_compaction` (1339 Zeilen)
  - → `contextra-ports`: prod: `SeededRng::from_entropy`
- `engine_local` (7 Zeilen)
  - → `contextra-engine`: prod: `(crate)`
- `graph_local` (9 Zeilen)
  - → `contextra-graph`: prod: `(crate)`
- `memory_consolidation` (1168 Zeilen)
  - → `contextra-ports`: dev: `BoxFuture`
  - → `contextra-types`: prod: `Result`×3, `TenantId`×3, `ContextraError::InvalidInput`×3, `ContextraError::PolicyViolation` · dev: `Result`×2, `ContextraError::Internal`
- `ports_local` (8 Zeilen)
  - → `contextra-ports`: prod: `(crate)`
- `synthesis_phase` (208 Zeilen)
  - → `contextra-types`: dev: `ContextraError::Internal`
- `types_local` (6 Zeilen)
  - → `contextra-types`: prod: `(crate)`
- `tests/aggregation_phase_tests.rs` (372 Zeilen)
  - → `contextra-graph`: dev: `HyperEdgeId`
- `tests/grounding_in_consolidation_pipeline.rs` (202 Zeilen)
  - → `contextra-engine`: dev: `collection::Collection::new`, `Language::English`
  - → `contextra-graph`: dev: `CsrGraph`
  - → `contextra-types`: dev: `MemoryType::Episodic`×2, `ContextraError::PolicyViolation`
- `tests/grounding_validator_rich_path_test.rs` (198 Zeilen)
  - → `contextra-engine`: dev: `collection::Collection::new`, `Language::German`
  - → `contextra-graph`: dev: `CsrGraph`
  - → `contextra-ports`: dev: `GroundingAssessment`×2, `GroundingVerdict::Pass`
  - → `contextra-types`: dev: `DocId::new`×3, `ContextChunk`×2, `ContextraError::PolicyViolation`×2, `MemoryType::Episodic`×2 … (+1)
- `tests/leanrag_aggregation.rs` (405 Zeilen)
  - → `contextra-graph`: dev: `HyperEdgeId`
- `tests/leanrag_stage3_wiring.rs` (179 Zeilen)
  - → `contextra-engine`: dev: `collection::Collection`, `Language::English`
  - → `contextra-graph`: dev: `csr::EdgeType`, `CsrGraph`, `hyperedge`
  - → `contextra-types`: dev: `Result`
- `tests/leanrag_worker_opt_in.rs` (376 Zeilen)
  - → `contextra-engine`: dev: `collection::Collection`, `Language::English`
  - → `contextra-graph`: dev: `csr::EdgeType`, `CsrGraph`, `hyperedge`
  - → `contextra-types`: dev: `Result`, `ContextraError::Internal`
- `tests/maintenance_scheduler_tests.rs` (216 Zeilen)
  - → `contextra-engine`: dev: `collection::Collection`, `Language::English`
  - → `contextra-graph`: dev: `CsrGraph`
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-testkit`: dev: `ManualClock`
- `tests/pre_condensed_input_intake.rs` (242 Zeilen)
  - → `contextra-ports`: dev: `ResponseGroundingValidator`
- `tests/rng_seed_determinism.rs` (183 Zeilen)
  - → `contextra-graph`: dev: `HyperEdgeId`
- `tests/semantic_aggregation_facade.rs` (263 Zeilen)
  - → `contextra-engine`: dev: `collection::Collection`, `Language::English`
  - → `contextra-graph`: dev: `csr::EdgeType`, `CsrGraph`, `hyperedge`
  - → `contextra-testkit`: dev: `MockEmbedder`
  - → `contextra-types`: dev: `Result`

**Öffentliche Schnittstelle** (erreichbar über `pub mod`-Kette oder `pub use`)

- `aggregation_phase`
  - 3 fn, 7 struct, 1 trait
    - `struct AggregationConfig` · 1 Methoden
    - `struct AggregationPhaseResult`
    - `struct ConsolidationPipelineResult`
    - `struct AggregationNode`
    - `struct AggregationEdge`
    - `struct SuperEdgeDraft`
    - `struct AlphaNode`
    - `trait SuperEdgeSink` · Methoden: invalidate_edge, write_super_edge, commit
    - `fn check_compaction_budget`
    - `fn compute_entity_community_hash`
    - `fn run_aggregation_pass`
- `channel`
  - 3 struct, 2 trait
    - `trait CognitiveInputChannel` · Methoden: send_pre_condensed
    - `trait CognitiveOutputSink` · Methoden: emit
    - `struct InMemoryCognitiveChannel` · 1 Methoden
    - `struct DirectCognitiveChannel`
    - `struct InMemoryOutputSink` · 4 Methoden
- `consolidation_executor`
  - 7 fn, 3 struct
    - `struct ConsolidationLockGuard` · 1 Methoden
    - `fn execute_consolidation_pass` ⚠ fremd: `contextra-types`
    - `struct GroundingValidatorAdapter`
    - `fn execute_background_consolidation` ⚠ fremd: `contextra-types`
    - `fn execute_background_consolidation_with_rich_validator` ⚠ fremd: `contextra-types`
    - `fn execute_leanrag_aggregation_stage` ⚠ fremd: `contextra-types`
    - `fn execute_sleep_cycle` ⚠ fremd: `contextra-types`
    - `fn start_consolidation_worker`
    - `fn start_consolidation_reaper`
    - `struct ConsolidationEngine` · 9 Methoden
- `consolidation_locks`
  - 1 struct
    - `struct ConsolidationNodesGuard` · 3 Methoden
- `context`
  - 2 struct
    - `struct ContextManager` · 6 Methoden
    - `struct SpatialFence` · 2 Methoden
- `context_compaction`
  - 1 fn, 2 struct
    - `fn cleanup_orphaned_consolidation_intents` *(re-exportiert)*
    - `struct ContextCompactor` *(re-exportiert)* · 7 Methoden
    - `struct ConsolidationSession` *(re-exportiert)* · 7 Methoden
- `graph_sink`
  - 1 struct
    - `struct CsrGraphSuperEdgeSink` · 1 Methoden
- `leanrag_input`
  - 1 const, 1 fn, 1 struct
    - `const DEFAULT_MAX_LEANRAG_NODES`
    - `struct LeanRagInputs`
    - `fn build_leanrag_inputs`
- `maintenance_config`
  - 2 struct
    - `struct PercolationConfig`
    - `struct MaintenanceConfig`
- `maintenance_scheduler`
  - 1 struct
    - `struct MaintenanceScheduler` · 8 Methoden
- `memory_consolidation`
  - 3 const, 7 fn, 8 struct
    - `struct ConsolidationConfig`
    - `struct TurnSegment`
    - `struct ConsolidationPhaseResult`
    - `fn group_turns_into_segments`
    - `fn detect_near_duplicates`
    - `fn run_consolidation_pass`
    - `struct SynthesisConfig`
    - `struct CommunityStabilityTracker` · 3 Methoden
    - `struct MetaChunk`
    - `struct SynthesisPhaseResult`
    - `fn compute_community_hash`
    - `fn run_structural_synthesis_pass` ⚠ fremd: `contextra-types`
    - `const MAX_PRE_CONDENSED_DIGEST_LEN`
    - `const MAX_PRE_CONDENSED_LABEL_LEN`
    - `const MAX_PRE_CONDENSED_PROVENANCE_COUNT`
    - `struct PreCondensedInput` ⚠ fremd: `contextra-types`
    - `fn intake_pre_condensed_inputs` ⚠ fremd: `contextra-types`
    - `fn compact_segment_via_context_compactor`
- `semantic_aggregation_facade`
  - 1 fn
    - `fn consolidate_semantic_hyperedges`
- `synthesis_phase`
  - 1 fn, 2 struct, 1 type
    - `struct SegmentSynthesisResult`
    - `type SynthesisPhaseResult`
    - `struct SynthesizedChunk`
    - `fn run_synthesis_pass`
- `transitivity_veto`
  - 2 fn
    - `fn validate_transitivity_veto`
    - `fn filter_candidates_with_transitivity_veto`
- Zusätzlich 3 `pub`-Items in privaten Modulen ohne Re-Export (nicht Teil der echten API).
- Interne Re-Exporte (`pub use`): `TraitSegmentSynthesizer`
- ⚠ Re-Exporte **fremder Crates**: `contextra_engine`, `contextra_graph`, `contextra_ports`, `contextra_types`

**Eingehend (wer nutzt dieses Crate)**

Wird von keinem anderen analysierten Workspace-Crate importiert.

**Nach außen ungenutzte öffentliche Items (Heuristik, 64 von 64)**: `AggregationConfig`, `AggregationPhaseResult`, `ConsolidationPipelineResult`, `AggregationNode`, `AggregationEdge`, `SuperEdgeDraft`, `AlphaNode`, `SuperEdgeSink`, `check_compaction_budget`, `compute_entity_community_hash`, `run_aggregation_pass`, `CognitiveInputChannel`, `CognitiveOutputSink`, `InMemoryCognitiveChannel`, `DirectCognitiveChannel`, `InMemoryOutputSink`, `ConsolidationLockGuard`, `execute_consolidation_pass`, `GroundingValidatorAdapter`, `execute_background_consolidation`, `execute_background_consolidation_with_rich_validator`, `execute_leanrag_aggregation_stage`, `execute_sleep_cycle`, `start_consolidation_worker`, `start_consolidation_reaper`, `ConsolidationEngine`, `ConsolidationNodesGuard`, `ContextManager`, `SpatialFence`, `cleanup_orphaned_consolidation_intents` …

### contextra-crypto

- Pfad: `crates/contextra-crypto` · Zeilen: 16267 · Rust-Dateien: 74 · Ring: 0
- Interne Cargo-Abhängigkeiten: `contextra-testkit` (dev), `contextra-ports` (normal), `contextra-types` (normal)

**Ausgehend (Importe aus anderen Workspace-Crates)**

| Ziel | prod | dev | Dateien (prod) | Symbole (prod) | Symbole (nur dev) |
|---|---:|---:|---:|---|---|
| `contextra-types` | 35 | 18 | 10 | `TenantId`×8, `Result`×7, `ScanMethod::FeatureDisabled`×3, `ContextraError::Crypto`×3, `TenantScoped`×2, `ScanMethod::Structural`×2, `ScanMethod::NotMeasured`×2, `ContextraError`×2, `RequestContext`, `SagaRecord` … (+4) | `ContextraError::Io`×4, `ContextraError::NotFound` |
| `contextra-ports` | 6 | 34 | 2 | `scanner::WitnessLayer`, `scanner::LayerWitness`, `Clock`, `scanner::ScanTechnique::NotMeasured`, `ScanTechnique`, `scanner` | `Result`×24, `SystemClock`×6, `scanner::ScanTechnique`×4 |
| `contextra-testkit` | 0 | 5 | 0 | – | `mint_witness_for_tests`×4, `FaultVfs` |

**Pro Komponente**

- `crypto` (822 Zeilen)
  - → `contextra-types`: prod: `TenantId`×4, `TenantScoped`×2 · dev: `TenantId`×2
- `deletion_proof` (1315 Zeilen)
  - → `contextra-ports`: prod: `scanner::WitnessLayer`, `scanner::LayerWitness`, `Clock`, `scanner::ScanTechnique::NotMeasured`, `ScanTechnique`, `scanner`
  - → `contextra-types`: prod: `ScanMethod::FeatureDisabled`×3, `ScanMethod::Structural`×2, `ScanMethod::NotMeasured`×2, `Result`, `RequestContext`, `SagaRecord`, `error::HnswDeletionError::VerificationFailed`, `ContextraError::Internal`, `TenantId`
- `error` (121 Zeilen)
  - → `contextra-types`: prod: `ContextraError`×2 · dev: `ContextraError`
- `kv_cipher` (350 Zeilen)
  - → `contextra-types`: prod: `Result`×6, `ContextraError::Crypto`×3, `TenantId`, `ModelFingerprint`, `ContextraError::ParseError`
- `kv_segment` (1453 Zeilen)
  - → `contextra-types`: prod: `TenantId`×2 · dev: `TenantId`
- `revocation_log` (450 Zeilen)
  - → `contextra-ports`: dev: `Result`×8, `SystemClock`
  - → `contextra-testkit`: dev: `FaultVfs`
  - → `contextra-types`: dev: `ContextraError::Io`×4, `ContextraError::NotFound`
- `benches/crypto_benchmarks.rs` (127 Zeilen)
  - → `contextra-types`: dev: `TenantId`
- `fuzz/fuzz_targets/deletion_proof_tamper.rs` (137 Zeilen)
  - → `contextra-ports`: dev: `scanner::ScanTechnique`, `SystemClock`
  - → `contextra-testkit`: dev: `mint_witness_for_tests`
- `tests/audit_chain_tests.rs` (213 Zeilen)
  - → `contextra-ports`: dev: `scanner::ScanTechnique`, `SystemClock`
  - → `contextra-testkit`: dev: `mint_witness_for_tests`
- `tests/campaign_cov_revocation_log.rs` (38 Zeilen)
  - → `contextra-ports`: dev: `SystemClock`
- `tests/campaign_j25.rs` (426 Zeilen)
  - → `contextra-types`: dev: `TenantId`
- `tests/campaign_j26_hypotheses.rs` (784 Zeilen)
  - → `contextra-ports`: dev: `Result`×8, `scanner::ScanTechnique`, `SystemClock`
  - → `contextra-testkit`: dev: `mint_witness_for_tests`
- `tests/deletion_proof_signature_version_serde.rs` (197 Zeilen)
  - → `contextra-ports`: dev: `scanner::ScanTechnique`, `SystemClock`
  - → `contextra-testkit`: dev: `mint_witness_for_tests`
- `tests/eviction_worker_tokio_runtime_nonblocking_regression.rs` (65 Zeilen)
  - → `contextra-types`: dev: `TenantId`
- `tests/kv_segment_concurrency.rs` (271 Zeilen)
  - → `contextra-types`: dev: `TenantId`
- `tests/kv_segment_integration.rs` (120 Zeilen)
  - → `contextra-types`: dev: `TenantId`
- `tests/kv_segment_proptests.rs` (82 Zeilen)
  - → `contextra-types`: dev: `TenantId`
- `tests/loom_kv_deferred_zeroize.rs` (161 Zeilen)
  - → `contextra-types`: dev: `TenantId`
- `tests/proptests.rs` (130 Zeilen)
  - → `contextra-types`: dev: `TenantId`
- `tests/revocation_log_vfs.rs` (201 Zeilen)
  - → `contextra-ports`: dev: `Result`×8
  - → `contextra-types`: dev: `ContextraError`

**Öffentliche Schnittstelle** (erreichbar über `pub mod`-Kette oder `pub use`)

- `aead`
  - 1 const, 4 fn
    - `const REC_AAD`
    - `fn seal_record_with_nonce`
    - `fn seal_record_with_rng`
    - `fn seal_record`
    - `fn open_record`
- `anti_tamper`
  - 2 fn, 2 struct
    - `struct VolatileEncryptionKey` · 3 Methoden
    - `fn constant_time_compare`
    - `fn verify_tag_constant_time`
    - `struct Blake3Chain` · 5 Methoden
- `audit_chain`
  - 1 enum, 1 fn, 5 struct
    - `enum DataClass`
    - `struct AuditChainEntry` · 1 Methoden
    - `struct AuditChainHeadSignature`
    - `struct AuditHeadAnchor` · 2 Methoden
    - `struct EncryptedCommitmentSalt` · 2 Methoden
    - `fn compute_record_commitment`
    - `struct AuditChain` · 10 Methoden
- `crypto`
  - 1 struct
    - `struct KeyManager` · 15 Methoden
- `dek`
  - 3 const, 3 fn, 3 struct, 2 type
    - `struct UnitKey` · 2 Methoden
    - `struct Dek` · 2 Methoden
    - `type Salt`
    - `type DocTag`
    - `struct AncestorSalts`
    - `const DEK_LABEL`
    - `const DEK_SALT_LABEL`
    - `const WRAP_AAD`
    - `fn derive_dek`
    - `fn wrap_dek`
    - `fn unwrap_dek`
- `deletion_proof`
  - 1 const, 7 enum, 8 fn, 7 struct, 1 trait
    - `fn verify_proof_sequence` ⚠ fremd: `contextra-types`
    - `fn compute_wal_delete_receipt`
    - `const DELETION_PROOF_KEY_PURPOSE`
    - `fn derive_deletion_proof_key_for_tenant`
    - `trait LayerWitnessExt` · Methoden: wal_seq_after, control_matches
    - `fn map_witness_layer_to_deletion_layer`
    - `struct GraphRepairAttestation`
    - `fn hash_deleted_keys_length_prefixed`
    - `enum CustodyLevel`
    - `enum SigningKeyHandle`
    - `enum VerificationKey`
    - `struct LayerCleanupProof` · 2 Methoden
    - `enum DeletionLayer`
    - `struct UncoveredLayer`
    - `fn serialize` *(re-exportiert)* ⚠ fremd: `contextra-ports`
    - `fn deserialize` *(re-exportiert)* ⚠ fremd: `contextra-ports`
    - `struct LayerWitnessMeasurement` ⚠ fremd: `contextra-ports`
    - `enum ExcludedScope`
    - `enum DeletionScope`
    - `struct DeletionProof`
    - `fn verify_wal_delete_receipt`
    - `enum VerificationErrorReason` ⚠ fremd: `contextra-types`
    - `struct VerificationResult` · 2 Methoden
    - `struct DeletionProofVerifier` · 3 Methoden
- `ed25519_proof`
  - 2 enum, 2 fn, 1 struct
    - `enum DeletionProofError`
    - `struct DeletionProofKeyPair` · 6 Methoden
    - `fn sign_deletion_proof_v3`
    - `fn verify_deletion_proof_v3`
    - `enum SignatureVersion` · 1 Methoden
- `error`
  - 1 enum, 1 type
    - `type Result`
    - `enum CryptoError` · 1 Methoden ⚠ fremd: `contextra-types`
- `kdf`
  - 10 const, 2 fn, 3 struct
    - `const KDF_HEADER_MAGIC`
    - `const KDF_HEADER_VERSION_1`
    - `const KDF_ID_ARGON2ID`
    - `const MIN_SALT_LEN`
    - `const MIN_M_COST_KIB`
    - `const MIN_T_COST`
    - `const MIN_P_COST`
    - `const DEFAULT_M_COST_KIB`
    - `const DEFAULT_T_COST`
    - `const DEFAULT_P_COST`
    - `struct KdfParams` · 1 Methoden
    - `struct KdfHeader` · 4 Methoden
    - `struct DerivedKey`
    - `fn derive_key_argon2id`
    - `fn hkdf_expand`
- `kv_cipher`
  - 1 const, 2 struct, 1 trait
    - `const CURRENT_KV_FORMAT_VERSION`
    - `struct EncryptedKvLayer` ⚠ fremd: `contextra-types`
    - `trait KvCipher` · Methoden: seal, open
    - `struct KvSegmentCipher` · 6 Methoden
- `kv_segment`
  - 1 const, 1 enum, 1 fn, 7 struct, 1 type
    - `enum KvDeleteMode`
    - `struct EvictionWorker` · 3 Methoden
- `kv_shredding`
  - 1 const, 1 fn, 5 struct
- `revocation_log`
  - 1 enum, 1 fn, 2 struct
- `secret`
  - 2 struct
- `wal_completeness`
  - 1 fn
- `wal_crypto`
  - 6 struct, 1 trait
  - … 30 weitere Items nicht aufgelistet (`--max-items`)
- Zusätzlich 9 `pub`-Items in privaten Modulen ohne Re-Export (nicht Teil der echten API).
- Interne Re-Exporte (`pub use`): `DeletionProofKeyPair`, `hash_deleted_keys_length_prefixed`
- ⚠ Re-Exporte **fremder Crates**: `contextra_types::ModelFingerprint`

**Eingehend (wer nutzt dieses Crate)**

| Nutzer | Symbole prod | Symbole dev |
|---|---|---|
| `contextra-kvcache` | `KvCipher`×2, `KvSegmentCipher`×2, `CryptoError`×2, `CryptoKey`, `ModelFingerprint` | – |
| `contextra-store` | `wal_crypto`×2, `verify_wal_chain_completeness`, `wal_crypto::WalHmac`, `crypto::KeyManager` | – |
| `contextra-mcp` | `CryptoKey`×3, `CryptoKey::try_new` | `AuditChain` |
| `contextra-infer-candle` | `EncryptedKvLayer` | `KvSegment` |

**Nach außen ungenutzte öffentliche Items (Heuristik, 101 von 110)**: `REC_AAD`, `seal_record_with_nonce`, `seal_record_with_rng`, `seal_record`, `open_record`, `VolatileEncryptionKey`, `constant_time_compare`, `verify_tag_constant_time`, `Blake3Chain`, `DataClass`, `AuditChainEntry`, `AuditChainHeadSignature`, `AuditHeadAnchor`, `EncryptedCommitmentSalt`, `compute_record_commitment`, `UnitKey`, `Dek`, `Salt`, `DocTag`, `AncestorSalts`, `DEK_LABEL`, `DEK_SALT_LABEL`, `WRAP_AAD`, `derive_dek`, `wrap_dek`, `unwrap_dek`, `verify_proof_sequence`, `compute_wal_delete_receipt`, `DELETION_PROOF_KEY_PURPOSE`, `derive_deletion_proof_key_for_tenant` …

### contextra-db

- Pfad: `crates/contextra-db` · Zeilen: 12245 · Rust-Dateien: 60 · Ring: 3
- Interne Cargo-Abhängigkeiten: `contextra-adapt` (dev), `contextra-checkpoint` (dev), `contextra-graph` (dev), `contextra-store` (dev), `contextra-testkit` (dev), `contextra-text` (dev), `contextra-vector` (dev), `contextra-engine` (normal), `contextra-ports` (normal), `contextra-types` (normal)

**Ausgehend (Importe aus anderen Workspace-Crates)**

| Ziel | prod | dev | Dateien (prod) | Symbole (prod) | Symbole (nur dev) |
|---|---:|---:|---:|---|---|
| `contextra-engine` | 6 | 0 | 1 | `(crate)`, `DurabilityMode`, `SandboxBridge`, `decay_controller::*`, `fusion`, `collection` | – |
| `contextra-types` | 4 | 46 | 2 | `DistanceMetric`, `MAX_SEARCH_K`, `EntityId`, `EntityId::from_key` | `Result`×6, `TxId`×6, `DocId`×4, `FusionWeights`×3, `TxId::new`×3 … (+15) |
| `contextra-ports` | 1 | 12 | 1 | `QueryRewriter` | `GraphIndex`×4, `StorageEngine`×2, `BoxFuture`×2, `LlmTextGenerator`, `EmbeddingError` … (+2) |
| `contextra-adapt` | 0 | 1 | 0 | – | `PidController` |
| `contextra-checkpoint` | 0 | 4 | 0 | – | `PersistentCheckpointStore`×2, `clear_all_orphaned_checkpoints`, `get_orphaned_checkpoints` |
| `contextra-graph` | 0 | 15 | 0 | – | `CsrGraph`×10, `path_rag::PathRAGEngine`×2, `csr::CsrGraph`×2, `PathRAGEngine` |
| `contextra-rank` | 0 | 1 | 0 | – | `IsotonicCalibrator::new` |
| `contextra-store` | 0 | 12 | 0 | – | `LsmConfig`×5, `LsmStorage::new`×4, `LsmStorage`×2, `lsm` |
| `contextra-text` | 0 | 11 | 0 | – | `Language::English`×7, `Language`×2, `bm25::score_term`, `inverted::InvertedIndex` |
| `contextra-vector` | 0 | 6 | 0 | – | `HnswConfig`×3, `HnswIndex`×2, `HnswIndex::try_new` |

**Pro Komponente**

- `(crate-root)` (796 Zeilen)
  - → `contextra-engine`: prod: `(crate)`, `DurabilityMode`, `SandboxBridge`, `decay_controller::*`, `fusion`, `collection`
  - → `contextra-rank`: dev: `IsotonicCalibrator::new`
  - → `contextra-types`: prod: `DistanceMetric` · dev: `TenantId`, `Result`, `ContextraError::InvalidInput`
- `multistep` (403 Zeilen)
  - → `contextra-ports`: prod: `QueryRewriter`
  - → `contextra-types`: prod: `MAX_SEARCH_K`, `EntityId`, `EntityId::from_key`
- `examples/collections.rs` (114 Zeilen)
  - → `contextra-types`: dev: `Result`, `TenantId::new`
- `examples/db_standalone.rs` (36 Zeilen)
  - → `contextra-types`: dev: `Result`, `ContextraError::Io`
- `examples/hybrid_search.rs` (97 Zeilen)
  - → `contextra-types`: dev: `Result`
- `examples/quickstart.rs` (65 Zeilen)
  - → `contextra-types`: dev: `Result`
- `tests/atomic_commit.rs` (100 Zeilen)
  - → `contextra-types`: dev: `EntityId`
- `tests/auto_community_detection_test.rs` (127 Zeilen)
  - → `contextra-types`: dev: `EntityId`
- `tests/cascade_invalidation_integration.rs` (139 Zeilen)
  - → `contextra-graph`: dev: `path_rag::PathRAGEngine`
  - → `contextra-ports`: dev: `GraphIndex`
- `tests/cascade_invalidation_test.rs` (116 Zeilen)
  - → `contextra-graph`: dev: `path_rag::PathRAGEngine`
  - → `contextra-ports`: dev: `GraphIndex`
  - → `contextra-types`: dev: `LinkRelation`
- `tests/checkpoint_layer_bounds.rs` (137 Zeilen)
  - → `contextra-checkpoint`: dev: `PersistentCheckpointStore`
  - → `contextra-store`: dev: `LsmConfig`×2, `LsmStorage::new`×2
  - → `contextra-types`: dev: `TxId`
- `tests/checkpoint_stress.rs` (78 Zeilen)
  - → `contextra-checkpoint`: dev: `PersistentCheckpointStore`
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-types`: dev: `TxId`
- `tests/compound_split_recall_impact.rs` (402 Zeilen)
  - → `contextra-types`: dev: `FusionWeights`
- `tests/compound_split_recall_impact_test.rs` (456 Zeilen)
  - → `contextra-types`: dev: `FusionWeights`
- `tests/consolidation_double_trigger_test.rs` (143 Zeilen)
  - → `contextra-graph`: dev: `CsrGraph`
  - → `contextra-store`: dev: `LsmStorage`, `LsmConfig`
  - → `contextra-text`: dev: `Language::English`
  - → `contextra-vector`: dev: `HnswIndex`, `HnswConfig`
- `tests/consolidation_integration_test.rs` (236 Zeilen)
  - → `contextra-ports`: dev: `BoxFuture`, `LlmTextGenerator`
  - → `contextra-types`: dev: `DocId`, `Result`
- `tests/context_compaction_retry_test.rs` (207 Zeilen)
  - → `contextra-store`: dev: `LsmConfig`, `LsmStorage::new`
  - → `contextra-types`: dev: `TxId::new`×2
- `tests/cross_domain_chaos_matrix_test.rs` (281 Zeilen)
  - → `contextra-graph`: dev: `csr::CsrGraph`
  - → `contextra-ports`: dev: `EmbeddingError`
  - → `contextra-store`: dev: `lsm`
  - → `contextra-text`: dev: `Language`
  - → `contextra-types`: dev: `DocId`
- `tests/diskann_collection_fallback.rs` (97 Zeilen)
  - → `contextra-graph`: dev: `csr::CsrGraph`
  - → `contextra-ports`: dev: `VectorIndex`
  - → `contextra-store`: dev: `LsmStorage`, `LsmConfig`
  - → `contextra-text`: dev: `Language`
  - → `contextra-types`: dev: `TxId::new`
- `tests/docid_128_consolidation.rs` (42 Zeilen)
  - → `contextra-types`: dev: `DocId`, `TxId`
- `tests/embedding_dimension_switch.rs` (91 Zeilen)
  - → `contextra-types`: dev: `ContextraError`
- `tests/fault_injection_2pc.rs` (783 Zeilen)
  - → `contextra-graph`: dev: `CsrGraph`
  - → `contextra-text`: dev: `Language::English`
- `tests/graph_entity_delete_test.rs` (161 Zeilen)
  - → `contextra-graph`: dev: `CsrGraph`
  - → `contextra-ports`: dev: `GraphIndex`
  - → `contextra-text`: dev: `Language::English`
  - → `contextra-vector`: dev: `HnswIndex`, `HnswConfig`
- `tests/graph_hybrid_search.rs` (299 Zeilen)
  - → `contextra-ports`: dev: `GraphIndex`
  - → `contextra-types`: dev: `ContextraError::SnapshotUnsupportedForSignal`×2, `GraphTraversalStrategy`, `EntityId::from_key`, `PprAlgorithm::Auto` … (+1)
- `tests/hnsw_delete_and_backfill.rs` (223 Zeilen)
  - → `contextra-graph`: dev: `CsrGraph`
  - → `contextra-text`: dev: `Language::English`
- `tests/instance_orphan_registry_test.rs` (133 Zeilen)
  - → `contextra-checkpoint`: dev: `clear_all_orphaned_checkpoints`, `get_orphaned_checkpoints`
  - → `contextra-types`: dev: `TxId`
- `tests/integration_temporal_filter.rs` (206 Zeilen)
  - → `contextra-graph`: dev: `CsrGraph`
  - → `contextra-types`: dev: `TxId`
- `tests/multistep_query_rewriter_modalities.rs` (204 Zeilen)
  - → `contextra-graph`: dev: `CsrGraph`
  - → `contextra-text`: dev: `Language::English`
  - → `contextra-types`: dev: `ContextraError::Internal`
- `tests/multistep_retrieval_tests.rs` (299 Zeilen)
  - → `contextra-graph`: dev: `CsrGraph`
  - → `contextra-ports`: dev: `BoxFuture`
  - → `contextra-text`: dev: `Language::English`
  - → `contextra-types`: dev: `ContextraError::Internal`
- `tests/pid_pool_production_caller.rs` (122 Zeilen)
  - → `contextra-adapt`: dev: `PidController`
  - → `contextra-graph`: dev: `CsrGraph`
  - → `contextra-ports`: dev: `Clock`
  - → `contextra-text`: dev: `Language::English`
- `tests/proptest_search_invariants.rs` (211 Zeilen)
  - → `contextra-text`: dev: `bm25::score_term`
- `tests/search_result_bound.rs` (221 Zeilen)
  - → `contextra-graph`: dev: `CsrGraph`
  - → `contextra-types`: dev: `DistanceMetric`
- `tests/snapshot_api.rs` (165 Zeilen)
  - → `contextra-types`: dev: `ContextraError::CapabilityUnsupported`×2, `EntityId::new`
  - → `contextra-vector`: dev: `HnswIndex::try_new`, `HnswConfig`
- `tests/split_brain_vector_orphan_test.rs` (515 Zeilen)
  - → `contextra-types`: dev: `DocId`
- `tests/sprint2_acid_tests.rs` (349 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-store`: dev: `LsmStorage::new`
  - → `contextra-types`: dev: `TenantId`, `TxId`
- `tests/stabilization_sprint_tests.rs` (145 Zeilen)
  - → `contextra-types`: dev: `FusionWeights`
- `tests/supersedes_cascading_tombstone_test.rs` (171 Zeilen)
  - → `contextra-graph`: dev: `PathRAGEngine`
- `tests/text_integration.rs` (102 Zeilen)
  - → `contextra-text`: dev: `inverted::InvertedIndex`
- `tests/truncation_filter_recall_test.rs` (284 Zeilen)
  - → `contextra-graph`: dev: `CsrGraph`
  - → `contextra-types`: dev: `MemoryType`
- `tests/zettelkasten_links_test.rs` (276 Zeilen)
  - → `contextra-types`: dev: `HybridQuery`, `FusionWeights::new`

**Öffentliche Schnittstelle** (erreichbar über `pub mod`-Kette oder `pub use`)

- `homeostat`
  - 2 struct
    - `struct RerankDeadline` · 3 Methoden
    - `struct RerankPidController` · 3 Methoden
- `multistep`
  - 2 const, 3 struct
    - `const DEFAULT_MAX_SUB_QUERIES`
    - `const DEFAULT_RERANK_POOL_MULTIPLIER`
    - `struct MultiStepConfig`
    - `struct MultiStepResult`
    - `struct MultiStepEngine` · 4 Methoden
- `pid_latency_controller`
  - 3 const, 2 struct
    - `const DEFAULT_TARGET_LATENCY_MS`
    - `const MIN_SCALING_FACTOR`
    - `const MAX_SCALING_FACTOR`
    - `struct PidLatencyController` · 5 Methoden
    - `struct LatencyBudgetGuard` · 6 Methoden
- `volatile_vault`
  - 2 enum, 6 struct
    - `enum SignalModality`
    - `struct VaultChunk` · 3 Methoden
    - `struct VaultConfig`
    - `struct VolatileDrainReceipt`
    - `struct CommitReceipt`
    - `enum VaultError`
    - `struct VolatileContextVault` · 10 Methoden
    - `struct VaultChunkMetadata`
- Zusätzlich 7 `pub`-Items in privaten Modulen ohne Re-Export (nicht Teil der echten API).
- Interne Re-Exporte (`pub use`): `*`, `RerankDeadline`, `RerankPidController`
- ⚠ Re-Exporte **fremder Crates**: `contextra_engine`, `contextra_engine::DurabilityMode`, `contextra_engine::SandboxBridge`, `contextra_engine::decay_controller::*`, `contextra_engine::fusion`, `contextra_types::DistanceMetric`, `contextra_ports::QueryRewriter`

**Eingehend (wer nutzt dieses Crate)**

| Nutzer | Symbole prod | Symbole dev |
|---|---|---|
| `contextra` | `DurabilityMode`×3, `DurabilityMode::Full`×2, `DurabilityMode::WalNoHmac`×2, `DurabilityMode::MemoryOnly`×2, `QueryRewriter` | – |
| `contextra-agent` | `Collection`×2, `Contextra`×2, `volatile_vault` | `Contextra::open_with_config`×19, `ContextraConfig::default`×16, `volatile_vault::VaultConfig`×3, `ContextraConfig`×2 … (+1) |

**Nach außen ungenutzte öffentliche Items (Heuristik, 19 von 20)**: `RerankDeadline`, `RerankPidController`, `DEFAULT_MAX_SUB_QUERIES`, `DEFAULT_RERANK_POOL_MULTIPLIER`, `MultiStepConfig`, `MultiStepResult`, `MultiStepEngine`, `DEFAULT_TARGET_LATENCY_MS`, `MIN_SCALING_FACTOR`, `MAX_SCALING_FACTOR`, `PidLatencyController`, `LatencyBudgetGuard`, `SignalModality`, `VaultChunk`, `VolatileDrainReceipt`, `CommitReceipt`, `VaultError`, `VolatileContextVault`, `VaultChunkMetadata`

**Auffälligkeiten**

- contextra-testkit: kein Code-Zugriff gefunden (prüfen: Makro, Feature, cfg, oder entfernen)
- contextra-rank: genutzt, aber nicht in Cargo.toml deklariert (cfg-/Feature-Pfad oder Fehler)

### contextra-durable-fs

- Pfad: `crates/contextra-durable-fs` · Zeilen: 2750 · Rust-Dateien: 21 · Ring: 0
- Interne Cargo-Abhängigkeiten: keine

**Ausgehend (Importe aus anderen Workspace-Crates)**

Keine.

**Pro Komponente**

Keine Importe aus anderen Workspace-Crates.

**Öffentliche Schnittstelle** (erreichbar über `pub mod`-Kette oder `pub use`)

- `(crate-root)`
  - 2 enum, 12 fn, 3 struct
    - `enum FsClass` · 1 Methoden
    - `fn flush_file`
    - `fn flush_dir`
    - `fn fs_class`
    - `fn rename_durable`
    - `struct PoisonedHandle`
    - `struct DurableHandle` · 8 Methoden
    - `enum LockMode`
    - `struct DirLock` · 2 Methoden
    - `fn sync_dir`
    - `fn sync_dir_with_injector`
    - `fn atomic_replace`
    - `fn atomic_replace_with_injector`
    - `fn durable_remove`
    - `fn durable_remove_with_injector`
    - `fn scrub_and_remove`
    - `fn scrub_and_remove_with_injector`
- `error`
  - 1 enum, 2 type
    - `enum DurableFsError` · 1 Methoden
    - `type Error`
    - `type Result`
- `fault_injector`
  - 1 enum, 2 struct, 1 trait
    - `trait FaultInjector` · Methoden: on_write, on_fsync, on_rename, on_dir_fsync
    - `struct NoFault`
    - `enum InjectedFault`
    - `struct FaultInjectingVfs` · 6 Methoden
- `health`
  - 2 enum, 1 struct
    - `enum HealthState` · 4 Methoden
    - `enum HealthEvent`
    - `struct AtomicHealthState` · 3 Methoden
- `journal`
  - 2 const, 2 enum, 1 fn, 3 struct
    - `enum JournalSyncPolicy`
    - `enum RecordRecoveryPolicy`
    - `const JOURNAL_MAGIC`
    - `const RECORD_HEADER_SIZE`
    - `fn calculate_crc32`
    - `struct JournalRecord`
    - `struct AppendOnlyJournal` · 8 Methoden
    - `struct PendingDeleteJournal` · 6 Methoden
- `platform`
  - 1 enum, 2 fn
    - `enum FlushKind`
    - `fn sync_file`
    - `fn sync_dir_path`
- `scrub`
  - 3 enum, 3 fn, 2 struct
    - `struct ScrubReceipt`
    - `enum ScrubPolicy`
    - `enum ScrubRefusal`
    - `enum ScrubOutcome`
    - `struct ScrubProgress`
    - `fn scrub_file_with_rate_limit`
    - `fn scrub_if_safe`
    - `fn scrub_if_safe_with_injector`
- `vfs`
  - 1 struct, 1 trait, 1 type
    - `trait Vfs` · Methoden: read, write_atomic, append, sync_file, sync_dir, remove…
    - `struct StdVfs`
    - `type RealFs`

**Eingehend (wer nutzt dieses Crate)**

Wird von keinem anderen analysierten Workspace-Crate importiert.

**Nach außen ungenutzte öffentliche Items (Heuristik, 49 von 49)**: `DurableFsError`, `Error`, `Result`, `FaultInjector`, `NoFault`, `InjectedFault`, `FaultInjectingVfs`, `HealthState`, `HealthEvent`, `AtomicHealthState`, `JournalSyncPolicy`, `RecordRecoveryPolicy`, `JOURNAL_MAGIC`, `RECORD_HEADER_SIZE`, `calculate_crc32`, `JournalRecord`, `AppendOnlyJournal`, `PendingDeleteJournal`, `FsClass`, `flush_file`, `flush_dir`, `fs_class`, `rename_durable`, `PoisonedHandle`, `DurableHandle`, `LockMode`, `DirLock`, `sync_dir`, `sync_dir_with_injector`, `atomic_replace` …

### contextra-engine

- Pfad: `crates/contextra-engine` · Zeilen: 26359 · Rust-Dateien: 127 · Ring: 3
- Interne Cargo-Abhängigkeiten: `contextra-testkit` (dev), `contextra-vector` (dev), `contextra-adapt` (normal/optional via adaptive-candidate-pool-sizing), `contextra-checkpoint` (normal), `contextra-graph` (normal), `contextra-infer-onnx` (normal/optional via onnx), `contextra-mvcc` (normal), `contextra-ports` (normal), `contextra-rank` (normal), `contextra-sandbox` (normal/optional via sandbox), `contextra-store` (normal), `contextra-text` (normal), `contextra-types` (normal), `contextra-vector` (normal)

**Ausgehend (Importe aus anderen Workspace-Crates)**

| Ziel | prod | dev | Dateien (prod) | Symbole (prod) | Symbole (nur dev) |
|---|---:|---:|---:|---|---|
| `contextra-types` | 236 | 179 | 35 | `ContextraError::invalid_input`×50, `Result`×18, `EntityId`×15, `HybridQuery`×13, `ContextraError::Internal`×11, `MAX_SEARCH_K`×11, `ContextraError::Serialization`×8, `ImportanceScore::new`×8, `domain::MemoryLink`×7, `EntityId::from_key`×7 … (+40) | `ContextraError::from`×10, `domain::LinkRelation::Elaborates`×2, `ContextraError::HnswConnectivityDegraded`, `FusionWeights::new`, `PprConfig::default` … (+7) |
| `contextra-ports` | 35 | 19 | 17 | `MetricsSink`×9, `StorageEngine`×5, `Clock`×3, `BoxFuture`×2, `VectorIndexStats`×2, `Reranker`×2, `DriftStatusProvider`×2, `VectorIndex`, `SystemClock::new`, `GraphIndex` … (+7) | – |
| `contextra-store` | 22 | 104 | 8 | `LsmStorage`×6, `SystemPressure`×3, `tenant_codec::TenantScopedStorage`×3, `tenant_codec::TenantScopedStorage::new`×3, `PressureLevel::Critical`×2, `PressureLevel::Normal`, `PressureLevel::Elevated`, `LsmConfig`, `lsm::DurabilityMode`, `MergeOperator` | `lsm`×10, `compaction`×2, `sstable`×2 |
| `contextra-graph` | 20 | 75 | 7 | `hyperedge::HyperEdgeId`×2, `percolation::PercolationConfig`×2, `CsrGraph::load_from_storage`×2, `hyperedge::HyperEdge`×2, `cascade_invalidate_edges_for_superseded_doc`, `percolation::compute_percolation_health`, `percolation::should_trigger_rebonding`, `percolation::find_rebonding_candidates`, `CsrGraph`, `hyperedge::RoleId::new` … (+6) | `csr::CsrGraph`×15, `csr`, `CsrGraph::new` |
| `contextra-rank` | 14 | 2 | 4 | `SearchResult`×8, `IsotonicCalibrator`×2, `fusion`, `fusion::fuse_search_results_with_strategy`, `fusion::fuse_search_results_with_signal_strategies`, `fusion::fuse_signals` | – |
| `contextra-vector` | 9 | 117 | 8 | `HnswIndex`×4, `hnsw::HnswIndex`×3, `acorn::FilteredIndex`, `HnswConfig` | `hnsw::HnswConfig::default`, `hnsw::HnswIndex::try_new`, `acorn`, `HnswIndex::try_new` |
| `contextra-checkpoint` | 4 | 0 | 3 | `InstanceOrphanRegistry`×2, `InstanceOrphanRegistry::new`, `(crate)` | – |
| `contextra-text` | 4 | 68 | 3 | `inverted::InvertedIndex`×2, `Language`×2 | `Language::English`×63, `Language::German`×4 |
| `contextra-adapt` | 2 | 1 | 1 | `PidController`×2 | `PidController::default` |
| `contextra-infer-onnx` | 2 | 0 | 1 | `ensure_onnx_model_download`, `OnnxEmbedder::load` | – |
| `contextra-mvcc` | 1 | 5 | 1 | `tx_buffer::TxBuffer` | `SnapshotRegistry`×2, `tx_buffer::IndexOp` |
| `contextra-testkit` | 0 | 1 | 0 | – | `InMemoryStorageEngine` |

**Pro Komponente**

- `(crate-root)` (451 Zeilen)
  - → `contextra-checkpoint`: prod: `(crate)`, `InstanceOrphanRegistry`
  - → `contextra-ports`: prod: `MetricsSink`×3, `license::LicenseGate`, `BoxFuture`, `StorageEngine`, `TextEmbeddingEngine`, `VectorDeleteMode`, `DriftStatusProvider`, `VectorIndexStats`, `StorageStats`
  - → `contextra-rank`: prod: `IsotonicCalibrator`
  - → `contextra-store`: prod: `lsm::DurabilityMode`, `LsmStorage`, `tenant_codec::TenantScopedStorage`
  - → `contextra-text`: prod: `Language`
  - → `contextra-types`: prod: `Result`×6, `DistanceMetric`×2, `ContextraError::Serialization`×2, `ContextraError::Internal`×2, `TenantId`, `DistanceMetric::Cosine`
  - → `contextra-vector`: prod: `HnswIndex`
- `adapters` (61 Zeilen)
  - → `contextra-ports`: prod: `BoxFuture`
  - → `contextra-text`: prod: `inverted::InvertedIndex`
  - → `contextra-types`: prod: `Result`, `TenantId`
- `backends` (28 Zeilen)
  - → `contextra-store`: prod: `LsmStorage`
  - → `contextra-vector`: prod: `HnswIndex`
- `background_workers` (1157 Zeilen)
  - → `contextra-graph`: prod: `hyperedge::HyperEdgeId` · dev: `CsrGraph`×8, `hyperedge::HyperEdgeId`
  - → `contextra-mvcc`: prod: `tx_buffer::TxBuffer` · dev: `tx_buffer::TxBuffer`×2, `tx_buffer::IndexOp`
  - → `contextra-ports`: prod: `StorageEngine`×2, `VectorIndex`, `Clock`, `SystemClock::new`
  - → `contextra-store`: dev: `LsmStorage`×8, `LsmConfig`×8
  - → `contextra-text`: dev: `Language::English`×8
  - → `contextra-types`: prod: `Result`×4 · dev: `Result`×2, `ContextraError::HnswConnectivityDegraded`
  - → `contextra-vector`: prod: `hnsw::HnswIndex`×3 · dev: `HnswIndex`×8, `HnswConfig`×8, `hnsw::HnswConfig::default`, `hnsw::HnswIndex::try_new`
- `collection` (11246 Zeilen)
  - → `contextra-adapt`: prod: `PidController`×2 · dev: `PidController::default`
  - → `contextra-graph`: prod: `percolation::PercolationConfig`×2, `cascade_invalidate_edges_for_superseded_doc`, `percolation::compute_percolation_health`, `percolation::should_trigger_rebonding`, `percolation::find_rebonding_candidates`, `CsrGraph`, `hyperedge::HyperEdgeId`, `hyperedge::RoleId::new`, `hyperedge::HyperEdgeId::new`, `hyperedge::RoleBinding` … (+4) · dev: `CsrGraph`×40, `csr::CsrGraph`×15, `csr`, `percolation::PercolationConfig`
  - → `contextra-ports`: prod: `VectorIndexStats`, `GraphIndex`, `Reranker`, `StorageEngine` · dev: `StorageStats`×5, `StorageEngine`×3, `TextEmbeddingEngine`, `BoxFuture`
  - → `contextra-store`: prod: `SystemPressure`×3, `PressureLevel::Critical`×2, `PressureLevel::Normal`, `PressureLevel::Elevated`, `LsmStorage` · dev: `LsmStorage`×36, `LsmConfig`×35, `lsm`×10
  - → `contextra-text`: prod: `inverted::InvertedIndex`, `Language` · dev: `Language::English`×50, `Language::German`×4, `Language`
  - → `contextra-types`: prod: `ContextraError::invalid_input`×37, `HybridQuery`×12, `MAX_SEARCH_K`×11, `EntityId`×11, `ContextraError::Internal`×9, `ImportanceScore::new`×8, `domain::MemoryLink`×7, `EntityId::from_key`×7, `Entity::new`×6, `DecayFunction::None`×6 … (+34) · dev: `Result`×25, `ContextraError::InvalidInput`×15, `ContextraError::from`×10, `MAX_SEARCH_K`×4 … (+27)
  - → `contextra-vector`: prod: `HnswIndex`, `acorn::FilteredIndex` · dev: `HnswIndex`×44, `HnswConfig`×44
- `contextra_impl` (1284 Zeilen)
  - → `contextra-checkpoint`: prod: `InstanceOrphanRegistry`, `InstanceOrphanRegistry::new`
  - → `contextra-graph`: prod: `CsrGraph::load_from_storage`×2
  - → `contextra-infer-onnx`: prod: `ensure_onnx_model_download`, `OnnxEmbedder::load`
  - → `contextra-ports`: prod: `MetricsSink`×3, `Reranker`, `license`, `NoopMetricsSink`, `DriftStatusProvider`
  - → `contextra-rank`: prod: `IsotonicCalibrator`
  - → `contextra-store`: prod: `LsmStorage`×3, `tenant_codec::TenantScopedStorage::new`×3, `tenant_codec::TenantScopedStorage`×2, `LsmConfig`
  - → `contextra-types`: prod: `ContextraError::invalid_input`×8, `EntityId`×4, `FusionWeights`×2, `ContextraError::PolicyViolation`×2, `ContextraError::Storage`×2, `ContextraError::Transaction`, `MemoryType`, `GraphTraversalStrategy`, `HybridQuery`, `Result` · dev: `Result`×3
  - → `contextra-vector`: prod: `HnswConfig`
- `decay_controller` (172 Zeilen)
  - → `contextra-ports`: prod: `Clock`
- `export` (343 Zeilen)
  - → `contextra-types`: prod: `TxId::new` · dev: `DocId::from_key`×4
- `extraction` (556 Zeilen)
  - → `contextra-ports`: prod: `LlmTextGenerator`
- `filter` (384 Zeilen)
  - → `contextra-types`: prod: `ContextraError::invalid_input`×2, `ContextraError` · dev: `ContextraError::InvalidInput`×2
- `fusion` (54 Zeilen)
  - → `contextra-rank`: prod: `fusion`, `fusion::fuse_search_results_with_strategy`, `fusion::fuse_search_results_with_signal_strategies`, `fusion::fuse_signals`
  - → `contextra-types`: prod: `FusionStrategy`, `SignalFusionStrategies`
- `import` (129 Zeilen)
  - → `contextra-types`: prod: `ContextraError::invalid_input`×3, `ContextraError::Serialization`×2, `Result`
- `merge_operator_adapter` (181 Zeilen)
  - → `contextra-store`: prod: `MergeOperator`
- `metrics_names` (54 Zeilen)
  - → `contextra-ports`: prod: `MetricsSink`×3
- `temporal_filter` (339 Zeilen)
  - → `contextra-ports`: prod: `Clock`
  - → `contextra-rank`: prod: `SearchResult`×8 · dev: `SearchResult`×2
  - → `contextra-types`: prod: `TxId`
- `transaction` (1502 Zeilen)
  - → `contextra-graph`: prod: `hyperedge::HyperEdge`×2 · dev: `CsrGraph`
  - → `contextra-ports`: prod: `StorageEngine`
  - → `contextra-store`: dev: `LsmStorage`, `LsmConfig`
  - → `contextra-text`: dev: `Language::English`
  - → `contextra-types`: dev: `DocId::new`×4, `DocId`×2, `EntityId`
  - → `contextra-vector`: prod: `HnswIndex` · dev: `HnswIndex`, `HnswConfig`
- `tests/auto_extraction_runtime_mode.rs` (236 Zeilen)
  - → `contextra-types`: dev: `Result`
- `tests/auto_extraction_wiring.rs` (292 Zeilen)
  - → `contextra-types`: dev: `EntityId::from_key`×5
- `tests/auto_triple_extraction.rs` (118 Zeilen)
  - → `contextra-types`: dev: `EntityId::from_key`×2
- `tests/chunker_golden_tests.rs` (618 Zeilen)
  - → `contextra-types`: dev: `DocId`
- `tests/chunker_hardening_tests.rs` (145 Zeilen)
  - → `contextra-types`: dev: `DocId`
- `tests/chunker_robustness.rs` (65 Zeilen)
  - → `contextra-types`: dev: `DocId`
- `tests/collection_tenant_isolation_veto_f10.rs` (26 Zeilen)
  - → `contextra-types`: dev: `DocId`
- `tests/commit_crash_consistency.rs` (296 Zeilen)
  - → `contextra-graph`: dev: `CsrGraph`
  - → `contextra-text`: dev: `Language::English`
  - → `contextra-vector`: dev: `HnswIndex`, `HnswConfig`
- `tests/crud_error_robustness.rs` (110 Zeilen)
  - → `contextra-types`: dev: `ContextraError`, `Result`
- `tests/decay_and_expiry.rs` (158 Zeilen)
  - → `contextra-ports`: dev: `Clock`
  - → `contextra-types`: dev: `Result`×2, `MemoryType`
- `tests/engine_commit_crash_points.rs` (225 Zeilen)
  - → `contextra-graph`: dev: `CsrGraph`
  - → `contextra-text`: dev: `Language::English`
  - → `contextra-vector`: dev: `HnswIndex`, `HnswConfig`
- `tests/engine_recovery_crash_windows.rs` (375 Zeilen)
  - → `contextra-graph`: dev: `CsrGraph`
  - → `contextra-text`: dev: `Language::English`
  - → `contextra-vector`: dev: `HnswIndex`, `HnswConfig`
- `tests/export_import_roundtrip.rs` (92 Zeilen)
  - → `contextra-types`: dev: `ContextraError`, `Result`
- `tests/filter_metadata_eval.rs` (106 Zeilen)
  - → `contextra-types`: dev: `FilterExpr`, `Result`
- `tests/hybrid_adaptive_overfetch.rs` (134 Zeilen)
  - → `contextra-graph`: dev: `CsrGraph`
- `tests/hybrid_pathrag_snapshot.rs` (112 Zeilen)
  - → `contextra-graph`: dev: `CsrGraph`
- `tests/hybrid_query_fusion.rs` (95 Zeilen)
  - → `contextra-types`: dev: `Result`
- `tests/hybrid_signal_failure_semantics.rs` (110 Zeilen)
  - → `contextra-graph`: dev: `CsrGraph`
- `tests/invariant_determinism.rs` (81 Zeilen)
  - → `contextra-types`: dev: `Result`×2
- `tests/invariant_durability.rs` (107 Zeilen)
  - → `contextra-types`: dev: `Result`
- `tests/invariant_isolation.rs` (129 Zeilen)
  - → `contextra-types`: dev: `Result`, `ContextraError::Internal`
- `tests/invariant_tenant_isolation.rs` (137 Zeilen)
  - → `contextra-types`: dev: `Result`×2
- `tests/metrics_instrumentation.rs` (203 Zeilen)
  - → `contextra-ports`: dev: `MetricsSink`
  - → `contextra-types`: dev: `Result`×5
- `tests/multi_index_transaction.rs` (106 Zeilen)
  - → `contextra-types`: dev: `Result`
- `tests/query_builder_scope_acorn.rs` (249 Zeilen)
  - → `contextra-graph`: dev: `CsrGraph`
  - → `contextra-vector`: dev: `acorn`
- `tests/query_builder_time_travel.rs` (82 Zeilen)
  - → `contextra-types`: dev: `Result`
- `tests/rule_based_extraction_determinism.rs` (176 Zeilen)
  - → `contextra-types`: dev: `Result`
- `tests/ssi_read_path_coverage.rs` (171 Zeilen)
  - → `contextra-graph`: dev: `CsrGraph::new`
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-testkit`: dev: `InMemoryStorageEngine`
  - → `contextra-text`: dev: `Language::English`
  - → `contextra-types`: dev: `domain::LinkRelation::Elaborates`, `DistanceMetric::Cosine`
  - → `contextra-vector`: dev: `HnswIndex::try_new`, `HnswConfig`
- `tests/tenant_collection_handle_sharing.rs` (70 Zeilen)
  - → `contextra-types`: dev: `Result`×2, `TenantId`
- `tests/tenant_index_persistence_reload.rs` (95 Zeilen)
  - → `contextra-types`: dev: `Result`×2, `TenantId`
- `tests/tenant_policy_required.rs` (59 Zeilen)
  - → `contextra-types`: dev: `Result`×2
- `tests/tenant_wrapper_coverage.rs` (217 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-types`: dev: `Result`
- `tests/tx_fault_injection.rs` (283 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`, `VectorIndex`
  - → `contextra-types`: dev: `Result`×7
- `tests/tx_recovery_and_erasure.rs` (295 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`, `VectorIndex`
  - → `contextra-types`: dev: `Result`×7
- `tests/wasm_merge_operator.rs` (364 Zeilen)
  - → `contextra-mvcc`: dev: `SnapshotRegistry`×2
  - → `contextra-store`: dev: `compaction`×2, `sstable`×2, `MergeOperator`
  - → `contextra-types`: dev: `ContextraError`
- `tests/write_skew_via_engine_api.rs` (207 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-types`: dev: `Result`×2, `ContextraError`

**Öffentliche Schnittstelle** (erreichbar über `pub mod`-Kette oder `pub use`)

- `(crate-root)`
  - 2 enum, 1 fn, 5 struct, 1 trait, 3 type
    - `type ConsolidationLauncher` ⚠ fremd: `contextra-store`
    - `fn register_consolidation_launcher` ⚠ fremd: `contextra-store`
    - `struct Document`
    - `struct ContextraStats` ⚠ fremd: `contextra-ports`
    - `type DbStats`
    - `struct CommunityDetectionConfig`
    - `enum EmbeddingBackend`
    - `enum TenantPolicy`
    - `struct ContextraConfig` · 3 Methoden ⚠ fremd: `contextra-ports`, `contextra-store`, `contextra-types`
    - `type TenantCollectionMap` ⚠ fremd: `contextra-store`, `contextra-types`
    - `struct Contextra` · 3 Methoden
    - `trait SandboxBridge` · Methoden: db_search, db_insert, db_get
- `adapters`
  - 1 fn, 1 struct, 1 trait
    - `trait OllamaBridge` · Methoden: generate
    - `fn to_canonical_tenant_id` ⚠ fremd: `contextra-types`
    - `struct Bm25Scorer` · 1 Methoden
- `backends`
  - 1 struct, 1 trait
    - `trait Backends`
    - `struct DefaultBackends`
- `background_workers`
  - 1 fn
    - `fn start_decay_cleanup_worker` *(re-exportiert)* ⚠ fremd: `contextra-ports`
- `chunker`
  - 1 fn, 2 struct
    - `struct ChunkerConfig` *(re-exportiert)*
    - `fn estimate_tokens` *(re-exportiert)*
    - `struct MarkdownChunker` · 3 Methoden
- `collection`
  - 4 const, 1 enum, 5 fn, 7 struct
    - `const MAX_SCAN_RESULTS` *(re-exportiert)*
    - `fn sorted_unique_shards`
    - `struct PercolationResult`
    - `struct CollectionConfig`
    - `struct StoredDocument`
    - `struct StoredDocumentMeta`
    - `fn parse_importance_score`
    - `fn compute_default_importance` ⚠ fremd: `contextra-types`
    - `fn ensure_importance_metadata`
    - `fn extract_effective_importance`
    - `struct Collection` · 34 Methoden ⚠ fremd: `contextra-store`, `contextra-vector`
    - `const METRIC_SEARCH_LATENCY_VECTOR`
    - `const METRIC_SEARCH_LATENCY_TEXT`
    - `const METRIC_SEARCH_LATENCY_GRAPH`
    - `struct ScopeConstraint` *(re-exportiert)* · 2 Methoden ⚠ fremd: `contextra-types`
    - `enum SearchStrategy` *(re-exportiert)* · 3 Methoden ⚠ fremd: `contextra-types`
    - `struct SignalWeights` *(re-exportiert)* · 1 Methoden
- `decay_controller`
  - 3 struct
    - `struct DecaySignalInputs`
    - `struct DecayControllerConfig`
    - `struct AdaptiveDecayController` · 11 Methoden
- `export`
  - 1 const, 4 struct
    - `const SCHEMA_VERSION_V1`
    - `struct ExportDocumentV1`
    - `struct ExportCollectionV1`
    - `struct ExportMemoryV1`
    - `struct ExportRelationV1`
- `extraction`
  - 2 const, 1 enum, 4 fn, 4 struct, 1 trait
    - `trait EntityExtractor` · Methoden: extract_triples
    - `struct OpenIeExtractor`
    - `fn create_extractor`
    - `fn extract_triples_for_mode`
    - `fn extract_triples_for_config`
    - `fn extract_triples` ⚠ fremd: `contextra-ports`
    - `const MAX_EXTRACTION_TEXT_LENGTH`
    - `const MAX_EXTRACTED_TRIPLES`
    - `struct RuleBasedExtractor` · 2 Methoden
    - `struct ExtractedTriple`
    - `struct EntityExtractionConfig`
    - `enum ExtractorMode`
- `filter`
  - 2 const, 2 enum, 2 fn
    - `const MAX_FILTER_DEPTH`
    - `const MAX_FILTER_NODES`
    - `fn validate_filter_expr_bounds`
    - `fn extract_memory_type`
    - `enum FilterOp`
    - `enum MetadataFilter` · 2 Methoden
- `fusion`
  - 3 fn
    - `fn fuse_search_results_with_strategy` ⚠ fremd: `contextra-types`
    - `fn fuse_search_results_with_signal_strategies` ⚠ fremd: `contextra-types`
    - `fn fuse_signals`
- `import`
  - 1 const, 1 struct
    - `struct ImportSummary`
    - `const MAX_IMPORT_COLLECTIONS`
- `merge_operator_adapter`
  - 1 struct
    - `struct WasmMergeOperator` · 2 Methoden
- `metrics_names`
  - 9 const, 3 fn
    - `const COMMIT_LATENCY`
    - `const REBUILD_DURATION`
    - `const BACKPRESSURE_LEVEL`
    - `const CHECKPOINT_DURATION`
    - `const SEARCH_LATENCY_VECTOR`
    - `const SEARCH_LATENCY_TEXT`
    - `const SEARCH_LATENCY_GRAPH`
    - `const INSERT_LATENCY`
    - `const REAPER_DOCS_REAPED`
    - `fn record_histogram_safe` ⚠ fremd: `contextra-ports`
- `orchestration`
  - 8 struct, 1 trait
- `temporal_filter`
  - 5 fn, 1 struct, 1 type
- `transaction`
  - 1 enum, 1 struct
  - … 20 weitere Items nicht aufgelistet (`--max-items`)
- Zusätzlich 42 `pub`-Items in privaten Modulen ohne Re-Export (nicht Teil der echten API).
- ⚠ Re-Exporte **fremder Crates**: `contextra_types::AutoExtractionMode`, `contextra_graph::percolation::PercolationConfig`, `contextra_ports::TextEmbeddingEngine`, `contextra_ports::VectorDeleteMode`, `contextra_store::lsm::DurabilityMode`, `contextra_checkpoint`, `contextra_text::Language`, `contextra_ports::DriftStatusProvider`, `contextra_types::DistanceMetric`

**Eingehend (wer nutzt dieses Crate)**

| Nutzer | Symbole prod | Symbole dev |
|---|---|---|
| `contextra-db` | `(crate)`, `DurabilityMode`, `SandboxBridge`, `decay_controller::*`, `fusion`, `collection` | – |
| `contextra-cognition` | `(crate)` | `Language::English`×5, `collection::Collection`×4, `collection::Collection::new`×2, `Language::German` |

*Hinweis: Mindestens ein Crate importiert per Glob (`*`); ungenutzte Items sind deshalb nicht bestimmbar.*

**Auffälligkeiten**

- contextra-sandbox: kein Code-Zugriff gefunden (prüfen: Makro, Feature, cfg, oder entfernen)

### contextra-erasure

- Pfad: `crates/contextra-erasure` · Zeilen: 20048 · Rust-Dateien: 97 · Ring: 3
- Interne Cargo-Abhängigkeiten: keine

**Ausgehend (Importe aus anderen Workspace-Crates)**

Keine.

**Pro Komponente**

Keine Importe aus anderen Workspace-Crates.

**Öffentliche Schnittstelle** (erreichbar über `pub mod`-Kette oder `pub use`)

- `backend`
  - 2 trait
    - `trait StorageBackend` · Methoden: purge, layer_id
    - `trait ErasureTarget` · Methoden: target_id, target_type
- `crypto_local`
  - 4 fn, 4 struct
    - `struct SecretKey` · 2 Methoden
    - `struct Dek`
    - `struct UnitKey`
    - `struct AncestorSalts`
    - `fn hkdf_expand`
    - `fn derive_dek`
    - `fn seal_record`
    - `fn open_record`
- `deletion_proof`
  - 7 enum, 4 fn, 8 struct, 1 trait, 2 type
    - `type CryptoError`
    - `type Result`
    - `trait LayerWitnessExt` · Methoden: wal_seq_after, control_matches
    - `fn map_witness_layer_to_deletion_layer`
    - `struct GraphRepairAttestation`
    - `fn hash_deleted_keys_length_prefixed`
    - `struct DeletionProofKeyPair` · 3 Methoden
    - `enum CustodyLevel`
    - `enum SigningKeyHandle`
    - `enum VerificationKey`
    - `struct LayerCleanupProof` · 3 Methoden
    - `enum DeletionLayer`
    - `struct UncoveredLayer`
    - `struct LayerWitnessMeasurement`
    - `enum ExcludedScope`
    - `enum DeletionScope`
    - `struct DeletionProof` · 12 Methoden
    - `fn compute_wal_delete_receipt`
    - `fn verify_wal_delete_receipt`
    - `enum VerificationErrorReason`
    - `struct VerificationResult` · 2 Methoden
    - `struct DeletionProofVerifier` · 2 Methoden
- `ed25519_proof`
  - 2 enum, 3 fn, 1 struct
    - `fn hash_deleted_keys_length_prefixed`
    - `enum DeletionProofError`
    - `struct DeletionProofKeyPair` · 5 Methoden
    - `fn sign_deletion_proof_v3`
    - `fn verify_deletion_proof_v3`
    - `enum SignatureVersion` · 1 Methoden
- `error`
  - 1 enum
    - `enum ErasureError`
- `gate`
  - 2 enum, 3 fn, 6 struct
    - `struct BatchGateRequest` · 2 Methoden
    - `struct BatchGateOutcome`
    - `struct GateBatchCoordinator` · 2 Methoden
    - `fn bind_global_erased_set`
    - `fn add_batch`
    - `fn evaluate_restricted_access`
    - `struct ErasedSet` · 19 Methoden
    - `enum GatePredicate`
    - `enum GateLogEntry`
    - `struct GateLog` · 9 Methoden
    - `struct GateGuard` · 3 Methoden
- `io`
  - 3 fn
    - `fn write_all_at`
    - `fn read_exact_at`
    - `fn zero_at`
- `journal`
  - 2 enum, 2 fn, 6 struct, 1 trait
    - `struct DurableJournal` · 5 Methoden
    - `struct AppendLog` · 6 Methoden
    - `trait EpochKeys` · Methoden: commit_key
    - `fn compute_commitment`
    - `struct Epoch` · 2 Methoden
    - `enum RecordCode` · 3 Methoden
    - `enum TargetType` · 2 Methoden
    - `struct JournalRecord` · 3 Methoden
    - `struct JournalEntry` · 3 Methoden
    - `fn reduce`
    - `struct ErasureJournal` · 5 Methoden
- `kv_shredding`
  - 1 const, 1 fn, 5 struct
    - `const DEFAULT_SHRED_KEY_GROUP_SIZE`
    - `struct SubKey`
    - `struct GroupKek`
    - `struct RecordDek`
    - `struct EncryptedRecordPayload`
    - `fn derive_subkey`
    - `struct KeyRegistry` · 20 Methoden
- `ledger`
  - 1 const, 1 fn, 5 struct
    - `struct LedgerEntry` · 2 Methoden
    - `const GENESIS_HASH`
    - `struct InMemoryLedger` · 8 Methoden
    - `struct FinalizeParams`
    - `struct FinalizeOutcome`
    - `fn finalize_erasure`
    - `struct InMemoryLedger` · 1 Methoden
- `measure`
  - 2 enum, 12 fn, 10 struct, 1 trait
    - `enum VerifiedVia` · 1 Methoden
    - `trait StructureDecoder` · Methoden: name, can_decode, decode_and_check
- `orchestrator`
  - 2 const, 1 enum, 2 fn, 3 struct, 1 trait
- `purge`
  - 1 enum, 4 fn, 4 struct
- `replace`
  - 1 enum, 1 struct
- `report`
  - 2 enum, 11 fn, 11 struct, 1 type
- `restore`
  - 1 enum, 1 fn, 3 struct
- `restricted`
  - 2 enum, 3 struct
- `revocation_log`
  - 1 enum, 1 fn, 2 struct, 1 type
- `saga`
  - 4 enum, 5 fn, 8 struct, 7 trait
- `scope`
  - 5 fn, 2 struct, 1 type
- `supersede`
  - 1 enum, 2 struct
- `types_local`
  - 23 enum, 1 fn, 44 struct, 16 trait, 1 type
- `vault`
  - 19 const, 3 enum, 20 fn, 12 struct, 1 trait, 1 type
  - … 259 weitere Items nicht aufgelistet (`--max-items`)
- Zusätzlich 11 `pub`-Items in privaten Modulen ohne Re-Export (nicht Teil der echten API).
- Interne Re-Exporte (`pub use`): `SagaState`, `SagaState`, `SagaEraseRequest`, `VerifiedVia`

**Eingehend (wer nutzt dieses Crate)**

Wird von keinem anderen analysierten Workspace-Crate importiert.

**Nach außen ungenutzte öffentliche Items (Heuristik, 339 von 339)**: `StorageBackend`, `ErasureTarget`, `SecretKey`, `Dek`, `UnitKey`, `AncestorSalts`, `hkdf_expand`, `derive_dek`, `seal_record`, `open_record`, `CryptoError`, `Result`, `LayerWitnessExt`, `map_witness_layer_to_deletion_layer`, `GraphRepairAttestation`, `hash_deleted_keys_length_prefixed`, `DeletionProofKeyPair`, `CustodyLevel`, `SigningKeyHandle`, `VerificationKey`, `LayerCleanupProof`, `DeletionLayer`, `UncoveredLayer`, `LayerWitnessMeasurement`, `ExcludedScope`, `DeletionScope`, `DeletionProof`, `compute_wal_delete_receipt`, `verify_wal_delete_receipt`, `VerificationErrorReason` …

### contextra-graph

- Pfad: `crates/contextra-graph` · Zeilen: 28673 · Rust-Dateien: 133 · Ring: 0
- Interne Cargo-Abhängigkeiten: `contextra-ports` (normal/optional via types-compat), `contextra-types` (normal/optional via types-compat)

**Ausgehend (Importe aus anderen Workspace-Crates)**

| Ziel | prod | dev | Dateien (prod) | Symbole (prod) | Symbole (nur dev) |
|---|---:|---:|---:|---|---|
| `contextra-types` | 46 | 60 | 28 | `EntityId`×18, `ContextraError::InvalidInput`×5, `Result`×5, `ContextraError`×3, `TxId`×3, `Edge`×2, `Edge::new`×2, `PprConfig`×2, `ContextraError::Internal`×2, `MemoryLink` … (+3) | `Entity::new`×10, `TlHfdParams::default`×4, `PprAlgorithm::Auto`×4, `Entity`×2, `TenantId::new`×2 |
| `contextra-ports` | 29 | 44 | 12 | `BoxFuture`×5, `GraphIndex`×4, `StorageEngine`×2, `graph::HyperEdgeId::new`×2, `GraphIndex::personalized_page_rank_at`, `GraphIndex::rollback_to_tx`, `graph::RoleBinding`, `graph::HyperEdgeId`, `graph::GraphMutationError`, `graph::GraphMutationError::LockAcquisitionTimeout` … (+10) | `Result`×3, `graph_index::GraphIndex` |

**Pro Komponente**

- `(crate-root)` (133 Zeilen)
  - → `contextra-ports`: prod: `GraphIndex`, `BoxFuture`
  - → `contextra-types`: prod: `TxId`, `Result`
- `apprh` (982 Zeilen)
  - → `contextra-types`: prod: `EntityId`×4, `ContextraError`×2, `ContextraError::InvalidInput`×2 · dev: `Entity`
- `cascade` (939 Zeilen)
  - → `contextra-ports`: dev: `GraphIndex`
  - → `contextra-types`: prod: `ContextraError::InvalidInput`×2 · dev: `Entity::new`×10
- `community` (1292 Zeilen)
  - → `contextra-ports`: dev: `GraphIndex`
- `csr` (6098 Zeilen)
  - → `contextra-ports`: prod: `BoxFuture`×4, `graph::HyperEdgeId::new`×2, `StorageEngine`, `GraphIndex::personalized_page_rank_at`, `GraphIndex::rollback_to_tx`, `graph::RoleBinding`, `graph::HyperEdgeId`, `graph::GraphMutationError`, `graph::GraphMutationError::LockAcquisitionTimeout`, `graph::GraphMutationError::RoleBindingInvalid` … (+7) · dev: `GraphIndex`×3
  - → `contextra-types`: prod: `EntityId`×5, `Edge`×2, `Edge::new`×2, `PprConfig`×2, `Result`×2, `TxId`×2, `MemoryLink`, `LinkRelation::Supersedes` · dev: `Result`
- `edge_reinforcement` (148 Zeilen)
  - → `contextra-types`: dev: `EntityId`
- `edge_reinforcement_buffer` (191 Zeilen)
  - → `contextra-ports`: dev: `GraphIndex`
  - → `contextra-types`: prod: `EntityId` · dev: `EntityId`
- `entity_extraction` (285 Zeilen)
  - → `contextra-types`: prod: `ContextraError::Internal`
- `hyperedge` (711 Zeilen)
  - → `contextra-ports`: prod: `graph`
- `hyperedge_suggest` (160 Zeilen)
  - → `contextra-ports`: prod: `LlmTextGenerator`
- `path_rag` (1418 Zeilen)
  - → `contextra-types`: prod: `EntityId`×2, `Result`×2, `DocId`
- `percolation` (285 Zeilen)
  - → `contextra-ports`: prod: `GraphIndex` · dev: `GraphIndex`
  - → `contextra-types`: prod: `EntityId`
- `ppr` (3586 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`×3, `GraphIndex`×2
  - → `contextra-types`: prod: `TlHfdParams`, `EntityId` · dev: `TlHfdParams::default`×4, `TlHfdParams`
- `ppr_stream` (145 Zeilen)
  - → `contextra-ports`: prod: `GraphIndex`
  - → `contextra-types`: prod: `ContextraError::Internal`
- `provenance` (165 Zeilen)
  - → `contextra-types`: dev: `EntityId`
- `purger` (104 Zeilen)
  - → `contextra-ports`: prod: `scanner` · dev: `GraphIndex`
  - → `contextra-types`: dev: `TenantId::new`×2
- `session_dag` (883 Zeilen)
  - → `contextra-ports`: prod: `StorageEngine`
- `tl_hfd` (748 Zeilen)
  - → `contextra-types`: prod: `EntityId`×4, `ContextraError`, `ContextraError::InvalidInput` · dev: `Entity`
- `benches/csr_traversal_bench.rs` (199 Zeilen)
  - → `contextra-ports`: dev: `GraphIndex`
  - → `contextra-types`: dev: `Result`×4, `Edge::new`×2
- `examples/isolated_graph_traversal.rs` (54 Zeilen)
  - → `contextra-ports`: dev: `GraphIndex`
- `tests/algorithm_test_doubles.rs` (55 Zeilen)
  - → `contextra-types`: dev: `EntityId`
- `tests/apprh_determinism_permutation.rs` (129 Zeilen)
  - → `contextra-types`: dev: `EntityId`
- `tests/apprh_gate_monitor.rs` (291 Zeilen)
  - → `contextra-types`: dev: `EntityId`
- `tests/apprh_selector_shadow_to_production.rs` (220 Zeilen)
  - → `contextra-types`: dev: `EntityId`
- `tests/apprh_tests.rs` (215 Zeilen)
  - → `contextra-types`: dev: `EntityId`
- `tests/budgeted_k_path_hub_entity.rs` (151 Zeilen)
  - → `contextra-types`: dev: `EntityId`
- `tests/community_hyperedges_included_flag.rs` (127 Zeilen)
  - → `contextra-ports`: dev: `GraphIndex`
- `tests/csr_benchmark.rs` (163 Zeilen)
  - → `contextra-ports`: dev: `GraphIndex`
- `tests/csr_complexity_bench.rs` (152 Zeilen)
  - → `contextra-ports`: dev: `GraphIndex`
- `tests/dangling_nodes_audit_test.rs` (245 Zeilen)
  - → `contextra-ports`: dev: `GraphIndex`
  - → `contextra-types`: dev: `PprAlgorithm::Auto`×3
- `tests/diffusion_shadow_gate_thresholds.rs` (87 Zeilen)
  - → `contextra-types`: dev: `EntityId`
- `tests/entity_extraction_edge_cases.rs` (127 Zeilen)
  - → `contextra-types`: dev: `DocId`
- `tests/entity_extraction_tests.rs` (81 Zeilen)
  - → `contextra-types`: dev: `DocId`
- `tests/golden_ports_type_compatibility.rs` (60 Zeilen)
  - → `contextra-types`: dev: `EntityId`
- `tests/graph_collection_mutation_trait_test.rs` (68 Zeilen)
  - → `contextra-ports`: dev: `graph`
- `tests/hub_node_benchmark.rs` (96 Zeilen)
  - → `contextra-ports`: dev: `GraphIndex`
  - → `contextra-types`: dev: `Edge::new`×2
- `tests/hyperedge_cascade_crash_recovery.rs` (151 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
- `tests/hyperedge_cascade_queue_persistence.rs` (69 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
- `tests/hyperedge_memory_budget.rs` (96 Zeilen)
  - → `contextra-types`: dev: `EntityId`
- `tests/hyperedge_payload_sharing.rs` (37 Zeilen)
  - → `contextra-types`: dev: `EntityId`
- `tests/hyperedge_persistence_survives_restart.rs` (100 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
- `tests/hyperedge_suggest.rs` (161 Zeilen)
  - → `contextra-ports`: dev: `Result`
- `tests/hyperedge_suggest_llm_test.rs` (117 Zeilen)
  - → `contextra-ports`: dev: `Result`
- `tests/hyperedge_suggest_tests.rs` (329 Zeilen)
  - → `contextra-ports`: dev: `Result`
- `tests/integration_graph.rs` (116 Zeilen)
  - → `contextra-ports`: dev: `GraphIndex`
- `tests/leiden_reference_validation.rs` (155 Zeilen)
  - → `contextra-ports`: dev: `GraphIndex`
- `tests/path_rag_at_snapshot.rs` (311 Zeilen)
  - → `contextra-ports`: dev: `GraphIndex`
- `tests/ppr_alloc_test.rs` (112 Zeilen)
  - → `contextra-types`: dev: `PprAlgorithm::Auto`
- `tests/ppr_candidate_stream.rs` (315 Zeilen)
  - → `contextra-ports`: dev: `BoxFuture`×9
  - → `contextra-types`: dev: `Result`×8
- `tests/ppr_reference_validation.rs` (265 Zeilen)
  - → `contextra-ports`: dev: `GraphIndex`
- `tests/ppr_snapshot_isolation.rs` (356 Zeilen)
  - → `contextra-ports`: dev: `GraphIndex`
- `tests/ppr_strategy_equivalence.rs` (97 Zeilen)
  - → `contextra-ports`: dev: `graph_index::GraphIndex`
- `tests/proptest_csr_invariants.rs` (334 Zeilen)
  - → `contextra-ports`: dev: `GraphIndex`
- `tests/proptest_tl_hfd_diffusion.rs` (276 Zeilen)
  - → `contextra-types`: dev: `EntityId`
- `tests/proptest_tl_hfd_lovasz.rs` (267 Zeilen)
  - → `contextra-types`: dev: `EntityId`
- `tests/provider_tests.rs` (115 Zeilen)
  - → `contextra-types`: dev: `EntityId`
- `tests/source_doc_ids_populated.rs` (154 Zeilen)
  - → `contextra-ports`: dev: `GraphIndex`
- `tests/star_expansion_equals_clique.rs` (48 Zeilen)
  - → `contextra-types`: dev: `EntityId`
- `tests/tlhfd_shadow_mode_parity.rs` (288 Zeilen)
  - → `contextra-types`: dev: `EntityId`

**Öffentliche Schnittstelle** (erreichbar über `pub mod`-Kette oder `pub use`)

- `(crate-root)`
  - 1 trait
    - `trait GraphIndexExt` · Methoden: remove_entity ⚠ fremd: `contextra-ports`
- `apprh`
  - 2 enum, 4 fn, 9 struct, 1 trait
    - `fn run_apprh_push` ⚠ fremd: `contextra-types`
    - `enum ApprhError`
    - `struct ApprhGateMonitorConfig` · 1 Methoden
    - `struct ApprhObservationRecord`
    - `struct ApprhGateMonitorSnapshot`
    - `struct ApprhGateMonitor` · 7 Methoden
    - `fn apprh_local` ⚠ fremd: `contextra-types`
    - `fn forward_push_apprh` ⚠ fremd: `contextra-types`
    - `struct ApprhParams` · 1 Methoden
    - `enum ApprhMode`
    - `struct ApprhSelectorConfig`
    - `struct ApprhSelector` · 8 Methoden
    - `trait ApprhFlipGate` · Methoden: should_flip
    - `struct DefaultApprhFlipGate`
    - `struct ApprhShadowComparison` ⚠ fremd: `contextra-types`
    - `fn shadow_compare_forward_push_vs_apprh` ⚠ fremd: `contextra-types`
- `arc_slice`
  - 1 struct
    - `struct ArcSlice` · 9 Methoden
- `cascade`
  - 3 const, 1 enum, 4 fn, 4 struct, 1 trait, 1 type
    - `const MAX_HYPEREDGE_CASCADE_FANOUT`
    - `const DEFAULT_HYPEREDGE_CASCADE_FANOUT_LIMIT`
    - `struct CascadeInvalidationReport`
    - `fn cascade_invalidate_edges_for_superseded_doc`
    - `const CASCADE_QUEUE_PREFIX`
    - `enum CascadeStatus`
    - `struct CascadeDeletionRecord`
    - `struct CascadeTicket`
    - `trait GraphGarbageCollection` · Methoden: sweep_orphans
    - `struct HyperedgeCascadeReport`
    - `type CascadeReport`
    - `fn topological_sort_hyperedge_closure`
    - `fn cascade_invalidate_hyperedges_for_superseded_doc`
    - `fn verify_doc_hyperedge_deletion_completeness`
- `community`
  - 1 fn, 4 struct
    - `struct CommunityDetectionConfig`
    - `struct VirtualHyperedgeNode` · 1 Methoden
    - `struct StarExpansionIterator` · 2 Methoden
    - `struct CommunityAssignment`
    - `fn detect_communities`
- `consistency_enforcement`
  - 4 struct, 1 trait, 1 type
    - `type EdgeId`
    - `struct ConflictPattern`
    - `struct EdgeAssertion` · 1 Methoden
    - `trait ContradictionDetector` · Methoden: conflicts
    - `struct ExactPredicateConflictDetector`
    - `struct ConsistencyEnforcer` · 9 Methoden
- `contract`
  - 1 enum
    - `enum Error` *(re-exportiert)*
- `csr`
  - 2 struct
    - `struct CsrGraph` *(re-exportiert)* · 16 Methoden
    - `struct MemoryEstimate` *(re-exportiert)* · 1 Methoden
- `edge_reinforcement`
  - 4 fn, 1 struct
    - `struct EdgeReinforcementConfig`
    - `fn compute_edge_weight`
    - `fn apply_cooccurrence_reinforcement`
    - `fn apply_traversal_reinforcement`
    - `fn apply_weight_normalization`
- `edge_reinforcement_buffer`
  - 3 struct
    - `struct CooccurrenceSignal` ⚠ fremd: `contextra-types`
    - `struct TraversalSignal` ⚠ fremd: `contextra-types`
    - `struct EdgeReinforcementBuffer` · 6 Methoden
- `entity_extraction`
  - 1 fn, 2 struct, 1 trait
    - `struct ExtractedRelation` · 1 Methoden
    - `trait EntityExtractor` · Methoden: extract
    - `struct RuleBasedEntityExtractor` · 1 Methoden
    - `fn ingest_extracted_relations`
- `error`
  - 2 enum, 1 type
    - `type Result`
    - `enum GraphError`
    - `enum GraphMutationError`
- `hyperedge`
  - 2 const, 2 fn, 4 struct
    - `const HYPEREDGE_PREFIX`
    - `const HYPEREDGE_BY_ENTITY_PREFIX`
    - `fn star_weight`
    - `struct HyperEdge` · 9 Methoden
    - `struct HyperEdgeView` · 2 Methoden
    - `struct RoleInterner` · 8 Methoden
    - `fn sort_dedup_entities`
    - `struct ConsolidationNodesGuard` · 4 Methoden
- `hyperedge_suggest`
  - 1 const, 1 enum, 2 fn, 2 struct
    - `const MAX_RELATE_PARTICIPANTS`
    - `struct HyperEdgeCandidate`
    - `struct ValidatedHyperEdgeCandidate`
    - `enum HyperedgeSuggestError`
    - `fn compute_co_occurrence_candidates`
    - `fn validate_candidates_with_llm` ⚠ fremd: `contextra-ports`
- `path_rag`
  - 1 const, 1 fn, 7 struct, 2 trait
    - `struct KPathConfig`
    - `struct KPathResult` ⚠ fremd: `contextra-types`
    - `trait KPathDiffusion` · Methoden: find_k_paths
    - `struct GraphPath` ⚠ fremd: `contextra-types`
    - `trait PathGraph` · Methoden: neighbors_with_weights, predecessors_with_weights, hyperedges_for_entity, get_hyperedge, hyperedge_participants
- `percolation`
  - 3 fn, 1 struct
- `ppr`
  - 3 fn, 7 struct, 1 trait
- `ppr_stream`
  - 1 const, 1 struct
- `provenance`
  - 2 struct
- `provider`
  - 2 fn, 1 struct, 1 trait
- `purger`
  - 1 fn, 1 struct
- `session_dag`
  - 2 const, 6 struct, 1 type
- `test_doubles`
  - 3 struct
- `tl_hfd`
  - 2 const, 1 enum, 5 fn, 4 struct, 1 trait
- `types`
  - 2 struct
  - … 58 weitere Items nicht aufgelistet (`--max-items`)
- Zusätzlich 35 `pub`-Items in privaten Modulen ohne Re-Export (nicht Teil der echten API).
- ⚠ Re-Exporte **fremder Crates**: `contextra_types::EntityId`

**Eingehend (wer nutzt dieses Crate)**

| Nutzer | Symbole prod | Symbole dev |
|---|---|---|
| `contextra-engine` | `hyperedge::HyperEdgeId`×2, `percolation::PercolationConfig`×2, `CsrGraph::load_from_storage`×2, `hyperedge::HyperEdge`×2, `cascade_invalidate_edges_for_superseded_doc`, `percolation::compute_percolation_health`, `percolation::should_trigger_rebonding`, `percolation::find_rebonding_candidates`, `CsrGraph`, `hyperedge::RoleId::new` … (+6) | `csr::CsrGraph`×15, `csr`, `CsrGraph::new` |
| `contextra-cognition` | `(crate)` | `CsrGraph`×6, `HyperEdgeId`×3, `csr::EdgeType`×3, `hyperedge`×3 |
| `contextra-agent` | – | `CsrGraph`×2 |
| `contextra-db` | – | `CsrGraph`×10, `path_rag::PathRAGEngine`×2, `csr::CsrGraph`×2, `PathRAGEngine` |

**Nach außen ungenutzte öffentliche Items (Heuristik, 129 von 138)**: `run_apprh_push`, `ApprhError`, `ApprhGateMonitorConfig`, `ApprhObservationRecord`, `ApprhGateMonitorSnapshot`, `ApprhGateMonitor`, `apprh_local`, `forward_push_apprh`, `ApprhParams`, `ApprhMode`, `ApprhSelectorConfig`, `ApprhSelector`, `ApprhFlipGate`, `DefaultApprhFlipGate`, `ApprhShadowComparison`, `shadow_compare_forward_push_vs_apprh`, `ArcSlice`, `MAX_HYPEREDGE_CASCADE_FANOUT`, `DEFAULT_HYPEREDGE_CASCADE_FANOUT_LIMIT`, `CascadeInvalidationReport`, `CASCADE_QUEUE_PREFIX`, `CascadeStatus`, `CascadeDeletionRecord`, `CascadeTicket`, `GraphGarbageCollection`, `HyperedgeCascadeReport`, `CascadeReport`, `topological_sort_hyperedge_closure`, `cascade_invalidate_hyperedges_for_superseded_doc`, `verify_doc_hyperedge_deletion_completeness` …

### contextra-infer-candle

- Pfad: `crates/contextra-infer-candle` · Zeilen: 7269 · Rust-Dateien: 36 · Ring: 2
- Interne Cargo-Abhängigkeiten: `contextra-crypto` (dev), `contextra-crypto` (normal/optional via kv-bridge), `contextra-ports` (normal), `contextra-rank` (normal), `contextra-types` (normal)

**Ausgehend (Importe aus anderen Workspace-Crates)**

| Ziel | prod | dev | Dateien (prod) | Symbole (prod) | Symbole (nur dev) |
|---|---:|---:|---:|---|---|
| `contextra-ports` | 16 | 15 | 5 | `kv::KvLayout`×6, `kv::RopeConfig`×6, `embedding::EmbeddingError`, `kv::KvPrefixStore`, `kv`, `kv::KvBlock` | `kv::PrefixKey`×4, `ContextSegment`×2, `GroundingValidator`×2, `EmbeddingProvider`×2, `BoxFuture` … (+2) |
| `contextra-types` | 6 | 9 | 5 | `ContextraError`×4, `DocId::from`, `ModelFingerprint` | `DocId`×3, `ContextChunk`×2, `Result`×2, `TenantId` |
| `contextra-crypto` | 1 | 1 | 1 | `EncryptedKvLayer` | `KvSegment` |
| `contextra-rank` | 1 | 0 | 1 | `IsotonicCalibrator` | – |

**Pro Komponente**

- `embedding_provider` (261 Zeilen)
  - → `contextra-ports`: prod: `embedding::EmbeddingError`
- `gasp` (590 Zeilen)
  - → `contextra-rank`: prod: `IsotonicCalibrator`
  - → `contextra-types`: prod: `DocId::from` · dev: `DocId`
- `inference` (2127 Zeilen)
  - → `contextra-ports`: prod: `kv::KvLayout`×6, `kv::RopeConfig`×6, `kv::KvPrefixStore`, `kv` · dev: `ContextSegment`, `GroundingValidator`
  - → `contextra-types`: prod: `ContextraError` · dev: `TenantId`, `ContextChunk`, `DocId`
- `kv_bridge` (601 Zeilen)
  - → `contextra-crypto`: prod: `EncryptedKvLayer` · dev: `KvSegment`
  - → `contextra-ports`: dev: `BoxFuture`
  - → `contextra-types`: dev: `Result`
- `kv_state` (282 Zeilen)
  - → `contextra-ports`: prod: `kv::KvBlock`
  - → `contextra-types`: prod: `ContextraError`
- `model` (545 Zeilen)
  - → `contextra-types`: prod: `ContextraError`
- `model_registry` (140 Zeilen)
  - → `contextra-types`: prod: `ContextraError`, `ModelFingerprint`
- `tests/embedding_normalization_proptest.rs` (114 Zeilen)
  - → `contextra-ports`: dev: `EmbeddingProvider`
- `tests/embedding_provider_conformance.rs` (126 Zeilen)
  - → `contextra-ports`: dev: `embedding::EmbeddingError`, `EmbeddingProvider`
  - → `contextra-types`: dev: `Result`
- `tests/gasp_mutant_test.rs` (88 Zeilen)
  - → `contextra-ports`: dev: `GroundingValidator`
- `tests/kv_bridge_and_stress_test.rs` (151 Zeilen)
  - → `contextra-ports`: dev: `ResponseGroundingValidator`
- `tests/kv_prefix_reuse_test.rs` (345 Zeilen)
  - → `contextra-ports`: dev: `kv::PrefixKey`×4, `ContextSegment`, `LlmTextGenerator`, `kv`
- `tests/kv_stage_b_test.rs` (138 Zeilen)
  - → `contextra-types`: dev: `ContextraError`
- `tests/proptest_candle.rs` (73 Zeilen)
  - → `contextra-types`: dev: `ContextChunk`, `DocId`

**Öffentliche Schnittstelle** (erreichbar über `pub mod`-Kette oder `pub use`)

- `attention_exporter`
  - 1 const, 1 struct
    - `const MAX_TRACKED_REQUESTS`
    - `struct CandleAttentionExporter` · 5 Methoden
- `embedding`
  - 1 const, 3 struct, 1 trait
    - `const DEFAULT_MAX_CONCURRENT_EMBEDDINGS`
    - `trait CandleEmbedInner` · Methoden: embed, dim
    - `struct CandleEmbedClient` · 4 Methoden
    - `struct BertEmbedModel` · 1 Methoden
    - `struct DefaultCandleEmbedModel`
- `embedding_provider`
  - 1 const, 1 fn
    - `const MAX_CANDLE_EMBED_BATCH_SIZE`
    - `fn verify_embedding_provider`
- `error`
  - 1 enum, 1 type
    - `enum Error`
    - `type Result`
- `fake`
  - 2 struct
    - `struct FakeEmbedder` · 1 Methoden
    - `struct FakeTextGenerator` · 1 Methoden
- `gasp`
  - 1 const, 2 struct
    - `const DEFAULT_GROUNDING_THRESHOLD`
    - `struct GaspConfig`
    - `struct GaspValidator` · 9 Methoden
- `gguf_loader`
  - 4 const, 2 fn, 1 struct
    - `const MAX_GGUF_TENSORS`
    - `const MAX_GGUF_METADATA_KVS`
    - `const MAX_GGUF_STRING_LEN`
    - `const MAX_GGUF_METADATA_DEPTH`
    - `struct GgufMetadata`
    - `fn parse_gguf_metadata`
    - `fn parse_gguf_metadata_with_checksum`
- `inference`
  - 1 const, 2 fn, 6 struct, 1 trait
    - `trait CandleModelInner` *(re-exportiert)* · Methoden: generate, generate_stream, kv_layout, generate_stream_with_prefix
    - `const DEFAULT_MAX_CONCURRENT_INFERENCES` *(re-exportiert)*
    - `struct CandleLlmClient` *(re-exportiert)* · 11 Methoden
    - `struct QuantizedLlamaModel` *(re-exportiert)* · 1 Methoden
    - `struct PrefixSeed` *(re-exportiert)*
    - `struct PrefixRun` *(re-exportiert)*
    - `struct KvPrefixContext` *(re-exportiert)*
    - `fn build_prefix_key` *(re-exportiert)*
    - `fn seed_from_hit` *(re-exportiert)*
    - `struct QuantizedLlamaModel` *(re-exportiert)* · 5 Methoden
- `kv_bridge`
  - 2 struct
    - `struct KvCacheKey` · 1 Methoden
    - `struct KvBridgeAdapter` · 8 Methoden
- `kv_state`
  - 2 struct
    - `struct LayerKv` · 1 Methoden
    - `struct KvState` · 8 Methoden
- `model`
  - 1 const, 3 struct
    - `const MAX_SEQ_LEN`
    - `struct QuantizedMatMul`
    - `struct LayerWeights`
    - `struct ModelWeights` · 6 Methoden
- `model_registry`
  - 1 enum, 1 fn
    - `enum CandleQuantization`
    - `fn compute_fingerprint` ⚠ fremd: `contextra-types`
- `traits`
  - 1 struct, 4 trait
    - `trait Embedder` · Methoden: embed, embed_batch, dimension
    - `struct GenerationConfig`
    - `trait TextGenerator` · Methoden: generate
    - `trait KvStateSink` · Methoden: store_segment
    - `trait KvStateSource` · Methoden: load_segment
- Zusätzlich 1 `pub`-Items in privaten Modulen ohne Re-Export (nicht Teil der echten API).
- ⚠ Re-Exporte **fremder Crates**: `contextra_types::ModelFingerprint`

**Eingehend (wer nutzt dieses Crate)**

| Nutzer | Symbole prod | Symbole dev |
|---|---|---|
| `contextra-mcp` | `model_registry::CandleQuantization::Q4KM`×2, `CandleEmbedClient::from_dir`, `CandleLlmClient::from_dir` | – |
| `contextra-bench` | `CandleEmbedClient::from_dir`, `model_registry::CandleQuantization::Q4KM` | – |
| `contextra` | `(crate)` | – |

**Nach außen ungenutzte öffentliche Items (Heuristik, 48 von 48)**: `MAX_TRACKED_REQUESTS`, `CandleAttentionExporter`, `DEFAULT_MAX_CONCURRENT_EMBEDDINGS`, `CandleEmbedInner`, `CandleEmbedClient`, `BertEmbedModel`, `DefaultCandleEmbedModel`, `MAX_CANDLE_EMBED_BATCH_SIZE`, `verify_embedding_provider`, `Error`, `Result`, `FakeEmbedder`, `FakeTextGenerator`, `DEFAULT_GROUNDING_THRESHOLD`, `GaspConfig`, `GaspValidator`, `MAX_GGUF_TENSORS`, `MAX_GGUF_METADATA_KVS`, `MAX_GGUF_STRING_LEN`, `MAX_GGUF_METADATA_DEPTH`, `GgufMetadata`, `parse_gguf_metadata`, `parse_gguf_metadata_with_checksum`, `CandleModelInner`, `DEFAULT_MAX_CONCURRENT_INFERENCES`, `CandleLlmClient`, `QuantizedLlamaModel`, `PrefixSeed`, `PrefixRun`, `KvPrefixContext` …

### contextra-infer-ollama

- Pfad: `crates/contextra-infer-ollama` · Zeilen: 6311 · Rust-Dateien: 24 · Ring: 2
- Interne Cargo-Abhängigkeiten: `contextra-ports` (normal/optional via ports-compat), `contextra-types` (normal/optional via ports-compat)

**Ausgehend (Importe aus anderen Workspace-Crates)**

| Ziel | prod | dev | Dateien (prod) | Symbole (prod) | Symbole (nur dev) |
|---|---:|---:|---:|---|---|
| `contextra-types` | 34 | 1 | 3 | `Result`×6, `ContextraError::Internal`×5, `ContextraError`×3, `ContextraError::Io`×3, `QueryRewriteOutput`×3, `ImportanceScore`×3, `ContextraError::InvalidInput`×2, `ContextraError::NotFound`×2, `ContextraError::Storage`×2, `ContextraError::PolicyViolation`×2 … (+3) | – |
| `contextra-ports` | 16 | 2 | 3 | `Rng`×2, `BoxFuture`×2, `LlmTextGenerator`, `LlmTextGeneratorStreaming`, `BoxStream`, `SegmentSynthesizer`, `EmbeddingError`, `EmbeddingError::Unavailable`, `EmbeddingError::ComputationFailed`, `SystemClock` … (+4) | `EmbeddingProvider` |

**Pro Komponente**

- `client` (3531 Zeilen)
  - → `contextra-ports`: prod: `Rng`×2, `BoxFuture`×2, `LlmTextGenerator`, `LlmTextGeneratorStreaming`, `BoxStream`, `SegmentSynthesizer` · dev: `EmbeddingProvider`, `SegmentSynthesizer`
  - → `contextra-types`: prod: `Result`×4, `ConfigFingerprint` · dev: `ConfigFingerprint`
- `error` (249 Zeilen)
  - → `contextra-ports`: prod: `EmbeddingError`, `EmbeddingError::Unavailable`, `EmbeddingError::ComputationFailed`, `SystemClock`, `Clock::now_unix_nanos`, `Clock::monotonic_nanos`, `SeededRng`, `Rng::next_u64`
  - → `contextra-types`: prod: `ContextraError::Internal`×4, `ContextraError`×3, `ContextraError::Io`×3, `QueryRewriteOutput`×3, `ImportanceScore`×3, `ContextraError::InvalidInput`×2, `ContextraError::NotFound`×2, `ContextraError::Storage`×2, `ContextraError::PolicyViolation`×2, `QueryRewriteOutput::new` … (+1)
- `query_rewriter` (315 Zeilen)
  - → `contextra-types`: prod: `Result`×2, `ContextraError::Internal`

**Öffentliche Schnittstelle** (erreichbar über `pub mod`-Kette oder `pub use`)

- `client`
  - 6 const, 8 fn, 6 struct, 2 trait
    - `struct CircuitBreaker` *(re-exportiert)* · 5 Methoden
    - `struct EndpointPolicy`
    - `const DEFAULT_BASE_URL`
    - `const DEFAULT_EMBED_MODEL`
    - `const MAX_RETRIES`
    - `const MAX_BATCH_SIZE`
    - `const MAX_TEXT_BYTES`
    - `const DEFAULT_MAX_RESPONSE_BYTES`
    - `struct OllamaConfig`
    - `fn is_transient_network_error` *(re-exportiert)*
    - `fn classify_reqwest_error` *(re-exportiert)*
    - `fn is_transient_error` *(re-exportiert)*
    - `trait EgressAuditSink` · Methoden: audit_egress, is_noop
    - `trait PayloadGate` · Methoden: check_payload
    - `struct PermissivePayloadGate`
    - `struct NoopEgressAuditSink`
    - `struct EndpointPolicy` · 2 Methoden
    - `fn xml_escape`
    - `fn build_rag_prompt`
    - `fn validate_text_length`
    - `fn validate_batch_size`
    - `fn validate_model_name`
- `context_prefixer`
  - 3 fn, 2 struct, 1 type
    - `struct ContextPrefixConfig`
    - `struct ContextPrefixEngine` · 5 Methoden
    - `type ContextPrefixer`
    - `fn compute_max_prefix_chars`
    - `fn truncate_chars`
    - `fn truncate_prefix`
- `embedding`
  - 1 struct
    - `struct OllamaEmbedder` *(re-exportiert)* · 7 Methoden
- `error`
  - 1 enum, 4 struct, 2 trait, 1 type
    - `enum OllamaError`
    - `type Result`
    - `trait LocalClock` · Methoden: now, monotonic_nanos
    - `struct SystemClock` ⚠ fremd: `contextra-ports`
    - `trait LocalRng` · Methoden: next_u64, next_f64
    - `struct LocalSeededRng` · 1 Methoden
    - `struct LocalQueryRewriteOutput` · 2 Methoden
    - `struct LocalImportanceScore` · 1 Methoden
- `importance`
  - 1 enum, 5 fn, 1 struct, 1 trait
    - `trait ImportanceCalibrator` · Methoden: calibrated_probability, record_outcome
    - `fn score_importance_batch`
    - `enum Confidence`
    - `struct ImportanceAssessment` · 4 Methoden
    - `fn score_importance`
    - `fn score_importance_with_calibrator`
    - `fn record_importance_outcome`
    - `fn parse_importance_score_response`
- `model_info`
  - 1 fn, 1 struct
    - `struct ModelInfo`
    - `fn known_dimension`
- `query_rewriter`
  - 3 fn, 2 struct
    - `struct OllamaQueryRewriterConfig`
    - `struct OllamaQueryRewriter` · 4 Methoden
    - `fn truncate_chars`
    - `fn build_rewrite_prompt_from_snippets`
    - `fn parse_rewrite_response`
- `traits`
  - 3 trait, 1 type
    - `type BoxFuture`
    - `trait Embedder` · Methoden: provider_name, embed, embedding_dim, embed_batch
    - `trait AsyncEmbedder` · Methoden: model_name, embed_async, embed_batch_async
    - `trait TextGenerator` · Methoden: generate
- Zusätzlich 5 `pub`-Items in privaten Modulen ohne Re-Export (nicht Teil der echten API).

**Eingehend (wer nutzt dieses Crate)**

| Nutzer | Symbole prod | Symbole dev |
|---|---|---|
| `contextra-mcp` | `OllamaEmbedder::new`, `OllamaConfig`, `OllamaClient::with_config` | – |

**Nach außen ungenutzte öffentliche Items (Heuristik, 55 von 56)**: `CircuitBreaker`, `EndpointPolicy`, `DEFAULT_BASE_URL`, `DEFAULT_EMBED_MODEL`, `MAX_RETRIES`, `MAX_BATCH_SIZE`, `MAX_TEXT_BYTES`, `DEFAULT_MAX_RESPONSE_BYTES`, `is_transient_network_error`, `classify_reqwest_error`, `is_transient_error`, `EgressAuditSink`, `PayloadGate`, `PermissivePayloadGate`, `NoopEgressAuditSink`, `EndpointPolicy`, `xml_escape`, `build_rag_prompt`, `validate_text_length`, `validate_batch_size`, `validate_model_name`, `ContextPrefixConfig`, `ContextPrefixEngine`, `ContextPrefixer`, `compute_max_prefix_chars`, `truncate_chars`, `truncate_prefix`, `OllamaEmbedder`, `OllamaError`, `Result` …

### contextra-infer-onnx

- Pfad: `crates/contextra-infer-onnx` · Zeilen: 966 · Rust-Dateien: 12 · Ring: 2
- Interne Cargo-Abhängigkeiten: keine

**Ausgehend (Importe aus anderen Workspace-Crates)**

Keine.

**Pro Komponente**

Keine Importe aus anderen Workspace-Crates.

**Öffentliche Schnittstelle** (erreichbar über `pub mod`-Kette oder `pub use`)

- `(crate-root)`
  - 1 fn
    - `fn default_model_cache_dir`
- `config`
  - 2 enum, 1 fn, 2 struct
    - `enum ModelHash` · 1 Methoden
    - `fn verify_path_safety`
    - `enum PoolingMode`
    - `struct EmbedderConfig` · 2 Methoden
    - `struct RerankerConfig` · 2 Methoden
- `embedder`
  - 1 struct
    - `struct OnnxEmbedder` · 2 Methoden
- `error`
  - 1 enum, 1 type
    - `enum Error`
    - `type Result`
- `fakes`
  - 2 struct
    - `struct FakeEmbedder` · 1 Methoden
    - `struct FakeReranker` · 1 Methoden
- `reranker`
  - 1 enum, 1 struct
    - `struct OnnxReranker` · 2 Methoden
    - `enum CrossEncoderReranker` · 2 Methoden
- `traits`
  - 2 trait
    - `trait Embedder` · Methoden: embed, embed_batch, dimension
    - `trait Reranker` · Methoden: score

**Eingehend (wer nutzt dieses Crate)**

| Nutzer | Symbole prod | Symbole dev |
|---|---|---|
| `contextra-mcp` | `OnnxEmbedder::from_path`, `default_model_cache_dir`, `OnnxEmbedder::load` | – |
| `contextra-engine` | `ensure_onnx_model_download`, `OnnxEmbedder::load` | – |

**Nach außen ungenutzte öffentliche Items (Heuristik, 14 von 15)**: `ModelHash`, `verify_path_safety`, `PoolingMode`, `EmbedderConfig`, `RerankerConfig`, `OnnxEmbedder`, `Error`, `Result`, `FakeEmbedder`, `FakeReranker`, `OnnxReranker`, `CrossEncoderReranker`, `Embedder`, `Reranker`

### contextra-ingest

- Pfad: `crates/contextra-ingest` · Zeilen: 26371 · Rust-Dateien: 122 · Ring: 4
- Interne Cargo-Abhängigkeiten: keine

**Ausgehend (Importe aus anderen Workspace-Crates)**

Keine.

**Pro Komponente**

Keine Importe aus anderen Workspace-Crates.

**Öffentliche Schnittstelle** (erreichbar über `pub mod`-Kette oder `pub use`)

- `canonical`
  - 1 fn, 1 struct
    - `struct CanonicalText`
    - `fn sanitize_text`
- `chunking`
  - 1 enum, 8 fn, 17 struct, 1 trait
    - `fn token_count`
    - `fn estimate_tokens`
    - `fn window_split`
    - `fn chunk_text_with_overlap`
    - `fn finalize`
    - `struct ChunkingConfig` · 1 Methoden
    - `struct ProseOptions`
    - `enum TabularMode`
    - `struct TabularOptions`
    - `struct PagedOptions`
    - `struct StructuredOptions`
    - `struct EmailOptions`
    - `struct EmailChunker` *(re-exportiert)*
    - `struct FallbackChunker`
    - `fn build_chunk_metadata`
    - `struct ChunkDraft`
    - `trait IrChunker` · Methoden: strategy, draft
    - `fn draft_document`
    - `fn draft_block_range`
    - `struct PagedChunker` *(re-exportiert)*
    - `struct ProseChunker` *(re-exportiert)*
    - `struct StructuredChunker` *(re-exportiert)*
    - `struct ProseChunker`
    - `struct TabularChunker`
    - `struct DocId` · 2 Methoden
    - `struct ContextChunkLink`
    - `struct ContextChunk`
- `detect`
  - 1 const, 1 enum, 2 fn, 2 struct
    - `const MAX_DETECT_BYTES`
    - `enum Confidence`
    - `struct DetectHints`
    - `struct Detection`
    - `fn detect_reader`
    - `fn detect`
- `error`
  - 2 enum
    - `enum LimitKind`
    - `enum IngestError`
- `extractor`
  - 3 struct, 1 trait
    - `trait DocumentExtractor` · Methoden: formats, extract
    - `struct ResourceBudget` · 7 Methoden
    - `struct NestGuard`
    - `struct ExtractContext` · 1 Methoden
- `format`
  - 2 enum
    - `enum FormatId` · 5 Methoden
    - `enum ChunkStrategy`
- `formats`
  - 2 enum, 45 fn, 22 struct
    - `struct DocumentParseResult`
    - `fn parse_document_xml`
    - `struct DocxExtractor` · 1 Methoden
    - `struct ExtractedNote`
    - `fn parse_footnotes_or_endnotes`
    - `fn parse_comments`
    - `fn parse_header_footer`
    - `struct NumMap` · 2 Methoden
    - `struct StyleInfo`
    - `struct StyleMap` · 2 Methoden
    - `fn match_heading_string`
    - `fn extract_eml`
    - `fn strip_html`
    - `fn extract_mbox`
    - `struct EmailExtractor` · 1 Methoden
    - `fn parse_content`
    - `fn parse_meta`
    - `struct OdfExtractor` · 1 Methoden
    - `fn parse_presentation`
    - `struct SafeCfbReader` · 7 Methoden
    - `fn extract_doc`
    - `struct OleExtractor` · 1 Methoden
    - `fn extract_msg`
    - `fn extract_ppt`
    - `struct PdfExtractor` · 1 Methoden
    - `fn clean_page_text`
    - `fn join_end_of_line_hyphens`
    - `fn reconstruct_paragraphs`
    - `fn normalize_line_digits`
    - `struct HeaderFooterAnalysis`
    - `fn analyze_headers_footers`
    - `struct PptxExtractor` · 1 Methoden
    - `struct SlideRef`
    - `fn parse_presentation_slides`
    - `struct SlideExtractionResult`
    - `fn parse_slide_content`
    - `fn extract_csv_tsv`
- `fuzz_support`
  - 1 fn
- `ir`
  - 4 enum, 7 struct
- `limits`
  - 2 struct
- `pipeline`
  - 1 fn, 3 struct
- `registry`
  - 1 struct
- `safe`
  - 1 enum, 4 fn, 7 struct
  - … 63 weitere Items nicht aufgelistet (`--max-items`)
- Zusätzlich 25 `pub`-Items in privaten Modulen ohne Re-Export (nicht Teil der echten API).
- Interne Re-Exporte (`pub use`): `EmailChunker`, `PagedChunker`, `ProseChunker`, `StructuredChunker`, `TabularChunker`

**Eingehend (wer nutzt dieses Crate)**

Wird von keinem anderen analysierten Workspace-Crate importiert.

**Nach außen ungenutzte öffentliche Items (Heuristik, 143 von 143)**: `CanonicalText`, `sanitize_text`, `token_count`, `estimate_tokens`, `window_split`, `chunk_text_with_overlap`, `finalize`, `ChunkingConfig`, `ProseOptions`, `TabularMode`, `TabularOptions`, `PagedOptions`, `StructuredOptions`, `EmailOptions`, `EmailChunker`, `FallbackChunker`, `build_chunk_metadata`, `ChunkDraft`, `IrChunker`, `draft_document`, `draft_block_range`, `PagedChunker`, `ProseChunker`, `StructuredChunker`, `ProseChunker`, `TabularChunker`, `DocId`, `ContextChunkLink`, `ContextChunk`, `MAX_DETECT_BYTES` …

### contextra-kvcache

- Pfad: `crates/contextra-kvcache` · Zeilen: 8578 · Rust-Dateien: 37 · Ring: 1
- Interne Cargo-Abhängigkeiten: `contextra-crypto` (normal/optional via crypto-compat,kv-encryption), `contextra-ports` (normal/optional via ports-compat), `contextra-types` (normal/optional via ports-compat)

**Ausgehend (Importe aus anderen Workspace-Crates)**

| Ziel | prod | dev | Dateien (prod) | Symbole (prod) | Symbole (nur dev) |
|---|---:|---:|---:|---|---|
| `contextra-ports` | 21 | 16 | 6 | `kv_bridge_port::KvBridgeStorage`×2, `scanner::PurgeStats`×2, `scanner::ScanSession`×2, `embedding::EmbeddingProvider`, `license::FeatureRing`, `storage::StorageEngine`, `BoxFuture`, `plugin`, `RequestId`, `kv` … (+8) | `kv::KvLayout`, `kv::RopeConfig`, `clock::SystemClock`, `scanner`, `kv::KvBlock` … (+3) |
| `contextra-types` | 18 | 17 | 5 | `TenantId`×6, `RequestContext`×2, `Result`×2, `TenantId::try_new`, `TenantId::SYSTEM`, `model_fingerprint::ModelFingerprint`, `model_fingerprint::ModelFingerprint::new`, `ContextraError`, `ContextraError::Internal`, `error::Result` … (+1) | – |
| `contextra-crypto` | 8 | 3 | 2 | `KvCipher`×2, `KvSegmentCipher`×2, `CryptoError`×2, `CryptoKey`, `ModelFingerprint` | – |

**Pro Komponente**

- `adapters` (206 Zeilen)
  - → `contextra-ports`: prod: `embedding::EmbeddingProvider`, `kv_bridge_port::KvBridgeStorage`, `license::FeatureRing`, `storage::StorageEngine`, `BoxFuture`, `plugin`
- `attention_score` (281 Zeilen)
  - → `contextra-ports`: prod: `RequestId`
  - → `contextra-types`: prod: `TenantId`
- `compat` (61 Zeilen)
  - → `contextra-types`: prod: `TenantId`×3, `TenantId::try_new`, `TenantId::SYSTEM`, `model_fingerprint::ModelFingerprint`, `model_fingerprint::ModelFingerprint::new`, `ContextraError`, `ContextraError::Internal`
- `eviction_worker` (283 Zeilen)
  - → `contextra-types`: prod: `TenantId` · dev: `TenantId`
- `prefix_store` (602 Zeilen)
  - → `contextra-ports`: prod: `kv` · dev: `kv::PrefixKey`, `kv::KvLayout`, `kv::RopeConfig`
  - → `contextra-types`: dev: `model_fingerprint::ModelFingerprint::new`
- `scratchpad_invalidation` (264 Zeilen)
  - → `contextra-ports`: prod: `kv::PrefixKey` · dev: `kv`
  - → `contextra-types`: prod: `error::Result`, `TenantId` · dev: `model_fingerprint::ModelFingerprint`
- `segment` (601 Zeilen)
  - → `contextra-crypto`: prod: `KvCipher`×2, `CryptoKey`
- `service` (524 Zeilen)
  - → `contextra-ports`: prod: `attention::AttentionExporter`, `kv_bridge_port::KvBridgeStorage`, `clock`
- `store` (1150 Zeilen)
  - → `contextra-crypto`: prod: `KvSegmentCipher`×2, `CryptoError`×2, `ModelFingerprint`
  - → `contextra-ports`: prod: `scanner::PurgeStats`×2, `scanner::ScanSession`×2, `scanner::Purger`, `scanner::ScanTarget`, `scanner::LayerScanner`, `scanner::WitnessLayer`, `scanner::WitnessLayer::KvCacheSegments` · dev: `clock::SystemClock`, `scanner`
  - → `contextra-types`: prod: `RequestContext`×2, `Result`×2, `error`
- `benches/kvcache_bench.rs` (49 Zeilen)
  - → `contextra-types`: dev: `TenantId`
- `examples/standalone_prefix_cache.rs` (38 Zeilen)
  - → `contextra-types`: dev: `TenantId`
- `tests/clm_pinned_base_preserved.rs` (102 Zeilen)
  - → `contextra-ports`: dev: `kv`
  - → `contextra-types`: dev: `model_fingerprint::ModelFingerprint`, `TenantId`
- `tests/content_hash_hit_rate_improvement.rs` (102 Zeilen)
  - → `contextra-types`: dev: `TenantId`
- `tests/decryption_negative_tests.rs` (95 Zeilen)
  - → `contextra-types`: dev: `TenantId`
- `tests/eviction_segment_mapping.rs` (131 Zeilen)
  - → `contextra-types`: dev: `TenantId`
- `tests/eviction_with_attention_exporter.rs` (90 Zeilen)
  - → `contextra-types`: dev: `TenantId`
- `tests/kivi_aead_order_enforced.rs` (115 Zeilen)
  - → `contextra-crypto`: dev: `KvCipher`×2, `CryptoKey`
- `tests/kivi_quantize_dequantize_roundtrip.rs` (161 Zeilen)
  - → `contextra-types`: dev: `ContextraError`
- `tests/prefix_isolation_test.rs` (29 Zeilen)
  - → `contextra-ports`: dev: `kv::KvBlock`
- `tests/prefix_store_port.rs` (206 Zeilen)
  - → `contextra-ports`: dev: `kv`
  - → `contextra-types`: dev: `model_fingerprint::ModelFingerprint`
- `tests/radix_differential_test.rs` (101 Zeilen)
  - → `contextra-types`: dev: `TenantId`
- `tests/scratchpad_invalidation_isolation.rs` (91 Zeilen)
  - → `contextra-ports`: dev: `kv`
  - → `contextra-types`: dev: `model_fingerprint::ModelFingerprint`, `TenantId`
- `tests/service_tests.rs` (346 Zeilen)
  - → `contextra-ports`: dev: `kv_bridge_port::KvBridgeStorage`, `license::FeatureRing`, `plugin::PluginManifest`, `BoxFuture` … (+2)
- `tests/store_default_attention_source.rs` (111 Zeilen)
  - → `contextra-types`: dev: `TenantId`

**Öffentliche Schnittstelle** (erreichbar über `pub mod`-Kette oder `pub use`)

- `adapters`
  - 4 struct
    - `struct StorageEngineKvBridge` · 1 Methoden
    - `struct EmbeddingProviderSemanticEmbedder` · 2 Methoden
    - `struct KvCachePluginConfig` · 1 Methoden ⚠ fremd: `contextra-ports`
    - `struct KvCachePluginManifest` · 1 Methoden
- `attention_score`
  - 3 fn, 1 struct, 1 trait
    - `trait AttentionScoreSource` · Methoden: importance_score, importance_score_for_tenant, register_segment, unregister_segment
    - `struct NullAttentionScoreSource`
    - `fn rank_for_eviction`
    - `fn rank_for_eviction_weighted`
    - `fn rank_for_eviction_weighted_for_tenant` ⚠ fremd: `contextra-types`
- `cipher`
  - 1 struct, 1 trait
    - `trait SegmentCipher` · Methoden: encrypt_segment, decrypt_segment
    - `struct PlaintextSegmentCipher`
- `compat`
  - 1 struct
    - `struct CryptoSegmentCipher`
- `error`
  - 1 enum, 1 type
    - `enum KvCacheError`
    - `type Result`
- `eviction_worker`
  - 1 fn, 1 struct
    - `struct EvictionWorker` · 8 Methoden
    - `fn emergency_wipe`
- `prefix_store`
  - 1 const, 2 struct
    - `const DEFAULT_BYTE_BUDGET_PER_TENANT`
    - `struct TenantPrefixKvStore` · 7 Methoden
    - `struct NamespacePrefixStore` · 8 Methoden
- `quantize_kivi`
  - 6 fn, 4 struct
    - `struct KiviQuantizeConfig`
    - `struct KvTensorView` · 3 Methoden
    - `struct KiviBlockMeta`
    - `struct KiviQuantizedBlock`
    - `fn kivi_quantize`
    - `fn kivi_dequantize`
    - `fn unpack_kivi_block`
    - `fn pack_kivi_block`
    - `fn compress_bytes`
    - `fn decompress_bytes`
- `radix`
  - 2 enum, 8 struct, 1 trait
    - `enum KvReusePolicy` · 1 Methoden
    - `struct KvBlockGuard` · 4 Methoden
    - `struct PrefixMatch`
    - `struct PrefixRadixTree` · 8 Methoden
    - `struct KvSegmentRef` · 2 Methoden
    - `enum KvLookupResult`
    - `trait SemanticEmbedder` · Methoden: embed
    - `struct SemanticCacheConfig` · 1 Methoden
    - `struct ContentAddressedKvStore` · 10 Methoden
    - `struct SharedSegmentRef` · 4 Methoden
    - `struct NamespaceRadixTree` · 8 Methoden
- `scratchpad_invalidation`
  - 4 struct, 1 trait
    - `struct ScratchpadCacheScope` · 1 Methoden ⚠ fremd: `contextra-types`
    - `trait ScratchpadInvalidator` · Methoden: invalidate_scope
    - `struct PrefixStoreScratchpadInvalidator` · 2 Methoden ⚠ fremd: `contextra-ports`
    - `struct InvalidationBudget`
    - `struct BoundedScratchpadInvalidator` · 3 Methoden ⚠ fremd: `contextra-ports`
- `segment`
  - 1 const, 1 enum, 4 struct, 1 type
    - `enum KvSegmentContent`
    - `const CURRENT_KV_KEY_DERIVATION_VERSION`
    - `struct ShreddableSegmentKey` · 6 Methoden
    - `struct Tier2EncryptedSegment` · 2 Methoden
    - `struct EncryptedSegmentPayload`
    - `struct KvSegment` · 12 Methoden
    - `type SegmentKey`
- `service`
  - 4 struct
    - `struct KvCacheStats`
    - `struct KvCacheConfig`
    - `struct KvCacheServiceBuilder` · 11 Methoden
    - `struct KvCacheService` · 8 Methoden
- `store`
  - 1 const, 3 struct, 1 type
    - `type SpillHandler`
    - `const MAX_PINNED_BYTES_PER_TENANT`
    - `struct TenantIsolatedKvStore` · 24 Methoden
    - `struct KvCachePruner` · 1 Methoden
    - `struct KvCacheScanner` · 1 Methoden
- `types`
  - 1 enum, 4 struct
    - `struct TenantId` · 3 Methoden
    - `struct StepId` · 2 Methoden
    - `struct RequestId` · 3 Methoden
    - `struct ModelFingerprint` · 3 Methoden
    - `enum CacheDirective`

**Eingehend (wer nutzt dieses Crate)**

Wird von keinem anderen analysierten Workspace-Crate importiert.

**Nach außen ungenutzte öffentliche Items (Heuristik, 66 von 66)**: `StorageEngineKvBridge`, `EmbeddingProviderSemanticEmbedder`, `KvCachePluginConfig`, `KvCachePluginManifest`, `AttentionScoreSource`, `NullAttentionScoreSource`, `rank_for_eviction`, `rank_for_eviction_weighted`, `rank_for_eviction_weighted_for_tenant`, `SegmentCipher`, `PlaintextSegmentCipher`, `CryptoSegmentCipher`, `KvCacheError`, `Result`, `EvictionWorker`, `emergency_wipe`, `DEFAULT_BYTE_BUDGET_PER_TENANT`, `TenantPrefixKvStore`, `NamespacePrefixStore`, `KiviQuantizeConfig`, `KvTensorView`, `KiviBlockMeta`, `KiviQuantizedBlock`, `kivi_quantize`, `kivi_dequantize`, `unpack_kivi_block`, `pack_kivi_block`, `compress_bytes`, `decompress_bytes`, `KvReusePolicy` …

### contextra-license

- Pfad: `crates/contextra-license` · Zeilen: 2997 · Rust-Dateien: 18 · Ring: 4
- Interne Cargo-Abhängigkeiten: `contextra-ports` (normal/optional via ports-compat), `contextra-types` (normal/optional via ports-compat)

**Ausgehend (Importe aus anderen Workspace-Crates)**

| Ziel | prod | dev | Dateien (prod) | Symbole (prod) | Symbole (nur dev) |
|---|---:|---:|---:|---|---|
| `contextra-ports` | 26 | 0 | 2 | `license::FeatureRing`×5, `license::LicenseError`×5, `license::LicenseGate`×2, `license::FeatureRing::Fast`×2, `license::FeatureRing::Sovereign`×2, `license::FeatureRing::Compliance`×2, `license::LicenseError::NotActivated`×2, `license::LicenseError::InvalidSignature`×2, `license::LicenseError::Expired`×2, `clock::Clock` … (+1) | – |
| `contextra-types` | 4 | 0 | 1 | `TenantId`×4 | – |

**Pro Komponente**

- `signed_gate` (741 Zeilen)
  - → `contextra-ports`: prod: `license::LicenseGate`, `license::FeatureRing`, `license::LicenseError`
- `types` (249 Zeilen)
  - → `contextra-ports`: prod: `license::FeatureRing`×4, `license::LicenseError`×4, `license::FeatureRing::Fast`×2, `license::FeatureRing::Sovereign`×2, `license::FeatureRing::Compliance`×2, `license::LicenseError::NotActivated`×2, `license::LicenseError::InvalidSignature`×2, `license::LicenseError::Expired`×2, `license::LicenseGate`, `clock::Clock` … (+1)
  - → `contextra-types`: prod: `TenantId`×4

**Öffentliche Schnittstelle** (erreichbar über `pub mod`-Kette oder `pub use`)

- `signed_gate`
  - 1 const, 1 enum, 9 fn, 4 struct
    - `const MAX_PAYLOAD_BYTES`
    - `fn constant_time_eq_32`
    - `struct SignedLicenseOptions`
    - `enum StepVerificationError` · 1 Methoden
    - `struct SignedActivation` · 3 Methoden
    - `fn derive_local_installation_id_hash`
    - `struct LicensePayload`
    - `fn verify_step_1_fast_ring`
    - `fn verify_step_2_presence`
    - `fn verify_step_3_installation`
    - `fn verify_step_4_signature`
    - `fn verify_step_5_expiration`
    - `fn verify_step_6_ring_permission`
    - `fn verify_step_7_success`
    - `struct SignedLicenseGate` · 19 Methoden
- `types`
  - 2 enum, 6 struct, 2 trait, 1 type
    - `enum FeatureRing`
    - `struct AuthorizedRing` · 2 Methoden
    - `struct TenantId` · 2 Methoden ⚠ fremd: `contextra-types`
    - `enum LicenseError`
    - `type Result`
    - `trait Clock` · Methoden: now_unix_nanos, monotonic_nanos
    - `struct FnClock`
    - `struct SystemClock` · 1 Methoden
    - `struct FixedClock`
    - `trait LicenseGate` · Methoden: check_ring, authorize
    - `struct OpenFastGate`
- Zusätzlich 2 `pub`-Items in privaten Modulen ohne Re-Export (nicht Teil der echten API).

**Eingehend (wer nutzt dieses Crate)**

Wird von keinem anderen analysierten Workspace-Crate importiert.

**Nach außen ungenutzte öffentliche Items (Heuristik, 26 von 26)**: `MAX_PAYLOAD_BYTES`, `constant_time_eq_32`, `SignedLicenseOptions`, `StepVerificationError`, `SignedActivation`, `derive_local_installation_id_hash`, `LicensePayload`, `verify_step_1_fast_ring`, `verify_step_2_presence`, `verify_step_3_installation`, `verify_step_4_signature`, `verify_step_5_expiration`, `verify_step_6_ring_permission`, `verify_step_7_success`, `SignedLicenseGate`, `FeatureRing`, `AuthorizedRing`, `TenantId`, `LicenseError`, `Result`, `Clock`, `FnClock`, `SystemClock`, `FixedClock`, `LicenseGate`, `OpenFastGate`

### contextra-mcp

- Pfad: `crates/contextra-mcp` · Zeilen: 8089 · Rust-Dateien: 34 · Ring: 4
- Interne Cargo-Abhängigkeiten: `contextra` (normal), `contextra-crypto` (normal), `contextra-infer-candle` (normal/optional via candle), `contextra-infer-ollama` (normal/optional via ollama), `contextra-infer-onnx` (normal/optional via onnx), `contextra-ports` (normal), `contextra-privacy` (normal), `contextra-rank` (normal), `contextra-router` (normal), `contextra-types` (normal), `contextra-wire` (normal)

**Ausgehend (Importe aus anderen Workspace-Crates)**

| Ziel | prod | dev | Dateien (prod) | Symbole (prod) | Symbole (nur dev) |
|---|---:|---:|---:|---|---|
| `contextra` | 15 | 9 | 6 | `Contextra`×3, `Collection`×2, `chunker`, `chunker::MarkdownChunker::new`, `chunker::ChunkerConfig::default`, `collection_profile`, `collection_profile::DeploymentTier::EdgeMinimal`, `collection_profile::DeploymentTier::PowerUserLocal`, `collection_profile::DeploymentTier::EnterpriseShared`, `collection_profile::DeploymentTier::EnterpriseRegulated` … (+2) | `collection_profile::DeploymentTier` |
| `contextra-router` | 10 | 1 | 3 | `SlmProfile`×4, `DefaultRouterEngine`×2, `ports_local::HybridSearchProvider`, `ports_local::CommunityResolver`, `ports_local::ContextPreparer`, `RouterEngine::new` | `SlmProfile::new` |
| `contextra-ports` | 8 | 38 | 2 | `DriftStatusProvider`×3, `BoxFuture`×2, `plugin::PluginRegistry`×2, `EmbeddingProvider` | `EmbeddingError`×17, `Clock` |
| `contextra-privacy` | 7 | 1 | 3 | `bulk_exfiltration_detector::*`, `egress_gateway::*`, `EgressError`, `egress_vault`, `InjectionDetector`, `egress_guard::*`, `egress_vault::BoxFuture` | `ContextEditAuditRecord` |
| `contextra-types` | 6 | 5 | 3 | `ContextraError`×4, `ContextraError::InvalidInput`, `ContextraError::NotFound` | `TenantId::try_new`, `AutoExtractionMode`, `TokenBudget::new`, `TenantId`, `MAX_SEARCH_K` |
| `contextra-crypto` | 4 | 3 | 1 | `CryptoKey`×3, `CryptoKey::try_new` | `AuditChain` |
| `contextra-infer-candle` | 4 | 0 | 1 | `model_registry::CandleQuantization::Q4KM`×2, `CandleEmbedClient::from_dir`, `CandleLlmClient::from_dir` | – |
| `contextra-infer-ollama` | 3 | 0 | 1 | `OllamaEmbedder::new`, `OllamaConfig`, `OllamaClient::with_config` | – |
| `contextra-infer-onnx` | 3 | 0 | 1 | `OnnxEmbedder::from_path`, `default_model_cache_dir`, `OnnxEmbedder::load` | – |
| `contextra-rank` | 3 | 0 | 2 | `fusion::ProvenanceRecord`, `IsotonicCalibrator`, `IsotonicCalibrator::with_defaults` | – |

**Pro Komponente**

- `bin` (79 Zeilen)
  - → `contextra`: prod: `Contextra`
  - → `contextra-router`: prod: `SlmProfile`
- `bulk_exfiltration_detector` (3 Zeilen)
  - → `contextra-privacy`: prod: `bulk_exfiltration_detector::*`
- `config` (812 Zeilen)
  - → `contextra-infer-candle`: prod: `model_registry::CandleQuantization::Q4KM`×2, `CandleEmbedClient::from_dir`, `CandleLlmClient::from_dir`
  - → `contextra-infer-ollama`: prod: `OllamaEmbedder::new`, `OllamaConfig`, `OllamaClient::with_config`
  - → `contextra-infer-onnx`: prod: `OnnxEmbedder::from_path`, `default_model_cache_dir`, `OnnxEmbedder::load`
  - → `contextra-ports`: dev: `BoxFuture`×2, `EmbeddingError`
  - → `contextra-router`: prod: `SlmProfile`×3
  - → `contextra-types`: prod: `ContextraError`
- `egress_gateway` (24 Zeilen)
  - → `contextra-privacy`: prod: `egress_gateway::*`, `EgressError`, `egress_vault`, `InjectionDetector`
- `egress_guard` (48 Zeilen)
  - → `contextra`: prod: `Collection`
  - → `contextra-privacy`: prod: `egress_guard::*`, `egress_vault::BoxFuture`
- `explain` (239 Zeilen)
  - → `contextra`: dev: `Contextra`
  - → `contextra-ports`: dev: `EmbeddingError`×2, `BoxFuture`, `EmbeddingProvider`
  - → `contextra-rank`: prod: `fusion::ProvenanceRecord`
- `prompt_injection` (1087 Zeilen)
  - → `contextra-ports`: dev: `Clock`
- `protocol` (127 Zeilen)
  - → `contextra-types`: prod: `ContextraError`×2, `ContextraError::InvalidInput`, `ContextraError::NotFound`
- `routing` (102 Zeilen)
  - → `contextra`: prod: `Contextra`, `Collection`
  - → `contextra-ports`: prod: `DriftStatusProvider`×3, `BoxFuture`×2
  - → `contextra-rank`: prod: `IsotonicCalibrator`, `IsotonicCalibrator::with_defaults`
  - → `contextra-router`: prod: `DefaultRouterEngine`×2, `ports_local::HybridSearchProvider`, `ports_local::CommunityResolver`, `ports_local::ContextPreparer`, `RouterEngine::new`
- `sandbox` (697 Zeilen)
  - → `contextra-crypto`: prod: `CryptoKey`×3, `CryptoKey::try_new` · dev: `CryptoKey::try_new`×2
  - → `contextra-types`: prod: `ContextraError`
- `server` (153 Zeilen)
  - → `contextra`: prod: `Contextra`
  - → `contextra-ports`: prod: `plugin::PluginRegistry`×2, `EmbeddingProvider`
- `server_tools` (344 Zeilen)
  - → `contextra`: prod: `chunker`, `chunker::MarkdownChunker::new`, `chunker::ChunkerConfig::default`
- `tests` (872 Zeilen)
  - → `contextra`: dev: `Contextra`
  - → `contextra-ports`: dev: `EmbeddingError`×2, `BoxFuture`, `EmbeddingProvider`
  - → `contextra-types`: dev: `TenantId::try_new`
- `tools_crud` (172 Zeilen)
  - → `contextra`: prod: `collection_profile`, `collection_profile::DeploymentTier::EdgeMinimal`, `collection_profile::DeploymentTier::PowerUserLocal`, `collection_profile::DeploymentTier::EnterpriseShared`, `collection_profile::DeploymentTier::EnterpriseRegulated`, `collection_profile::AutoExtractionMode::Enabled`, `collection_profile::AutoExtractionMode::Disabled`
- `tests/campaign_j42_mcp_audit.rs` (71 Zeilen)
  - → `contextra`: dev: `collection_profile::DeploymentTier`
  - → `contextra-crypto`: dev: `AuditChain`
  - → `contextra-privacy`: dev: `ContextEditAuditRecord`
  - → `contextra-types`: dev: `AutoExtractionMode`
- `tests/confirm_required_semantics.rs` (143 Zeilen)
  - → `contextra`: dev: `Contextra`
  - → `contextra-ports`: dev: `BoxFuture`×2, `EmbeddingError`×2, `EmbeddingProvider`
- `tests/dispatch_direct_tests.rs` (326 Zeilen)
  - → `contextra`: dev: `Contextra`
  - → `contextra-ports`: dev: `EmbeddingError`×2, `BoxFuture`, `EmbeddingProvider`
- `tests/mcp_test.rs` (1129 Zeilen)
  - → `contextra`: dev: `Contextra`
  - → `contextra-ports`: dev: `EmbeddingError`×2, `BoxFuture`, `EmbeddingProvider`
  - → `contextra-router`: dev: `SlmProfile::new`
  - → `contextra-types`: dev: `TokenBudget::new`
- `tests/server_lifecycle_tests.rs` (171 Zeilen)
  - → `contextra`: dev: `Contextra`
  - → `contextra-ports`: dev: `EmbeddingError`×2, `BoxFuture`, `EmbeddingProvider`
  - → `contextra-types`: dev: `TenantId`
- `tests/tool_registry_classification.rs` (158 Zeilen)
  - → `contextra`: dev: `Contextra`
  - → `contextra-ports`: dev: `BoxFuture`×2, `EmbeddingError`×2, `EmbeddingProvider`
- `tests/tools_crud_direct_tests.rs` (484 Zeilen)
  - → `contextra`: dev: `Contextra`
  - → `contextra-ports`: dev: `EmbeddingError`×2, `BoxFuture`, `EmbeddingProvider`
  - → `contextra-types`: dev: `MAX_SEARCH_K`

**Öffentliche Schnittstelle** (erreichbar über `pub mod`-Kette oder `pub use`)

- `config`
  - 4 fn, 3 struct
    - `fn available_compiled_providers`
    - `fn verify_embedding_provider` ⚠ fremd: `contextra-types`
    - `struct EmbeddingConfig` · 4 Methoden
    - `fn create_embedding_provider` ⚠ fremd: `contextra-types`
    - `struct LlmConfig` · 3 Methoden
    - `fn create_llm_text_generator` ⚠ fremd: `contextra-types`
    - `struct RouterConfig` · 1 Methoden ⚠ fremd: `contextra-router`
- `egress_guard`
  - 1 struct
    - `struct CollectionSearchEngine` ⚠ fremd: `contextra`
- `explain`
  - 2 struct
    - `struct ExplainRequest`
    - `struct ExplainResponse` ⚠ fremd: `contextra-rank`
- `io`
  - 2 const, 1 fn
    - `const MAX_RPC_BYTES`
    - `const MAX_SEARCH_QUERY_BYTES`
    - `fn read_line_bounded`
- `protocol`
  - 2 const, 1 enum, 3 fn
    - `const MAX_JSON_DEPTH`
    - `const MAX_BATCH_SIZE`
    - `enum McpError` · 6 Methoden
    - `fn measure_json_depth`
    - `fn validate_json_depth`
    - `fn response_from_error`
- `routing`
  - 1 fn, 1 struct
    - `struct RoutingHandle` ⚠ fremd: `contextra-ports`, `contextra-rank`, `contextra-router`
    - `fn setup_routing` ⚠ fremd: `contextra`
- `sandbox`
  - 4 const, 1 enum, 4 struct
    - `const MAX_VOLATILE_RESULTS`
    - `const MAX_VOLATILE_KEY_BYTES`
    - `const MAX_VOLATILE_OUTPUT_BYTES`
    - `struct ToolDefinition`
    - `const TOOL_REGISTRY`
    - `enum ToolCategory`
    - `struct SandboxPolicy`
    - `struct VolatileToolResult` · 2 Methoden
    - `struct McpSandbox` · 8 Methoden
- `server`
  - 1 struct
    - `struct McpServer` · 9 Methoden ⚠ fremd: `contextra`, `contextra-ports`
- `validation`
  - 2 fn
    - `fn is_write_allowed_by_env`
    - `fn validate_collection_name`
- Zusätzlich 9 `pub`-Items in privaten Modulen ohne Re-Export (nicht Teil der echten API).
- ⚠ Re-Exporte **fremder Crates**: `contextra_privacy::bulk_exfiltration_detector::*`, `contextra_privacy::egress_gateway::*`, `contextra_privacy::egress_guard::*`

**Eingehend (wer nutzt dieses Crate)**

Wird von keinem anderen analysierten Workspace-Crate importiert.

**Nach außen ungenutzte öffentliche Items (Heuristik, 33 von 33)**: `available_compiled_providers`, `verify_embedding_provider`, `EmbeddingConfig`, `create_embedding_provider`, `LlmConfig`, `create_llm_text_generator`, `RouterConfig`, `CollectionSearchEngine`, `ExplainRequest`, `ExplainResponse`, `MAX_RPC_BYTES`, `MAX_SEARCH_QUERY_BYTES`, `read_line_bounded`, `MAX_JSON_DEPTH`, `MAX_BATCH_SIZE`, `McpError`, `measure_json_depth`, `validate_json_depth`, `response_from_error`, `RoutingHandle`, `setup_routing`, `MAX_VOLATILE_RESULTS`, `MAX_VOLATILE_KEY_BYTES`, `MAX_VOLATILE_OUTPUT_BYTES`, `ToolDefinition`, `TOOL_REGISTRY`, `ToolCategory`, `SandboxPolicy`, `VolatileToolResult`, `McpSandbox` …

**Auffälligkeiten**

- contextra-wire: kein Code-Zugriff gefunden (prüfen: Makro, Feature, cfg, oder entfernen)

### contextra-mvcc

- Pfad: `crates/contextra-mvcc` · Zeilen: 6532 · Rust-Dateien: 24 · Ring: 0
- Interne Cargo-Abhängigkeiten: `contextra-testkit` (dev), `contextra-ports` (normal), `contextra-types` (normal)

**Ausgehend (Importe aus anderen Workspace-Crates)**

| Ziel | prod | dev | Dateien (prod) | Symbole (prod) | Symbole (nur dev) |
|---|---:|---:|---:|---|---|
| `contextra-ports` | 3 | 1 | 1 | `MetricsSink`×3 | – |
| `contextra-testkit` | 0 | 6 | 0 | – | `ManualClock`×6 |
| `contextra-types` | 0 | 7 | 0 | – | `TxId`×5, `DocId`, `types` |

**Pro Komponente**

- `seq_log` (738 Zeilen)
  - → `contextra-testkit`: dev: `ManualClock`
- `snapshot` (825 Zeilen)
  - → `contextra-testkit`: dev: `ManualClock`
- `ssi` (1069 Zeilen)
  - → `contextra-ports`: prod: `MetricsSink`×3
- `tx_buffer` (1281 Zeilen)
  - → `contextra-testkit`: dev: `ManualClock`
- `benches/mvcc_bench.rs` (125 Zeilen)
  - → `contextra-types`: dev: `TxId`×2, `DocId`
- `examples/mvcc_basic.rs` (63 Zeilen)
  - → `contextra-types`: dev: `types`
- `tests/b16_reproduction.rs` (113 Zeilen)
  - → `contextra-testkit`: dev: `ManualClock`
- `tests/campaign_cov_ssi_coarsening.rs` (51 Zeilen)
  - → `contextra-types`: dev: `TxId`
- `tests/campaign_cov_tx_buffer_reap.rs` (61 Zeilen)
  - → `contextra-testkit`: dev: `ManualClock`
  - → `contextra-types`: dev: `TxId`
- `tests/history_anomaly_checker.rs` (266 Zeilen)
  - → `contextra-types`: dev: `TxId`
- `tests/isolation_tests.rs` (712 Zeilen)
  - → `contextra-testkit`: dev: `ManualClock`
- `tests/ssi_metrics_observability.rs` (130 Zeilen)
  - → `contextra-ports`: dev: `MetricsSink`

**Öffentliche Schnittstelle** (erreichbar über `pub mod`-Kette oder `pub use`)

- `seq_log`
  - 1 const, 1 enum, 2 struct
    - `const DEFAULT_MAX_PIN_DURATION`
    - `struct SeqLogEntry` · 1 Methoden
    - `enum SeqLogChange` · 2 Methoden
    - `struct SequenceLog` · 25 Methoden
- `snapshot`
  - 3 struct
    - `struct SnapshotRegistryConfig`
    - `struct SnapshotRegistry` · 29 Methoden
    - `struct SnapshotGuard` · 2 Methoden
- `ssi`
  - 1 const, 3 struct, 1 trait
    - `const DEFAULT_MAX_TRACKED_COMMIT_KEYS`
    - `struct ReadSet` · 13 Methoden
    - `trait SsiValidator` · Methoden: validate
    - `struct PruningBlockerInfo`
    - `struct SequenceLogSsiValidator` · 25 Methoden
- `tx_buffer`
  - 6 const, 1 enum, 2 struct, 1 trait
    - `const STAGING_ENTRY_OVERHEAD_BYTES`
    - `const DEFAULT_SHARD_COUNT`
    - `const DEFAULT_MAX_OPS_PER_TX`
    - `const DEFAULT_MAX_READ_SET_KEYS`
    - `const DEFAULT_MAX_TX_STAGED_BYTES`
    - `const DEFAULT_MAX_TOTAL_STAGED_BYTES`
    - `struct TxBufferConfig`
    - `enum IndexOp` · 2 Methoden
    - `trait StagedOpSize` · Methoden: staged_bytes
    - `struct TxBuffer` · 37 Methoden
- Zusätzlich 2 `pub`-Items in privaten Modulen ohne Re-Export (nicht Teil der echten API).

**Eingehend (wer nutzt dieses Crate)**

| Nutzer | Symbole prod | Symbole dev |
|---|---|---|
| `contextra-vector` | `IndexOp::Insert`×2, `IndexOp::Delete`×2, `SequenceLog::new`×2, `SeqLogChange::Insert`, `SeqLogChange::Delete`, `TxBuffer`, `TxBuffer::new_with_config`, `SequenceLog` | – |
| `contextra-store` | `SsiValidator`×4, `tx_buffer::STAGING_ENTRY_OVERHEAD_BYTES`×2, `SnapshotRegistry`, `SequenceLogSsiValidator`, `IndexOp`, `SequenceLogSsiValidator::new_with_bounds` | `SnapshotRegistry::new` |
| `contextra-checkpoint` | `SnapshotRegistry` | – |
| `contextra-engine` | `tx_buffer::TxBuffer` | `SnapshotRegistry`×2, `tx_buffer::IndexOp` |

**Nach außen ungenutzte öffentliche Items (Heuristik, 15 von 22)**: `DEFAULT_MAX_PIN_DURATION`, `SeqLogEntry`, `SeqLogChange`, `SnapshotRegistryConfig`, `SnapshotGuard`, `DEFAULT_MAX_TRACKED_COMMIT_KEYS`, `ReadSet`, `PruningBlockerInfo`, `DEFAULT_SHARD_COUNT`, `DEFAULT_MAX_OPS_PER_TX`, `DEFAULT_MAX_READ_SET_KEYS`, `DEFAULT_MAX_TX_STAGED_BYTES`, `DEFAULT_MAX_TOTAL_STAGED_BYTES`, `TxBufferConfig`, `StagedOpSize`

**Auffälligkeiten**

- contextra-types: nur in Tests/Benches genutzt, aber als normale Abhängigkeit deklariert → dev-dependency?

### contextra-ports

- Pfad: `crates/contextra-ports` · Zeilen: 6843 · Rust-Dateien: 43 · Ring: 0
- Interne Cargo-Abhängigkeiten: `contextra-types` (normal)

**Ausgehend (Importe aus anderen Workspace-Crates)**

| Ziel | prod | dev | Dateien (prod) | Symbole (prod) | Symbole (nur dev) |
|---|---:|---:|---:|---|---|
| `contextra-types` | 3 | 0 | 3 | `RequestContext`×2, `ids::ErasureId` | – |

**Pro Komponente**

- `observer` (141 Zeilen)
  - → `contextra-types`: prod: `ids::ErasureId`
- `policy` (30 Zeilen)
  - → `contextra-types`: prod: `RequestContext`
- `scanner` (483 Zeilen)
  - → `contextra-types`: prod: `RequestContext`

**Öffentliche Schnittstelle** (erreichbar über `pub mod`-Kette oder `pub use`)

- `(crate-root)`
  - 2 type
    - `type BoxFuture`
    - `type BoxStream`
- `attention`
  - 1 struct, 1 trait
    - `struct RequestId`
    - `trait AttentionExporter` · Methoden: export_attention_weights
- `barrier`
  - 4 struct, 2 trait
    - `struct BarrierShared` · 1 Methoden
    - `struct BarrierExclusive` · 1 Methoden
    - `struct CompactionPermit` · 1 Methoden
    - `struct PurgePermit` · 1 Methoden
    - `trait CommitBarrier` · Methoden: shared, exclusive
    - `trait PurgeBarrier` · Methoden: compaction, purge, purge_waiting
- `checkpoint`
  - 2 trait
    - `trait Checkpoint` · Methoden: take_snapshot, restore
    - `trait CheckpointCoordinator` · Methoden: create_named_checkpoint, restore_named_checkpoint, drop_named_checkpoint, list_named_checkpoints
- `clock`
  - 1 struct, 1 trait
    - `trait Clock` · Methoden: now_unix_nanos, monotonic_nanos
    - `struct SystemClock` · 1 Methoden
- `conformance`
  - 20 fn
    - `fn assert_checkpoint_contract`
    - `fn assert_checkpoint_coordinator_contract`
    - `fn assert_vector_index_contract`
    - `fn assert_attention_exporter_contract`
    - `fn assert_graph_index_contract`
    - `fn assert_lifecycle_contract`
    - `fn assert_embedding_contract`
    - `fn assert_observability_contract`
    - `fn assert_license_contract`
    - `fn assert_reranker_contract`
    - `fn assert_text_index_contract`
    - `fn assert_storage_engine_contract`
    - `fn assert_purger_contract`
    - `fn assert_vfs_file_contract`
    - `fn assert_read_gate_contract`
    - `fn assert_keys_contract`
    - `fn assert_ledger_contract`
    - `fn assert_erasure_layer_contract`
    - `fn assert_egress_policy_contract`
    - `fn assert_visitor_methods`
- `embedding`
  - 1 enum, 1 struct, 5 trait
    - `struct ContextSegment` · 3 Methoden
    - `trait TextEmbeddingEngine` · Methoden: embed, embed_batch
    - `enum EmbeddingError`
    - `trait EmbeddingProvider` · Methoden: provider_name, embed, embedding_dim, embed_batch
    - `trait TextGenerator` · Methoden: generate_text
    - `trait LlmTextGenerator` · Methoden: generate, generate_with_context
    - `trait LlmTextGeneratorStreaming` · Methoden: generate_stream
- `erasure`
  - 2 enum, 6 struct, 3 trait
    - `struct LayerId` · 2 Methoden
    - `enum EraseTarget`
    - `struct LayerScope` · 4 Methoden
    - `struct PurgeCx` · 1 Methoden
    - `struct PurgeOutcome` · 1 Methoden
    - `struct Armed`
    - `struct Scanner` · 1 Methoden
    - `enum KeyCoverage`
    - `trait ErasureLayer` · Methoden: id, resolve, on_gate, purge, witness, key_coverage
    - `trait StorageBackend`
    - `trait ErasureTarget`
- `gate`
  - 3 enum, 3 struct, 1 trait
    - `struct Seq` · 2 Methoden
    - `struct KeyBuf` · 2 Methoden
    - `enum KeyBound` · 1 Methoden
    - `struct KeyRange` · 2 Methoden
    - `enum GateDecision`
    - `enum RangeGate`
    - `trait ReadGate` · Methoden: check, check_bound, variant_for_range, gated_seq, gate_epoch, covers…
- `graph`
  - 1 enum, 3 struct, 1 trait
    - `struct RoleId` · 2 Methoden
    - `struct RoleBinding` · 1 Methoden
    - `struct HyperEdgeId` · 2 Methoden
    - `enum GraphMutationError`
    - `trait GraphCollectionMutation` · Methoden: relate_n_ary
- `graph_index`
  - 1 struct, 2 trait
    - `trait GraphIndex` · Methoden: traverse, neighbors, remove_edge, add_bidirectional, multi_traverse, multi_traverse_at…
    - `struct GraphIndexStats`
    - `trait CommunityResolver` · Methoden: get_community
- `id_gen`
  - 1 struct, 1 trait
    - `trait IdGen` · Methoden: next_id
    - `struct SequentialIdGen` · 1 Methoden
- `keys`
  - 1 enum, 4 struct, 2 trait
    - `struct FileId` · 2 Methoden
    - `enum FileKind`
    - `struct FileKey` · 3 Methoden
    - `struct SealCtx` · 1 Methoden
    - `struct Envelope` · 2 Methoden
    - `trait RecordSealer` · Methoden: seal, open, evict, make_slots_durable
    - `trait FileKeyring` · Methoden: create, open, destroy
- `kv`
  - 5 struct, 1 trait
    - `struct KvLayout`
    - `struct RopeConfig`
    - `struct PrefixKey`
    - `struct KvBlock`
- `kv_bridge_port`
  - 1 trait
- `ledger`
  - 3 struct, 1 trait
- `license`
  - 3 enum, 2 struct, 1 trait
- `lifecycle`
  - 1 struct, 3 trait
- `metrics`
  - 5 struct, 1 trait
- `observability`
  - 1 trait
- `observer`
  - 3 enum, 1 struct, 1 trait, 1 type
- `plugin`
  - 1 enum, 2 struct, 1 trait
- `policy`
  - 1 enum, 1 trait
- `query_rewriter`
  - 1 trait
- `report_signer`
  - 1 enum, 1 struct, 1 trait, 1 type
- `reranker`
  - 1 trait
- `rng`
  - 1 struct, 1 trait
- `scanner`
  - 2 enum, 4 struct, 2 trait
- `storage`
  - 1 const, 1 struct, 1 trait
- `text_index`
  - 1 struct, 2 trait
- `vector_index`
  - 1 struct, 2 trait
- `vfs`
  - 1 enum, 1 struct, 2 trait
  - … 65 weitere Items nicht aufgelistet (`--max-items`)
- Zusätzlich 10 `pub`-Items in privaten Modulen ohne Re-Export (nicht Teil der echten API).
- Interne Re-Exporte (`pub use`): `LayerWitness`, `ReportSigner`, `QueryRewriter`, `Reranker`, `KvPrefixStore`, `TextEmbeddingEngine`

**Eingehend (wer nutzt dieses Crate)**

| Nutzer | Symbole prod | Symbole dev |
|---|---|---|
| `contextra-checkpoint` | `Clock`×20, `StorageEngine`×14, `Vfs`×7, `MetricsSink`×6, `BoxFuture`×3, `SystemClock`×2, `Checkpoint`×2, `CheckpointCoordinator` | – |
| `contextra-store` | `Clock`×10, `MetricsSink`×6, `MAX_SCAN_MERGE_ACCUMULATOR`×6, `StorageStats`×5, `scanner`×5, `Vfs`×3, `SystemClock::new`×3, `StorageEngine`×3, `IdGen`×3, `ResourceTracker`×2 … (+7) | `ResourceBudget`×28, `ResourceTracker::new`×27, `clock::SystemClock`, `storage::StorageEngine` |
| `contextra-engine` | `MetricsSink`×9, `StorageEngine`×5, `Clock`×3, `BoxFuture`×2, `VectorIndexStats`×2, `Reranker`×2, `DriftStatusProvider`×2, `VectorIndex`, `SystemClock::new`, `GraphIndex` … (+7) | – |
| `contextra-vector` | `Clock`×15, `MetricsSink`×6, `Rng`×4, `SeededRng::new`×3, `VectorIndex`×2, `scanner`×2, `NoopMetricsSink`×2 | `SeededRng`×5, `vector_index::VectorIndex`, `clock::SystemClock` |
| `contextra-graph` | `BoxFuture`×5, `GraphIndex`×4, `StorageEngine`×2, `graph::HyperEdgeId::new`×2, `GraphIndex::personalized_page_rank_at`, `GraphIndex::rollback_to_tx`, `graph::RoleBinding`, `graph::HyperEdgeId`, `graph::GraphMutationError`, `graph::GraphMutationError::LockAcquisitionTimeout` … (+10) | `Result`×3, `graph_index::GraphIndex` |
| `contextra-license` | `license::FeatureRing`×5, `license::LicenseError`×5, `license::LicenseGate`×2, `license::FeatureRing::Fast`×2, `license::FeatureRing::Sovereign`×2, `license::FeatureRing::Compliance`×2, `license::LicenseError::NotActivated`×2, `license::LicenseError::InvalidSignature`×2, `license::LicenseError::Expired`×2, `clock::Clock` … (+1) | – |
| `contextra-kvcache` | `kv_bridge_port::KvBridgeStorage`×2, `scanner::PurgeStats`×2, `scanner::ScanSession`×2, `embedding::EmbeddingProvider`, `license::FeatureRing`, `storage::StorageEngine`, `BoxFuture`, `plugin`, `RequestId`, `kv` … (+8) | `kv::KvLayout`, `kv::RopeConfig`, `clock::SystemClock`, `scanner` … (+4) |
| `contextra-infer-candle` | `kv::KvLayout`×6, `kv::RopeConfig`×6, `embedding::EmbeddingError`, `kv::KvPrefixStore`, `kv`, `kv::KvBlock` | `kv::PrefixKey`×4, `ContextSegment`×2, `GroundingValidator`×2, `EmbeddingProvider`×2 … (+3) |
| `contextra-infer-ollama` | `Rng`×2, `BoxFuture`×2, `LlmTextGenerator`, `LlmTextGeneratorStreaming`, `BoxStream`, `SegmentSynthesizer`, `EmbeddingError`, `EmbeddingError::Unavailable`, `EmbeddingError::ComputationFailed`, `SystemClock` … (+4) | `EmbeddingProvider` |
| `contextra-agent` | `BoxFuture`×7, `MetricsSink`×3, `StorageEngine`×2, `VectorIndex`×2, `Checkpoint` | `SystemClock` |
| `contextra-testkit` | `Clock`×2, `Vfs`×2, `GraphIndex`, `MetricsSink`, `Rng`, `StorageEngine`, `TextIndex`, `VectorIndex`, `scanner::WitnessLayer`, `scanner::ScanTechnique` … (+2) | `TextEmbeddingEngine` |
| `contextra` | `license::FeatureRing`×2, `license::LicenseGate`×2, `IdGen`, `license`, `license::GateToken::for_test`, `license::VectorDeleteMode`, `license::LicenseError` | `BoxFuture`×3, `license::GateToken`, `license::LicenseError::InsufficientRing`, `SequentialIdGen` |
| `contextra-mcp` | `DriftStatusProvider`×3, `BoxFuture`×2, `plugin::PluginRegistry`×2, `EmbeddingProvider` | `EmbeddingError`×17, `Clock` |
| `contextra-text` | `StorageEngine`×6, `scanner`×2 | `BoxFuture`×50, `StorageStats`×35, `clock::SystemClock` |
| `contextra-crypto` | `scanner::WitnessLayer`, `scanner::LayerWitness`, `Clock`, `scanner::ScanTechnique::NotMeasured`, `ScanTechnique`, `scanner` | `Result`×24, `SystemClock`×6, `scanner::ScanTechnique`×4 |
| `contextra-mvcc` | `MetricsSink`×3 | – |
| `contextra-bench` | `VectorIndex`, `EmbeddingProvider` | `StorageEngine` |
| `contextra-cognition` | `SeededRng::from_entropy`, `(crate)` | `GroundingAssessment`×2, `BoxFuture`, `GroundingVerdict::Pass`, `StorageEngine` … (+1) |
| `contextra-rank` | `Clock`×2 | – |
| `contextra-router` | `vfs::Vfs`, `clock` | `clock::Clock`×2, `BoxFuture`×2, `HybridSearchProvider`, `CommunityResolver` … (+1) |
| `contextra-db` | `QueryRewriter` | `GraphIndex`×4, `StorageEngine`×2, `BoxFuture`×2, `LlmTextGenerator` … (+3) |

**Nach außen ungenutzte öffentliche Items (Heuristik, 83 von 145)**: `BarrierShared`, `BarrierExclusive`, `CompactionPermit`, `PurgePermit`, `CommitBarrier`, `PurgeBarrier`, `assert_checkpoint_contract`, `assert_checkpoint_coordinator_contract`, `assert_vector_index_contract`, `assert_attention_exporter_contract`, `assert_graph_index_contract`, `assert_lifecycle_contract`, `assert_embedding_contract`, `assert_observability_contract`, `assert_license_contract`, `assert_reranker_contract`, `assert_text_index_contract`, `assert_storage_engine_contract`, `assert_purger_contract`, `assert_vfs_file_contract`, `assert_read_gate_contract`, `assert_keys_contract`, `assert_ledger_contract`, `assert_erasure_layer_contract`, `assert_egress_policy_contract`, `assert_visitor_methods`, `TextGenerator`, `LayerId`, `EraseTarget`, `LayerScope` …

### contextra-privacy

- Pfad: `crates/contextra-privacy` · Zeilen: 7821 · Rust-Dateien: 25 · Ring: 3
- Interne Cargo-Abhängigkeiten: keine

**Ausgehend (Importe aus anderen Workspace-Crates)**

Keine.

**Pro Komponente**

Keine Importe aus anderen Workspace-Crates.

**Öffentliche Schnittstelle** (erreichbar über `pub mod`-Kette oder `pub use`)

- `audit_trace`
  - 2 fn, 1 trait
    - `fn extract_rule_id`
    - `fn compute_audit_trace`
    - `trait EgressClassifierTrace` · Methoden: classify_with_trace
- `bulk_exfiltration_detector`
  - 1 enum, 4 struct
    - `struct DeterministicBlake3Hasher`
    - `struct DeterministicBuildHasher`
    - `struct SessionId`
    - `enum BulkExfiltrationOutcome`
    - `struct BulkExfiltrationDetector` · 5 Methoden
- `context_edit_audit`
  - 2 enum, 4 fn, 2 struct
    - `enum ContextEditKind`
    - `enum ContextEditAuditError`
    - `struct ContextEditInput`
    - `struct ContextEditAuditRecord`
    - `fn compute_record_hash`
    - `fn build_context_edit_audit_record`
    - `fn verify_audit_chain`
    - `fn render_audit_line`
- `egress_gateway`
  - 1 const, 12 fn, 6 struct, 2 trait, 1 type
    - `type DefaultEgressClassifier`
    - `const MAX_SEARCH_QUERY_BYTES`
    - `trait EgressGuardCheck` · Methoden: check
    - `struct DefaultEgressGuard`
    - `trait InjectionDetector` · Methoden: detect
    - `struct EgressGateway` · 3 Methoden
    - `struct EgressGatewayBuilder` · 9 Methoden
    - `struct CloudResponseRehydrator` · 2 Methoden
    - `fn pii_vault_requires_isolated_retention`
    - `fn pii_vault_forces_crypto_shred`
    - `fn resolve_effective_retention_mode`
    - `fn resolve_effective_kv_delete_mode`
    - `struct CloudQueryRequest`
    - `struct CloudQueryResponse`
    - `fn handle_cloud_query`
    - `fn handle_cloud_query_scoped`
    - `fn handle_cloud_query_scoped_with_guard`
    - `fn handle_cloud_query_scoped_with_bulk_detector`
    - `fn handle_cloud_query_with_guard`
    - `fn check_bulk_exfiltration`
    - `fn handle_cloud_query_with_bulk_detector`
    - `fn process_cloud_response`
- `egress_guard`
  - 3 const, 2 struct, 1 trait
    - `struct TextSearchResult`
    - `trait TextSearchEngine` · Methoden: search_text
    - `const DEFAULT_EGRESS_GUARD_TIMEOUT`
    - `const DEFAULT_EGRESS_GUARD_THRESHOLD`
    - `const DEFAULT_EGRESS_GUARD_MIN_BYTES`
    - `struct EgressGuard` · 7 Methoden
- `egress_vault`
  - 1 const, 4 enum, 2 fn, 4 struct, 2 trait, 1 type
    - `trait EntityRecognizer` · Methoden: recognize
    - `struct NoOpRecognizer`
    - `struct SurrogateVault` · 8 Methoden
    - `type BoxFuture`
    - `enum PolicyCategory`
    - `enum BlockReason` · 1 Methoden
    - `enum EgressClassification`
    - `struct CompiledPattern` · 1 Methoden
    - `enum EgressVaultError`
    - `const MAX_CLASSIFY_PAYLOAD_BYTES`
    - `fn classify_layer1`
    - `fn classify_layer1_arc`
    - `struct EgressVault` · 12 Methoden
    - `trait EgressClassifier` · Methoden: classify
- `entity_recognizer`
  - 1 const, 3 struct, 1 type
    - `const DEFAULT_MAX_INPUT_LENGTH`
    - `type PatternRecognizer`
    - `struct EntityMatch`
    - `struct Gazetteer` · 4 Methoden
    - `struct RuleBasedEntityRecognizer` · 9 Methoden
- `error`
  - 1 enum, 2 type
    - `enum EgressError` · 8 Methoden
    - `type Error`
    - `type Result`
- `guarded_payload`
  - 3 struct
    - `struct Unsanitized`
    - `struct Sanitized`
    - `struct GuardedPayload` · 4 Methoden
- `processing_registry`
  - 1 enum, 3 fn, 3 struct
    - `enum ProcessorRole` · 1 Methoden
    - `fn default_version`
    - `struct ProcessingActivityRecord`
    - `struct ProcessingRegistry` · 4 Methoden
    - `struct ProcessingRegistryExport` · 2 Methoden
    - `fn generate_registry`
    - `fn render_markdown`
- `types`
  - 2 enum, 3 struct, 2 trait
    - `trait Clock` · Methoden: now_unix_nanos, monotonic_nanos
    - `struct SystemClock` · 1 Methoden
    - `struct TenantId` · 3 Methoden
    - `enum TenantScopeViolation`
  - … 3 weitere Items nicht aufgelistet (`--max-items`)
- Zusätzlich 1 `pub`-Items in privaten Modulen ohne Re-Export (nicht Teil der echten API).

**Eingehend (wer nutzt dieses Crate)**

| Nutzer | Symbole prod | Symbole dev |
|---|---|---|
| `contextra-mcp` | `bulk_exfiltration_detector::*`, `egress_gateway::*`, `EgressError`, `egress_vault`, `InjectionDetector`, `egress_guard::*`, `egress_vault::BoxFuture` | `ContextEditAuditRecord` |

*Hinweis: Mindestens ein Crate importiert per Glob (`*`); ungenutzte Items sind deshalb nicht bestimmbar.*

### contextra-py

- Pfad: `crates/contextra-py` · Zeilen: 3253 · Rust-Dateien: 17 · Ring: 4
- Interne Cargo-Abhängigkeiten: `contextra` (normal), `contextra-types` (normal), `contextra-wire` (normal)

**Ausgehend (Importe aus anderen Workspace-Crates)**

| Ziel | prod | dev | Dateien (prod) | Symbole (prod) | Symbole (nur dev) |
|---|---:|---:|---:|---|---|
| `contextra-wire` | 8 | 0 | 1 | `ScoredDocument::create`×2, `ScoredDocumentArgs`×2, `SearchResponse::create`×2, `SearchResponseArgs`×2 | – |
| `contextra` | 7 | 3 | 4 | `Collection`, `Document`, `SearchResult`, `Contextra`, `DistanceMetric::Cosine`, `DistanceMetric::Euclidean`, `DistanceMetric::DotProduct` | – |
| `contextra-types` | 5 | 0 | 3 | `FusionWeights::new`×2, `ContextraError`, `ContextraErrorDto::from`, `TenantId::SYSTEM` | – |

**Pro Komponente**

- `bindings` (2823 Zeilen)
  - → `contextra`: prod: `Collection`, `Document`, `SearchResult`, `Contextra`, `DistanceMetric::Cosine`, `DistanceMetric::Euclidean`, `DistanceMetric::DotProduct` · dev: `Document`×2, `SearchResult`
  - → `contextra-types`: prod: `FusionWeights::new`×2, `ContextraError`, `ContextraErrorDto::from`, `TenantId::SYSTEM`
  - → `contextra-wire`: prod: `ScoredDocument::create`×2, `ScoredDocumentArgs`×2, `SearchResponse::create`×2, `SearchResponseArgs`×2

**Öffentliche Schnittstelle** (erreichbar über `pub mod`-Kette oder `pub use`)

- `(crate-root)`
  - 1 const
    - `const VERSION`
- `bindings`
  - 6 const, 21 fn, 8 struct
    - `struct PyCollection` · 6 Methoden
    - `const MIN_WORKER_THREADS`
    - `const MAX_WORKER_THREADS`
    - `const MAX_ID_LENGTH`
    - `const MAX_COLLECTION_NAME_LENGTH`
    - `const MAX_LABEL_LENGTH`
    - `const MAX_BATCH_SIZE`
    - `fn dict_to_json`
    - `fn opt_dict_to_json`
    - `fn validate_label`
    - `fn validate_id`
    - `fn validate_collection_name`
    - `fn validate_db_path`
    - `fn validate_query_text`
    - `fn validate_batch_size`
    - `fn validate_vector`
    - `fn validate_id_obj`
    - `fn check_subinterpreter_guard`
    - `fn run_blocking_ffi`
    - `fn json_to_py`
    - `fn doc_to_py` ⚠ fremd: `contextra`
    - `fn results_to_py` ⚠ fremd: `contextra`
    - `fn contextra_err` ⚠ fremd: `contextra-types`
    - `struct PyContextra` · 11 Methoden
    - `struct PyDbStats`
    - `struct PyDocument`
    - `fn open`
    - `fn _trigger_panic_for_test`
    - `fn validate_hyperedge_args`
    - `fn parse_worker_threads_env`
    - `struct PyRuntimeState`
    - `fn get_runtime`
    - `struct PySearchResult`
    - `struct PyStorageStats`
    - `struct PyVectorIndexStats`
- `kv_links`
  - 1 fn
    - `fn parse_link_relation`

**Eingehend (wer nutzt dieses Crate)**

Wird von keinem anderen analysierten Workspace-Crate importiert.

**Nach außen ungenutzte öffentliche Items (Heuristik, 37 von 37)**: `PyCollection`, `MIN_WORKER_THREADS`, `MAX_WORKER_THREADS`, `MAX_ID_LENGTH`, `MAX_COLLECTION_NAME_LENGTH`, `MAX_LABEL_LENGTH`, `MAX_BATCH_SIZE`, `dict_to_json`, `opt_dict_to_json`, `validate_label`, `validate_id`, `validate_collection_name`, `validate_db_path`, `validate_query_text`, `validate_batch_size`, `validate_vector`, `validate_id_obj`, `check_subinterpreter_guard`, `run_blocking_ffi`, `json_to_py`, `doc_to_py`, `results_to_py`, `contextra_err`, `PyContextra`, `PyDbStats`, `PyDocument`, `open`, `_trigger_panic_for_test`, `validate_hyperedge_args`, `parse_worker_threads_env` …

### contextra-rank

- Pfad: `crates/contextra-rank` · Zeilen: 7660 · Rust-Dateien: 42 · Ring: 0
- Interne Cargo-Abhängigkeiten: `contextra-ports` (normal/optional via types-compat), `contextra-types` (normal/optional via types-compat)

**Ausgehend (Importe aus anderen Workspace-Crates)**

| Ziel | prod | dev | Dateien (prod) | Symbole (prod) | Symbole (nur dev) |
|---|---:|---:|---:|---|---|
| `contextra-types` | 38 | 0 | 4 | `DocId`×5, `SignalFusionStrategies`×4, `ContextraError::invalid_input`×3, `FusionWeights`×3, `FusionStrategy`×3, `OnSignalFailure`×3, `ContextraError`×2, `ContextChunk`×2, `ConfigFingerprint`×2, `FusionStrategy::Rrf`×2 … (+6) | – |
| `contextra-ports` | 2 | 0 | 1 | `Clock`×2 | – |

**Pro Komponente**

- `calibration` (843 Zeilen)
  - → `contextra-ports`: prod: `Clock`×2
- `error` (28 Zeilen)
  - → `contextra-types`: prod: `ContextraError::invalid_input`×3, `ContextraError`
- `fusion` (2133 Zeilen)
  - → `contextra-types`: prod: `ContextChunk`×2, `FusionWeights`, `ContextraError`, `DocId::from_key`, `ContextraError::InvalidInput`, `MemoryLink`, `DocId`
- `local_types` (241 Zeilen)
  - → `contextra-types`: prod: `DocId`×4, `SignalFusionStrategies`×4, `FusionStrategy`×3, `OnSignalFailure`×3, `ConfigFingerprint`×2, `FusionStrategy::Rrf`×2, `FusionStrategy::ScoreNormalized`×2, `OnSignalFailure::Fail`×2, `OnSignalFailure::Degrade`×2, `FusionWeights`×2

**Öffentliche Schnittstelle** (erreichbar über `pub mod`-Kette oder `pub use`)

- `calibration`
  - 2 enum, 1 fn, 4 struct, 2 trait
    - `enum ConformalError`
    - `trait ConformalCalibrator` · Methoden: update, threshold
    - `struct AdaptiveConformalCalibrator` · 7 Methoden
    - `trait Clock` · Methoden: now_unix_nanos, monotonic_nanos ⚠ fremd: `contextra-ports`
    - `struct SystemClock` · 1 Methoden
    - `struct IsotonicCalibrator` · 13 Methoden
    - `enum Calibrator` · 3 Methoden
    - `fn calibrate_score`
    - `struct PlattScaler` · 10 Methoden
- `dibud`
  - 2 enum, 2 fn, 3 struct
    - `fn fuse_exact_prefix`
    - `fn fuse_exact_prefix_async`
    - `enum DiBudStep`
    - `struct DiBudFusionState` · 7 Methoden
    - `enum BudgetedChannel` · 2 Methoden
    - `struct FusionBudget` · 1 Methoden
    - `struct DiBudOutcome`
- `drift`
  - 1 enum, 1 struct
    - `enum DriftStatus` · 1 Methoden
    - `struct DriftDetector` · 6 Methoden
- `error`
  - 1 enum, 1 type
    - `enum RankError`
    - `type Result`
- `explain`
  - 1 fn, 2 struct
    - `struct ExplanationEntry`
    - `struct RetrievalExplanation` · 1 Methoden
    - `fn explain`
- `fusion`
  - 1 enum, 1 fn, 2 struct
    - `fn weights_to_signal_factors_types_compat` *(re-exportiert)* ⚠ fremd: `contextra-types`
    - `struct ProvenanceBuilder` *(re-exportiert)* · 10 Methoden
    - `enum SignalKind` *(re-exportiert)* · 3 Methoden
    - `struct BoundedTopK` *(re-exportiert)* · 6 Methoden
- `local_types`
  - 1 const, 2 enum, 4 struct
    - `const MAX_SEARCH_K`
    - `struct DocId` · 1 Methoden ⚠ fremd: `contextra-types`
    - `struct ConfigFingerprint` · 3 Methoden ⚠ fremd: `contextra-types`
    - `enum FusionStrategy` ⚠ fremd: `contextra-types`
    - `enum SignalFailurePolicy`
    - `struct SignalFusionStrategies` · 1 Methoden ⚠ fremd: `contextra-types`
    - `struct FusionWeights` ⚠ fremd: `contextra-types`
- `metrics`
  - 5 fn
    - `fn reciprocal_rank`
    - `fn mean_reciprocal_rank`
    - `fn precision_at_k`
    - `fn recall_at_k`
    - `fn ndcg_at_k`
- Zusätzlich 27 `pub`-Items in privaten Modulen ohne Re-Export (nicht Teil der echten API).
- Interne Re-Exporte (`pub use`): `SignalKind`, `FusionStrategy`, `SignalFailurePolicy`

**Eingehend (wer nutzt dieses Crate)**

| Nutzer | Symbole prod | Symbole dev |
|---|---|---|
| `contextra-engine` | `SearchResult`×8, `IsotonicCalibrator`×2, `fusion`, `fusion::fuse_search_results_with_strategy`, `fusion::fuse_search_results_with_signal_strategies`, `fusion::fuse_signals` | – |
| `contextra-mcp` | `fusion::ProvenanceRecord`, `IsotonicCalibrator`, `IsotonicCalibrator::with_defaults` | – |
| `contextra-bench` | `RankError` | – |
| `contextra-infer-candle` | `IsotonicCalibrator` | – |
| `contextra-db` | – | `IsotonicCalibrator::new` |

**Nach außen ungenutzte öffentliche Items (Heuristik, 37 von 39)**: `ConformalError`, `ConformalCalibrator`, `AdaptiveConformalCalibrator`, `Clock`, `SystemClock`, `Calibrator`, `calibrate_score`, `PlattScaler`, `fuse_exact_prefix`, `fuse_exact_prefix_async`, `DiBudStep`, `DiBudFusionState`, `BudgetedChannel`, `FusionBudget`, `DiBudOutcome`, `DriftStatus`, `DriftDetector`, `Result`, `ExplanationEntry`, `RetrievalExplanation`, `explain`, `weights_to_signal_factors_types_compat`, `ProvenanceBuilder`, `SignalKind`, `BoundedTopK`, `MAX_SEARCH_K`, `DocId`, `ConfigFingerprint`, `FusionStrategy`, `SignalFailurePolicy` …

### contextra-router

- Pfad: `crates/contextra-router` · Zeilen: 8958 · Rust-Dateien: 39 · Ring: 3
- Interne Cargo-Abhängigkeiten: `contextra-adapt` (normal), `contextra-ports` (normal), `contextra-types` (normal), `contextra-wire` (normal)

**Ausgehend (Importe aus anderen Workspace-Crates)**

| Ziel | prod | dev | Dateien (prod) | Symbole (prod) | Symbole (nur dev) |
|---|---:|---:|---:|---|---|
| `contextra-adapt` | 4 | 4 | 2 | `bandit`, `lyapunov`, `offpolicy`, `offpolicy::RandomizedLoggingPolicy` | `bandit::BanditProfileState`×2, `BanditError`, `FcTsError::InvalidConfig` |
| `contextra-ports` | 2 | 7 | 1 | `vfs::Vfs`, `clock` | `clock::Clock`×2, `BoxFuture`×2, `HybridSearchProvider`, `CommunityResolver`, `ContextPreparer` |
| `contextra-types` | 2 | 21 | 2 | `RetrievalStrategy`, `TokenBudget::default` | `DocId::new`×8, `ConfigFingerprint`×7, `TokenBudget`×4, `Result` |

**Pro Komponente**

- `(crate-root)` (68 Zeilen)
  - → `contextra-adapt`: prod: `bandit`, `lyapunov`, `offpolicy`
- `arm_registry` (171 Zeilen)
  - → `contextra-types`: prod: `RetrievalStrategy`
- `bandit_regret_tests` (274 Zeilen)
  - → `contextra-adapt`: dev: `BanditError`
- `router` (1485 Zeilen)
  - → `contextra-adapt`: prod: `offpolicy::RandomizedLoggingPolicy`
  - → `contextra-ports`: prod: `vfs::Vfs`, `clock`
  - → `contextra-types`: prod: `TokenBudget::default` · dev: `TokenBudget`
- `tests` (3651 Zeilen)
  - → `contextra-ports`: dev: `clock::Clock`
  - → `contextra-types`: dev: `ConfigFingerprint`×7, `DocId::new`×5, `TokenBudget`×2, `Result`
- `benches/cost_routing_savings_bench.rs` (151 Zeilen)
  - → `contextra-types`: dev: `TokenBudget`
- `benches/router_bench.rs` (162 Zeilen)
  - → `contextra-types`: dev: `DocId::new`
- `examples/simple_router.rs` (75 Zeilen)
  - → `contextra-types`: dev: `DocId::new`
- `tests/arm_registry_five_arms.rs` (61 Zeilen)
  - → `contextra-types`: dev: `RetrievalStrategy`
- `tests/fc_ts_dispatch_test.rs` (104 Zeilen)
  - → `contextra-adapt`: dev: `FcTsError::InvalidConfig`
- `tests/regret_monotonicity.rs` (262 Zeilen)
  - → `contextra-adapt`: dev: `bandit::BanditProfileState`
- `tests/router_audit_and_isolation_tests.rs` (780 Zeilen)
  - → `contextra-ports`: dev: `BoxFuture`×2, `clock::Clock`, `HybridSearchProvider`, `CommunityResolver` … (+1)
  - → `contextra-types`: dev: `DocId::new`
- `tests/router_bandit_dispatch.rs` (256 Zeilen)
  - → `contextra-adapt`: dev: `bandit::BanditProfileState`

**Öffentliche Schnittstelle** (erreichbar über `pub mod`-Kette oder `pub use`)

- `arm_registry`
  - 1 enum, 1 struct
    - `struct ArmRegistry` · 2 Methoden
    - `enum ArmRegistryError`
- `dispatch`
  - 1 fn
    - `fn dispatch_to_slm`
- `fc_ts_dispatch`
  - 1 enum, 2 fn
    - `enum FcTsDispatchError`
    - `fn select_profile_fc_ts`
    - `fn deterministic_fc_ts_rng`
- `outcome`
  - 1 enum, 2 struct
    - `struct DecisionId` · 2 Methoden
    - `struct DecisionIdGenerator` · 2 Methoden
    - `enum RoutingOutcome` · 1 Methoden
- `profile`
  - 1 enum, 3 struct, 1 type
    - `enum QuantizationLevel`
    - `struct SlmProfile` · 6 Methoden
    - `struct ProfileCalibrator` · 4 Methoden
    - `type ConformalCalibrator`
    - `struct ProfileCalibrationState` · 6 Methoden
- `router`
  - 4 struct, 1 type
    - `struct ConfidenceMetrics`
    - `struct RoutingDecision`
    - `struct RouterState`
    - `type DefaultRouterEngine`
    - `struct RouterEngine`
- `routing_strategy`
  - 1 enum
    - `enum RoutingStrategy`
- `serde_helpers`
  - 2 fn
    - `fn serialize`
    - `fn deserialize`
- `transport`
  - 1 enum
    - `enum Transport` · 1 Methoden
- Zusätzlich 12 `pub`-Items in privaten Modulen ohne Re-Export (nicht Teil der echten API).
- ⚠ Re-Exporte **fremder Crates**: `contextra_adapt::bandit`, `contextra_adapt::lyapunov`, `contextra_adapt::offpolicy`

**Eingehend (wer nutzt dieses Crate)**

| Nutzer | Symbole prod | Symbole dev |
|---|---|---|
| `contextra-mcp` | `SlmProfile`×4, `DefaultRouterEngine`×2, `ports_local::HybridSearchProvider`, `ports_local::CommunityResolver`, `ports_local::ContextPreparer`, `RouterEngine::new` | `SlmProfile::new` |
| `contextra-agent` | `DefaultRouterEngine`, `DecisionId`, `RoutingOutcome::Success`, `RoutingOutcome::Rejected` | – |

**Nach außen ungenutzte öffentliche Items (Heuristik, 20 von 23)**: `ArmRegistry`, `ArmRegistryError`, `dispatch_to_slm`, `FcTsDispatchError`, `select_profile_fc_ts`, `deterministic_fc_ts_rng`, `DecisionIdGenerator`, `RoutingOutcome`, `QuantizationLevel`, `ProfileCalibrator`, `ConformalCalibrator`, `ProfileCalibrationState`, `ConfidenceMetrics`, `RoutingDecision`, `RouterState`, `RouterEngine`, `RoutingStrategy`, `serialize`, `deserialize`, `Transport`

**Auffälligkeiten**

- contextra-wire: kein Code-Zugriff gefunden (prüfen: Makro, Feature, cfg, oder entfernen)

### contextra-sandbox

- Pfad: `crates/contextra-sandbox` · Zeilen: 3980 · Rust-Dateien: 20 · Ring: 2
- Interne Cargo-Abhängigkeiten: keine

**Ausgehend (Importe aus anderen Workspace-Crates)**

Keine.

**Pro Komponente**

Keine Importe aus anderen Workspace-Crates.

**Öffentliche Schnittstelle** (erreichbar über `pub mod`-Kette oder `pub use`)

- `approval`
  - 3 enum, 1 fn, 1 struct
    - `enum ApprovalRisk`
    - `fn classify_risk`
    - `enum ApprovalStatus`
    - `enum ApprovalTransitionError`
    - `struct ApprovalRequest` · 7 Methoden
- `capabilities`
  - 2 struct
    - `struct WasmCapabilities` · 1 Methoden
    - `struct MergeOperatorCapabilities` · 2 Methoden
- `error`
  - 1 enum
    - `enum SandboxError`
- `executor`
  - 2 struct
    - `struct WasmExecutor` · 3 Methoden
    - `struct CapabilityViolationError` *(re-exportiert)*
- `merge`
  - 1 struct
    - `struct WasmMergeFunction` · 4 Methoden
- `output`
  - 1 struct
    - `struct WasmOutput` · 1 Methoden
- `wasi`
  - 2 struct
    - `struct ProcessExitError`
    - `struct OutputLimitExceededError`
- Zusätzlich 1 `pub`-Items in privaten Modulen ohne Re-Export (nicht Teil der echten API).

**Eingehend (wer nutzt dieses Crate)**

Wird von keinem anderen analysierten Workspace-Crate importiert.

**Nach außen ungenutzte öffentliche Items (Heuristik, 14 von 14)**: `ApprovalRisk`, `classify_risk`, `ApprovalStatus`, `ApprovalTransitionError`, `ApprovalRequest`, `WasmCapabilities`, `MergeOperatorCapabilities`, `SandboxError`, `WasmExecutor`, `CapabilityViolationError`, `WasmMergeFunction`, `WasmOutput`, `ProcessExitError`, `OutputLimitExceededError`

### contextra-simd

- Pfad: `crates/contextra-simd` · Zeilen: 5137 · Rust-Dateien: 17 · Ring: 0
- Interne Cargo-Abhängigkeiten: `contextra-types` (normal/optional via types-compat)

**Ausgehend (Importe aus anderen Workspace-Crates)**

| Ziel | prod | dev | Dateien (prod) | Symbole (prod) | Symbole (nur dev) |
|---|---:|---:|---:|---|---|
| `contextra-types` | 13 | 0 | 2 | `ContextraError::invalid_input`×3, `DistanceMetric`×3, `DistanceMetric::Cosine`×2, `DistanceMetric::Euclidean`×2, `DistanceMetric::DotProduct`×2, `ContextraError` | – |

**Pro Komponente**

- `error` (32 Zeilen)
  - → `contextra-types`: prod: `ContextraError::invalid_input`×3, `ContextraError`
- `metric` (29 Zeilen)
  - → `contextra-types`: prod: `DistanceMetric`×3, `DistanceMetric::Cosine`×2, `DistanceMetric::Euclidean`×2, `DistanceMetric::DotProduct`×2

**Öffentliche Schnittstelle** (erreichbar über `pub mod`-Kette oder `pub use`)

- `(crate-root)`
  - 4 fn
    - `fn validate_vector`
    - `fn compute_distance`
    - `fn compute_distance_trusted`
    - `fn compute_distance_f32_bytes_trusted`
- `dispatch`
  - 1 enum, 10 fn, 1 struct
    - `enum SimdTarget`
    - `struct Dispatcher` · 13 Methoden
    - `fn detect_best_target`
    - `fn cosine_distance`
    - `fn euclidean_distance`
    - `fn dot_product_distance`
    - `fn cosine_distance_f32_bytes`
    - `fn euclidean_distance_f32_bytes`
    - `fn dot_product_distance_f32_bytes`
    - `fn dot_product_u8`
    - `fn euclidean_distance_sq_u8`
    - `fn cosine_similarity_parts_u8`
- `error`
  - 1 enum, 2 type
    - `enum SimdError`
    - `type Result`
    - `type Error`
- `kernels`
  - 13 fn, 2 struct
    - `struct CosineSimilarityPartsU8`
    - `struct CosineSimilarityPartsF32U8`
    - `fn cosine_distance_scalar`
    - `fn euclidean_distance_scalar`
    - `fn dot_product_scalar`
    - `fn cosine_distance_f32_bytes_scalar`
    - `fn euclidean_distance_f32_bytes_scalar`
    - `fn dot_product_f32_bytes_scalar`
    - `fn dot_product_u8_scalar`
    - `fn euclidean_distance_sq_u8_scalar`
    - `fn cosine_similarity_parts_u8_scalar`
    - `fn dot_product_f32_u8`
    - `fn euclidean_distance_sq_f32_u8`
    - `fn cosine_similarity_parts_f32_u8`
    - `fn normalize_inplace`
- `metric`
  - 1 enum
    - `enum DistanceMetric` ⚠ fremd: `contextra-types`

**Eingehend (wer nutzt dieses Crate)**

Wird von keinem anderen analysierten Workspace-Crate importiert.

**Nach außen ungenutzte öffentliche Items (Heuristik, 35 von 35)**: `SimdTarget`, `Dispatcher`, `detect_best_target`, `cosine_distance`, `euclidean_distance`, `dot_product_distance`, `cosine_distance_f32_bytes`, `euclidean_distance_f32_bytes`, `dot_product_distance_f32_bytes`, `dot_product_u8`, `euclidean_distance_sq_u8`, `cosine_similarity_parts_u8`, `SimdError`, `Result`, `Error`, `CosineSimilarityPartsU8`, `CosineSimilarityPartsF32U8`, `cosine_distance_scalar`, `euclidean_distance_scalar`, `dot_product_scalar`, `cosine_distance_f32_bytes_scalar`, `euclidean_distance_f32_bytes_scalar`, `dot_product_f32_bytes_scalar`, `dot_product_u8_scalar`, `euclidean_distance_sq_u8_scalar`, `cosine_similarity_parts_u8_scalar`, `dot_product_f32_u8`, `euclidean_distance_sq_f32_u8`, `cosine_similarity_parts_f32_u8`, `normalize_inplace` …

### contextra-store

- Pfad: `crates/contextra-store` · Zeilen: 48330 · Rust-Dateien: 234 · Ring: 1
- Interne Cargo-Abhängigkeiten: `contextra-testkit` (dev), `contextra-crypto` (normal), `contextra-mvcc` (normal), `contextra-ports` (normal), `contextra-sys` (normal), `contextra-types` (normal)

**Ausgehend (Importe aus anderen Workspace-Crates)**

| Ziel | prod | dev | Dateien (prod) | Symbole (prod) | Symbole (nur dev) |
|---|---:|---:|---:|---|---|
| `contextra-ports` | 53 | 149 | 20 | `Clock`×10, `MetricsSink`×6, `MAX_SCAN_MERGE_ACCUMULATOR`×6, `StorageStats`×5, `scanner`×5, `Vfs`×3, `SystemClock::new`×3, `StorageEngine`×3, `IdGen`×3, `ResourceTracker`×2 … (+7) | `ResourceBudget`×28, `ResourceTracker::new`×27, `clock::SystemClock`, `storage::StorageEngine` |
| `contextra-types` | 37 | 123 | 15 | `ContextraError::Storage`×8, `Result`×5, `ContextraError`×5, `TOMBSTONE_BIT`×4, `TxId`×4, `ContextraError::MemoryBudgetExceeded`×2, `ContextraError::Internal`×2, `TxId::INTERNAL_BASE`×2, `ContextraError::Sandbox`, `ContextraError::DurabilityConfig` … (+3) | `CollectionId`×6, `DocId`×6, `ContextraError::InvalidInput`×2, `TenantId`, `*` |
| `contextra-mvcc` | 10 | 7 | 6 | `SsiValidator`×4, `tx_buffer::STAGING_ENTRY_OVERHEAD_BYTES`×2, `SnapshotRegistry`, `SequenceLogSsiValidator`, `IndexOp`, `SequenceLogSsiValidator::new_with_bounds` | `SnapshotRegistry::new` |
| `contextra-crypto` | 5 | 2 | 5 | `wal_crypto`×2, `verify_wal_chain_completeness`, `wal_crypto::WalHmac`, `crypto::KeyManager` | – |
| `contextra-sys` | 3 | 4 | 2 | `mmap_readonly`×2, `set_restrictive_file_acl` | `reopen_and_dup2`×2, `verify_file_acl_owner_only`, `set_fd_limit` |
| `contextra-testkit` | 0 | 8 | 0 | – | `ManualClock`×4, `ReferenceModel`×4 |

**Pro Komponente**

- `compaction` (3432 Zeilen)
  - → `contextra-mvcc`: prod: `SnapshotRegistry` · dev: `SnapshotRegistry`
  - → `contextra-ports`: prod: `StorageStats`×2, `ResourceTracker`×2, `Clock`×2, `Vfs`, `SystemClock::new`, `ContextraError::NotFound` · dev: `ResourceTracker::new`×27, `ResourceBudget`×27, `StorageEngine`×2
  - → `contextra-types`: prod: `ContextraError::MemoryBudgetExceeded`×2, `ContextraError::Internal`×2, `ContextraError::Sandbox`, `Result`, `TOMBSTONE_BIT` · dev: `TxId::new`×3, `TOMBSTONE_BIT`×2, `ContextraError::MemoryBudgetExceeded`×2, `TxId`
- `kv` (48 Zeilen)
  - → `contextra-types`: prod: `Result`
- `lsm` (6601 Zeilen)
  - → `contextra-crypto`: prod: `verify_wal_chain_completeness`
  - → `contextra-mvcc`: prod: `SsiValidator`×4, `tx_buffer::STAGING_ENTRY_OVERHEAD_BYTES`, `SequenceLogSsiValidator`, `IndexOp`, `SequenceLogSsiValidator::new_with_bounds` · dev: `tx_buffer::STAGING_ENTRY_OVERHEAD_BYTES`
  - → `contextra-ports`: prod: `MAX_SCAN_MERGE_ACCUMULATOR`×6, `Clock`×5, `MetricsSink`×5, `Vfs`×2, `StorageStats`×2, `StorageEngine`×2, `license::FeatureRing`, `license::FeatureRing::Sovereign`, `NoopMetricsSink` · dev: `Clock`
  - → `contextra-testkit`: dev: `ManualClock`
  - → `contextra-types`: prod: `ContextraError::Storage`×5, `TxId`×4, `ContextraError`, `ContextraError::DurabilityConfig`, `Result`, `TxId::new`, `ContextraError::NotFound`, `TxId::INTERNAL_BASE` · dev: `TOMBSTONE_BIT`
- `memtable` (969 Zeilen)
  - → `contextra-mvcc`: prod: `tx_buffer::STAGING_ENTRY_OVERHEAD_BYTES`
  - → `contextra-types`: dev: `TOMBSTONE_BIT`
- `scanner` (1203 Zeilen)
  - → `contextra-ports`: prod: `scanner`×5, `scanner::ScanTarget::Key`, `scanner::ScanTechnique::DirectLookup`, `SystemClock::new`, `StorageEngine` · dev: `clock::SystemClock`, `StorageEngine`, `scanner`
  - → `contextra-types`: prod: `ContextraError`×4, `ContextraError::Storage`×3, `Result`×2, `RequestContext::new_root` · dev: `CollectionId`×6, `DocId`×6, `ContextraError`
- `sstable` (3533 Zeilen)
  - → `contextra-ports`: prod: `VfsFile`
  - → `contextra-types`: prod: `TOMBSTONE_BIT`×3, `TxId::INTERNAL_BASE` · dev: `ContextraError`
- `system_pressure` (345 Zeilen)
  - → `contextra-ports`: prod: `Clock`×3, `SystemClock::new`, `MetricsSink`
- `tenant_codec` (837 Zeilen)
  - → `contextra-ports`: prod: `StorageStats`
- `wal` (7279 Zeilen)
  - → `contextra-crypto`: prod: `wal_crypto`×2, `wal_crypto::WalHmac`, `crypto::KeyManager` · dev: `wal_crypto::WalHmac`, `crypto::KeyManager`
  - → `contextra-ports`: prod: `IdGen`×3
  - → `contextra-sys`: prod: `mmap_readonly`×2, `set_restrictive_file_acl` · dev: `verify_file_acl_owner_only`
  - → `contextra-types`: dev: `TxId`×6
- `benches/lsm_concurrency_bench.rs` (129 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-types`: dev: `TxId`
- `benches/redb_comparative_bench.rs` (508 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-types`: dev: `TxId`
- `benches/wal_bench.rs` (143 Zeilen)
  - → `contextra-types`: dev: `TxId`
- `examples/chaos_writer.rs` (55 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
- `examples/store_standalone.rs` (36 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-types`: dev: `TxId`
- `fuzz/fuzz_targets/fuzz_compaction_interleave.rs` (110 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-types`: dev: `TxId`
- `fuzz/fuzz_targets/fuzz_recovery_arbitrary_state.rs` (52 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
- `fuzz/fuzz_targets/wal_mutation_chaos.rs` (270 Zeilen)
  - → `contextra-types`: dev: `TxId`
- `tests/adaptive_compaction_determinism.rs` (35 Zeilen)
  - → `contextra-ports`: dev: `StorageStats`
- `tests/adaptive_compaction_regression.rs` (35 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-types`: dev: `TxId`
- `tests/adaptive_compaction_tombstone.rs` (40 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-types`: dev: `TxId`
- `tests/amplification_benchmark.rs` (343 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-types`: dev: `TxId`
- `tests/blocking_util_pool_measurement.rs` (90 Zeilen)
  - → `contextra-ports`: dev: `MetricsSink`
- `tests/budget_drift.rs` (54 Zeilen)
  - → `contextra-mvcc`: dev: `tx_buffer::STAGING_ENTRY_OVERHEAD_BYTES`
- `tests/campaign_fuzzregress_memtable_concurrent.rs` (105 Zeilen)
  - → `contextra-types`: dev: `TxId`
- `tests/campaign_j11_memory_deletion.rs` (48 Zeilen)
  - → `contextra-ports`: dev: `storage::StorageEngine`
  - → `contextra-types`: dev: `TxId`
- `tests/campaign_j21_hypotheses.rs` (433 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
- `tests/campaign_j41_store.rs` (36 Zeilen)
  - → `contextra-types`: dev: `TenantId`
- `tests/campaign_prop_lsm_ops.rs` (407 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-types`: dev: `TxId`
- `tests/campaign_stress_store.rs` (294 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-types`: dev: `TxId`
- `tests/chaos_bitflip_sstable.rs` (405 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
- `tests/chaos_dropped_write.rs` (146 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-sys`: dev: `reopen_and_dup2`×2
- `tests/chaos_matrix.rs` (278 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
- `tests/chaos_memory_pressure.rs` (217 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
- `tests/chaos_power_cut.rs` (159 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-types`: dev: `ContextraError`
- `tests/chaos_task_massacre.rs` (163 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-types`: dev: `TxId`
- `tests/commit_cancel_chain_consistency.rs` (49 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-types`: dev: `TxId`
- `tests/commit_cancellation_atomicity.rs` (147 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-types`: dev: `TxId`
- `tests/commit_core_batch_rollback.rs` (43 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-types`: dev: `TxId`
- `tests/commit_core_cancel_safety.rs` (54 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-types`: dev: `TxId`
- `tests/commit_paths_equivalence.rs` (95 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-types`: dev: `TxId`
- `tests/commit_read_lock_concurrency.rs` (61 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-types`: dev: `TxId`
- `tests/commit_rollback_batch_scoped.rs` (97 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-types`: dev: `TxId`
- `tests/compaction_contiguous_selection.rs` (154 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`, `StorageStats`
  - → `contextra-types`: dev: `TxId`
- `tests/compaction_correctness_and_pinning.rs` (80 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
- `tests/compaction_key_range_selection.rs` (101 Zeilen)
  - → `contextra-mvcc`: dev: `SnapshotRegistry::new`
  - → `contextra-ports`: dev: `ResourceTracker`, `ResourceBudget`
- `tests/compaction_merge_operator.rs` (482 Zeilen)
  - → `contextra-mvcc`: dev: `SnapshotRegistry`
  - → `contextra-testkit`: dev: `ManualClock`
- `tests/compaction_noncontiguous_shadowing.rs` (78 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
- `tests/compaction_overlap_scan_and_tombstone.rs` (216 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-types`: dev: `TxId`
- `tests/crash_kill_durability.rs` (441 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-types`: dev: `TxId`
- `tests/crash_prefix_enumeration.rs` (133 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-testkit`: dev: `ReferenceModel`
  - → `contextra-types`: dev: `TxId`
- `tests/crash_recovery.rs` (111 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-types`: dev: `TxId`
- `tests/determinism_injection_test.rs` (131 Zeilen)
  - → `contextra-ports`: dev: `scanner`
  - → `contextra-types`: dev: `TxId`
- `tests/differential/redb_operation_sequence.rs` (853 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-types`: dev: `TxId`
- `tests/disk_full_during_write.rs` (76 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-types`: dev: `TxId`
- `tests/docid_128_kv_engine.rs` (64 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
- `tests/durability_mode_feature_gate.rs` (120 Zeilen)
  - → `contextra-ports`: dev: `license::FeatureRing`
- `tests/encryption_test.rs` (106 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-types`: dev: `TxId`
- `tests/fault_injection_recovery.rs` (120 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-types`: dev: `TxId`
- `tests/filename_uniqueness_test.rs` (88 Zeilen)
  - → `contextra-mvcc`: dev: `SnapshotRegistry`
  - → `contextra-types`: dev: `TxId`
- `tests/flush_crash_simulation.rs` (179 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
- `tests/flush_durability.rs` (64 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
- `tests/flush_mvcc_versions.rs` (184 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-types`: dev: `TxId`
- `tests/flush_recovery_lifecycle.rs` (186 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-types`: dev: `TxId`
- `tests/flush_threshold_boundary.rs` (68 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
- `tests/fsync_syscall_verification.rs` (62 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-types`: dev: `TxId`
- `tests/fsync_timeout_rollback.rs` (67 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-types`: dev: `TxId`
- `tests/group_commit_fault_injection.rs` (96 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
- `tests/group_commit_flush_race.rs` (171 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-types`: dev: `TxId`
- `tests/group_commit_follower_timeout_chain.rs` (69 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-types`: dev: `TxId`
- `tests/group_commit_leader_cancel_chain.rs` (65 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-types`: dev: `TxId`
- `tests/group_commit_overlapping_batch_rollback.rs` (73 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-types`: dev: `TxId`
- `tests/group_commit_stress.rs` (129 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-types`: dev: `TxId`
- `tests/group_commit_test.rs` (223 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-types`: dev: `ContextraError`
- `tests/group_commit_timeout.rs` (171 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
- `tests/lsm_robustness_fixes.rs` (139 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-types`: dev: `TxId`
- `tests/lsm_storage_merge_operator.rs` (112 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-types`: dev: `ContextraError::InvalidInput`×2
- `tests/manifest_compaction_audit.rs` (117 Zeilen)
  - → `contextra-mvcc`: dev: `SnapshotRegistry`
  - → `contextra-types`: dev: `TOMBSTONE_BIT`
- `tests/manifest_crash_no_resurrection.rs` (206 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
- `tests/manifest_replace_rank_compat.rs` (68 Zeilen)
  - → `contextra-types`: dev: `Result`
- `tests/memory_only_crash_semantics.rs` (50 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-types`: dev: `TxId`
- `tests/metrics_sink_test.rs` (278 Zeilen)
  - → `contextra-ports`: dev: `MetricsSink`, `StorageEngine`
  - → `contextra-types`: dev: `TxId`
- `tests/model_based_mvcc_flush.rs` (185 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-testkit`: dev: `ReferenceModel`
  - → `contextra-types`: dev: `TxId`
- `tests/observer_blocking_failopen.rs` (169 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-types`: dev: `TxId`
- `tests/observer_deadline_no_runtime_block.rs` (67 Zeilen)
  - → `contextra-types`: dev: `TxId`
- `tests/proptest_model_based.rs` (237 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-types`: dev: `TxId`, `Result`
- `tests/read_path_audit.rs` (210 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
- `tests/recovery_fd_exhaustion_regression.rs` (93 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-sys`: dev: `set_fd_limit`
  - → `contextra-types`: dev: `TxId`
- `tests/resource_leaks.rs` (88 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
- `tests/rollback_sstables.rs` (62 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-types`: dev: `TxId`
- `tests/scan_limits_and_ranges.rs` (59 Zeilen)
  - → `contextra-types`: dev: `TxId`
- `tests/spec_invariants_agents_sec4.rs` (236 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
- `tests/ssi_tracked_read_seq_clamp.rs` (122 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
- `tests/ssi_tracked_reads_via_trait.rs` (120 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
- `tests/ssi_unbounded_snapshot_seq.rs` (163 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
- `tests/ssi_write_skew_integration.rs` (274 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
- `tests/sstable_format_v3_to_v4_compat.rs` (158 Zeilen)
  - → `contextra-types`: dev: `TOMBSTONE_BIT`
- `tests/support/mod.rs` (79 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-testkit`: dev: `ReferenceModel`
- `tests/tenant_scoped_storage.rs` (131 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
- `tests/test_wp02_flush_mvcc_bug.rs` (39 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-testkit`: dev: `ReferenceModel`
  - → `contextra-types`: dev: `TxId`, `Result`
- `tests/toctou_put_if_absent.rs` (209 Zeilen)
  - → `contextra-types`: dev: `ContextraError`, `Result`, `*`
- `tests/tombstone_scan.rs` (120 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-types`: dev: `TxId`
- `tests/wal_backpressure.rs` (294 Zeilen)
  - → `contextra-types`: dev: `TxId`
- `tests/wal_flusher_rollback.rs` (102 Zeilen)
  - → `contextra-types`: dev: `Result`×2, `TxId`
- `tests/wal_hmac_chain_ordering.rs` (178 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-types`: dev: `TxId`
- `tests/wal_hmac_rollback_race.rs` (122 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-types`: dev: `TxId`
- `tests/wal_legacy_key_forced_migration.rs` (228 Zeilen)
  - → `contextra-types`: dev: `TxId`
- `tests/wal_legacy_key_migration_store.rs` (93 Zeilen)
  - → `contextra-types`: dev: `TxId`
- `tests/wal_legacy_key_retirement_marker.rs` (109 Zeilen)
  - → `contextra-types`: dev: `TxId`
- `tests/wal_legacy_migration_crash.rs` (44 Zeilen)
  - → `contextra-types`: dev: `TxId`
- `tests/wal_no_hmac_replay.rs` (52 Zeilen)
  - → `contextra-types`: dev: `TxId`
- `tests/wal_observer.rs` (178 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-types`: dev: `TxId`
- `tests/wal_observer_contract.rs` (150 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-testkit`: dev: `ManualClock`
  - → `contextra-types`: dev: `TxId`
- `tests/wal_observer_integration.rs` (269 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-testkit`: dev: `ManualClock`
  - → `contextra-types`: dev: `TxId`
- `tests/wal_observer_test.rs` (181 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-types`: dev: `TxId`
- `tests/wal_poison_causes.rs` (169 Zeilen)
  - → `contextra-types`: dev: `Result`×5, `TxId`
- `tests/wal_recovery_runtime_stall.rs` (116 Zeilen)
  - → `contextra-types`: dev: `TxId`
- `tests/wal_systematic_crash.rs` (199 Zeilen)
  - → `contextra-ports`: dev: `StorageEngine`
  - → `contextra-types`: dev: `ContextraError::Storage`×2, `TxId`
- `tests/wal_torn_write_fault_injection.rs` (113 Zeilen)
  - → `contextra-types`: dev: `TxId`
- `tests/wal_truncate_ordering.rs` (257 Zeilen)
  - → `contextra-types`: dev: `ContextraError`

**Öffentliche Schnittstelle** (erreichbar über `pub mod`-Kette oder `pub use`)

- `compaction`
  - 1 enum, 1 fn, 7 struct, 2 trait
    - `struct WorkloadMetrics` · 4 Methoden
    - `struct WorkloadMetricsSnapshot`
    - `enum CompactionStrategy`
    - `struct AdaptiveCompactionPlan`
    - `trait AdaptiveCompactionPlanner` · Methoden: plan_compaction
    - `struct CostBasedAdaptivePlanner` · 2 Methoden
    - `struct CompactionConfig` *(re-exportiert)*
    - `struct CompactionEngine` *(re-exportiert)* · 17 Methoden
    - `trait MergeOperator` · Methoden: merge
    - `fn retain_key_versions`
    - `struct TtlMetadata`
- `engine`
  - 1 struct
    - `struct InMemoryStorageEngine` · 1 Methoden
- `fsync`
  - 3 fn
    - `fn sync_file`
    - `fn sync_parent_dir`
    - `fn fsync_parent_dir`
- `io_seam`
  - 3 struct
    - `struct RealFs`
    - `struct RealFsFile` · 2 Methoden
    - `struct StoreIo` · 13 Methoden
- `key_source`
  - 2 struct, 1 trait
    - `trait KeyMaterialSource` · Methoden: fill_key_bytes
    - `struct OsKeyMaterialSource`
    - `struct StaticKeyMaterialSource` · 1 Methoden
- `lsm`
  - 17 const, 4 enum, 2 fn, 8 struct, 1 trait
    - `enum DurabilityMode` · 2 Methoden
    - `enum DurabilityConfigError`
    - `struct LsmConfig`
    - `enum StorageHealth` *(re-exportiert)*
    - `struct LsmStorage` *(re-exportiert)* · 32 Methoden ⚠ fremd: `contextra-mvcc`
    - `struct FlushPurgeFence` · 7 Methoden
    - `fn publish_visibility`
    - `fn load_snapshot_tx`
    - `const RANK_DOCUMENT_INTENT`
    - `const RANK_COMMIT_MUTEX`
    - `const RANK_FLUSH_MUTEX`
    - `const RANK_STATE_RWLOCK`
    - `const RANK_WAL_TRUNCATE`
    - `const RANK_SSTABLES_RWLOCK`
    - `const RANK_MEMTABLE_SHARDS`
    - `const RANK_MANIFEST_MUTEX`
    - `const RANK_OBSERVER_REGISTRY`
    - `const RANK_BLOCK_CACHE`
    - `const MAX_KEY_SIZE`
    - `const MAX_VALUE_SIZE`
    - `const MAX_BATCH_SIZE`
    - `const MAX_INTERNAL_MERGE_ENTRIES_FACTOR`
    - `const MAX_GROUP_COMMIT_BATCH_SIZE`
    - `const MIN_ENTRIES_FOR_SSTABLE_REBUILD`
    - `const DEFAULT_MAX_OBSERVER_LATENCY`
    - `enum WriteOrigin`
    - `struct WalEntryRef` · 6 Methoden
    - `struct CommitContext` ⚠ fremd: `contextra-types`
    - `struct CommittedBatch`
    - `trait WalObserver` · Methoden: requires_durability, on_commit
    - `struct ObserverRegistry` · 14 Methoden
    - `struct AsyncObserverAdapter` · 1 Methoden
- `manifest`
  - 2 const, 1 enum, 1 struct
    - `struct Manifest` *(re-exportiert)* · 9 Methoden
    - `enum ManifestEntry` *(re-exportiert)* · 2 Methoden
    - `const MAX_MANIFEST_ENTRY_SIZE`
    - `const DEFAULT_ROLLOVER_THRESHOLD_BYTES`
- `memtable`
  - 1 const, 2 fn, 1 struct
    - `fn is_tombstone`
    - `fn is_valid_range_bounds`
    - `const RESERVED_PREFIXES`
    - `struct MemTable` · 16 Methoden
- `scanner`
  - 6 fn, 7 struct
    - `struct MemtableScanner` · 2 Methoden
    - `struct ResidualInspectionReport` · 1 Methoden
    - `struct StoreInspector` · 4 Methoden
    - `fn lsm_scanners`
    - `fn lsm_scanners_with_handle`
    - `fn lsm_scanners_with_runtime`
    - `fn lsm_purgers`
    - `fn lsm_purgers_with_handle`
    - `fn lsm_purgers_with_runtime`
    - `struct LsmPurger` · 2 Methoden
    - `struct ScannerRuntime` · 5 Methoden
    - `struct SstableScanner` · 2 Methoden
    - `struct WalScanner` · 2 Methoden
- `sstable`
  - 3 const, 9 fn, 10 struct, 1 trait, 2 type
    - `const BLOCK_CACHE_SHARDS` *(re-exportiert)*
    - `struct SieveCacheBackend` *(re-exportiert)*
    - `trait BlockCacheBackend` *(re-exportiert)* · Methoden: new, get, insert, len, is_empty, contains…
    - `struct LruBlockCacheBackend` *(re-exportiert)*
    - `struct QuickCacheBlockCacheBackend` *(re-exportiert)*
    - `type BlockCacheShard` *(re-exportiert)*
- `system_pressure`
  - 6 const, 1 enum, 2 struct, 1 trait
- `tenant_codec`
  - 2 struct, 1 trait
- `util`
  - 1 struct, 2 trait
- `wal`
  - 5 const, 5 enum, 2 fn, 7 static, 6 struct, 1 trait, 1 type
  - … 62 weitere Items nicht aufgelistet (`--max-items`)
- Zusätzlich 29 `pub`-Items in privaten Modulen ohne Re-Export (nicht Teil der echten API).
- ⚠ Re-Exporte **fremder Crates**: `contextra_crypto::crypto::KeyManager`

**Eingehend (wer nutzt dieses Crate)**

| Nutzer | Symbole prod | Symbole dev |
|---|---|---|
| `contextra-engine` | `LsmStorage`×6, `SystemPressure`×3, `tenant_codec::TenantScopedStorage`×3, `tenant_codec::TenantScopedStorage::new`×3, `PressureLevel::Critical`×2, `PressureLevel::Normal`, `PressureLevel::Elevated`, `LsmConfig`, `lsm::DurabilityMode`, `MergeOperator` | `lsm`×10, `compaction`×2, `sstable`×2 |
| `contextra-db` | – | `LsmConfig`×5, `LsmStorage::new`×4, `LsmStorage`×2, `lsm` |

**Nach außen ungenutzte öffentliche Items (Heuristik, 136 von 142)**: `WorkloadMetrics`, `WorkloadMetricsSnapshot`, `CompactionStrategy`, `AdaptiveCompactionPlan`, `AdaptiveCompactionPlanner`, `CostBasedAdaptivePlanner`, `CompactionConfig`, `CompactionEngine`, `retain_key_versions`, `TtlMetadata`, `InMemoryStorageEngine`, `sync_file`, `sync_parent_dir`, `fsync_parent_dir`, `RealFs`, `RealFsFile`, `StoreIo`, `KeyMaterialSource`, `OsKeyMaterialSource`, `StaticKeyMaterialSource`, `DurabilityConfigError`, `StorageHealth`, `FlushPurgeFence`, `publish_visibility`, `load_snapshot_tx`, `RANK_DOCUMENT_INTENT`, `RANK_COMMIT_MUTEX`, `RANK_FLUSH_MUTEX`, `RANK_STATE_RWLOCK`, `RANK_WAL_TRUNCATE` …

### contextra-sys

- Pfad: `crates/contextra-sys` · Zeilen: 2321 · Rust-Dateien: 11 · Ring: 0
- Interne Cargo-Abhängigkeiten: keine

**Ausgehend (Importe aus anderen Workspace-Crates)**

Keine.

**Pro Komponente**

Keine Importe aus anderen Workspace-Crates.

**Öffentliche Schnittstelle** (erreichbar über `pub mod`-Kette oder `pub use`)

- `acl_win32`
  - 6 fn
    - `fn set_restrictive_file_acl`
    - `fn verify_file_acl_owner_only`
    - `fn set_restrictive_file_acl`
    - `fn verify_file_acl_owner_only`
    - `fn set_restrictive_file_acl`
    - `fn verify_file_acl_owner_only`
- `durability`
  - 3 fn, 1 struct
    - `fn durable_flush_file`
    - `fn durable_flush_dir`
    - `fn write_all_durable`
    - `struct DurableFile` · 6 Methoden
- `error`
  - 1 enum, 1 type
    - `enum Error`
    - `type Result`
- `mlock`
  - 1 enum, 5 fn
    - `enum MemLockPolicy`
    - `fn apply_dont_dump`
    - `fn mem_lock_typed`
    - `fn mem_lock_with_policy`
    - `fn mem_unlock`
    - `fn mem_lock`
- `mmap`
  - 1 fn, 4 struct
    - `fn mmap_readonly`
    - `struct SealedReadOnly`
    - `struct Unsealed`
    - `struct SealedFile` · 4 Methoden
    - `struct SealedMap` · 5 Methoden
- `posix`
  - 4 fn
    - `fn reopen_and_dup2`
    - `fn reopen_and_dup2`
    - `fn ensure_fd_limit`
    - `fn set_fd_limit`
- `vault`
  - 2 struct
    - `struct LockedRegions` · 7 Methoden
    - `struct VaultBuffer` · 8 Methoden

**Eingehend (wer nutzt dieses Crate)**

| Nutzer | Symbole prod | Symbole dev |
|---|---|---|
| `contextra-store` | `mmap_readonly`×2, `set_restrictive_file_acl` | `reopen_and_dup2`×2, `verify_file_acl_owner_only`, `set_fd_limit` |
| `contextra-vector` | `mmap_readonly`×2 | – |
| `contextra-bench` | – | `ensure_fd_limit` |

**Nach außen ungenutzte öffentliche Items (Heuristik, 18 von 29)**: `durable_flush_file`, `durable_flush_dir`, `write_all_durable`, `DurableFile`, `Error`, `Result`, `MemLockPolicy`, `apply_dont_dump`, `mem_lock_typed`, `mem_lock_with_policy`, `mem_unlock`, `mem_lock`, `SealedReadOnly`, `Unsealed`, `SealedFile`, `SealedMap`, `LockedRegions`, `VaultBuffer`

### contextra-testkit

- Pfad: `crates/contextra-testkit` · Zeilen: 4857 · Rust-Dateien: 24 · Ring: tooling
- Interne Cargo-Abhängigkeiten: `contextra-ports` (normal), `contextra-types` (normal)

**Ausgehend (Importe aus anderen Workspace-Crates)**

| Ziel | prod | dev | Dateien (prod) | Symbole (prod) | Symbole (nur dev) |
|---|---:|---:|---:|---|---|
| `contextra-types` | 20 | 10 | 2 | `ContextraError::Io`×9, `Result`×8, `ContextraError::NotFound`×2, `TxId` | – |
| `contextra-ports` | 14 | 4 | 11 | `Clock`×2, `Vfs`×2, `GraphIndex`, `MetricsSink`, `Rng`, `StorageEngine`, `TextIndex`, `VectorIndex`, `scanner::WitnessLayer`, `scanner::ScanTechnique` … (+2) | `TextEmbeddingEngine` |

**Pro Komponente**

- `(crate-root)` (125 Zeilen)
  - → `contextra-ports`: prod: `scanner::WitnessLayer`, `scanner::ScanTechnique`, `scanner::LayerWitness`, `scanner::mint_witness_for_testkit`
- `conformance` (1486 Zeilen)
  - → `contextra-ports`: prod: `Clock`, `GraphIndex`, `MetricsSink`, `Rng`, `StorageEngine`, `TextIndex`, `VectorIndex`, `Vfs`
  - → `contextra-types`: prod: `TxId`
- `fault_vfs` (410 Zeilen)
  - → `contextra-ports`: prod: `Vfs` · dev: `Vfs`×2
  - → `contextra-types`: prod: `ContextraError::Io`×9, `Result`×8, `ContextraError::NotFound`×2 · dev: `Result`
- `manual_clock` (150 Zeilen)
  - → `contextra-ports`: prod: `Clock`
- `mocks` (276 Zeilen)
  - → `contextra-ports`: dev: `TextEmbeddingEngine`
- `benches/testkit_bench.rs` (28 Zeilen)
  - → `contextra-ports`: dev: `Clock`
- `examples/standalone_testkit.rs` (49 Zeilen)
  - → `contextra-types`: dev: `TxId`
- `tests/conformance_selftest.rs` (476 Zeilen)
  - → `contextra-types`: dev: `Result`×8

**Öffentliche Schnittstelle** (erreichbar über `pub mod`-Kette oder `pub use`)

- `(crate-root)`
  - 1 fn
    - `fn mint_witness_for_tests` ⚠ fremd: `contextra-ports`
- `conformance`
  - 8 fn, 1 struct, 1 type
    - `fn run_clock_suite` ⚠ fremd: `contextra-ports`
    - `struct ConformanceFailure` · 1 Methoden
    - `type ConformanceResult`
    - `fn run_graph_index_suite` ⚠ fremd: `contextra-ports`
    - `fn run_metrics_sink_suite` ⚠ fremd: `contextra-ports`
    - `fn run_rng_suite` ⚠ fremd: `contextra-ports`
    - `fn run_storage_engine_suite` ⚠ fremd: `contextra-ports`
    - `fn run_text_index_suite` ⚠ fremd: `contextra-ports`
    - `fn run_vector_index_suite` ⚠ fremd: `contextra-ports`
    - `fn run_vfs_suite` ⚠ fremd: `contextra-ports`
- `fault_vfs`
  - 2 struct
    - `struct FaultConfig`
    - `struct FaultVfs` · 10 Methoden
- `in_memory_graph`
  - 1 struct
    - `struct InMemoryGraphIndex` · 1 Methoden
- `in_memory_store`
  - 1 struct
    - `struct InMemoryStorageEngine` · 1 Methoden
- `in_memory_text`
  - 1 struct
    - `struct InMemoryTextIndex` · 1 Methoden
- `in_memory_vector`
  - 1 struct
    - `struct InMemoryVectorIndex` · 1 Methoden
- `manual_clock`
  - 1 struct
    - `struct ManualClock` · 6 Methoden
- `mocks`
  - 1 enum, 2 struct
    - `struct MockEmbedder` · 2 Methoden
    - `enum MetricEvent`
    - `struct TestMetricsSink` · 3 Methoden
- `reference_model`
  - 1 enum, 1 struct
    - `enum RefOp`
    - `struct ReferenceModel` · 13 Methoden
- Interne Re-Exporte (`pub use`): `InMemoryGraphIndex`, `InMemoryStorageEngine`, `InMemoryTextIndex`, `InMemoryVectorIndex`, `ManualClock`, `*`

**Eingehend (wer nutzt dieses Crate)**

| Nutzer | Symbole prod | Symbole dev |
|---|---|---|
| `contextra-agent` | – | `InMemoryStorageEngine`×2, `TestMetricsSink` |
| `contextra-checkpoint` | – | `InMemoryStorageEngine`×2, `FaultVfs` |
| `contextra-cognition` | – | `ManualClock`, `MockEmbedder` |
| `contextra-crypto` | – | `mint_witness_for_tests`×4, `FaultVfs` |
| `contextra-engine` | – | `InMemoryStorageEngine` |
| `contextra-mvcc` | – | `ManualClock`×6 |
| `contextra-store` | – | `ManualClock`×4, `ReferenceModel`×4 |

**Nach außen ungenutzte öffentliche Items (Heuristik, 16 von 23)**: `run_clock_suite`, `ConformanceFailure`, `ConformanceResult`, `run_graph_index_suite`, `run_metrics_sink_suite`, `run_rng_suite`, `run_storage_engine_suite`, `run_text_index_suite`, `run_vector_index_suite`, `run_vfs_suite`, `FaultConfig`, `InMemoryGraphIndex`, `InMemoryTextIndex`, `InMemoryVectorIndex`, `MetricEvent`, `RefOp`

### contextra-text

- Pfad: `crates/contextra-text` · Zeilen: 15245 · Rust-Dateien: 62 · Ring: 0
- Interne Cargo-Abhängigkeiten: `contextra-ports` (normal/optional via types-compat), `contextra-types` (normal/optional via types-compat)

**Ausgehend (Importe aus anderen Workspace-Crates)**

| Ziel | prod | dev | Dateien (prod) | Symbole (prod) | Symbole (nur dev) |
|---|---:|---:|---:|---|---|
| `contextra-types` | 22 | 137 | 2 | `ContextraError`×3, `ContextraError::InvalidInput`×3, `ContextraError::Storage`×3, `DocId`×3, `ScoredDocument`×3, `ContextraError::Text`×2, `ContextraError::Io`×2, `DocId::new`×2, `ContextraError::Internal` | `Result`×67, `TOMBSTONE_BIT`×57, `TxId`×7, `TxId::new`×2, `ContextraError::CapabilityUnsupported`×2 … (+1) |
| `contextra-ports` | 8 | 87 | 5 | `StorageEngine`×6, `scanner`×2 | `BoxFuture`×50, `StorageStats`×35, `clock::SystemClock` |

**Pro Komponente**

- `(crate-root)` (322 Zeilen)
  - → `contextra-ports`: prod: `StorageEngine`×3 · dev: `BoxFuture`
- `error` (65 Zeilen)
  - → `contextra-types`: prod: `ContextraError`×3, `ContextraError::InvalidInput`×3, `ContextraError::Storage`×3, `ContextraError::Text`×2, `ContextraError::Io`×2, `ContextraError::Internal`
- `inverted` (2866 Zeilen)
  - → `contextra-ports`: prod: `scanner`×2, `StorageEngine` · dev: `StorageStats`×3
  - → `contextra-types`: dev: `TOMBSTONE_BIT`×6, `ContextChunk`
- `stream` (162 Zeilen)
  - → `contextra-ports`: prod: `StorageEngine`
- `types` (114 Zeilen)
  - → `contextra-types`: prod: `DocId`×3, `ScoredDocument`×3, `DocId::new`×2
- `wand` (814 Zeilen)
  - → `contextra-ports`: prod: `StorageEngine` · dev: `StorageStats`×2, `BoxFuture`
  - → `contextra-types`: dev: `TxId`×7, `TOMBSTONE_BIT`×6, `TxId::new`×2
- `tests/alloc_profiler.rs` (569 Zeilen)
  - → `contextra-ports`: dev: `StorageStats`×2
  - → `contextra-types`: dev: `Result`×16
- `tests/audit_inspection.rs` (578 Zeilen)
  - → `contextra-ports`: dev: `BoxFuture`×32, `StorageStats`×4
  - → `contextra-types`: dev: `Result`×34, `TOMBSTONE_BIT`×10
- `tests/bm25_candidate_stream.rs` (321 Zeilen)
  - → `contextra-ports`: dev: `StorageStats`×2
- `tests/concurrent_metadata.rs` (196 Zeilen)
  - → `contextra-ports`: dev: `StorageStats`×2
- `tests/docid_128_tombstone_resolution.rs` (249 Zeilen)
  - → `contextra-ports`: dev: `BoxFuture`×16, `StorageStats`×2
  - → `contextra-types`: dev: `Result`×17, `TOMBSTONE_BIT`×6
- `tests/pipeline_identity_regression.rs` (271 Zeilen)
  - → `contextra-ports`: dev: `StorageStats`×2
  - → `contextra-types`: dev: `TOMBSTONE_BIT`×6
- `tests/posting_roundtrip_tests.rs` (122 Zeilen)
  - → `contextra-types`: dev: `DocId`
- `tests/proptest_bm25_invariants.rs` (335 Zeilen)
  - → `contextra-ports`: dev: `StorageStats`×2
  - → `contextra-types`: dev: `TOMBSTONE_BIT`×6
- `tests/text_scanner_purger_tests.rs` (303 Zeilen)
  - → `contextra-ports`: dev: `StorageStats`×2, `clock::SystemClock`, `scanner`
- `tests/text_search_at_wrappers.rs` (331 Zeilen)
  - → `contextra-ports`: dev: `StorageStats`×2
  - → `contextra-types`: dev: `TOMBSTONE_BIT`×6, `ContextraError::CapabilityUnsupported`×2
- `tests/tombstone_read_your_writes.rs` (267 Zeilen)
  - → `contextra-ports`: dev: `StorageStats`×2
  - → `contextra-types`: dev: `TOMBSTONE_BIT`×6
- `tests/tombstone_update.rs` (221 Zeilen)
  - → `contextra-ports`: dev: `StorageStats`×2
- `tests/wand_differential.rs` (430 Zeilen)
  - → `contextra-ports`: dev: `StorageStats`×2
  - → `contextra-types`: dev: `TOMBSTONE_BIT`×5
- `tests/wand_tie_break_threshold.rs` (230 Zeilen)
  - → `contextra-ports`: dev: `StorageStats`×2
- `tests/write_amplification.rs` (219 Zeilen)
  - → `contextra-ports`: dev: `StorageStats`×2

**Öffentliche Schnittstelle** (erreichbar über `pub mod`-Kette oder `pub use`)

- `(crate-root)`
  - 1 struct
    - `struct Bm25Scorer` · 1 Methoden ⚠ fremd: `contextra-ports`
- `bm25`
  - 2 const, 3 fn, 3 struct, 1 type
    - `const BM25_K1`
    - `const BM25_B`
    - `struct BM25` · 2 Methoden
    - `fn score_term`
    - `fn score_term_with_params`
    - `type FieldId`
    - `struct FieldWeight` · 1 Methoden
    - `struct BM25F` · 2 Methoden
    - `fn score_term_bm25f`
- `contract`
  - 1 enum, 2 struct, 2 trait, 1 type
    - `enum Error`
    - `type Result`
    - `trait Clock` · Methoden: now_unix_ms
    - `trait Rng` · Methoden: next_u64
    - `struct DocKey`
    - `struct Epoch`
- `domain`
  - 4 fn, 4 static, 2 struct, 1 trait
    - `struct LegalDomainVocabulary`
    - `fn get_legal_compound_stems`
    - `fn get_legal_protected_terms`
    - `static LEGAL_COMPOUND_STEMS`
    - `static LEGAL_PROTECTED_TERMS`
    - `struct MedicalDomainVocabulary`
    - `fn get_medical_compound_stems`
    - `fn get_medical_protected_terms`
    - `static MEDICAL_COMPOUND_STEMS`
    - `static MEDICAL_PROTECTED_TERMS`
    - `trait DomainVocabulary` · Methoden: compound_stems, protected_terms, domain_id
- `error`
  - 1 enum, 1 type
    - `enum TextError`
    - `type Result`
- `inverted`
  - 1 enum, 2 fn, 5 struct
    - `struct InvertedIndex` *(re-exportiert)* · 14 Methoden
    - `struct BM25MorphIndex` *(re-exportiert)* · 2 Methoden
    - `struct PurgeReport` · 3 Methoden
    - `struct TextPurger` · 5 Methoden ⚠ fremd: `contextra-ports`
    - `fn text_purger` ⚠ fremd: `contextra-ports`
    - `enum ParsedTextTarget` · 1 Methoden
    - `struct TextScanner` · 4 Methoden
    - `fn text_scanner`
- `memory_index`
  - 3 const, 4 struct
    - `const MAX_DOC_BYTES`
    - `const MAX_DOC_TERMS`
    - `const MAX_POSTINGS_PER_TERM`
    - `struct CompactStats`
    - `struct KeyInspection`
    - `struct IndexSnapshot` · 1 Methoden
    - `struct MemoryTextIndex` · 9 Methoden
- `morphology`
  - 2 struct
    - `struct TokenReductionMetrics` *(re-exportiert)* · 1 Methoden
    - `struct PassthroughTokenizer` *(re-exportiert)* · 1 Methoden
- `posting_list`
  - 2 const, 4 struct, 2 type
    - `const POSTING_LIST_V2_MAGIC`
    - `const BLOCK_SIZE`
    - `type DocIdRaw`
    - `type DocIdRaw`
    - `struct PostingBlockInfo`
    - `struct Posting` · 2 Methoden
    - `struct PostingList` · 10 Methoden
    - `struct ResidentPostingIndex` · 7 Methoden
- `stream`
  - 1 const, 1 struct
    - `const DEFAULT_STREAM_BATCH_SIZE`
    - `struct Bm25CandidateStream` · 7 Methoden ⚠ fremd: `contextra-ports`
- `tokenizer`
  - 2 enum, 3 fn, 5 struct, 3 trait
    - `fn compile_protected_regex`
    - `fn get_protected_regex`
    - `trait Tokenizer` · Methoden: tokenize
    - `struct DefaultTokenizer` · 2 Methoden
    - `struct GermanMorphTokenizer` · 2 Methoden
    - `fn tokenize`
    - `struct Token` · 1 Methoden
    - `enum OverflowPolicy`
    - `struct PipelineLimits`
    - `enum NormalizationPolicy` · 1 Methoden
    - `trait TokenFilter` · Methoden: filter, filter_id
    - `trait Analyzer` · Methoden: analyze, fingerprint
    - `struct AnalyzerPipeline` · 8 Methoden
- `traits`
  - 2 trait
    - `trait Index` · Methoden: insert, remove, search
    - `trait Maintain` · Methoden: compact, vacuum
- `types`
  - 1 const, 3 struct
    - `struct DocId` ⚠ fremd: `contextra-types`
    - `struct DocId` · 5 Methoden ⚠ fremd: `contextra-types`
    - `struct TextHit` ⚠ fremd: `contextra-types`
    - `const MAX_SEARCH_K`
- `wand`
  - 1 fn, 2 struct
    - `struct BoundedTopK` · 4 Methoden
    - `struct WandSearchResult`
    - `fn block_max_wand_search` ⚠ fremd: `contextra-ports`
- Zusätzlich 9 `pub`-Items in privaten Modulen ohne Re-Export (nicht Teil der echten API).

**Eingehend (wer nutzt dieses Crate)**

| Nutzer | Symbole prod | Symbole dev |
|---|---|---|
| `contextra-engine` | `inverted::InvertedIndex`×2, `Language`×2 | `Language::English`×63, `Language::German`×4 |
| `contextra-agent` | – | `Language::English`×5 |
| `contextra-db` | – | `Language::English`×7, `Language`×2, `bm25::score_term`, `inverted::InvertedIndex` |

**Nach außen ungenutzte öffentliche Items (Heuristik, 76 von 78)**: `BM25_K1`, `BM25_B`, `BM25`, `score_term_with_params`, `FieldId`, `FieldWeight`, `BM25F`, `score_term_bm25f`, `Error`, `Result`, `Clock`, `Rng`, `DocKey`, `Epoch`, `LegalDomainVocabulary`, `get_legal_compound_stems`, `get_legal_protected_terms`, `LEGAL_COMPOUND_STEMS`, `LEGAL_PROTECTED_TERMS`, `MedicalDomainVocabulary`, `get_medical_compound_stems`, `get_medical_protected_terms`, `MEDICAL_COMPOUND_STEMS`, `MEDICAL_PROTECTED_TERMS`, `DomainVocabulary`, `TextError`, `Result`, `BM25MorphIndex`, `PurgeReport`, `TextPurger` …

### contextra-types

- Pfad: `crates/contextra-types` · Zeilen: 8885 · Rust-Dateien: 38 · Ring: 0
- Interne Cargo-Abhängigkeiten: keine

**Ausgehend (Importe aus anderen Workspace-Crates)**

Keine.

**Pro Komponente**

Keine Importe aus anderen Workspace-Crates.

**Öffentliche Schnittstelle** (erreichbar über `pub mod`-Kette oder `pub use`)

- `erasure`
  - 14 enum, 10 struct
    - `enum EraseTarget`
    - `enum Timing`
    - `enum EraseMode`
    - `enum RetentionCode`
    - `struct RetentionAttestation`
    - `struct EraseRequest`
    - `struct EraseReceipt`
    - `struct LayerId` · 2 Methoden
    - `enum PendingReason`
    - `enum RefusalReason`
    - `enum Phase`
    - `enum ErasureStatus`
    - `enum KeyCoverage`
    - `enum GateVariant`
    - `enum MeasureClass`
    - `enum ScanMethod` · 1 Methoden
    - `enum RiskCondition`
    - `enum ResidualRisk` · 2 Methoden
    - `struct Unarmed`
    - `struct Armed`
    - `struct ControlEvidence` · 1 Methoden
    - `struct Residue`
    - `struct Scanner` · 4 Methoden
    - `struct LayerWitness` · 8 Methoden
- `error`
  - 3 enum, 1 struct, 1 type
    - `enum HnswDeletionError`
    - `struct StepId` · 2 Methoden
    - `enum CacheDirective`
    - `type Result`
    - `enum ContextraError` · 14 Methoden
- `error_dto`
  - 1 struct
    - `struct ContextraErrorDto` · 2 Methoden
- `ids`
  - 1 const, 7 struct
    - `const MAX_REF_LEN`
    - `struct DocRef`
    - `struct ErasureId` · 2 Methoden
    - `struct FileId` · 2 Methoden
    - `struct StoreId` · 2 Methoden
    - `struct GateId` · 2 Methoden
    - `struct Seq` · 2 Methoden
    - `struct Timestamp` · 2 Methoden
- `model_fingerprint`
  - 1 struct
    - `struct ModelFingerprint` · 1 Methoden
- `retrieval_strategy`
  - 1 enum
    - `enum RetrievalStrategy`
- `schema`
  - 2 enum
    - `enum DocIdWidth` · 2 Methoden
    - `enum ManifestSchemaVersion` · 4 Methoden
- `tenant_scope`
  - 1 enum, 1 struct
    - `struct TenantScoped` · 4 Methoden
    - `enum TenantScopeViolation`
- `tombstone`
  - 1 struct, 1 trait
    - `trait TombstoneSemanticsCheck` · Methoden: is_tombstone, make_tombstone
    - `struct SeqBitTombstone`
- `types`
  - 3 const, 12 enum, 2 fn, 30 struct, 2 type
    - `enum BudgetStrategy`
    - `struct TokenBudget` · 11 Methoden
    - `struct Reservation` · 1 Methoden
    - `struct ResourceBudget`
    - `struct ResourceTracker` · 6 Methoden
    - `enum DistanceMetric` *(re-exportiert)* · 2 Methoden
    - `struct Embedding` *(re-exportiert)* · 5 Methoden
    - `struct ScoredDocument` *(re-exportiert)* · 1 Methoden
    - `struct RerankResult` *(re-exportiert)*
    - `struct TenantId` *(re-exportiert)* · 5 Methoden
    - `struct CollectionId` *(re-exportiert)* · 4 Methoden
    - `struct DocId` *(re-exportiert)*
    - `struct DocId` *(re-exportiert)* · 8 Methoden
    - `struct EntityId` *(re-exportiert)* · 8 Methoden
    - `struct TxId` *(re-exportiert)* · 6 Methoden
    - `fn serialize` *(re-exportiert)*
    - `fn deserialize` *(re-exportiert)*
    - `struct WorkflowState` *(re-exportiert)*
    - `const TOMBSTONE_BIT` *(re-exportiert)*
    - `const EXPIRY_METADATA_KEY` *(re-exportiert)*
    - `const MAX_SEARCH_K` *(re-exportiert)*
    - `enum LinkRelation` *(re-exportiert)*
    - `struct MemoryLink` *(re-exportiert)*
    - `struct Entity` *(re-exportiert)* · 2 Methoden
    - `struct Edge` *(re-exportiert)* · 7 Methoden
    - `enum MemoryType` *(re-exportiert)* · 3 Methoden
    - `struct TlHfdParams` *(re-exportiert)*
    - `enum PprAlgorithm` *(re-exportiert)*
    - `struct PprConfig` *(re-exportiert)*
    - `struct ConfigFingerprint` *(re-exportiert)* · 4 Methoden
    - `enum AutoExtractionMode` *(re-exportiert)* · 1 Methoden
    - `enum FilterExpr` · 1 Methoden
    - `struct ImportanceScore` · 2 Methoden
    - `enum DecayFunction` · 2 Methoden
- `witness`
  - 1 enum, 2 struct
  - … 18 weitere Items nicht aufgelistet (`--max-items`)
- Zusätzlich 1 `pub`-Items in privaten Modulen ohne Re-Export (nicht Teil der echten API).
- Interne Re-Exporte (`pub use`): `TenantScoped`

**Eingehend (wer nutzt dieses Crate)**

| Nutzer | Symbole prod | Symbole dev |
|---|---|---|
| `contextra-engine` | `ContextraError::invalid_input`×50, `Result`×18, `EntityId`×15, `HybridQuery`×13, `ContextraError::Internal`×11, `MAX_SEARCH_K`×11, `ContextraError::Serialization`×8, `ImportanceScore::new`×8, `domain::MemoryLink`×7, `EntityId::from_key`×7 … (+40) | `ContextraError::from`×10, `domain::LinkRelation::Elaborates`×2, `ContextraError::HnswConnectivityDegraded`, `FusionWeights::new` … (+8) |
| `contextra-vector` | `ContextraError::invalid_input`×21, `MAX_SEARCH_K`×15, `Result`×9, `ContextraError::Index`×7, `DocId`×6, `ContextraError::HnswConnectivityDegraded`×5, `DistanceMetric`×5, `TxId`×4, `ScoredDocument`×4, `ContextraError`×3 … (+11) | `TxId::new` |
| `contextra-graph` | `EntityId`×18, `ContextraError::InvalidInput`×5, `Result`×5, `ContextraError`×3, `TxId`×3, `Edge`×2, `Edge::new`×2, `PprConfig`×2, `ContextraError::Internal`×2, `MemoryLink` … (+3) | `Entity::new`×10, `TlHfdParams::default`×4, `PprAlgorithm::Auto`×4, `Entity`×2 … (+1) |
| `contextra-rank` | `DocId`×5, `SignalFusionStrategies`×4, `ContextraError::invalid_input`×3, `FusionWeights`×3, `FusionStrategy`×3, `OnSignalFailure`×3, `ContextraError`×2, `ContextChunk`×2, `ConfigFingerprint`×2, `FusionStrategy::Rrf`×2 … (+6) | – |
| `contextra-store` | `ContextraError::Storage`×8, `Result`×5, `ContextraError`×5, `TOMBSTONE_BIT`×4, `TxId`×4, `ContextraError::MemoryBudgetExceeded`×2, `ContextraError::Internal`×2, `TxId::INTERNAL_BASE`×2, `ContextraError::Sandbox`, `ContextraError::DurabilityConfig` … (+3) | `CollectionId`×6, `DocId`×6, `ContextraError::InvalidInput`×2, `TenantId` … (+1) |
| `contextra-crypto` | `TenantId`×8, `Result`×7, `ScanMethod::FeatureDisabled`×3, `ContextraError::Crypto`×3, `TenantScoped`×2, `ScanMethod::Structural`×2, `ScanMethod::NotMeasured`×2, `ContextraError`×2, `RequestContext`, `SagaRecord` … (+4) | `ContextraError::Io`×4, `ContextraError::NotFound` |
| `contextra-infer-ollama` | `Result`×6, `ContextraError::Internal`×5, `ContextraError`×3, `ContextraError::Io`×3, `QueryRewriteOutput`×3, `ImportanceScore`×3, `ContextraError::InvalidInput`×2, `ContextraError::NotFound`×2, `ContextraError::Storage`×2, `ContextraError::PolicyViolation`×2 … (+3) | – |
| `contextra-agent` | `ContextraError::InvalidInput`×10, `Result`×4, `TxId`×3, `WorkflowState`×3, `TokenBudget::new`×2, `ContextraError::Internal`, `DocId::from_key`, `types::budget`, `error` | `TokenBudget`×13, `TxId::new`×5, `ContextraError::Conflict`, `ContextraError::Serialization` … (+1) |
| `contextra-text` | `ContextraError`×3, `ContextraError::InvalidInput`×3, `ContextraError::Storage`×3, `DocId`×3, `ScoredDocument`×3, `ContextraError::Text`×2, `ContextraError::Io`×2, `DocId::new`×2, `ContextraError::Internal` | `Result`×67, `TOMBSTONE_BIT`×57, `TxId`×7, `TxId::new`×2 … (+2) |
| `contextra-testkit` | `ContextraError::Io`×9, `Result`×8, `ContextraError::NotFound`×2, `TxId` | – |
| `contextra-cognition` | `ContextraError::Internal`×3, `Result`×3, `TenantId`×3, `ContextraError::InvalidInput`×3, `ContextChunk`×2, `ContextraError::PolicyViolation`×2, `TxId`×2, `(crate)` | `MemoryType::Episodic`×6, `DocId::new`×5, `DocId` |
| `contextra-kvcache` | `TenantId`×6, `RequestContext`×2, `Result`×2, `TenantId::try_new`, `TenantId::SYSTEM`, `model_fingerprint::ModelFingerprint`, `model_fingerprint::ModelFingerprint::new`, `ContextraError`, `ContextraError::Internal`, `error::Result` … (+1) | – |
| `contextra-simd` | `ContextraError::invalid_input`×3, `DistanceMetric`×3, `DistanceMetric::Cosine`×2, `DistanceMetric::Euclidean`×2, `DistanceMetric::DotProduct`×2, `ContextraError` | – |
| `contextra` | `error::ContextraError`×4, `DistanceMetric`×2, `types::domain::AutoExtractionMode`, `types::domain`, `ContextraError`, `ContextraError::DurabilityConfig`, `error::Result`, `ScoredEntry` | `DocId`×2, `DocId::from_u64`×2, `ContextraError::InvalidInput`, `ScoredDocument` … (+1) |
| `contextra-bench` | `ContextraError::Internal`×3, `Result`×2, `ContextraError::SnapshotUnsupportedForSignal`×2, `ContextraError::CapabilityUnsupported` | `TxId` |
| `contextra-infer-candle` | `ContextraError`×4, `DocId::from`, `ModelFingerprint` | `DocId`×3, `ContextChunk`×2, `Result`×2, `TenantId` |
| `contextra-mcp` | `ContextraError`×4, `ContextraError::InvalidInput`, `ContextraError::NotFound` | `TenantId::try_new`, `AutoExtractionMode`, `TokenBudget::new`, `TenantId` … (+1) |
| `contextra-py` | `FusionWeights::new`×2, `ContextraError`, `ContextraErrorDto::from`, `TenantId::SYSTEM` | – |
| `contextra-audit-export` | `TenantId`×4 | – |
| `contextra-db` | `DistanceMetric`, `MAX_SEARCH_K`, `EntityId`, `EntityId::from_key` | `Result`×6, `TxId`×6, `DocId`×4, `FusionWeights`×3 … (+16) |
| `contextra-license` | `TenantId`×4 | – |
| `contextra-ports` | `RequestContext`×2, `ids::ErasureId` | – |
| `contextra-router` | `RetrievalStrategy`, `TokenBudget::default` | `DocId::new`×8, `ConfigFingerprint`×7, `TokenBudget`×4, `Result` |
| `contextra-adapt` | – | `TenantId` |
| `contextra-checkpoint` | – | `TxId`×4, `Result`×3, `ContextraError`×2, `ContextraError::Internal`×2 … (+2) |
| `contextra-mvcc` | – | `TxId`×5, `DocId`, `types` |

*Hinweis: Mindestens ein Crate importiert per Glob (`*`); ungenutzte Items sind deshalb nicht bestimmbar.*

### contextra-vector

- Pfad: `crates/contextra-vector` · Zeilen: 25229 · Rust-Dateien: 127 · Ring: 0
- Interne Cargo-Abhängigkeiten: `contextra-mvcc` (normal/optional via ports-compat), `contextra-ports` (normal/optional via ports-compat), `contextra-sys` (normal/optional via ports-compat), `contextra-types` (normal/optional via ports-compat)

**Ausgehend (Importe aus anderen Workspace-Crates)**

| Ziel | prod | dev | Dateien (prod) | Symbole (prod) | Symbole (nur dev) |
|---|---:|---:|---:|---|---|
| `contextra-types` | 97 | 40 | 16 | `ContextraError::invalid_input`×21, `MAX_SEARCH_K`×15, `Result`×9, `ContextraError::Index`×7, `DocId`×6, `ContextraError::HnswConnectivityDegraded`×5, `DistanceMetric`×5, `TxId`×4, `ScoredDocument`×4, `ContextraError`×3 … (+11) | `TxId::new` |
| `contextra-ports` | 34 | 78 | 7 | `Clock`×15, `MetricsSink`×6, `Rng`×4, `SeededRng::new`×3, `VectorIndex`×2, `scanner`×2, `NoopMetricsSink`×2 | `SeededRng`×5, `vector_index::VectorIndex`, `clock::SystemClock` |
| `contextra-mvcc` | 11 | 0 | 3 | `IndexOp::Insert`×2, `IndexOp::Delete`×2, `SequenceLog::new`×2, `SeqLogChange::Insert`, `SeqLogChange::Delete`, `TxBuffer`, `TxBuffer::new_with_config`, `SequenceLog` | – |
| `contextra-sys` | 2 | 0 | 2 | `mmap_readonly`×2 | – |

**Pro Komponente**

- `acorn` (180 Zeilen)
  - → `contextra-types`: prod: `DocId`
- `candidate_stream` (147 Zeilen)
  - → `contextra-ports`: prod: `VectorIndex`
- `diskann` (3661 Zeilen)
  - → `contextra-ports`: prod: `VectorIndex` · dev: `VectorIndex`×2
  - → `contextra-sys`: prod: `mmap_readonly`
  - → `contextra-types`: prod: `MAX_SEARCH_K`×9
- `distance` (617 Zeilen)
  - → `contextra-types`: dev: `DistanceMetric`
- `error` (97 Zeilen)
  - → `contextra-types`: prod: `ContextraError`×3, `ContextraError::Index`×3, `ContextraError::invalid_input`×2, `ContextraError::HnswConnectivityDegraded`×2, `ContextraError::GraphRepairFailed`×2, `ContextraError::Io`×2, `HnswDeletionError::VerificationFailed`, `ContextraError::capability_unsupported`, `ContextraError::InvalidInput`, `ContextraError::CapabilityUnsupported`
- `hnsw` (6368 Zeilen)
  - → `contextra-mvcc`: prod: `IndexOp::Insert`×2, `IndexOp::Delete`×2, `SequenceLog::new`×2, `SeqLogChange::Insert`, `SeqLogChange::Delete`, `TxBuffer`, `TxBuffer::new_with_config`, `SequenceLog`
  - → `contextra-ports`: prod: `Clock`×15, `MetricsSink`×6, `Rng`×4, `SeededRng::new`×3, `scanner`×2, `NoopMetricsSink`×2 · dev: `Rng`×3, `SeededRng::new`×3, `MetricsSink`×2, `VectorIndex`
  - → `contextra-types`: prod: `MAX_SEARCH_K`×6, `ContextraError::Index`×4, `Result`×3, `ContextraError::HnswConnectivityDegraded`×3, `DocId`×2, `ContextraError::Conflict`×2, `DistanceMetric` · dev: `DistanceMetric`
- `persistence` (1140 Zeilen)
  - → `contextra-ports`: dev: `VectorIndex`×3
  - → `contextra-sys`: prod: `mmap_readonly`
  - → `contextra-types`: dev: `Result`×2
- `quantize` (1022 Zeilen)
  - → `contextra-types`: prod: `ContextraError::invalid_input`×19, `Result`×6, `DistanceMetric` · dev: `ContextraError::InvalidInput`×6, `Result`×3
- `types` (307 Zeilen)
  - → `contextra-types`: prod: `TxId`×4, `ScoredDocument`×4, `DocId`×3, `DistanceMetric`×3, `DocId::new`×2, `DistanceMetric::Cosine`×2, `DistanceMetric::Euclidean`×2, `DistanceMetric::DotProduct`×2
- `benches/acorn_vs_post_filter.rs` (53 Zeilen)
  - → `contextra-types`: dev: `DocId`
- `benches/audit_benchmarks.rs` (430 Zeilen)
  - → `contextra-ports`: dev: `VectorIndex`
- `benches/distance_nan_impact_bench.rs` (24 Zeilen)
  - → `contextra-types`: dev: `DistanceMetric`
- `benches/filtered_search_selectivity_bench.rs` (166 Zeilen)
  - → `contextra-ports`: dev: `VectorIndex`
- `benches/flush_threshold_amplification.rs` (177 Zeilen)
  - → `contextra-ports`: dev: `VectorIndex`
- `benches/hnsw_bench.rs` (90 Zeilen)
  - → `contextra-ports`: dev: `VectorIndex`
- `benches/hnsw_delete_search_bench.rs` (86 Zeilen)
  - → `contextra-ports`: dev: `VectorIndex`
- `benches/overfetch_selectivity.rs` (79 Zeilen)
  - → `contextra-ports`: dev: `SeededRng`, `VectorIndex`
- `benches/partial_rebuild_recall_bench.rs` (136 Zeilen)
  - → `contextra-ports`: dev: `VectorIndex`
- `benches/sq8_bench.rs` (40 Zeilen)
  - → `contextra-types`: dev: `DistanceMetric`
- `examples/standalone_vector.rs` (41 Zeilen)
  - → `contextra-ports`: dev: `VectorIndex`
- `fuzz/fuzz_targets/fuzz_hnsw_persistence.rs` (119 Zeilen)
  - → `contextra-ports`: dev: `VectorIndex`
- `fuzz/fuzz_targets/hnsw_insert_search.rs` (79 Zeilen)
  - → `contextra-ports`: dev: `VectorIndex`
- `tests/acorn_hnsw_differential.rs` (253 Zeilen)
  - → `contextra-ports`: dev: `VectorIndex`
- `tests/acorn_naive_recall.rs` (70 Zeilen)
  - → `contextra-types`: dev: `DocId`
- `tests/acorn_selectivity_edge_cases_test.rs` (78 Zeilen)
  - → `contextra-ports`: dev: `VectorIndex`
- `tests/adaptive_ef_recall.rs` (279 Zeilen)
  - → `contextra-ports`: dev: `VectorIndex`
- `tests/campaign_hnsw_backoff.rs` (50 Zeilen)
  - → `contextra-ports`: dev: `VectorIndex`, `SeededRng::new`
- `tests/campaign_j23_vector_index.rs` (478 Zeilen)
  - → `contextra-ports`: dev: `VectorIndex`
  - → `contextra-types`: dev: `ContextraError::Index`×6
- `tests/campaign_j24_test.rs` (609 Zeilen)
  - → `contextra-ports`: dev: `VectorIndex`
- `tests/connectivity_regression.rs` (301 Zeilen)
  - → `contextra-ports`: dev: `VectorIndex`
- `tests/deleted_nodes_lock_contention.rs` (176 Zeilen)
  - → `contextra-ports`: dev: `VectorIndex`
- `tests/deletion_ghost_pointer_proptest.rs` (47 Zeilen)
  - → `contextra-ports`: dev: `VectorIndex`
- `tests/deletion_zeroize_test.rs` (45 Zeilen)
  - → `contextra-ports`: dev: `VectorIndex`
- `tests/determinism_test.rs` (111 Zeilen)
  - → `contextra-ports`: dev: `VectorIndex`
- `tests/differential_testing.rs` (78 Zeilen)
  - → `contextra-ports`: dev: `VectorIndex`
- `tests/diskann_corruption_fallback_test.rs` (103 Zeilen)
  - → `contextra-ports`: dev: `VectorIndex`
  - → `contextra-types`: dev: `TxId::new`
- `tests/diskann_docid128_tombstones.rs` (67 Zeilen)
  - → `contextra-ports`: dev: `vector_index::VectorIndex`
  - → `contextra-types`: dev: `ContextraError::Io`
- `tests/diskann_fault_injection_test.rs` (73 Zeilen)
  - → `contextra-ports`: dev: `VectorIndex`
- `tests/diskann_recall_test.rs` (219 Zeilen)
  - → `contextra-ports`: dev: `VectorIndex`
  - → `contextra-types`: dev: `TxId`
- `tests/diskann_search_filtered.rs` (171 Zeilen)
  - → `contextra-ports`: dev: `VectorIndex`
  - → `contextra-types`: dev: `MAX_SEARCH_K`
- `tests/distance_determinism.rs` (64 Zeilen)
  - → `contextra-types`: dev: `DistanceMetric`
- `tests/docid_128_hnsw.rs` (70 Zeilen)
  - → `contextra-ports`: dev: `VectorIndex`
  - → `contextra-types`: dev: `Result`×2
- `tests/filtered_selectivity_recall.rs` (154 Zeilen)
  - → `contextra-ports`: dev: `SeededRng`, `VectorIndex`
- `tests/golden_distance_oracle.rs` (159 Zeilen)
  - → `contextra-types`: dev: `DistanceMetric`
- `tests/hnsw_edge_cases.rs` (321 Zeilen)
  - → `contextra-ports`: dev: `VectorIndex`
- `tests/hnsw_layer_determinism.rs` (110 Zeilen)
  - → `contextra-ports`: dev: `VectorIndex`
- `tests/hnsw_rebuild_connectivity_test.rs` (67 Zeilen)
  - → `contextra-ports`: dev: `VectorIndex`
- `tests/hnsw_rebuild_search_consistency.rs` (134 Zeilen)
  - → `contextra-ports`: dev: `VectorIndex`
  - → `contextra-types`: dev: `DistanceMetric::Euclidean`
- `tests/hnsw_scanner_purge_test.rs` (228 Zeilen)
  - → `contextra-ports`: dev: `clock::SystemClock`, `VectorIndex`, `scanner`
- `tests/hnsw_seed_determinism.rs` (295 Zeilen)
  - → `contextra-ports`: dev: `SeededRng`, `VectorIndex`
- `tests/hnsw_snapshot_delete_test.rs` (129 Zeilen)
  - → `contextra-ports`: dev: `VectorIndex`
- `tests/k02_snapshot_rebuild_determinism.rs` (153 Zeilen)
  - → `contextra-ports`: dev: `VectorIndex`
  - → `contextra-types`: dev: `DistanceMetric::Euclidean`
- `tests/loom_quantizer_race_test.rs` (222 Zeilen)
  - → `contextra-ports`: dev: `VectorIndex`×2
  - → `contextra-types`: dev: `ContextraError`
- `tests/mmap_toctou_test.rs` (228 Zeilen)
  - → `contextra-ports`: dev: `VectorIndex`×2
- `tests/nan_validation_policy.rs` (171 Zeilen)
  - → `contextra-ports`: dev: `VectorIndex`
- `tests/partial_rebuild_recall.rs` (146 Zeilen)
  - → `contextra-ports`: dev: `VectorIndex`
- `tests/partial_rebuild_recall_regression.rs` (328 Zeilen)
  - → `contextra-ports`: dev: `VectorIndex`
- `tests/partial_rebuild_veto_f02_guard.rs` (97 Zeilen)
  - → `contextra-ports`: dev: `SeededRng`, `VectorIndex`
- `tests/poisoning.rs` (109 Zeilen)
  - → `contextra-ports`: dev: `VectorIndex`
- `tests/proptest_distance_quantize.rs` (89 Zeilen)
  - → `contextra-types`: dev: `DistanceMetric`
- `tests/quantize_persistence_audit.rs` (302 Zeilen)
  - → `contextra-ports`: dev: `VectorIndex`
  - → `contextra-types`: dev: `ContextraError`
- `tests/quantizer_encapsulation_regression.rs` (35 Zeilen)
  - → `contextra-ports`: dev: `VectorIndex`
- `tests/rabitq_recall_calibration.rs` (117 Zeilen)
  - → `contextra-types`: dev: `ContextraError`
- `tests/ram_reduction.rs` (59 Zeilen)
  - → `contextra-ports`: dev: `VectorIndex`
- `tests/recalibration.rs` (56 Zeilen)
  - → `contextra-ports`: dev: `VectorIndex`
  - → `contextra-types`: dev: `DistanceMetric::Euclidean`
- `tests/recall.rs` (67 Zeilen)
  - → `contextra-ports`: dev: `VectorIndex`
- `tests/recall_audit.rs` (405 Zeilen)
  - → `contextra-ports`: dev: `VectorIndex`
- `tests/recall_vs_brute_force.rs` (155 Zeilen)
  - → `contextra-ports`: dev: `VectorIndex`
- `tests/reconstruction_attack_test.rs` (71 Zeilen)
  - → `contextra-ports`: dev: `VectorIndex`
- `tests/rollback.rs` (55 Zeilen)
  - → `contextra-ports`: dev: `VectorIndex`
- `tests/simd_numerical_audit.rs` (171 Zeilen)
  - → `contextra-types`: dev: `DistanceMetric`
- `tests/sq8_bias_calibration.rs` (20 Zeilen)
  - → `contextra-types`: dev: `DistanceMetric`
- `tests/sq8_rerank_recall.rs` (163 Zeilen)
  - → `contextra-ports`: dev: `SeededRng`, `VectorIndex`

**Öffentliche Schnittstelle** (erreichbar über `pub mod`-Kette oder `pub use`)

- `acorn`
  - 1 enum, 1 fn, 1 struct, 1 trait
    - `fn compute_gamma_edge_budget`
    - `trait FilteredIndex` · Methoden: search_knn_acorn
    - `enum AcornError`
    - `struct NaiveReferenceIndex` · 3 Methoden
- `candidate_stream`
  - 1 const, 1 struct
    - `const DEFAULT_VECTOR_STREAM_BATCH_SIZE`
    - `struct VectorCandidateStream` · 8 Methoden ⚠ fremd: `contextra-ports`
- `casting`
  - 8 fn
    - `fn to_u32`
    - `fn u32_to_usize`
    - `fn u64_to_usize`
    - `fn u64_to_u32`
    - `fn to_u8`
    - `fn to_u64`
    - `fn f32_to_usize`
    - `fn f64_to_usize`
- `compute_pool`
  - 2 struct
    - `struct TaskHandle` · 1 Methoden
    - `struct ComputePool` · 5 Methoden
- `contract`
  - 2 enum, 2 struct, 6 trait, 1 type
    - `enum Error`
    - `type Result`
    - `trait Clock` · Methoden: now_unix_ms
    - `trait Rng` · Methoden: next_u64
    - `struct ExternalId`
    - `enum Metric`
    - `struct ScoredPoint`
    - `trait DistanceFn` · Methoden: distance
    - `trait Index` · Methoden: insert, remove, search
    - `trait Maintain` · Methoden: compact, vacuum
    - `trait BlockTransform` · Methoden: transform, inverse_transform
- `diskann`
  - 1 struct
    - `struct DiskAnnIndex` *(re-exportiert)* · 2 Methoden
- `distance`
  - 16 fn, 4 struct, 1 trait
    - `struct CosineSimilarityPartsU8`
    - `trait DistanceKernel` · Methoden: distance, distance_f32_bytes, distance_trusted, distance_f32_bytes_trusted, cosine_distance, dot_product_distance…
    - `struct ScalarKernel`
    - `struct ScalarDistance` · 2 Methoden
    - `struct InjectedDistanceKernel` · 1 Methoden
    - `fn validate_vector`
    - `fn compute_distance` ⚠ fremd: `contextra-types`
    - `fn compute_distance_trusted` ⚠ fremd: `contextra-types`
    - `fn compute_distance_f32_bytes_trusted` ⚠ fremd: `contextra-types`
    - `fn cosine_distance`
    - `fn dot_product_distance`
    - `fn euclidean_distance`
    - `fn euclidean_distance_sq_u8`
    - `fn euclidean_distance_sq_f32_u8`
    - `fn dot_product_u8`
    - `fn dot_product_f32_u8`
    - `fn cosine_similarity_parts_u8`
    - `fn normalize_inplace`
    - `fn cosine_distance_scalar`
    - `fn dot_product_scalar`
    - `fn euclidean_distance_scalar`
- `error`
  - 1 enum, 1 type
    - `enum VectorError`
    - `type Result`
- `fixed_hasher`
  - 3 fn, 2 type
    - `fn fixed_hasher`
    - `type FixedHashSet`
    - `type FixedHashMap`
    - `fn new_fixed_hashset`
    - `fn new_fixed_hashmap`
- `hnsw`
  - 2 const, 1 enum, 2 fn, 12 struct, 1 trait, 1 type
    - `struct AdaptiveEfPolicy` · 2 Methoden
    - `struct AdaptiveEfStats`
    - `struct AdaptiveEfStateMachine` · 6 Methoden
    - `const ARENA_ALIGNMENT_BYTES`
    - `const ARENA_ALIGNMENT_U32`
    - `type BacklinkKey`
    - `struct BacklinkTable` · 4 Methoden
    - `struct HnswArena` · 10 Methoden
    - `struct MonotonicCounterClock` · 2 Methoden
    - `struct DeletionStats`
    - `trait GhostFreeVectorIndex` · Methoden: remove_with_graph_repair
    - `struct HnswPurger` · 1 Methoden
    - `fn hnsw_purger`
    - `struct HnswScanner` · 2 Methoden
    - `fn hnsw_scanner`
    - `struct ResidualReport` · 1 Methoden
    - `struct Sq8Bias` · 2 Methoden
    - `enum VectorStagingOp`
    - `struct VectorTxBuffer` · 4 Methoden
- `partial_rebuild`
  - 1 fn, 2 struct
    - `struct PartialRebuildConfig`
    - `struct TraversalTracker` · 4 Methoden
    - `fn should_trigger_partial_rebuild`
- `persistence`
  - 1 fn, 2 struct
    - `fn write_atomic`
    - `struct MmapIndex` *(re-exportiert)* · 5 Methoden
- `quantize`
  - 3 const, 1 struct
- `quantize_rabitq`
  - 1 struct
- `types`
  - 2 const, 1 enum, 7 struct, 4 trait
  - … 20 weitere Items nicht aufgelistet (`--max-items`)
- Zusätzlich 31 `pub`-Items in privaten Modulen ohne Re-Export (nicht Teil der echten API).

**Eingehend (wer nutzt dieses Crate)**

| Nutzer | Symbole prod | Symbole dev |
|---|---|---|
| `contextra-engine` | `HnswIndex`×4, `hnsw::HnswIndex`×3, `acorn::FilteredIndex`, `HnswConfig` | `hnsw::HnswConfig::default`, `hnsw::HnswIndex::try_new`, `acorn`, `HnswIndex::try_new` |
| `contextra-db` | – | `HnswConfig`×3, `HnswIndex`×2, `HnswIndex::try_new` |

**Nach außen ungenutzte öffentliche Items (Heuristik, 99 von 100)**: `compute_gamma_edge_budget`, `AcornError`, `NaiveReferenceIndex`, `DEFAULT_VECTOR_STREAM_BATCH_SIZE`, `VectorCandidateStream`, `to_u32`, `u32_to_usize`, `u64_to_usize`, `u64_to_u32`, `to_u8`, `to_u64`, `f32_to_usize`, `f64_to_usize`, `TaskHandle`, `ComputePool`, `Error`, `Result`, `Clock`, `Rng`, `ExternalId`, `Metric`, `ScoredPoint`, `DistanceFn`, `Index`, `Maintain`, `BlockTransform`, `DiskAnnIndex`, `CosineSimilarityPartsU8`, `DistanceKernel`, `ScalarKernel` …

### contextra-wire

- Pfad: `crates/contextra-wire` · Zeilen: 5541 · Rust-Dateien: 16 · Ring: 0
- Interne Cargo-Abhängigkeiten: keine

**Ausgehend (Importe aus anderen Workspace-Crates)**

Keine.

**Pro Komponente**

Keine Importe aus anderen Workspace-Crates.

**Öffentliche Schnittstelle** (erreichbar über `pub mod`-Kette oder `pub use`)

- `(crate-root)`
  - 1 enum, 1 type
    - `enum Error`
    - `type Result`
- `adapter`
  - 3 struct, 1 trait
    - `struct Unverified`
    - `struct Verified`
    - `trait VerifiableRoot`
    - `struct WireBuffer` · 8 Methoden
- `jsonrpc`
  - 1 enum, 2 fn, 4 struct
    - `struct JsonRpcConfig`
    - `enum JsonRpcPayload`
    - `struct JsonRpcRequest` · 1 Methoden
    - `struct JsonRpcResponse` · 2 Methoden
    - `struct JsonRpcError` · 5 Methoden
    - `fn parse_jsonrpc_payload`
    - `fn parse_jsonrpc_payload_with_config`
- `verification`
  - 1 enum, 9 fn, 1 struct
    - `enum FlatBufferError`
    - `struct FlatBufferConfig`
    - `fn verify_flatbuffer`
    - `fn verify_and_parse_search_response`
    - `fn verify_and_parse_embedding`
    - `fn verify_and_parse_scored_document`
    - `fn verify_and_parse_vector_index_update`
    - `fn verify_and_parse_role_id`
    - `fn verify_and_parse_hyper_edge_id`
    - `fn verify_and_parse_role_binding`
    - `fn verify_and_parse_hyper_edge`
- Zusätzlich 40 `pub`-Items in privaten Modulen ohne Re-Export (nicht Teil der echten API).

**Eingehend (wer nutzt dieses Crate)**

| Nutzer | Symbole prod | Symbole dev |
|---|---|---|
| `contextra-py` | `ScoredDocument::create`×2, `ScoredDocumentArgs`×2, `SearchResponse::create`×2, `SearchResponseArgs`×2 | – |

**Nach außen ungenutzte öffentliche Items (Heuristik, 24 von 24)**: `Unverified`, `Verified`, `VerifiableRoot`, `WireBuffer`, `JsonRpcConfig`, `JsonRpcPayload`, `JsonRpcRequest`, `JsonRpcResponse`, `JsonRpcError`, `parse_jsonrpc_payload`, `parse_jsonrpc_payload_with_config`, `Error`, `Result`, `FlatBufferError`, `FlatBufferConfig`, `verify_flatbuffer`, `verify_and_parse_search_response`, `verify_and_parse_embedding`, `verify_and_parse_scored_document`, `verify_and_parse_vector_index_update`, `verify_and_parse_role_id`, `verify_and_parse_hyper_edge_id`, `verify_and_parse_role_binding`, `verify_and_parse_hyper_edge`

