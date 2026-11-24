# Schnittstellenspezifikation: `contextra-db`

> Erzeugt durch `tools/extract_interfaces.py` aus dem Quellcode. Keine Handpflege. Jede Angabe trägt Datei:Zeile. Maßgeblich bleibt der Quellcode.

## 1. Kopf

| Feld | Wert |
|---|---|
| Name | `contextra-db` |
| Version | workspace |
| Pfad | `crates/contextra-db` |
| Rolle | Datenbank-Fassade (Hauptkonsument Store) |
| Bibliothekspfad | src/lib.rs (Standard) |

**Crate-Dokumentation (//!):**

> # Contextra DB — Facade & Storage Engine
>
> `contextra-db` bildet die benutzerfreundliche Hauptfassade und Speicher-Engine
> für Anwendungen, welche die Contextra-Datenbankbibliothek als eingebettete Engine nutzen.

## 2. Abhängigkeiten

| Name | Art | Quelle | optional | verwendet von (normal/dev) |
|---|---|---|---|---|
| `ahash` | normal | workspace | nein |  |
| `blake3` | normal | workspace | nein |  |
| `bytes` | normal | workspace | nein |  |
| `contextra-adapt` | dev | workspace | nein | contextra-db, contextra-router |
| `contextra-checkpoint` | dev | workspace | nein | contextra-agent, contextra-db, contextra-engine |
| `contextra-cognition` | normal | path | ja | contextra-db |
| `contextra-graph` | dev | workspace | nein | contextra-agent, contextra-cognition, contextra-db, contextra-engine, contextra-graph, contextra-graph-fuzz |
| `contextra-infer-onnx` | normal | workspace | ja | contextra-db, contextra-mcp |
| `contextra-ingest` | dev | workspace | nein | contextra-db, contextra-engine, contextra-ingest-fuzz |
| `contextra-mvcc` | normal | workspace | nein | contextra-db, contextra-store, contextra-vector, contextra-vector-fuzz |
| `contextra-ports` | normal | workspace | nein | contextra, contextra-adapt, contextra-agent, contextra-checkpoint, contextra-cognition, contextra-crypto, contextra-crypto-fuzz, contextra-db, contextra-engine, contextra-graph, contextra-graph-fuzz, contextra-infer-candle, contextra-infer-ollama, contextra-infer-onnx, contextra-kvcache, contextra-mcp, contextra-mvcc, contextra-rank, contextra-router, contextra-store, contextra-store-fuzz, contextra-testkit, contextra-text, contextra-vector, contextra-vector-fuzz |
| `contextra-rank` | normal | workspace | nein | contextra-bench, contextra-cognition, contextra-db, contextra-engine, contextra-infer-candle, contextra-mcp |
| `contextra-sandbox` | normal | workspace | ja | contextra-db |
| `contextra-store` | dev | workspace | nein | contextra-agent, contextra-bench, contextra-db, contextra-engine, contextra-store-fuzz |
| `contextra-testkit` | dev | workspace | nein | contextra-agent, contextra-checkpoint, contextra-cognition, contextra-crypto-fuzz, contextra-db, contextra-engine, contextra-store |
| `contextra-text` | dev | workspace | nein | contextra-agent, contextra-bench, contextra-cognition, contextra-db, contextra-engine, contextra-text-fuzz |
| `contextra-types` | normal | workspace | nein | contextra, contextra-adapt, contextra-agent, contextra-bench, contextra-checkpoint, contextra-cognition, contextra-crypto, contextra-crypto-fuzz, contextra-db, contextra-engine, contextra-graph, contextra-graph-fuzz, contextra-infer-candle, contextra-infer-ollama, contextra-infer-onnx, contextra-ingest, contextra-kvcache, contextra-mcp, contextra-mvcc, contextra-ports, contextra-py, contextra-rank, contextra-router, contextra-simd, contextra-store, contextra-store-fuzz, contextra-testkit, contextra-text, contextra-vault, contextra-vector, contextra-vector-fuzz |
| `contextra-vault` | normal | path | nein | contextra-db |
| `contextra-vector` | dev | workspace | nein | contextra-agent, contextra-bench, contextra-db, contextra-engine, contextra-vector-fuzz |
| `criterion` | dev | registry | nein |  |
| `parking_lot` | normal | workspace | nein |  |
| `proptest` | dev | registry | nein |  |
| `serde` | normal | workspace | nein |  |
| `serde_json` | normal | workspace | nein |  |
| `sha2` | dev | registry | nein |  |
| `tempfile` | dev | workspace | nein |  |
| `thiserror` | normal | workspace | nein |  |
| `tokio` | dev | workspace | nein |  |
| `tokio-util` | normal | registry | nein |  |
| `tracing` | normal | workspace | nein |  |
| `wat` | dev | registry | nein |  |
| `zeroize` | normal | registry | nein |  |

**Verwendet von (Crate-Ebene):** `contextra`, `contextra-agent`, `contextra-bench`, `contextra-cognition`, `contextra-db-fuzz`, `contextra-engine`

## 3. Features

| Feature | Aktiviert |
|---|---|
| `adaptive-candidate-pool-sizing` | `dep:contextra-adapt` |
| `auto-extraction-opt-out` | – |
| `background-maintenance` | – |
| `bench` | – |
| `cognition` | `dep:contextra-cognition` |
| `coherence-bonus-fusion` | – |
| `default` | – |
| `edge-reinforcement-learning` | – |
| `entity-extraction` | `exp-extraction` |
| `exp-extraction` | – |
| `experimental-diskann` | `contextra-vector/experimental-diskann` |
| `graph-connectivity-health` | `contextra-graph/graph-connectivity-health` |
| `license` | – |
| `onnx` | `dep:contextra-infer-onnx`, `contextra-infer-onnx/onnx` |
| `reranking` | – |
| `sandbox` | `dep:contextra-sandbox` |
| `volatile-vault` | – |

## 4. Metriken (Quellcode)

| Kennzahl | Wert |
|---|---|
| Quelldateien | 94 |
| Quellzeilen | 23313 |
| Öffentliche Items (Modul-Ebene) | 191 |
| davon `fn` | 43 |
| davon `struct` | 58 |
| davon `enum` | 8 |
| davon `trait` | 7 |
| davon `type` | 4 |
| davon `const` | 33 |
| davon `mod` | 38 |
| Öffentliche Makros (`#[macro_export]`) | 0 |
| Öffentliche Re-Exports (`pub use`) | 77 |
| `unsafe`-Vorkommen (Produktionscode) | 0 |
| `.unwrap()` (Produktionscode) | 433 |
| `.expect(` (Produktionscode) | 8 |
| `panic!` (Produktionscode) | 12 |
| `todo!/unimplemented!/unreachable!` | 2 |
| Indexierungen `x[…]` (grob, Produktionscode) | 90 |
| `#[deprecated]` | 57 |
| `#[cfg(feature…)]`-Gates | 32 |
| Parser-Abdeckung (geparste `pub`-Deklarationen / Grep-Treffer) | 497 / 502 = 99 % |

## 5. Modulbaum und Dateien

| Modul | Datei | Zeilen |
|---|---|---|
| `adapters` | `crates/contextra-db/src/adapters/mod.rs` | 8 |
| `adapters::ollama` | `crates/contextra-db/src/adapters/ollama.rs` | 10 |
| `adapters::privacy` | `crates/contextra-db/src/adapters/privacy.rs` | 8 |
| `adapters::text` | `crates/contextra-db/src/adapters/text.rs` | 8 |
| `backends` | `crates/contextra-db/src/backends.rs` | 65 |
| `background_workers::config` | `crates/contextra-db/src/background_workers/config.rs` | 70 |
| `background_workers::expiry_workers` | `crates/contextra-db/src/background_workers/expiry_workers.rs` | 121 |
| `background_workers::hyperedge_worker` | `crates/contextra-db/src/background_workers/hyperedge_worker.rs` | 109 |
| `background_workers` | `crates/contextra-db/src/background_workers/mod.rs` | 36 |
| `background_workers::orphan_workers` | `crates/contextra-db/src/background_workers/orphan_workers.rs` | 196 |
| `background_workers::tests` | `crates/contextra-db/src/background_workers/tests.rs` | 766 |
| `chunker::blocks` | `crates/contextra-db/src/chunker/blocks.rs` | 229 |
| `chunker::config` | `crates/contextra-db/src/chunker/config.rs` | 37 |
| `chunker::estimate` | `crates/contextra-db/src/chunker/estimate.rs` | 92 |
| `chunker` | `crates/contextra-db/src/chunker/mod.rs` | 399 |
| `chunker::sections` | `crates/contextra-db/src/chunker/sections.rs` | 26 |
| `chunker::split` | `crates/contextra-db/src/chunker/split.rs` | 249 |
| `chunker::unit_tests` | `crates/contextra-db/src/chunker/unit_tests.rs` | 411 |
| `chunker::windows` | `crates/contextra-db/src/chunker/windows.rs` | 90 |
| `cognition::bridge` | `crates/contextra-db/src/cognition/bridge.rs` | 15 |
| `cognition` | `crates/contextra-db/src/cognition/mod.rs` | 4 |
| `collection::crud::auto_extraction` | `crates/contextra-db/src/collection/crud/auto_extraction.rs` | 340 |
| `collection::crud::delete` | `crates/contextra-db/src/collection/crud/delete.rs` | 74 |
| `collection::crud::insert` | `crates/contextra-db/src/collection/crud/insert.rs` | 565 |
| `collection::crud::internal` | `crates/contextra-db/src/collection/crud/internal.rs` | 104 |
| `collection::crud::kv` | `crates/contextra-db/src/collection/crud/kv.rs` | 69 |
| `collection::crud::links` | `crates/contextra-db/src/collection/crud/links.rs` | 241 |
| `collection::crud` | `crates/contextra-db/src/collection/crud/mod.rs` | 28 |
| `collection::crud::read` | `crates/contextra-db/src/collection/crud/read.rs` | 316 |
| `collection::crud::tests` | `crates/contextra-db/src/collection/crud/tests.rs` | 439 |
| `collection::crud::update` | `crates/contextra-db/src/collection/crud/update.rs` | 130 |
| `collection::kv_lock` | `crates/contextra-db/src/collection/kv_lock.rs` | 278 |
| `collection::maintenance` | `crates/contextra-db/src/collection/maintenance.rs` | 972 |
| `collection` | `crates/contextra-db/src/collection/mod.rs` | 730 |
| `collection::query_builder::builder` | `crates/contextra-db/src/collection/query_builder/builder.rs` | 312 |
| `collection::query_builder::builder_exec` | `crates/contextra-db/src/collection/query_builder/builder_exec.rs` | 427 |
| `collection::query_builder` | `crates/contextra-db/src/collection/query_builder/mod.rs` | 37 |
| `collection::query_builder::scope` | `crates/contextra-db/src/collection/query_builder/scope.rs` | 65 |
| `collection::query_builder::strategy` | `crates/contextra-db/src/collection/query_builder/strategy.rs` | 87 |
| `collection::query_builder::tests` | `crates/contextra-db/src/collection/query_builder/tests.rs` | 712 |
| `collection::query_builder::weights` | `crates/contextra-db/src/collection/query_builder/weights.rs` | 47 |
| `collection::relate` | `crates/contextra-db/src/collection/relate.rs` | 289 |
| `collection::search::basic` | `crates/contextra-db/src/collection/search/basic.rs` | 120 |
| `collection::search::checkpoint` | `crates/contextra-db/src/collection/search/checkpoint.rs` | 106 |
| `collection::search::filtered` | `crates/contextra-db/src/collection/search/filtered.rs` | 262 |
| `collection::search::hybrid` | `crates/contextra-db/src/collection/search/hybrid/mod.rs` | 398 |
| `collection::search::hybrid::query` | `crates/contextra-db/src/collection/search/hybrid/query.rs` | 590 |
| `collection::search::hydrate` | `crates/contextra-db/src/collection/search/hydrate.rs` | 116 |
| `collection::search` | `crates/contextra-db/src/collection/search/mod.rs` | 32 |
| `collection::tests::concurrency_tests` | `crates/contextra-db/src/collection/tests/concurrency_tests.rs` | 642 |
| `collection::tests::fixtures` | `crates/contextra-db/src/collection/tests/fixtures.rs` | 10 |
| `collection::tests::hybrid_search_tests` | `crates/contextra-db/src/collection/tests/hybrid_search_tests.rs` | 1287 |
| `collection::tests::insert_tests` | `crates/contextra-db/src/collection/tests/insert_tests.rs` | 366 |
| `collection::tests::link_tests` | `crates/contextra-db/src/collection/tests/link_tests.rs` | 430 |
| `collection::tests::misc_tests` | `crates/contextra-db/src/collection/tests/misc_tests.rs` | 606 |
| `collection::tests` | `crates/contextra-db/src/collection/tests/mod.rs` | 23 |
| `collection::tests::read_tests` | `crates/contextra-db/src/collection/tests/read_tests.rs` | 361 |
| `collection::tests::search_tests` | `crates/contextra-db/src/collection/tests/search_tests.rs` | 537 |
| `collection::tests::update_delete_tests` | `crates/contextra-db/src/collection/tests/update_delete_tests.rs` | 343 |
| `collection::tx` | `crates/contextra-db/src/collection/tx.rs` | 59 |
| `contextra_impl::collections` | `crates/contextra-db/src/contextra_impl/collections.rs` | 330 |
| `contextra_impl::crud` | `crates/contextra-db/src/contextra_impl/crud.rs` | 406 |
| `contextra_impl::diagnostics` | `crates/contextra-db/src/contextra_impl/diagnostics.rs` | 68 |
| `contextra_impl::lifecycle` | `crates/contextra-db/src/contextra_impl/lifecycle.rs` | 600 |
| `contextra_impl` | `crates/contextra-db/src/contextra_impl/mod.rs` | 12 |
| `contextra_impl::wiring` | `crates/contextra-db/src/contextra_impl/wiring.rs` | 36 |
| `decay_controller` | `crates/contextra-db/src/decay_controller.rs` | 198 |
| `export` | `crates/contextra-db/src/export.rs` | 395 |
| `extraction` | `crates/contextra-db/src/extraction/mod.rs` | 100 |
| `extraction::open_ie` | `crates/contextra-db/src/extraction/open_ie.rs` | 129 |
| `extraction::rule_based` | `crates/contextra-db/src/extraction/rule_based.rs` | 240 |
| `extraction::tests` | `crates/contextra-db/src/extraction/tests.rs` | 112 |
| `extraction::types` | `crates/contextra-db/src/extraction/types.rs` | 61 |
| `filter` | `crates/contextra-db/src/filter.rs` | 425 |
| `fusion` | `crates/contextra-db/src/fusion.rs` | 72 |
| `homeostat` | `crates/contextra-db/src/homeostat.rs` | 58 |
| `import` | `crates/contextra-db/src/import.rs` | 144 |
| `crate` | `crates/contextra-db/src/lib.rs` | 1205 |
| `license_seam` | `crates/contextra-db/src/license_seam.rs` | 7 |
| `merge_operator_adapter` | `crates/contextra-db/src/merge_operator_adapter.rs` | 201 |
| `metrics_names` | `crates/contextra-db/src/metrics_names.rs` | 66 |
| `multistep` | `crates/contextra-db/src/multistep.rs` | 442 |
| `orchestration` | `crates/contextra-db/src/orchestration/mod.rs` | 330 |
| `pid_latency_controller` | `crates/contextra-db/src/pid_latency_controller.rs` | 134 |
| `temporal_filter` | `crates/contextra-db/src/temporal_filter.rs` | 383 |
| `transaction::cleanup` | `crates/contextra-db/src/transaction/cleanup.rs` | 129 |
| `transaction::compensating_actions` | `crates/contextra-db/src/transaction/compensating_actions.rs` | 402 |
| `transaction::db_transaction` | `crates/contextra-db/src/transaction/db_transaction.rs` | 201 |
| `transaction::db_transaction_commit` | `crates/contextra-db/src/transaction/db_transaction_commit.rs` | 395 |
| `transaction::db_transaction_rollback` | `crates/contextra-db/src/transaction/db_transaction_rollback.rs` | 258 |
| `transaction::intent` | `crates/contextra-db/src/transaction/intent.rs` | 45 |
| `transaction` | `crates/contextra-db/src/transaction/mod.rs` | 95 |
| `transaction::tests` | `crates/contextra-db/src/transaction/tests.rs` | 133 |
| `volatile_vault` | `crates/contextra-db/src/volatile_vault.rs` | 2 |

## 6. Re-Exports (`pub use`)

- `pub use contextra_text::compat::scorer::Bm25Scorer` — `crates/contextra-db/src/adapters/text.rs:8` (Modul `adapters::text`)
- `pub use config::` — `crates/contextra-db/src/background_workers/mod.rs:22` (Modul `background_workers`)
- `pub use expiry_workers::` — `crates/contextra-db/src/background_workers/mod.rs:25` (Modul `background_workers`)
- `pub use expiry_workers::` — `crates/contextra-db/src/background_workers/mod.rs:27` (Modul `background_workers`)
- `pub use hyperedge_worker::` — `crates/contextra-db/src/background_workers/mod.rs:28` (Modul `background_workers`)
- `pub use orphan_workers::` — `crates/contextra-db/src/background_workers/mod.rs:33` (Modul `background_workers`)
- `pub use config::ChunkerConfig` — `crates/contextra-db/src/chunker/mod.rs:23` (Modul `chunker`)
- `pub use estimate::estimate_tokens` — `crates/contextra-db/src/chunker/mod.rs:24` (Modul `chunker`)
- `pub use windows::` — `crates/contextra-db/src/chunker/mod.rs:25` (Modul `chunker`)
- `pub use bridge::*` — `crates/contextra-db/src/cognition/mod.rs:4` (Modul `cognition`)
- `pub use crate::extraction::` — `crates/contextra-db/src/collection/crud/auto_extraction.rs:16` (Modul `collection::crud::auto_extraction`)
- `pub use contextra_types::AutoExtractionMode` — `crates/contextra-db/src/collection/crud/auto_extraction.rs:18` (Modul `collection::crud::auto_extraction`)
- `pub use auto_extraction::` — `crates/contextra-db/src/collection/crud/mod.rs:26` (Modul `collection::crud`)
- `pub use read::` — `crates/contextra-db/src/collection/crud/mod.rs:28` (Modul `collection::crud`)
- `pub use contextra_graph::percolation::PercolationConfig` — `crates/contextra-db/src/collection/maintenance.rs:879` (Modul `collection::maintenance`)
- `pub use builder::` — `crates/contextra-db/src/collection/query_builder/mod.rs:32` (Modul `collection::query_builder`)
- `pub use scope::ScopeConstraint` — `crates/contextra-db/src/collection/query_builder/mod.rs:33` (Modul `collection::query_builder`)
- `pub use strategy::SearchStrategy` — `crates/contextra-db/src/collection/query_builder/mod.rs:34` (Modul `collection::query_builder`)
- `pub use weights::SignalWeights` — `crates/contextra-db/src/collection/query_builder/mod.rs:35` (Modul `collection::query_builder`)
- `pub use checkpoint::` — `crates/contextra-db/src/collection/search/mod.rs:25` (Modul `collection::search`)
- `pub use crate::temporal_filter::` — `crates/contextra-db/src/collection/search/mod.rs:30` (Modul `collection::search`)
- `pub use open_ie::extract_triples` — `crates/contextra-db/src/extraction/mod.rs:23` (Modul `extraction`)
- `pub use rule_based::` — `crates/contextra-db/src/extraction/mod.rs:24` (Modul `extraction`)
- `pub use types::` — `crates/contextra-db/src/extraction/mod.rs:25` (Modul `extraction`)
- `pub use contextra_rank::fusion::` — `crates/contextra-db/src/fusion.rs:5` (Modul `fusion`)
- `pub use contextra_types::` — `crates/contextra-db/src/fusion.rs:13` (Modul `fusion`)
- `pub use contextra_types::OnSignalFailure as SignalFailurePolicy` — `crates/contextra-db/src/fusion.rs:16` (Modul `fusion`)
- `pub use contextra_ports::TextEmbeddingEngine` — `crates/contextra-db/src/lib.rs:58` (Modul `crate`)
- `pub use contextra_ports::VectorDeleteMode` — `crates/contextra-db/src/lib.rs:59` (Modul `crate`)
- `pub use contextra_store::lsm::DurabilityMode` — `crates/contextra-db/src/lib.rs:61` (Modul `crate`)
- `pub use merge_operator_adapter::WasmMergeOperator` — `crates/contextra-db/src/lib.rs:72` (Modul `crate`)
- `pub use decay_controller::` — `crates/contextra-db/src/lib.rs:75` (Modul `crate`)
- `pub use export::` — `crates/contextra-db/src/lib.rs:78` (Modul `crate`)
- `pub use import::ImportSummary` — `crates/contextra-db/src/lib.rs:82` (Modul `crate`)
- `pub use background_workers::start_decay_cleanup_worker` — `crates/contextra-db/src/lib.rs:86` (Modul `crate`)
- `pub use background_workers::` — `crates/contextra-db/src/lib.rs:88` (Modul `crate`)
- `pub use backends::Backends` — `crates/contextra-db/src/lib.rs:91` (Modul `crate`)
- `pub use backends::DefaultBackends` — `crates/contextra-db/src/lib.rs:93` (Modul `crate`)
- `pub use collection::crud::MAX_SCAN_RESULTS` — `crates/contextra-db/src/lib.rs:95` (Modul `crate`)
- `pub use collection::maintenance::PercolationConfig` — `crates/contextra-db/src/lib.rs:97` (Modul `crate`)
- `pub use collection::maintenance::PercolationResult` — `crates/contextra-db/src/lib.rs:100` (Modul `crate`)
- `pub use collection::query_builder::` — `crates/contextra-db/src/lib.rs:102` (Modul `crate`)
- `pub use collection::` — `crates/contextra-db/src/lib.rs:103` (Modul `crate`)
- `pub use contextra_checkpoint` — `crates/contextra-db/src/lib.rs:104` (Modul `crate`)
- `pub use contextra_text::Language` — `crates/contextra-db/src/lib.rs:105` (Modul `crate`)
- `pub use filter::MetadataFilter` — `crates/contextra-db/src/lib.rs:108` (Modul `crate`)
- `pub use fusion::` — `crates/contextra-db/src/lib.rs:146` (Modul `crate`)
- `pub use contextra_ports::DriftStatusProvider` — `crates/contextra-db/src/lib.rs:171` (Modul `crate`)
- `pub use contextra_types::DistanceMetric` — `crates/contextra-db/src/lib.rs:358` (Modul `crate`)
- `pub use serde_json::json` — `crates/contextra-db/src/lib.rs:359` (Modul `crate`)
- `pub use crate::collection::` — `crates/contextra-db/src/lib.rs:459` (Modul `prelude`)
- `pub use crate::` — `crates/contextra-db/src/lib.rs:460` (Modul `prelude`)
- `pub use contextra_types::` — `crates/contextra-db/src/lib.rs:465` (Modul `prelude`)
- `pub use crate::multistep::` — `crates/contextra-db/src/lib.rs:467` (Modul `prelude`)
- `pub use crate::volatile_vault::*` — `crates/contextra-db/src/lib.rs:470` (Modul `prelude`)
- `pub use crate::decay_controller::` — `crates/contextra-db/src/lib.rs:474` (Modul `advanced`)
- `pub use crate::homeostat::RerankDeadline` — `crates/contextra-db/src/lib.rs:477` (Modul `advanced`)
- `pub use crate::homeostat::RerankPidController` — `crates/contextra-db/src/lib.rs:479` (Modul `advanced`)
- `pub use crate::pid_latency_controller::` — `crates/contextra-db/src/lib.rs:480` (Modul `advanced`)
- `pub use crate as engine` — `crates/contextra-db/src/lib.rs:485` (Modul `advanced`)
- `pub use crate::DurabilityMode` — `crates/contextra-db/src/lib.rs:486` (Modul `advanced`)
- `pub use crate::` — `crates/contextra-db/src/lib.rs:488` (Modul `advanced`)
- `pub use crate::SandboxBridge` — `crates/contextra-db/src/lib.rs:495` (Modul `advanced`)
- `pub use advanced::*` — `crates/contextra-db/src/lib.rs:498` (Modul `crate`)
- `pub use prelude::*` — `crates/contextra-db/src/lib.rs:499` (Modul `crate`)
- `pub use homeostat::RerankDeadline` — `crates/contextra-db/src/lib.rs:501` (Modul `crate`)
- `pub use homeostat::RerankPidController` — `crates/contextra-db/src/lib.rs:503` (Modul `crate`)
- `pub use multistep::` — `crates/contextra-db/src/lib.rs:504` (Modul `crate`)
- `pub use pid_latency_controller::` — `crates/contextra-db/src/lib.rs:505` (Modul `crate`)
- `pub use volatile_vault::*` — `crates/contextra-db/src/lib.rs:511` (Modul `crate`)
- `pub use contextra_ports::license::` — `crates/contextra-db/src/license_seam.rs:7` (Modul `license_seam`)
- `pub use contextra_ports::QueryRewriter` — `crates/contextra-db/src/multistep.rs:28` (Modul `multistep`)
- `pub use cleanup::` — `crates/contextra-db/src/transaction/mod.rs:89` (Modul `transaction`)
- `pub use compensating_actions::` — `crates/contextra-db/src/transaction/mod.rs:90` (Modul `transaction`)
- `pub use db_transaction::DbTransaction` — `crates/contextra-db/src/transaction/mod.rs:94` (Modul `transaction`)
- `pub use intent::CommitIntent` — `crates/contextra-db/src/transaction/mod.rs:95` (Modul `transaction`)
- `pub use contextra_vault::*` — `crates/contextra-db/src/volatile_vault.rs:2` (Modul `volatile_vault`)

## 8. Öffentliche Schnittstelle je Item

### Modul `adapters`

#### `mod ollama`
- Ort: `crates/contextra-db/src/adapters/mod.rs:6`
```rust
pub mod ollama
```

#### `mod privacy`
- Ort: `crates/contextra-db/src/adapters/mod.rs:7`
```rust
pub mod privacy
```

#### `mod text`
- Ort: `crates/contextra-db/src/adapters/mod.rs:8`
```rust
pub mod text
```

### Modul `adapters::ollama`

#### `trait OllamaBridge`
- Ort: `crates/contextra-db/src/adapters/ollama.rs:7`
```rust
pub trait OllamaBridge: Send + Sync
```
Dokumentation: Neutral Ollama generator contract bridge.

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn generate<'a>(&'a self, prompt: &'a str) -> BoxFuture<'a, Result<String>>` | `crates/contextra-db/src/adapters/ollama.rs:9` | Dispatches prompt generation to an underlying model provider. |

### Modul `adapters::privacy`

#### `fn to_canonical_tenant_id`
- Ort: `crates/contextra-db/src/adapters/privacy.rs:6`
```rust
pub fn to_canonical_tenant_id(raw: u64) -> TenantId
```
Dokumentation: Converts a raw tenant ID integer into the canonical [`TenantId`].

### Modul `backends`

#### `struct DefaultBackends`
- Ort: `crates/contextra-db/src/backends.rs:56`
```rust
pub struct DefaultBackends
```
Dokumentation: Standard-Backends unter Verwendung von LSM-Storage, HNSW-Index, Inverted-Text und CSR-Graph. Wird standardmäßig von der Engine genutzt, wenn keine abweichenden Implementierungen konfiguriert werden.
- Impl-Block (Trait `Backends`) `crates/contextra-db/src/backends.rs:59`

#### `trait Backends`
- Ort: `crates/contextra-db/src/backends.rs:26`
```rust
pub trait Backends: Send + Sync + 'static
```
Dokumentation: Baukasten-Vertrag (M3, WP-ENG-1): Assoziiertes Trait-Bündel zur Entkopplung der Engine von konkreten Backend-Implementierungen (`store`, `vector`, `text`, `graph`). Nachfolgende Entwickler können diesen Vertrag implementieren, um alternative Speicher- oder Index-Engines (z. B. In-Memory-Mocks für Unittests oder verteilte Speichersysteme) einzuhängen.

Assoziierte Konstanten/Typen:

- `type Storage: StorageEngine` — `crates/contextra-db/src/backends.rs:28`
- `type Vector: VectorIndex` — `crates/contextra-db/src/backends.rs:30`
- `type Text: TextIndex` — `crates/contextra-db/src/backends.rs:32`
- `type Graph: GraphIndex` — `crates/contextra-db/src/backends.rs:34`

### Modul `background_workers::config`

#### `struct OrphanCleanupBackoffConfig`
- Ort: `crates/contextra-db/src/background_workers/config.rs:16`
```rust
pub struct OrphanCleanupBackoffConfig
```
Dokumentation: Configuration parameters for HNSW index rebuild backoff and failure escalation.
- Impl-Block (Trait `Default`) `crates/contextra-db/src/background_workers/config.rs:25`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `base_delay` | `Duration` | `crates/contextra-db/src/background_workers/config.rs:18` |
| `max_delay` | `Duration` | `crates/contextra-db/src/background_workers/config.rs:20` |
| `alert_threshold` | `u32` | `crates/contextra-db/src/background_workers/config.rs:22` |

#### `trait OrphanCleanupIndex`
- Ort: `crates/contextra-db/src/background_workers/config.rs:52`
```rust
pub trait OrphanCleanupIndex: Send + Sync
```
Dokumentation: Abstraction trait over vector indexes capable of connectivity check and rebuild.

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn check_connectivity(&self) -> contextra_types::Result<()>` | `crates/contextra-db/src/background_workers/config.rs:53` |  |
| `fn rebuild( &self, ) -> std::pin::Pin<Box<dyn std::future::Future<Output = contextra_types::Result<()>> + Send + '_>>` | `crates/contextra-db/src/background_workers/config.rs:54` |  |

#### `fn calculate_rebuild_cooldown`
- Ort: `crates/contextra-db/src/background_workers/config.rs:36`
```rust
pub fn calculate_rebuild_cooldown( consecutive_failures: u32, base_delay: Duration, max_delay: Duration, ) -> Duration
```
Dokumentation: Helper calculating exponential backoff cooldown given consecutive failures.

### Modul `background_workers::expiry_workers`

#### `fn start_decay_cleanup_worker`
- Ort: `crates/contextra-db/src/background_workers/expiry_workers.rs:86`
```rust
pub fn start_decay_cleanup_worker<S: StorageEngine, V: VectorIndex>( collection: Arc<Collection<S, V>>, decay_config: DecayControllerConfig, interval: Duration, cancel_token: tokio_util::sync::CancellationToken, ) -> tokio::task::JoinHandle<()>
```
Dokumentation: Starts a background task for decay-controller-driven importance-score eviction. Nur aktiv wenn `background-maintenance` Feature-Flag gesetzt.
- Gate: `#[cfg(feature = "background-maintenance")]`

#### `fn start_expiry_cleanup_worker`
- Ort: `crates/contextra-db/src/background_workers/expiry_workers.rs:25`
```rust
pub fn start_expiry_cleanup_worker<S: StorageEngine>( collection: Arc<Collection<S>>, interval: Duration, cancel_token: tokio_util::sync::CancellationToken, ) -> tokio::task::JoinHandle<()>
```
Dokumentation: Starts a background task to periodically clean up expired documents with TTL.

#### `fn start_expiry_reaper`
- Ort: `crates/contextra-db/src/background_workers/expiry_workers.rs:75`
```rust
pub fn start_expiry_reaper<S: StorageEngine>( collection: Arc<Collection<S>>, interval: Duration, cancel_token: tokio_util::sync::CancellationToken, ) -> tokio::task::JoinHandle<()>
```
Dokumentation: Deprecated legacy alias for `start_expiry_cleanup_worker`.
- **deprecated**

#### `fn start_thermostat_reaper`
- Ort: `crates/contextra-db/src/background_workers/expiry_workers.rs:114`
```rust
pub fn start_thermostat_reaper<S: StorageEngine, V: VectorIndex>( collection: Arc<Collection<S, V>>, decay_config: DecayControllerConfig, interval: Duration, cancel_token: tokio_util::sync::CancellationToken, ) -> tokio::task::JoinHandle<()>
```
Dokumentation: Deprecated legacy alias for `start_decay_cleanup_worker`.
- **deprecated**
- Gate: `#[cfg(feature = "background-maintenance")]`

#### `const MAX_EXPIRED_PER_TICK`
- Ort: `crates/contextra-db/src/background_workers/expiry_workers.rs:22`
```rust
pub const MAX_EXPIRED_PER_TICK: usize = 100
```
Dokumentation: Maximum number of expired documents processed in a single expiry cleanup tick.

### Modul `background_workers::hyperedge_worker`

#### `struct DeferredHyperedgeQueue`
- Ort: `crates/contextra-db/src/background_workers/hyperedge_worker.rs:26`
```rust
pub struct DeferredHyperedgeQueue
```
Dokumentation: Thread-safe FIFO queue for storing hyperedges whose cascade invalidation was deferred to the background worker due to fan-out limits (H5 / AK-6).
- Impl-Block `crates/contextra-db/src/background_workers/hyperedge_worker.rs:30`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new() -> Self` | `crates/contextra-db/src/background_workers/hyperedge_worker.rs:32` | Creates a new empty `DeferredHyperedgeQueue`. |
| `pub async fn enqueue(&self, ids: impl IntoIterator<Item = HyperEdgeId>)` | `crates/contextra-db/src/background_workers/hyperedge_worker.rs:39` | Enqueues hyperedge IDs for deferred background tombstoning. |
| `pub async fn drain_up_to(&self, max: usize) -> Vec<HyperEdgeId>` | `crates/contextra-db/src/background_workers/hyperedge_worker.rs:45` | Drains up to `max` hyperedge IDs from the front of the queue (FIFO). |

#### `fn start_hyperedge_cascade_deferred_worker`
- Ort: `crates/contextra-db/src/background_workers/hyperedge_worker.rs:53`
```rust
pub fn start_hyperedge_cascade_deferred_worker<S: StorageEngine>( collection: Arc<Collection<S>>, queue: Arc<DeferredHyperedgeQueue>, interval: Duration, cancel_token: tokio_util::sync::CancellationToken, ) -> tokio::task::JoinHandle<()>
```
Dokumentation: Starts a background task to process deferred hyperedge tombstones.

#### `const MAX_DEFERRED_HYPEREDGES_PER_TICK`
- Ort: `crates/contextra-db/src/background_workers/hyperedge_worker.rs:21`
```rust
pub const MAX_DEFERRED_HYPEREDGES_PER_TICK: usize = 1_000
```
Dokumentation: Maximum number of deferred hyperedges processed in a single worker tick.

### Modul `background_workers::orphan_workers`

#### `fn reap_orphans_step`
- Ort: `crates/contextra-db/src/background_workers/orphan_workers.rs:24`
```rust
pub fn reap_orphans_step<T: Clone + Send + Sync + 'static>( buffer: &TxBuffer<T>, limit: usize, ) -> Vec<contextra_types::TxId>
```
Dokumentation: Performs a single synchronous step of orphan transaction reaping.

#### `fn start_orphan_cleanup_worker`
- Ort: `crates/contextra-db/src/background_workers/orphan_workers.rs:171`
```rust
pub fn start_orphan_cleanup_worker<T: Clone + Send + Sync + 'static>( buffer: Arc<TxBuffer<T>>, hnsw_index: Arc<contextra_vector::hnsw::HnswIndex>, interval: Duration, cancel_token: tokio_util::sync::CancellationToken, ) -> tokio::task::JoinHandle<()>
```
Dokumentation: Starts a background task to periodically clean up orphan transactions. This worker handles the cleanup of transactions that have exceeded their configured timeout without being committed or rolled back.

#### `fn start_orphan_cleanup_worker_with_config`
- Ort: `crates/contextra-db/src/background_workers/orphan_workers.rs:32`
```rust
pub fn start_orphan_cleanup_worker_with_config< T: Clone + Send + Sync + 'static, I: OrphanCleanupIndex + 'static, >( buffer: Arc<TxBuffer<T>>, hnsw_index: Arc<I>, interval: Duration, backoff_config: OrphanCleanupBackoffConfig, cancel_token: tokio_util::sync::CancellationToken, ) -> (tokio::task::JoinHandle<()>, Arc<AtomicU32>)
```
Dokumentation: Starts a background task to periodically clean up orphan transactions and manage HNSW rebuilds with exponential backoff.

#### `fn start_orphan_reaper`
- Ort: `crates/contextra-db/src/background_workers/orphan_workers.rs:189`
```rust
pub fn start_orphan_reaper<T: Clone + Send + Sync + 'static>( buffer: Arc<TxBuffer<T>>, hnsw_index: Arc<contextra_vector::hnsw::HnswIndex>, interval: Duration, cancel_token: tokio_util::sync::CancellationToken, ) -> tokio::task::JoinHandle<()>
```
Dokumentation: Deprecated legacy alias for `start_orphan_cleanup_worker`.
- **deprecated**

#### `const MAX_ORPHANS_PER_TICK`
- Ort: `crates/contextra-db/src/background_workers/orphan_workers.rs:21`
```rust
pub const MAX_ORPHANS_PER_TICK: usize = 100
```
Dokumentation: Maximum number of orphan transactions processed in a single worker tick to avoid starving foreground operations.

### Modul `chunker`

#### `struct MarkdownChunker`
- Ort: `crates/contextra-db/src/chunker/mod.rs:37`
```rust
pub struct MarkdownChunker
```
Dokumentation: A deterministic chunker for markdown files respecting structural blocks (headings, code blocks, tables, paragraphs) and UTF-8 safety.
- Impl-Block `crates/contextra-db/src/chunker/mod.rs:132`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(config: ChunkerConfig) -> Self` | `crates/contextra-db/src/chunker/mod.rs:134` | Creates a new MarkdownChunker with the given configuration. |
| `pub fn with_defaults() -> Self` | `crates/contextra-db/src/chunker/mod.rs:139` | Creates a new MarkdownChunker with default configuration. |
| `pub fn chunk(&self, doc_id: DocId, markdown: &str) -> Vec<ContextChunk>` | `crates/contextra-db/src/chunker/mod.rs:144` | Chunks a Markdown document into semantically coherent pieces. |

### Modul `chunker::config`

#### `struct ChunkerConfig`
- Ort: `crates/contextra-db/src/chunker/config.rs:16`
```rust
pub struct ChunkerConfig
```
Dokumentation: Configuration for the Markdown chunker.
- Impl-Block (Trait `Default`) `crates/contextra-db/src/chunker/config.rs:28`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `max_tokens` | `usize` | `crates/contextra-db/src/chunker/config.rs:19` |
| `min_tokens` | `usize` | `crates/contextra-db/src/chunker/config.rs:21` |
| `include_breadcrumbs` | `bool` | `crates/contextra-db/src/chunker/config.rs:23` |
| `split_levels` | `Vec<u8>` | `crates/contextra-db/src/chunker/config.rs:25` |

### Modul `chunker::estimate`

#### `fn estimate_tokens`
- Ort: `crates/contextra-db/src/chunker/estimate.rs:20`
```rust
pub fn estimate_tokens(text: &str) -> usize
```
Dokumentation: Schätzt die Token-Anzahl eines Textes mit heuristischen Regeln.

### Modul `chunker::windows`

#### `fn chunk_text`
- Ort: `crates/contextra-db/src/chunker/windows.rs:63`
```rust
pub fn chunk_text(text: &str, chunk_size: usize) -> Vec<&str>
```
Dokumentation: Segmentiert Text in Abschnitte bis zur Maximallänge ohne Überlappung.

#### `fn chunk_text_with_overlap`
- Ort: `crates/contextra-db/src/chunker/windows.rs:16`
```rust
pub fn chunk_text_with_overlap<'a>( text: &'a str, window_chars: usize, overlap_chars: usize, ) -> Vec<&'a str>
```
Dokumentation: Erzeugt überlappende Text-Chunks mit definierter Fenstergröße und Überlappung.

### Modul `cognition`

#### `mod bridge`
- Ort: `crates/contextra-db/src/cognition/mod.rs:3`
```rust
pub mod bridge
```

### Modul `cognition::bridge`

#### `struct DbDocumentSource`
- Ort: `crates/contextra-db/src/cognition/bridge.rs:6`
```rust
pub struct DbDocumentSource<'a, S, V>
```
Dokumentation: Brückenadapter zur Anbindung von Collections an Cognition-Pipelines.
- Impl-Block `crates/contextra-db/src/cognition/bridge.rs:10`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(collection: &'a crate::collection::Collection<S, V>) -> Self` | `crates/contextra-db/src/cognition/bridge.rs:12` | Erstellt eine neue Instanz gebunden an eine Collection. |

### Modul `collection`

#### `mod crud`
- Ort: `crates/contextra-db/src/collection/mod.rs:21`
```rust
pub mod crud
```

#### `mod kv_lock`
- Ort: `crates/contextra-db/src/collection/mod.rs:18`
```rust
pub mod kv_lock
```

#### `mod maintenance`
- Ort: `crates/contextra-db/src/collection/mod.rs:23`
```rust
pub mod maintenance
```

#### `mod query_builder`
- Ort: `crates/contextra-db/src/collection/mod.rs:25`
```rust
pub mod query_builder
```

#### `mod relate`
- Ort: `crates/contextra-db/src/collection/mod.rs:27`
```rust
pub mod relate
```

#### `mod search`
- Ort: `crates/contextra-db/src/collection/mod.rs:29`
```rust
pub mod search
```

#### `mod tx`
- Ort: `crates/contextra-db/src/collection/mod.rs:31`
```rust
pub mod tx
```

#### `struct Collection`
- Ort: `crates/contextra-db/src/collection/mod.rs:278`
```rust
pub struct Collection<S: StorageEngine = LsmStorage, V: VectorIndex = HnswIndex>
```
Dokumentation: # Concurrency & Lock Hierarchy Lock acquisition within `Collection` follows strict ordering to prevent deadlocks: 1. `Collection::kv_locks` (`KvKeyLocks`): Key-granular sharded locks protecting key-level mutations (`insert`, `update`, `delete`, `relate`). 2. `Collection::embedder` (`parking_lot::RwLock`): Read/write lock for the configured `TextEmbeddingEngine`. Never acquired before `kv_locks` if both are needed. A logically isolated collection of documents (namespace). Each collection provides its own vector index and inverted text index, while sharing the underlying LSM-Tree storage with ot
- Impl-Block `crates/contextra-db/src/collection/crud/auto_extraction.rs:168`
- Impl-Block `crates/contextra-db/src/collection/crud/delete.rs:12`
- Impl-Block `crates/contextra-db/src/collection/crud/insert.rs:17`
- Impl-Block `crates/contextra-db/src/collection/crud/internal.rs:54`
- Impl-Block `crates/contextra-db/src/collection/crud/kv.rs:13`
- Impl-Block `crates/contextra-db/src/collection/crud/links.rs:12`
- Impl-Block `crates/contextra-db/src/collection/crud/read.rs:24`
- Impl-Block `crates/contextra-db/src/collection/crud/update.rs:15`
- Impl-Block `crates/contextra-db/src/collection/maintenance.rs:25`
- Impl-Block `crates/contextra-db/src/collection/maintenance.rs:890`
- Impl-Block (Trait `Clone`) `crates/contextra-db/src/collection/mod.rs:308`
- Impl-Block `crates/contextra-db/src/collection/mod.rs:336`
- Impl-Block `crates/contextra-db/src/collection/mod.rs:360`
- Impl-Block `crates/contextra-db/src/collection/query_builder/builder.rs:307`
- Impl-Block `crates/contextra-db/src/collection/relate.rs:12`
- Impl-Block `crates/contextra-db/src/collection/search/basic.rs:18`
- Impl-Block `crates/contextra-db/src/collection/search/filtered.rs:22`
- Impl-Block `crates/contextra-db/src/collection/search/hybrid/mod.rs:18`
- Impl-Block `crates/contextra-db/src/collection/search/hybrid/query.rs:37`
- Impl-Block `crates/contextra-db/src/collection/search/hydrate.rs:15`
- Impl-Block `crates/contextra-db/src/collection/tx.rs:24`
- Impl-Block `crates/contextra-db/src/export.rs:87`
- Impl-Block `crates/contextra-db/src/import.rs:33`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `name` | `String` | `crates/contextra-db/src/collection/mod.rs:279` |
| `prefix` | `Vec<u8>` | `crates/contextra-db/src/collection/mod.rs:280` |
| `index` | `Arc<V>` | `crates/contextra-db/src/collection/mod.rs:281` |
| `text_index` | `InvertedIndex<S>` | `crates/contextra-db/src/collection/mod.rs:282` |
| `graph_index` | `Arc<CsrGraph>` | `crates/contextra-db/src/collection/mod.rs:283` |
| `storage` | `Arc<S>` | `crates/contextra-db/src/collection/mod.rs:284` |
| `next_tx` | `Arc<AtomicU64>` | `crates/contextra-db/src/collection/mod.rs:285` |
| `dimension` | `usize` | `crates/contextra-db/src/collection/mod.rs:286` |
| `embedder` | `parking_lot::RwLock<Option<Arc<dyn TextEmbeddingEngine>>>` | `crates/contextra-db/src/collection/mod.rs:287` |
| `consolidation_guard` | `Arc<tokio::sync::Mutex<()>>` | `crates/contextra-db/src/collection/mod.rs:288` |
| `kv_locks` | `Arc<kv_lock::KvKeyLocks>` | `crates/contextra-db/src/collection/mod.rs:289` |
| `mutations_since_community_detection` | `Arc<AtomicU64>` | `crates/contextra-db/src/collection/mod.rs:291` |
| `community_detection_trigger_threshold` | `Arc<AtomicU64>` | `crates/contextra-db/src/collection/mod.rs:293` |
| `consolidation_in_progress` | `Arc<AtomicBool>` | `crates/contextra-db/src/collection/mod.rs:295` |
| `config` | `parking_lot::RwLock<CollectionConfig>` | `crates/contextra-db/src/collection/mod.rs:297` |
| `clock` | `parking_lot::RwLock<Arc<dyn Clock>>` | `crates/contextra-db/src/collection/mod.rs:302` |
| `metrics` | `parking_lot::RwLock<Arc<dyn MetricsSink>>` | `crates/contextra-db/src/collection/mod.rs:304` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn with_auto_extraction( self, generator: Arc<dyn LlmTextGenerator>, config: AutoExtractionConfig, ) -> Self` | `crates/contextra-db/src/collection/crud/auto_extraction.rs:170` | Configures auto extraction with an LLM generator and config (builder pattern). |
| `pub fn set_auto_extraction( &self, generator: Arc<dyn LlmTextGenerator>, config: AutoExtractionConfig, )` | `crates/contextra-db/src/collection/crud/auto_extraction.rs:180` | Sets or updates the auto extraction generator and config for this collection. |
| `pub fn set_auto_extraction_with_mode( &self, generator: Arc<dyn LlmTextGenerator>, config: AutoExtractionConfig, extractor_mode: ExtractorMode, )` | `crates/contextra-db/src/collection/crud/auto_extraction.rs:189` | Sets or updates auto extraction with an explicit extractor mode. |
| `pub fn set_auto_extraction_config(&self, config: AutoExtractionConfig)` | `crates/contextra-db/src/collection/crud/auto_extraction.rs:202` | Sets or updates auto extraction without an external LLM generator (suitable for RuleBased mode). |
| `pub fn auto_extraction_config( &self, ) -> Option<(Arc<dyn LlmTextGenerator>, AutoExtractionConfig)>` | `crates/contextra-db/src/collection/crud/auto_extraction.rs:208` | Returns the configured auto extraction generator and config, if present. |
| `pub fn auto_extraction_config_with_extractor( &self, ) -> Option<( Arc<dyn LlmTextGenerator>, AutoExtractionConfig, ExtractorMode, )>` | `crates/contextra-db/src/collection/crud/auto_extraction.rs:219` | Returns the configured auto extraction generator, config, and extractor mode. |
| `pub async fn delete(&self, id: &str) -> Result<()>` | `crates/contextra-db/src/collection/crud/delete.rs:15` | Deletes a document from the collection by its ID. |
| `pub async fn delete_op( &self, db_tx: &mut crate::transaction::DbTransaction<S, V>, id: &str, ) -> Result<()>` | `crates/contextra-db/src/collection/crud/delete.rs:38` | Dokumentation für `delete_op` im Rahmen der Collection- und Suchabstraktion. |
| `pub async fn insert_text_only( &self, id: &str, text: &str, mut metadata: Option<serde_json::Value>, ) -> Result<()>` | `crates/contextra-db/src/collection/crud/insert.rs:20` | Inserts a text document, automatically generating its embedding. |
| `pub async fn upsert_text_only( &self, id: &str, text: &str, mut metadata: Option<serde_json::Value>, ) -> Result<()>` | `crates/contextra-db/src/collection/crud/insert.rs:62` | Upserts a text document, automatically generating its embedding. |
| `pub async fn insert_with_ttl( &self, id: &str, embedding: &[f32], metadata: Option<serde_json::Value>, ttl_committed_ops: u64, ) -> Result<()>` | `crates/contextra-db/src/collection/crud/insert.rs:104` | Inserts a document with a Sequence-based Time-To-Live (TTL in committed ops). |
| `pub async fn insert_typed( &self, id: &str, embedding: &[f32], memory_type: contextra_types::MemoryType, metadata: Option<serde_json::Value>, ) -> Result<()>` | `crates/contextra-db/src/collection/crud/insert.rs:135` | Speichert ein Dokument mit expliziter kognitiver Gedächtnisklassifikation. |
| `pub async fn insert( &self, id: &str, embedding: &[f32], metadata: Option<serde_json::Value>, ) -> Result<()>` | `crates/contextra-db/src/collection/crud/insert.rs:172` | Inserts a document with an embedding and optional metadata. |
| `pub async fn insert_op( &self, db_tx: &crate::transaction::DbTransaction<S, V>, id: &str, embedding: &[f32], metadata: Option<serde_json::Value>, ) -> Result<()>` | `crates/contextra-db/src/collection/crud/insert.rs:305` | Dokumentation für `insert_op` im Rahmen der Collection- und Suchabstraktion. |
| `pub async fn insert_many( &self, docs: &[(String, Vec<f32>, Option<serde_json::Value>)], ) -> Result<()>` | `crates/contextra-db/src/collection/crud/insert.rs:372` | Inserts multiple documents in a single atomic transaction under key-granular locks. |
| `pub async fn upsert( &self, id: &str, embedding: &[f32], metadata: Option<serde_json::Value>, ) -> Result<()>` | `crates/contextra-db/src/collection/crud/insert.rs:448` | Upserts a document (inserts if missing, updates if exists) atomically. |
| `pub async fn upsert_many( &self, docs: &[(String, Vec<f32>, Option<serde_json::Value>)], ) -> Result<()>` | `crates/contextra-db/src/collection/crud/insert.rs:494` | Upserts multiple documents in a single transaction. |
| `pub async fn put_kv(&self, id: &str, value: &serde_json::Value) -> Result<()>` | `crates/contextra-db/src/collection/crud/kv.rs:16` | Stores a non-vector key-value entry directly in LSM storage without touching vector, text, or graph indices. |
| `pub async fn put_kv_if_absent(&self, id: &str, value: &serde_json::Value) -> Result<()>` | `crates/contextra-db/src/collection/crud/kv.rs:31` | Stores a non-vector key-value entry directly in LSM storage only if the key does not already exist. Returns `ContextraError::Conflict` if the key is already pre |
| `pub async fn get_kv(&self, id: &str) -> Result<Option<serde_json::Value>>` | `crates/contextra-db/src/collection/crud/kv.rs:59` | Retrieves a key-value entry directly from LSM storage. |
| `pub async fn link_memories( &self, from: DocId, to: DocId, relation: contextra_types::domain::LinkRelation, ) -> Result<()>` | `crates/contextra-db/src/collection/crud/links.rs:15` | Links two memories together with a specific relation (Zettelkasten A-MEM). |
| `pub async fn get_links( &self, doc_id: DocId, ) -> Result<Vec<contextra_types::domain::MemoryLink>>` | `crates/contextra-db/src/collection/crud/links.rs:216` | Retrieves all links for a given document. |
| `pub async fn snapshot_seq(&self) -> Result<u64>` | `crates/contextra-db/src/collection/crud/read.rs:30` |  |
| `pub async fn get(&self, id: &str) -> Result<Option<crate::Document>>` | `crates/contextra-db/src/collection/crud/read.rs:36` | Retrieves a document by its user-provided string ID. |
| `pub async fn get_at_snapshot(&self, id: &str, seq_no: u64) -> Result<Option<crate::Document>>` | `crates/contextra-db/src/collection/crud/read.rs:42` | Retrieves a document at a specific snapshot point. |
| `pub async fn get_doc_tx(&self, doc_id: DocId) -> Result<Option<TxId>>` | `crates/contextra-db/src/collection/crud/read.rs:62` | Returns the latest transaction ID for a document (via `updated_at_tx` or `created_at_tx`). |
| `pub async fn scan_prefix( &self, prefix: &str, limit: Option<usize>, ) -> Result<Vec<(String, serde_json::Value)>>` | `crates/contextra-db/src/collection/crud/read.rs:88` | Scans documents in the collection that match a given key prefix. Caps maximum results to `MAX_SCAN_RESULTS_DEFAULT` to protect against unbounded memory growth. |
| `pub async fn scan( &self, start: std::ops::Bound<&[u8]>, end: std::ops::Bound<&[u8]>, limit: Option<usize>, ) -> Result<Vec<(String, serde_json::Value)>>` | `crates/contextra-db/src/collection/crud/read.rs:174` | Performs a range scan of documents in the collection. |
| `pub async fn get_tracked(&self, tx: TxId, id: &str) -> Result<Option<StoredDocument>>` | `crates/contextra-db/src/collection/crud/read.rs:291` | Fetches a document by user ID and tracks the read in the active transaction's `ReadSet`. |
| `pub async fn get_at_seq_tracked( &self, tx: TxId, id: &str, max_seq: u64, ) -> Result<Option<StoredDocument>>` | `crates/contextra-db/src/collection/crud/read.rs:302` | Fetches a document snapshot bounded by `seq` and tracks the read in `tx`'s `ReadSet`. |
| `pub async fn update( &self, id: &str, embedding: &[f32], metadata: Option<serde_json::Value>, ) -> Result<()>` | `crates/contextra-db/src/collection/crud/update.rs:18` | Updates an existing document in the collection. |
| `pub async fn update_op( &self, db_tx: &crate::transaction::DbTransaction<S, V>, id: &str, embedding: &[f32], metadata: Option<serde_json::Value>, ) -> Result<()>` | `crates/contextra-db/src/collection/crud/update.rs:54` | Dokumentation für `update_op` im Rahmen der Collection- und Suchabstraktion. |
| `pub async fn evict_decayed_chunks( &self, decay_controller: &AdaptiveDecayController, max_per_tick: usize, ) -> Result<usize>` | `crates/contextra-db/src/collection/maintenance.rs:30` | Führt einen Importance-Score-Sweep via Adaptive Decay Controller durch. Evictet Chunks deren effective_score unter eviction_threshold liegt. Gibt Anzahl der evi |
| `pub async fn repair(&self) -> Result<()>` | `crates/contextra-db/src/collection/maintenance.rs:151` | Repairs the index by re-syncing with the storage. Scans the storage for any documents that are missing from the index and reconciles them. This is critical for  |
| `pub async fn stats(&self) -> Result<contextra_ports::VectorIndexStats>` | `crates/contextra-db/src/collection/maintenance.rs:445` | Returns statistics for the collection's vector index. |
| `pub async fn reap_expired_documents(&self, max_expired: usize) -> Result<usize>` | `crates/contextra-db/src/collection/maintenance.rs:452` | Rebuilds the HNSW index from storage. Scans the collection for documents with expired sequence-based TTLs and deletes them in batches. |
| `pub async fn trigger_expiry_cleanup(&self) -> Result<usize>` | `crates/contextra-db/src/collection/maintenance.rs:555` | Scans the collection for documents with expired TTLs or decayed importance scores and deletes them. Reads `created_at_ms` (or `timestamp_ms`) and `ttl_ms` from  |
| `pub async fn update_document_importance( &self, doc_id: &str, importance_score: f32, model_id: &str, ) -> Result<()>` | `crates/contextra-db/src/collection/maintenance.rs:685` | Aktualisiert den Importance-Score sowie die Provenance (Modell-ID) in den Metadaten. Diese Funktion entkoppelt die LLM-Bewertung (die z.B. über Ollama geschieht |
| `pub async fn run_community_detection(&self) -> Result<Vec<CommunityAssignment>>` | `crates/contextra-db/src/collection/maintenance.rs:760` | Runs Leiden Community Detection on the collection's graph index and persists the resulting assignments in storage using TxId allocation. |
| `pub async fn run_community_detection_with_config( &self, config: &CommunityDetectionConfig, ) -> Result<Vec<CommunityAssignment>>` | `crates/contextra-db/src/collection/maintenance.rs:768` | Runs Leiden Community Detection with custom configuration and persists the resulting assignments in storage using TxId allocation. |
| `pub async fn get_community(&self, entity_id: EntityId) -> Result<Option<u64>>` | `crates/contextra-db/src/collection/maintenance.rs:795` | Retrieves the persisted community ID for a given entity. |
| `pub async fn get_communities_batch( &self, entity_ids: &[EntityId], ) -> Result<std::collections::HashMap<EntityId, u64>>` | `crates/contextra-db/src/collection/maintenance.rs:813` | Retrieves community assignments for a batch of entity IDs in a single operation. |
| `pub async fn drop_collection(&self) -> Result<()>` | `crates/contextra-db/src/collection/maintenance.rs:852` | Removes all data belonging to this collection from storage. |
| `pub async fn run_percolation_check( &self, config: &contextra_graph::percolation::PercolationConfig, ) -> Result<PercolationResult>` | `crates/contextra-db/src/collection/maintenance.rs:894` | Überprüft die Perkolations-Gesundheit des Wissensgraphen und löst bei Unterschreitung des Schwellenwerts automatisch einen Re-Bonding-Pass aus. |
| `pub fn with_hnsw( name: String, storage: Arc<S>, index: Arc<HnswIndex>, graph_index: Arc<CsrGraph>, next_tx: Arc<AtomicU64>, dimension: usize, language: Language, ) -> Self` | `crates/contextra-db/src/collection/mod.rs:338` | Convenience constructor for creating a `Collection` with `HnswIndex`. |
| `pub fn new( name: String, storage: Arc<S>, index: Arc<V>, graph_index: Arc<CsrGraph>, next_tx: Arc<AtomicU64>, dimension: usize, language: Language, ) -> Self` | `crates/contextra-db/src/collection/mod.rs:369` | Creates a new `Collection` instance with explicit language configuration. The `language` parameter controls the BM25 tokenizer. Use `Language::German` for Germa |
| `pub fn with_metrics_sink(self, metrics: Arc<dyn MetricsSink>) -> Self` | `crates/contextra-db/src/collection/mod.rs:413` | Sets or overrides the metrics sink implementation (builder version). |
| `pub fn set_metrics_sink(&self, metrics: Arc<dyn MetricsSink>)` | `crates/contextra-db/src/collection/mod.rs:419` | Sets or overrides the metrics sink implementation. |
| `pub fn metrics_sink(&self) -> Arc<dyn MetricsSink>` | `crates/contextra-db/src/collection/mod.rs:424` | Returns a cloned reference to the active metrics sink. |
| `pub fn with_clock(self, clock: Arc<dyn Clock>) -> Self` | `crates/contextra-db/src/collection/mod.rs:429` | Sets or overrides the clock implementation (builder version). |
| `pub fn set_clock(&self, clock: Arc<dyn Clock>)` | `crates/contextra-db/src/collection/mod.rs:435` | Sets or overrides the clock implementation. |
| `pub fn clock(&self) -> Arc<dyn Clock>` | `crates/contextra-db/src/collection/mod.rs:440` | Returns a cloned reference to the injected clock. |
| `pub fn with_config(self, config: CollectionConfig) -> Self` | `crates/contextra-db/src/collection/mod.rs:445` | Sets or updates the collection configuration. |
| `pub fn set_config(&self, config: CollectionConfig)` | `crates/contextra-db/src/collection/mod.rs:451` | Sets or updates the collection configuration. |
| `pub fn config(&self) -> CollectionConfig` | `crates/contextra-db/src/collection/mod.rs:456` | Returns a copy of the current collection configuration. |
| `pub fn with_pressure_receiver( self, pressure_rx: tokio::sync::watch::Receiver<contextra_store::SystemPressure>, ) -> Self` | `crates/contextra-db/src/collection/mod.rs:461` | Sets or overrides the pressure receiver channel. |
| `pub fn set_pressure_receiver( &self, pressure_rx: tokio::sync::watch::Receiver<contextra_store::SystemPressure>, )` | `crates/contextra-db/src/collection/mod.rs:470` | Sets or overrides the pressure receiver channel. |
| `pub fn consolidation_in_progress(&self) -> Arc<AtomicBool>` | `crates/contextra-db/src/collection/mod.rs:478` | Returns a reference handle to the collection's consolidation lock flag. |
| `pub fn graph_index(&self) -> Arc<CsrGraph>` | `crates/contextra-db/src/collection/mod.rs:483` | Returns the CSR graph index for this collection. |
| `pub fn graph_port(&self) -> Arc<dyn contextra_ports::GraphIndex>` | `crates/contextra-db/src/collection/mod.rs:488` | Returns the abstract graph index port for this collection (M3 Trait-Decoupling). |
| `pub fn embedder(&self) -> Option<Arc<dyn TextEmbeddingEngine>>` | `crates/contextra-db/src/collection/mod.rs:493` | Returns a clone of the configured embedder engine, if set. |
| `pub fn with_embedder(self, embedder: Arc<dyn TextEmbeddingEngine>) -> Self` | `crates/contextra-db/src/collection/mod.rs:499` | Sets the text embedder for this collection (consuming version). |
| `pub async fn set_embedder(&self, embedder: Arc<dyn TextEmbeddingEngine>) -> Result<()>` | `crates/contextra-db/src/collection/mod.rs:509` | Configures the text embedder for this collection. |
| `pub fn vector_index(&self) -> &Arc<V>` | `crates/contextra-db/src/collection/mod.rs:516` | Returns a reference to the vector index of this collection. |
| `pub fn namespaced_key(&self, key: &[u8], key_type: u8) -> Vec<u8>` | `crates/contextra-db/src/collection/mod.rs:522` | Internal helper to generate namespaced keys. key_type: 0 = user key, 1 = docid mapping, 2 = relationship, 3 = tx intent, 4 = system/community |
| `pub fn storage(&self) -> &Arc<S>` | `crates/contextra-db/src/collection/mod.rs:555` | Returns a reference to the underlying storage engine. |
| `pub fn user_key_prefix(&self) -> Vec<u8>` | `crates/contextra-db/src/collection/mod.rs:560` | Returns the namespaced prefix for user document keys in this collection. |
| `pub fn doc_key_prefix(&self) -> Vec<u8>` | `crates/contextra-db/src/collection/mod.rs:565` | Returns the namespaced prefix for internal doc_id keys in this collection. |
| `pub fn name(&self) -> &str` | `crates/contextra-db/src/collection/mod.rs:571` | Returns the name of the collection. |
| `pub async fn len(&self) -> usize` | `crates/contextra-db/src/collection/mod.rs:577` | Returns the number of documents in the collection. |
| `pub fn dimension(&self) -> usize` | `crates/contextra-db/src/collection/mod.rs:583` | Returns the vector dimension for this collection. |
| `pub async fn is_empty(&self) -> bool` | `crates/contextra-db/src/collection/mod.rs:589` | Returns true if the collection is empty. |
| `pub async fn load_index(&self) -> Result<()>` | `crates/contextra-db/src/collection/mod.rs:595` | Performs a range scan of documents in the collection. |
| `pub async fn migrate_doc_keys_v1(&self) -> Result<u64>` | `crates/contextra-db/src/collection/mod.rs:629` | Migrates old doc_keys (with Embedding) to new doc_keys (only Metadata). Safe to call multiple times (idempotent). |
| `pub async fn load_text_stats(&self) -> Result<()>` | `crates/contextra-db/src/collection/mod.rs:666` | Loads text index statistics from storage. |
| `pub fn set_community_detection_trigger_threshold(&self, threshold: u64)` | `crates/contextra-db/src/collection/mod.rs:671` | Sets the mutation threshold for auto-triggered community detection (0 = disabled). |
| `pub fn with_community_detection_trigger_threshold(self, threshold: u64) -> Self` | `crates/contextra-db/src/collection/mod.rs:677` | Builder method to set the mutation threshold for auto-triggered community detection (0 = disabled). |
| `pub fn community_detection_trigger_threshold(&self) -> u64` | `crates/contextra-db/src/collection/mod.rs:683` | Returns the configured mutation threshold for auto-triggered community detection. |
| `pub fn mutations_since_community_detection(&self) -> u64` | `crates/contextra-db/src/collection/mod.rs:689` | Returns the number of graph mutations since the last community detection execution. |
| `pub fn query(&self) -> HybridQueryBuilder<'_, S, V>` | `crates/contextra-db/src/collection/query_builder/builder.rs:309` | Recommended unified entrypoint for search queries using fluent `HybridQueryBuilder`. |
| `pub async fn relate_with_provenance( &self, from: &str, to: &str, label: &str, source_doc_id: Option<&str>, ) -> Result<()>` | `crates/contextra-db/src/collection/relate.rs:16` | Creates a directional relationship between two documents in the collection with optional source provenance document ID. |
| `pub async fn relate(&self, from: &str, to: &str, label: &str) -> Result<()>` | `crates/contextra-db/src/collection/relate.rs:87` | Creates a directional relationship between two documents in the collection. |
| `pub async fn relate_bidirectional(&self, from: &str, to: &str, label: &str) -> Result<()>` | `crates/contextra-db/src/collection/relate.rs:93` | Creates a bidirectional relationship atomically. |
| `pub async fn relate_n_ary( &self, predicate: &str, participants: &[(&str, &str)], source_doc_id: Option<&str>, ) -> Result<contextra_graph::hyperedge::HyperEdgeId>` | `crates/contextra-db/src/collection/relate.rs:107` | Creates an n-ary hyperedge relationship connecting two or more participants. # Deadlock Prevention & Canonical Lock Ordering `participants` contain document IDs |
| `pub async fn search( &self, query_embedding: &[f32], k: usize, ) -> Result<Vec<crate::SearchResult>>` | `crates/contextra-db/src/collection/search/basic.rs:23` | Performs semantic k-NN search over stored embeddings. |
| `pub async fn search_with_filter( &self, query: &[f32], k: usize, filter: Option<MetadataFilter>, ) -> Result<Vec<crate::SearchResult>>` | `crates/contextra-db/src/collection/search/basic.rs:41` | Performs semantic search with an advanced metadata filter. |
| `pub async fn search_with_filter_expr( &self, query: &[f32], k: usize, filter: Option<FilterExpr>, ) -> Result<Vec<crate::SearchResult>>` | `crates/contextra-db/src/collection/search/basic.rs:58` | Performs semantic search with an advanced metadata filter expression (`FilterExpr`). |
| `pub async fn search_text( &self, query_text: &str, k: usize, ) -> Result<Vec<crate::SearchResult>>` | `crates/contextra-db/src/collection/search/basic.rs:104` | Performs semantic search using a raw text query (automatically embedded). |
| `pub async fn search_filtered( &self, query: &[f32], k: usize, filter: Option<&(dyn Fn(DocId) -> bool + Send + Sync)>, ) -> Result<Vec<crate::SearchResult>>` | `crates/contextra-db/src/collection/search/filtered.rs:181` |  |
| `pub async fn search_filtered_at( &self, query: &[f32], k: usize, filter: Option<&(dyn Fn(DocId) -> bool + Send + Sync)>, seq: u64, ) -> Result<Vec<crate::SearchResult>>` | `crates/contextra-db/src/collection/search/filtered.rs:198` | Performs filtered semantic vector search at a specific MVCC sequence number. |
| `pub async fn hybrid_search( &self, text: &str, vector: &[f32], k: usize, anchor_entities: Option<&[contextra_types::EntityId]>, ) -> Result<Vec<crate::SearchResult>>` | `crates/contextra-db/src/collection/search/hybrid/mod.rs:23` | Performs hybrid search combining BM25, vector search, and graph traversal results via RRF. |
| `pub async fn hybrid_search_reranked( &self, text: &str, vector: &[f32], k: usize, reranker: Option<&dyn Reranker>, anchor_entities: Option<&[contextra_types::EntityId]>, ) -> Result<Vec<crate::SearchResult>>` | `crates/contextra-db/src/collection/search/hybrid/mod.rs:45` | Dokumentation für `hybrid_search_reranked` im Rahmen der Collection- und Suchabstraktion. |
| `pub async fn hybrid_search_with_weights( &self, text: &str, vector: &[f32], k: usize, anchor_entities: Option<&[contextra_types::EntityId]>, weights: Option<&contextra_types::FusionWeights>, ) -> Result<Vec<crate::SearchResult>>` | `crates/contextra-db/src/collection/search/hybrid/mod.rs:68` | Performs hybrid search with custom fusion weights for vector, text, and graph signals, and optional community filtering/boosting. |
| `pub async fn hybrid_search_with_strategy( &self, text: &str, vector: &[f32], k: usize, anchor_entities: Option<&[contextra_types::EntityId]>, weights: Option<&contextra_types::FusionWeights>, strategy: Option<&contextra_types::GraphTraversalStrategy>, same_community_as: Option<EntityId>, fusion_strategy: Option<contextra_types::SignalFusionStrategies>, ) -> Result<Vec<crate::SearchResult>>` | `crates/contextra-db/src/collection/search/hybrid/mod.rs:94` | Performs hybrid search with custom signal fusion weights and graph traversal strategy. |
| `pub async fn traverse_links( &self, start: DocId, max_depth: usize, ) -> Result<Vec<(DocId, usize)>>` | `crates/contextra-db/src/collection/search/hybrid/mod.rs:346` | Traverses Zettelkasten memory links starting from a given document up to a maximum hop depth (ADR-038). Performs iterative BFS with cycle detection using `VecDe |
| `pub fn filter_by_importance( results: Vec<crate::SearchResult>, min_threshold: f32, now_tx: TxId, ) -> Vec<crate::SearchResult>` | `crates/contextra-db/src/collection/search/hybrid/mod.rs:385` | Filters a candidate list of search results by effective importance score threshold. Candidate results with `effective_score(now_tx) < min_threshold` are removed |
| `pub async fn hybrid_search_with_query( &self, query: &contextra_types::HybridQuery, ) -> Result<Vec<crate::SearchResult>>` | `crates/contextra-db/src/collection/search/hybrid/query.rs:45` | Performs hybrid search combining BM25, vector, and graph signals configured via `HybridQuery`. Applies `memory_type_filter` and metadata `FilterExpr` as Pre-RRF |
| `pub async fn hybrid_search_with_query_at( &self, query: &contextra_types::HybridQuery, seq: u64, ) -> Result<Vec<crate::SearchResult>>` | `crates/contextra-db/src/collection/search/hybrid/query.rs:59` | Performs hybrid search combining BM25, vector, and graph signals configured via `HybridQuery` at a specific snapshot sequence. |
| `pub async fn hybrid_search_with_query_and_report_at( &self, query: &contextra_types::HybridQuery, seq: u64, _on_signal_failure: SignalFailurePolicy, metrics: Option<&dyn MetricsSink>, ) -> Result<(Vec<crate::SearchResult>, SearchReport)>` | `crates/contextra-db/src/collection/search/hybrid/query.rs:72` | Performs hybrid search returning both search results and an execution `SearchReport`. |
| `pub fn next_tx(&self) -> Result<TxId>` | `crates/contextra-db/src/collection/tx.rs:30` |  |
| `pub fn allocate_tx(&self) -> Result<TxId>` | `crates/contextra-db/src/collection/tx.rs:40` | Allokiert eine eindeutige, atomar inkrementierte Transaction-ID. Externe Crates verwenden diese Methode statt eigener TxId-Generierung. Verhindert TxId-Kollisio |
| `pub fn begin_transaction(&self) -> Result<crate::transaction::DbTransaction<S, V>>` | `crates/contextra-db/src/collection/tx.rs:54` | Begins a new atomic transaction for this collection. |
| `pub async fn export_memories(&self) -> Result<ExportCollectionV1>` | `crates/contextra-db/src/export.rs:89` | Exportiert alle Memories und Graph-Relationen der Collection in das ExportCollectionV1 Schema. |
| `pub async fn import_memories(&self, col_export: ExportCollectionV1) -> Result<ImportSummary>` | `crates/contextra-db/src/import.rs:35` | Importiert alle Memories und Graph-Relationen einer ExportCollectionV1 idempotent (Upsert-Semantik). |

Assoziierte Konstanten/Typen:

- `pub const DECAY_DELETION_THRESHOLD: f32 = 0.05` — `crates/contextra-db/src/collection/mod.rs:362`
- `pub const DEFAULT_MIN_RERANK_CANDIDATES: usize = 100` — `crates/contextra-db/src/collection/search/hybrid/mod.rs:38`
- `pub const OVERFETCH_FACTOR: usize = 3` — `crates/contextra-db/src/collection/search/hybrid/mod.rs:284`
- `pub const DEFAULT_COMMUNITY_BOOST: f32 = 1.2` — `crates/contextra-db/src/collection/search/hybrid/mod.rs:287`

#### `struct CollectionConfig`
- Ort: `crates/contextra-db/src/collection/mod.rs:62`
```rust
pub struct CollectionConfig
```
Dokumentation: Configuration for a collection.
- Impl-Block (Trait `Default`) `crates/contextra-db/src/collection/mod.rs:68`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `backpressure_delay_ms` | `Option<u64>` | `crates/contextra-db/src/collection/mod.rs:65` |

#### `struct StoredDocument`
- Ort: `crates/contextra-db/src/collection/mod.rs:79`
```rust
pub struct StoredDocument
```
Dokumentation: Dokumentation für `StoredDocument` im Rahmen der Collection- und Suchabstraktion.

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `id` | `String` | `crates/contextra-db/src/collection/mod.rs:80` |
| `embedding` | `Vec<f32>` | `crates/contextra-db/src/collection/mod.rs:81` |
| `metadata` | `Option<serde_json::Value>` | `crates/contextra-db/src/collection/mod.rs:82` |

#### `struct StoredDocumentMeta`
- Ort: `crates/contextra-db/src/collection/mod.rs:88`
```rust
pub struct StoredDocumentMeta
```
Dokumentation: Leichtgewichtige Metadaten (für doc_key, key_type=1) — KEIN Embedding. Wird für DocId-basierte Hydration nach HNSW/BM25-Suche verwendet.
- Impl-Block (Trait `From<&StoredDocument>`) `crates/contextra-db/src/collection/mod.rs:93`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `id` | `String` | `crates/contextra-db/src/collection/mod.rs:89` |
| `metadata` | `Option<serde_json::Value>` | `crates/contextra-db/src/collection/mod.rs:90` |

#### `fn compute_default_importance`
- Ort: `crates/contextra-db/src/collection/mod.rs:124`
```rust
pub fn compute_default_importance(text_opt: Option<&str>) -> contextra_types::ImportanceScore
```
Dokumentation: Computes a non-LLM heuristic baseline importance score based on text length and character entropy.

#### `fn ensure_importance_metadata`
- Ort: `crates/contextra-db/src/collection/mod.rs:141`
```rust
pub fn ensure_importance_metadata( metadata: &mut Option<serde_json::Value>, tx: TxId, text_opt: Option<&str>, )
```
Dokumentation: Ensures document metadata contains a valid `MemoryImportance` JSON payload.

#### `fn extract_effective_importance`
- Ort: `crates/contextra-db/src/collection/mod.rs:211`
```rust
pub fn extract_effective_importance(metadata: &Option<serde_json::Value>, now_tx: TxId) -> f32
```
Dokumentation: Extracts the effective importance score of a document at a given transaction ID.

#### `fn parse_importance_score`
- Ort: `crates/contextra-db/src/collection/mod.rs:104`
```rust
pub fn parse_importance_score(response: &str) -> f32
```
Dokumentation: Parses an LLM response string into an f32 importance score in `[0.0, 1.0]`.
- **deprecated**

### Modul `collection::crud::auto_extraction`

#### `struct AutoExtractionConfig`
- Ort: `crates/contextra-db/src/collection/crud/auto_extraction.rs:96`
```rust
pub struct AutoExtractionConfig
```
Dokumentation: Konfiguration fuer die automatische Entitaetsextraktion beim Einfuegen von Dokumenten.
- Impl-Block `crates/contextra-db/src/collection/crud/auto_extraction.rs:106`
- Impl-Block (Trait `Default`) `crates/contextra-db/src/collection/crud/auto_extraction.rs:138`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `mode` | `AutoExtractionMode` | `crates/contextra-db/src/collection/crud/auto_extraction.rs:99` |
| `enabled` | `bool` | `crates/contextra-db/src/collection/crud/auto_extraction.rs:101` |
| `entity_config` | `EntityExtractionConfig` | `crates/contextra-db/src/collection/crud/auto_extraction.rs:103` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(mode: AutoExtractionMode) -> Self` | `crates/contextra-db/src/collection/crud/auto_extraction.rs:108` | Erstellt eine neue `AutoExtractionConfig` mit dem angegebenen Modus. |
| `pub fn with_mode(mut self, mode: AutoExtractionMode) -> Self` | `crates/contextra-db/src/collection/crud/auto_extraction.rs:117` | Setzt den Laufzeit-Modus (Builder Pattern). |
| `pub fn is_enabled(&self) -> bool` | `crates/contextra-db/src/collection/crud/auto_extraction.rs:130` | Prueft, ob die automatische Extraktion unter Beruecksichtigung aller Abschaltwege aktiv ist. Precedence / Abschaltkette: 1. Compile-Zeit feature `auto-extractio |

#### `struct EntityExtractionConfig`
- Ort: `crates/contextra-db/src/collection/crud/auto_extraction.rs:43`
```rust
pub struct EntityExtractionConfig
```
Dokumentation: Dokumentation für `EntityExtractionConfig` im Rahmen der Collection- und Suchabstraktion.
- Impl-Block (Trait `Default`) `crates/contextra-db/src/collection/crud/auto_extraction.rs:50`
- Impl-Block (Trait `Default`) `crates/contextra-db/src/extraction/types.rs:45`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `enabled` | `bool` | `crates/contextra-db/src/collection/crud/auto_extraction.rs:44` |
| `max_llm_calls_per_cycle` | `usize` | `crates/contextra-db/src/collection/crud/auto_extraction.rs:45` |
| `min_confidence` | `f32` | `crates/contextra-db/src/collection/crud/auto_extraction.rs:46` |

#### `enum ExtractorMode`
- Ort: `crates/contextra-db/src/collection/crud/auto_extraction.rs:27`
```rust
pub enum ExtractorMode
```
Dokumentation: Dokumentation für `ExtractorMode` im Rahmen der Collection- und Suchabstraktion.
- Impl-Block (Trait `Default`) `crates/contextra-db/src/collection/crud/auto_extraction.rs:33`

Varianten:

- `OpenIe` — `crates/contextra-db/src/collection/crud/auto_extraction.rs:28`
- `RuleBased` — `crates/contextra-db/src/collection/crud/auto_extraction.rs:29`

#### `fn recommended_mode_for_regulated`
- Ort: `crates/contextra-db/src/collection/crud/auto_extraction.rs:86`
```rust
pub fn recommended_mode_for_regulated(is_regulated: bool) -> AutoExtractionMode
```
Dokumentation: Liefert den empfohlenen `AutoExtractionMode` fuer das gegebene "regulated"-Flag. Gemäss v17 Teil 10.5 soll in EnterpriseRegulated / Regulierungs-Kontexten die automatische Extraktion standardmäßig deaktiviert werden (`Disabled`), um Datenminimierung sicherzustellen. **Hinweis:** Diese Funktion ändert NICHT den globalen Standardwert von `AutoExtractionMode::default()`. Die Entscheidung über eine globale Anpassung obliegt der Produktverantwortung und bleibt als offener Punkt im PR-Text dokumentiert.

### Modul `collection::crud::read`

#### `const DEFAULT_SCAN_LIMIT`
- Ort: `crates/contextra-db/src/collection/crud/read.rs:15`
```rust
pub const DEFAULT_SCAN_LIMIT: usize = 10_000
```
Dokumentation: Harte Obergrenze für scan()/scan_prefix()-Ergebnisse, falls kein explizites `limit` übergeben wird. Verhindert unbeabsichtigten Vollscan bei generischen Präfixen.

#### `const HARD_SCAN_CEILING`
- Ort: `crates/contextra-db/src/collection/crud/read.rs:19`
```rust
pub const HARD_SCAN_CEILING: usize = 100_000
```
Dokumentation: Harte Obergrenze für den maximal erlaubten `limit`-Parameter in scan() und scan_prefix(). Verhindert unbegrenzte Materialisierung im Speicher selbst wenn explizit ein riesiges Limit angefragt wird.

#### `const MAX_SCAN_RESULTS`
- Ort: `crates/contextra-db/src/collection/crud/read.rs:22`
```rust
pub const MAX_SCAN_RESULTS: usize = DEFAULT_SCAN_LIMIT
```
Dokumentation: Alias for backwards compatibility with earlier MAX_SCAN_RESULTS references.

### Modul `collection::kv_lock`

#### `fn sorted_unique_shards`
- Ort: `crates/contextra-db/src/collection/kv_lock.rs:101`
```rust
pub fn sorted_unique_shards<'a>( keys: impl IntoIterator<Item = &'a str>, shard_count: usize, hasher: &ahash::RandomState, ) -> Vec<usize>
```
Dokumentation: Pure, synchronous function to map string keys to unique shard indices in ascending order. Deduplicates shard indices to prevent self-deadlock when multiple keys map to the same shard and sorts indices in ascending order to prevent cross-operation deadlocks.

### Modul `collection::maintenance`

#### `struct PercolationResult`
- Ort: `crates/contextra-db/src/collection/maintenance.rs:883`
```rust
pub struct PercolationResult
```
- Gate: `#[cfg(feature = "graph-connectivity-health")]`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `health` | `Option<f32>` | `crates/contextra-db/src/collection/maintenance.rs:884` |
| `rebonding_triggered` | `bool` | `crates/contextra-db/src/collection/maintenance.rs:885` |
| `new_edges_added` | `usize` | `crates/contextra-db/src/collection/maintenance.rs:886` |

### Modul `collection::query_builder`

#### `const METRIC_SEARCH_LATENCY_GRAPH`
- Ort: `crates/contextra-db/src/collection/query_builder/mod.rs:30`
```rust
pub const METRIC_SEARCH_LATENCY_GRAPH: &str = ""
```
Dokumentation: Metric counter name for graph search signal latency in seconds.

#### `const METRIC_SEARCH_LATENCY_TEXT`
- Ort: `crates/contextra-db/src/collection/query_builder/mod.rs:27`
```rust
pub const METRIC_SEARCH_LATENCY_TEXT: &str = ""
```
Dokumentation: Metric counter name for full-text search signal latency in seconds.

#### `const METRIC_SEARCH_LATENCY_VECTOR`
- Ort: `crates/contextra-db/src/collection/query_builder/mod.rs:24`
```rust
pub const METRIC_SEARCH_LATENCY_VECTOR: &str = ""
```
Dokumentation: Metric counter name for vector search signal latency in seconds.

### Modul `collection::query_builder::builder`

#### `struct HybridQueryBuilder`
- Ort: `crates/contextra-db/src/collection/query_builder/builder.rs:31`
```rust
pub struct HybridQueryBuilder<'a, S: StorageEngine, V: VectorIndex>
```
Dokumentation: Fluent query builder for unifying vector, text, graph, and hybrid search operations.
- Impl-Block `crates/contextra-db/src/collection/query_builder/builder.rs:60`
- Impl-Block `crates/contextra-db/src/collection/query_builder/builder_exec.rs:12`
- Impl-Block `crates/contextra-db/src/collection/query_builder/builder_exec.rs:328`
- Impl-Block `crates/contextra-db/src/collection/query_builder/scope.rs:56`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `text` | `Option<String>` | `crates/contextra-db/src/collection/query_builder/builder.rs:33` |
| `vector` | `Option<Vec<f32>>` | `crates/contextra-db/src/collection/query_builder/builder.rs:34` |
| `k` | `Option<usize>` | `crates/contextra-db/src/collection/query_builder/builder.rs:35` |
| `weights` | `Option<FusionWeights>` | `crates/contextra-db/src/collection/query_builder/builder.rs:36` |
| `fusion_strategy` | `Option<SignalFusionStrategies>` | `crates/contextra-db/src/collection/query_builder/builder.rs:37` |
| `strategy` | `Option<SearchStrategy>` | `crates/contextra-db/src/collection/query_builder/builder.rs:38` |
| `filter` | `Option<FilterExpr>` | `crates/contextra-db/src/collection/query_builder/builder.rs:39` |
| `anchor_entities` | `Option<Vec<EntityId>>` | `crates/contextra-db/src/collection/query_builder/builder.rs:40` |
| `same_community_as` | `Option<EntityId>` | `crates/contextra-db/src/collection/query_builder/builder.rs:41` |
| `memory_type_filter` | `Option<Vec<MemoryType>>` | `crates/contextra-db/src/collection/query_builder/builder.rs:42` |
| `include_superseded` | `bool` | `crates/contextra-db/src/collection/query_builder/builder.rs:43` |
| `include_provenance` | `bool` | `crates/contextra-db/src/collection/query_builder/builder.rs:44` |
| `filter_fn` | `Option<Box<dyn Fn(DocId) -> bool + Send + Sync>>` | `crates/contextra-db/src/collection/query_builder/builder.rs:45` |
| `hard_scope` | `Option<super::scope::ScopeConstraint>` | `crates/contextra-db/src/collection/query_builder/builder.rs:46` |
| `reranker` | `Option<&'a (dyn Reranker + 'a)>` | `crates/contextra-db/src/collection/query_builder/builder.rs:47` |
| `rerank_pool_multiplier` | `Option<usize>` | `crates/contextra-db/src/collection/query_builder/builder.rs:48` |
| `rerank_pool_max` | `Option<usize>` | `crates/contextra-db/src/collection/query_builder/builder.rs:49` |
| `on_signal_failure` | `SignalFailurePolicy` | `crates/contextra-db/src/collection/query_builder/builder.rs:50` |
| `pid_controller` | `Option<Arc<parking_lot::Mutex<contextra_adapt::PidController>>>` | `crates/contextra-db/src/collection/query_builder/builder.rs:52` |
| `seq` | `Option<u64>` | `crates/contextra-db/src/collection/query_builder/builder.rs:53` |
| `as_of_timestamp` | `Option<u64>` | `crates/contextra-db/src/collection/query_builder/builder.rs:54` |
| `query_timestamp` | `Option<u64>` | `crates/contextra-db/src/collection/query_builder/builder.rs:55` |
| `current_tx` | `Option<contextra_types::TxId>` | `crates/contextra-db/src/collection/query_builder/builder.rs:56` |
| `metrics` | `Option<Arc<dyn MetricsSink>>` | `crates/contextra-db/src/collection/query_builder/builder.rs:57` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(collection: &'a Collection<S, V>) -> Self` | `crates/contextra-db/src/collection/query_builder/builder.rs:62` | Creates a new `HybridQueryBuilder` for a given `Collection`. |
| `pub fn text(mut self, text: impl Into<String>) -> Self` | `crates/contextra-db/src/collection/query_builder/builder.rs:94` | Sets full-text search query string. |
| `pub fn embedding(mut self, embedding: impl AsRef<[f32]>) -> Self` | `crates/contextra-db/src/collection/query_builder/builder.rs:100` | Sets dense vector query embedding. |
| `pub fn vector(self, vector: impl AsRef<[f32]>) -> Self` | `crates/contextra-db/src/collection/query_builder/builder.rs:106` | Alias for `.embedding()` to specify dense vector query embedding. |
| `pub fn k(mut self, k: usize) -> Self` | `crates/contextra-db/src/collection/query_builder/builder.rs:111` | Sets top-k maximum result count. |
| `pub fn weights<W>(mut self, weights: W) -> Self where W: TryInto<FusionWeights, Error = contextra_types::ContextraError>,` | `crates/contextra-db/src/collection/query_builder/builder.rs:117` | Sets custom signal fusion weights using `SignalWeights` or `FusionWeights`. |
| `pub fn fusion_weights(mut self, weights: FusionWeights) -> Self` | `crates/contextra-db/src/collection/query_builder/builder.rs:128` | Sets custom signal fusion weights directly using `FusionWeights`. |
| `pub fn fusion_strategy(mut self, strategy: impl Into<SignalFusionStrategies>) -> Self` | `crates/contextra-db/src/collection/query_builder/builder.rs:134` | Sets search fusion strategy per signal or globally (`FusionStrategy` or `SignalFusionStrategies`). |
| `pub fn filter(mut self, filter: FilterExpr) -> Self` | `crates/contextra-db/src/collection/query_builder/builder.rs:140` | Sets metadata expression filter (`FilterExpr`). |
| `pub fn metadata_filter(mut self, filter: MetadataFilter) -> Self` | `crates/contextra-db/src/collection/query_builder/builder.rs:147` | Sets legacy metadata filter (`MetadataFilter`). |
| `pub fn strategy(mut self, strategy: impl Into<SearchStrategy>) -> Self` | `crates/contextra-db/src/collection/query_builder/builder.rs:155` | Sets hybrid search / graph traversal strategy. |
| `pub fn anchor_entities(mut self, anchors: impl IntoIterator<Item = EntityId>) -> Self` | `crates/contextra-db/src/collection/query_builder/builder.rs:161` | Sets anchor seed entities for graph traversal signal. |
| `pub fn anchors(self, anchors: impl IntoIterator<Item = EntityId>) -> Self` | `crates/contextra-db/src/collection/query_builder/builder.rs:167` | Alias for `.anchor_entities()`. |
| `pub fn same_community_as(mut self, entity_id: EntityId) -> Self` | `crates/contextra-db/src/collection/query_builder/builder.rs:172` | Sets target entity for community context boosting/filtering. |
| `pub fn memory_type_filter(mut self, types: impl IntoIterator<Item = MemoryType>) -> Self` | `crates/contextra-db/src/collection/query_builder/builder.rs:178` | Sets cognitive memory type filter (Pre-RRF filter). |
| `pub fn include_provenance(mut self, include: bool) -> Self` | `crates/contextra-db/src/collection/query_builder/builder.rs:184` | Sets whether to calculate and attach ProvenanceRecord to output results. |
| `pub fn include_superseded(mut self, include: bool) -> Self` | `crates/contextra-db/src/collection/query_builder/builder.rs:190` | Sets whether to include superseded documents (Post-RRF Supersedes Displacement, ADR-038). |
| `pub fn memory_types(self, types: impl IntoIterator<Item = MemoryType>) -> Self` | `crates/contextra-db/src/collection/query_builder/builder.rs:196` | Alias for `.memory_type_filter()`. |
| `pub fn filter_fn(mut self, f: impl Fn(DocId) -> bool + Send + Sync + 'static) -> Self` | `crates/contextra-db/src/collection/query_builder/builder.rs:201` | Sets custom filter predicate function for vector search candidates. |
| `pub fn reranker(mut self, reranker: &'a (dyn Reranker + 'a)) -> Self` | `crates/contextra-db/src/collection/query_builder/builder.rs:207` | Sets optional CrossEncoder reranker for post-retrieval ranking. |
| `pub fn rerank_pool_multiplier(mut self, multiplier: usize) -> Self` | `crates/contextra-db/src/collection/query_builder/builder.rs:216` | Sets the candidate pool multiplier for pre-reranking candidate expansion. Default: 10 (`DEFAULT_RERANK_POOL_MULTIPLIER`), yielding ~100 candidates for `k=10` pe |
| `pub fn rerank_pool_max(mut self, max: usize) -> Self` | `crates/contextra-db/src/collection/query_builder/builder.rs:224` | Sets the upper cap for pre-reranking candidate pool expansion. Default: 200 (`DEFAULT_RERANK_POOL_MAX`), capping pre-retrieval pool size for large `k`. |
| `pub fn pid_controller( mut self, pid: Arc<parking_lot::Mutex<contextra_adapt::PidController>>, ) -> Self` | `crates/contextra-db/src/collection/query_builder/builder.rs:231` | Sets optional PID controller for adaptive reranking candidate pool sizing. |
| `pub fn seq(mut self, seq_no: u64) -> Self` | `crates/contextra-db/src/collection/query_builder/builder.rs:240` | Sets snapshot sequence number for MVCC snapshot-isolated queries. |
| `pub fn as_of(mut self, timestamp: u64) -> Self` | `crates/contextra-db/src/collection/query_builder/builder.rs:246` | Sets historical point-in-time "as-of" timestamp for temporal validity filtering (Post-RRF, Pre-Reranking). |
| `pub fn query_timestamp(mut self, timestamp: u64) -> Self` | `crates/contextra-db/src/collection/query_builder/builder.rs:252` | Sets query business timestamp for temporal validity filtering (Post-RRF, Pre-Reranking). |
| `pub fn current_tx(mut self, tx: contextra_types::TxId) -> Self` | `crates/contextra-db/src/collection/query_builder/builder.rs:258` | Sets current system transaction ID for temporal validity filtering (Post-RRF, Pre-Reranking). |
| `pub fn on_signal_failure(mut self, policy: SignalFailurePolicy) -> Self` | `crates/contextra-db/src/collection/query_builder/builder.rs:267` | Sets signal failure policy (`Fail` or `Degrade`). Spec v17 §6.5: `Fail` (default) fails the entire query if any signal fails. `Degrade` skips failing signals, r |
| `pub fn with_metrics_sink(mut self, sink: Arc<dyn MetricsSink>) -> Self` | `crates/contextra-db/src/collection/query_builder/builder.rs:273` | Configures an optional metrics sink for search signal latency instrumentation. |
| `pub fn query_config(mut self, query: &contextra_types::HybridQuery) -> Self` | `crates/contextra-db/src/collection/query_builder/builder.rs:279` | Configures builder options from an existing `HybridQuery` struct. |
| `pub async fn execute(self) -> Result<Vec<crate::SearchResult>>` | `crates/contextra-db/src/collection/query_builder/builder_exec.rs:17` | Executes search query, delegating core signal retrieval to `execute_with_report()`. Under `SignalFailurePolicy::Fail` (default), returns an error if any signal  |
| `pub async fn execute_with_report( self, ) -> Result<(Vec<crate::SearchResult>, crate::fusion::SearchReport)>` | `crates/contextra-db/src/collection/query_builder/builder_exec.rs:39` | Executes search query and returns both search results and an execution `SearchReport`. The `SearchReport` captures degraded signals (when using `SignalFailurePo |
| `pub fn scope(mut self, constraint: ScopeConstraint) -> Self` | `crates/contextra-db/src/collection/query_builder/scope.rs:61` | Sets an explicit ACORN hard-boundary scope constraint for vector retrieval. Spec §5.3: `.scope(ScopeConstraint)` enforces strict document boundary scoping via A |

#### `const DEFAULT_RERANK_POOL_MAX`
- Ort: `crates/contextra-db/src/collection/query_builder/builder.rs:28`
```rust
pub const DEFAULT_RERANK_POOL_MAX: usize = 200
```
Dokumentation: Default upper cap for pre-reranking candidate pool expansion (200 candidates). Prevents retrieval cost explosion for queries with large `k`.

#### `const DEFAULT_RERANK_POOL_MULTIPLIER`
- Ort: `crates/contextra-db/src/collection/query_builder/builder.rs:23`
```rust
pub const DEFAULT_RERANK_POOL_MULTIPLIER: usize = 10
```
Dokumentation: Default candidate pool expansion multiplier for Cross-Encoder reranking (10x requested k). Grounded in empirical evaluation from T2-RAGBench (arXiv:2604.01733), showing Recall@5 = 0.888 with ~100 candidate items compared to 0.458 with only 20 candidates.

### Modul `collection::query_builder::scope`

#### `struct ScopeConstraint`
- Ort: `crates/contextra-db/src/collection/query_builder/scope.rs:20`
```rust
pub struct ScopeConstraint
```
Dokumentation: Scoping constraint defining an explicit hard-boundary set of allowed document IDs for predicate-agnostic vector search (ACORN-Hard-Boundary, Spec §5.3, §8.5).
- Impl-Block `crates/contextra-db/src/collection/query_builder/scope.rs:28`
- Impl-Block (Trait `Default`) `crates/contextra-db/src/collection/query_builder/scope.rs:46`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `allowed_doc_ids` | `BTreeSet<DocId>` | `crates/contextra-db/src/collection/query_builder/scope.rs:22` |
| `gamma` | `u32` | `crates/contextra-db/src/collection/query_builder/scope.rs:25` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(allowed_doc_ids: BTreeSet<DocId>, gamma: u32) -> Self` | `crates/contextra-db/src/collection/query_builder/scope.rs:30` | Creates a new `ScopeConstraint` with a custom `gamma` factor. |
| `pub fn from_allowed_ids(allowed_doc_ids: BTreeSet<DocId>) -> Self` | `crates/contextra-db/src/collection/query_builder/scope.rs:38` | Creates a new `ScopeConstraint` with default `gamma = 2` (Spec §8.5). |

### Modul `collection::query_builder::strategy`

#### `enum SearchStrategy`
- Ort: `crates/contextra-db/src/collection/query_builder/strategy.rs:12`
```rust
pub enum SearchStrategy
```
Dokumentation: Strategy for hybrid search and graph signal traversal.
- Impl-Block `crates/contextra-db/src/collection/query_builder/strategy.rs:33`
- Impl-Block (Trait `From<GraphTraversalStrategy>`) `crates/contextra-db/src/collection/query_builder/strategy.rs:71`

Varianten:

- `Rrf` — `crates/contextra-db/src/collection/query_builder/strategy.rs:14`
- `ScoreNormalized` — `crates/contextra-db/src/collection/query_builder/strategy.rs:16`
- `Hops{` — `crates/contextra-db/src/collection/query_builder/strategy.rs:18`
- `PersonalizedPageRank(contextra_graph::PprConfig)` — `crates/contextra-db/src/collection/query_builder/strategy.rs:23`
- `PathRag{` — `crates/contextra-db/src/collection/query_builder/strategy.rs:25`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn to_graph_strategy(&self) -> GraphTraversalStrategy` | `crates/contextra-db/src/collection/query_builder/strategy.rs:35` | Converts `SearchStrategy` into core `GraphTraversalStrategy`. |
| `pub fn to_fusion_strategy(&self) -> FusionStrategy` | `crates/contextra-db/src/collection/query_builder/strategy.rs:57` | Converts `SearchStrategy` into core `FusionStrategy`. |
| `pub fn to_signal_fusion_strategies(&self) -> SignalFusionStrategies` | `crates/contextra-db/src/collection/query_builder/strategy.rs:66` | Converts `SearchStrategy` into core `SignalFusionStrategies`. |

### Modul `collection::query_builder::weights`

#### `struct SignalWeights`
- Ort: `crates/contextra-db/src/collection/query_builder/weights.rs:12`
```rust
pub struct SignalWeights
```
Dokumentation: Custom weights for vector, text, and graph signals in hybrid search.
- Impl-Block `crates/contextra-db/src/collection/query_builder/weights.rs:21`
- Impl-Block (Trait `From<FusionWeights>`) `crates/contextra-db/src/collection/query_builder/weights.rs:39`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `vector` | `f32` | `crates/contextra-db/src/collection/query_builder/weights.rs:14` |
| `text` | `f32` | `crates/contextra-db/src/collection/query_builder/weights.rs:16` |
| `graph` | `f32` | `crates/contextra-db/src/collection/query_builder/weights.rs:18` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(vector: f32, text: f32, graph: f32) -> Result<Self>` | `crates/contextra-db/src/collection/query_builder/weights.rs:23` | Creates a new `SignalWeights` instance, ensuring weights sum to 1.0. |

### Modul `collection::search::checkpoint`

#### `struct CheckpointPinGuard`
- Ort: `crates/contextra-db/src/collection/search/checkpoint.rs:22`
```rust
pub struct CheckpointPinGuard<'a, S: StorageEngine + ?Sized>
```
Dokumentation: RAII Guard for pinning snapshot checkpoints during search/scan operations. # Note on async Drop Rust does not support native `async Drop`. Therefore, callers MUST explicitly call [`release`](Self::release) to unpin the checkpoint. The `Drop` implementation serves as a fallback safety net: if `release` was not invoked before dropping (e.g. due to early `?` return or panic), `Drop` emits an `error!` log warning that a checkpoint pin may have leaked.
- Impl-Block `crates/contextra-db/src/collection/search/checkpoint.rs:28`
- Impl-Block (Trait `Drop`) `crates/contextra-db/src/collection/search/checkpoint.rs:62`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub async fn new(storage: &'a S, seq: u64) -> Result<Self>` | `crates/contextra-db/src/collection/search/checkpoint.rs:30` | Creates a new guard and pins the checkpoint at sequence `seq`. |
| `pub async fn new_at_latest(storage: &'a S) -> Result<(Self, u64)>` | `crates/contextra-db/src/collection/search/checkpoint.rs:41` | Pin-first, read-after: the seq is read under the protection of the pin, eliminating the TOCTOU window between snapshot_seq() and pin activation. |
| `pub async fn release(mut self) -> Result<()>` | `crates/contextra-db/src/collection/search/checkpoint.rs:55` | Explicitly unpins the checkpoint and consumes the guard, preventing the fallback `Drop` warning. |

#### `fn with_pinned_checkpoint`
- Ort: `crates/contextra-db/src/collection/search/checkpoint.rs:76`
```rust
pub async fn with_pinned_checkpoint<S, F, Fut, T>(storage: &S, seq: u64, f: F) -> Result<T> where S: StorageEngine + ?Sized, F: FnOnce() -> Fut, Fut: std::future::Future<Output = Result<T>>,
```
Dokumentation: Higher-order function wrapping an async block with a pinned checkpoint guard, ensuring `release()` is ALWAYS called even on error paths.

#### `fn with_pinned_checkpoint_at_latest`
- Ort: `crates/contextra-db/src/collection/search/checkpoint.rs:94`
```rust
pub async fn with_pinned_checkpoint_at_latest<S, F, Fut, T>(storage: &S, f: F) -> Result<T> where S: StorageEngine + ?Sized, F: FnOnce(u64) -> Fut, Fut: std::future::Future<Output = Result<T>>,
```
Dokumentation: Higher-order function providing pin-first, read-after semantics with automatic release on completion/error. Pin-first, read-after: the seq is read under the protection of the pin, eliminating the TOCTOU window between snapshot_seq() and pin activation.

### Modul `collection::search::filtered`

#### `const MAX_OVERFETCH_SCAN`
- Ort: `crates/contextra-db/src/collection/search/filtered.rs:20`
```rust
pub const MAX_OVERFETCH_SCAN: usize = contextra_types::MAX_SEARCH_K
```
Dokumentation: Upper bound scan limit for adaptive candidate overfetching.

#### `const OVERFETCH_STAGE_MULTIPLIERS`
- Ort: `crates/contextra-db/src/collection/search/filtered.rs:17`
```rust
pub const OVERFETCH_STAGE_MULTIPLIERS: [usize
```
Dokumentation: Named overfetch stage multipliers for adaptive candidate overfetching (Spec v17 §6.5).

### Modul `crate`

#### `mod adapters`
- Ort: `crates/contextra-db/src/lib.rs:12`
```rust
pub mod adapters
```

#### `mod advanced`
- Ort: `crates/contextra-db/src/lib.rs:473`
```rust
pub mod advanced
```

#### `mod backends`
- Ort: `crates/contextra-db/src/lib.rs:13`
```rust
pub mod backends
```

#### `mod background_workers`
- Ort: `crates/contextra-db/src/lib.rs:15`
```rust
pub mod background_workers
```

#### `mod chunker`
- Ort: `crates/contextra-db/src/lib.rs:17`
```rust
pub mod chunker
```

#### `mod cognition`
- Ort: `crates/contextra-db/src/lib.rs:49`
```rust
pub mod cognition
```
- Gate: `#[cfg(feature = "cognition")]`

#### `mod collection`
- Ort: `crates/contextra-db/src/lib.rs:18`
```rust
pub mod collection
```

#### `mod contextra_impl`
- Ort: `crates/contextra-db/src/lib.rs:20`
```rust
pub mod contextra_impl
```

#### `mod decay_controller`
- Ort: `crates/contextra-db/src/lib.rs:22`
```rust
pub mod decay_controller
```

#### `mod export`
- Ort: `crates/contextra-db/src/lib.rs:24`
```rust
pub mod export
```

#### `mod extraction`
- Ort: `crates/contextra-db/src/lib.rs:27`
```rust
pub mod extraction
```

#### `mod filter`
- Ort: `crates/contextra-db/src/lib.rs:29`
```rust
pub mod filter
```

#### `mod fusion`
- Ort: `crates/contextra-db/src/lib.rs:31`
```rust
pub mod fusion
```

#### `mod homeostat`
- Ort: `crates/contextra-db/src/lib.rs:32`
```rust
pub mod homeostat
```

#### `mod import`
- Ort: `crates/contextra-db/src/lib.rs:34`
```rust
pub mod import
```

#### `mod merge_operator_adapter`
- Ort: `crates/contextra-db/src/lib.rs:37`
```rust
pub mod merge_operator_adapter
```
- Gate: `#[cfg(feature = "sandbox")]`

#### `mod metrics_names`
- Ort: `crates/contextra-db/src/lib.rs:39`
```rust
pub mod metrics_names
```

#### `mod multistep`
- Ort: `crates/contextra-db/src/lib.rs:40`
```rust
pub mod multistep
```

#### `mod orchestration`
- Ort: `crates/contextra-db/src/lib.rs:41`
```rust
pub mod orchestration
```

#### `mod pid_latency_controller`
- Ort: `crates/contextra-db/src/lib.rs:42`
```rust
pub mod pid_latency_controller
```

#### `mod prelude`
- Ort: `crates/contextra-db/src/lib.rs:458`
```rust
pub mod prelude
```

#### `mod temporal_filter`
- Ort: `crates/contextra-db/src/lib.rs:44`
```rust
pub mod temporal_filter
```

#### `mod transaction`
- Ort: `crates/contextra-db/src/lib.rs:46`
```rust
pub mod transaction
```

#### `mod volatile_vault`
- Ort: `crates/contextra-db/src/lib.rs:52`
```rust
pub mod volatile_vault
```
- Gate: `#[cfg(feature = "volatile-vault")]`

#### `struct CommunityDetectionConfig`
- Ort: `crates/contextra-db/src/lib.rs:177`
```rust
pub struct CommunityDetectionConfig
```
Dokumentation: Konfiguration für Community-Erkennungs- und Graph-Clustering-Algorithmen.
- Impl-Block (Trait `Default`) `crates/contextra-db/src/lib.rs:181`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `auto_trigger_threshold` | `u64` | `crates/contextra-db/src/lib.rs:178` |

#### `struct Contextra`
- Ort: `crates/contextra-db/src/lib.rs:337`
```rust
pub struct Contextra
```
- Impl-Block `crates/contextra-db/src/contextra_impl/collections.rs:15`
- Impl-Block `crates/contextra-db/src/contextra_impl/crud.rs:15`
- Impl-Block `crates/contextra-db/src/contextra_impl/diagnostics.rs:13`
- Impl-Block `crates/contextra-db/src/contextra_impl/lifecycle.rs:20`
- Impl-Block (Trait `Drop`) `crates/contextra-db/src/contextra_impl/lifecycle.rs:476`
- Impl-Block `crates/contextra-db/src/contextra_impl/lifecycle.rs:482`
- Impl-Block `crates/contextra-db/src/contextra_impl/wiring.rs:11`
- Impl-Block `crates/contextra-db/src/export.rs:377`
- Impl-Block `crates/contextra-db/src/import.rs:113`
- Impl-Block `crates/contextra-db/src/lib.rs:362`
- Impl-Block (Trait `SandboxBridge`) `crates/contextra-db/src/lib.rs:396`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub async fn collection_for_tenant( &self, name: &str, tenant_id: TenantId, ) -> Result<Arc<Collection<contextra_store::tenant_codec::TenantScopedStorage<Arc<LsmStorage>>>>>` | `crates/contextra-db/src/contextra_impl/collections.rs:18` | Liefert oder initialisiert eine Collection für einen Mandanten. |
| `pub async fn collection_with_language_and_tenant( &self, name: &str, language: Language, tenant_id: TenantId, ) -> Result<Arc<Collection<contextra_store::tenant_codec::TenantScopedStorage<Arc<LsmStorage>>>>>` | `crates/contextra-db/src/contextra_impl/collections.rs:31` | Öffnet oder erstellt eine Collection mit Sprachkonfiguration für einen Mandanten. |
| `pub async fn collection(&self, name: &str) -> Result<Arc<Collection<LsmStorage>>>` | `crates/contextra-db/src/contextra_impl/collections.rs:130` | Retrieves or creates an un-tenanted [`Collection`] handle (Admin / Single-Tenant path). # Errors Returns [`ContextraError::InvalidInput`] if [`TenantPolicy::Req |
| `pub async fn collection_with_language( &self, name: &str, language: Language, ) -> Result<Arc<Collection<LsmStorage>>>` | `crates/contextra-db/src/contextra_impl/collections.rs:140` | Retrieves or creates an un-tenanted [`Collection`] handle with specified language (Admin / Single-Tenant path). # Errors Returns [`ContextraError::InvalidInput` |
| `pub fn allocate_tx(&self) -> Result<TxId>` | `crates/contextra-db/src/contextra_impl/collections.rs:232` | Dokumentation für `allocate_tx` im Rahmen der Transaktions- und Facade-Orchestrierung. |
| `pub async fn list_collections(&self) -> Result<Vec<String>>` | `crates/contextra-db/src/contextra_impl/collections.rs:245` | Dokumentation für `list_collections` im Rahmen der Transaktions- und Facade-Orchestrierung. |
| `pub async fn list_collections_for_tenant(&self, tenant_id: TenantId) -> Result<Vec<String>>` | `crates/contextra-db/src/contextra_impl/collections.rs:272` | Dokumentation für `list_collections_for_tenant` im Rahmen der Transaktions- und Facade-Orchestrierung. |
| `pub async fn drop_collection(&self, name: &str, tenant_id: TenantId) -> Result<()>` | `crates/contextra-db/src/contextra_impl/collections.rs:299` | Dokumentation für `drop_collection` im Rahmen der Transaktions- und Facade-Orchestrierung. Führt die rein mechanische Entfernung einer Collection und ihrer Date |
| `pub async fn put_kv(&self, id: &str, value: &Value) -> Result<()>` | `crates/contextra-db/src/contextra_impl/crud.rs:25` | Dokumentation für `put_kv` im Rahmen der Transaktions- und Facade-Orchestrierung. |
| `pub async fn get_kv(&self, id: &str) -> Result<Option<Value>>` | `crates/contextra-db/src/contextra_impl/crud.rs:32` | Dokumentation für `get_kv` im Rahmen der Transaktions- und Facade-Orchestrierung. |
| `pub async fn insert(&self, id: &str, embedding: &[f32], metadata: Option<Value>) -> Result<()>` | `crates/contextra-db/src/contextra_impl/crud.rs:39` | Dokumentation für `insert` im Rahmen der Transaktions- und Facade-Orchestrierung. |
| `pub async fn insert_typed( &self, collection_name: &str, id: &str, embedding: &[f32], memory_type: contextra_types::MemoryType, metadata: Option<Value>, ) -> Result<()>` | `crates/contextra-db/src/contextra_impl/crud.rs:49` | Dokumentation für `insert_typed` im Rahmen der Transaktions- und Facade-Orchestrierung. |
| `pub async fn upsert(&self, id: &str, embedding: &[f32], metadata: Option<Value>) -> Result<()>` | `crates/contextra-db/src/contextra_impl/crud.rs:66` | Dokumentation für `upsert` im Rahmen der Transaktions- und Facade-Orchestrierung. |
| `pub async fn insert_many(&self, docs: &[(String, Vec<f32>, Option<Value>)]) -> Result<()>` | `crates/contextra-db/src/contextra_impl/crud.rs:76` | Dokumentation für `insert_many` im Rahmen der Transaktions- und Facade-Orchestrierung. |
| `pub async fn upsert_many(&self, docs: &[(String, Vec<f32>, Option<Value>)]) -> Result<()>` | `crates/contextra-db/src/contextra_impl/crud.rs:83` | Dokumentation für `upsert_many` im Rahmen der Transaktions- und Facade-Orchestrierung. |
| `pub async fn get(&self, id: &str) -> Result<Option<Document>>` | `crates/contextra-db/src/contextra_impl/crud.rs:90` | Dokumentation für `get` im Rahmen der Transaktions- und Facade-Orchestrierung. |
| `pub async fn get_at_snapshot(&self, id: &str, seq_no: u64) -> Result<Option<Document>>` | `crates/contextra-db/src/contextra_impl/crud.rs:97` | Dokumentation für `get_at_snapshot` im Rahmen der Transaktions- und Facade-Orchestrierung. |
| `pub async fn last_committed_seq(&self) -> Result<u64>` | `crates/contextra-db/src/contextra_impl/crud.rs:104` | Dokumentation für `last_committed_seq` im Rahmen der Transaktions- und Facade-Orchestrierung. |
| `pub async fn create_snapshot(&self) -> Result<u64>` | `crates/contextra-db/src/contextra_impl/crud.rs:111` | Dokumentation für `create_snapshot` im Rahmen der Transaktions- und Facade-Orchestrierung. |
| `pub async fn update(&self, id: &str, embedding: &[f32], metadata: Option<Value>) -> Result<()>` | `crates/contextra-db/src/contextra_impl/crud.rs:118` | Dokumentation für `update` im Rahmen der Transaktions- und Facade-Orchestrierung. |
| `pub async fn search(&self, query: &[f32], k: usize) -> Result<Vec<SearchResult>>` | `crates/contextra-db/src/contextra_impl/crud.rs:129` | Dokumentation für `search` im Rahmen der Transaktions- und Facade-Orchestrierung. |
| `pub async fn search_with_filter( &self, query: &[f32], k: usize, filter: Option<MetadataFilter>, ) -> Result<Vec<SearchResult>>` | `crates/contextra-db/src/contextra_impl/crud.rs:147` | Dokumentation für `search_with_filter` im Rahmen der Transaktions- und Facade-Orchestrierung. |
| `pub async fn search_with_filter_expr( &self, query: &[f32], k: usize, filter: Option<FilterExpr>, ) -> Result<Vec<SearchResult>>` | `crates/contextra-db/src/contextra_impl/crud.rs:165` | Dokumentation für `search_with_filter_expr` im Rahmen der Transaktions- und Facade-Orchestrierung. |
| `pub async fn insert_text_only( &self, id: &str, text: &str, metadata: Option<Value>, ) -> Result<()>` | `crates/contextra-db/src/contextra_impl/crud.rs:182` | Dokumentation für `insert_text_only` im Rahmen der Transaktions- und Facade-Orchestrierung. |
| `pub async fn upsert_text_only( &self, id: &str, text: &str, metadata: Option<Value>, ) -> Result<()>` | `crates/contextra-db/src/contextra_impl/crud.rs:197` | Dokumentation für `upsert_text_only` im Rahmen der Transaktions- und Facade-Orchestrierung. |
| `pub async fn search_text(&self, text: &str, k: usize) -> Result<Vec<SearchResult>>` | `crates/contextra-db/src/contextra_impl/crud.rs:213` | Dokumentation für `search_text` im Rahmen der Transaktions- und Facade-Orchestrierung. |
| `pub async fn search_filtered( &self, query: &[f32], k: usize, filter: Option<&(dyn Fn(DocId) -> bool + Send + Sync)>, ) -> Result<Vec<SearchResult>>` | `crates/contextra-db/src/contextra_impl/crud.rs:227` | Dokumentation für `search_filtered` im Rahmen der Transaktions- und Facade-Orchestrierung. |
| `pub async fn hybrid_search( &self, text: &str, vector: &[f32], k: usize, anchor_entities: Option<&[contextra_types::EntityId]>, ) -> Result<Vec<SearchResult>>` | `crates/contextra-db/src/contextra_impl/crud.rs:244` | Dokumentation für `hybrid_search` im Rahmen der Transaktions- und Facade-Orchestrierung. |
| `pub async fn hybrid_search_reranked( &self, text: &str, vector: &[f32], k: usize, reranker: Option<&dyn contextra_ports::Reranker>, anchor_entities: Option<&[contextra_types::EntityId]>, ) -> Result<Vec<SearchResult>>` | `crates/contextra-db/src/contextra_impl/crud.rs:263` | Dokumentation für `hybrid_search_reranked` im Rahmen der Transaktions- und Facade-Orchestrierung. |
| `pub async fn hybrid_search_with_weights( &self, text: &str, vector: &[f32], k: usize, anchor_entities: Option<&[contextra_types::EntityId]>, weights: Option<&contextra_types::FusionWeights>, ) -> Result<Vec<SearchResult>>` | `crates/contextra-db/src/contextra_impl/crud.rs:286` | Dokumentation für `hybrid_search_with_weights` im Rahmen der Transaktions- und Facade-Orchestrierung. |
| `pub async fn hybrid_search_with_strategy( &self, text: &str, vector: &[f32], k: usize, anchor_entities: Option<&[contextra_types::EntityId]>, weights: Option<&contextra_types::FusionWeights>, strategy: Option<&contextra_types::GraphTraversalStrategy>, ) -> Result<Vec<SearchResult>>` | `crates/contextra-db/src/contextra_impl/crud.rs:309` | Dokumentation für `hybrid_search_with_strategy` im Rahmen der Transaktions- und Facade-Orchestrierung. |
| `pub async fn hybrid_search_with_query( &self, query: &contextra_types::HybridQuery, ) -> Result<Vec<SearchResult>>` | `crates/contextra-db/src/contextra_impl/crud.rs:336` | Dokumentation für `hybrid_search_with_query` im Rahmen der Transaktions- und Facade-Orchestrierung. |
| `pub async fn delete(&self, id: &str) -> Result<()>` | `crates/contextra-db/src/contextra_impl/crud.rs:350` | Dokumentation für `delete` im Rahmen der Transaktions- und Facade-Orchestrierung. |
| `pub async fn relate(&self, from: &str, to: &str, label: &str) -> Result<()>` | `crates/contextra-db/src/contextra_impl/crud.rs:357` | Dokumentation für `relate` im Rahmen der Transaktions- und Facade-Orchestrierung. |
| `pub async fn relate_bidirectional(&self, from: &str, to: &str, label: &str) -> Result<()>` | `crates/contextra-db/src/contextra_impl/crud.rs:365` | Dokumentation für `relate_bidirectional` im Rahmen der Transaktions- und Facade-Orchestrierung. |
| `pub async fn scan_prefix( &self, prefix: &str, limit: Option<usize>, ) -> Result<Vec<(String, Value)>>` | `crates/contextra-db/src/contextra_impl/crud.rs:373` | Dokumentation für `scan_prefix` im Rahmen der Transaktions- und Facade-Orchestrierung. |
| `pub async fn len(&self) -> Result<usize>` | `crates/contextra-db/src/contextra_impl/crud.rs:384` | Dokumentation für `len` im Rahmen der Transaktions- und Facade-Orchestrierung. |
| `pub async fn is_empty(&self) -> Result<bool>` | `crates/contextra-db/src/contextra_impl/crud.rs:391` | Dokumentation für `is_empty` im Rahmen der Transaktions- und Facade-Orchestrierung. |
| `pub async fn scan( &self, start: std::ops::Bound<&[u8]>, end: std::ops::Bound<&[u8]>, limit: Option<usize>, ) -> Result<Vec<(String, Value)>>` | `crates/contextra-db/src/contextra_impl/crud.rs:398` | Dokumentation für `scan` im Rahmen der Transaktions- und Facade-Orchestrierung. |
| `pub async fn stats(&self) -> Result<ContextraStats>` | `crates/contextra-db/src/contextra_impl/diagnostics.rs:17` | Dokumentation für `stats` im Rahmen der Transaktions- und Facade-Orchestrierung. |
| `pub async fn flush(&self) -> Result<()>` | `crates/contextra-db/src/contextra_impl/diagnostics.rs:58` | Dokumentation für `flush` im Rahmen der Transaktions- und Facade-Orchestrierung. |
| `pub fn orphan_registry(&self) -> &Arc<contextra_checkpoint::InstanceOrphanRegistry>` | `crates/contextra-db/src/contextra_impl/diagnostics.rs:65` | Liefert Zugriff auf die Registry verwaister Knoten. |
| `pub async fn open(path: impl AsRef<Path>) -> Result<Self>` | `crates/contextra-db/src/contextra_impl/lifecycle.rs:24` | Opens or creates a `Contextra` instance at `path` in standard durability mode and using default [`crate::license_seam::LicenseGate`]. |
| `pub async fn open_with_config( path: impl AsRef<Path>, mut config: ContextraConfig, ) -> Result<Self>` | `crates/contextra-db/src/contextra_impl/lifecycle.rs:38` | Opens or creates a `Contextra` instance at `path` with explicit `config` using default [`LicenseGate`]. |
| `pub async fn open_authorized( path: impl AsRef<Path>, mut config: ContextraConfig, authorized_ring: &AuthorizedRing, gate: Arc<dyn LicenseGate>, ) -> Result<Self>` | `crates/contextra-db/src/contextra_impl/lifecycle.rs:53` | Opens or creates a `Contextra` instance with an explicitly authorized [`AuthorizedRing`] token and attached [`LicenseGate`]. |
| `pub async fn repair_on_open(&self) -> Result<()>` | `crates/contextra-db/src/contextra_impl/lifecycle.rs:241` | Dokumentation für `repair_on_open` im Rahmen der Transaktions- und Facade-Orchestrierung. |
| `pub fn shutdown(&self)` | `crates/contextra-db/src/contextra_impl/lifecycle.rs:461` | Signalisiert den Shutdown aller Hintergrunddienste. |
| `pub async fn wait_shutdown(&self)` | `crates/contextra-db/src/contextra_impl/lifecycle.rs:467` | Wartet asynchron auf den geordneten Stopp aller Worker. |
| `pub async fn close(self) -> Result<()>` | `crates/contextra-db/src/contextra_impl/lifecycle.rs:486` | Dokumentation für `close` im Rahmen der Transaktions- und Facade-Orchestrierung. |
| `pub async fn with_embedder(self, embedder: Arc<dyn TextEmbeddingEngine>) -> Self` | `crates/contextra-db/src/contextra_impl/lifecycle.rs:496` | Dokumentation für `with_embedder` im Rahmen der Transaktions- und Facade-Orchestrierung. |
| `pub async fn set_embedder(&self, embedder: Arc<dyn TextEmbeddingEngine>) -> Result<()>` | `crates/contextra-db/src/contextra_impl/lifecycle.rs:513` | Dokumentation für `set_embedder` im Rahmen der Transaktions- und Facade-Orchestrierung. |
| `pub fn set_metrics_sink(&self, sink: Arc<dyn contextra_ports::MetricsSink>)` | `crates/contextra-db/src/contextra_impl/wiring.rs:13` | Configures or replaces the active [`MetricsSink`]. |
| `pub fn metrics_sink(&self) -> Arc<dyn contextra_ports::MetricsSink>` | `crates/contextra-db/src/contextra_impl/wiring.rs:18` | Returns the active [`MetricsSink`]. |
| `pub fn set_router(&self, router: std::sync::Weak<dyn DriftStatusProvider>)` | `crates/contextra-db/src/contextra_impl/wiring.rs:24` | Registriert die Router-Komponente. |
| `pub fn set_calibrator( &self, calibrator: std::sync::Weak<parking_lot::Mutex<contextra_rank::IsotonicCalibrator>>, )` | `crates/contextra-db/src/contextra_impl/wiring.rs:30` | Registriert die Score-Kalibrator-Komponente. |
| `pub async fn export_memories(&self) -> Result<ExportDocumentV1>` | `crates/contextra-db/src/export.rs:379` | Exportiert alle Collections der Contextra-Datenbank in das ExportDocumentV1 Schema. |
| `pub async fn import_memories(&self, doc: ExportDocumentV1) -> Result<ImportSummary>` | `crates/contextra-db/src/import.rs:115` | Importiert ein ExportDocumentV1 in die Contextra-Datenbank mit Schema-Versionsprüfung und Idempotenz. |
| `pub fn inner_storage(&self) -> Arc<LsmStorage>` | `crates/contextra-db/src/lib.rs:364` |  |
| `pub fn config(&self) -> &ContextraConfig` | `crates/contextra-db/src/lib.rs:368` |  |
| `pub fn license_gate(&self) -> Arc<dyn LicenseGate>` | `crates/contextra-db/src/lib.rs:372` |  |

#### `struct ContextraConfig`
- Ort: `crates/contextra-db/src/lib.rs:221`
```rust
pub struct ContextraConfig
```
Dokumentation: Globale Konfiguration für die Contextra-Engine-Initialisierung.
- Impl-Block `crates/contextra-db/src/lib.rs:245`
- Impl-Block (Trait `std::fmt::Debug`) `crates/contextra-db/src/lib.rs:263`
- Impl-Block (Trait `Default`) `crates/contextra-db/src/lib.rs:303`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `dimension` | `usize` | `crates/contextra-db/src/lib.rs:222` |
| `max_elements` | `usize` | `crates/contextra-db/src/lib.rs:223` |
| `distance_metric` | `contextra_types::DistanceMetric` | `crates/contextra-db/src/lib.rs:224` |
| `encryption_passphrase` | `Option<String>` | `crates/contextra-db/src/lib.rs:225` |
| `max_ram_mb` | `u64` | `crates/contextra-db/src/lib.rs:226` |
| `group_commit_window_micros` | `u64` | `crates/contextra-db/src/lib.rs:227` |
| `durability_mode` | `DurabilityMode` | `crates/contextra-db/src/lib.rs:228` |
| `deletion_proof_active` | `bool` | `crates/contextra-db/src/lib.rs:229` |
| `vector_delete_mode` | `VectorDeleteMode` | `crates/contextra-db/src/lib.rs:230` |
| `memtable_size_limit` | `usize` | `crates/contextra-db/src/lib.rs:231` |
| `expiry_reaper_interval` | `std::time::Duration` | `crates/contextra-db/src/lib.rs:232` |
| `orphan_registry_path` | `Option<std::path::PathBuf>` | `crates/contextra-db/src/lib.rs:233` |
| `community_detection` | `CommunityDetectionConfig` | `crates/contextra-db/src/lib.rs:234` |
| `embedding_backend` | `EmbeddingBackend` | `crates/contextra-db/src/lib.rs:235` |
| `consolidation_enabled` | `bool` | `crates/contextra-db/src/lib.rs:236` |
| `consolidation_interval` | `std::time::Duration` | `crates/contextra-db/src/lib.rs:237` |
| `max_llm_calls_per_cycle` | `usize` | `crates/contextra-db/src/lib.rs:238` |
| `consolidation_launcher` | `Option<ConsolidationLauncher>` | `crates/contextra-db/src/lib.rs:239` |
| `tenant_policy` | `TenantPolicy` | `crates/contextra-db/src/lib.rs:240` |
| `metrics_sink` | `Option<Arc<dyn contextra_ports::MetricsSink>>` | `crates/contextra-db/src/lib.rs:241` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn with_consolidation_launcher(mut self, launcher: ConsolidationLauncher) -> Self` | `crates/contextra-db/src/lib.rs:246` |  |
| `pub fn with_tenant_policy(mut self, policy: TenantPolicy) -> Self` | `crates/contextra-db/src/lib.rs:251` |  |
| `pub fn with_metrics_sink(mut self, sink: Arc<dyn contextra_ports::MetricsSink>) -> Self` | `crates/contextra-db/src/lib.rs:256` |  |

#### `struct ContextraStats`
- Ort: `crates/contextra-db/src/lib.rs:158`
```rust
pub struct ContextraStats
```
Dokumentation: Aggregierte Betriebs- und Belegungsstatistiken der Engine.

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `drift_status` | `String` | `crates/contextra-db/src/lib.rs:159` |
| `calibration_ece` | `Option<f32>` | `crates/contextra-db/src/lib.rs:161` |
| `last_calibration_at` | `Option<u64>` | `crates/contextra-db/src/lib.rs:163` |
| `active_memory_count` | `usize` | `crates/contextra-db/src/lib.rs:164` |
| `pid_pool_size` | `Option<usize>` | `crates/contextra-db/src/lib.rs:166` |
| `index_stats` | `contextra_ports::VectorIndexStats` | `crates/contextra-db/src/lib.rs:167` |
| `storage_stats` | `contextra_ports::StorageStats` | `crates/contextra-db/src/lib.rs:168` |

#### `struct Document`
- Ort: `crates/contextra-db/src/lib.rs:151`
```rust
pub struct Document
```
Dokumentation: Repräsentiert ein Dokument mit eindeutiger ID, Text, Vektoreinbettung und Metadaten.

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `id` | `String` | `crates/contextra-db/src/lib.rs:152` |
| `metadata` | `Option<Value>` | `crates/contextra-db/src/lib.rs:153` |

#### `enum EmbeddingBackend`
- Ort: `crates/contextra-db/src/lib.rs:191`
```rust
pub enum EmbeddingBackend
```
Dokumentation: Wählbares Backend zur Berechnung von Text-Embeddings.
- Impl-Block (Trait `Default`) `crates/contextra-db/src/lib.rs:199`

Varianten:

- `Onnx{` — `crates/contextra-db/src/lib.rs:192`
- `None` — `crates/contextra-db/src/lib.rs:196`

#### `enum TenantPolicy`
- Ort: `crates/contextra-db/src/lib.rs:210`
```rust
pub enum TenantPolicy
```
Dokumentation: Tenant isolation enforcement policy.

Varianten:

- `Optional` — `crates/contextra-db/src/lib.rs:213`
- `Required` — `crates/contextra-db/src/lib.rs:215`

#### `trait SandboxBridge`
- Ort: `crates/contextra-db/src/lib.rs:378`
```rust
pub trait SandboxBridge: Send + Sync
```
- Gate: `#[cfg(feature = "sandbox")]`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn db_search<'a>( &'a self, query: &'a [u8], k: usize, ) -> BoxFuture<'a, contextra_types::Result<Vec<u8>>>` | `crates/contextra-db/src/lib.rs:379` |  |
| `fn db_insert<'a>( &'a self, key: &'a [u8], value: &'a [u8], ) -> BoxFuture<'a, contextra_types::Result<()>>` | `crates/contextra-db/src/lib.rs:384` |  |
| `fn db_get<'a>( &'a self, key: &'a [u8], ) -> BoxFuture<'a, contextra_types::Result<Option<Vec<u8>>>>` | `crates/contextra-db/src/lib.rs:389` |  |

#### `fn register_consolidation_launcher`
- Ort: `crates/contextra-db/src/lib.rs:128`
```rust
pub fn register_consolidation_launcher<F>(_f: F) where F: Fn( Arc<Collection<LsmStorage>>, std::time::Duration, usize, tokio_util::sync::CancellationToken, ) -> tokio::task::JoinHandle<()> + Send + Sync + 'static,
```

#### `type ConsolidationLauncher`
- Ort: `crates/contextra-db/src/lib.rs:113`
```rust
pub type ConsolidationLauncher = Arc< dyn Fn( Arc<Collection<LsmStorage>>, std::time::Duration, usize, tokio_util::sync::CancellationToken, ) -> tokio::task::JoinHandle<()> + Send + Sync, >
```
Dokumentation: Funktions-Typdefinition zum Starten von Wissenskonsolidierungs-Workern.

#### `type DbStats`
- Ort: `crates/contextra-db/src/lib.rs:173`
```rust
pub type DbStats = ContextraStats
```
Dokumentation: Alias für `ContextraStats` zur Abwärtskompatibilität.

#### `type TenantCollectionMap`
- Ort: `crates/contextra-db/src/lib.rs:331`
```rust
pub type TenantCollectionMap = ahash::AHashMap< (TenantId, String), Arc<Collection<contextra_store::tenant_codec::TenantScopedStorage<Arc<LsmStorage>>>>, >
```

### Modul `decay_controller`

#### `struct AdaptiveDecayController`
- Ort: `crates/contextra-db/src/decay_controller.rs:68`
```rust
pub struct AdaptiveDecayController
```
Dokumentation: Adaptive Decay Controller für time-weighted Cache-Eviction mit injizierter Clock.
- Impl-Block (Trait `std::fmt::Debug`) `crates/contextra-db/src/decay_controller.rs:74`
- Impl-Block `crates/contextra-db/src/decay_controller.rs:83`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(config: DecayControllerConfig) -> Self` | `crates/contextra-db/src/decay_controller.rs:86` | Erstellt eine neue Instanz des `AdaptiveDecayController` mit der gegebenen Konfiguration. |
| `pub fn with_clock(config: DecayControllerConfig, clock: Arc<dyn Clock>) -> Self` | `crates/contextra-db/src/decay_controller.rs:95` | Erstellt eine neue Instanz des `AdaptiveDecayController` mit injizierter Clock (B11). |
| `pub fn with_base_half_life_ms(mut self, base_ms: u64) -> Self` | `crates/contextra-db/src/decay_controller.rs:104` | Setzt die Basis-Halbwertszeit in Millisekunden für Wall-Clock-Decay. |
| `pub fn with_defaults() -> Self` | `crates/contextra-db/src/decay_controller.rs:111` | Initialisiert den Controller mit den Standardwerten der Spezifikation. |
| `pub fn set_clock(&mut self, clock: Arc<dyn Clock>)` | `crates/contextra-db/src/decay_controller.rs:116` | Setzt oder aktualisiert die injizierte Clock. |
| `pub fn system_temperature(&self, inputs: &DecaySignalInputs) -> f32` | `crates/contextra-db/src/decay_controller.rs:123` | Berechnet die aktuelle Systemtemperatur basierend auf Tombstone-Verhältnis und Abfragelast. |
| `pub fn effective_half_life(&self, temperature: f32) -> f32` | `crates/contextra-db/src/decay_controller.rs:131` | Berechnet die temperaturangepasste effektive Halbwertszeit in Transaktionszyklen. |
| `pub fn effective_half_life_ms(&self, temperature: f32) -> f32` | `crates/contextra-db/src/decay_controller.rs:137` | Berechnet die temperaturangepasste effektive Halbwertszeit in Millisekunden. |
| `pub fn effective_score( &self, base_score: f32, elapsed_tx: u64, inputs: &DecaySignalInputs, ) -> f32` | `crates/contextra-db/src/decay_controller.rs:144` | Berechnet den aktuellen abgewerteten Score eines Objekts unter Berücksichtigung von Alter und Systemtemperatur. |
| `pub fn effective_score_wall_clock( &self, base_score: f32, created_at_ms: u64, inputs: &DecaySignalInputs, ) -> f32` | `crates/contextra-db/src/decay_controller.rs:157` | Berechnet den abgewerteten Score basierend auf der Zeitdifferenz zur injizierten Clock. |
| `pub fn should_evict( &self, base_score: f32, elapsed_tx: u64, inputs: &DecaySignalInputs, ) -> bool` | `crates/contextra-db/src/decay_controller.rs:178` | Prüft, ob ein Eintrag aufgrund seines verfallenen Scores unter den Eviktions-Schwellenwert gefallen ist. |
| `pub fn should_evict_wall_clock( &self, base_score: f32, created_at_ms: u64, inputs: &DecaySignalInputs, ) -> bool` | `crates/contextra-db/src/decay_controller.rs:189` | Prüft, ob ein Eintrag unter Verwendung der injizierten Clock eviktiert werden soll. |

#### `struct DecayControllerConfig`
- Ort: `crates/contextra-db/src/decay_controller.rs:47`
```rust
pub struct DecayControllerConfig
```
Dokumentation: Konfiguration für den Adaptive Decay Controller.
- Impl-Block (Trait `Default`) `crates/contextra-db/src/decay_controller.rs:56`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `kappa` | `f32` | `crates/contextra-db/src/decay_controller.rs:49` |
| `base_half_life_tx` | `u64` | `crates/contextra-db/src/decay_controller.rs:51` |
| `eviction_threshold` | `f32` | `crates/contextra-db/src/decay_controller.rs:53` |

#### `struct DecaySignalInputs`
- Ort: `crates/contextra-db/src/decay_controller.rs:23`
```rust
pub struct DecaySignalInputs
```
Dokumentation: Inputs für den Adaptive Decay Controller.
- Impl-Block (Trait `Default`) `crates/contextra-db/src/decay_controller.rs:34`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `tombstone_ratio` | `f32` | `crates/contextra-db/src/decay_controller.rs:25` |
| `query_load_inverse` | `f32` | `crates/contextra-db/src/decay_controller.rs:27` |
| `w_tombstone` | `f32` | `crates/contextra-db/src/decay_controller.rs:29` |
| `w_query` | `f32` | `crates/contextra-db/src/decay_controller.rs:31` |

### Modul `export`

#### `struct ExportCollectionV1`
- Ort: `crates/contextra-db/src/export.rs:37`
```rust
pub struct ExportCollectionV1
```
Dokumentation: Export-Repräsentation einer einzelnen Collection.

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `name` | `String` | `crates/contextra-db/src/export.rs:39` |
| `memories` | `Vec<ExportMemoryV1>` | `crates/contextra-db/src/export.rs:41` |
| `relations` | `Vec<ExportRelationV1>` | `crates/contextra-db/src/export.rs:43` |

#### `struct ExportDocumentV1`
- Ort: `crates/contextra-db/src/export.rs:25`
```rust
pub struct ExportDocumentV1
```
Dokumentation: Wurzel-Dokument für den Export aller DB-Collections.

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `schema_version` | `String` | `crates/contextra-db/src/export.rs:27` |
| `exported_at` | `Option<String>` | `crates/contextra-db/src/export.rs:30` |
| `collections` | `Vec<ExportCollectionV1>` | `crates/contextra-db/src/export.rs:32` |

#### `struct ExportMemoryV1`
- Ort: `crates/contextra-db/src/export.rs:48`
```rust
pub struct ExportMemoryV1
```
Dokumentation: Export-Repräsentation eines einzelnen Memory-Eintrags.

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `id` | `String` | `crates/contextra-db/src/export.rs:50` |
| `memory_type` | `MemoryType` | `crates/contextra-db/src/export.rs:53` |
| `content` | `Option<String>` | `crates/contextra-db/src/export.rs:56` |
| `embedding` | `Option<Vec<f32>>` | `crates/contextra-db/src/export.rs:59` |
| `embedding_model` | `Option<String>` | `crates/contextra-db/src/export.rs:62` |
| `created_at` | `Option<u64>` | `crates/contextra-db/src/export.rs:65` |
| `importance_score` | `f32` | `crates/contextra-db/src/export.rs:67` |
| `metadata` | `Option<serde_json::Value>` | `crates/contextra-db/src/export.rs:70` |
| `links` | `Vec<MemoryLink>` | `crates/contextra-db/src/export.rs:73` |

#### `struct ExportRelationV1`
- Ort: `crates/contextra-db/src/export.rs:78`
```rust
pub struct ExportRelationV1
```
Dokumentation: Export-Repräsentation einer Graph-Beziehung.

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `from` | `String` | `crates/contextra-db/src/export.rs:80` |
| `to` | `String` | `crates/contextra-db/src/export.rs:82` |
| `label` | `String` | `crates/contextra-db/src/export.rs:84` |

#### `const SCHEMA_VERSION_V1`
- Ort: `crates/contextra-db/src/export.rs:21`
```rust
pub const SCHEMA_VERSION_V1: &str = ""
```
Dokumentation: Aktuelle Schema-Version für das JSON Export-Format.

### Modul `extraction`

#### `mod open_ie`
- Ort: `crates/contextra-db/src/extraction/mod.rs:17`
```rust
pub mod open_ie
```

#### `mod rule_based`
- Ort: `crates/contextra-db/src/extraction/mod.rs:18`
```rust
pub mod rule_based
```

#### `mod types`
- Ort: `crates/contextra-db/src/extraction/mod.rs:21`
```rust
pub mod types
```

#### `struct OpenIeExtractor`
- Ort: `crates/contextra-db/src/extraction/mod.rs:43`
```rust
pub struct OpenIeExtractor
```
Dokumentation: Implementierung des OpenIE-Extraktors hinter `EntityExtractor`.
- Impl-Block (Trait `EntityExtractor`) `crates/contextra-db/src/extraction/mod.rs:45`

#### `trait EntityExtractor`
- Ort: `crates/contextra-db/src/extraction/mod.rs:31`
```rust
pub trait EntityExtractor: Send + Sync
```
Dokumentation: Gemeinsamer Trait für alle Entitätsextraktoren in Contextra-Engine.

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn extract_triples<'a>( &'a self, text: &'a str, generator: Option<&'a dyn LlmTextGenerator>, config: &'a EntityExtractionConfig, ) -> BoxFuture<'a, Result<Vec<ExtractedTriple>>>` | `crates/contextra-db/src/extraction/mod.rs:33` | Extrahiert Wissens-Tripel aus dem übergebenen Freitext. |

#### `fn create_extractor`
- Ort: `crates/contextra-db/src/extraction/mod.rs:75`
```rust
pub fn create_extractor(mode: ExtractorMode) -> Box<dyn EntityExtractor>
```
Dokumentation: Erstellt eine Instanz des konfigurierten Extraktors.

#### `fn extract_triples_for_config`
- Ort: `crates/contextra-db/src/extraction/mod.rs:94`
```rust
pub async fn extract_triples_for_config( text: &str, generator: Option<&dyn LlmTextGenerator>, config: &EntityExtractionConfig, ) -> Result<Vec<ExtractedTriple>>
```
Dokumentation: Hilfsfunktion zur Ausführung der Extraktion basierend auf der Standard-Konfiguration (OpenIE).

#### `fn extract_triples_for_mode`
- Ort: `crates/contextra-db/src/extraction/mod.rs:83`
```rust
pub async fn extract_triples_for_mode( mode: ExtractorMode, text: &str, generator: Option<&dyn LlmTextGenerator>, config: &EntityExtractionConfig, ) -> Result<Vec<ExtractedTriple>>
```
Dokumentation: Hilfsfunktion zur Ausführung der Extraktion für einen expliziten Extraktor-Modus.

### Modul `extraction::open_ie`

#### `fn extract_triples`
- Ort: `crates/contextra-db/src/extraction/open_ie.rs:80`
```rust
pub async fn extract_triples( text: &str, generator: &dyn LlmTextGenerator, config: &EntityExtractionConfig, ) -> Result<Vec<ExtractedTriple>>
```
Dokumentation: Extrahiert Wissens-Tripel (Subjekt, Prädikat, Objekt) aus dem gegebenen Text mittels LLM. # Fehlerverhalten - Wenn `config.enabled == false`, wird sofort eine leere Liste zurückgegeben. - Wenn die Anzahl der Textchunks `config.max_llm_calls_per_cycle` überschreitet, wird abgebrochen und ein `ContextraError::LimitExceeded` zurückgegeben. - Fehlerhafte LLM-Antworten werden in `ContextraError::ParseError` oder `ContextraError::Serialization` umgewandelt. Es wird niemals gepanict.

### Modul `extraction::rule_based`

#### `struct RuleBasedExtractor`
- Ort: `crates/contextra-db/src/extraction/rule_based.rs:125`
```rust
pub struct RuleBasedExtractor
```
Dokumentation: Regelbasierter Entitäts- und Relationsextraktor.
- Impl-Block (Trait `EntityExtractor`) `crates/contextra-db/src/extraction/mod.rs:63`
- Impl-Block `crates/contextra-db/src/extraction/rule_based.rs:127`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new() -> Self` | `crates/contextra-db/src/extraction/rule_based.rs:129` | Erstellt eine neue Instanz von `RuleBasedExtractor`. |
| `pub fn extract( &self, text: &str, config: &EntityExtractionConfig, ) -> Result<Vec<ExtractedTriple>>` | `crates/contextra-db/src/extraction/rule_based.rs:134` | Extrahieren von Wissens-Tripeln rein regelbasiert und deterministisch. |

#### `const MAX_EXTRACTED_TRIPLES`
- Ort: `crates/contextra-db/src/extraction/rule_based.rs:22`
```rust
pub const MAX_EXTRACTED_TRIPLES: usize = 5_000
```
Dokumentation: Maximale Anzahl extrahierter Tripel pro Durchlauf.

#### `const MAX_EXTRACTION_TEXT_LENGTH`
- Ort: `crates/contextra-db/src/extraction/rule_based.rs:19`
```rust
pub const MAX_EXTRACTION_TEXT_LENGTH: usize = 500_000
```
Dokumentation: Maximale zulässige Eingabelänge in Zeichen für die regelbasierte Extraktion.

### Modul `extraction::types`

#### `struct EntityExtractionConfig`
- Ort: `crates/contextra-db/src/extraction/types.rs:36`
```rust
pub struct EntityExtractionConfig
```
Dokumentation: Konfiguration für die automatische Entitätsextraktion.

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `enabled` | `bool` | `crates/contextra-db/src/extraction/types.rs:38` |
| `max_llm_calls_per_cycle` | `usize` | `crates/contextra-db/src/extraction/types.rs:40` |
| `min_confidence` | `f32` | `crates/contextra-db/src/extraction/types.rs:42` |

#### `struct ExtractedTriple`
- Ort: `crates/contextra-db/src/extraction/types.rs:21`
```rust
pub struct ExtractedTriple
```
Dokumentation: Extrahiertes Wissens-Tripel (Subjekt, Prädikat, Objekt).

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `subject` | `String` | `crates/contextra-db/src/extraction/types.rs:23` |
| `predicate` | `String` | `crates/contextra-db/src/extraction/types.rs:25` |
| `object` | `String` | `crates/contextra-db/src/extraction/types.rs:27` |
| `confidence` | `f32` | `crates/contextra-db/src/extraction/types.rs:29` |

#### `enum ExtractorMode`
- Ort: `crates/contextra-db/src/extraction/types.rs:57`
```rust
pub enum ExtractorMode
```
Dokumentation: Modus der Entitätsextraktion (LLM-basiertes OpenIE oder regelbasiert).

Varianten:

- `OpenIe` — `crates/contextra-db/src/extraction/types.rs:59`
- `RuleBased` — `crates/contextra-db/src/extraction/types.rs:60`

### Modul `filter`

#### `enum FilterOp`
- Ort: `crates/contextra-db/src/filter.rs:84`
```rust
pub enum FilterOp
```
Dokumentation: Vergleichsoperationen für Metadaten-Prädikatfilter (z. B. Gleichheit, Größer, Kleiner, Inklusion).

Varianten:

- `Eq` — `crates/contextra-db/src/filter.rs:86`
- `Ne` — `crates/contextra-db/src/filter.rs:88`
- `Gt` — `crates/contextra-db/src/filter.rs:90`
- `Gte` — `crates/contextra-db/src/filter.rs:92`
- `Lt` — `crates/contextra-db/src/filter.rs:94`
- `Lte` — `crates/contextra-db/src/filter.rs:96`
- `In` — `crates/contextra-db/src/filter.rs:98`
- `NotIn` — `crates/contextra-db/src/filter.rs:100`
- `Exists` — `crates/contextra-db/src/filter.rs:102`

#### `enum MetadataFilter`
- Ort: `crates/contextra-db/src/filter.rs:113`
```rust
pub enum MetadataFilter
```
- Impl-Block `crates/contextra-db/src/filter.rs:130`

Varianten:

- `Condition{` — `crates/contextra-db/src/filter.rs:115`
- `And(Vec<MetadataFilter>)` — `crates/contextra-db/src/filter.rs:122`
- `Or(Vec<MetadataFilter>)` — `crates/contextra-db/src/filter.rs:124`
- `Not(Box<MetadataFilter>)` — `crates/contextra-db/src/filter.rs:126`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn validate_bounds(&self) -> Result<()>` | `crates/contextra-db/src/filter.rs:132` | Validiert, dass der `MetadataFilter` die Tiefen- und Knotengrenzen einhält. |
| `pub fn matches(&self, metadata: &Value) -> bool` | `crates/contextra-db/src/filter.rs:176` | Prüft, ob das gegebene JSON-Metadatenobjekt die Filterbedingungen erfüllt. |

#### `fn extract_memory_type`
- Ort: `crates/contextra-db/src/filter.rs:68`
```rust
pub fn extract_memory_type(metadata: &Option<Value>) -> MemoryType
```
Dokumentation: Extrahiert den MemoryType aus Dokument-Metadaten (Rückwärtskompatibel).

#### `fn validate_filter_expr_bounds`
- Ort: `crates/contextra-db/src/filter.rs:26`
```rust
pub fn validate_filter_expr_bounds(expr: &FilterExpr) -> Result<()>
```
Dokumentation: Validiert, dass ein `FilterExpr` die Tiefen- und Knotengrenzen einhält.

#### `const MAX_FILTER_DEPTH`
- Ort: `crates/contextra-db/src/filter.rs:20`
```rust
pub const MAX_FILTER_DEPTH: usize = 16
```
Dokumentation: Maximale Rekursionstiefe für verschachtelte Filterausdrücke (Schutz vor Stack Overflow).

#### `const MAX_FILTER_NODES`
- Ort: `crates/contextra-db/src/filter.rs:23`
```rust
pub const MAX_FILTER_NODES: usize = 128
```
Dokumentation: Maximale Anzahl von Filterknoten (Schutz vor DoS / unbeschränkter Allokation).

### Modul `fusion`

#### `fn fuse_search_results_with_signal_strategies`
- Ort: `crates/contextra-db/src/fusion.rs:48`
```rust
pub fn fuse_search_results_with_signal_strategies( result_sets: Vec<(String, Vec<SearchResult>, f32)>, max_results: usize, priority: MetadataMergePriority, include_provenance: bool, resonance_config: Option<&ResonanceConfig>, strategies: contextra_types::SignalFusionStrategies, ) -> Vec<SearchResult>
```
Dokumentation: Fuses search result sets using per-signal fusion strategies (`SignalFusionStrategies`).

#### `fn fuse_search_results_with_strategy`
- Ort: `crates/contextra-db/src/fusion.rs:29`
```rust
pub fn fuse_search_results_with_strategy( result_sets: Vec<(String, Vec<SearchResult>, f32)>, max_results: usize, priority: MetadataMergePriority, include_provenance: bool, resonance_config: Option<&ResonanceConfig>, strategy: contextra_types::FusionStrategy, ) -> Vec<SearchResult>
```
Dokumentation: Fuses search result sets using the specified FusionStrategy.

#### `fn fuse_signals`
- Ort: `crates/contextra-db/src/fusion.rs:67`
```rust
pub fn fuse_signals( result_sets: Vec<(String, Vec<SearchResult>, f32)>, max_results: usize, ) -> Vec<SearchResult>
```
Dokumentation: Convenience function for fusing multi-signal search results.

#### `fn weights_to_signal_factors`
- Ort: `crates/contextra-db/src/fusion.rs:21`
```rust
pub fn weights_to_signal_factors( weights: Option<&contextra_types::FusionWeights>, ) -> (f32, f32, f32)
```
Dokumentation: Berechnet Signalfaktoren aus den Workspace-Fusionsgewichten. Konvertiert die `contextra_types`-Gewichte an der Grenze in den Rank-Typ.

### Modul `homeostat`

#### `struct RerankDeadline`
- Ort: `crates/contextra-db/src/homeostat.rs:10`
```rust
pub struct RerankDeadline
```
- Impl-Block `crates/contextra-db/src/homeostat.rs:14`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(timeout: Duration) -> Self` | `crates/contextra-db/src/homeostat.rs:15` |  |
| `pub fn is_expired(&self) -> bool` | `crates/contextra-db/src/homeostat.rs:21` |  |
| `pub fn remaining(&self) -> Duration` | `crates/contextra-db/src/homeostat.rs:25` |  |

#### `struct RerankPidController`
- Ort: `crates/contextra-db/src/homeostat.rs:32`
```rust
pub struct RerankPidController
```
- **deprecated**
- Impl-Block `crates/contextra-db/src/homeostat.rs:38`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(target_ms: f32, initial_pool: usize) -> Self` | `crates/contextra-db/src/homeostat.rs:39` |  |
| `pub fn pool_size(&self) -> usize` | `crates/contextra-db/src/homeostat.rs:46` |  |
| `pub fn update(&mut self, elapsed_ms: f32) -> usize` | `crates/contextra-db/src/homeostat.rs:50` |  |

### Modul `import`

#### `struct ImportSummary`
- Ort: `crates/contextra-db/src/import.rs:22`
```rust
pub struct ImportSummary
```
Dokumentation: Zusammenfassung eines Import-Vorgangs.

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `imported_memories` | `usize` | `crates/contextra-db/src/import.rs:24` |
| `skipped_memories` | `usize` | `crates/contextra-db/src/import.rs:26` |
| `imported_relations` | `usize` | `crates/contextra-db/src/import.rs:28` |
| `skipped_relations` | `usize` | `crates/contextra-db/src/import.rs:30` |

#### `const MAX_IMPORT_COLLECTIONS`
- Ort: `crates/contextra-db/src/import.rs:111`
```rust
pub const MAX_IMPORT_COLLECTIONS: usize = 10_000
```
Dokumentation: Maximum allowable collections in an import document (bounded resource principle).

### Modul `merge_operator_adapter`

#### `struct WasmMergeOperator`
- Ort: `crates/contextra-db/src/merge_operator_adapter.rs:46`
```rust
pub struct WasmMergeOperator
```
Dokumentation: Synchronous wrapper around [`WasmMergeFunction`] implementing [`MergeOperator`].
- Impl-Block (Trait `std::fmt::Debug`) `crates/contextra-db/src/merge_operator_adapter.rs:52`
- Impl-Block `crates/contextra-db/src/merge_operator_adapter.rs:60`
- Impl-Block (Trait `MergeOperator`) `crates/contextra-db/src/merge_operator_adapter.rs:133`
- Impl-Block (Trait `Drop`) `crates/contextra-db/src/merge_operator_adapter.rs:178`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(wasm_bytes: impl Into<Arc<[u8]>>) -> Result<Self>` | `crates/contextra-db/src/merge_operator_adapter.rs:62` | Creates a new `WasmMergeOperator` with default capability limits (10,000,000 fuel, 5s timeout). |
| `pub fn new_with_config( wasm_bytes: impl Into<Arc<[u8]>>, max_fuel: u64, wall_clock_timeout: Duration, ) -> Result<Self>` | `crates/contextra-db/src/merge_operator_adapter.rs:76` | Creates a new `WasmMergeOperator` with custom `max_fuel` and `wall_clock_timeout` configuration. # Errors Returns `ContextraError::Sandbox` if WASM validation f |

### Modul `metrics_names`

#### `fn record_counter_safe`
- Ort: `crates/contextra-db/src/metrics_names.rs:57`
```rust
pub fn record_counter_safe( sink: &dyn contextra_ports::MetricsSink, name: &str, value: u64, labels: &[(&str, &str)], )
```
Dokumentation: Helper to record a counter metric best-effort without allowing panics to propagate.

#### `fn record_gauge_safe`
- Ort: `crates/contextra-db/src/metrics_names.rs:45`
```rust
pub fn record_gauge_safe( sink: &dyn contextra_ports::MetricsSink, name: &str, value: f64, labels: &[(&str, &str)], )
```
Dokumentation: Helper to record a gauge metric best-effort without allowing panics to propagate.

#### `fn record_histogram_safe`
- Ort: `crates/contextra-db/src/metrics_names.rs:33`
```rust
pub fn record_histogram_safe( sink: &dyn contextra_ports::MetricsSink, name: &str, value: f64, labels: &[(&str, &str)], )
```
Dokumentation: Helper to record a histogram metric best-effort without allowing panics to propagate.

#### `const BACKPRESSURE_LEVEL`
- Ort: `crates/contextra-db/src/metrics_names.rs:12`
```rust
pub const BACKPRESSURE_LEVEL: &str = ""
```
Dokumentation: Metric name for system backpressure level gauge readings (0.0 = Normal, 1.0 = Elevated, 2.0 = Critical).

#### `const CHECKPOINT_DURATION`
- Ort: `crates/contextra-db/src/metrics_names.rs:15`
```rust
pub const CHECKPOINT_DURATION: &str = ""
```
Dokumentation: Metric name for checkpoint operation duration histogram observations.

#### `const COMMIT_LATENCY`
- Ort: `crates/contextra-db/src/metrics_names.rs:6`
```rust
pub const COMMIT_LATENCY: &str = ""
```
Dokumentation: Metric name for engine commit latency histogram observations.

#### `const INSERT_LATENCY`
- Ort: `crates/contextra-db/src/metrics_names.rs:27`
```rust
pub const INSERT_LATENCY: &str = ""
```
Dokumentation: Metric name for document insert operation duration histogram observations.

#### `const REAPER_DOCS_REAPED`
- Ort: `crates/contextra-db/src/metrics_names.rs:30`
```rust
pub const REAPER_DOCS_REAPED: &str = ""
```
Dokumentation: Metric name for expired or decayed document reaper count observations.

#### `const REBUILD_DURATION`
- Ort: `crates/contextra-db/src/metrics_names.rs:9`
```rust
pub const REBUILD_DURATION: &str = ""
```
Dokumentation: Metric name for index rebuild duration histogram observations.

#### `const SEARCH_LATENCY_GRAPH`
- Ort: `crates/contextra-db/src/metrics_names.rs:24`
```rust
pub const SEARCH_LATENCY_GRAPH: &str = ""
```
Dokumentation: Reserved metric name for graph signal search latency observations.

#### `const SEARCH_LATENCY_TEXT`
- Ort: `crates/contextra-db/src/metrics_names.rs:21`
```rust
pub const SEARCH_LATENCY_TEXT: &str = ""
```
Dokumentation: Reserved metric name for text signal search latency observations.

#### `const SEARCH_LATENCY_VECTOR`
- Ort: `crates/contextra-db/src/metrics_names.rs:18`
```rust
pub const SEARCH_LATENCY_VECTOR: &str = ""
```
Dokumentation: Reserved metric name for vector signal search latency observations.

### Modul `multistep`

#### `struct MultiStepConfig`
- Ort: `crates/contextra-db/src/multistep.rs:48`
```rust
pub struct MultiStepConfig
```
Dokumentation: Konfiguration für das mehrstufige iterative Multi-Step Retrieval. Steuert die Abbruchbedingungen, Qualitätsanforderungen und Zeitbudgets.
- Impl-Block (Trait `Default`) `crates/contextra-db/src/multistep.rs:62`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `max_rounds` | `usize` | `crates/contextra-db/src/multistep.rs:50` |
| `quality_threshold` | `f32` | `crates/contextra-db/src/multistep.rs:53` |
| `min_quality_hits` | `usize` | `crates/contextra-db/src/multistep.rs:55` |
| `latency_budget_ms` | `f64` | `crates/contextra-db/src/multistep.rs:57` |
| `rerank_pool_multiplier` | `usize` | `crates/contextra-db/src/multistep.rs:59` |

#### `struct MultiStepEngine`
- Ort: `crates/contextra-db/src/multistep.rs:97`
```rust
pub struct MultiStepEngine<S: StorageEngine>
```
Dokumentation: Multi-Step Retrieval Engine für iterative Abfragen. Bietet adaptive, mehrstufige Retrieval-Zyklen auf einer zugrundeliegenden [`Collection`]. Nutzt einen optionalen [`QueryRewriter`] für LLM-basierte Anfrage-Reformulierungen sowie einen internen PID-Regler zur dynamischen Latenzkontrolle.
- Impl-Block `crates/contextra-db/src/multistep.rs:104`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(collection: Arc<Collection<S>>, config: MultiStepConfig) -> Self` | `crates/contextra-db/src/multistep.rs:114` | Erzeugt eine neue `MultiStepEngine` für eine gegebene [`Collection`]. # Parameter - `collection`: Die Contextra-Kollektion, auf der gesucht wird. - `config`: Mu |
| `pub fn with_clock(mut self, clock: Arc<dyn Clock>) -> Self` | `crates/contextra-db/src/multistep.rs:130` | Überschreibt den Zeit-Port mit einem benutzerdefinierten [`Clock`]-Port. # Zweck Ermöglicht deterministische Tests ohne Abhängigkeit von realer Wanduhr-Zeit (ge |
| `pub async fn search( &self, original_query: &str, vector: &[f32], k: usize, rewriter: Option<&dyn QueryRewriter>, ) -> Result<MultiStepResult>` | `crates/contextra-db/src/multistep.rs:143` | Führt eine iterative Hybrid-Suche ohne separaten Vektor-Embedder durch. # Ablauf 1. Runde 1: Standard-Hybrid-Suche mit Originalanfrage und Originalvektor. 2. Qu |
| `pub async fn search_with_embedder( &self, original_query: &str, vector: &[f32], k: usize, rewriter: Option<&dyn QueryRewriter>, embedder: Option<&dyn TextEmbeddingEngine>, ) -> Result<MultiStepResult>` | `crates/contextra-db/src/multistep.rs:174` | Führt eine iterative multi-signale Hybrid-Suche mit Vektor-Embedder durch. # Ablauf im Detail: 1. Eingabeprüfung & Schranke: `k` wird gegen `MAX_SEARCH_K` begre |

#### `struct MultiStepResult`
- Ort: `crates/contextra-db/src/multistep.rs:78`
```rust
pub struct MultiStepResult
```
Dokumentation: Ergebnis einer mehrstufigen Suche inklusive vollständiger Audit- und Nachvollziehbarkeitsdaten.

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `results` | `Vec<SearchResult>` | `crates/contextra-db/src/multistep.rs:80` |
| `rounds_executed` | `usize` | `crates/contextra-db/src/multistep.rs:82` |
| `sub_queries` | `Vec<String>` | `crates/contextra-db/src/multistep.rs:84` |
| `sub_rewrites` | `Vec<QueryRewriteOutput>` | `crates/contextra-db/src/multistep.rs:86` |
| `sub_vectors` | `Vec<Vec<f32>>` | `crates/contextra-db/src/multistep.rs:88` |

#### `const DEFAULT_MAX_SUB_QUERIES`
- Ort: `crates/contextra-db/src/multistep.rs:36`
```rust
pub const DEFAULT_MAX_SUB_QUERIES: usize = 3
```
Dokumentation: Standardmäßige Obergrenze für generierte Teilfragen pro Iterationsrunde (N=3). Verhindert kombinatorische Explosion und hält Latenzschranken ein.

#### `const DEFAULT_RERANK_POOL_MULTIPLIER`
- Ort: `crates/contextra-db/src/multistep.rs:41`
```rust
pub const DEFAULT_RERANK_POOL_MULTIPLIER: usize = 2
```
Dokumentation: Standardmäßiger Multiplikator zur Vergrößerung des Kandidatenpools vor dem Re-Ranking. Verdoppelt die Trefferanzahl (2 * k), um genügend Recall vor Filtern zu sichern.

### Modul `orchestration`

#### `struct Completed`
- Ort: `crates/contextra-db/src/orchestration/mod.rs:97`
```rust
pub struct Completed
```

#### `struct Draft`
- Ort: `crates/contextra-db/src/orchestration/mod.rs:94`
```rust
pub struct Draft
```

#### `struct EngineServiceContainer`
- Ort: `crates/contextra-db/src/orchestration/mod.rs:14`
```rust
pub struct EngineServiceContainer<S, V, T> where S: ?Sized, V: ?Sized, T: ?Sized,
```
Dokumentation: Service-Container für Dependency Injection in State-Machines und Orchestratoren.
- Impl-Block `crates/contextra-db/src/orchestration/mod.rs:25`

#### `struct Executing`
- Ort: `crates/contextra-db/src/orchestration/mod.rs:96`
```rust
pub struct Executing
```

#### `struct Failed`
- Ort: `crates/contextra-db/src/orchestration/mod.rs:98`
```rust
pub struct Failed
```

#### `struct Prepared`
- Ort: `crates/contextra-db/src/orchestration/mod.rs:95`
```rust
pub struct Prepared
```

#### `struct WorkflowContext`
- Ort: `crates/contextra-db/src/orchestration/mod.rs:58`
```rust
pub struct WorkflowContext
```
Dokumentation: Dynamischer Workflow-Kontext zur Übergabe von Zwischenergebnissen zwischen Schritten.
- Impl-Block `crates/contextra-db/src/orchestration/mod.rs:62`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new() -> Self` | `crates/contextra-db/src/orchestration/mod.rs:64` | Erstellt einen leeren Kontext. |
| `pub fn set(&mut self, key: impl Into<String>, val: impl Into<String>)` | `crates/contextra-db/src/orchestration/mod.rs:71` | Setzt einen Schlüssel-Wert-Eintrag. |
| `pub fn get(&self, key: &str) -> Option<&str>` | `crates/contextra-db/src/orchestration/mod.rs:76` | Liest einen Wert aus. |

#### `struct WorkflowState`
- Ort: `crates/contextra-db/src/orchestration/mod.rs:101`
```rust
pub struct WorkflowState<State>
```
Dokumentation: Typestate-basierte State Machine für Workflows.
- Impl-Block `crates/contextra-db/src/orchestration/mod.rs:110`
- Impl-Block `crates/contextra-db/src/orchestration/mod.rs:147`
- Impl-Block `crates/contextra-db/src/orchestration/mod.rs:161`
- Impl-Block `crates/contextra-db/src/orchestration/mod.rs:191`
- Impl-Block `crates/contextra-db/src/orchestration/mod.rs:208`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(workflow_id: impl Into<String>) -> Self` | `crates/contextra-db/src/orchestration/mod.rs:112` | Startet die Erstellung eines neuen Workflows im Zustand `Draft`. |
| `pub fn add_step(mut self, step: Box<dyn WorkflowStep>) -> Self` | `crates/contextra-db/src/orchestration/mod.rs:124` | Fügt dem Entwurf einen Schritt hinzu. |
| `pub fn prepare(self) -> Result<WorkflowState<Prepared>>` | `crates/contextra-db/src/orchestration/mod.rs:130` | Validiert und überführt den Workflow in den Zustand `Prepared`. |
| `pub fn start_execution(self) -> WorkflowState<Executing>` | `crates/contextra-db/src/orchestration/mod.rs:149` | Beginnt die Ausführung und wechselt in den Typestate `Executing`. |
| `pub async fn execute_all( mut self, ) -> std::result::Result<WorkflowState<Completed>, WorkflowState<Failed>>` | `crates/contextra-db/src/orchestration/mod.rs:163` | Führt alle Schritte sequentiell aus und wechselt je nach Ergebnis in `Completed` oder `Failed`. |
| `pub fn workflow_id(&self) -> &str` | `crates/contextra-db/src/orchestration/mod.rs:193` | Liefert die ID des abgeschlossenen Workflows. |
| `pub fn completed_steps(&self) -> usize` | `crates/contextra-db/src/orchestration/mod.rs:198` | Liefert die Anzahl erfolgreich ausgeführter Schritte. |
| `pub fn context(&self) -> &WorkflowContext` | `crates/contextra-db/src/orchestration/mod.rs:203` | Gibt eine Referenz auf den resultierenden Kontext zurück. |
| `pub fn workflow_id(&self) -> &str` | `crates/contextra-db/src/orchestration/mod.rs:210` | Liefert die ID des fehlgeschlagenen Workflows. |
| `pub fn completed_steps(&self) -> usize` | `crates/contextra-db/src/orchestration/mod.rs:215` | Liefert die Anzahl bis zum Abbruch ausgeführter Schritte. |
| `pub fn error(&self) -> Option<&ContextraError>` | `crates/contextra-db/src/orchestration/mod.rs:220` | Liefert den aufgetretenen Fehler. |

#### `trait WorkflowStep`
- Ort: `crates/contextra-db/src/orchestration/mod.rs:82`
```rust
pub trait WorkflowStep: Send + Sync
```
Dokumentation: Trait für zustandslose oder isolierte Workflow-Schritte (Dyn-kompatibel via Pin<Box<dyn Future>>).

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn name(&self) -> &str` | `crates/contextra-db/src/orchestration/mod.rs:84` | Name oder Bezeichner des Schritts. |
| `fn run<'a>( &'a self, context: &'a mut WorkflowContext, ) -> Pin<Box<dyn Future<Output = Result<()>> + Send + 'a>>` | `crates/contextra-db/src/orchestration/mod.rs:87` | Führt den Schritt asynchron auf einem Kontext-Objekt aus. |

### Modul `pid_latency_controller`

#### `struct LatencyBudgetGuard`
- Ort: `crates/contextra-db/src/pid_latency_controller.rs:88`
```rust
pub struct LatencyBudgetGuard
```
- Impl-Block `crates/contextra-db/src/pid_latency_controller.rs:94`
- Impl-Block (Trait `std::fmt::Debug`) `crates/contextra-db/src/pid_latency_controller.rs:127`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(budget_ms: f64) -> Self` | `crates/contextra-db/src/pid_latency_controller.rs:95` |  |
| `pub fn with_clock(budget_ms: f64, clock: Arc<dyn Clock>) -> Self` | `crates/contextra-db/src/pid_latency_controller.rs:99` |  |
| `pub fn elapsed_ms(&self) -> f64` | `crates/contextra-db/src/pid_latency_controller.rs:108` |  |
| `pub fn is_exceeded(&self) -> bool` | `crates/contextra-db/src/pid_latency_controller.rs:114` |  |
| `pub fn remaining_ms(&self) -> f64` | `crates/contextra-db/src/pid_latency_controller.rs:118` |  |
| `pub fn budget_ms(&self) -> f64` | `crates/contextra-db/src/pid_latency_controller.rs:122` |  |

#### `struct PidLatencyController`
- Ort: `crates/contextra-db/src/pid_latency_controller.rs:14`
```rust
pub struct PidLatencyController
```
- Impl-Block (Trait `Default`) `crates/contextra-db/src/pid_latency_controller.rs:24`
- Impl-Block `crates/contextra-db/src/pid_latency_controller.rs:30`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(target_latency_ms: f64) -> Self` | `crates/contextra-db/src/pid_latency_controller.rs:31` |  |
| `pub fn target_latency_ms(&self) -> f64` | `crates/contextra-db/src/pid_latency_controller.rs:43` |  |
| `pub fn current_scaling_factor(&self) -> f64` | `crates/contextra-db/src/pid_latency_controller.rs:47` |  |
| `pub fn update(&mut self, elapsed_ms: f64, dt_seconds: f64) -> f64` | `crates/contextra-db/src/pid_latency_controller.rs:51` |  |
| `pub fn reset(&mut self)` | `crates/contextra-db/src/pid_latency_controller.rs:80` |  |

#### `const DEFAULT_TARGET_LATENCY_MS`
- Ort: `crates/contextra-db/src/pid_latency_controller.rs:9`
```rust
pub const DEFAULT_TARGET_LATENCY_MS: f64 = 100.0
```

#### `const MAX_SCALING_FACTOR`
- Ort: `crates/contextra-db/src/pid_latency_controller.rs:11`
```rust
pub const MAX_SCALING_FACTOR: f64 = 1.0
```

#### `const MIN_SCALING_FACTOR`
- Ort: `crates/contextra-db/src/pid_latency_controller.rs:10`
```rust
pub const MIN_SCALING_FACTOR: f64 = 0.3
```

### Modul `temporal_filter`

#### `struct ValidityWindow`
- Ort: `crates/contextra-db/src/temporal_filter.rs:26`
```rust
pub struct ValidityWindow
```
Dokumentation: Extracted bi-temporal validity window representation for a document or search candidate.
- Impl-Block `crates/contextra-db/src/temporal_filter.rs:37`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `tx_valid_from` | `Option<TxId>` | `crates/contextra-db/src/temporal_filter.rs:28` |
| `tx_valid_to` | `Option<TxId>` | `crates/contextra-db/src/temporal_filter.rs:30` |
| `business_valid_from` | `Option<i64>` | `crates/contextra-db/src/temporal_filter.rs:32` |
| `business_valid_to` | `Option<i64>` | `crates/contextra-db/src/temporal_filter.rs:34` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn is_empty(&self) -> bool` | `crates/contextra-db/src/temporal_filter.rs:43` | Returns `true` if no validity window metadata constraints are set. Candidates without validity window metadata (legacy data) are treated as always valid (Fail-O |
| `pub fn is_valid_bitemporal(&self, as_of_tx: TxId, as_of_business: Option<u64>) -> bool` | `crates/contextra-db/src/temporal_filter.rs:53` | Evaluates bi-temporal validity using strict AND logic across system transaction time and business time. Intervall-Semantik: `[valid_from, valid_until)` (inclusi |

#### `fn apply_temporal_validity_filter`
- Ort: `crates/contextra-db/src/temporal_filter.rs:187`
```rust
pub fn apply_temporal_validity_filter( results: Vec<contextra_rank::SearchResult>, current_tx: TxId, query_timestamp: Option<u64>, ) -> Vec<contextra_rank::SearchResult>
```
Dokumentation: Applies bi-temporal validity filtering to candidates post-RRF fusion. Filters out candidate results that are invalid relative to the current transaction `current_tx` or business timestamp `query_timestamp`. Candidates without validity window metadata (legacy data) are retained (Fail-Open).

#### `fn apply_temporal_validity_filter_at`
- Ort: `crates/contextra-db/src/temporal_filter.rs:207`
```rust
pub fn apply_temporal_validity_filter_at( results: Vec<contextra_rank::SearchResult>, as_of_timestamp: u64, ) -> Vec<contextra_rank::SearchResult>
```
Dokumentation: Applies historical "as-of" bi-temporal validity filtering to candidates post-RRF fusion. Evaluates candidate validity at historical point-in-time `as_of_timestamp` (analogous to Graphitis Episode-Pinning). A candidate is retained ONLY IF `as_of_timestamp` lies within `[valid_from, valid_until)` AND transaction validity (`tx_valid_to`, ADR-033) is active at that historical timestamp. Candidates without validity window metadata (legacy data) are retained (Fail-Open for backward compatibility).

#### `fn apply_temporal_validity_filter_with_clock`
- Ort: `crates/contextra-db/src/temporal_filter.rs:173`
```rust
pub fn apply_temporal_validity_filter_with_clock( results: Vec<contextra_rank::SearchResult>, current_tx: TxId, clock: &dyn contextra_ports::Clock, ) -> Vec<contextra_rank::SearchResult>
```
Dokumentation: Applies bi-temporal validity filtering using an injected `Clock` port trait (B11).

#### `fn extract_validity_window`
- Ort: `crates/contextra-db/src/temporal_filter.rs:120`
```rust
pub fn extract_validity_window(metadata: Option<&Value>) -> ValidityWindow
```
Dokumentation: Extracts bi-temporal validity window metadata from candidate document metadata.

#### `fn sort_candidates_deterministic`
- Ort: `crates/contextra-db/src/temporal_filter.rs:163`
```rust
pub fn sort_candidates_deterministic(results: &mut [contextra_rank::SearchResult])
```
Dokumentation: Sorts candidates deterministically by score descending, breaking ties by document ID ascending (B11).

#### `type FusionResult`
- Ort: `crates/contextra-db/src/temporal_filter.rs:22`
```rust
pub type FusionResult = contextra_rank::SearchResult
```
Dokumentation: Type alias for search & fusion results in post-retrieval pipelines.

### Modul `transaction::cleanup`

#### `fn cleanup_orphaned_consolidation_intents`
- Ort: `crates/contextra-db/src/transaction/cleanup.rs:14`
```rust
pub async fn cleanup_orphaned_consolidation_intents<S: StorageEngine>( storage: &S, next_tx: &std::sync::atomic::AtomicU64, ) -> Result<usize>
```
Dokumentation: Aufgerufen beim Öffnen einer Collection / DB. Findet und entfernt verwaiste ConsolidationIntents aus dem WAL/Storage.

#### `fn recover_pending_intents`
- Ort: `crates/contextra-db/src/transaction/cleanup.rs:52`
```rust
pub async fn recover_pending_intents<S: StorageEngine>( storage: &S, next_tx: &std::sync::atomic::AtomicU64, ) -> Result<usize>
```
Dokumentation: Recovers or reconciles any pending transaction intents found in storage upon engine startup. For each pending intent: - If storage records exist for the transaction's doc_ids, the storage commit succeeded. Recovery preserves the committed transaction and updates the intent marker to `Committed`. - If storage records do NOT exist, storage commit never completed. Recovery compensates any leftover index entries via compensating actions and updates the intent to `Aborted`.

### Modul `transaction::compensating_actions`

#### `struct CommitLedger`
- Ort: `crates/contextra-db/src/transaction/compensating_actions.rs:21`
```rust
pub struct CommitLedger
```
Dokumentation: A ledger of compensating actions collected during 2PC, executed in reverse order (LIFO) on failure.
- Impl-Block `crates/contextra-db/src/transaction/compensating_actions.rs:25`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new() -> Self` | `crates/contextra-db/src/transaction/compensating_actions.rs:28` | Initialisiert eine leere Aktionsliste. |
| `pub fn push<A: CompensatingAction + 'static>(&mut self, action: A)` | `crates/contextra-db/src/transaction/compensating_actions.rs:36` | Dokumentation für `push` im Rahmen der Transaktions- und Facade-Orchestrierung. |
| `pub async fn execute_rollback(&mut self)` | `crates/contextra-db/src/transaction/compensating_actions.rs:42` | Dokumentation für `execute_rollback` im Rahmen der Transaktions- und Facade-Orchestrierung. |

#### `struct CompensateGraphAction`
- Ort: `crates/contextra-db/src/transaction/compensating_actions.rs:197`
```rust
pub struct CompensateGraphAction<S: StorageEngine, V: VectorIndex>
```
Dokumentation: Dokumentation für `CompensateGraphAction` im Rahmen der Transaktions- und Facade-Orchestrierung.
- Impl-Block `crates/contextra-db/src/transaction/compensating_actions.rs:202`
- Impl-Block (Trait `CompensatingAction`) `crates/contextra-db/src/transaction/compensating_actions.rs:214`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `doc_ids` | `Arc<Vec<DocId>>` | `crates/contextra-db/src/transaction/compensating_actions.rs:199` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(collection: Collection<S, V>, doc_ids: Arc<Vec<DocId>>) -> Self` | `crates/contextra-db/src/transaction/compensating_actions.rs:206` | Dokumentation für `new` im Rahmen der Transaktions- und Facade-Orchestrierung. |

#### `struct CompensateHnswAction`
- Ort: `crates/contextra-db/src/transaction/compensating_actions.rs:241`
```rust
pub struct CompensateHnswAction<S: StorageEngine, V: VectorIndex>
```
Dokumentation: Dokumentation für `CompensateHnswAction` im Rahmen der Transaktions- und Facade-Orchestrierung.
- Impl-Block `crates/contextra-db/src/transaction/compensating_actions.rs:247`
- Impl-Block (Trait `CompensatingAction`) `crates/contextra-db/src/transaction/compensating_actions.rs:263`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `tenant_id` | `TenantId` | `crates/contextra-db/src/transaction/compensating_actions.rs:243` |
| `doc_ids` | `Arc<Vec<DocId>>` | `crates/contextra-db/src/transaction/compensating_actions.rs:244` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new( collection: Collection<S, V>, tenant_id: TenantId, doc_ids: Arc<Vec<DocId>>, ) -> Self` | `crates/contextra-db/src/transaction/compensating_actions.rs:250` | Dokumentation für `new` im Rahmen der Transaktions- und Facade-Orchestrierung. |

#### `struct CompensateLsmAction`
- Ort: `crates/contextra-db/src/transaction/compensating_actions.rs:53`
```rust
pub struct CompensateLsmAction<S: StorageEngine, V: VectorIndex>
```
Dokumentation: Dokumentation für `CompensateLsmAction` im Rahmen der Transaktions- und Facade-Orchestrierung.
- Impl-Block `crates/contextra-db/src/transaction/compensating_actions.rs:61`
- Impl-Block (Trait `CompensatingAction`) `crates/contextra-db/src/transaction/compensating_actions.rs:81`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `intent_key` | `Vec<u8>` | `crates/contextra-db/src/transaction/compensating_actions.rs:55` |
| `doc_ids` | `Arc<Vec<DocId>>` | `crates/contextra-db/src/transaction/compensating_actions.rs:56` |
| `f_keys` | `Vec<StagedKeyOp>` | `crates/contextra-db/src/transaction/compensating_actions.rs:57` |
| `r_keys` | `Vec<StagedKeyOp>` | `crates/contextra-db/src/transaction/compensating_actions.rs:58` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new( collection: Collection<S, V>, intent_key: Vec<u8>, doc_ids: Arc<Vec<DocId>>, f_keys: Vec<StagedKeyOp>, r_keys: Vec<StagedKeyOp>, ) -> Self` | `crates/contextra-db/src/transaction/compensating_actions.rs:64` | Dokumentation für `new` im Rahmen der Transaktions- und Facade-Orchestrierung. |

#### `struct CompensateTextAction`
- Ort: `crates/contextra-db/src/transaction/compensating_actions.rs:322`
```rust
pub struct CompensateTextAction<S: StorageEngine, V: VectorIndex>
```
Dokumentation: Dokumentation für `CompensateTextAction` im Rahmen der Transaktions- und Facade-Orchestrierung.
- Impl-Block `crates/contextra-db/src/transaction/compensating_actions.rs:327`
- Impl-Block (Trait `CompensatingAction`) `crates/contextra-db/src/transaction/compensating_actions.rs:338`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `doc_ids` | `Arc<Vec<DocId>>` | `crates/contextra-db/src/transaction/compensating_actions.rs:324` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(collection: Collection<S, V>, doc_ids: Arc<Vec<DocId>>) -> Self` | `crates/contextra-db/src/transaction/compensating_actions.rs:330` | Dokumentation für `new` im Rahmen der Transaktions- und Facade-Orchestrierung. |

#### `struct RollbackStagedAction`
- Ort: `crates/contextra-db/src/transaction/compensating_actions.rs:371`
```rust
pub struct RollbackStagedAction<S: StorageEngine, V: VectorIndex>
```
Dokumentation: Dokumentation für `RollbackStagedAction` im Rahmen der Transaktions- und Facade-Orchestrierung.
- Impl-Block `crates/contextra-db/src/transaction/compensating_actions.rs:376`
- Impl-Block (Trait `CompensatingAction`) `crates/contextra-db/src/transaction/compensating_actions.rs:384`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `tx_id` | `TxId` | `crates/contextra-db/src/transaction/compensating_actions.rs:373` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(collection: Collection<S, V>, tx_id: TxId) -> Self` | `crates/contextra-db/src/transaction/compensating_actions.rs:379` | Dokumentation für `new` im Rahmen der Transaktions- und Facade-Orchestrierung. |

#### `trait CompensatingAction`
- Ort: `crates/contextra-db/src/transaction/compensating_actions.rs:15`
```rust
pub trait CompensatingAction: Send + Sync
```
Dokumentation: Trait representing an undo operation to be executed during transaction rollback/compensation.

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn execute<'a>(&'a self) -> BoxFuture<'a, Result<()>>` | `crates/contextra-db/src/transaction/compensating_actions.rs:16` |  |

### Modul `transaction::db_transaction`

#### `struct DbTransaction`
- Ort: `crates/contextra-db/src/transaction/db_transaction.rs:22`
```rust
pub struct DbTransaction<S: StorageEngine, V: VectorIndex = contextra_vector::HnswIndex>
```
Dokumentation: A transaction wrapper that ensures atomic multi-index commits across LSM-Store, HNSW-Index, Text-Index, and Graph-Index.
- Impl-Block `crates/contextra-db/src/transaction/db_transaction.rs:38`
- Impl-Block (Trait `Drop`) `crates/contextra-db/src/transaction/db_transaction.rs:163`
- Impl-Block `crates/contextra-db/src/transaction/db_transaction_commit.rs:20`
- Impl-Block `crates/contextra-db/src/transaction/db_transaction_rollback.rs:15`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `tx_id` | `TxId` | `crates/contextra-db/src/transaction/db_transaction.rs:23` |
| `staged_forward_keys` | `Mutex<Vec<StagedKeyOp>>` | `crates/contextra-db/src/transaction/db_transaction.rs:25` |
| `staged_reverse_keys` | `Mutex<Vec<StagedKeyOp>>` | `crates/contextra-db/src/transaction/db_transaction.rs:26` |
| `staged_doc_ids` | `Mutex<Arc<Vec<DocId>>>` | `crates/contextra-db/src/transaction/db_transaction.rs:27` |
| `staged_text_deletes` | `Mutex<Vec<DocId>>` | `crates/contextra-db/src/transaction/db_transaction.rs:29` |
| `staged_graph_entities` | `Mutex<Vec<Entity>>` | `crates/contextra-db/src/transaction/db_transaction.rs:30` |
| `staged_graph_edges` | `Mutex<Vec<Edge>>` | `crates/contextra-db/src/transaction/db_transaction.rs:31` |
| `staged_graph_entity_deletes` | `Mutex<Vec<EntityId>>` | `crates/contextra-db/src/transaction/db_transaction.rs:32` |
| `staged_hyperedges` | `Mutex<Vec<contextra_graph::hyperedge::HyperEdge>>` | `crates/contextra-db/src/transaction/db_transaction.rs:34` |
| `committed` | `AtomicBool` | `crates/contextra-db/src/transaction/db_transaction.rs:35` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(collection: Collection<S, V>, tx_id: TxId) -> Self` | `crates/contextra-db/src/transaction/db_transaction.rs:41` | Dokumentation für `new` im Rahmen der Transaktions- und Facade-Orchestrierung. |
| `pub fn record_keys(&self, forward: Vec<u8>, reverse: Vec<u8>, doc_id: DocId)` | `crates/contextra-db/src/transaction/db_transaction.rs:60` | Records keys and IDs that have been operated on for potential compensating rollback. |
| `pub fn record_keys_with_old_values( &self, forward: Vec<u8>, old_forward: Option<Bytes>, reverse: Vec<u8>, old_reverse: Option<Bytes>, doc_id: DocId, )` | `crates/contextra-db/src/transaction/db_transaction.rs:65` | Records keys and IDs along with pre-write values for potential compensating rollback. |
| `pub fn stage_text_insert(&self, doc_id: DocId, text: String)` | `crates/contextra-db/src/transaction/db_transaction.rs:94` | Dokumentation für `stage_text_insert` im Rahmen der Transaktions- und Facade-Orchestrierung. |
| `pub fn stage_text_delete(&self, doc_id: DocId)` | `crates/contextra-db/src/transaction/db_transaction.rs:104` | Dokumentation für `stage_text_delete` im Rahmen der Transaktions- und Facade-Orchestrierung. |
| `pub fn stage_graph_entity(&self, entity: Entity)` | `crates/contextra-db/src/transaction/db_transaction.rs:114` | Dokumentation für `stage_graph_entity` im Rahmen der Transaktions- und Facade-Orchestrierung. |
| `pub fn stage_graph_edge(&self, edge: Edge)` | `crates/contextra-db/src/transaction/db_transaction.rs:124` | Dokumentation für `stage_graph_edge` im Rahmen der Transaktions- und Facade-Orchestrierung. |
| `pub fn stage_graph_entity_delete(&self, entity_id: EntityId)` | `crates/contextra-db/src/transaction/db_transaction.rs:134` | Dokumentation für `stage_graph_entity_delete` im Rahmen der Transaktions- und Facade-Orchestrierung. |
| `pub fn stage_graph_edge_delete(&self, from: EntityId, to: EntityId)` | `crates/contextra-db/src/transaction/db_transaction.rs:144` | Dokumentation für `stage_graph_edge_delete` im Rahmen der Transaktions- und Facade-Orchestrierung. |
| `pub fn stage_hyperedge(&self, hyperedge: contextra_graph::hyperedge::HyperEdge)` | `crates/contextra-db/src/transaction/db_transaction.rs:154` | Dokumentation für `stage_hyperedge` im Rahmen der Transaktions- und Facade-Orchestrierung. |
| `pub async fn commit(self) -> Result<()>` | `crates/contextra-db/src/transaction/db_transaction_commit.rs:131` | Commits the transaction atomically across all 4 indices (LSM, HNSW, BM25, CSR). |
| `pub async fn rollback(self) -> Result<()>` | `crates/contextra-db/src/transaction/db_transaction_rollback.rs:199` | Rolls back any uncommitted changes applied to all 4 sub-systems in reverse commit order. |

### Modul `transaction::intent`

#### `enum CommitIntent`
- Ort: `crates/contextra-db/src/transaction/intent.rs:18`
```rust
pub enum CommitIntent
```
Dokumentation: Status of a multi-index transaction during the 2-phase commit.

Varianten:

- `Pending{` — `crates/contextra-db/src/transaction/intent.rs:24`
- `Committed` — `crates/contextra-db/src/transaction/intent.rs:34`
- `Aborted` — `crates/contextra-db/src/transaction/intent.rs:36`
- `Consolidation{` — `crates/contextra-db/src/transaction/intent.rs:38`
- `Failed{ reason: String }` — `crates/contextra-db/src/transaction/intent.rs:44`

## 9. Impl-Blöcke ohne Typ im selben Crate

- `impl OrphanCleanupIndex for contextra_vector::hnsw::HnswIndex` — `crates/contextra-db/src/background_workers/config.rs:59` (0 pub Methoden)
- `impl From<SignalWeights> for FusionWeights` — `crates/contextra-db/src/collection/query_builder/weights.rs:33` (0 pub Methoden)
- `impl TryFrom<MetadataFilter> for FilterExpr` — `crates/contextra-db/src/filter.rs:189` (0 pub Methoden)
