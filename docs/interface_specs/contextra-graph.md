# Schnittstellenspezifikation: `contextra-graph`

> Erzeugt durch `tools/extract_interfaces.py` aus dem Quellcode. Keine Handpflege. Jede Angabe trägt Datei:Zeile. Maßgeblich bleibt der Quellcode.

## 1. Kopf

| Feld | Wert |
|---|---|
| Name | `contextra-graph` |
| Version | workspace |
| Pfad | `crates/contextra-graph` |
| Rolle | Wissensgraph |
| Bibliothekspfad | src/lib.rs (Standard) |

**Crate-Dokumentation (//!):**

> Contextra Graph — CSR-Graph for Entity-Relation Traversal & Session DAG.
>
> This crate provides the graph signal (Signal 3) for the 5-Signal Fusion
> architecture (optional: Edge-Reinforcement-Signal hinter `edge-reinforcement-learning`-Flag).
> It implements a Compressed Sparse Row (CSR) graph for
> memory-efficient BFS traversal with score-decay, and a Session-DAG for
> agent state branching (Contextra Session-DAG Pattern).
>
> # Architecture Role (Triebwerk — Layer 1)
>
> Peer to `contextra-store` and `contextra-index`. Provides the `GraphIndex`
> trait implementation via [`CsrGraph`] and conversation branching via [`SessionBranchTree`].

## 2. Abhängigkeiten

| Name | Art | Quelle | optional | verwendet von (normal/dev) |
|---|---|---|---|---|
| `ahash` | normal | workspace | nein |  |
| `arc-swap` | normal | registry | nein |  |
| `bincode` | normal | workspace | nein |  |
| `blake3` | normal | workspace | nein |  |
| `bytes` | normal | workspace | nein |  |
| `contextra-env` | normal | workspace | nein | contextra-checkpoint, contextra-crypto, contextra-graph, contextra-infer-candle, contextra-kvcache, contextra-mvcc, contextra-ports, contextra-store, contextra-text, contextra-vector |
| `contextra-graph` | dev | path | nein | contextra-agent, contextra-cognition, contextra-db, contextra-engine, contextra-graph, contextra-graph-fuzz |
| `contextra-ids` | normal | workspace | nein | contextra-checkpoint, contextra-crypto, contextra-graph, contextra-mvcc, contextra-store, contextra-text, contextra-types, contextra-vector |
| `contextra-ports` | normal | workspace | ja | contextra, contextra-adapt, contextra-agent, contextra-checkpoint, contextra-cognition, contextra-crypto, contextra-crypto-fuzz, contextra-db, contextra-engine, contextra-graph, contextra-graph-fuzz, contextra-infer-candle, contextra-infer-ollama, contextra-infer-onnx, contextra-kvcache, contextra-mcp, contextra-mvcc, contextra-rank, contextra-router, contextra-store, contextra-store-fuzz, contextra-testkit, contextra-text, contextra-vector, contextra-vector-fuzz |
| `contextra-types` | normal | workspace | ja | contextra, contextra-adapt, contextra-agent, contextra-bench, contextra-checkpoint, contextra-cognition, contextra-crypto, contextra-crypto-fuzz, contextra-db, contextra-engine, contextra-graph, contextra-graph-fuzz, contextra-infer-candle, contextra-infer-ollama, contextra-infer-onnx, contextra-ingest, contextra-kvcache, contextra-mcp, contextra-mvcc, contextra-ports, contextra-py, contextra-rank, contextra-router, contextra-simd, contextra-store, contextra-store-fuzz, contextra-testkit, contextra-text, contextra-vault, contextra-vector, contextra-vector-fuzz |
| `criterion` | dev | workspace | nein |  |
| `futures` | dev | registry | nein |  |
| `parking_lot` | normal | workspace | nein |  |
| `proptest` | dev | registry | nein |  |
| `scc` | normal | registry | nein |  |
| `serde` | normal | workspace | nein |  |
| `serde_json` | normal | workspace | nein |  |
| `tempfile` | dev | workspace | nein |  |
| `thiserror` | normal | workspace | nein |  |
| `tokio` | dev | workspace | nein |  |
| `tracing` | normal | workspace | nein |  |
| `tracing-subscriber` | dev | workspace | nein |  |

**Verwendet von (Crate-Ebene):** `contextra-agent`, `contextra-cognition`, `contextra-db`, `contextra-engine`, `contextra-graph`, `contextra-graph-fuzz`

## 3. Features

| Feature | Aktiviert |
|---|---|
| `apprh-diffusion` | – |
| `default` | `session-dag`, `entity-extraction`, `hyperedge-suggest`, `experimental-graph` |
| `docid-128` | – |
| `edge-reinforcement-learning` | – |
| `entity-extraction` | – |
| `experimental-graph` | `session-dag` |
| `graph-connectivity-health` | – |
| `hyperedge-suggest` | – |
| `k-path-diffusion` | – |
| `physio-percolation` | `graph-connectivity-health` |
| `ports-compat` | `dep:contextra-types`, `dep:contextra-ports` |
| `ppr-forward-push` | – |
| `session-dag` | – |
| `test-support` | – |
| `types-compat` | `dep:contextra-types`, `dep:contextra-ports` |
| `workspace` | `dep:contextra-types`, `dep:contextra-ports`, `types-compat`, `ports-compat` |

## 4. Metriken (Quellcode)

| Kennzahl | Wert |
|---|---|
| Quelldateien | 90 |
| Quellzeilen | 23595 |
| Öffentliche Items (Modul-Ebene) | 242 |
| davon `fn` | 40 |
| davon `struct` | 87 |
| davon `enum` | 9 |
| davon `trait` | 13 |
| davon `type` | 6 |
| davon `const` | 17 |
| davon `mod` | 70 |
| Öffentliche Makros (`#[macro_export]`) | 0 |
| Öffentliche Re-Exports (`pub use`) | 60 |
| `unsafe`-Vorkommen (Produktionscode) | 0 |
| `.unwrap()` (Produktionscode) | 575 |
| `.expect(` (Produktionscode) | 28 |
| `panic!` (Produktionscode) | 0 |
| `todo!/unimplemented!/unreachable!` | 0 |
| Indexierungen `x[…]` (grob, Produktionscode) | 221 |
| `#[deprecated]` | 2 |
| `#[cfg(feature…)]`-Gates | 85 |
| Parser-Abdeckung (geparste `pub`-Deklarationen / Grep-Treffer) | 484 / 484 = 100 % |

## 5. Modulbaum und Dateien

| Modul | Datei | Zeilen |
|---|---|---|
| `apprh::diffusion` | `crates/contextra-graph/src/apprh/diffusion.rs` | 187 |
| `apprh::error` | `crates/contextra-graph/src/apprh/error.rs` | 31 |
| `apprh::gate_monitor` | `crates/contextra-graph/src/apprh/gate_monitor.rs` | 281 |
| `apprh` | `crates/contextra-graph/src/apprh/mod.rs` | 161 |
| `apprh::params` | `crates/contextra-graph/src/apprh/params.rs` | 127 |
| `apprh::selector` | `crates/contextra-graph/src/apprh/selector.rs` | 226 |
| `apprh::shadow` | `crates/contextra-graph/src/apprh/shadow.rs` | 152 |
| `arc_slice` | `crates/contextra-graph/src/arc_slice.rs` | 179 |
| `cascade` | `crates/contextra-graph/src/cascade.rs` | 1100 |
| `community::tests` | `crates/contextra-graph/src/community/tests.rs` | 839 |
| `community` | `crates/contextra-graph/src/community.rs` | 642 |
| `compat::algo_error_bridge` | `crates/contextra-graph/src/compat/algo_error_bridge.rs` | 19 |
| `compat::collection_mutation` | `crates/contextra-graph/src/compat/collection_mutation.rs` | 79 |
| `compat::error` | `crates/contextra-graph/src/compat/error.rs` | 54 |
| `compat::graph_index::inherent_shims` | `crates/contextra-graph/src/compat/graph_index/inherent_shims.rs` | 26 |
| `compat::graph_index` | `crates/contextra-graph/src/compat/graph_index/mod.rs` | 309 |
| `compat::graph_index::mutations` | `crates/contextra-graph/src/compat/graph_index/mutations.rs` | 332 |
| `compat::graph_index::traversal` | `crates/contextra-graph/src/compat/graph_index/traversal.rs` | 301 |
| `compat::graph_index_ext` | `crates/contextra-graph/src/compat/graph_index_ext.rs` | 10 |
| `compat::llm_bridge` | `crates/contextra-graph/src/compat/llm_bridge.rs` | 1 |
| `compat::provider` | `crates/contextra-graph/src/compat/provider.rs` | 309 |
| `compat::purger` | `crates/contextra-graph/src/compat/purger.rs` | 138 |
| `compat::replay_hook` | `crates/contextra-graph/src/compat/replay_hook.rs` | 1 |
| `compat::storage_bridge` | `crates/contextra-graph/src/compat/storage_bridge.rs` | 89 |
| `compat` | `crates/contextra-graph/src/compat.rs` | 22 |
| `consistency_enforcement` | `crates/contextra-graph/src/consistency_enforcement.rs` | 494 |
| `csr::graph_index` | `crates/contextra-graph/src/csr/graph_index.rs` | 17 |
| `csr::graph_persist` | `crates/contextra-graph/src/csr/graph_persist.rs` | 283 |
| `csr::graph_read::communities` | `crates/contextra-graph/src/csr/graph_read/communities.rs` | 88 |
| `csr::graph_read` | `crates/contextra-graph/src/csr/graph_read/mod.rs` | 183 |
| `csr::graph_read::pagerank` | `crates/contextra-graph/src/csr/graph_read/pagerank.rs` | 147 |
| `csr::graph_read::path_rag` | `crates/contextra-graph/src/csr/graph_read/path_rag.rs` | 83 |
| `csr::graph_write::edge_mutation` | `crates/contextra-graph/src/csr/graph_write/edge_mutation.rs` | 227 |
| `csr::graph_write::entity` | `crates/contextra-graph/src/csr/graph_write/entity.rs` | 44 |
| `csr::graph_write::hyperedge` | `crates/contextra-graph/src/csr/graph_write/hyperedge.rs` | 401 |
| `csr::graph_write` | `crates/contextra-graph/src/csr/graph_write/mod.rs` | 208 |
| `csr::graph_write::tombstone` | `crates/contextra-graph/src/csr/graph_write/tombstone.rs` | 134 |
| `csr::inner::compaction` | `crates/contextra-graph/src/csr/inner/compaction.rs` | 259 |
| `csr::inner::estimate` | `crates/contextra-graph/src/csr/inner/estimate.rs` | 269 |
| `csr::inner` | `crates/contextra-graph/src/csr/inner/mod.rs` | 185 |
| `csr::inner::reinforcement` | `crates/contextra-graph/src/csr/inner/reinforcement.rs` | 169 |
| `csr::path_graph` | `crates/contextra-graph/src/csr/path_graph.rs` | 212 |
| `csr::tests::basic_tests` | `crates/contextra-graph/src/csr/tests/basic_tests.rs` | 794 |
| `csr::tests::bitemporal_tests` | `crates/contextra-graph/src/csr/tests/bitemporal_tests.rs` | 581 |
| `csr::tests::graph_index_tests` | `crates/contextra-graph/src/csr/tests/graph_index_tests.rs` | 698 |
| `csr::tests::mutation_sequence_tests` | `crates/contextra-graph/src/csr/tests/mutation_sequence_tests.rs` | 122 |
| `csr::tests::persistence_tests` | `crates/contextra-graph/src/csr/tests/persistence_tests.rs` | 832 |
| `csr::tests` | `crates/contextra-graph/src/csr/tests.rs` | 22 |
| `csr::types` | `crates/contextra-graph/src/csr/types.rs` | 136 |
| `csr::visibility` | `crates/contextra-graph/src/csr/visibility.rs` | 100 |
| `csr` | `crates/contextra-graph/src/csr.rs` | 36 |
| `edge_reinforcement` | `crates/contextra-graph/src/edge_reinforcement.rs` | 181 |
| `edge_reinforcement_buffer` | `crates/contextra-graph/src/edge_reinforcement_buffer.rs` | 223 |
| `entity_extraction` | `crates/contextra-graph/src/entity_extraction.rs` | 323 |
| `error` | `crates/contextra-graph/src/error.rs` | 118 |
| `graph_view` | `crates/contextra-graph/src/graph_view.rs` | 34 |
| `hyperedge` | `crates/contextra-graph/src/hyperedge.rs` | 807 |
| `hyperedge_suggest` | `crates/contextra-graph/src/hyperedge_suggest.rs` | 197 |
| `crate` | `crates/contextra-graph/src/lib.rs` | 159 |
| `path_rag::k_path` | `crates/contextra-graph/src/path_rag/k_path.rs` | 336 |
| `path_rag` | `crates/contextra-graph/src/path_rag/mod.rs` | 484 |
| `path_rag::snapshot` | `crates/contextra-graph/src/path_rag/snapshot.rs` | 227 |
| `path_rag::tests` | `crates/contextra-graph/src/path_rag/tests.rs` | 587 |
| `percolation` | `crates/contextra-graph/src/percolation.rs` | 343 |
| `ppr::cost` | `crates/contextra-graph/src/ppr/cost.rs` | 273 |
| `ppr::dense` | `crates/contextra-graph/src/ppr/dense.rs` | 411 |
| `ppr::forward_push` | `crates/contextra-graph/src/ppr/forward_push.rs` | 278 |
| `ppr` | `crates/contextra-graph/src/ppr/mod.rs` | 385 |
| `ppr::shadow_hook` | `crates/contextra-graph/src/ppr/shadow_hook.rs` | 122 |
| `ppr::snapshot` | `crates/contextra-graph/src/ppr/snapshot.rs` | 147 |
| `ppr::tests::algo_tests` | `crates/contextra-graph/src/ppr/tests/algo_tests.rs` | 1205 |
| `ppr::tests::context_tests` | `crates/contextra-graph/src/ppr/tests/context_tests.rs` | 858 |
| `ppr::tests::golden_tests` | `crates/contextra-graph/src/ppr/tests/golden_tests.rs` | 364 |
| `ppr::tests` | `crates/contextra-graph/src/ppr/tests.rs` | 20 |
| `ppr::types` | `crates/contextra-graph/src/ppr/types.rs` | 89 |
| `ppr_stream` | `crates/contextra-graph/src/ppr_stream.rs` | 172 |
| `provenance` | `crates/contextra-graph/src/provenance.rs` | 198 |
| `provider` | `crates/contextra-graph/src/provider.rs` | 15 |
| `purger` | `crates/contextra-graph/src/purger.rs` | 15 |
| `session_dag` | `crates/contextra-graph/src/session_dag.rs` | 1016 |
| `storage` | `crates/contextra-graph/src/storage.rs` | 129 |
| `test_doubles` | `crates/contextra-graph/src/test_doubles.rs` | 215 |
| `text_gen` | `crates/contextra-graph/src/text_gen.rs` | 8 |
| `tl_hfd::diffusion` | `crates/contextra-graph/src/tl_hfd/diffusion.rs` | 256 |
| `tl_hfd::error` | `crates/contextra-graph/src/tl_hfd/error.rs` | 27 |
| `tl_hfd::lovasz` | `crates/contextra-graph/src/tl_hfd/lovasz.rs` | 210 |
| `tl_hfd` | `crates/contextra-graph/src/tl_hfd/mod.rs` | 109 |
| `tl_hfd::params` | `crates/contextra-graph/src/tl_hfd/params.rs` | 150 |
| `tl_hfd::shadow` | `crates/contextra-graph/src/tl_hfd/shadow.rs` | 170 |
| `types` | `crates/contextra-graph/src/types.rs` | 395 |

## 6. Re-Exports (`pub use`)

- `pub use diffusion::run_apprh_push` — `crates/contextra-graph/src/apprh/mod.rs:19` (Modul `apprh`)
- `pub use error::ApprhError` — `crates/contextra-graph/src/apprh/mod.rs:20` (Modul `apprh`)
- `pub use gate_monitor::` — `crates/contextra-graph/src/apprh/mod.rs:21` (Modul `apprh`)
- `pub use params::ApprhParams` — `crates/contextra-graph/src/apprh/mod.rs:24` (Modul `apprh`)
- `pub use selector::` — `crates/contextra-graph/src/apprh/mod.rs:25` (Modul `apprh`)
- `pub use shadow::` — `crates/contextra-graph/src/apprh/mod.rs:26` (Modul `apprh`)
- `pub use contextra_ports::GraphIndex` — `crates/contextra-graph/src/compat/graph_index/mod.rs:21` (Modul `compat::graph_index`)
- `pub use graph_write::CsrGraph` — `crates/contextra-graph/src/csr.rs:26` (Modul `csr`)
- `pub use inner::` — `crates/contextra-graph/src/csr.rs:27` (Modul `csr`)
- `pub use types::` — `crates/contextra-graph/src/csr.rs:28` (Modul `csr`)
- `pub use visibility::` — `crates/contextra-graph/src/csr.rs:32` (Modul `csr`)
- `pub use crate::compat::graph_index` — `crates/contextra-graph/src/csr/graph_index.rs:13` (Modul `csr::graph_index`)
- `pub use estimate::MemoryEstimate` — `crates/contextra-graph/src/csr/inner/mod.rs:29` (Modul `csr::inner`)
- `pub use crate::types::` — `crates/contextra-graph/src/hyperedge.rs:48` (Modul `hyperedge`)
- `pub use apprh::` — `crates/contextra-graph/src/lib.rs:79` (Modul `crate`)
- `pub use arc_slice::ArcSlice` — `crates/contextra-graph/src/lib.rs:84` (Modul `crate`)
- `pub use cascade::` — `crates/contextra-graph/src/lib.rs:85` (Modul `crate`)
- `pub use community::` — `crates/contextra-graph/src/lib.rs:90` (Modul `crate`)
- `pub use consistency_enforcement::` — `crates/contextra-graph/src/lib.rs:94` (Modul `crate`)
- `pub use csr::CsrGraph` — `crates/contextra-graph/src/lib.rs:98` (Modul `crate`)
- `pub use edge_reinforcement::` — `crates/contextra-graph/src/lib.rs:100` (Modul `crate`)
- `pub use edge_reinforcement_buffer::` — `crates/contextra-graph/src/lib.rs:105` (Modul `crate`)
- `pub use entity_extraction::` — `crates/contextra-graph/src/lib.rs:106` (Modul `crate`)
- `pub use error::GraphError as Error` — `crates/contextra-graph/src/lib.rs:109` (Modul `crate`)
- `pub use error::` — `crates/contextra-graph/src/lib.rs:110` (Modul `crate`)
- `pub use hyperedge::` — `crates/contextra-graph/src/lib.rs:111` (Modul `crate`)
- `pub use hyperedge_suggest::` — `crates/contextra-graph/src/lib.rs:115` (Modul `crate`)
- `pub use hyperedge_suggest::validate_candidates_with_llm` — `crates/contextra-graph/src/lib.rs:120` (Modul `crate`)
- `pub use path_rag::` — `crates/contextra-graph/src/lib.rs:121` (Modul `crate`)
- `pub use path_rag::` — `crates/contextra-graph/src/lib.rs:123` (Modul `crate`)
- `pub use percolation::` — `crates/contextra-graph/src/lib.rs:125` (Modul `crate`)
- `pub use ppr::compute_ppr_with_apprh_selector` — `crates/contextra-graph/src/lib.rs:130` (Modul `crate`)
- `pub use ppr::` — `crates/contextra-graph/src/lib.rs:131` (Modul `crate`)
- `pub use ppr_stream::` — `crates/contextra-graph/src/lib.rs:133` (Modul `crate`)
- `pub use provenance::` — `crates/contextra-graph/src/lib.rs:134` (Modul `crate`)
- `pub use provider::` — `crates/contextra-graph/src/lib.rs:136` (Modul `crate`)
- `pub use purger::` — `crates/contextra-graph/src/lib.rs:140` (Modul `crate`)
- `pub use session_dag::` — `crates/contextra-graph/src/lib.rs:142` (Modul `crate`)
- `pub use tl_hfd::` — `crates/contextra-graph/src/lib.rs:145` (Modul `crate`)
- `pub use compat::graph_index_ext::GraphIndexExt` — `crates/contextra-graph/src/lib.rs:152` (Modul `crate`)
- `pub use storage::` — `crates/contextra-graph/src/lib.rs:156` (Modul `crate`)
- `pub use graph_view::GraphView` — `crates/contextra-graph/src/lib.rs:157` (Modul `crate`)
- `pub use text_gen::TextGenerator` — `crates/contextra-graph/src/lib.rs:158` (Modul `crate`)
- `pub use types::` — `crates/contextra-graph/src/lib.rs:159` (Modul `crate`)
- `pub use k_path::` — `crates/contextra-graph/src/path_rag/mod.rs:26` (Modul `path_rag`)
- `pub use snapshot::SnapshotPathGraph` — `crates/contextra-graph/src/path_rag/mod.rs:29` (Modul `path_rag`)
- `pub use crate::hyperedge::` — `crates/contextra-graph/src/path_rag/mod.rs:31` (Modul `path_rag`)
- `pub use crate::types::EntityId` — `crates/contextra-graph/src/path_rag/mod.rs:34` (Modul `path_rag`)
- `pub use self::snapshot::compute_ppr_at` — `crates/contextra-graph/src/ppr/mod.rs:25` (Modul `ppr`)
- `pub use self::types::` — `crates/contextra-graph/src/ppr/mod.rs:26` (Modul `ppr`)
- `pub use crate::compat::provider::*` — `crates/contextra-graph/src/provider.rs:11` (Modul `provider`)
- `pub use crate::compat::purger::*` — `crates/contextra-graph/src/purger.rs:11` (Modul `purger`)
- `pub use diffusion::run_diffusion` — `crates/contextra-graph/src/tl_hfd/mod.rs:18` (Modul `tl_hfd`)
- `pub use error::TlHfdError` — `crates/contextra-graph/src/tl_hfd/mod.rs:19` (Modul `tl_hfd`)
- `pub use lovasz::` — `crates/contextra-graph/src/tl_hfd/mod.rs:20` (Modul `tl_hfd`)
- `pub use params::TlHfdParams` — `crates/contextra-graph/src/tl_hfd/mod.rs:21` (Modul `tl_hfd`)
- `pub use shadow::` — `crates/contextra-graph/src/tl_hfd/mod.rs:22` (Modul `tl_hfd`)
- `pub use contextra_ports::graph::` — `crates/contextra-graph/src/types.rs:387` (Modul `types`)
- `pub use contextra_types::` — `crates/contextra-graph/src/types.rs:389` (Modul `types`)
- `pub use local::*` — `crates/contextra-graph/src/types.rs:395` (Modul `types`)

## 8. Öffentliche Schnittstelle je Item

### Modul `apprh`

#### `mod diffusion`
- Ort: `crates/contextra-graph/src/apprh/mod.rs:12`
```rust
pub mod diffusion
```

#### `mod error`
- Ort: `crates/contextra-graph/src/apprh/mod.rs:13`
```rust
pub mod error
```

#### `mod gate_monitor`
- Ort: `crates/contextra-graph/src/apprh/mod.rs:14`
```rust
pub mod gate_monitor
```

#### `mod params`
- Ort: `crates/contextra-graph/src/apprh/mod.rs:15`
```rust
pub mod params
```

#### `mod selector`
- Ort: `crates/contextra-graph/src/apprh/mod.rs:16`
```rust
pub mod selector
```

#### `mod shadow`
- Ort: `crates/contextra-graph/src/apprh/mod.rs:17`
```rust
pub mod shadow
```

#### `fn apprh_local`
- Ort: `crates/contextra-graph/src/apprh/mod.rs:45`
```rust
pub fn apprh_local<G: PathGraph>( graph: &G, seeds: &[EntityId], params: &ApprhParams, ) -> Result<AHashMap<EntityId, f32>, ApprhError>
```

#### `fn forward_push_apprh`
- Ort: `crates/contextra-graph/src/apprh/mod.rs:61`
```rust
pub fn forward_push_apprh<G: PathGraph>( graph: &G, seeds: &[EntityId], params: &PprParams, hyperedge_decay_factor: f32, ) -> GraphResult<AHashMap<EntityId, f32>>
```

### Modul `apprh::diffusion`

#### `fn run_apprh_push`
- Ort: `crates/contextra-graph/src/apprh/diffusion.rs:32`
```rust
pub fn run_apprh_push<G: PathGraph>( graph: &G, seeds: &[EntityId], params: &ApprhParams, ) -> Result<AHashMap<EntityId, f32>, ApprhError>
```

### Modul `apprh::error`

#### `enum ApprhError`
- Ort: `crates/contextra-graph/src/apprh/error.rs:17`
```rust
pub enum ApprhError
```

Varianten:

- `InvalidParameter(String)` — `crates/contextra-graph/src/apprh/error.rs:20`
- `EmptySeeds` — `crates/contextra-graph/src/apprh/error.rs:24`
- `NonFinite` — `crates/contextra-graph/src/apprh/error.rs:28`

### Modul `apprh::gate_monitor`

#### `struct ApprhGateMonitor`
- Ort: `crates/contextra-graph/src/apprh/gate_monitor.rs:112`
```rust
pub struct ApprhGateMonitor
```
- Impl-Block `crates/contextra-graph/src/apprh/gate_monitor.rs:118`
- Impl-Block (Trait `Default`) `crates/contextra-graph/src/apprh/gate_monitor.rs:249`
- Impl-Block (Trait `std::fmt::Debug`) `crates/contextra-graph/src/apprh/gate_monitor.rs:265`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(config: ApprhGateMonitorConfig) -> Result<Self, ApprhError>` | `crates/contextra-graph/src/apprh/gate_monitor.rs:124` |  |
| `pub fn with_gate<G>(config: ApprhGateMonitorConfig, gate: G) -> Result<Self, ApprhError> where G: ApprhFlipGate + Send + Sync + 'static,` | `crates/contextra-graph/src/apprh/gate_monitor.rs:133` |  |
| `pub fn observe(&self, report: &ApprhShadowComparison)` | `crates/contextra-graph/src/apprh/gate_monitor.rs:155` |  |
| `pub fn production_ready(&self) -> bool` | `crates/contextra-graph/src/apprh/gate_monitor.rs:186` |  |
| `pub fn snapshot(&self) -> ApprhGateMonitorSnapshot` | `crates/contextra-graph/src/apprh/gate_monitor.rs:193` |  |
| `pub fn reset(&self)` | `crates/contextra-graph/src/apprh/gate_monitor.rs:235` |  |
| `pub fn config(&self) -> &ApprhGateMonitorConfig` | `crates/contextra-graph/src/apprh/gate_monitor.rs:244` |  |

#### `struct ApprhGateMonitorConfig`
- Ort: `crates/contextra-graph/src/apprh/gate_monitor.rs:22`
```rust
pub struct ApprhGateMonitorConfig
```
- Impl-Block (Trait `Default`) `crates/contextra-graph/src/apprh/gate_monitor.rs:29`
- Impl-Block `crates/contextra-graph/src/apprh/gate_monitor.rs:38`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `window_size` | `usize` | `crates/contextra-graph/src/apprh/gate_monitor.rs:24` |
| `required_consecutive_passes` | `usize` | `crates/contextra-graph/src/apprh/gate_monitor.rs:26` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn validate(&self) -> Result<(), ApprhError>` | `crates/contextra-graph/src/apprh/gate_monitor.rs:44` |  |

#### `struct ApprhGateMonitorSnapshot`
- Ort: `crates/contextra-graph/src/apprh/gate_monitor.rs:81`
```rust
pub struct ApprhGateMonitorSnapshot
```

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `total_observations` | `u64` | `crates/contextra-graph/src/apprh/gate_monitor.rs:83` |
| `current_streak` | `usize` | `crates/contextra-graph/src/apprh/gate_monitor.rs:85` |
| `last_overlap` | `Option<f32>` | `crates/contextra-graph/src/apprh/gate_monitor.rs:87` |
| `max_abs_diff_in_window` | `f32` | `crates/contextra-graph/src/apprh/gate_monitor.rs:89` |
| `min_overlap_in_window` | `f32` | `crates/contextra-graph/src/apprh/gate_monitor.rs:91` |
| `discrepancy_count_in_window` | `usize` | `crates/contextra-graph/src/apprh/gate_monitor.rs:93` |
| `production_ready` | `bool` | `crates/contextra-graph/src/apprh/gate_monitor.rs:95` |

#### `struct ApprhObservationRecord`
- Ort: `crates/contextra-graph/src/apprh/gate_monitor.rs:67`
```rust
pub struct ApprhObservationRecord
```

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `max_abs_diff` | `f32` | `crates/contextra-graph/src/apprh/gate_monitor.rs:69` |
| `top_k_overlap` | `f32` | `crates/contextra-graph/src/apprh/gate_monitor.rs:71` |
| `discrepancy` | `bool` | `crates/contextra-graph/src/apprh/gate_monitor.rs:73` |
| `passed` | `bool` | `crates/contextra-graph/src/apprh/gate_monitor.rs:75` |

### Modul `apprh::params`

#### `struct ApprhParams`
- Ort: `crates/contextra-graph/src/apprh/params.rs:21`
```rust
pub struct ApprhParams
```
- Impl-Block (Trait `Default`) `crates/contextra-graph/src/apprh/params.rs:30`
- Impl-Block `crates/contextra-graph/src/apprh/params.rs:40`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `ppr` | `PprParams` | `crates/contextra-graph/src/apprh/params.rs:23` |
| `hyperedge_decay_factor` | `f32` | `crates/contextra-graph/src/apprh/params.rs:25` |
| `max_iterations` | `u32` | `crates/contextra-graph/src/apprh/params.rs:27` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn validate(&self) -> Result<(), ApprhError>` | `crates/contextra-graph/src/apprh/params.rs:49` |  |

### Modul `apprh::selector`

#### `struct ApprhSelector`
- Ort: `crates/contextra-graph/src/apprh/selector.rs:70`
```rust
pub struct ApprhSelector
```
- Impl-Block `crates/contextra-graph/src/apprh/selector.rs:77`
- Impl-Block (Trait `std::fmt::Debug`) `crates/contextra-graph/src/apprh/selector.rs:215`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(config: ApprhSelectorConfig, monitor: Arc<ApprhGateMonitor>) -> Self` | `crates/contextra-graph/src/apprh/selector.rs:80` |  |
| `pub fn with_default_monitor(config: ApprhSelectorConfig) -> Result<Self, ApprhError>` | `crates/contextra-graph/src/apprh/selector.rs:94` |  |
| `pub fn mode(&self) -> ApprhMode` | `crates/contextra-graph/src/apprh/selector.rs:101` |  |
| `pub fn set_mode(&self, new_mode: ApprhMode) -> Result<(), ApprhError>` | `crates/contextra-graph/src/apprh/selector.rs:111` |  |
| `pub fn promote_to_production(&self) -> Result<(), ApprhError>` | `crates/contextra-graph/src/apprh/selector.rs:127` |  |
| `pub fn demote_to_shadow(&self)` | `crates/contextra-graph/src/apprh/selector.rs:133` |  |
| `pub fn monitor(&self) -> &ApprhGateMonitor` | `crates/contextra-graph/src/apprh/selector.rs:139` |  |
| `pub fn evaluate_and_run<G: PathGraph>( &self, graph: &G, seeds: &[EntityId], ppr_params: &PprParams, apprh_params: &ApprhParams, ) -> Result<Vec<(EntityId, f32)>, ApprhError>` | `crates/contextra-graph/src/apprh/selector.rs:148` |  |

#### `struct ApprhSelectorConfig`
- Ort: `crates/contextra-graph/src/apprh/selector.rs:42`
```rust
pub struct ApprhSelectorConfig
```
- Impl-Block (Trait `Default`) `crates/contextra-graph/src/apprh/selector.rs:51`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `sample_interval` | `u64` | `crates/contextra-graph/src/apprh/selector.rs:44` |
| `max_seeds` | `usize` | `crates/contextra-graph/src/apprh/selector.rs:46` |
| `shadow_top_k` | `usize` | `crates/contextra-graph/src/apprh/selector.rs:48` |

#### `enum ApprhMode`
- Ort: `crates/contextra-graph/src/apprh/selector.rs:27`
```rust
pub enum ApprhMode
```

Varianten:

- `Off` — `crates/contextra-graph/src/apprh/selector.rs:29`
- `Shadow` — `crates/contextra-graph/src/apprh/selector.rs:33`
- `Production` — `crates/contextra-graph/src/apprh/selector.rs:36`

### Modul `apprh::shadow`

#### `struct ApprhShadowComparison`
- Ort: `crates/contextra-graph/src/apprh/shadow.rs:41`
```rust
pub struct ApprhShadowComparison
```

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `max_abs_diff` | `f32` | `crates/contextra-graph/src/apprh/shadow.rs:43` |
| `top_k_overlap` | `f32` | `crates/contextra-graph/src/apprh/shadow.rs:45` |
| `discrepancy` | `bool` | `crates/contextra-graph/src/apprh/shadow.rs:47` |

#### `struct DefaultApprhFlipGate`
- Ort: `crates/contextra-graph/src/apprh/shadow.rs:30`
```rust
pub struct DefaultApprhFlipGate
```
- Impl-Block (Trait `ApprhFlipGate`) `crates/contextra-graph/src/apprh/shadow.rs:32`

#### `trait ApprhFlipGate`
- Ort: `crates/contextra-graph/src/apprh/shadow.rs:22`
```rust
pub trait ApprhFlipGate
```

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn should_flip(&self, report: &ApprhShadowComparison) -> bool` | `crates/contextra-graph/src/apprh/shadow.rs:24` | Evaluates whether APPRH meets default flip criteria. |

#### `fn shadow_compare_forward_push_vs_apprh`
- Ort: `crates/contextra-graph/src/apprh/shadow.rs:63`
```rust
pub fn shadow_compare_forward_push_vs_apprh<G: PathGraph>( graph: &G, seeds: &[EntityId], ppr: &PprParams, apprh_params: &ApprhParams, top_k: usize, ) -> Result<ApprhShadowComparison, ApprhError>
```

### Modul `arc_slice`

#### `struct ArcSlice`
- Ort: `crates/contextra-graph/src/arc_slice.rs:21`
```rust
pub struct ArcSlice<T>
```
- Impl-Block `crates/contextra-graph/src/arc_slice.rs:27`
- Impl-Block (Trait `Deref`) `crates/contextra-graph/src/arc_slice.rs:123`
- Impl-Block (Trait `From<Arc<[T]>>`) `crates/contextra-graph/src/arc_slice.rs:132`
- Impl-Block (Trait `From<Vec<T>>`) `crates/contextra-graph/src/arc_slice.rs:139`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(shared: Arc<[T]>) -> Self` | `crates/contextra-graph/src/arc_slice.rs:31` |  |
| `pub fn from_vec(vec: Vec<T>) -> Self` | `crates/contextra-graph/src/arc_slice.rs:43` |  |
| `pub fn try_slice(&self, range: impl RangeBounds<usize>) -> Option<Self>` | `crates/contextra-graph/src/arc_slice.rs:51` |  |
| `pub fn slice(&self, range: impl RangeBounds<usize>) -> Self` | `crates/contextra-graph/src/arc_slice.rs:79` |  |
| `pub fn as_arc(&self) -> &Arc<[T]>` | `crates/contextra-graph/src/arc_slice.rs:90` |  |
| `pub fn offset(&self) -> usize` | `crates/contextra-graph/src/arc_slice.rs:97` |  |
| `pub fn len(&self) -> usize` | `crates/contextra-graph/src/arc_slice.rs:104` |  |
| `pub fn is_empty(&self) -> bool` | `crates/contextra-graph/src/arc_slice.rs:111` |  |
| `pub fn as_slice(&self) -> &[T]` | `crates/contextra-graph/src/arc_slice.rs:118` |  |

### Modul `cascade`

#### `struct CascadeDeletionRecord`
- Ort: `crates/contextra-graph/src/cascade.rs:100`
```rust
pub struct CascadeDeletionRecord
```

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `doc_id` | `DocId` | `crates/contextra-graph/src/cascade.rs:101` |
| `tombstoned_count` | `usize` | `crates/contextra-graph/src/cascade.rs:102` |
| `wal_seq` | `u64` | `crates/contextra-graph/src/cascade.rs:103` |

#### `struct CascadeInvalidationReport`
- Ort: `crates/contextra-graph/src/cascade.rs:30`
```rust
pub struct CascadeInvalidationReport
```

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `tombstoned_edge_count` | `usize` | `crates/contextra-graph/src/cascade.rs:32` |
| `affected_node_ids` | `Vec<EntityId>` | `crates/contextra-graph/src/cascade.rs:34` |

#### `struct CascadeTicket`
- Ort: `crates/contextra-graph/src/cascade.rs:109`
```rust
pub struct CascadeTicket
```

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `id` | `u64` | `crates/contextra-graph/src/cascade.rs:110` |
| `doc_id` | `DocId` | `crates/contextra-graph/src/cascade.rs:111` |
| `status` | `CascadeStatus` | `crates/contextra-graph/src/cascade.rs:112` |
| `proof` | `Option<CascadeDeletionRecord>` | `crates/contextra-graph/src/cascade.rs:113` |

#### `struct HyperedgeCascadeReport`
- Ort: `crates/contextra-graph/src/cascade.rs:160`
```rust
pub struct HyperedgeCascadeReport
```

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `invalidated` | `Vec<crate::hyperedge::HyperEdgeId>` | `crates/contextra-graph/src/cascade.rs:162` |
| `deferred` | `Vec<crate::hyperedge::HyperEdgeId>` | `crates/contextra-graph/src/cascade.rs:164` |
| `queued_for_background` | `usize` | `crates/contextra-graph/src/cascade.rs:166` |

#### `enum CascadeStatus`
- Ort: `crates/contextra-graph/src/cascade.rs:86`
```rust
pub enum CascadeStatus
```

Varianten:

- `Pending` — `crates/contextra-graph/src/cascade.rs:88`
- `InFlight` — `crates/contextra-graph/src/cascade.rs:90`
- `Completed` — `crates/contextra-graph/src/cascade.rs:92`
- `Failed` — `crates/contextra-graph/src/cascade.rs:94`

#### `trait GraphGarbageCollection`
- Ort: `crates/contextra-graph/src/cascade.rs:118`
```rust
pub trait GraphGarbageCollection
```

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn sweep_orphans(&self, wal_tx: TxId) -> Result<usize>` | `crates/contextra-graph/src/cascade.rs:120` | Sweeps orphaned hyperedges whose participants no longer exist or are fully tombstoned. |

#### `fn cascade_invalidate_edges_for_superseded_doc`
- Ort: `crates/contextra-graph/src/cascade.rs:43`
```rust
pub async fn cascade_invalidate_edges_for_superseded_doc( graph: &CsrGraph, superseded_doc_id: DocId, wal_seq: u64, ) -> Result<CascadeInvalidationReport>
```
Dokumentation: Tombstoniert alle Kanten, die von `superseded_doc_id` abgeleitet wurden, als Reaktion auf ein Supersedes-Ereignis. Idempotent: mehrfacher Aufruf mit derselben DocId hat keinen zusätzlichen Effekt. INVARIANTE: Jede hier tombstonierte Kante erhält einen Provenienz-Eintrag mit der WAL-Sequenznummer dieses Aufrufs (INV-GRAPH-PROV-1).

#### `fn cascade_invalidate_hyperedges_for_superseded_doc`
- Ort: `crates/contextra-graph/src/cascade.rs:264`
```rust
pub async fn cascade_invalidate_hyperedges_for_superseded_doc( graph: &CsrGraph, superseded_doc_id: DocId, wal_seq: u64, ) -> Result<HyperedgeCascadeReport>
```
Dokumentation: Cascade invalidates hyperedges derived from `superseded_doc_id` with fan-out protection and recursive super-edge ancestor invalidation. Up to `MAX_HYPEREDGE_CASCADE_FANOUT` hyperedges (ancestors and document hyperedges combined) are tombstoned synchronously in topological order (parents before children). Any excess hyperedges are persisted to LSM storage under `__graph:cascade_queue:...` and queued in `CsrGraph`'s cascade queue for deferred background processing. INVARIANTE: Jede hier tombstonierte Hyperkante erhält einen Provenienz-Eintrag mit der WAL-Sequenznummer dieses Aufrufs (INV-GRAPH-P

#### `fn topological_sort_hyperedge_closure`
- Ort: `crates/contextra-graph/src/cascade.rs:190`
```rust
pub fn topological_sort_hyperedge_closure( graph: &CsrGraph, closure_nodes: &[crate::hyperedge::HyperEdgeId], ) -> Vec<crate::hyperedge::HyperEdgeId>
```

#### `fn verify_doc_hyperedge_deletion_completeness`
- Ort: `crates/contextra-graph/src/cascade.rs:384`
```rust
pub fn verify_doc_hyperedge_deletion_completeness( graph: &CsrGraph, target_doc_id: DocId, ) -> Result<()>
```

#### `type CascadeReport`
- Ort: `crates/contextra-graph/src/cascade.rs:171`
```rust
pub type CascadeReport = HyperedgeCascadeReport
```

#### `const CASCADE_QUEUE_PREFIX`
- Ort: `crates/contextra-graph/src/cascade.rs:81`
```rust
pub const CASCADE_QUEUE_PREFIX: &str = ""
```

#### `const DEFAULT_HYPEREDGE_CASCADE_FANOUT_LIMIT`
- Ort: `crates/contextra-graph/src/cascade.rs:25`
```rust
pub const DEFAULT_HYPEREDGE_CASCADE_FANOUT_LIMIT: usize = MAX_HYPEREDGE_CASCADE_FANOUT
```

#### `const MAX_HYPEREDGE_CASCADE_FANOUT`
- Ort: `crates/contextra-graph/src/cascade.rs:21`
```rust
pub const MAX_HYPEREDGE_CASCADE_FANOUT: usize = 1_000
```

### Modul `community`

#### `struct CommunityAssignment`
- Ort: `crates/contextra-graph/src/community.rs:144`
```rust
pub struct CommunityAssignment
```

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `entity_id` | `EntityId` | `crates/contextra-graph/src/community.rs:146` |
| `community_id` | `u64` | `crates/contextra-graph/src/community.rs:148` |
| `hyperedges_included` | `bool` | `crates/contextra-graph/src/community.rs:151` |

#### `struct CommunityDetectionConfig`
- Ort: `crates/contextra-graph/src/community.rs:33`
```rust
pub struct CommunityDetectionConfig
```
- Impl-Block (Trait `Default`) `crates/contextra-graph/src/community.rs:131`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `max_iterations` | `u32` | `crates/contextra-graph/src/community.rs:35` |
| `seed` | `u64` | `crates/contextra-graph/src/community.rs:37` |
| `hyperedges_included` | `bool` | `crates/contextra-graph/src/community.rs:40` |

#### `struct StarExpansionIterator`
- Ort: `crates/contextra-graph/src/community.rs:70`
```rust
pub struct StarExpansionIterator
```
- Impl-Block `crates/contextra-graph/src/community.rs:77`
- Impl-Block (Trait `Iterator`) `crates/contextra-graph/src/community.rs:122`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(graph: &CsrGraph) -> Self` | `crates/contextra-graph/src/community.rs:80` |  |
| `pub fn next_with_weight(&mut self) -> Option<(EntityId, VirtualHyperedgeNode, f32)>` | `crates/contextra-graph/src/community.rs:100` |  |

#### `struct VirtualHyperedgeNode`
- Ort: `crates/contextra-graph/src/community.rs:47`
```rust
pub struct VirtualHyperedgeNode
```
- Impl-Block `crates/contextra-graph/src/community.rs:52`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `hyperedge_id` | `crate::hyperedge::HyperEdgeId` | `crates/contextra-graph/src/community.rs:49` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(hyperedge_id: u64) -> Self` | `crates/contextra-graph/src/community.rs:55` |  |

#### `fn detect_communities`
- Ort: `crates/contextra-graph/src/community.rs:215`
```rust
pub async fn detect_communities( graph: &CsrGraph, config: &CommunityDetectionConfig, ) -> Result<Vec<CommunityAssignment>>
```
Dokumentation: Detects semantic communities in the given CSR graph using the deterministic Leiden algorithm. # Determinism and Reproducibility Guarantees - Node traversal order per iteration is randomized using a deterministic `SimpleRng` seeded by `config.seed.wrapping_add(iter)`. - Initial node sequence is sorted by `EntityId` ascending. - Tie-breaking rule when multiple candidate communities yield equal modularity gain: The community with the smallest `u64` numerical value (smallest `EntityId`) wins. - Re-running community detection over the identical graph structure with identical `config.seed` guarantee

### Modul `compat`

#### `mod algo_error_bridge`
- Ort: `crates/contextra-graph/src/compat.rs:20`
```rust
pub mod algo_error_bridge
```
- Gate: `#[cfg(feature = "workspace")]`

#### `mod collection_mutation`
- Ort: `crates/contextra-graph/src/compat.rs:12`
```rust
pub mod collection_mutation
```
- Gate: `#[cfg(feature = "workspace")]`

#### `mod error`
- Ort: `crates/contextra-graph/src/compat.rs:4`
```rust
pub mod error
```
- Gate: `#[cfg(feature = "workspace")]`

#### `mod graph_index`
- Ort: `crates/contextra-graph/src/compat.rs:10`
```rust
pub mod graph_index
```
- Gate: `#[cfg(feature = "workspace")]`

#### `mod graph_index_ext`
- Ort: `crates/contextra-graph/src/compat.rs:16`
```rust
pub mod graph_index_ext
```
- Gate: `#[cfg(feature = "workspace")]`

#### `mod llm_bridge`
- Ort: `crates/contextra-graph/src/compat.rs:22`
```rust
pub mod llm_bridge
```
- Gate: `#[cfg(feature = "workspace")]`

#### `mod provider`
- Ort: `crates/contextra-graph/src/compat.rs:6`
```rust
pub mod provider
```
- Gate: `#[cfg(feature = "workspace")]`

#### `mod purger`
- Ort: `crates/contextra-graph/src/compat.rs:8`
```rust
pub mod purger
```
- Gate: `#[cfg(feature = "workspace")]`

#### `mod replay_hook`
- Ort: `crates/contextra-graph/src/compat.rs:18`
```rust
pub mod replay_hook
```
- Gate: `#[cfg(feature = "workspace")]`

#### `mod storage_bridge`
- Ort: `crates/contextra-graph/src/compat.rs:14`
```rust
pub mod storage_bridge
```
- Gate: `#[cfg(feature = "workspace")]`

### Modul `compat::graph_index`

#### `mod inherent_shims`
- Ort: `crates/contextra-graph/src/compat/graph_index/mod.rs:309`
```rust
pub mod inherent_shims
```

#### `mod mutations`
- Ort: `crates/contextra-graph/src/compat/graph_index/mod.rs:16`
```rust
pub mod mutations
```

#### `mod traversal`
- Ort: `crates/contextra-graph/src/compat/graph_index/mod.rs:17`
```rust
pub mod traversal
```

### Modul `compat::graph_index_ext`

#### `trait GraphIndexExt`
- Ort: `crates/contextra-graph/src/compat/graph_index_ext.rs:4`
```rust
pub trait GraphIndexExt: contextra_ports::GraphIndex
```

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn remove_entity<'a>( &'a self, tx: contextra_types::TxId, entity: EntityId, ) -> contextra_ports::BoxFuture<'a, contextra_types::Result<()>>` | `crates/contextra-graph/src/compat/graph_index_ext.rs:5` |  |

### Modul `compat::provider`

#### `struct InMemoryGraphProvider`
- Ort: `crates/contextra-graph/src/compat/provider.rs:76`
```rust
pub struct InMemoryGraphProvider
```
Dokumentation: Standalone in-memory reference implementation of `GraphProvider`.
- Impl-Block (Trait `PathGraph`) `crates/contextra-graph/src/compat/provider.rs:56`
- Impl-Block `crates/contextra-graph/src/compat/provider.rs:84`
- Impl-Block (Trait `GraphProvider`) `crates/contextra-graph/src/compat/provider.rs:122`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new() -> Self` | `crates/contextra-graph/src/compat/provider.rs:85` |  |
| `pub fn add_node(&mut self, node: EntityId)` | `crates/contextra-graph/src/compat/provider.rs:88` |  |
| `pub fn add_edge(&mut self, source: EntityId, target: EntityId, weight: f32)` | `crates/contextra-graph/src/compat/provider.rs:93` |  |
| `pub fn add_hyperedge(&mut self, hyperedge: HyperEdge)` | `crates/contextra-graph/src/compat/provider.rs:109` |  |

#### `trait GraphProvider`
- Ort: `crates/contextra-graph/src/compat/provider.rs:19`
```rust
pub trait GraphProvider: Send + Sync
```

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn node_ids(&self) -> Vec<EntityId>` | `crates/contextra-graph/src/compat/provider.rs:21` | Returns all active node IDs in the graph. |
| `fn node_count(&self) -> usize` | `crates/contextra-graph/src/compat/provider.rs:23` | Returns the number of active nodes. |
| `fn contains_node(&self, node: EntityId) -> bool` | `crates/contextra-graph/src/compat/provider.rs:27` | Checks if a node exists. |
| `fn outgoing_edges(&self, node: EntityId) -> Vec<(EntityId, f32)>` | `crates/contextra-graph/src/compat/provider.rs:29` | Returns outgoing neighbors of `node` with their edge weights. |
| `fn incoming_edges(&self, node: EntityId) -> Vec<(EntityId, f32)>` | `crates/contextra-graph/src/compat/provider.rs:31` | Returns incoming predecessors of `node` with their edge weights. |
| `fn out_degree(&self, node: EntityId) -> usize` | `crates/contextra-graph/src/compat/provider.rs:33` | Returns out-degree. |
| `fn in_degree(&self, node: EntityId) -> usize` | `crates/contextra-graph/src/compat/provider.rs:37` | Returns in-degree. |
| `fn hyperedges_for_entity(&self, _node: EntityId) -> Vec<HyperEdgeId>` | `crates/contextra-graph/src/compat/provider.rs:41` | Returns hyperedge IDs for entity. |
| `fn get_hyperedge(&self, _id: HyperEdgeId) -> Option<Arc<HyperEdge>>` | `crates/contextra-graph/src/compat/provider.rs:45` | Resolves HyperEdgeId. |
| `fn hyperedge_participants(&self, id: HyperEdgeId) -> Arc<[RoleBinding]>` | `crates/contextra-graph/src/compat/provider.rs:49` | Returns role bindings. |

#### `fn compute_pagerank`
- Ort: `crates/contextra-graph/src/compat/provider.rs:150`
```rust
pub fn compute_pagerank<G: GraphProvider>( graph: &G, damping: f32, max_iter: usize, tol: f32, ) -> HashMap<EntityId, f32>
```
Dokumentation: Generic PageRank computation executing against any `GraphProvider`.

#### `fn detect_communities_on_provider`
- Ort: `crates/contextra-graph/src/compat/provider.rs:215`
```rust
pub fn detect_communities_on_provider<G: GraphProvider>( graph: &G, config: &crate::community::CommunityDetectionConfig, ) -> Result<Vec<crate::community::CommunityAssignment>>
```
Dokumentation: Generic Community Detection on any `GraphProvider`.

### Modul `compat::purger`

#### `struct GraphPurger`
- Ort: `crates/contextra-graph/src/compat/purger.rs:33`
```rust
pub struct GraphPurger
```
- Impl-Block `crates/contextra-graph/src/compat/purger.rs:37`
- Impl-Block (Trait `Purger`) `crates/contextra-graph/src/compat/purger.rs:52`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(graph: Arc<CsrGraph>) -> Self` | `crates/contextra-graph/src/compat/purger.rs:39` | Erzeugt einen neuen [`GraphPurger`]. |

#### `fn graph_purger`
- Ort: `crates/contextra-graph/src/compat/purger.rs:48`
```rust
pub fn graph_purger(graph: Arc<CsrGraph>) -> Arc<dyn Purger>
```

### Modul `compat::storage_bridge`

#### `struct EngineHandle`
- Ort: `crates/contextra-graph/src/compat/storage_bridge.rs:53`
```rust
pub struct EngineHandle(pub Arc<dyn StorageEngine>)
```
- Impl-Block (Trait `GraphStorage`) `crates/contextra-graph/src/compat/storage_bridge.rs:55`

### Modul `consistency_enforcement`

#### `struct ConflictPattern`
- Ort: `crates/contextra-graph/src/consistency_enforcement.rs:42`
```rust
pub struct ConflictPattern
```

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `pattern_hash` | `[u8; 32]` | `crates/contextra-graph/src/consistency_enforcement.rs:44` |
| `contradiction_count` | `u32` | `crates/contextra-graph/src/consistency_enforcement.rs:46` |
| `first_detected_tx` | `TxId` | `crates/contextra-graph/src/consistency_enforcement.rs:48` |
| `last_detected_tx` | `TxId` | `crates/contextra-graph/src/consistency_enforcement.rs:50` |
| `suppressed` | `bool` | `crates/contextra-graph/src/consistency_enforcement.rs:52` |

#### `struct ConsistencyEnforcer`
- Ort: `crates/contextra-graph/src/consistency_enforcement.rs:104`
```rust
pub struct ConsistencyEnforcer
```
- Impl-Block `crates/contextra-graph/src/consistency_enforcement.rs:109`
- Impl-Block (Trait `Default`) `crates/contextra-graph/src/consistency_enforcement.rs:217`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(suppression_threshold: u32) -> Self` | `crates/contextra-graph/src/consistency_enforcement.rs:116` |  |
| `pub fn check_before_insert(&mut self, assertion: &EdgeAssertion) -> Option<ConflictPattern>` | `crates/contextra-graph/src/consistency_enforcement.rs:126` |  |
| `pub fn detect_contradiction<D: ContradictionDetector>( &self, detector: &D, a: &EdgeAssertion, b: &EdgeAssertion, ) -> bool` | `crates/contextra-graph/src/consistency_enforcement.rs:135` |  |
| `pub fn record_contradiction( &mut self, pattern_hash: [u8` | `crates/contextra-graph/src/consistency_enforcement.rs:149` |  |
| `pub fn is_suppressed(&self, pattern_hash: [u8` | `crates/contextra-graph/src/consistency_enforcement.rs:177` |  |
| `pub fn active_patterns(&self) -> impl Iterator<Item = &ConflictPattern>` | `crates/contextra-graph/src/consistency_enforcement.rs:185` |  |
| `pub fn get_pattern(&self, pattern_hash: &[u8` | `crates/contextra-graph/src/consistency_enforcement.rs:191` |  |
| `pub fn suppression_threshold(&self) -> u32` | `crates/contextra-graph/src/consistency_enforcement.rs:197` |  |
| `pub fn suggest_tombstone_candidates( &self, csr_edges_matching_pattern: &[EdgeId], ) -> Vec<EdgeId>` | `crates/contextra-graph/src/consistency_enforcement.rs:209` |  |

Assoziierte Konstanten/Typen:

- `pub const DEFAULT_SUPPRESSION_THRESHOLD: u32 = 3` — `crates/contextra-graph/src/consistency_enforcement.rs:112`

#### `struct EdgeAssertion`
- Ort: `crates/contextra-graph/src/consistency_enforcement.rs:58`
```rust
pub struct EdgeAssertion
```
- Impl-Block `crates/contextra-graph/src/consistency_enforcement.rs:67`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `subject` | `NodeIdx` | `crates/contextra-graph/src/consistency_enforcement.rs:60` |
| `predicate_hash` | `[u8; 32]` | `crates/contextra-graph/src/consistency_enforcement.rs:62` |
| `object_repr` | `Vec<u8>` | `crates/contextra-graph/src/consistency_enforcement.rs:64` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn pattern_hash(&self) -> [u8` | `crates/contextra-graph/src/consistency_enforcement.rs:70` |  |

#### `struct ExactPredicateConflictDetector`
- Ort: `crates/contextra-graph/src/consistency_enforcement.rs:91`
```rust
pub struct ExactPredicateConflictDetector
```
- Impl-Block (Trait `ContradictionDetector`) `crates/contextra-graph/src/consistency_enforcement.rs:93`

#### `trait ContradictionDetector`
- Ort: `crates/contextra-graph/src/consistency_enforcement.rs:81`
```rust
pub trait ContradictionDetector
```

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn conflicts(&self, a: &EdgeAssertion, b: &EdgeAssertion) -> bool` | `crates/contextra-graph/src/consistency_enforcement.rs:83` | Prüft, ob zwei Aussagen im Widerspruch zueinander stehen. |

#### `type EdgeId`
- Ort: `crates/contextra-graph/src/consistency_enforcement.rs:37`
```rust
pub type EdgeId = (EntityId, EntityId)
```

#### `type NodeIdx`
- Ort: `crates/contextra-graph/src/consistency_enforcement.rs:29`
```rust
pub type NodeIdx = u64
```

### Modul `crate`

#### `mod apprh`
- Ort: `crates/contextra-graph/src/lib.rs:43`
```rust
pub mod apprh
```
- Gate: `#[cfg(feature = "apprh-diffusion")]`

#### `mod arc_slice`
- Ort: `crates/contextra-graph/src/lib.rs:44`
```rust
pub mod arc_slice
```

#### `mod cascade`
- Ort: `crates/contextra-graph/src/lib.rs:45`
```rust
pub mod cascade
```

#### `mod community`
- Ort: `crates/contextra-graph/src/lib.rs:46`
```rust
pub mod community
```

#### `mod compat`
- Ort: `crates/contextra-graph/src/lib.rs:48`
```rust
pub mod compat
```

#### `mod consistency_enforcement`
- Ort: `crates/contextra-graph/src/lib.rs:49`
```rust
pub mod consistency_enforcement
```

#### `mod csr`
- Ort: `crates/contextra-graph/src/lib.rs:50`
```rust
pub mod csr
```

#### `mod edge_reinforcement`
- Ort: `crates/contextra-graph/src/lib.rs:52`
```rust
pub mod edge_reinforcement
```
- Gate: `#[cfg(feature = "edge-reinforcement-learning")]`

#### `mod edge_reinforcement_buffer`
- Ort: `crates/contextra-graph/src/lib.rs:54`
```rust
pub mod edge_reinforcement_buffer
```
- Gate: `#[cfg(feature = "edge-reinforcement-learning")]`

#### `mod entity_extraction`
- Ort: `crates/contextra-graph/src/lib.rs:55`
```rust
pub mod entity_extraction
```

#### `mod error`
- Ort: `crates/contextra-graph/src/lib.rs:56`
```rust
pub mod error
```

#### `mod graph_view`
- Ort: `crates/contextra-graph/src/lib.rs:75`
```rust
pub mod graph_view
```

#### `mod hyperedge`
- Ort: `crates/contextra-graph/src/lib.rs:57`
```rust
pub mod hyperedge
```

#### `mod hyperedge_suggest`
- Ort: `crates/contextra-graph/src/lib.rs:58`
```rust
pub mod hyperedge_suggest
```

#### `mod path_rag`
- Ort: `crates/contextra-graph/src/lib.rs:59`
```rust
pub mod path_rag
```

#### `mod percolation`
- Ort: `crates/contextra-graph/src/lib.rs:61`
```rust
pub mod percolation
```
- Gate: `#[cfg(feature = "graph-connectivity-health")]`

#### `mod ppr`
- Ort: `crates/contextra-graph/src/lib.rs:62`
```rust
pub mod ppr
```

#### `mod ppr_stream`
- Ort: `crates/contextra-graph/src/lib.rs:64`
```rust
pub mod ppr_stream
```
- Gate: `#[cfg(feature = "workspace")]`

#### `mod provenance`
- Ort: `crates/contextra-graph/src/lib.rs:65`
```rust
pub mod provenance
```

#### `mod provider`
- Ort: `crates/contextra-graph/src/lib.rs:66`
```rust
pub mod provider
```

#### `mod purger`
- Ort: `crates/contextra-graph/src/lib.rs:67`
```rust
pub mod purger
```

#### `mod session_dag`
- Ort: `crates/contextra-graph/src/lib.rs:69`
```rust
pub mod session_dag
```
- Gate: `#[cfg(feature = "experimental-graph")]`

#### `mod storage`
- Ort: `crates/contextra-graph/src/lib.rs:74`
```rust
pub mod storage
```

#### `mod test_doubles`
- Ort: `crates/contextra-graph/src/lib.rs:71`
```rust
pub mod test_doubles
```

#### `mod text_gen`
- Ort: `crates/contextra-graph/src/lib.rs:76`
```rust
pub mod text_gen
```

#### `mod tl_hfd`
- Ort: `crates/contextra-graph/src/lib.rs:73`
```rust
pub mod tl_hfd
```

#### `mod types`
- Ort: `crates/contextra-graph/src/lib.rs:72`
```rust
pub mod types
```

### Modul `csr::graph_read`

#### `mod communities`
- Ort: `crates/contextra-graph/src/csr/graph_read/mod.rs:15`
```rust
pub mod communities
```

#### `mod pagerank`
- Ort: `crates/contextra-graph/src/csr/graph_read/mod.rs:16`
```rust
pub mod pagerank
```

#### `mod path_rag`
- Ort: `crates/contextra-graph/src/csr/graph_read/mod.rs:17`
```rust
pub mod path_rag
```

### Modul `csr::graph_write`

#### `mod edge_mutation`
- Ort: `crates/contextra-graph/src/csr/graph_write/mod.rs:15`
```rust
pub mod edge_mutation
```

#### `mod entity`
- Ort: `crates/contextra-graph/src/csr/graph_write/mod.rs:16`
```rust
pub mod entity
```

#### `mod hyperedge`
- Ort: `crates/contextra-graph/src/csr/graph_write/mod.rs:17`
```rust
pub mod hyperedge
```

#### `mod tombstone`
- Ort: `crates/contextra-graph/src/csr/graph_write/mod.rs:18`
```rust
pub mod tombstone
```

#### `struct CsrGraph`
- Ort: `crates/contextra-graph/src/csr/graph_write/mod.rs:35`
```rust
pub struct CsrGraph
```
- Impl-Block `crates/contextra-graph/src/apprh/mod.rs:86`
- Impl-Block (Trait `GraphGarbageCollection`) `crates/contextra-graph/src/cascade.rs:123`
- Impl-Block (Trait `GraphCollectionMutation`) `crates/contextra-graph/src/compat/collection_mutation.rs:19`
- Impl-Block `crates/contextra-graph/src/compat/graph_index/inherent_shims.rs:5`
- Impl-Block (Trait `Default`) `crates/contextra-graph/src/compat/graph_index/mod.rs:99`
- Impl-Block (Trait `GraphIndex`) `crates/contextra-graph/src/compat/graph_index/mod.rs:105`
- Impl-Block (Trait `GraphIndexExt`) `crates/contextra-graph/src/compat/graph_index/mutations.rs:51`
- Impl-Block `crates/contextra-graph/src/csr/graph_persist.rs:34`
- Impl-Block `crates/contextra-graph/src/csr/graph_read/communities.rs:21`
- Impl-Block `crates/contextra-graph/src/csr/graph_read/mod.rs:26`
- Impl-Block `crates/contextra-graph/src/csr/graph_read/pagerank.rs:20`
- Impl-Block `crates/contextra-graph/src/csr/graph_read/path_rag.rs:21`
- Impl-Block `crates/contextra-graph/src/csr/graph_write/edge_mutation.rs:22`
- Impl-Block `crates/contextra-graph/src/csr/graph_write/entity.rs:20`
- Impl-Block `crates/contextra-graph/src/csr/graph_write/hyperedge.rs:49`
- Impl-Block `crates/contextra-graph/src/csr/graph_write/mod.rs:47`
- Impl-Block `crates/contextra-graph/src/csr/graph_write/tombstone.rs:22`
- Impl-Block (Trait `crate::provider::GraphProvider`) `crates/contextra-graph/src/csr/path_graph.rs:23`
- Impl-Block (Trait `crate::path_rag::PathGraph`) `crates/contextra-graph/src/csr/path_graph.rs:64`
- Impl-Block (Trait `crate::path_rag::PathGraph`) `crates/contextra-graph/src/csr/path_graph.rs:176`
- Impl-Block (Trait `GraphView`) `crates/contextra-graph/src/graph_view.rs:16`
- Impl-Block `crates/contextra-graph/src/tl_hfd/mod.rs:48`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `config` | `CsrGraphConfig` | `crates/contextra-graph/src/csr/graph_write/mod.rs:36` |
| `inner` | `Arc<ArcSwap<GraphInner>>` | `crates/contextra-graph/src/csr/graph_write/mod.rs:37` |
| `write_state` | `Arc<Mutex<GraphInner>>` | `crates/contextra-graph/src/csr/graph_write/mod.rs:38` |
| `storage` | `Option<Arc<dyn StorageEngine>>` | `crates/contextra-graph/src/csr/graph_write/mod.rs:40` |
| `last_tx_id` | `AtomicU64` | `crates/contextra-graph/src/csr/graph_write/mod.rs:41` |
| `consistency_enforcer` | `Option<RwLock<ConsistencyEnforcer>>` | `crates/contextra-graph/src/csr/graph_write/mod.rs:42` |
| `doc_edge_index` | `crate::provenance::DocEdgeIndex` | `crates/contextra-graph/src/csr/graph_write/mod.rs:43` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn apprh_diffusion( &self, seeds: &[EntityId], params: &ApprhParams, ) -> Result<AHashMap<EntityId, f32>, ApprhError>` | `crates/contextra-graph/src/apprh/mod.rs:92` |  |
| `pub fn apprh_shadow_compare( &self, seeds: &[EntityId], ppr: &PprParams, apprh_params: &ApprhParams, top_k: usize, ) -> Result<ApprhShadowComparison, ApprhError>` | `crates/contextra-graph/src/apprh/mod.rs:102` |  |
| `pub fn personalized_page_rank_at<'a>( &'a self, seed_nodes: &'a [contextra_types::EntityId], config: &'a contextra_types::PprConfig, seq_no: u64, ) -> contextra_ports::BoxFuture< 'a, contextra_types::Result<Vec<(contextra_types::EntityId, f32)>>, >` | `crates/contextra-graph/src/compat/graph_index/inherent_shims.rs:7` | Delegiert an [`contextra_ports::GraphIndex::personalized_page_rank_at`]. |
| `pub fn rollback_to_tx<'a>( &'a self, tx_id: contextra_types::TxId, ) -> contextra_ports::BoxFuture<'a, contextra_types::Result<()>>` | `crates/contextra-graph/src/compat/graph_index/inherent_shims.rs:20` | Delegiert an [`contextra_ports::GraphIndex::rollback_to_tx`]. |
| `pub async fn persist_entity<S: StorageEngine + ?Sized>( &self, storage: &S, tx: TxId, entity: &Entity, ) -> Result<()>` | `crates/contextra-graph/src/csr/graph_persist.rs:36` | Persistiert eine einzelne Entity in den übergebenen Storage. |
| `pub async fn persist_edge<S: StorageEngine + ?Sized>( &self, storage: &S, tx: TxId, from: &EntityId, to: &EntityId, payload: &PersistedEdgePayload, ) -> Result<()>` | `crates/contextra-graph/src/csr/graph_persist.rs:49` | Persistiert eine einzelne Edge in den übergebenen Storage. |
| `pub async fn delete_edge_persistence<S: StorageEngine + ?Sized>( &self, storage: &S, tx: TxId, from: &EntityId, to: &EntityId, ) -> Result<()>` | `crates/contextra-graph/src/csr/graph_persist.rs:70` | Löscht eine einzelne Edge aus dem übergebenen Storage. |
| `pub async fn load_from_storage<S: StorageEngine + ?Sized>(storage: &S) -> Result<Self>` | `crates/contextra-graph/src/csr/graph_persist.rs:88` | Lädt den kompletten Graph-Zustand aus dem Storage (beim Startup). |
| `pub fn set_communities_batch(&self, assignments: &[crate::CommunityAssignment])` | `crates/contextra-graph/src/csr/graph_read/communities.rs:23` |  |
| `pub async fn get_communities_batch( &self, entity_ids: &[EntityId], ) -> Result<HashMap<EntityId, u64>>` | `crates/contextra-graph/src/csr/graph_read/communities.rs:31` |  |
| `pub fn compact(&self)` | `crates/contextra-graph/src/csr/graph_read/mod.rs:28` |  |
| `pub async fn compact_async(&self) -> Result<()>` | `crates/contextra-graph/src/csr/graph_read/mod.rs:51` |  |
| `pub async fn neighbors(&self, start: EntityId) -> Result<Vec<EntityId>>` | `crates/contextra-graph/src/csr/graph_read/mod.rs:90` |  |
| `pub fn entity_count(&self) -> usize` | `crates/contextra-graph/src/csr/graph_read/mod.rs:140` |  |
| `pub fn entity_exists(&self, id: EntityId) -> bool` | `crates/contextra-graph/src/csr/graph_read/mod.rs:149` |  |
| `pub async fn is_entity_deleted(&self, entity: EntityId) -> bool` | `crates/contextra-graph/src/csr/graph_read/mod.rs:158` |  |
| `pub fn edge_count(&self) -> usize` | `crates/contextra-graph/src/csr/graph_read/mod.rs:168` |  |
| `pub async fn remove_entity(&self, tx: TxId, entity: EntityId) -> Result<()>` | `crates/contextra-graph/src/csr/graph_read/mod.rs:176` |  |
| `pub async fn pagerank( &self, damping_factor: f32, max_iterations: usize, tolerance: f32, ) -> HashMap<EntityId, f32>` | `crates/contextra-graph/src/csr/graph_read/pagerank.rs:21` |  |
| `pub async fn get_deleted_node_indices(&self) -> HashSet<usize>` | `crates/contextra-graph/src/csr/graph_read/pagerank.rs:101` |  |
| `pub async fn deleted_view(&self) -> crate::DeletedView` | `crates/contextra-graph/src/csr/graph_read/pagerank.rs:125` |  |
| `pub async fn personalized_page_rank_with_context_async( &self, seed_nodes: &[EntityId], config: &PprConfig, ctx: &mut crate::PprContext, ) -> Vec<(EntityId, f32)>` | `crates/contextra-graph/src/csr/graph_read/pagerank.rs:129` |  |
| `pub fn path_rag_at<'a>( &'a self, start_nodes: &'a [EntityId], max_hops: usize, seq_no: u64, ) -> GraphFuture<'a, Result<Vec<(EntityId, f32)>>>` | `crates/contextra-graph/src/csr/graph_read/path_rag.rs:23` |  |
| `pub async fn add_edge( self: &Arc<Self>, from: EntityId, to: EntityId, weight: f32, tx_valid_from: Option<TxId>, tx_valid_to: Option<TxId>, business_valid_from: Option<i64>, business_valid_to: Option<i64>, source_doc_id: Option<DocId>, predicate_hash: Option<[u8` | `crates/contextra-graph/src/csr/graph_write/edge_mutation.rs:57` |  |
| `pub async fn insert_edge_direct( self: &Arc<Self>, from: EntityId, to: EntityId, weight: f32, ) -> Result<()>` | `crates/contextra-graph/src/csr/graph_write/edge_mutation.rs:119` |  |
| `pub async fn insert_edge_direct_with_validity( self: &Arc<Self>, from: EntityId, to: EntityId, weight: f32, tx_valid_from: Option<TxId>, tx_valid_to: Option<TxId>, ) -> Result<()>` | `crates/contextra-graph/src/csr/graph_write/edge_mutation.rs:129` |  |
| `pub async fn insert_edge_direct_with_bitemporal_validity( self: &Arc<Self>, from: EntityId, to: EntityId, weight: f32, tx_valid_from: Option<TxId>, tx_valid_to: Option<TxId>, business_valid_from: Option<i64>, business_valid_to: Option<i64>, source_doc_id: Option<DocId>, ) -> Result<()>` | `crates/contextra-graph/src/csr/graph_write/edge_mutation.rs:153` |  |
| `pub fn load_edge_direct( &self, from: EntityId, to: EntityId, weight: f32, tx_valid_from: Option<TxId>, tx_valid_to: Option<TxId>, business_valid_from: Option<i64>, business_valid_to: Option<i64>, source_doc_id: Option<DocId>, ) -> Result<()>` | `crates/contextra-graph/src/csr/graph_write/edge_mutation.rs:181` |  |
| `pub fn insert_entity_direct(&self, entity: Entity) -> Result<()>` | `crates/contextra-graph/src/csr/graph_write/entity.rs:22` |  |
| `pub fn load_entity_direct(&self, entity: Entity) -> Result<()>` | `crates/contextra-graph/src/csr/graph_write/entity.rs:33` |  |
| `pub fn enqueue_cascade_deferred( &self, doc_id: DocId, hyperedge_ids: &[crate::hyperedge::HyperEdgeId], )` | `crates/contextra-graph/src/csr/graph_write/hyperedge.rs:51` |  |
| `pub fn pending_cascade_queue_len(&self) -> usize` | `crates/contextra-graph/src/csr/graph_write/hyperedge.rs:63` |  |
| `pub async fn process_cascade_queue(&self, batch_size: usize, wal_tx: TxId) -> Result<usize>` | `crates/contextra-graph/src/csr/graph_write/hyperedge.rs:67` |  |
| `pub fn relate_n_ary( &self, id: crate::hyperedge::HyperEdgeId, predicate: crate::csr::EdgeType, participants: Vec<crate::hyperedge::RoleBinding>, weight: f32, doc_id: Option<DocId>, ) -> std::result::Result<crate::hyperedge::HyperEdge, GraphMutationError>` | `crates/contextra-graph/src/csr/graph_write/hyperedge.rs:117` |  |
| `pub async fn persist_hyperedge( &self, tx: TxId, hyperedge: &crate::hyperedge::HyperEdge, ) -> Result<()>` | `crates/contextra-graph/src/csr/graph_write/hyperedge.rs:139` |  |
| `pub fn insert_hyperedge_direct(&self, hyperedge: crate::hyperedge::HyperEdge)` | `crates/contextra-graph/src/csr/graph_write/hyperedge.rs:168` |  |
| `pub fn parent_hyperedges_of( &self, child: crate::hyperedge::HyperEdgeId, ) -> Vec<crate::hyperedge::HyperEdgeId>` | `crates/contextra-graph/src/csr/graph_write/hyperedge.rs:197` |  |
| `pub fn hyperedges_for_doc(&self, doc_id: DocId) -> Vec<crate::hyperedge::HyperEdgeId>` | `crates/contextra-graph/src/csr/graph_write/hyperedge.rs:222` |  |
| `pub fn max_hyperedge_id(&self) -> u64` | `crates/contextra-graph/src/csr/graph_write/hyperedge.rs:242` |  |
| `pub fn get_hyperedge( &self, id: crate::hyperedge::HyperEdgeId, ) -> Option<Arc<crate::hyperedge::HyperEdge>>` | `crates/contextra-graph/src/csr/graph_write/hyperedge.rs:253` |  |
| `pub fn hyperedges_for_entity(&self, entity_id: EntityId) -> Vec<crate::hyperedge::HyperEdgeId>` | `crates/contextra-graph/src/csr/graph_write/hyperedge.rs:266` |  |
| `pub fn commit_super_edge_batch( &self, tombstone_ids: &[crate::hyperedge::HyperEdgeId], new_edges: Vec<crate::hyperedge::HyperEdge>, wal_tx: TxId, ) -> std::result::Result<(), GraphMutationError>` | `crates/contextra-graph/src/csr/graph_write/hyperedge.rs:369` |  |
| `pub fn insert_hyperedge(&self, edge: crate::hyperedge::HyperEdge)` | `crates/contextra-graph/src/csr/graph_write/hyperedge.rs:396` |  |
| `pub fn new() -> Self` | `crates/contextra-graph/src/csr/graph_write/mod.rs:49` |  |
| `pub fn with_config(config: CsrGraphConfig) -> Self` | `crates/contextra-graph/src/csr/graph_write/mod.rs:54` |  |
| `pub fn with_storage(storage: Arc<dyn StorageEngine>) -> Self` | `crates/contextra-graph/src/csr/graph_write/mod.rs:71` |  |
| `pub fn with_config_and_storage( config: CsrGraphConfig, storage: Arc<dyn StorageEngine>, ) -> Self` | `crates/contextra-graph/src/csr/graph_write/mod.rs:77` |  |
| `pub fn with_consistency_enforcer(suppression_threshold: u32) -> Self` | `crates/contextra-graph/src/csr/graph_write/mod.rs:95` |  |
| `pub fn inner_read(&self) -> arc_swap::Guard<Arc<GraphInner>>` | `crates/contextra-graph/src/csr/graph_write/mod.rs:113` |  |
| `pub fn set_storage(&mut self, storage: Arc<dyn StorageEngine>)` | `crates/contextra-graph/src/csr/graph_write/mod.rs:126` |  |
| `pub fn storage(&self) -> Option<Arc<dyn StorageEngine>>` | `crates/contextra-graph/src/csr/graph_write/mod.rs:132` |  |
| `pub fn estimate_memory(&self, presized: bool) -> MemoryEstimate` | `crates/contextra-graph/src/csr/graph_write/mod.rs:137` |  |
| `pub fn estimate_memory_bytes(&self) -> usize` | `crates/contextra-graph/src/csr/graph_write/mod.rs:143` |  |
| `pub fn estimate_compaction_peak_bytes(&self) -> usize` | `crates/contextra-graph/src/csr/graph_write/mod.rs:149` |  |
| `pub fn get_source_doc_id(&self, index: usize) -> Option<DocId>` | `crates/contextra-graph/src/csr/graph_write/mod.rs:155` |  |
| `pub fn source_doc_id_at(&self, from: EntityId, to: EntityId) -> Option<DocId>` | `crates/contextra-graph/src/csr/graph_write/mod.rs:161` |  |
| `pub fn edges_for_doc(&self, doc_id: DocId) -> Vec<(EntityId, EntityId)>` | `crates/contextra-graph/src/csr/graph_write/mod.rs:197` |  |
| `pub async fn tombstone_edge( &self, edge_id: crate::consistency_enforcement::EdgeId, tx: TxId, ) -> Result<()>` | `crates/contextra-graph/src/csr/graph_write/tombstone.rs:24` |  |
| `pub fn tombstone_hyperedge(&self, id: crate::hyperedge::HyperEdgeId, wal_tx: TxId) -> bool` | `crates/contextra-graph/src/csr/graph_write/tombstone.rs:84` |  |
| `pub fn thresholded_local_hfd( &self, seeds: &[EntityId], params: &TlHfdParams, ) -> Result<AHashMap<EntityId, f32>, TlHfdError>` | `crates/contextra-graph/src/tl_hfd/mod.rs:57` |  |

### Modul `csr::inner`

#### `mod compaction`
- Ort: `crates/contextra-graph/src/csr/inner/mod.rs:15`
```rust
pub mod compaction
```

#### `mod estimate`
- Ort: `crates/contextra-graph/src/csr/inner/mod.rs:16`
```rust
pub mod estimate
```

#### `mod reinforcement`
- Ort: `crates/contextra-graph/src/csr/inner/mod.rs:17`
```rust
pub mod reinforcement
```

#### `struct GraphInner`
- Ort: `crates/contextra-graph/src/csr/inner/mod.rs:33`
```rust
pub struct GraphInner
```
- Impl-Block `crates/contextra-graph/src/csr/inner/compaction.rs:20`
- Impl-Block `crates/contextra-graph/src/csr/inner/estimate.rs:48`
- Impl-Block (Trait `Default`) `crates/contextra-graph/src/csr/inner/mod.rs:74`
- Impl-Block `crates/contextra-graph/src/csr/inner/mod.rs:80`
- Impl-Block `crates/contextra-graph/src/csr/inner/reinforcement.rs:25`
- Impl-Block (Trait `crate::path_rag::PathGraph`) `crates/contextra-graph/src/ppr/mod.rs:264`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `reverse_map` | `Vec<EntityId>` | `crates/contextra-graph/src/csr/inner/mod.rs:35` |
| `entities` | `Vec<Entity>` | `crates/contextra-graph/src/csr/inner/mod.rs:36` |
| `communities_loaded` | `bool` | `crates/contextra-graph/src/csr/inner/mod.rs:38` |
| `offsets` | `Vec<usize>` | `crates/contextra-graph/src/csr/inner/mod.rs:40` |
| `targets` | `Vec<InternalIndex>` | `crates/contextra-graph/src/csr/inner/mod.rs:41` |
| `weights` | `Vec<f32>` | `crates/contextra-graph/src/csr/inner/mod.rs:42` |
| `tx_valid_froms` | `Vec<TxId>` | `crates/contextra-graph/src/csr/inner/mod.rs:43` |
| `tx_valid_tos` | `Vec<TxId>` | `crates/contextra-graph/src/csr/inner/mod.rs:44` |
| `business_valid_froms` | `Vec<i64>` | `crates/contextra-graph/src/csr/inner/mod.rs:45` |
| `business_valid_tos` | `Vec<i64>` | `crates/contextra-graph/src/csr/inner/mod.rs:46` |
| `source_doc_ids` | `Vec<DocId>` | `crates/contextra-graph/src/csr/inner/mod.rs:47` |
| `out_weight_sums` | `Vec<f32>` | `crates/contextra-graph/src/csr/inner/mod.rs:48` |
| `pending_edge_count` | `usize` | `crates/contextra-graph/src/csr/inner/mod.rs:57` |
| `is_dirty` | `bool` | `crates/contextra-graph/src/csr/inner/mod.rs:58` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn get_or_create_index(&mut self, id: EntityId) -> InternalIndex` | `crates/contextra-graph/src/csr/inner/compaction.rs:22` |  |
| `pub fn recompute_node_out_weight_sum(&mut self, node_idx: usize)` | `crates/contextra-graph/src/csr/inner/compaction.rs:47` |  |
| `pub fn compact(&mut self)` | `crates/contextra-graph/src/csr/inner/compaction.rs:191` |  |
| `pub fn new() -> Self` | `crates/contextra-graph/src/csr/inner/mod.rs:82` |  |

### Modul `csr::inner::estimate`

#### `struct MemoryEstimate`
- Ort: `crates/contextra-graph/src/csr/inner/estimate.rs:25`
```rust
pub struct MemoryEstimate
```
- Impl-Block `crates/contextra-graph/src/csr/inner/estimate.rs:30`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `shared_payload_bytes` | `usize` | `crates/contextra-graph/src/csr/inner/estimate.rs:26` |
| `private_bytes` | `usize` | `crates/contextra-graph/src/csr/inner/estimate.rs:27` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn total_bytes(&self) -> usize` | `crates/contextra-graph/src/csr/inner/estimate.rs:33` |  |

### Modul `csr::types`

#### `struct CsrEdge`
- Ort: `crates/contextra-graph/src/csr/types.rs:39`
```rust
pub struct CsrEdge
```
- Impl-Block `crates/contextra-graph/src/csr/types.rs:49`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `target` | `EntityId` | `crates/contextra-graph/src/csr/types.rs:40` |
| `weight` | `f32` | `crates/contextra-graph/src/csr/types.rs:41` |
| `edge_type` | `EdgeType` | `crates/contextra-graph/src/csr/types.rs:42` |
| `cooccurrence_weight` | `f32` | `crates/contextra-graph/src/csr/types.rs:44` |
| `traversal_weight` | `f32` | `crates/contextra-graph/src/csr/types.rs:46` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(target: EntityId, weight: f32) -> Self` | `crates/contextra-graph/src/csr/types.rs:51` |  |

#### `struct CsrGraphConfig`
- Ort: `crates/contextra-graph/src/csr/types.rs:96`
```rust
pub struct CsrGraphConfig
```
- Impl-Block (Trait `Default`) `crates/contextra-graph/src/csr/types.rs:102`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `rebuild_threshold` | `usize` | `crates/contextra-graph/src/csr/types.rs:97` |
| `max_compaction_peak_memory_mb` | `Option<usize>` | `crates/contextra-graph/src/csr/types.rs:98` |
| `resource_tracker` | `Option<Arc<ResourceTracker>>` | `crates/contextra-graph/src/csr/types.rs:99` |

#### `struct PersistedEdgePayload`
- Ort: `crates/contextra-graph/src/csr/types.rs:68`
```rust
pub struct PersistedEdgePayload
```

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `weight` | `f32` | `crates/contextra-graph/src/csr/types.rs:69` |
| `tx_valid_from` | `Option<TxId>` | `crates/contextra-graph/src/csr/types.rs:71` |
| `tx_valid_to` | `Option<TxId>` | `crates/contextra-graph/src/csr/types.rs:73` |
| `business_valid_from` | `Option<i64>` | `crates/contextra-graph/src/csr/types.rs:75` |
| `business_valid_to` | `Option<i64>` | `crates/contextra-graph/src/csr/types.rs:77` |
| `source_doc_id` | `Option<DocId>` | `crates/contextra-graph/src/csr/types.rs:79` |

#### `enum EdgeType`
- Ort: `crates/contextra-graph/src/csr/types.rs:31`
```rust
pub enum EdgeType
```

Varianten:

- `Default` — `crates/contextra-graph/src/csr/types.rs:33`

#### `const MAX_TRAVERSAL_HOPS`
- Ort: `crates/contextra-graph/src/csr/types.rs:88`
```rust
pub const MAX_TRAVERSAL_HOPS: u8 = 3
```

#### `const MAX_VISITED_NODES`
- Ort: `crates/contextra-graph/src/csr/types.rs:92`
```rust
pub const MAX_VISITED_NODES: usize = 10_000
```

#### `const SCORE_DECAY`
- Ort: `crates/contextra-graph/src/csr/types.rs:84`
```rust
pub const SCORE_DECAY: f32 = 0.7
```

### Modul `csr::visibility`

#### `fn is_edge_visible`
- Ort: `crates/contextra-graph/src/csr/visibility.rs:55`
```rust
pub fn is_edge_visible( tx_valid_from: Option<TxId>, tx_valid_to: Option<TxId>, as_of: TxId, ) -> bool
```

#### `fn is_edge_visible_bitemporal`
- Ort: `crates/contextra-graph/src/csr/visibility.rs:81`
```rust
pub fn is_edge_visible_bitemporal( tx_valid_from: Option<TxId>, tx_valid_to: Option<TxId>, as_of_tx: TxId, business_valid_from: Option<i64>, business_valid_to: Option<i64>, as_of_business: Option<i64>, ) -> bool
```

#### `fn is_edge_visible_business`
- Ort: `crates/contextra-graph/src/csr/visibility.rs:66`
```rust
pub fn is_edge_visible_business( business_valid_from: Option<i64>, business_valid_to: Option<i64>, business_as_of: i64, ) -> bool
```

#### `fn is_suspicious_tx_id`
- Ort: `crates/contextra-graph/src/csr/visibility.rs:47`
```rust
pub fn is_suspicious_tx_id(tx: TxId) -> bool
```

#### `const WALLCLOCK_TX_HEURISTIC_MIN`
- Ort: `crates/contextra-graph/src/csr/visibility.rs:26`
```rust
pub const WALLCLOCK_TX_HEURISTIC_MIN: u64 = 1_400_000_000_000_000_000
```

### Modul `edge_reinforcement`

#### `struct EdgeReinforcementConfig`
- Ort: `crates/contextra-graph/src/edge_reinforcement.rs:17`
```rust
pub struct EdgeReinforcementConfig
```
- Impl-Block (Trait `Default`) `crates/contextra-graph/src/edge_reinforcement.rs:26`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `eta` | `f32` | `crates/contextra-graph/src/edge_reinforcement.rs:18` |
| `delta` | `f32` | `crates/contextra-graph/src/edge_reinforcement.rs:19` |
| `w_max` | `f32` | `crates/contextra-graph/src/edge_reinforcement.rs:20` |
| `rho` | `f32` | `crates/contextra-graph/src/edge_reinforcement.rs:21` |
| `q` | `f32` | `crates/contextra-graph/src/edge_reinforcement.rs:22` |
| `alpha` | `f32` | `crates/contextra-graph/src/edge_reinforcement.rs:23` |

#### `fn apply_cooccurrence_reinforcement`
- Ort: `crates/contextra-graph/src/edge_reinforcement.rs:146`
```rust
pub fn apply_cooccurrence_reinforcement( edge: &mut CsrEdge, co_activation: f32, config: &EdgeReinforcementConfig, ) -> bool
```

#### `fn apply_traversal_reinforcement`
- Ort: `crates/contextra-graph/src/edge_reinforcement.rs:157`
```rust
pub fn apply_traversal_reinforcement( edge: &mut CsrEdge, path_length: usize, config: &EdgeReinforcementConfig, )
```

#### `fn apply_weight_normalization`
- Ort: `crates/contextra-graph/src/edge_reinforcement.rs:171`
```rust
pub fn apply_weight_normalization(outgoing_edges: &mut [CsrEdge], w_max: f32)
```

#### `fn compute_edge_weight`
- Ort: `crates/contextra-graph/src/edge_reinforcement.rs:141`
```rust
pub fn compute_edge_weight(cooccurrence_weight: f32, traversal_weight: f32, alpha: f32) -> f32
```

### Modul `edge_reinforcement_buffer`

#### `struct CooccurrenceSignal`
- Ort: `crates/contextra-graph/src/edge_reinforcement_buffer.rs:31`
```rust
pub struct CooccurrenceSignal
```

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `from` | `EntityId` | `crates/contextra-graph/src/edge_reinforcement_buffer.rs:32` |
| `to` | `EntityId` | `crates/contextra-graph/src/edge_reinforcement_buffer.rs:33` |
| `co_activation` | `f32` | `crates/contextra-graph/src/edge_reinforcement_buffer.rs:34` |

#### `struct EdgeReinforcementBuffer`
- Ort: `crates/contextra-graph/src/edge_reinforcement_buffer.rs:55`
```rust
pub struct EdgeReinforcementBuffer
```
- Impl-Block `crates/contextra-graph/src/edge_reinforcement_buffer.rs:61`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new() -> Self` | `crates/contextra-graph/src/edge_reinforcement_buffer.rs:63` |  |
| `pub fn push_cooccurrence(&self, from: EntityId, to: EntityId, co_activation: f32)` | `crates/contextra-graph/src/edge_reinforcement_buffer.rs:70` |  |
| `pub fn push_traversal(&self, from: EntityId, to: EntityId, path_length: usize)` | `crates/contextra-graph/src/edge_reinforcement_buffer.rs:80` |  |
| `pub fn cooccurrence_count(&self) -> usize` | `crates/contextra-graph/src/edge_reinforcement_buffer.rs:90` |  |
| `pub fn traversal_count(&self) -> usize` | `crates/contextra-graph/src/edge_reinforcement_buffer.rs:96` |  |
| `pub fn flush_to_graph(&self, graph: &CsrGraph, config: &EdgeReinforcementConfig)` | `crates/contextra-graph/src/edge_reinforcement_buffer.rs:105` |  |

#### `struct TraversalSignal`
- Ort: `crates/contextra-graph/src/edge_reinforcement_buffer.rs:41`
```rust
pub struct TraversalSignal
```

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `from` | `EntityId` | `crates/contextra-graph/src/edge_reinforcement_buffer.rs:42` |
| `to` | `EntityId` | `crates/contextra-graph/src/edge_reinforcement_buffer.rs:43` |
| `path_length` | `usize` | `crates/contextra-graph/src/edge_reinforcement_buffer.rs:44` |

### Modul `entity_extraction`

#### `struct ExtractedRelation`
- Ort: `crates/contextra-graph/src/entity_extraction.rs:30`
```rust
pub struct ExtractedRelation
```
- Impl-Block `crates/contextra-graph/src/entity_extraction.rs:41`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `source` | `String` | `crates/contextra-graph/src/entity_extraction.rs:32` |
| `target` | `String` | `crates/contextra-graph/src/entity_extraction.rs:34` |
| `relation_type` | `String` | `crates/contextra-graph/src/entity_extraction.rs:36` |
| `confidence` | `f32` | `crates/contextra-graph/src/entity_extraction.rs:38` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new( source: impl Into<String>, target: impl Into<String>, relation_type: impl Into<String>, confidence: f32, ) -> Self` | `crates/contextra-graph/src/entity_extraction.rs:44` |  |

#### `struct RuleBasedEntityExtractor`
- Ort: `crates/contextra-graph/src/entity_extraction.rs:89`
```rust
pub struct RuleBasedEntityExtractor
```
- Impl-Block `crates/contextra-graph/src/entity_extraction.rs:96`
- Impl-Block (Trait `Default`) `crates/contextra-graph/src/entity_extraction.rs:182`
- Impl-Block (Trait `EntityExtractor`) `crates/contextra-graph/src/entity_extraction.rs:188`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(min_confidence: f32) -> Self` | `crates/contextra-graph/src/entity_extraction.rs:99` |  |

#### `trait EntityExtractor`
- Ort: `crates/contextra-graph/src/entity_extraction.rs:67`
```rust
pub trait EntityExtractor: Send + Sync
```

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn extract(&self, text: &str) -> Vec<ExtractedRelation>` | `crates/contextra-graph/src/entity_extraction.rs:69` | Extrahiert Relationen aus dem gegebenen Freitext. |

#### `fn ingest_extracted_relations`
- Ort: `crates/contextra-graph/src/entity_extraction.rs:254`
```rust
pub fn ingest_extracted_relations( graph: &CsrGraph, relations: &[ExtractedRelation], min_confidence_threshold: f32, doc_id: Option<DocId>, ) -> Result<Vec<HyperEdge>>
```

### Modul `error`

#### `enum GraphError`
- Ort: `crates/contextra-graph/src/error.rs:22`
```rust
pub enum GraphError
```
- Impl-Block (Trait `From<ContextraError>`) `crates/contextra-graph/src/compat/error.rs:21`
- Impl-Block (Trait `From<std::num::ParseIntError>`) `crates/contextra-graph/src/error.rs:56`
- Impl-Block (Trait `From<contextra_ids::IdError>`) `crates/contextra-graph/src/error.rs:62`

Varianten:

- `InvalidInput(String)` — `crates/contextra-graph/src/error.rs:25`
- `NotFound(String)` — `crates/contextra-graph/src/error.rs:29`
- `PolicyViolation(String)` — `crates/contextra-graph/src/error.rs:33`
- `Serialization(String)` — `crates/contextra-graph/src/error.rs:37`
- `Storage(String)` — `crates/contextra-graph/src/error.rs:41`
- `BudgetExhausted(String)` — `crates/contextra-graph/src/error.rs:45`
- `Mutation(#[from] GraphMutationError)` — `crates/contextra-graph/src/error.rs:49`
- `Internal(String)` — `crates/contextra-graph/src/error.rs:53`

#### `enum GraphMutationError`
- Ort: `crates/contextra-graph/src/error.rs:72`
```rust
pub enum GraphMutationError
```
- Impl-Block (Trait `From<HyperedgeSuggestError>`) `crates/contextra-graph/src/hyperedge_suggest.rs:61`

Varianten:

- `LockAcquisitionTimeout(String)` — `crates/contextra-graph/src/error.rs:75`
- `RoleBindingInvalid(String)` — `crates/contextra-graph/src/error.rs:79`
- `EpochReclamationPending(u64)` — `crates/contextra-graph/src/error.rs:83`
- `InsufficientParticipants{` — `crates/contextra-graph/src/error.rs:89`
- `HyperedgeNotFound(u64)` — `crates/contextra-graph/src/error.rs:98`
- `InvalidWeight{` — `crates/contextra-graph/src/error.rs:102`
- `DuplicateHyperEdgeId(u64)` — `crates/contextra-graph/src/error.rs:111`
- `Internal(String)` — `crates/contextra-graph/src/error.rs:115`

#### `type Result`
- Ort: `crates/contextra-graph/src/error.rs:16`
```rust
pub type Result<T, E = GraphError> = std::result::Result<T, E>
```

### Modul `graph_view`

#### `trait GraphView`
- Ort: `crates/contextra-graph/src/graph_view.rs:7`
```rust
pub trait GraphView: Send + Sync
```

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn neighbors<'a>(&'a self, entity: EntityId) -> GraphFuture<'a, Result<Vec<EntityId>>>` | `crates/contextra-graph/src/graph_view.rs:8` |  |
| `fn personalized_page_rank<'a>( &'a self, seeds: &'a [EntityId], config: &'a PprConfig, ) -> GraphFuture<'a, Result<Vec<(EntityId, f32)>>>` | `crates/contextra-graph/src/graph_view.rs:9` |  |

### Modul `hyperedge`

#### `struct ConsolidationNodesGuard`
- Ort: `crates/contextra-graph/src/hyperedge.rs:321`
```rust
pub struct ConsolidationNodesGuard
```
- Impl-Block `crates/contextra-graph/src/hyperedge.rs:325`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn acquire(entities: &[EntityId]) -> Self` | `crates/contextra-graph/src/hyperedge.rs:329` |  |
| `pub fn try_acquire(entities: &[EntityId]) -> std::result::Result<Self, GraphMutationError>` | `crates/contextra-graph/src/hyperedge.rs:339` |  |
| `pub fn entities(&self) -> &[EntityId]` | `crates/contextra-graph/src/hyperedge.rs:347` |  |
| `pub fn with_entities<F, R>(&self, f: F) -> R where F: FnOnce(&[EntityId]) -> R,` | `crates/contextra-graph/src/hyperedge.rs:353` |  |

#### `struct HyperEdge`
- Ort: `crates/contextra-graph/src/hyperedge.rs:53`
```rust
pub struct HyperEdge
```
- Impl-Block `crates/contextra-graph/src/hyperedge.rs:103`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `id` | `HyperEdgeId` | `crates/contextra-graph/src/hyperedge.rs:55` |
| `predicate` | `EdgeType` | `crates/contextra-graph/src/hyperedge.rs:57` |
| `participants` | `Arc<[RoleBinding]>` | `crates/contextra-graph/src/hyperedge.rs:62` |
| `weight` | `f32` | `crates/contextra-graph/src/hyperedge.rs:64` |
| `tx_valid_from` | `Option<TxId>` | `crates/contextra-graph/src/hyperedge.rs:67` |
| `tx_valid_to` | `Option<TxId>` | `crates/contextra-graph/src/hyperedge.rs:70` |
| `business_valid_from` | `Option<i64>` | `crates/contextra-graph/src/hyperedge.rs:73` |
| `business_valid_to` | `Option<i64>` | `crates/contextra-graph/src/hyperedge.rs:76` |
| `source_doc_id` | `Option<DocId>` | `crates/contextra-graph/src/hyperedge.rs:79` |
| `child_edge_ids` | `Arc<[HyperEdgeId]>` | `crates/contextra-graph/src/hyperedge.rs:82` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new( id: HyperEdgeId, predicate: EdgeType, participants: impl Into<Arc<[RoleBinding]>>, weight: f32, ) -> Self` | `crates/contextra-graph/src/hyperedge.rs:106` |  |
| `pub fn with_tx_validity(mut self, valid_from: Option<TxId>, valid_to: Option<TxId>) -> Self` | `crates/contextra-graph/src/hyperedge.rs:128` |  |
| `pub fn with_business_validity( mut self, valid_from: Option<i64>, valid_to: Option<i64>, ) -> Self` | `crates/contextra-graph/src/hyperedge.rs:136` |  |
| `pub fn with_source_doc_id(mut self, source_doc_id: Option<DocId>) -> Self` | `crates/contextra-graph/src/hyperedge.rs:148` |  |
| `pub fn with_child_edge_ids(mut self, ids: impl Into<Arc<[HyperEdgeId]>>) -> Self` | `crates/contextra-graph/src/hyperedge.rs:157` |  |
| `pub fn validate(&self) -> std::result::Result<(), GraphMutationError>` | `crates/contextra-graph/src/hyperedge.rs:177` |  |
| `pub fn serialize(&self) -> Result<Vec<u8>>` | `crates/contextra-graph/src/hyperedge.rs:201` |  |
| `pub fn deserialize(bytes: &[u8]) -> Result<Self>` | `crates/contextra-graph/src/hyperedge.rs:209` |  |
| `pub fn view(&self) -> HyperEdgeView<'_>` | `crates/contextra-graph/src/hyperedge.rs:232` |  |

#### `struct HyperEdgeView`
- Ort: `crates/contextra-graph/src/hyperedge.rs:240`
```rust
pub struct HyperEdgeView<'a>
```
- Impl-Block `crates/contextra-graph/src/hyperedge.rs:245`
- Impl-Block (Trait `Deref`) `crates/contextra-graph/src/hyperedge.rs:265`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `edge` | `&'a HyperEdge` | `crates/contextra-graph/src/hyperedge.rs:241` |
| `participants` | `ArcSlice<RoleBinding>` | `crates/contextra-graph/src/hyperedge.rs:242` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(edge: &'a HyperEdge) -> Self` | `crates/contextra-graph/src/hyperedge.rs:248` |  |
| `pub fn slice_participants( &self, range: impl std::ops::RangeBounds<usize>, ) -> ArcSlice<RoleBinding>` | `crates/contextra-graph/src/hyperedge.rs:257` |  |

#### `struct RoleInterner`
- Ort: `crates/contextra-graph/src/hyperedge.rs:280`
```rust
pub struct RoleInterner
```
- Impl-Block (Trait `Default`) `crates/contextra-graph/src/hyperedge.rs:286`
- Impl-Block `crates/contextra-graph/src/hyperedge.rs:361`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new() -> Self` | `crates/contextra-graph/src/hyperedge.rs:364` |  |
| `pub fn get_or_intern(&self, name: &str) -> RoleId` | `crates/contextra-graph/src/hyperedge.rs:377` |  |
| `pub fn resolve<F, R>(&self, id: RoleId, f: F) -> Option<R> where F: FnOnce(&str) -> R,` | `crates/contextra-graph/src/hyperedge.rs:398` |  |
| `pub fn resolve_string(&self, id: RoleId) -> Option<String>` | `crates/contextra-graph/src/hyperedge.rs:407` |  |
| `pub fn contains_role(&self, name: &str) -> bool` | `crates/contextra-graph/src/hyperedge.rs:413` |  |
| `pub fn contains_id(&self, id: RoleId) -> bool` | `crates/contextra-graph/src/hyperedge.rs:419` |  |
| `pub fn len(&self) -> usize` | `crates/contextra-graph/src/hyperedge.rs:425` |  |
| `pub fn is_empty(&self) -> bool` | `crates/contextra-graph/src/hyperedge.rs:431` |  |

#### `fn sort_dedup_entities`
- Ort: `crates/contextra-graph/src/hyperedge.rs:298`
```rust
pub fn sort_dedup_entities(entities: &[EntityId]) -> Vec<EntityId>
```

#### `fn star_weight`
- Ort: `crates/contextra-graph/src/hyperedge.rs:39`
```rust
pub fn star_weight(weight: f32, cardinality: usize) -> f32
```

#### `const HYPEREDGE_BY_ENTITY_PREFIX`
- Ort: `crates/contextra-graph/src/hyperedge.rs:31`
```rust
pub const HYPEREDGE_BY_ENTITY_PREFIX: &str = ""
```

#### `const HYPEREDGE_PREFIX`
- Ort: `crates/contextra-graph/src/hyperedge.rs:27`
```rust
pub const HYPEREDGE_PREFIX: &str = ""
```

### Modul `hyperedge_suggest`

#### `struct HyperEdgeCandidate`
- Ort: `crates/contextra-graph/src/hyperedge_suggest.rs:29`
```rust
pub struct HyperEdgeCandidate
```

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `participants` | `Vec<EntityId>` | `crates/contextra-graph/src/hyperedge_suggest.rs:31` |
| `co_occurrence_count` | `usize` | `crates/contextra-graph/src/hyperedge_suggest.rs:33` |
| `source_doc_ids` | `Vec<DocId>` | `crates/contextra-graph/src/hyperedge_suggest.rs:35` |

#### `struct ValidatedHyperEdgeCandidate`
- Ort: `crates/contextra-graph/src/hyperedge_suggest.rs:41`
```rust
pub struct ValidatedHyperEdgeCandidate
```

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `candidate` | `HyperEdgeCandidate` | `crates/contextra-graph/src/hyperedge_suggest.rs:43` |
| `predicate` | `String` | `crates/contextra-graph/src/hyperedge_suggest.rs:45` |
| `llm_confidence` | `f32` | `crates/contextra-graph/src/hyperedge_suggest.rs:47` |
| `accepted` | `bool` | `crates/contextra-graph/src/hyperedge_suggest.rs:49` |

#### `enum HyperedgeSuggestError`
- Ort: `crates/contextra-graph/src/hyperedge_suggest.rs:55`
```rust
pub enum HyperedgeSuggestError
```

Varianten:

- `LlmValidationBudgetExceeded(usize)` — `crates/contextra-graph/src/hyperedge_suggest.rs:58`

#### `fn compute_co_occurrence_candidates`
- Ort: `crates/contextra-graph/src/hyperedge_suggest.rs:77`
```rust
pub fn compute_co_occurrence_candidates( entities_per_document: &[(DocId, Vec<EntityId>)], min_co_occurrence: usize, ) -> Vec<HyperEdgeCandidate>
```

#### `fn validate_candidates_with_llm`
- Ort: `crates/contextra-graph/src/hyperedge_suggest.rs:129`
```rust
pub async fn validate_candidates_with_llm( candidates: &[HyperEdgeCandidate], entity_labels: &dyn Fn(EntityId) -> Option<String>, generator: &dyn LlmTextGenerator, max_llm_calls_per_cycle: usize, ) -> Result<Vec<ValidatedHyperEdgeCandidate>, GraphMutationError>
```
Dokumentation: Validates hyperedge candidates using an [`LlmTextGenerator`]. For each candidate (up to `max_llm_calls_per_cycle`), queries the LLM to confirm if the joint occurrence implies a semantically meaningful relation and suggest a predicate. # Budget Guard If `candidates.len() > max_llm_calls_per_cycle`, returns `Err(GraphMutationError)` wrapping `HyperedgeSuggestError::LlmValidationBudgetExceeded`.
- Gate: `#[cfg(feature = "workspace")]`

#### `const MAX_RELATE_PARTICIPANTS`
- Ort: `crates/contextra-graph/src/hyperedge_suggest.rs:24`
```rust
pub const MAX_RELATE_PARTICIPANTS: usize = 64
```

### Modul `path_rag`

#### `mod k_path`
- Ort: `crates/contextra-graph/src/path_rag/mod.rs:23`
```rust
pub mod k_path
```

#### `mod snapshot`
- Ort: `crates/contextra-graph/src/path_rag/mod.rs:28`
```rust
pub mod snapshot
```

#### `struct GraphPath`
- Ort: `crates/contextra-graph/src/path_rag/mod.rs:42`
```rust
pub struct GraphPath
```

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `nodes` | `Vec<EntityId>` | `crates/contextra-graph/src/path_rag/mod.rs:44` |
| `edge_weights` | `Vec<f32>` | `crates/contextra-graph/src/path_rag/mod.rs:46` |
| `confidence` | `f64` | `crates/contextra-graph/src/path_rag/mod.rs:48` |
| `total_flow` | `f32` | `crates/contextra-graph/src/path_rag/mod.rs:50` |

#### `struct PathRAGConfig`
- Ort: `crates/contextra-graph/src/path_rag/mod.rs:180`
```rust
pub struct PathRAGConfig
```
- Impl-Block (Trait `Default`) `crates/contextra-graph/src/path_rag/mod.rs:191`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `max_hops` | `usize` | `crates/contextra-graph/src/path_rag/mod.rs:182` |
| `sufficiency_threshold` | `f64` | `crates/contextra-graph/src/path_rag/mod.rs:184` |
| `hyperedge_expansion_enabled` | `bool` | `crates/contextra-graph/src/path_rag/mod.rs:186` |
| `hyperedge_weight_discount` | `f32` | `crates/contextra-graph/src/path_rag/mod.rs:188` |

#### `struct PathRAGEngine`
- Ort: `crates/contextra-graph/src/path_rag/mod.rs:203`
```rust
pub struct PathRAGEngine<G: PathGraph>
```
- Impl-Block (Trait `KPathDiffusion`) `crates/contextra-graph/src/path_rag/k_path.rs:102`
- Impl-Block `crates/contextra-graph/src/path_rag/mod.rs:208`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `config` | `PathRAGConfig` | `crates/contextra-graph/src/path_rag/mod.rs:205` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(graph: G, max_hops: usize, sufficiency_threshold: f64) -> Self` | `crates/contextra-graph/src/path_rag/mod.rs:210` |  |
| `pub fn with_defaults(graph: G) -> Self` | `crates/contextra-graph/src/path_rag/mod.rs:223` |  |
| `pub fn with_config(graph: G, config: PathRAGConfig) -> Self` | `crates/contextra-graph/src/path_rag/mod.rs:228` |  |
| `pub fn max_hops(&self) -> usize` | `crates/contextra-graph/src/path_rag/mod.rs:233` |  |
| `pub fn sufficiency_threshold(&self) -> f64` | `crates/contextra-graph/src/path_rag/mod.rs:238` |  |
| `pub fn find_path(&self, source: EntityId, target: EntityId) -> Option<GraphPath>` | `crates/contextra-graph/src/path_rag/mod.rs:288` |  |
| `pub fn find_all_paths(&self, source: EntityId) -> Vec<GraphPath>` | `crates/contextra-graph/src/path_rag/mod.rs:420` |  |
| `pub fn sufficiency_check(&self, path: &GraphPath) -> bool` | `crates/contextra-graph/src/path_rag/mod.rs:450` |  |
| `pub fn to_rrf_signal(&self, paths: &[GraphPath]) -> Vec<(DocId, f32)>` | `crates/contextra-graph/src/path_rag/mod.rs:460` |  |

#### `struct PprParams`
- Ort: `crates/contextra-graph/src/path_rag/mod.rs:82`
```rust
pub struct PprParams
```
- Impl-Block (Trait `Default`) `crates/contextra-graph/src/path_rag/mod.rs:91`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `alpha` | `f32` | `crates/contextra-graph/src/path_rag/mod.rs:84` |
| `epsilon` | `f32` | `crates/contextra-graph/src/path_rag/mod.rs:86` |
| `hyperedge_decay` | `f32` | `crates/contextra-graph/src/path_rag/mod.rs:88` |

#### `trait PathGraph`
- Ort: `crates/contextra-graph/src/path_rag/mod.rs:56`
```rust
pub trait PathGraph: Send + Sync
```

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn neighbors_with_weights(&self, node: EntityId) -> Vec<(EntityId, f32)>` | `crates/contextra-graph/src/path_rag/mod.rs:57` |  |
| `fn predecessors_with_weights(&self, node: EntityId) -> Vec<(EntityId, f32)>` | `crates/contextra-graph/src/path_rag/mod.rs:58` |  |
| `fn hyperedges_for_entity(&self, _node: EntityId) -> Vec<HyperEdgeId>` | `crates/contextra-graph/src/path_rag/mod.rs:61` | Returns hyperedge IDs associated with the given entity (default: empty). |
| `fn get_hyperedge(&self, _id: HyperEdgeId) -> Option<Arc<HyperEdge>>` | `crates/contextra-graph/src/path_rag/mod.rs:66` | Resolves a HyperEdgeId to its full HyperEdge details (default: None). |
| `fn hyperedge_participants(&self, id: HyperEdgeId) -> Arc<[RoleBinding]>` | `crates/contextra-graph/src/path_rag/mod.rs:71` | Returns participant role bindings for a given hyperedge ID (default: resolves via get_hyperedge). |

#### `fn forward_push_ppr`
- Ort: `crates/contextra-graph/src/path_rag/mod.rs:105`
```rust
pub fn forward_push_ppr<G: PathGraph>( graph: &G, seeds: &[EntityId], params: &PprParams, ) -> AHashMap<EntityId, f32>
```

#### `const DEFAULT_SUFFICIENCY_THRESHOLD`
- Ort: `crates/contextra-graph/src/path_rag/mod.rs:175`
```rust
pub const DEFAULT_SUFFICIENCY_THRESHOLD: f64 = 0.1
```

### Modul `path_rag::k_path`

#### `struct KPathConfig`
- Ort: `crates/contextra-graph/src/path_rag/k_path.rs:25`
```rust
pub struct KPathConfig
```
- Impl-Block (Trait `Default`) `crates/contextra-graph/src/path_rag/k_path.rs:34`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `k` | `usize` | `crates/contextra-graph/src/path_rag/k_path.rs:27` |
| `max_visited_nodes` | `usize` | `crates/contextra-graph/src/path_rag/k_path.rs:29` |
| `max_hops` | `usize` | `crates/contextra-graph/src/path_rag/k_path.rs:31` |

#### `struct KPathResult`
- Ort: `crates/contextra-graph/src/path_rag/k_path.rs:47`
```rust
pub struct KPathResult
```

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `paths` | `Vec<Vec<EntityId>>` | `crates/contextra-graph/src/path_rag/k_path.rs:49` |
| `budget_exhausted` | `bool` | `crates/contextra-graph/src/path_rag/k_path.rs:51` |

#### `trait KPathDiffusion`
- Ort: `crates/contextra-graph/src/path_rag/k_path.rs:56`
```rust
pub trait KPathDiffusion: Send + Sync
```

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn find_k_paths( &self, source: EntityId, target: EntityId, config: &KPathConfig, ) -> Result<KPathResult>` | `crates/contextra-graph/src/path_rag/k_path.rs:60` | Discovers up to `k` paths between `source` and `target` bounded by `config`. Budget exhaustion (`budget_exhausted: true`) is returned as a regular `Ok(KPathResu |

### Modul `path_rag::snapshot`

#### `struct SnapshotPathGraph`
- Ort: `crates/contextra-graph/src/path_rag/snapshot.rs:23`
```rust
pub struct SnapshotPathGraph<'a>
```
- Impl-Block `crates/contextra-graph/src/path_rag/snapshot.rs:28`
- Impl-Block (Trait `PathGraph`) `crates/contextra-graph/src/path_rag/snapshot.rs:36`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(inner: &'a GraphInner, as_of_tx: TxId) -> Self` | `crates/contextra-graph/src/path_rag/snapshot.rs:31` |  |

### Modul `percolation`

#### `struct PercolationConfig`
- Ort: `crates/contextra-graph/src/percolation.rs:27`
```rust
pub struct PercolationConfig
```
- Impl-Block (Trait `Default`) `crates/contextra-graph/src/percolation.rs:36`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `critical_threshold` | `f32` | `crates/contextra-graph/src/percolation.rs:29` |
| `rebonding_similarity` | `f32` | `crates/contextra-graph/src/percolation.rs:31` |
| `max_new_edges_per_pass` | `usize` | `crates/contextra-graph/src/percolation.rs:33` |

#### `fn compute_percolation_health`
- Ort: `crates/contextra-graph/src/percolation.rs:50`
```rust
pub fn compute_percolation_health( active_node_count: usize, active_edge_count: usize, ) -> Option<f32>
```

#### `fn find_rebonding_candidates`
- Ort: `crates/contextra-graph/src/percolation.rs:89`
```rust
pub async fn find_rebonding_candidates<G: contextra_ports::GraphIndex>( graph: &G, embeddings: &HashMap<EntityId, Vec<f32>>, config: &PercolationConfig, ) -> Vec<(EntityId, EntityId, f32)>
```
Dokumentation: Findet Kandidaten-Paare für Re-Bonding. Gibt `(EntityId, EntityId, similarity)`-Tupel zurück, sortiert absteigend nach Similarity.
- Gate: `#[cfg(feature = "workspace")]`

#### `fn should_trigger_rebonding`
- Ort: `crates/contextra-graph/src/percolation.rs:63`
```rust
pub fn should_trigger_rebonding(health: f32, config: &PercolationConfig) -> bool
```

### Modul `ppr`

#### `mod cost`
- Ort: `crates/contextra-graph/src/ppr/mod.rs:15`
```rust
pub mod cost
```

#### `mod dense`
- Ort: `crates/contextra-graph/src/ppr/mod.rs:16`
```rust
pub mod dense
```

#### `mod forward_push`
- Ort: `crates/contextra-graph/src/ppr/mod.rs:17`
```rust
pub mod forward_push
```

#### `mod shadow_hook`
- Ort: `crates/contextra-graph/src/ppr/mod.rs:18`
```rust
pub mod shadow_hook
```

#### `mod types`
- Ort: `crates/contextra-graph/src/ppr/mod.rs:20`
```rust
pub mod types
```

#### `fn compute_ppr`
- Ort: `crates/contextra-graph/src/ppr/mod.rs:30`
```rust
pub fn compute_ppr( inner: &GraphInner, seed_nodes: &[EntityId], config: &PprConfig, deleted_nodes: &DeletedView, ) -> Vec<(EntityId, f32)>
```

#### `fn compute_ppr_with_apprh_selector`
- Ort: `crates/contextra-graph/src/ppr/mod.rs:42`
```rust
pub fn compute_ppr_with_apprh_selector( inner: &GraphInner, seed_nodes: &[EntityId], config: &PprConfig, deleted_nodes: &DeletedView, ctx: &mut PprContext, selector: &crate::apprh::ApprhSelector, apprh_params: &crate::apprh::ApprhParams, ) -> Vec<(EntityId, f32)>
```

### Modul `ppr::cost`

#### `struct GraphStats`
- Ort: `crates/contextra-graph/src/ppr/cost.rs:19`
```rust
pub struct GraphStats
```
- Impl-Block `crates/contextra-graph/src/ppr/cost.rs:28`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `node_count` | `usize` | `crates/contextra-graph/src/ppr/cost.rs:21` |
| `edge_count` | `usize` | `crates/contextra-graph/src/ppr/cost.rs:23` |
| `avg_degree` | `f64` | `crates/contextra-graph/src/ppr/cost.rs:25` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(node_count: usize, edge_count: usize) -> Self` | `crates/contextra-graph/src/ppr/cost.rs:31` |  |

#### `struct PprCostCalibrator`
- Ort: `crates/contextra-graph/src/ppr/cost.rs:85`
```rust
pub struct PprCostCalibrator
```
- Impl-Block `crates/contextra-graph/src/ppr/cost.rs:89`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new() -> Self` | `crates/contextra-graph/src/ppr/cost.rs:92` |  |
| `pub fn record(&self, sample: PprCostSample)` | `crates/contextra-graph/src/ppr/cost.rs:98` |  |
| `pub fn sample_count(&self) -> usize` | `crates/contextra-graph/src/ppr/cost.rs:105` |  |
| `pub fn recommended_multiplier(&self) -> Option<f64>` | `crates/contextra-graph/src/ppr/cost.rs:114` |  |
| `pub fn clear(&self)` | `crates/contextra-graph/src/ppr/cost.rs:140` |  |

#### `struct PprCostEstimate`
- Ort: `crates/contextra-graph/src/ppr/cost.rs:48`
```rust
pub struct PprCostEstimate
```
- Impl-Block `crates/contextra-graph/src/ppr/cost.rs:55`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `forward_push` | `f64` | `crates/contextra-graph/src/ppr/cost.rs:50` |
| `power_iteration` | `f64` | `crates/contextra-graph/src/ppr/cost.rs:52` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn cheapest_strategy(&self) -> PprAlgorithm` | `crates/contextra-graph/src/ppr/cost.rs:60` |  |

#### `struct PprCostSample`
- Ort: `crates/contextra-graph/src/ppr/cost.rs:72`
```rust
pub struct PprCostSample
```

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `algorithm` | `PprAlgorithm` | `crates/contextra-graph/src/ppr/cost.rs:74` |
| `estimated_cost` | `f64` | `crates/contextra-graph/src/ppr/cost.rs:76` |
| `measured_edge_accesses` | `u64` | `crates/contextra-graph/src/ppr/cost.rs:78` |

#### `fn estimate_ppr_cost`
- Ort: `crates/contextra-graph/src/ppr/cost.rs:151`
```rust
pub fn estimate_ppr_cost( stats: Option<&GraphStats>, seed_count: usize, config: &PprConfig, ) -> (PprCostEstimate, PprAlgorithm)
```

### Modul `ppr::shadow_hook`

#### `struct ShadowDiscrepancy`
- Ort: `crates/contextra-graph/src/ppr/shadow_hook.rs:17`
```rust
pub struct ShadowDiscrepancy<T>
```

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `baseline` | `T` | `crates/contextra-graph/src/ppr/shadow_hook.rs:19` |
| `candidate` | `T` | `crates/contextra-graph/src/ppr/shadow_hook.rs:21` |
| `context_id` | `u64` | `crates/contextra-graph/src/ppr/shadow_hook.rs:23` |

#### `trait ShadowSink`
- Ort: `crates/contextra-graph/src/ppr/shadow_hook.rs:27`
```rust
pub trait ShadowSink<T>: Send + Sync
```
Dokumentation: Dyn-kompatibler Port für Diskrepanz-Senken.

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn record(&self, discrepancy: ShadowDiscrepancy<T>)` | `crates/contextra-graph/src/ppr/shadow_hook.rs:29` | Zeichnet einen Diskrepanz-Eintrag auf. |

#### `fn log_tl_hfd_vs_baseline_discrepancy`
- Ort: `crates/contextra-graph/src/ppr/shadow_hook.rs:35`
```rust
pub fn log_tl_hfd_vs_baseline_discrepancy( baseline: &AHashMap<EntityId, f32>, candidate: &AHashMap<EntityId, f32>, context_id: u64, sink: &dyn ShadowSink<f64>, )
```

### Modul `ppr::snapshot`

#### `fn compute_ppr_at`
- Ort: `crates/contextra-graph/src/ppr/snapshot.rs:107`
```rust
pub fn compute_ppr_at( inner: &GraphInner, seed_nodes: &[EntityId], config: &PprConfig, deleted_nodes: &DeletedView, as_of_tx: TxId, ) -> Vec<(EntityId, f32)>
```

### Modul `ppr::types`

#### `struct DeletedView`
- Ort: `crates/contextra-graph/src/ppr/types.rs:19`
```rust
pub struct DeletedView
```
- Impl-Block `crates/contextra-graph/src/ppr/types.rs:23`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn contains(&self, idx: usize) -> bool` | `crates/contextra-graph/src/ppr/types.rs:37` |  |
| `pub fn len(&self) -> usize` | `crates/contextra-graph/src/ppr/types.rs:43` |  |
| `pub fn is_empty(&self) -> bool` | `crates/contextra-graph/src/ppr/types.rs:49` |  |

#### `struct PprContext`
- Ort: `crates/contextra-graph/src/ppr/types.rs:56`
```rust
pub struct PprContext
```
- Impl-Block `crates/contextra-graph/src/ppr/types.rs:63`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `valid_seeds` | `Vec<usize>` | `crates/contextra-graph/src/ppr/types.rs:57` |
| `out_weight_sums` | `Vec<f32>` | `crates/contextra-graph/src/ppr/types.rs:58` |
| `ranks` | `Vec<f32>` | `crates/contextra-graph/src/ppr/types.rs:59` |
| `next_ranks` | `Vec<f32>` | `crates/contextra-graph/src/ppr/types.rs:60` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new() -> Self` | `crates/contextra-graph/src/ppr/types.rs:65` |  |

### Modul `ppr_stream`

#### `struct PprCandidateStream`
- Ort: `crates/contextra-graph/src/ppr_stream.rs:34`
```rust
pub struct PprCandidateStream<'a, G: GraphIndex + ?Sized>
```
- Impl-Block `crates/contextra-graph/src/ppr_stream.rs:44`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(graph: &'a G, seeds: impl Into<Vec<EntityId>>, config: PprConfig) -> Self` | `crates/contextra-graph/src/ppr_stream.rs:47` |  |
| `pub fn with_batch_size(mut self, n: usize) -> Self` | `crates/contextra-graph/src/ppr_stream.rs:61` |  |
| `pub fn with_exclude_seeds(mut self, exclude: bool) -> Self` | `crates/contextra-graph/src/ppr_stream.rs:68` |  |
| `pub async fn next_batch(&mut self) -> Result<Vec<(EntityId, f32)>>` | `crates/contextra-graph/src/ppr_stream.rs:75` | Fetches the next batch of candidate entity-score pairs (up to `batch_size`). Returns an empty `Vec` when exhausted. |
| `pub async fn next_entity(&mut self) -> Result<Option<EntityId>>` | `crates/contextra-graph/src/ppr_stream.rs:91` | Fetches the next single candidate entity ID. Returns `Ok(None)` when exhausted. |
| `pub async fn next_batch_docs<F>(&mut self, mut resolve: F) -> Result<Vec<DocId>> where F: FnMut(EntityId) -> Option<DocId>,` | `crates/contextra-graph/src/ppr_stream.rs:103` | Fetches the next batch of resolved [`DocId`]s (up to `batch_size`). Entities for which `resolve` returns `None` are skipped. |
| `pub fn yielded(&self) -> usize` | `crates/contextra-graph/src/ppr_stream.rs:125` |  |
| `pub fn is_exhausted(&self) -> bool` | `crates/contextra-graph/src/ppr_stream.rs:131` |  |
| `pub const fn max_depth() -> usize` | `crates/contextra-graph/src/ppr_stream.rs:140` |  |

#### `const DEFAULT_PPR_STREAM_BATCH_SIZE`
- Ort: `crates/contextra-graph/src/ppr_stream.rs:30`
```rust
pub const DEFAULT_PPR_STREAM_BATCH_SIZE: usize = 16
```

### Modul `provenance`

#### `struct DocEdgeIndex`
- Ort: `crates/contextra-graph/src/provenance.rs:49`
```rust
pub struct DocEdgeIndex
```
- Impl-Block `crates/contextra-graph/src/provenance.rs:53`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new() -> Self` | `crates/contextra-graph/src/provenance.rs:55` |  |
| `pub fn record(&self, doc_id: DocId, edge_id: EdgeId)` | `crates/contextra-graph/src/provenance.rs:63` |  |
| `pub fn record_provenance(&self, provenance: &EdgeProvenance)` | `crates/contextra-graph/src/provenance.rs:70` |  |
| `pub fn edges_for_doc(&self, doc_id: DocId) -> Vec<EdgeId>` | `crates/contextra-graph/src/provenance.rs:79` |  |
| `pub fn remove_doc(&self, doc_id: DocId)` | `crates/contextra-graph/src/provenance.rs:89` |  |

#### `struct EdgeProvenance`
- Ort: `crates/contextra-graph/src/provenance.rs:25`
```rust
pub struct EdgeProvenance
```
- Impl-Block `crates/contextra-graph/src/provenance.rs:34`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `edge_id` | `EdgeId` | `crates/contextra-graph/src/provenance.rs:27` |
| `source_doc_ids` | `Vec<DocId>` | `crates/contextra-graph/src/provenance.rs:29` |
| `created_at_tx` | `TxId` | `crates/contextra-graph/src/provenance.rs:31` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(edge_id: EdgeId, source_doc_ids: Vec<DocId>, created_at_tx: TxId) -> Self` | `crates/contextra-graph/src/provenance.rs:36` |  |

### Modul `session_dag`

#### `struct AgentStateNode`
- Ort: `crates/contextra-graph/src/session_dag.rs:116`
```rust
pub struct AgentStateNode
```

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `step_id` | `NodeIdx` | `crates/contextra-graph/src/session_dag.rs:117` |
| `prompt` | `String` | `crates/contextra-graph/src/session_dag.rs:119` |
| `response` | `String` | `crates/contextra-graph/src/session_dag.rs:121` |
| `snapshot_tx_id` | `Option<TxId>` | `crates/contextra-graph/src/session_dag.rs:123` |
| `tool_outputs` | `Vec<String>` | `crates/contextra-graph/src/session_dag.rs:125` |
| `compacted` | `bool` | `crates/contextra-graph/src/session_dag.rs:127` |

#### `struct DagEdge`
- Ort: `crates/contextra-graph/src/session_dag.rs:133`
```rust
pub struct DagEdge
```

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `parent` | `NodeIdx` | `crates/contextra-graph/src/session_dag.rs:134` |
| `child` | `NodeIdx` | `crates/contextra-graph/src/session_dag.rs:135` |
| `label` | `String` | `crates/contextra-graph/src/session_dag.rs:137` |

#### `struct NodesGuard`
- Ort: `crates/contextra-graph/src/session_dag.rs:35`
```rust
pub struct NodesGuard<'a>
```
- Impl-Block `crates/contextra-graph/src/session_dag.rs:40`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn nodes(&self) -> &HashMap<NodeIdx, AgentStateNode>` | `crates/contextra-graph/src/session_dag.rs:42` |  |
| `pub fn edges(&self) -> RwLockReadGuard<'_, Vec<DagEdge>>` | `crates/contextra-graph/src/session_dag.rs:49` |  |
| `pub fn edges_write(&self) -> RwLockWriteGuard<'_, Vec<DagEdge>>` | `crates/contextra-graph/src/session_dag.rs:54` |  |
| `pub fn active_head(&self) -> RwLockReadGuard<'_, NodeIdx>` | `crates/contextra-graph/src/session_dag.rs:59` |  |
| `pub fn active_head_write(&self) -> RwLockWriteGuard<'_, NodeIdx>` | `crates/contextra-graph/src/session_dag.rs:64` |  |

#### `struct NodesWriteGuard`
- Ort: `crates/contextra-graph/src/session_dag.rs:72`
```rust
pub struct NodesWriteGuard<'a>
```
- Impl-Block `crates/contextra-graph/src/session_dag.rs:77`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn nodes(&self) -> &HashMap<NodeIdx, AgentStateNode>` | `crates/contextra-graph/src/session_dag.rs:79` |  |
| `pub fn nodes_mut(&mut self) -> &mut HashMap<NodeIdx, AgentStateNode>` | `crates/contextra-graph/src/session_dag.rs:84` |  |
| `pub fn edges(&self) -> RwLockReadGuard<'_, Vec<DagEdge>>` | `crates/contextra-graph/src/session_dag.rs:89` |  |
| `pub fn edges_write(&self) -> RwLockWriteGuard<'_, Vec<DagEdge>>` | `crates/contextra-graph/src/session_dag.rs:94` |  |
| `pub fn active_head(&self) -> RwLockReadGuard<'_, NodeIdx>` | `crates/contextra-graph/src/session_dag.rs:99` |  |
| `pub fn active_head_write(&self) -> RwLockWriteGuard<'_, NodeIdx>` | `crates/contextra-graph/src/session_dag.rs:104` |  |

#### `struct SessionBranchTree`
- Ort: `crates/contextra-graph/src/session_dag.rs:166`
```rust
pub struct SessionBranchTree
```
- Impl-Block `crates/contextra-graph/src/session_dag.rs:173`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(root_prompt: String, root_response: String) -> Self` | `crates/contextra-graph/src/session_dag.rs:176` |  |
| `pub fn lock_nodes(&self) -> NodesGuard<'_>` | `crates/contextra-graph/src/session_dag.rs:199` |  |
| `pub fn lock_nodes_write(&self) -> NodesWriteGuard<'_>` | `crates/contextra-graph/src/session_dag.rs:209` |  |
| `pub fn append_step( &self, prompt: String, response: String, snapshot_tx_id: Option<TxId>, tool_outputs: Vec<String>, label: &str, ) -> Result<NodeIdx>` | `crates/contextra-graph/src/session_dag.rs:270` |  |
| `pub fn branch_from( &self, parent_node: NodeIdx, prompt: String, response: String, snapshot_tx_id: Option<TxId>, tool_outputs: Vec<String>, label: &str, ) -> Result<NodeIdx>` | `crates/contextra-graph/src/session_dag.rs:301` |  |
| `pub fn set_active_head(&self, node_idx: NodeIdx) -> Result<()>` | `crates/contextra-graph/src/session_dag.rs:326` |  |
| `pub fn path_to_head(&self) -> Vec<AgentStateNode>` | `crates/contextra-graph/src/session_dag.rs:342` |  |
| `pub fn children_of(&self, node_idx: NodeIdx) -> Vec<NodeIdx>` | `crates/contextra-graph/src/session_dag.rs:383` |  |
| `pub fn active_head(&self) -> NodeIdx` | `crates/contextra-graph/src/session_dag.rs:395` |  |
| `pub fn node_count(&self) -> usize` | `crates/contextra-graph/src/session_dag.rs:403` |  |
| `pub fn get_node(&self, node_idx: NodeIdx) -> Option<AgentStateNode>` | `crates/contextra-graph/src/session_dag.rs:409` |  |
| `pub async fn save<S: StorageEngine + ?Sized>( &self, storage: &S, namespace: &str, tx: TxId, ) -> Result<()>` | `crates/contextra-graph/src/session_dag.rs:415` | Persists all nodes, edges, active head, and next_id of the Session-DAG to the storage engine under the given namespace prefix. |
| `pub async fn load<S: StorageEngine + ?Sized>(storage: &S, namespace: &str) -> Result<Self>` | `crates/contextra-graph/src/session_dag.rs:464` | Loads and reconstructs a `SessionBranchTree` from storage for the specified namespace. |

#### `struct SessionDagStepOptions`
- Ort: `crates/contextra-graph/src/session_dag.rs:151`
```rust
pub struct SessionDagStepOptions<'a>
```

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `prompt` | `String` | `crates/contextra-graph/src/session_dag.rs:152` |
| `response` | `String` | `crates/contextra-graph/src/session_dag.rs:153` |
| `snapshot_tx_id` | `Option<TxId>` | `crates/contextra-graph/src/session_dag.rs:154` |
| `tool_outputs` | `Vec<String>` | `crates/contextra-graph/src/session_dag.rs:155` |
| `label` | `&'a str` | `crates/contextra-graph/src/session_dag.rs:156` |

#### `type NodeIdx`
- Ort: `crates/contextra-graph/src/session_dag.rs:111`
```rust
pub type NodeIdx = u64
```

#### `const MAX_DAG_STRING_BYTES`
- Ort: `crates/contextra-graph/src/session_dag.rs:142`
```rust
pub const MAX_DAG_STRING_BYTES: usize = 10 * 1024 * 1024
```

#### `const MAX_DAG_TRAVERSAL_DEPTH`
- Ort: `crates/contextra-graph/src/session_dag.rs:146`
```rust
pub const MAX_DAG_TRAVERSAL_DEPTH: usize = 10_000
```

### Modul `storage`

#### `mod conformance`
- Ort: `crates/contextra-graph/src/storage.rs:91`
```rust
pub mod conformance
```

#### `struct InMemoryGraphStorage`
- Ort: `crates/contextra-graph/src/storage.rs:26`
```rust
pub struct InMemoryGraphStorage
```
- Impl-Block `crates/contextra-graph/src/storage.rs:31`
- Impl-Block (Trait `GraphStorage`) `crates/contextra-graph/src/storage.rs:35`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new() -> Self` | `crates/contextra-graph/src/storage.rs:32` |  |

#### `trait GraphStorage`
- Ort: `crates/contextra-graph/src/storage.rs:15`
```rust
pub trait GraphStorage: Send + Sync + 'static
```

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn get<'a>(&'a self, key: &'a [u8]) -> GraphFuture<'a, Result<Option<Bytes>>>` | `crates/contextra-graph/src/storage.rs:16` |  |
| `fn put<'a>(&'a self, tx: TxId, key: &'a [u8], value: &'a [u8]) -> GraphFuture<'a, Result<()>>` | `crates/contextra-graph/src/storage.rs:17` |  |
| `fn delete<'a>(&'a self, tx: TxId, key: &'a [u8]) -> GraphFuture<'a, Result<()>>` | `crates/contextra-graph/src/storage.rs:18` |  |
| `fn scan_prefix<'a>(&'a self, prefix: &'a [u8]) -> GraphFuture<'a, Result<Vec<(Vec<u8>, Vec<u8>)>>>` | `crates/contextra-graph/src/storage.rs:19` |  |
| `fn commit<'a>(&'a self, tx: TxId) -> GraphFuture<'a, Result<()>>` | `crates/contextra-graph/src/storage.rs:20` |  |
| `fn flush<'a>(&'a self) -> GraphFuture<'a, Result<()>>` | `crates/contextra-graph/src/storage.rs:21` |  |
| `fn last_tx_id<'a>(&'a self) -> GraphFuture<'a, Result<TxId>>` | `crates/contextra-graph/src/storage.rs:22` |  |

#### `type GraphFuture`
- Ort: `crates/contextra-graph/src/storage.rs:13`
```rust
pub type GraphFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>
```

### Modul `storage::conformance`

#### `fn run_all`
- Ort: `crates/contextra-graph/src/storage.rs:94`
```rust
pub async fn run_all<S: GraphStorage>(storage: S)
```

### Modul `test_doubles`

#### `struct FaultConfig`
- Ort: `crates/contextra-graph/src/test_doubles.rs:33`
```rust
pub struct FaultConfig
```
Dokumentation: Fault configuration for crash simulation.

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `fail_all` | `bool` | `crates/contextra-graph/src/test_doubles.rs:35` |

#### `struct FaultVfs`
- Ort: `crates/contextra-graph/src/test_doubles.rs:26`
```rust
pub struct FaultVfs
```
- Impl-Block `crates/contextra-graph/src/test_doubles.rs:38`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new() -> Self` | `crates/contextra-graph/src/test_doubles.rs:40` | Creates a new `FaultVfs`. |
| `pub fn set_config(&self, config: FaultConfig)` | `crates/contextra-graph/src/test_doubles.rs:45` | Sets the fault injection config. |
| `pub fn trigger_crash(&self)` | `crates/contextra-graph/src/test_doubles.rs:50` | Triggers an immediate crash state. |
| `pub fn reset_counters(&self)` | `crates/contextra-graph/src/test_doubles.rs:55` | Resets fault counters and failure state. |
| `pub fn write_file( &self, path: &std::path::Path, data: &[u8], ) -> std::result::Result<(), String>` | `crates/contextra-graph/src/test_doubles.rs:60` | Writes data to a virtual path. |
| `pub fn read_file(&self, path: &std::path::Path) -> std::result::Result<Vec<u8>, String>` | `crates/contextra-graph/src/test_doubles.rs:73` | Reads data from a virtual path. |
| `pub fn sync(&self) -> std::result::Result<(), String>` | `crates/contextra-graph/src/test_doubles.rs:85` | Flushes virtual files to disk. |

#### `struct InMemoryStorageEngine`
- Ort: `crates/contextra-graph/src/test_doubles.rs:95`
```rust
pub struct InMemoryStorageEngine
```
Dokumentation: In-memory storage engine double implementing `contextra_ports::StorageEngine`.
- Impl-Block `crates/contextra-graph/src/test_doubles.rs:100`
- Impl-Block (Trait `StorageEngine`) `crates/contextra-graph/src/test_doubles.rs:108`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new() -> Self` | `crates/contextra-graph/src/test_doubles.rs:102` | Creates a new empty `InMemoryStorageEngine`. |

### Modul `text_gen`

#### `trait TextGenerator`
- Ort: `crates/contextra-graph/src/text_gen.rs:6`
```rust
pub trait TextGenerator: Send + Sync + 'static
```

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn generate<'a>(&'a self, prompt: &'a str) -> GraphFuture<'a, Result<String>>` | `crates/contextra-graph/src/text_gen.rs:7` |  |

### Modul `tl_hfd`

#### `mod diffusion`
- Ort: `crates/contextra-graph/src/tl_hfd/mod.rs:12`
```rust
pub mod diffusion
```

#### `mod error`
- Ort: `crates/contextra-graph/src/tl_hfd/mod.rs:13`
```rust
pub mod error
```

#### `mod lovasz`
- Ort: `crates/contextra-graph/src/tl_hfd/mod.rs:14`
```rust
pub mod lovasz
```

#### `mod params`
- Ort: `crates/contextra-graph/src/tl_hfd/mod.rs:15`
```rust
pub mod params
```

#### `mod shadow`
- Ort: `crates/contextra-graph/src/tl_hfd/mod.rs:16`
```rust
pub mod shadow
```

#### `fn tl_hfd_local`
- Ort: `crates/contextra-graph/src/tl_hfd/mod.rs:40`
```rust
pub fn tl_hfd_local<G: PathGraph>( graph: &G, seeds: &[EntityId], params: &TlHfdParams, ) -> Result<AHashMap<EntityId, f32>, TlHfdError>
```

### Modul `tl_hfd::diffusion`

#### `fn run_diffusion`
- Ort: `crates/contextra-graph/src/tl_hfd/diffusion.rs:37`
```rust
pub fn run_diffusion<G: PathGraph>( graph: &G, seeds: &[EntityId], params: &TlHfdParams, ) -> Result<AHashMap<EntityId, f32>, TlHfdError>
```

### Modul `tl_hfd::error`

#### `enum TlHfdError`
- Ort: `crates/contextra-graph/src/tl_hfd/error.rs:17`
```rust
pub enum TlHfdError
```

Varianten:

- `InvalidParameter(String)` — `crates/contextra-graph/src/tl_hfd/error.rs:20`
- `NonFinite` — `crates/contextra-graph/src/tl_hfd/error.rs:24`

### Modul `tl_hfd::lovasz`

#### `fn compute_lovasz_extension`
- Ort: `crates/contextra-graph/src/tl_hfd/lovasz.rs:65`
```rust
pub fn compute_lovasz_extension( participants: &[EntityId], get_x: impl Fn(EntityId) -> f32, ) -> (f32, Option<EntityId>, Option<EntityId>)
```

#### `fn truncate_participants`
- Ort: `crates/contextra-graph/src/tl_hfd/lovasz.rs:23`
```rust
pub fn truncate_participants( participants: &[EntityId], get_x: impl Fn(EntityId) -> f32, max_sort_size: usize, ) -> Vec<EntityId>
```

### Modul `tl_hfd::params`

#### `struct TlHfdParams`
- Ort: `crates/contextra-graph/src/tl_hfd/params.rs:20`
```rust
pub struct TlHfdParams
```
- Impl-Block (Trait `Default`) `crates/contextra-graph/src/tl_hfd/params.rs:35`
- Impl-Block `crates/contextra-graph/src/tl_hfd/params.rs:48`
- Impl-Block (Trait `Default`) `crates/contextra-graph/src/types.rs:254`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `sigma` | `f32` | `crates/contextra-graph/src/tl_hfd/params.rs:22` |
| `delta` | `f32` | `crates/contextra-graph/src/tl_hfd/params.rs:24` |
| `gamma` | `f32` | `crates/contextra-graph/src/tl_hfd/params.rs:26` |
| `max_iterations` | `u32` | `crates/contextra-graph/src/tl_hfd/params.rs:28` |
| `max_top_k_expansion` | `usize` | `crates/contextra-graph/src/tl_hfd/params.rs:30` |
| `max_hyperedge_sort_size` | `usize` | `crates/contextra-graph/src/tl_hfd/params.rs:32` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn validate(&self) -> Result<(), TlHfdError>` | `crates/contextra-graph/src/tl_hfd/params.rs:57` |  |

### Modul `tl_hfd::shadow`

#### `struct ShadowComparison`
- Ort: `crates/contextra-graph/src/tl_hfd/shadow.rs:66`
```rust
pub struct ShadowComparison
```

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `max_abs_diff` | `f32` | `crates/contextra-graph/src/tl_hfd/shadow.rs:68` |
| `top_k_overlap` | `f32` | `crates/contextra-graph/src/tl_hfd/shadow.rs:70` |
| `discrepancy` | `bool` | `crates/contextra-graph/src/tl_hfd/shadow.rs:72` |

#### `struct ShadowDiscrepancyReport`
- Ort: `crates/contextra-graph/src/tl_hfd/shadow.rs:31`
```rust
pub struct ShadowDiscrepancyReport
```

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `sample_count` | `u64` | `crates/contextra-graph/src/tl_hfd/shadow.rs:33` |
| `mean_topk_jaccard` | `f32` | `crates/contextra-graph/src/tl_hfd/shadow.rs:35` |
| `p99_latency_delta_us` | `f64` | `crates/contextra-graph/src/tl_hfd/shadow.rs:37` |
| `recall_improvement_ratio` | `Option<f32>` | `crates/contextra-graph/src/tl_hfd/shadow.rs:39` |

#### `struct TlHfdFlipGate`
- Ort: `crates/contextra-graph/src/tl_hfd/shadow.rs:59`
```rust
pub struct TlHfdFlipGate
```
- Impl-Block (Trait `DefaultFlipGate`) `crates/contextra-graph/src/tl_hfd/shadow.rs:61`

#### `trait DefaultFlipGate`
- Ort: `crates/contextra-graph/src/tl_hfd/shadow.rs:44`
```rust
pub trait DefaultFlipGate
```

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn should_flip(&self, report: &ShadowDiscrepancyReport) -> bool` | `crates/contextra-graph/src/tl_hfd/shadow.rs:49` | Returns `true` iff: 1. `sample_count >= MIN_SHADOW_SAMPLES` 2. `mean_topk_jaccard >= MIN_AGREEMENT_THRESHOLD` 3. `p99_latency_delta_us <= 0.0` |

#### `fn shadow_compare_forward_push_vs_tl_hfd`
- Ort: `crates/contextra-graph/src/tl_hfd/shadow.rs:88`
```rust
pub fn shadow_compare_forward_push_vs_tl_hfd<G: PathGraph>( graph: &G, seeds: &[EntityId], ppr: &PprParams, tl: &TlHfdParams, top_k: usize, ) -> Result<ShadowComparison, TlHfdError>
```

#### `const MIN_AGREEMENT_THRESHOLD`
- Ort: `crates/contextra-graph/src/tl_hfd/shadow.rs:26`
```rust
pub const MIN_AGREEMENT_THRESHOLD: f32 = 0.85
```

#### `const MIN_SHADOW_SAMPLES`
- Ort: `crates/contextra-graph/src/tl_hfd/shadow.rs:22`
```rust
pub const MIN_SHADOW_SAMPLES: u64 = 10_000
```

### Modul `types`

#### `mod local`
- Ort: `crates/contextra-graph/src/types.rs:4`
```rust
pub mod local
```

### Modul `types::local`

#### `struct DocId`
- Ort: `crates/contextra-graph/src/types.rs:55`
```rust
pub struct DocId(pub u64)
```
- Impl-Block `crates/contextra-graph/src/types.rs:57`
- Impl-Block (Trait `fmt::Display`) `crates/contextra-graph/src/types.rs:69`
- Impl-Block (Trait `From<u64>`) `crates/contextra-graph/src/types.rs:73`
- Impl-Block (Trait `From<&str>`) `crates/contextra-graph/src/types.rs:83`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub const fn new(id: u64) -> Self` | `crates/contextra-graph/src/types.rs:59` |  |
| `pub const fn inner(self) -> u64` | `crates/contextra-graph/src/types.rs:61` |  |
| `pub const fn as_u64(&self) -> u64` | `crates/contextra-graph/src/types.rs:63` |  |
| `pub fn from_key(key: &str) -> Result<Self, std::num::ParseIntError>` | `crates/contextra-graph/src/types.rs:64` |  |

#### `struct Edge`
- Ort: `crates/contextra-graph/src/types.rs:139`
```rust
pub struct Edge
```
- Impl-Block `crates/contextra-graph/src/types.rs:156`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `from` | `EntityId` | `crates/contextra-graph/src/types.rs:140` |
| `to` | `EntityId` | `crates/contextra-graph/src/types.rs:141` |
| `label` | `Arc<str>` | `crates/contextra-graph/src/types.rs:142` |
| `weight` | `f32` | `crates/contextra-graph/src/types.rs:143` |
| `tx_valid_from` | `Option<TxId>` | `crates/contextra-graph/src/types.rs:145` |
| `tx_valid_to` | `Option<TxId>` | `crates/contextra-graph/src/types.rs:147` |
| `business_valid_from` | `Option<i64>` | `crates/contextra-graph/src/types.rs:149` |
| `business_valid_to` | `Option<i64>` | `crates/contextra-graph/src/types.rs:151` |
| `source_doc_id` | `Option<DocId>` | `crates/contextra-graph/src/types.rs:153` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(from: EntityId, to: EntityId, label: impl Into<String>) -> Self` | `crates/contextra-graph/src/types.rs:157` |  |
| `pub fn with_weight(mut self, weight: f32) -> Self` | `crates/contextra-graph/src/types.rs:170` |  |
| `pub fn with_source_doc_id(mut self, doc_id: Option<DocId>) -> Self` | `crates/contextra-graph/src/types.rs:174` |  |

#### `struct Entity`
- Ort: `crates/contextra-graph/src/types.rs:119`
```rust
pub struct Entity
```
- Impl-Block `crates/contextra-graph/src/types.rs:127`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `id` | `EntityId` | `crates/contextra-graph/src/types.rs:120` |
| `name` | `Arc<str>` | `crates/contextra-graph/src/types.rs:121` |
| `entity_type` | `Arc<str>` | `crates/contextra-graph/src/types.rs:122` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(id: EntityId, name: impl Into<String>, entity_type: impl Into<String>) -> Self` | `crates/contextra-graph/src/types.rs:128` |  |

#### `struct EntityId`
- Ort: `crates/contextra-graph/src/types.rs:21`
```rust
pub struct EntityId(pub u64)
```
- Impl-Block `crates/contextra-graph/src/types.rs:23`
- Impl-Block (Trait `fmt::Display`) `crates/contextra-graph/src/types.rs:33`
- Impl-Block (Trait `From<u64>`) `crates/contextra-graph/src/types.rs:37`
- Impl-Block (Trait `From<&str>`) `crates/contextra-graph/src/types.rs:47`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub const fn new(id: u64) -> Self` | `crates/contextra-graph/src/types.rs:25` |  |
| `pub const fn inner(self) -> u64` | `crates/contextra-graph/src/types.rs:27` |  |
| `pub fn from_key(key: &str) -> Result<Self, std::num::ParseIntError>` | `crates/contextra-graph/src/types.rs:28` |  |

#### `struct HyperEdgeId`
- Ort: `crates/contextra-graph/src/types.rs:182`
```rust
pub struct HyperEdgeId(pub u64)
```
- Impl-Block `crates/contextra-graph/src/types.rs:184`
- Impl-Block (Trait `From<u64>`) `crates/contextra-graph/src/types.rs:191`
- Impl-Block (Trait `fmt::Display`) `crates/contextra-graph/src/types.rs:196`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub const fn new(id: u64) -> Self` | `crates/contextra-graph/src/types.rs:186` |  |
| `pub const fn inner(self) -> u64` | `crates/contextra-graph/src/types.rs:188` |  |

#### `struct PprConfig`
- Ort: `crates/contextra-graph/src/types.rs:281`
```rust
pub struct PprConfig
```
- Impl-Block (Trait `Default`) `crates/contextra-graph/src/types.rs:293`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `damping_factor` | `f32` | `crates/contextra-graph/src/types.rs:282` |
| `max_iterations` | `u32` | `crates/contextra-graph/src/types.rs:283` |
| `convergence_epsilon` | `f32` | `crates/contextra-graph/src/types.rs:284` |
| `algorithm` | `PprAlgorithm` | `crates/contextra-graph/src/types.rs:286` |
| `warn_on_non_convergence` | `bool` | `crates/contextra-graph/src/types.rs:288` |

#### `struct ResourceBudget`
- Ort: `crates/contextra-graph/src/types.rs:306`
```rust
pub struct ResourceBudget
```
- Impl-Block (Trait `Default`) `crates/contextra-graph/src/types.rs:310`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `memory_limit` | `u64` | `crates/contextra-graph/src/types.rs:307` |

#### `struct ResourceTracker`
- Ort: `crates/contextra-graph/src/types.rs:317`
```rust
pub struct ResourceTracker
```
- Impl-Block (Trait `Default`) `crates/contextra-graph/src/types.rs:322`
- Impl-Block `crates/contextra-graph/src/types.rs:326`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `budget` | `ResourceBudget` | `crates/contextra-graph/src/types.rs:318` |
| `memory_used` | `AtomicU64` | `crates/contextra-graph/src/types.rs:319` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(budget: ResourceBudget) -> Self` | `crates/contextra-graph/src/types.rs:327` |  |
| `pub fn unlimited() -> Self` | `crates/contextra-graph/src/types.rs:333` |  |
| `pub fn with_limit(max_bytes: usize) -> Self` | `crates/contextra-graph/src/types.rs:339` |  |
| `pub fn consume_memory(&self, bytes: u64) -> Result<(), crate::error::GraphError>` | `crates/contextra-graph/src/types.rs:345` |  |
| `pub fn record_allocation(&self, bytes: usize)` | `crates/contextra-graph/src/types.rs:361` |  |
| `pub fn release_memory(&self, bytes: u64)` | `crates/contextra-graph/src/types.rs:364` |  |
| `pub fn memory_used(&self) -> u64` | `crates/contextra-graph/src/types.rs:373` |  |
| `pub fn has_memory_capacity(&self) -> bool` | `crates/contextra-graph/src/types.rs:376` |  |
| `pub fn is_exhausted(&self) -> bool` | `crates/contextra-graph/src/types.rs:380` |  |

#### `struct RoleBinding`
- Ort: `crates/contextra-graph/src/types.rs:221`
```rust
pub struct RoleBinding
```
- Impl-Block `crates/contextra-graph/src/types.rs:226`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `role` | `RoleId` | `crates/contextra-graph/src/types.rs:222` |
| `entity` | `EntityId` | `crates/contextra-graph/src/types.rs:223` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(role: RoleId, entity: EntityId) -> Self` | `crates/contextra-graph/src/types.rs:228` |  |

#### `struct RoleId`
- Ort: `crates/contextra-graph/src/types.rs:202`
```rust
pub struct RoleId(pub u32)
```
- Impl-Block `crates/contextra-graph/src/types.rs:204`
- Impl-Block (Trait `From<u32>`) `crates/contextra-graph/src/types.rs:211`
- Impl-Block (Trait `fmt::Display`) `crates/contextra-graph/src/types.rs:216`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub const fn new(id: u32) -> Self` | `crates/contextra-graph/src/types.rs:206` |  |
| `pub const fn inner(self) -> u32` | `crates/contextra-graph/src/types.rs:208` |  |

#### `struct TlHfdParams`
- Ort: `crates/contextra-graph/src/types.rs:232`
```rust
pub struct TlHfdParams
```

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `sigma` | `f32` | `crates/contextra-graph/src/types.rs:234` |
| `delta` | `f32` | `crates/contextra-graph/src/types.rs:236` |
| `gamma` | `f32` | `crates/contextra-graph/src/types.rs:238` |
| `max_iterations` | `u32` | `crates/contextra-graph/src/types.rs:240` |
| `max_top_k_expansion` | `usize` | `crates/contextra-graph/src/types.rs:242` |
| `max_hyperedge_sort_size` | `usize` | `crates/contextra-graph/src/types.rs:244` |

#### `struct TxId`
- Ort: `crates/contextra-graph/src/types.rs:91`
```rust
pub struct TxId(pub u64)
```
- Impl-Block `crates/contextra-graph/src/types.rs:93`
- Impl-Block (Trait `fmt::Display`) `crates/contextra-graph/src/types.rs:104`
- Impl-Block (Trait `From<u64>`) `crates/contextra-graph/src/types.rs:108`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub const fn new(id: u64) -> Self` | `crates/contextra-graph/src/types.rs:97` |  |
| `pub const fn inner(self) -> u64` | `crates/contextra-graph/src/types.rs:99` |  |
| `pub const fn as_u64(self) -> u64` | `crates/contextra-graph/src/types.rs:101` |  |

Assoziierte Konstanten/Typen:

- `pub const INVALID: Self = Self(0)` — `crates/contextra-graph/src/types.rs:94`
- `pub const INTERNAL_BASE: u64 = u64::MAX - 1_000_000` — `crates/contextra-graph/src/types.rs:95`

#### `enum PprAlgorithm`
- Ort: `crates/contextra-graph/src/types.rs:270`
```rust
pub enum PprAlgorithm
```

Varianten:

- `Auto` — `crates/contextra-graph/src/types.rs:272`
- `DensePowerIteration` — `crates/contextra-graph/src/types.rs:273`
- `ForwardPush` — `crates/contextra-graph/src/types.rs:274`
- `ShadowMode` — `crates/contextra-graph/src/types.rs:275`
- `TlHfd(TlHfdParams)` — `crates/contextra-graph/src/types.rs:276`
- `ShadowModeTlHfd(TlHfdParams)` — `crates/contextra-graph/src/types.rs:277`

#### `const MAX_SEARCH_K`
- Ort: `crates/contextra-graph/src/types.rs:10`
```rust
pub const MAX_SEARCH_K: usize = 1_000
```

## 9. Impl-Blöcke ohne Typ im selben Crate

- `impl<'a, T> IntoIterator for &'a ArcSlice<T>` — `crates/contextra-graph/src/arc_slice.rs:146` (0 pub Methoden)
- `impl From<TlHfdError> for ContextraError` — `crates/contextra-graph/src/compat/algo_error_bridge.rs:8` (0 pub Methoden)
- `impl From<ApprhError> for ContextraError` — `crates/contextra-graph/src/compat/algo_error_bridge.rs:15` (0 pub Methoden)
- `impl From<GraphError> for ContextraError` — `crates/contextra-graph/src/compat/error.rs:6` (0 pub Methoden)
- `impl From<GraphMutationError> for ContextraError` — `crates/contextra-graph/src/compat/error.rs:39` (0 pub Methoden)
- `impl<T: StorageEngine + ?Sized> GraphStorage for T` — `crates/contextra-graph/src/compat/storage_bridge.rs:10` (0 pub Methoden)
- `impl crate::path_rag::PathGraph for Arc<CsrGraph>` — `crates/contextra-graph/src/csr/path_graph.rs:194` (0 pub Methoden)
- `impl From<EntityId> for u64` — `crates/contextra-graph/src/types.rs:42` (0 pub Methoden)
- `impl From<DocId> for u64` — `crates/contextra-graph/src/types.rs:78` (0 pub Methoden)
- `impl From<TxId> for u64` — `crates/contextra-graph/src/types.rs:113` (0 pub Methoden)
