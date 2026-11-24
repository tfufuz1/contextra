# Schnittstellenspezifikation: `contextra-vector`

> Erzeugt durch `tools/extract_interfaces.py` aus dem Quellcode. Keine Handpflege. Jede Angabe trägt Datei:Zeile. Maßgeblich bleibt der Quellcode.

## 1. Kopf

| Feld | Wert |
|---|---|
| Name | `contextra-vector` |
| Version | workspace |
| Pfad | `crates/contextra-vector` |
| Rolle | Vektorindex |
| Bibliothekspfad | src/lib.rs (Standard) |

**Crate-Dokumentation (//!):**

> Contextra Vector — HNSW vector index with SIMD distance computation.

## 2. Abhängigkeiten

| Name | Art | Quelle | optional | verwendet von (normal/dev) |
|---|---|---|---|---|
| `ahash` | normal | workspace | nein |  |
| `bincode` | dev | workspace | nein |  |
| `blake3` | normal | workspace | nein |  |
| `contextra-env` | normal | workspace | nein | contextra-checkpoint, contextra-crypto, contextra-graph, contextra-infer-candle, contextra-kvcache, contextra-mvcc, contextra-ports, contextra-store, contextra-text, contextra-vector |
| `contextra-ids` | normal | workspace | nein | contextra-checkpoint, contextra-crypto, contextra-graph, contextra-mvcc, contextra-store, contextra-text, contextra-types, contextra-vector |
| `contextra-mvcc` | normal | workspace | nein | contextra-db, contextra-store, contextra-vector, contextra-vector-fuzz |
| `contextra-ports` | normal | workspace | ja | contextra, contextra-adapt, contextra-agent, contextra-checkpoint, contextra-cognition, contextra-crypto, contextra-crypto-fuzz, contextra-db, contextra-engine, contextra-graph, contextra-graph-fuzz, contextra-infer-candle, contextra-infer-ollama, contextra-infer-onnx, contextra-kvcache, contextra-mcp, contextra-mvcc, contextra-rank, contextra-router, contextra-store, contextra-store-fuzz, contextra-testkit, contextra-text, contextra-vector, contextra-vector-fuzz |
| `contextra-simd` | normal | workspace | ja | contextra-vector |
| `contextra-sys` | normal | workspace | ja | contextra-bench, contextra-store, contextra-vault, contextra-vector |
| `contextra-types` | normal | workspace | ja | contextra, contextra-adapt, contextra-agent, contextra-bench, contextra-checkpoint, contextra-cognition, contextra-crypto, contextra-crypto-fuzz, contextra-db, contextra-engine, contextra-graph, contextra-graph-fuzz, contextra-infer-candle, contextra-infer-ollama, contextra-infer-onnx, contextra-ingest, contextra-kvcache, contextra-mcp, contextra-mvcc, contextra-ports, contextra-py, contextra-rank, contextra-router, contextra-simd, contextra-store, contextra-store-fuzz, contextra-testkit, contextra-text, contextra-vault, contextra-vector, contextra-vector-fuzz |
| `criterion` | dev | registry | nein |  |
| `loom` | dev | registry | nein |  |
| `memmap2` | normal | workspace | nein |  |
| `parking_lot` | normal | workspace | nein |  |
| `proptest` | dev | registry | nein |  |
| `rand` | normal | workspace | nein |  |
| `roaring` | normal | workspace | nein |  |
| `serde` | normal | workspace | nein |  |
| `serde_json` | dev | workspace | nein |  |
| `subtle` | normal | registry | nein |  |
| `tempfile` | dev | registry | nein |  |
| `thiserror` | normal | workspace | nein |  |
| `tokio` | dev | workspace | nein |  |
| `tracing` | normal | workspace | nein |  |
| `zeroize` | normal | registry | nein |  |

**Verwendet von (Crate-Ebene):** `contextra-agent`, `contextra-bench`, `contextra-db`, `contextra-engine`, `contextra-vector-fuzz`

## 3. Features

| Feature | Aktiviert |
|---|---|
| `default` | `std` |
| `diskann` | `experimental-diskann` |
| `docid-128` | `contextra-ids/docid-128` |
| `experimental` | `experimental-diskann`, `experimental-rabitq`, `experimental-predicate-augmented-search` |
| `experimental-all` | `experimental-diskann`, `experimental-rabitq`, `experimental-predicate-augmented-search` |
| `experimental-diskann` | – |
| `experimental-predicate-augmented-search` | – |
| `experimental-rabitq` | – |
| `graph` | – |
| `inspect` | – |
| `mmap` | `dep:contextra-sys` |
| `partial-index-rebuild` | – |
| `ports-compat` | `workspace` |
| `quantize-rabitq` | `experimental-rabitq` |
| `serde` | – |
| `simd` | `dep:contextra-simd`, `contextra-simd?/serde` |
| `std` | – |
| `tracing` | – |
| `workspace` | `dep:contextra-ports`, `dep:contextra-types`, `dep:contextra-sys`, `mmap` |

## 4. Metriken (Quellcode)

| Kennzahl | Wert |
|---|---|
| Quelldateien | 104 |
| Quellzeilen | 17744 |
| Öffentliche Items (Modul-Ebene) | 167 |
| davon `fn` | 32 |
| davon `struct` | 48 |
| davon `enum` | 9 |
| davon `trait` | 14 |
| davon `type` | 6 |
| davon `const` | 13 |
| davon `mod` | 45 |
| Öffentliche Makros (`#[macro_export]`) | 0 |
| Öffentliche Re-Exports (`pub use`) | 45 |
| `unsafe`-Vorkommen (Produktionscode) | 0 |
| `.unwrap()` (Produktionscode) | 52 |
| `.expect(` (Produktionscode) | 61 |
| `panic!` (Produktionscode) | 6 |
| `todo!/unimplemented!/unreachable!` | 0 |
| Indexierungen `x[…]` (grob, Produktionscode) | 288 |
| `#[deprecated]` | 15 |
| `#[cfg(feature…)]`-Gates | 48 |
| Parser-Abdeckung (geparste `pub`-Deklarationen / Grep-Treffer) | 432 / 432 = 100 % |

## 5. Modulbaum und Dateien

| Modul | Datei | Zeilen |
|---|---|---|
| `acorn::gamma_augmentation` | `crates/contextra-vector/src/acorn/gamma_augmentation.rs` | 53 |
| `acorn` | `crates/contextra-vector/src/acorn/mod.rs` | 67 |
| `acorn::naive_reference` | `crates/contextra-vector/src/acorn/naive_reference.rs` | 89 |
| `casting` | `crates/contextra-vector/src/casting.rs` | 86 |
| `compat::candidate_stream` | `crates/contextra-vector/src/compat/candidate_stream.rs` | 186 |
| `compat::diskann_vector_index` | `crates/contextra-vector/src/compat/diskann_vector_index.rs` | 190 |
| `compat::error` | `crates/contextra-vector/src/compat/error.rs` | 69 |
| `compat::hnsw_vector_index` | `crates/contextra-vector/src/compat/hnsw_vector_index.rs` | 828 |
| `compat` | `crates/contextra-vector/src/compat/mod.rs` | 16 |
| `compat::purge` | `crates/contextra-vector/src/compat/purge.rs` | 241 |
| `compat::scanner` | `crates/contextra-vector/src/compat/scanner.rs` | 340 |
| `compat::types` | `crates/contextra-vector/src/compat/types.rs` | 43 |
| `compute_pool` | `crates/contextra-vector/src/compute_pool.rs` | 294 |
| `contract` | `crates/contextra-vector/src/contract.rs` | 155 |
| `diskann::build::flush` | `crates/contextra-vector/src/diskann/build/flush.rs` | 223 |
| `diskann::build` | `crates/contextra-vector/src/diskann/build/mod.rs` | 147 |
| `diskann::build::pending_wal` | `crates/contextra-vector/src/diskann/build/pending_wal.rs` | 223 |
| `diskann::build::tombstone_wal` | `crates/contextra-vector/src/diskann/build/tombstone_wal.rs` | 178 |
| `diskann::config` | `crates/contextra-vector/src/diskann/config.rs` | 78 |
| `diskann::filtered` | `crates/contextra-vector/src/diskann/filtered.rs` | 74 |
| `diskann::format` | `crates/contextra-vector/src/diskann/format.rs` | 210 |
| `diskann::hnsw_bridge` | `crates/contextra-vector/src/diskann/hnsw_bridge.rs` | 58 |
| `diskann::integrity` | `crates/contextra-vector/src/diskann/integrity.rs` | 43 |
| `diskann::io` | `crates/contextra-vector/src/diskann/io.rs` | 133 |
| `diskann` | `crates/contextra-vector/src/diskann/mod.rs` | 24 |
| `diskann::persistence::delta` | `crates/contextra-vector/src/diskann/persistence/delta.rs` | 72 |
| `diskann::persistence::fallback` | `crates/contextra-vector/src/diskann/persistence/fallback.rs` | 22 |
| `diskann::persistence::loader` | `crates/contextra-vector/src/diskann/persistence/loader.rs` | 268 |
| `diskann::persistence` | `crates/contextra-vector/src/diskann/persistence/mod.rs` | 10 |
| `diskann::persistence::node` | `crates/contextra-vector/src/diskann/persistence/node.rs` | 172 |
| `diskann::persistence::writer` | `crates/contextra-vector/src/diskann/persistence/writer.rs` | 115 |
| `diskann::predicate_augmented` | `crates/contextra-vector/src/diskann/predicate_augmented.rs` | 273 |
| `diskann::search` | `crates/contextra-vector/src/diskann/search.rs` | 398 |
| `diskann::tests::build_search_tests` | `crates/contextra-vector/src/diskann/tests/build_search_tests.rs` | 327 |
| `diskann::tests::config_tests` | `crates/contextra-vector/src/diskann/tests/config_tests.rs` | 242 |
| `diskann::tests` | `crates/contextra-vector/src/diskann/tests/mod.rs` | 9 |
| `diskann::tests::persistence_tests` | `crates/contextra-vector/src/diskann/tests/persistence_tests.rs` | 273 |
| `diskann::tests::recall_tests` | `crates/contextra-vector/src/diskann/tests/recall_tests.rs` | 168 |
| `diskann::tests::wal_tests` | `crates/contextra-vector/src/diskann/tests/wal_tests.rs` | 116 |
| `diskann::types` | `crates/contextra-vector/src/diskann/types.rs` | 302 |
| `distance::dispatch` | `crates/contextra-vector/src/distance/dispatch.rs` | 115 |
| `distance::inject` | `crates/contextra-vector/src/distance/inject.rs` | 172 |
| `distance::kernel` | `crates/contextra-vector/src/distance/kernel.rs` | 81 |
| `distance` | `crates/contextra-vector/src/distance/mod.rs` | 114 |
| `distance::scalar` | `crates/contextra-vector/src/distance/scalar.rs` | 411 |
| `distance::u8_ops` | `crates/contextra-vector/src/distance/u8_ops.rs` | 48 |
| `distance::validate` | `crates/contextra-vector/src/distance/validate.rs` | 68 |
| `error` | `crates/contextra-vector/src/error.rs` | 94 |
| `fixed_hasher` | `crates/contextra-vector/src/fixed_hasher.rs` | 49 |
| `hnsw::acorn_filtered` | `crates/contextra-vector/src/hnsw/acorn_filtered.rs` | 297 |
| `hnsw::adaptive_ef` | `crates/contextra-vector/src/hnsw/adaptive_ef.rs` | 310 |
| `hnsw::arena` | `crates/contextra-vector/src/hnsw/arena.rs` | 429 |
| `hnsw::batch` | `crates/contextra-vector/src/hnsw/batch.rs` | 89 |
| `hnsw::clock` | `crates/contextra-vector/src/hnsw/clock.rs` | 49 |
| `hnsw::config` | `crates/contextra-vector/src/hnsw/config.rs` | 228 |
| `hnsw::core_insert::apply` | `crates/contextra-vector/src/hnsw/core_insert/apply.rs` | 72 |
| `hnsw::core_insert::compute` | `crates/contextra-vector/src/hnsw/core_insert/compute.rs` | 376 |
| `hnsw::core_insert::connectivity` | `crates/contextra-vector/src/hnsw/core_insert/connectivity.rs` | 114 |
| `hnsw::core_insert` | `crates/contextra-vector/src/hnsw/core_insert/mod.rs` | 16 |
| `hnsw::core_rebuild::backoff` | `crates/contextra-vector/src/hnsw/core_rebuild/backoff.rs` | 132 |
| `hnsw::core_rebuild::full` | `crates/contextra-vector/src/hnsw/core_rebuild/full.rs` | 436 |
| `hnsw::core_rebuild` | `crates/contextra-vector/src/hnsw/core_rebuild/mod.rs` | 138 |
| `hnsw::core_rebuild::region` | `crates/contextra-vector/src/hnsw/core_rebuild/region.rs` | 118 |
| `hnsw::core_rebuild::seq_replay` | `crates/contextra-vector/src/hnsw/core_rebuild/seq_replay.rs` | 195 |
| `hnsw::core_rebuild::snapshot` | `crates/contextra-vector/src/hnsw/core_rebuild/snapshot.rs` | 441 |
| `hnsw::core_search::adaptive` | `crates/contextra-vector/src/hnsw/core_search/adaptive.rs` | 263 |
| `hnsw::core_search::filtered` | `crates/contextra-vector/src/hnsw/core_search/filtered.rs` | 237 |
| `hnsw::core_search::layer` | `crates/contextra-vector/src/hnsw/core_search/layer.rs` | 174 |
| `hnsw::core_search` | `crates/contextra-vector/src/hnsw/core_search/mod.rs` | 20 |
| `hnsw::core_search::rerank` | `crates/contextra-vector/src/hnsw/core_search/rerank.rs` | 42 |
| `hnsw::core_search::validate` | `crates/contextra-vector/src/hnsw/core_search/validate.rs` | 47 |
| `hnsw::deletion` | `crates/contextra-vector/src/hnsw/deletion.rs` | 474 |
| `hnsw` | `crates/contextra-vector/src/hnsw/mod.rs` | 79 |
| `hnsw::sq8_bias` | `crates/contextra-vector/src/hnsw/sq8_bias.rs` | 108 |
| `hnsw::staging` | `crates/contextra-vector/src/hnsw/staging.rs` | 63 |
| `hnsw::types::builder` | `crates/contextra-vector/src/hnsw/types/builder.rs` | 192 |
| `hnsw::types::core` | `crates/contextra-vector/src/hnsw/types/core.rs` | 160 |
| `hnsw::types::data` | `crates/contextra-vector/src/hnsw/types/data.rs` | 71 |
| `hnsw::types::guards` | `crates/contextra-vector/src/hnsw/types/guards.rs` | 37 |
| `hnsw::types::index` | `crates/contextra-vector/src/hnsw/types/index.rs` | 261 |
| `hnsw::types` | `crates/contextra-vector/src/hnsw/types/mod.rs` | 22 |
| `hnsw::types::persist` | `crates/contextra-vector/src/hnsw/types/persist.rs` | 291 |
| `hnsw::types::seqlog` | `crates/contextra-vector/src/hnsw/types/seqlog.rs` | 22 |
| `crate` | `crates/contextra-vector/src/lib.rs` | 130 |
| `partial_rebuild` | `crates/contextra-vector/src/partial_rebuild.rs` | 377 |
| `persistence::atomic` | `crates/contextra-vector/src/persistence/atomic.rs` | 58 |
| `persistence::header` | `crates/contextra-vector/src/persistence/header.rs` | 468 |
| `persistence::io` | `crates/contextra-vector/src/persistence/io.rs` | 60 |
| `persistence::mmap` | `crates/contextra-vector/src/persistence/mmap.rs` | 273 |
| `persistence` | `crates/contextra-vector/src/persistence/mod.rs` | 19 |
| `persistence::node` | `crates/contextra-vector/src/persistence/node.rs` | 94 |
| `persistence::tests::atomic_tests` | `crates/contextra-vector/src/persistence/tests/atomic_tests.rs` | 34 |
| `persistence::tests::header_tests` | `crates/contextra-vector/src/persistence/tests/header_tests.rs` | 47 |
| `persistence::tests::mmap_tests` | `crates/contextra-vector/src/persistence/tests/mmap_tests.rs` | 447 |
| `persistence::tests` | `crates/contextra-vector/src/persistence/tests/mod.rs` | 10 |
| `persistence::tests::node_tests` | `crates/contextra-vector/src/persistence/tests/node_tests.rs` | 39 |
| `quantize::decode` | `crates/contextra-vector/src/quantize/decode.rs` | 46 |
| `quantize::distance` | `crates/contextra-vector/src/quantize/distance.rs` | 103 |
| `quantize::encode` | `crates/contextra-vector/src/quantize/encode.rs` | 88 |
| `quantize` | `crates/contextra-vector/src/quantize/mod.rs` | 116 |
| `quantize::tests` | `crates/contextra-vector/src/quantize/tests.rs` | 553 |
| `quantize::train` | `crates/contextra-vector/src/quantize/train.rs` | 195 |
| `quantize_rabitq` | `crates/contextra-vector/src/quantize_rabitq.rs` | 525 |
| `types` | `crates/contextra-vector/src/types.rs` | 92 |

## 6. Re-Exports (`pub use`)

- `pub use gamma_augmentation::compute_gamma_edge_budget` — `crates/contextra-vector/src/acorn/mod.rs:14` (Modul `acorn`)
- `pub use naive_reference::NaiveReferenceIndex` — `crates/contextra-vector/src/acorn/mod.rs:15` (Modul `acorn`)
- `pub use config::` — `crates/contextra-vector/src/diskann/mod.rs:23` (Modul `diskann`)
- `pub use types::DiskAnnIndex` — `crates/contextra-vector/src/diskann/mod.rs:24` (Modul `diskann`)
- `pub use crate::contract` — `crates/contextra-vector/src/distance/mod.rs:12` (Modul `distance`)
- `pub use dispatch::` — `crates/contextra-vector/src/distance/mod.rs:22` (Modul `distance`)
- `pub use inject::InjectedDistanceKernel` — `crates/contextra-vector/src/distance/mod.rs:27` (Modul `distance`)
- `pub use kernel::` — `crates/contextra-vector/src/distance/mod.rs:29` (Modul `distance`)
- `pub use scalar::` — `crates/contextra-vector/src/distance/mod.rs:31` (Modul `distance`)
- `pub use u8_ops::` — `crates/contextra-vector/src/distance/mod.rs:36` (Modul `distance`)
- `pub use validate::validate_vector` — `crates/contextra-vector/src/distance/mod.rs:41` (Modul `distance`)
- `pub use backoff::BackoffTracker` — `crates/contextra-vector/src/hnsw/core_rebuild/mod.rs:18` (Modul `hnsw::core_rebuild`)
- `pub use crate::compat::purge::` — `crates/contextra-vector/src/hnsw/mod.rs:42` (Modul `hnsw`)
- `pub use crate::compat::scanner::` — `crates/contextra-vector/src/hnsw/mod.rs:45` (Modul `hnsw`)
- `pub use crate::compat::hnsw_vector_index::` — `crates/contextra-vector/src/hnsw/mod.rs:64` (Modul `hnsw`)
- `pub use adaptive_ef::` — `crates/contextra-vector/src/hnsw/mod.rs:65` (Modul `hnsw`)
- `pub use arena::` — `crates/contextra-vector/src/hnsw/mod.rs:66` (Modul `hnsw`)
- `pub use batch::` — `crates/contextra-vector/src/hnsw/mod.rs:67` (Modul `hnsw`)
- `pub use clock::MonotonicCounterClock` — `crates/contextra-vector/src/hnsw/mod.rs:68` (Modul `hnsw`)
- `pub use config::` — `crates/contextra-vector/src/hnsw/mod.rs:69` (Modul `hnsw`)
- `pub use deletion::` — `crates/contextra-vector/src/hnsw/mod.rs:70` (Modul `hnsw`)
- `pub use sq8_bias::Sq8Bias` — `crates/contextra-vector/src/hnsw/mod.rs:71` (Modul `hnsw`)
- `pub use types::` — `crates/contextra-vector/src/hnsw/mod.rs:72` (Modul `hnsw`)
- `pub use staging::` — `crates/contextra-vector/src/hnsw/mod.rs:78` (Modul `hnsw`)
- `pub use self::core::` — `crates/contextra-vector/src/hnsw/types/mod.rs:16` (Modul `hnsw::types`)
- `pub use self::data::` — `crates/contextra-vector/src/hnsw/types/mod.rs:17` (Modul `hnsw::types`)
- `pub use self::guards::` — `crates/contextra-vector/src/hnsw/types/mod.rs:21` (Modul `hnsw::types`)
- `pub use self::index::` — `crates/contextra-vector/src/hnsw/types/mod.rs:22` (Modul `hnsw::types`)
- `pub use compat::candidate_stream::` — `crates/contextra-vector/src/lib.rs:102` (Modul `crate`)
- `pub use compute_pool::ComputePool` — `crates/contextra-vector/src/lib.rs:104` (Modul `crate`)
- `pub use contract::` — `crates/contextra-vector/src/lib.rs:106` (Modul `crate`)
- `pub use diskann::` — `crates/contextra-vector/src/lib.rs:113` (Modul `crate`)
- `pub use hnsw::` — `crates/contextra-vector/src/lib.rs:117` (Modul `crate`)
- `pub use hnsw::` — `crates/contextra-vector/src/lib.rs:119` (Modul `crate`)
- `pub use persistence::` — `crates/contextra-vector/src/lib.rs:121` (Modul `crate`)
- `pub use quantize::ScalarQuantizer` — `crates/contextra-vector/src/lib.rs:123` (Modul `crate`)
- `pub use quantize_rabitq::RaBitQQuantizer` — `crates/contextra-vector/src/lib.rs:126` (Modul `crate`)
- `pub use types::` — `crates/contextra-vector/src/lib.rs:128` (Modul `crate`)
- `pub use atomic::write_atomic` — `crates/contextra-vector/src/persistence/mod.rs:16` (Modul `persistence`)
- `pub use header::` — `crates/contextra-vector/src/persistence/mod.rs:17` (Modul `persistence`)
- `pub use mmap::MmapIndex` — `crates/contextra-vector/src/persistence/mod.rs:18` (Modul `persistence`)
- `pub use node::NodeRecord` — `crates/contextra-vector/src/persistence/mod.rs:19` (Modul `persistence`)
- `pub use contextra_ids::` — `crates/contextra-vector/src/types.rs:17` (Modul `types`)
- `pub use contextra_ids::TOMBSTONE_BIT` — `crates/contextra-vector/src/types.rs:75` (Modul `types`)
- `pub use contextra_env::` — `crates/contextra-vector/src/types.rs:80` (Modul `types`)

## 8. Öffentliche Schnittstelle je Item

### Modul `acorn`

#### `mod gamma_augmentation`
- Ort: `crates/contextra-vector/src/acorn/mod.rs:10`
```rust
pub mod gamma_augmentation
```

#### `mod naive_reference`
- Ort: `crates/contextra-vector/src/acorn/mod.rs:12`
```rust
pub mod naive_reference
```

#### `enum AcornError`
- Ort: `crates/contextra-vector/src/acorn/mod.rs:42`
```rust
pub enum AcornError
```
Dokumentation: Errors occurring during ACORN search or graph traversal operations.

Varianten:

- `DimensionMismatch{` — `crates/contextra-vector/src/acorn/mod.rs:45`
- `InvalidK{` — `crates/contextra-vector/src/acorn/mod.rs:54`
- `InvalidGamma(u32)` — `crates/contextra-vector/src/acorn/mod.rs:61`
- `Internal(String)` — `crates/contextra-vector/src/acorn/mod.rs:65`

#### `trait FilteredIndex`
- Ort: `crates/contextra-vector/src/acorn/mod.rs:27`
```rust
pub trait FilteredIndex
```
Dokumentation: Prädikatsagnostischer Suchpfad (ACORN-Muster, Patel et al. 2024). Im Gegensatz zu Post-Filtering wird das Prädikat WÄHREND der Graph-Traversal ausgewertet, mit erhöhter Kantenzahl (γ-Augmentierung) zur Konnektivitätssicherung bei hoher Selektivität.

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn search_knn_acorn( &self, query: &[f32], k: usize, predicate: &Self::Predicate, gamma: u32, ) -> Result<Vec<(DocId, f32)>, AcornError>` | `crates/contextra-vector/src/acorn/mod.rs:30` |  |

Assoziierte Konstanten/Typen:

- `type Predicate: ?Sized` — `crates/contextra-vector/src/acorn/mod.rs:28`

### Modul `acorn::gamma_augmentation`

#### `fn compute_gamma_edge_budget`
- Ort: `crates/contextra-vector/src/acorn/gamma_augmentation.rs:29`
```rust
pub fn compute_gamma_edge_budget(base_degree: usize, predicate_selectivity: f32) -> usize
```
Dokumentation: Computes the augmented edge budget factor $\gamma$ for graph expansion in ACORN search. Based on Patel et al. 2024 (ACORN: Performant and Predicate-Agnostic Vector Search): When predicate selectivity is high (meaning a low `predicate_selectivity` fraction of documents match, e.g. 0.01 = 1%), graph connectivity degrades significantly if traversing only matching edges. To preserve navigability, the graph traversal edge budget $\gamma \cdot M$ is augmented inversely to the predicate selectivity. # Monotonicity Lower `predicate_selectivity` values (more restrictive filters) return higher or equal 

### Modul `acorn::naive_reference`

#### `struct NaiveReferenceIndex`
- Ort: `crates/contextra-vector/src/acorn/naive_reference.rs:20`
```rust
pub struct NaiveReferenceIndex
```
Dokumentation: Naive reference index performing brute-force search over all stored points. **NICHT für Produktion — O(n) Referenzimplementierung** Serves exclusively as a ground-truth baseline for ACORN graph traversal correctness and recall benchmarking.
- Impl-Block `crates/contextra-vector/src/acorn/naive_reference.rs:25`
- Impl-Block (Trait `FilteredIndex`) `crates/contextra-vector/src/acorn/naive_reference.rs:48`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(metric: DistanceMetric) -> Self` | `crates/contextra-vector/src/acorn/naive_reference.rs:27` | Creates a new empty `NaiveReferenceIndex` with the specified distance metric. |
| `pub fn from_vectors(vectors: Vec<(DocId, Vec<f32>)>) -> Self` | `crates/contextra-vector/src/acorn/naive_reference.rs:35` | Creates a `NaiveReferenceIndex` pre-populated with vectors using Euclidean distance. |
| `pub fn insert(&mut self, id: DocId, vector: Vec<f32>)` | `crates/contextra-vector/src/acorn/naive_reference.rs:43` | Inserts a document vector into the index. |

### Modul `casting`

#### `fn f32_to_usize`
- Ort: `crates/contextra-vector/src/casting.rs:64`
```rust
pub fn f32_to_usize(val: f32) -> Result<usize>
```
Dokumentation: Konvertiert `f32` defensiv zu `usize` mit Validierung gegen Nicht-Endlichkeit und negative Werte.

#### `fn f64_to_usize`
- Ort: `crates/contextra-vector/src/casting.rs:77`
```rust
pub fn f64_to_usize(val: f64) -> Result<usize>
```
Dokumentation: Konvertiert `f64` defensiv zu `usize` mit Validierung gegen Nicht-Endlichkeit und negative Werte.

#### `fn to_u32`
- Ort: `crates/contextra-vector/src/casting.rs:19`
```rust
pub fn to_u32(val: usize) -> Result<u32>
```
Dokumentation: Konvertiert `usize` zu `u32`, liefert `VectorError::InvalidInput` bei Überlauf.

#### `fn to_u64`
- Ort: `crates/contextra-vector/src/casting.rs:54`
```rust
pub fn to_u64(val: usize) -> u64
```
Dokumentation: Konvertiert `usize` zu `u64`. Unfehlbar auf 32-Bit und 64-Bit Systemen.

#### `fn to_u8`
- Ort: `crates/contextra-vector/src/casting.rs:47`
```rust
pub fn to_u8(val: usize) -> Result<u8>
```
Dokumentation: Konvertiert `usize` zu `u8`, liefert `VectorError::InvalidInput` bei Überschreitung von 255.

#### `fn u32_to_usize`
- Ort: `crates/contextra-vector/src/casting.rs:26`
```rust
pub fn u32_to_usize(val: u32) -> usize
```
Dokumentation: Konvertiert `u32` zu `usize`. Unfehlbar auf 32-Bit und 64-Bit Zielarchitekturen.

#### `fn u64_to_u32`
- Ort: `crates/contextra-vector/src/casting.rs:40`
```rust
pub fn u64_to_u32(val: u64) -> Result<u32>
```
Dokumentation: Konvertiert `u64` zu `u32`, liefert `VectorError::InvalidInput` bei Überschreitung von `u32::MAX`.

#### `fn u64_to_usize`
- Ort: `crates/contextra-vector/src/casting.rs:33`
```rust
pub fn u64_to_usize(val: u64) -> Result<usize>
```
Dokumentation: Konvertiert `u64` zu `usize`, liefert `VectorError::InvalidInput` bei Überlauf auf 32-Bit.

### Modul `compat`

#### `mod candidate_stream`
- Ort: `crates/contextra-vector/src/compat/mod.rs:4`
```rust
pub mod candidate_stream
```

#### `mod diskann_vector_index`
- Ort: `crates/contextra-vector/src/compat/mod.rs:12`
```rust
pub mod diskann_vector_index
```

#### `mod error`
- Ort: `crates/contextra-vector/src/compat/mod.rs:6`
```rust
pub mod error
```

#### `mod hnsw_vector_index`
- Ort: `crates/contextra-vector/src/compat/mod.rs:10`
```rust
pub mod hnsw_vector_index
```

#### `mod purge`
- Ort: `crates/contextra-vector/src/compat/mod.rs:14`
```rust
pub mod purge
```

#### `mod scanner`
- Ort: `crates/contextra-vector/src/compat/mod.rs:16`
```rust
pub mod scanner
```

#### `mod types`
- Ort: `crates/contextra-vector/src/compat/mod.rs:8`
```rust
pub mod types
```

### Modul `compat::candidate_stream`

#### `struct VectorCandidateStream`
- Ort: `crates/contextra-vector/src/compat/candidate_stream.rs:51`
```rust
pub struct VectorCandidateStream<'a, V: VectorIndex>
```
Dokumentation: On-demand loading async pull-stream for vector search candidates. # Candidate Ordering & Determinism Vector search scores represent relevance (higher score = better candidate). Newly discovered candidates in each reload iteration are delivered in deterministic order: score descending, then [`DocId`] ascending. `f32::total_cmp` is used to ensure safe and deterministic ordering even with special floating point values. # Non-Prefix-Stability (Known Limitation) Approximate Nearest Neighbor (ANN) search algorithms (such as HNSW or DiskANN) are **not prefix-stable**: `search(q, 32)` does not invaria
- Impl-Block `crates/contextra-vector/src/compat/candidate_stream.rs:63`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(index: &'a V, query: impl Into<Vec<f32>>, seq_no: Option<u64>) -> Self` | `crates/contextra-vector/src/compat/candidate_stream.rs:66` |  |
| `pub fn with_batch_size(mut self, n: usize) -> Self` | `crates/contextra-vector/src/compat/candidate_stream.rs:81` | Sets a custom batch size (minimum 1). |
| `pub async fn next_batch(&mut self) -> Result<Vec<ScoredDocument>>` | `crates/contextra-vector/src/compat/candidate_stream.rs:89` |  |
| `pub async fn next_doc(&mut self) -> Result<Option<DocId>>` | `crates/contextra-vector/src/compat/candidate_stream.rs:108` |  |
| `pub fn yielded(&self) -> usize` | `crates/contextra-vector/src/compat/candidate_stream.rs:122` | Returns the total number of items yielded so far. |
| `pub fn fetched_depth(&self) -> usize` | `crates/contextra-vector/src/compat/candidate_stream.rs:127` | Returns the last $k$ search depth requested from the vector index. |
| `pub fn is_exhausted(&self) -> bool` | `crates/contextra-vector/src/compat/candidate_stream.rs:132` | Returns whether the candidate stream is exhausted. |
| `pub const fn max_depth() -> usize` | `crates/contextra-vector/src/compat/candidate_stream.rs:137` | Returns the maximum depth limit for candidate retrieval (`MAX_SEARCH_K`). |

#### `trait VectorIndex`
- Ort: `crates/contextra-vector/src/compat/candidate_stream.rs:11`
```rust
pub trait VectorIndex: Send + Sync
```

#### `const DEFAULT_VECTOR_STREAM_BATCH_SIZE`
- Ort: `crates/contextra-vector/src/compat/candidate_stream.rs:18`
```rust
pub const DEFAULT_VECTOR_STREAM_BATCH_SIZE: usize = 16
```
Dokumentation: Default batch size for vector candidate retrieval.

### Modul `compat::hnsw_vector_index`

#### `struct HnswCompactStats`
- Ort: `crates/contextra-vector/src/compat/hnsw_vector_index.rs:596`
```rust
pub struct HnswCompactStats
```
Dokumentation: Statistiken über den Kompaktierungsvorgang des HNSW-Vektorindex.

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `purged_docs` | `usize` | `crates/contextra-vector/src/compat/hnsw_vector_index.rs:598` |
| `active_docs` | `usize` | `crates/contextra-vector/src/compat/hnsw_vector_index.rs:600` |

#### `trait Index`
- Ort: `crates/contextra-vector/src/compat/hnsw_vector_index.rs:568`
```rust
pub trait Index<Id, Query: ?Sized>
```
Dokumentation: Normative Index-Schnittstelle gemäß SPEC §4.3 (Klasse A).

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn insert(&mut self, id: Id, payload: &[u8]) -> ContractResult<()>` | `crates/contextra-vector/src/compat/hnsw_vector_index.rs:573` | Fügt ein Dokument mit gegebener ID und Byte-Nutzlast in den Index ein. |
| `fn remove(&mut self, id: &Id) -> ContractResult<bool>` | `crates/contextra-vector/src/compat/hnsw_vector_index.rs:576` | Entfernt ein Dokument anhand seiner ID aus dem Index. |
| `fn search(&self, query: &Query, limit: usize) -> ContractResult<Vec<Self::Hit>>` | `crates/contextra-vector/src/compat/hnsw_vector_index.rs:579` | Führt eine Suche anhand der Query aus und liefert bis zu `limit` Treffer zurück. |

Assoziierte Konstanten/Typen:

- `type Hit` — `crates/contextra-vector/src/compat/hnsw_vector_index.rs:570`

#### `trait Maintain`
- Ort: `crates/contextra-vector/src/compat/hnsw_vector_index.rs:583`
```rust
pub trait Maintain
```
Dokumentation: Normative Wartungsschnittstelle gemäß SPEC §4.3 (Klasse A).

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn compact(&mut self) -> ContractResult<Self::Stats>` | `crates/contextra-vector/src/compat/hnsw_vector_index.rs:588` | Führt eine Kompaktierung durch und gibt gelöschte Einträge tatsächlich frei. |
| `fn vacuum(&mut self) -> ContractResult<()>` | `crates/contextra-vector/src/compat/hnsw_vector_index.rs:591` | Bereinigt fragmentierte interne Strukturen. |

Assoziierte Konstanten/Typen:

- `type Stats` — `crates/contextra-vector/src/compat/hnsw_vector_index.rs:585`

### Modul `compat::purge`

#### `struct HnswPurger`
- Ort: `crates/contextra-vector/src/compat/purge.rs:23`
```rust
pub struct HnswPurger
```
- Impl-Block `crates/contextra-vector/src/compat/purge.rs:28`
- Impl-Block (Trait `Purger`) `crates/contextra-vector/src/compat/purge.rs:45`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(index: Arc<HnswIndex>) -> Self` | `crates/contextra-vector/src/compat/purge.rs:30` | Erzeugt einen neuen `HnswPurger`. |

#### `fn hnsw_purger`
- Ort: `crates/contextra-vector/src/compat/purge.rs:40`
```rust
pub fn hnsw_purger(index: Arc<HnswIndex>) -> Arc<dyn Purger>
```

### Modul `compat::scanner`

#### `struct HnswScanner`
- Ort: `crates/contextra-vector/src/compat/scanner.rs:22`
```rust
pub struct HnswScanner
```
- Impl-Block `crates/contextra-vector/src/compat/scanner.rs:27`
- Impl-Block `crates/contextra-vector/src/compat/scanner.rs:212`
- Impl-Block (Trait `LayerScanner`) `crates/contextra-vector/src/compat/scanner.rs:226`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(index: Arc<HnswIndex>) -> Self` | `crates/contextra-vector/src/compat/scanner.rs:29` | Erzeugt einen neuen `HnswScanner`. |
| `pub fn scan_units(&self, session: &mut ScanSession, _ctx: &RequestContext) -> Result<()>` | `crates/contextra-vector/src/compat/scanner.rs:214` | Führt den Schicht-Scan für den HNSW-Index aus. |

#### `struct ResidualReport`
- Ort: `crates/contextra-vector/src/compat/scanner.rs:241`
```rust
pub struct ResidualReport
```
Dokumentation: Struktur für die lesende Inspektionssicht (`inspect`-Sicht). Ermittelt, ob Spuren einer Dokument-ID bzw. eines Vektors noch in ID-Mappings, Slots (Nicht-Null-Bytes), Graphkanten oder Zwischenpuffern vorkommen.
- Impl-Block `crates/contextra-vector/src/compat/scanner.rs:252`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `in_id_map` | `bool` | `crates/contextra-vector/src/compat/scanner.rs:243` |
| `non_zero_vector_bytes` | `bool` | `crates/contextra-vector/src/compat/scanner.rs:245` |
| `incoming_edge_count` | `usize` | `crates/contextra-vector/src/compat/scanner.rs:247` |
| `in_staging` | `bool` | `crates/contextra-vector/src/compat/scanner.rs:249` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn has_residual(&self) -> bool` | `crates/contextra-vector/src/compat/scanner.rs:254` | Gibt an, ob noch irgendwelche Restspuren (ID, Vektorbytes, Kanten, Staging) existieren. |

#### `fn hnsw_scanner`
- Ort: `crates/contextra-vector/src/compat/scanner.rs:207`
```rust
pub fn hnsw_scanner(index: Arc<HnswIndex>) -> Arc<dyn LayerScanner>
```

### Modul `compute_pool`

#### `struct ComputePool`
- Ort: `crates/contextra-vector/src/compute_pool.rs:59`
```rust
pub struct ComputePool
```
Dokumentation: Begrenzter Compute-Threadpool für Hintergrundberechnungen und rechenintensive Indizierung. Erzwingt eine strikte Obergrenze für Worker-Threads und eine beschränkte Auftragswarteschlange.
- Impl-Block `crates/contextra-vector/src/compute_pool.rs:79`
- Impl-Block (Trait `Default`) `crates/contextra-vector/src/compute_pool.rs:207`
- Impl-Block (Trait `std::fmt::Debug`) `crates/contextra-vector/src/compute_pool.rs:217`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(max_workers: usize) -> Self` | `crates/contextra-vector/src/compute_pool.rs:82` |  |
| `pub fn with_queue_capacity(max_workers: usize, queue_capacity: usize) -> Self` | `crates/contextra-vector/src/compute_pool.rs:90` | Erzeugt einen neuen [`ComputePool`] mit konfigurierbarer Pufferkapazität (begrenzter Kanal). |
| `pub fn max_workers(&self) -> usize` | `crates/contextra-vector/src/compute_pool.rs:158` | Gibt die maximale Worker-Kapazität des Pools zurück. |
| `pub fn active_jobs(&self) -> usize` | `crates/contextra-vector/src/compute_pool.rs:163` | Gibt die Anzahl aktuell auf Worker-Threads aktiver Jobs zurück. |
| `pub fn execute<F>(&self, f: F) where F: FnOnce() + Send + 'static,` | `crates/contextra-vector/src/compute_pool.rs:168` | Reicht eine Fire-and-Forget-Closure ein. Blockiert bei voller Warteschlange (Backpressure). |
| `pub fn spawn<F, T>(&self, f: F) -> TaskHandle<T> where F: FnOnce() -> T + Send + 'static, T: Send + 'static,` | `crates/contextra-vector/src/compute_pool.rs:182` | Reicht eine Funktion ein und liefert ein [`TaskHandle`], um das Ergebnis abzuwarten. |

#### `struct TaskHandle`
- Ort: `crates/contextra-vector/src/compute_pool.rs:22`
```rust
pub struct TaskHandle<T>
```
Dokumentation: Handle zum Abwarten des Ergebnisses eines an den [`ComputePool`] übergebenen Tasks.
- Impl-Block `crates/contextra-vector/src/compute_pool.rs:26`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn join(self) -> VectorResult<T>` | `crates/contextra-vector/src/compute_pool.rs:28` | Wartet auf den Abschluss des Tasks. Panics im Task werden als [`VectorError`] propagiert. |

#### `const DEFAULT_QUEUE_CAPACITY_PER_WORKER`
- Ort: `crates/contextra-vector/src/compute_pool.rs:16`
```rust
pub const DEFAULT_QUEUE_CAPACITY_PER_WORKER: usize = 128
```
Dokumentation: Standard-Kapazität für die synchrone Job-Warteschlange pro Worker-Thread.

### Modul `contract`

#### `struct ExternalId`
- Ort: `crates/contextra-vector/src/contract.rs:78`
```rust
pub struct ExternalId(pub u128)
```
Dokumentation: Externe Identifikationsnummer (128-Bit für breite Kompatibilität).
- Impl-Block (Trait `From<u64>`) `crates/contextra-vector/src/contract.rs:80`

#### `struct ScoredPoint`
- Ort: `crates/contextra-vector/src/contract.rs:101`
```rust
pub struct ScoredPoint
```
Dokumentation: Trefferpunkt mit Distanz-/Ähnlichkeitsbewertung.

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `id` | `ExternalId` | `crates/contextra-vector/src/contract.rs:103` |
| `score` | `f32` | `crates/contextra-vector/src/contract.rs:105` |

#### `enum Error`
- Ort: `crates/contextra-vector/src/contract.rs:25`
```rust
pub enum Error
```
Dokumentation: Lokales Fehler-Enum für Vertragsoperationen.

Varianten:

- `InvalidInput(&'static str)` — `crates/contextra-vector/src/contract.rs:28`
- `CapacityExceeded{` — `crates/contextra-vector/src/contract.rs:32`
- `Io(#[from] std::io::Error)` — `crates/contextra-vector/src/contract.rs:41`
- `StateConflict(&'static str)` — `crates/contextra-vector/src/contract.rs:45`
- `Corrupt(&'static str)` — `crates/contextra-vector/src/contract.rs:49`

#### `enum Metric`
- Ort: `crates/contextra-vector/src/contract.rs:89`
```rust
pub enum Metric
```
Dokumentation: Unterstützte Distanzmetriken.

Varianten:

- `Cosine` — `crates/contextra-vector/src/contract.rs:91`
- `Euclidean` — `crates/contextra-vector/src/contract.rs:93`
- `DotProduct` — `crates/contextra-vector/src/contract.rs:95`

#### `trait BlockTransform`
- Ort: `crates/contextra-vector/src/contract.rs:149`
```rust
pub trait BlockTransform: Send + Sync
```
Dokumentation: Generischer Transformations-Hook (z. B. für optionale Verschlüsselung/Transformation).

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn transform(&self, block: &mut [u8]) -> Result<()>` | `crates/contextra-vector/src/contract.rs:151` | Transformiert einen Datenblock vor der Persistierung. |
| `fn inverse_transform(&self, block: &mut [u8]) -> Result<()>` | `crates/contextra-vector/src/contract.rs:153` | Rekonstruiert einen transformierten Datenblock beim Lesen. |

#### `trait Clock`
- Ort: `crates/contextra-vector/src/contract.rs:60`
```rust
pub trait Clock: Send + Sync
```
Dokumentation: Trait zur Zeitabfrage.

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn now_unix_ms(&self) -> u64` | `crates/contextra-vector/src/contract.rs:62` | Liefert die aktuelle Zeit in Unix-Millisekunden. |

#### `trait DistanceFn`
- Ort: `crates/contextra-vector/src/contract.rs:113`
```rust
pub trait DistanceFn: Send + Sync
```
Dokumentation: Distanzfunktions-Schnittstelle zur externen Injektion von Berechnungen (z.B. SIMD).

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn distance(&self, a: &[f32], b: &[f32]) -> f32` | `crates/contextra-vector/src/contract.rs:115` | Berechnet die Distanz zwischen zwei gleich langen Vektoren. |

#### `trait Index`
- Ort: `crates/contextra-vector/src/contract.rs:120`
```rust
pub trait Index<Id, Query>
```
Dokumentation: Standardisierter Index-Trait für Klasse A Bausteine (SPEC §4.3).

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn insert(&mut self, id: Id, payload: &[u8]) -> Result<()>` | `crates/contextra-vector/src/contract.rs:125` | Fügt einen Vektordatenpunkt ein. |
| `fn remove(&mut self, id: &Id) -> Result<bool>` | `crates/contextra-vector/src/contract.rs:128` | Entfernt einen Vektordatenpunkt anhand seiner ID. |
| `fn search(&self, query: &Query, limit: usize) -> Result<Vec<Self::Hit>>` | `crates/contextra-vector/src/contract.rs:131` | Sucht die nächsten Nachbarn für eine gegebene Abfrage. |

Assoziierte Konstanten/Typen:

- `type Hit` — `crates/contextra-vector/src/contract.rs:122`

#### `trait Maintain`
- Ort: `crates/contextra-vector/src/contract.rs:136`
```rust
pub trait Maintain
```
Dokumentation: Neutraler Wartungs-Trait für Klasse A Bausteine (SPEC §4.3).

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn compact(&mut self) -> Result<Self::Stats>` | `crates/contextra-vector/src/contract.rs:141` | Kompaktiert den Index und gibt freigegebene Ressourcen zurück. |
| `fn vacuum(&mut self) -> Result<()>` | `crates/contextra-vector/src/contract.rs:144` | Führt physische Bereinigungen durch. |

Assoziierte Konstanten/Typen:

- `type Stats` — `crates/contextra-vector/src/contract.rs:138`

#### `trait Rng`
- Ort: `crates/contextra-vector/src/contract.rs:67`
```rust
pub trait Rng: Send
```
Dokumentation: Trait für deterministischen Zufall.

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn next_u64(&mut self) -> u64` | `crates/contextra-vector/src/contract.rs:69` | Liefert das nächste Pseudozufalls-`u64`. |

#### `type Result`
- Ort: `crates/contextra-vector/src/contract.rs:53`
```rust
pub type Result<T> = core::result::Result<T, Error>
```
Dokumentation: Standard-Result-Typ für Vertragsoperationen.

### Modul `crate`

#### `mod acorn`
- Ort: `crates/contextra-vector/src/lib.rs:77`
```rust
pub mod acorn
```

#### `mod casting`
- Ort: `crates/contextra-vector/src/lib.rs:35`
```rust
pub mod casting
```

#### `mod compat`
- Ort: `crates/contextra-vector/src/lib.rs:40`
```rust
pub mod compat
```

#### `mod compute_pool`
- Ort: `crates/contextra-vector/src/lib.rs:81`
```rust
pub mod compute_pool
```

#### `mod contract`
- Ort: `crates/contextra-vector/src/lib.rs:44`
```rust
pub mod contract
```

#### `mod diskann`
- Ort: `crates/contextra-vector/src/lib.rs:57`
```rust
pub mod diskann
```
- Gate: `#[cfg(feature = "experimental-diskann")]`

#### `mod distance`
- Ort: `crates/contextra-vector/src/lib.rs:61`
```rust
pub mod distance
```

#### `mod error`
- Ort: `crates/contextra-vector/src/lib.rs:48`
```rust
pub mod error
```

#### `mod fixed_hasher`
- Ort: `crates/contextra-vector/src/lib.rs:85`
```rust
pub mod fixed_hasher
```

#### `mod hnsw`
- Ort: `crates/contextra-vector/src/lib.rs:65`
```rust
pub mod hnsw
```

#### `mod partial_rebuild`
- Ort: `crates/contextra-vector/src/lib.rs:89`
```rust
pub mod partial_rebuild
```

#### `mod persistence`
- Ort: `crates/contextra-vector/src/lib.rs:69`
```rust
pub mod persistence
```

#### `mod quantize`
- Ort: `crates/contextra-vector/src/lib.rs:73`
```rust
pub mod quantize
```

#### `mod quantize_rabitq`
- Ort: `crates/contextra-vector/src/lib.rs:94`
```rust
pub mod quantize_rabitq
```
- Gate: `#[cfg(feature = "experimental-rabitq")]`

#### `mod types`
- Ort: `crates/contextra-vector/src/lib.rs:52`
```rust
pub mod types
```

### Modul `diskann`

#### `mod predicate_augmented`
- Ort: `crates/contextra-vector/src/diskann/mod.rs:16`
```rust
pub mod predicate_augmented
```
- Gate: `#[cfg(feature = "experimental-predicate-augmented-search")]`

### Modul `diskann::config`

#### `struct DiskAnnConfig`
- Ort: `crates/contextra-vector/src/diskann/config.rs:22`
```rust
pub struct DiskAnnConfig
```
Dokumentation: Configuration for DiskANN index.
- Impl-Block (Trait `Default`) `crates/contextra-vector/src/diskann/config.rs:48`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `index_path` | `PathBuf` | `crates/contextra-vector/src/diskann/config.rs:24` |
| `dimension` | `usize` | `crates/contextra-vector/src/diskann/config.rs:26` |
| `max_degree` | `usize` | `crates/contextra-vector/src/diskann/config.rs:28` |
| `beam_width` | `usize` | `crates/contextra-vector/src/diskann/config.rs:30` |
| `sector_size` | `usize` | `crates/contextra-vector/src/diskann/config.rs:32` |
| `memory_budget` | `usize` | `crates/contextra-vector/src/diskann/config.rs:34` |
| `distance_metric` | `DistanceMetric` | `crates/contextra-vector/src/diskann/config.rs:36` |
| `quantize` | `bool` | `crates/contextra-vector/src/diskann/config.rs:38` |
| `fallback_policy` | `DiskAnnFallbackPolicy` | `crates/contextra-vector/src/diskann/config.rs:40` |
| `pending_flush_threshold` | `Option<u64>` | `crates/contextra-vector/src/diskann/config.rs:43` |
| `compute_pool` | `Option<ComputePool>` | `crates/contextra-vector/src/diskann/config.rs:45` |

#### `enum DiskAnnFallbackPolicy`
- Ort: `crates/contextra-vector/src/diskann/config.rs:12`
```rust
pub enum DiskAnnFallbackPolicy
```
Dokumentation: Fallback behavior policy when DiskANN loading, integrity checks, or reads fail.

Varianten:

- `UseHnswOnFailure` — `crates/contextra-vector/src/diskann/config.rs:15`
- `FailFast` — `crates/contextra-vector/src/diskann/config.rs:17`

### Modul `diskann::integrity`

#### `struct DiskAnnHmac`
- Ort: `crates/contextra-vector/src/diskann/integrity.rs:6`
```rust
pub struct DiskAnnHmac
```
Dokumentation: Standalone keyed BLAKE3 hasher for DiskANN file integrity.
- Impl-Block `crates/contextra-vector/src/diskann/integrity.rs:10`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(integrity_key: &[u8]) -> Result<Self>` | `crates/contextra-vector/src/diskann/integrity.rs:12` | Creates a new `DiskAnnHmac` using the provided integrity key. |
| `pub fn update(&mut self, data: &[u8])` | `crates/contextra-vector/src/diskann/integrity.rs:35` | Feeds new data into the integrity calculation. |
| `pub fn finalize(self) -> [u8` | `crates/contextra-vector/src/diskann/integrity.rs:40` | Finalizes the calculation and yields the 32-byte digest. |

### Modul `diskann::types`

#### `struct DiskAnnIndex`
- Ort: `crates/contextra-vector/src/diskann/types.rs:26`
```rust
pub struct DiskAnnIndex
```
Dokumentation: DiskANN out-of-core vector index. This index uses `memmap2` to offload vector data and graph edges to disk. Note that mmap operations and page faults can cause the current thread to block while data is being loaded from the disk. For high-concurrency environments, consider using `ComputePool` or a background thread pool when calling methods on this index if latency spikes are a concern.
- Impl-Block (Trait `VectorIndex`) `crates/contextra-vector/src/compat/diskann_vector_index.rs:14`
- Impl-Block `crates/contextra-vector/src/diskann/build/flush.rs:12`
- Impl-Block `crates/contextra-vector/src/diskann/build/mod.rs:19`
- Impl-Block `crates/contextra-vector/src/diskann/build/pending_wal.rs:17`
- Impl-Block `crates/contextra-vector/src/diskann/build/tombstone_wal.rs:56`
- Impl-Block `crates/contextra-vector/src/diskann/filtered.rs:9`
- Impl-Block `crates/contextra-vector/src/diskann/persistence/delta.rs:9`
- Impl-Block `crates/contextra-vector/src/diskann/persistence/fallback.rs:10`
- Impl-Block `crates/contextra-vector/src/diskann/persistence/loader.rs:19`
- Impl-Block `crates/contextra-vector/src/diskann/persistence/node.rs:10`
- Impl-Block `crates/contextra-vector/src/diskann/persistence/writer.rs:18`
- Impl-Block `crates/contextra-vector/src/diskann/predicate_augmented.rs:13`
- Impl-Block `crates/contextra-vector/src/diskann/search.rs:16`
- Impl-Block (Trait `Clone`) `crates/contextra-vector/src/diskann/types.rs:51`
- Impl-Block `crates/contextra-vector/src/diskann/types.rs:187`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `inner` | `Arc<DiskAnnIndexInner>` | `crates/contextra-vector/src/diskann/types.rs:27` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn build_to_path_sync( &self, target_path: &std::path::Path, vectors: &[Vec<f32>], ids: &[DocId], ) -> Result<()>` | `crates/contextra-vector/src/diskann/build/mod.rs:20` |  |
| `pub async fn build(&self, vectors: &[Vec<f32>], ids: &[DocId]) -> Result<()>` | `crates/contextra-vector/src/diskann/build/mod.rs:102` | Builds the index from a set of vectors. |
| `pub fn build_sync(&self, vectors: &[Vec<f32>], ids: &[DocId]) -> Result<()>` | `crates/contextra-vector/src/diskann/build/mod.rs:106` |  |
| `pub fn recover_pending_delta( &self, ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<usize>> + Send + '_>>` | `crates/contextra-vector/src/diskann/build/pending_wal.rs:196` | Liest uncommittete Pending-Vektoren aus `pending.wal` und fügt sie in den Index ein. |
| `pub fn recover_pending_delta_sync(&self) -> Result<usize>` | `crates/contextra-vector/src/diskann/build/pending_wal.rs:202` |  |
| `pub fn trigger_background_persist_delta(&self)` | `crates/contextra-vector/src/diskann/persistence/delta.rs:10` |  |
| `pub async fn persist_delta(&self) -> Result<()>` | `crates/contextra-vector/src/diskann/persistence/delta.rs:28` |  |
| `pub fn persist_delta_sync(&self) -> Result<()>` | `crates/contextra-vector/src/diskann/persistence/delta.rs:32` |  |
| `pub async fn load(&self) -> Result<()>` | `crates/contextra-vector/src/diskann/persistence/loader.rs:51` |  |
| `pub fn load_sync(&self) -> Result<()>` | `crates/contextra-vector/src/diskann/persistence/loader.rs:55` |  |
| `pub async fn search_predicate_augmented( &self, query: &[f32], k: usize, filter: &(dyn Fn(DocId) -> bool + Send + Sync), gamma: usize, ) -> Result<Vec<ScoredDocument>>` | `crates/contextra-vector/src/diskann/predicate_augmented.rs:26` | Executes a predicate-augmented vector search using ACORN 2-hop neighbor expansion during graph traversal. # Parameters - `query`: Query vector slice. - `k`: Num |
| `pub async fn search_internal(&self, query: &[f32], k: usize) -> Result<Vec<ScoredDocument>>` | `crates/contextra-vector/src/diskann/search.rs:226` |  |
| `pub fn try_new(config: DiskAnnConfig) -> Result<Self>` | `crates/contextra-vector/src/diskann/types.rs:188` |  |
| `pub fn len_sync(&self) -> usize` | `crates/contextra-vector/src/diskann/types.rs:283` | Stößt `persist_delta()` asynchron im Tokio-Hintergrund-Task an, falls nicht bereits ein persist_delta-Lauf aktiv ist. |

### Modul `distance::dispatch`

#### `fn compute_distance`
- Ort: `crates/contextra-vector/src/distance/dispatch.rs:25`
```rust
pub fn compute_distance<M: Into<DistanceMetric>>(a: &[f32], b: &[f32], metric: M) -> Result<f32>
```
Dokumentation: Berechnet die Distanz zwischen zwei Vektoren nach Endlichkeitsvalidierung.

#### `fn compute_distance_f32_bytes_trusted`
- Ort: `crates/contextra-vector/src/distance/dispatch.rs:72`
```rust
pub fn compute_distance_f32_bytes_trusted<M: Into<DistanceMetric>>( a: &[f32], b_bytes: &[u8], metric: M, ) -> Result<f32>
```
Dokumentation: Berechnet die Distanz zwischen vorvalidiertem f32-Slice und Little-Endian f32 Byte-Slice.

#### `fn compute_distance_trusted`
- Ort: `crates/contextra-vector/src/distance/dispatch.rs:42`
```rust
pub fn compute_distance_trusted<M: Into<DistanceMetric>>( a: &[f32], b: &[f32], metric: M, ) -> Result<f32>
```
Dokumentation: Berechnet die Distanz zwischen zwei vorvalidierten f32-Slices.

#### `fn cosine_distance`
- Ort: `crates/contextra-vector/src/distance/dispatch.rs:101`
```rust
pub fn cosine_distance(a: &[f32], b: &[f32]) -> Result<f32>
```
Dokumentation: Bequemlichkeitsfunktion für Kosinus-Distanz mit automatischer Validierung.

#### `fn dot_product_distance`
- Ort: `crates/contextra-vector/src/distance/dispatch.rs:107`
```rust
pub fn dot_product_distance(a: &[f32], b: &[f32]) -> Result<f32>
```
Dokumentation: Bequemlichkeitsfunktion für Skalarprodukt-Distanz mit automatischer Validierung.

#### `fn euclidean_distance`
- Ort: `crates/contextra-vector/src/distance/dispatch.rs:113`
```rust
pub fn euclidean_distance(a: &[f32], b: &[f32]) -> Result<f32>
```
Dokumentation: Bequemlichkeitsfunktion für Euklidische Distanz mit automatischer Validierung.

### Modul `distance::inject`

#### `struct InjectedDistanceKernel`
- Ort: `crates/contextra-vector/src/distance/inject.rs:20`
```rust
pub struct InjectedDistanceKernel<D: DistanceFn>
```
Dokumentation: Adapter, der einen [`DistanceFn`]-Typ in das [`DistanceKernel`]-Interface einbettet.
- Impl-Block `crates/contextra-vector/src/distance/inject.rs:24`
- Impl-Block (Trait `DistanceKernel`) `crates/contextra-vector/src/distance/inject.rs:31`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `distance_fn` | `D` | `crates/contextra-vector/src/distance/inject.rs:21` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(distance_fn: D) -> Self` | `crates/contextra-vector/src/distance/inject.rs:26` | Erzeugt einen neuen injizierten Kernel aus einer [`DistanceFn`]. |

### Modul `distance::kernel`

#### `struct CosineSimilarityPartsU8`
- Ort: `crates/contextra-vector/src/distance/kernel.rs:19`
```rust
pub struct CosineSimilarityPartsU8
```
Dokumentation: Rohkomponenten zur Berechnung der ganzzahligen Kosinus-Ähnlichkeit auf u8-Vektoren.

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `dot` | `u32` | `crates/contextra-vector/src/distance/kernel.rs:21` |
| `norm_a_sq` | `u32` | `crates/contextra-vector/src/distance/kernel.rs:23` |
| `norm_b_sq` | `u32` | `crates/contextra-vector/src/distance/kernel.rs:25` |

#### `trait DistanceKernel`
- Ort: `crates/contextra-vector/src/distance/kernel.rs:30`
```rust
pub trait DistanceKernel: Send + Sync + 'static
```
Dokumentation: Schnittstelle für Distanzberechnungskerne auf f32- und quantisierten Vektoren.

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn distance(&self, metric: DistanceMetric, a: &[f32], b: &[f32]) -> Result<f32>` | `crates/contextra-vector/src/distance/kernel.rs:32` | Berechnet die Distanz zwischen zwei Slices nach Endlichkeitsvalidierung. |
| `fn distance_f32_bytes(&self, metric: DistanceMetric, a: &[f32], b_bytes: &[u8]) -> Result<f32>` | `crates/contextra-vector/src/distance/kernel.rs:35` | Berechnet die Distanz zwischen einer f32-Query und einem Byte-Slice (LE f32). |
| `fn distance_trusted(&self, metric: DistanceMetric, a: &[f32], b: &[f32]) -> Result<f32>` | `crates/contextra-vector/src/distance/kernel.rs:38` | Berechnet die Distanz auf vorvalidierten f32-Slices. |
| `fn distance_f32_bytes_trusted( &self, metric: DistanceMetric, a: &[f32], b_bytes: &[u8], ) -> Result<f32>` | `crates/contextra-vector/src/distance/kernel.rs:41` | Berechnet die Distanz zwischen vorvalidiertem f32-Slice und LE f32 Byte-Slice. |
| `fn cosine_distance(&self, a: &[f32], b: &[f32]) -> Result<f32>` | `crates/contextra-vector/src/distance/kernel.rs:49` | Kosinus-Distanz zwischen zwei f32-Slices. |
| `fn dot_product_distance(&self, a: &[f32], b: &[f32]) -> Result<f32>` | `crates/contextra-vector/src/distance/kernel.rs:52` | Skalarprodukt-Distanz (invertiertes Skalarprodukt) zwischen zwei f32-Slices. |
| `fn euclidean_distance(&self, a: &[f32], b: &[f32]) -> Result<f32>` | `crates/contextra-vector/src/distance/kernel.rs:55` | Euklidische Distanz zwischen zwei f32-Slices. |
| `fn euclidean_distance_sq_u8(&self, a: &[u8], b: &[u8]) -> u32` | `crates/contextra-vector/src/distance/kernel.rs:58` | Quadratische Euklidische Distanz zwischen zwei u8-Slices. |
| `fn euclidean_distance_sq_f32_u8( &self, a: &[f32], b: &[u8], alphas: &[f32], mins: &[f32], ) -> f32` | `crates/contextra-vector/src/distance/kernel.rs:61` | Gemischte f32/u8 quadratische Euklidische Distanz mit Rekonstruktionsparametern. |
| `fn dot_product_u8(&self, a: &[u8], b: &[u8]) -> u32` | `crates/contextra-vector/src/distance/kernel.rs:70` | Ganzzahliges Skalarprodukt zwischen zwei u8-Byte-Slices. |
| `fn dot_product_f32_u8(&self, a: &[f32], b: &[u8]) -> f32` | `crates/contextra-vector/src/distance/kernel.rs:73` | Gemischtes f32/u8 Skalarprodukt. |
| `fn cosine_similarity_parts_u8(&self, a: &[u8], b: &[u8]) -> CosineSimilarityPartsU8` | `crates/contextra-vector/src/distance/kernel.rs:76` | Rohkomponenten für u8 Kosinus-Ähnlichkeit. |
| `fn normalize_inplace(&self, v: &mut [f32])` | `crates/contextra-vector/src/distance/kernel.rs:79` | Normalisiert einen Vektor in-place auf Einheitslänge. |

### Modul `distance::scalar`

#### `struct ScalarDistance`
- Ort: `crates/contextra-vector/src/distance/scalar.rs:351`
```rust
pub struct ScalarDistance
```
Dokumentation: Schnittstellenadapter für [`DistanceFn`] über [`ScalarKernel`].
- Impl-Block `crates/contextra-vector/src/distance/scalar.rs:355`
- Impl-Block (Trait `DistanceFn`) `crates/contextra-vector/src/distance/scalar.rs:387`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `metric` | `ContractMetric` | `crates/contextra-vector/src/distance/scalar.rs:352` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub const fn new(metric: ContractMetric) -> Self` | `crates/contextra-vector/src/distance/scalar.rs:358` | Erzeugt eine neue skalare Distanzfunktion für die angegebene Metrik. |
| `pub fn compute(&self, a: &[f32], b: &[f32]) -> core::result::Result<f32, ContractError>` | `crates/contextra-vector/src/distance/scalar.rs:363` | Berechnet die Distanz zwischen zwei Vektoren und liefert bei ungültigen Eingaben Fehler. |

#### `struct ScalarKernel`
- Ort: `crates/contextra-vector/src/distance/scalar.rs:22`
```rust
pub struct ScalarKernel
```
Dokumentation: Vollständige skalare Referenz-Implementierung mit sicheren Iteratoren.
- Impl-Block `crates/contextra-vector/src/distance/scalar.rs:24`
- Impl-Block (Trait `DistanceKernel`) `crates/contextra-vector/src/distance/scalar.rs:270`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn distance<M: Into<DistanceMetric>>( &self, metric: M, a: &[f32], b: &[f32], ) -> Result<f32>` | `crates/contextra-vector/src/distance/scalar.rs:27` | Inhärente Distanzberechnung mit Unterstützung für alle Typen, die `Into<DistanceMetric>` implementieren. |
| `pub fn distance_f32_bytes<M: Into<DistanceMetric>>( &self, metric: M, a: &[f32], b_bytes: &[u8], ) -> Result<f32>` | `crates/contextra-vector/src/distance/scalar.rs:40` | Inhärente Distanzberechnung auf Byte-Slices mit flexibler Metrik. |
| `pub fn distance_trusted<M: Into<DistanceMetric>>( &self, metric: M, a: &[f32], b: &[f32], ) -> Result<f32>` | `crates/contextra-vector/src/distance/scalar.rs:53` | Inhärente Distanzberechnung auf vorvalidierten Slices. |
| `pub fn distance_f32_bytes_trusted<M: Into<DistanceMetric>>( &self, metric: M, a: &[f32], b_bytes: &[u8], ) -> Result<f32>` | `crates/contextra-vector/src/distance/scalar.rs:76` | Inhärente Distanzberechnung auf vorvalidierten Byte-Slices. |
| `pub fn cosine_distance(&self, a: &[f32], b: &[f32]) -> Result<f32>` | `crates/contextra-vector/src/distance/scalar.rs:137` | Kosinus-Distanz zwischen zwei f32-Slices. |
| `pub fn dot_product_distance(&self, a: &[f32], b: &[f32]) -> Result<f32>` | `crates/contextra-vector/src/distance/scalar.rs:163` | Dot-Produkt-Distanz (invertiertes Skalarprodukt) zwischen zwei f32-Slices. |
| `pub fn euclidean_distance(&self, a: &[f32], b: &[f32]) -> Result<f32>` | `crates/contextra-vector/src/distance/scalar.rs:173` | Euklidische Distanz zwischen zwei f32-Slices. |
| `pub fn euclidean_distance_sq_u8(&self, a: &[u8], b: &[u8]) -> u32` | `crates/contextra-vector/src/distance/scalar.rs:183` | Quadratische Euklidische Distanz zwischen zwei u8-Slices. |
| `pub fn euclidean_distance_sq_f32_u8( &self, a: &[f32], b: &[u8], alphas: &[f32], mins: &[f32], ) -> f32` | `crates/contextra-vector/src/distance/scalar.rs:195` | Gemischte f32/u8 quadratische Euklidische Distanz mit Rekonstruktionsparametern. |
| `pub fn dot_product_u8(&self, a: &[u8], b: &[u8]) -> u32` | `crates/contextra-vector/src/distance/scalar.rs:214` | Ganzzahliges Skalarprodukt zwischen zwei u8-Byte-Slices. |
| `pub fn dot_product_f32_u8(&self, a: &[f32], b: &[u8]) -> f32` | `crates/contextra-vector/src/distance/scalar.rs:225` | Gemischtes f32/u8 Skalarprodukt. |
| `pub fn cosine_similarity_parts_u8(&self, a: &[u8], b: &[u8]) -> CosineSimilarityPartsU8` | `crates/contextra-vector/src/distance/scalar.rs:236` | Rohkomponenten für u8 Kosinus-Ähnlichkeit. |
| `pub fn normalize_inplace(&self, v: &mut [f32])` | `crates/contextra-vector/src/distance/scalar.rs:259` | Normalisiert einen Vektor in-place auf Einheitslänge. |

#### `fn cosine_distance_scalar`
- Ort: `crates/contextra-vector/src/distance/scalar.rs:397`
```rust
pub fn cosine_distance_scalar(a: &[f32], b: &[f32]) -> f32
```
Dokumentation: Skalare Kosinus-Distanz als Referenz.

#### `fn dot_product_scalar`
- Ort: `crates/contextra-vector/src/distance/scalar.rs:403`
```rust
pub fn dot_product_scalar(a: &[f32], b: &[f32]) -> f32
```
Dokumentation: Skalares Dot-Produkt (invertiert) als Referenz.

#### `fn euclidean_distance_scalar`
- Ort: `crates/contextra-vector/src/distance/scalar.rs:409`
```rust
pub fn euclidean_distance_scalar(a: &[f32], b: &[f32]) -> f32
```
Dokumentation: Skalare Euklidische Distanz als Referenz.

### Modul `distance::u8_ops`

#### `fn cosine_similarity_parts_u8`
- Ort: `crates/contextra-vector/src/distance/u8_ops.rs:40`
```rust
pub fn cosine_similarity_parts_u8(a: &[u8], b: &[u8]) -> CosineSimilarityPartsU8
```
Dokumentation: Berechnet die ganzzahligen Summen-Komponenten für Kosinus-Ähnlichkeit auf u8-Vektoren.

#### `fn dot_product_f32_u8`
- Ort: `crates/contextra-vector/src/distance/u8_ops.rs:34`
```rust
pub fn dot_product_f32_u8(a: &[f32], b: &[u8]) -> f32
```
Dokumentation: Gemischtes Skalarprodukt zwischen f32-Query und quantisiertem u8-Vektor.

#### `fn dot_product_u8`
- Ort: `crates/contextra-vector/src/distance/u8_ops.rs:28`
```rust
pub fn dot_product_u8(a: &[u8], b: &[u8]) -> u32
```
Dokumentation: Ganzzahliges Skalarprodukt zwischen zwei u8-Vektoren.

#### `fn euclidean_distance_sq_f32_u8`
- Ort: `crates/contextra-vector/src/distance/u8_ops.rs:22`
```rust
pub fn euclidean_distance_sq_f32_u8(a: &[f32], b: &[u8], alphas: &[f32], mins: &[f32]) -> f32
```
Dokumentation: Gemischte quadratische Euklidische Distanz zwischen f32-Query und quantisiertem u8-Vektor.

#### `fn euclidean_distance_sq_u8`
- Ort: `crates/contextra-vector/src/distance/u8_ops.rs:16`
```rust
pub fn euclidean_distance_sq_u8(a: &[u8], b: &[u8]) -> u32
```
Dokumentation: Quadratische Euklidische Distanz zwischen zwei u8-Vektoren.

#### `fn normalize_inplace`
- Ort: `crates/contextra-vector/src/distance/u8_ops.rs:46`
```rust
pub fn normalize_inplace(v: &mut [f32])
```
Dokumentation: In-place Normalisierung eines Vektors auf Einheitslänge.

### Modul `distance::validate`

#### `fn validate_vector`
- Ort: `crates/contextra-vector/src/distance/validate.rs:16`
```rust
pub fn validate_vector(vec: &[f32]) -> Result<()>
```
Dokumentation: Prüft, ob ein Vektor frei von NaN oder Unendlichkeitswerten ist (INV-SIMD-FINITE-1).

### Modul `error`

#### `enum VectorError`
- Ort: `crates/contextra-vector/src/error.rs:20`
```rust
pub enum VectorError
```
Dokumentation: Fehlerzustände von `contextra-vector`.
- Impl-Block (Trait `From<contextra_types::ContextraError>`) `crates/contextra-vector/src/compat/error.rs:41`
- Impl-Block `crates/contextra-vector/src/error.rs:78`
- Impl-Block (Trait `From<std::io::Error>`) `crates/contextra-vector/src/error.rs:89`

Varianten:

- `InvalidInput(String)` — `crates/contextra-vector/src/error.rs:23`
- `Index(String)` — `crates/contextra-vector/src/error.rs:27`
- `Storage(String)` — `crates/contextra-vector/src/error.rs:31`
- `DimensionMismatch{` — `crates/contextra-vector/src/error.rs:35`
- `ConnectivityDegraded{` — `crates/contextra-vector/src/error.rs:44`
- `GraphRepairFailed(String)` — `crates/contextra-vector/src/error.rs:51`
- `NotFound(String)` — `crates/contextra-vector/src/error.rs:55`
- `Conflict(String)` — `crates/contextra-vector/src/error.rs:59`
- `Concurrency(String)` — `crates/contextra-vector/src/error.rs:63`
- `Io(String)` — `crates/contextra-vector/src/error.rs:67`
- `Unsupported(String)` — `crates/contextra-vector/src/error.rs:71`
- `Internal(String)` — `crates/contextra-vector/src/error.rs:75`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn invalid_input(msg: impl Into<String>) -> Self` | `crates/contextra-vector/src/error.rs:80` | Hilfsmethode zur Konstruktion von `InvalidInput`. |

#### `type Result`
- Ort: `crates/contextra-vector/src/error.rs:87`
```rust
pub type Result<T> = std::result::Result<T, VectorError>
```
Dokumentation: Standardmäßiger Result-Typ für `contextra-vector`.

### Modul `fixed_hasher`

#### `fn fixed_hasher`
- Ort: `crates/contextra-vector/src/fixed_hasher.rs:26`
```rust
pub fn fixed_hasher() -> RandomState
```
Dokumentation: Liefert einen deterministischen `RandomState` Build-Hasher mit fixierten Seeds.

#### `fn new_fixed_hashmap`
- Ort: `crates/contextra-vector/src/fixed_hasher.rs:46`
```rust
pub fn new_fixed_hashmap<K, V>(capacity: usize) -> FixedHashMap<K, V>
```
Dokumentation: Erzeugt eine neue deterministische `FixedHashMap` mit vorgegebener Kapazität.

#### `fn new_fixed_hashset`
- Ort: `crates/contextra-vector/src/fixed_hasher.rs:41`
```rust
pub fn new_fixed_hashset<T>(capacity: usize) -> FixedHashSet<T>
```
Dokumentation: Erzeugt ein neues deterministisches `FixedHashSet` mit vorgegebener Kapazität.

#### `type FixedHashMap`
- Ort: `crates/contextra-vector/src/fixed_hasher.rs:38`
```rust
pub type FixedHashMap<K, V> = HashMap<K, V, RandomState>
```
Dokumentation: Deterministische `HashMap` mit fixem Hasher.

#### `type FixedHashSet`
- Ort: `crates/contextra-vector/src/fixed_hasher.rs:35`
```rust
pub type FixedHashSet<T> = HashSet<T, RandomState>
```
Dokumentation: Deterministisches `HashSet` mit fixem Hasher.

### Modul `hnsw`

#### `mod adaptive_ef`
- Ort: `crates/contextra-vector/src/hnsw/mod.rs:23`
```rust
pub mod adaptive_ef
```

#### `mod arena`
- Ort: `crates/contextra-vector/src/hnsw/mod.rs:26`
```rust
pub mod arena
```

#### `mod clock`
- Ort: `crates/contextra-vector/src/hnsw/mod.rs:29`
```rust
pub mod clock
```

#### `mod deletion`
- Ort: `crates/contextra-vector/src/hnsw/mod.rs:32`
```rust
pub mod deletion
```

#### `mod sq8_bias`
- Ort: `crates/contextra-vector/src/hnsw/mod.rs:35`
```rust
pub mod sq8_bias
```

#### `mod staging`
- Ort: `crates/contextra-vector/src/hnsw/mod.rs:56`
```rust
pub mod staging
```

### Modul `hnsw::adaptive_ef`

#### `struct AdaptiveEfPolicy`
- Ort: `crates/contextra-vector/src/hnsw/adaptive_ef.rs:11`
```rust
pub struct AdaptiveEfPolicy
```
Dokumentation: Configuration policy for adaptive ef_search (Ada-ef).
- Impl-Block `crates/contextra-vector/src/hnsw/adaptive_ef.rs:24`
- Impl-Block (Trait `Default`) `crates/contextra-vector/src/hnsw/adaptive_ef.rs:95`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `min_ef` | `usize` | `crates/contextra-vector/src/hnsw/adaptive_ef.rs:13` |
| `max_ef` | `usize` | `crates/contextra-vector/src/hnsw/adaptive_ef.rs:15` |
| `growth_factor` | `f32` | `crates/contextra-vector/src/hnsw/adaptive_ef.rs:17` |
| `convergence_window` | `usize` | `crates/contextra-vector/src/hnsw/adaptive_ef.rs:19` |
| `stability_threshold` | `f32` | `crates/contextra-vector/src/hnsw/adaptive_ef.rs:21` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new( min_ef: usize, max_ef: usize, growth_factor: f32, convergence_window: usize, stability_threshold: f32, ) -> Result<Self>` | `crates/contextra-vector/src/hnsw/adaptive_ef.rs:26` | Validating constructor for `AdaptiveEfPolicy`. |
| `pub fn validate(&self) -> Result<()>` | `crates/contextra-vector/src/hnsw/adaptive_ef.rs:66` | Validates policy invariants. |

#### `struct AdaptiveEfStateMachine`
- Ort: `crates/contextra-vector/src/hnsw/adaptive_ef.rs:120`
```rust
pub struct AdaptiveEfStateMachine
```
Dokumentation: Deterministic state machine managing Ada-ef expansion rounds.
- Impl-Block `crates/contextra-vector/src/hnsw/adaptive_ef.rs:131`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(policy: AdaptiveEfPolicy, k: usize) -> Self` | `crates/contextra-vector/src/hnsw/adaptive_ef.rs:133` | Initializes a new state machine for adaptive search with target top-k. |
| `pub fn current_ef(&self) -> usize` | `crates/contextra-vector/src/hnsw/adaptive_ef.rs:148` | Returns the active `ef_search` for the current expansion round. |
| `pub fn round_count(&self) -> usize` | `crates/contextra-vector/src/hnsw/adaptive_ef.rs:153` | Returns the number of expansion rounds completed. |
| `pub fn is_terminated(&self) -> bool` | `crates/contextra-vector/src/hnsw/adaptive_ef.rs:158` | Returns whether the state machine has reached a termination condition. |
| `pub fn stats(&self) -> AdaptiveEfStats` | `crates/contextra-vector/src/hnsw/adaptive_ef.rs:163` | Returns the current execution statistics. |
| `pub fn step(&mut self, current_top_k: &[DocId]) -> bool` | `crates/contextra-vector/src/hnsw/adaptive_ef.rs:175` | Evaluates current top-k candidate set, updating state and determining if search should terminate. Returns `true` if search should terminate (due to stability co |

#### `struct AdaptiveEfStats`
- Ort: `crates/contextra-vector/src/hnsw/adaptive_ef.rs:109`
```rust
pub struct AdaptiveEfStats
```
Dokumentation: Execution statistics for an adaptive search run.

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `final_ef` | `usize` | `crates/contextra-vector/src/hnsw/adaptive_ef.rs:111` |
| `rounds` | `usize` | `crates/contextra-vector/src/hnsw/adaptive_ef.rs:113` |
| `converged` | `bool` | `crates/contextra-vector/src/hnsw/adaptive_ef.rs:115` |

### Modul `hnsw::arena`

#### `struct BacklinkTable`
- Ort: `crates/contextra-vector/src/hnsw/arena.rs:31`
```rust
pub struct BacklinkTable
```
Dokumentation: Deterministic backlink connection table using `BTreeMap`.
- Impl-Block `crates/contextra-vector/src/hnsw/arena.rs:35`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new() -> Self` | `crates/contextra-vector/src/hnsw/arena.rs:37` | Creates a new `BacklinkTable`. |
| `pub fn get(&self, neighbor_ram_idx: usize, layer: usize) -> Option<&Vec<u32>>` | `crates/contextra-vector/src/hnsw/arena.rs:45` | Gets connection list for (neighbor_ram_idx, layer) in O(1) time. |
| `pub fn insert(&mut self, neighbor_ram_idx: usize, layer: usize, connections: Vec<u32>)` | `crates/contextra-vector/src/hnsw/arena.rs:51` | Inserts or updates connection list for (neighbor_ram_idx, layer) in O(1) time. |
| `pub fn clear(&mut self)` | `crates/contextra-vector/src/hnsw/arena.rs:56` | Clears all stored backlinks. |

#### `struct HnswArena`
- Ort: `crates/contextra-vector/src/hnsw/arena.rs:67`
```rust
pub struct HnswArena
```
Dokumentation: Contiguous 64-byte aligned arena allocator for HNSW node connections with free-list slot reuse.
- Impl-Block (Trait `Default`) `crates/contextra-vector/src/hnsw/arena.rs:77`
- Impl-Block `crates/contextra-vector/src/hnsw/arena.rs:83`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `arena` | `RwLock<Vec<u32>>` | `crates/contextra-vector/src/hnsw/arena.rs:68` |
| `offsets` | `RwLock<Vec<usize>>` | `crates/contextra-vector/src/hnsw/arena.rs:69` |
| `capacities` | `RwLock<Vec<usize>>` | `crates/contextra-vector/src/hnsw/arena.rs:70` |
| `count_offsets` | `RwLock<Vec<usize>>` | `crates/contextra-vector/src/hnsw/arena.rs:71` |
| `counts` | `RwLock<Vec<u8>>` | `crates/contextra-vector/src/hnsw/arena.rs:72` |
| `free_list` | `RwLock<Vec<usize>>` | `crates/contextra-vector/src/hnsw/arena.rs:73` |
| `total_allocated` | `AtomicU64` | `crates/contextra-vector/src/hnsw/arena.rs:74` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new() -> Self` | `crates/contextra-vector/src/hnsw/arena.rs:85` | Creates a new `HnswArena`. |
| `pub fn layer_offset(node_offset: usize, layer: usize, m: usize) -> usize` | `crates/contextra-vector/src/hnsw/arena.rs:99` | Calculates layer offset within a node's allocated arena block. |
| `pub fn align_capacity(raw_capacity: usize) -> usize` | `crates/contextra-vector/src/hnsw/arena.rs:109` | Rounds up an allocation size in u32s to the nearest 64-byte (16 x u32) boundary. |
| `pub fn allocate_node( &self, max_layer: usize, m: usize, final_connections: &[Vec<u32>], ) -> Result<usize>` | `crates/contextra-vector/src/hnsw/arena.rs:116` |  |
| `pub fn get_ram_node_connections(&self, ram_idx: usize, layer: usize, m: usize) -> Vec<u32>` | `crates/contextra-vector/src/hnsw/arena.rs:201` |  |
| `pub fn update_backlink( &self, ram_idx: usize, layer: usize, m: usize, updated_connections: &[u32], ) -> Result<()>` | `crates/contextra-vector/src/hnsw/arena.rs:239` |  |
| `pub fn try_relink_pruned_neighbors( &self, ram_idx: usize, layer: usize, m: usize, kept_neighbors: &[u32], ) -> bool` | `crates/contextra-vector/src/hnsw/arena.rs:280` |  |
| `pub fn zeroize_node_connections(&self, ram_idx: usize, max_layer: usize, m: usize)` | `crates/contextra-vector/src/hnsw/arena.rs:319` |  |
| `pub fn zeroize_and_free_node(&self, ram_idx: usize, max_layer: usize, m: usize)` | `crates/contextra-vector/src/hnsw/arena.rs:350` | Zeroizes connection memory for a node slot across all layers, clears counts, and then marks the node's slot as free in the free_list. |
| `pub fn free_node(&self, ram_idx: usize)` | `crates/contextra-vector/src/hnsw/arena.rs:356` | Marks a node's slot as free for future slot reuse. |

#### `type BacklinkKey`
- Ort: `crates/contextra-vector/src/hnsw/arena.rs:23`
```rust
pub type BacklinkKey = (usize, usize)
```
Dokumentation: Composite key (neighbor_ram_idx, layer) for backlink lookups.

#### `const ARENA_ALIGNMENT_BYTES`
- Ort: `crates/contextra-vector/src/hnsw/arena.rs:18`
```rust
pub const ARENA_ALIGNMENT_BYTES: usize = 64
```
Dokumentation: Slot alignment in bytes (64-byte alignment = 16 u32 elements for SIMD cache-line compatibility).

#### `const ARENA_ALIGNMENT_U32`
- Ort: `crates/contextra-vector/src/hnsw/arena.rs:20`
```rust
pub const ARENA_ALIGNMENT_U32: usize = ARENA_ALIGNMENT_BYTES / std::mem::size_of::<u32>()
```
Dokumentation: Alignment in u32 elements (64 bytes / 4 bytes = 16 u32 elements).

### Modul `hnsw::batch`

#### `struct BatchContext`
- Ort: `crates/contextra-vector/src/hnsw/batch.rs:30`
```rust
pub struct BatchContext
```
Dokumentation: Batch tracking context for running state across multi-operation transaction commits.
- Impl-Block `crates/contextra-vector/src/hnsw/batch.rs:37`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `running_max_layer` | `usize` | `crates/contextra-vector/src/hnsw/batch.rs:31` |
| `has_entry_point` | `bool` | `crates/contextra-vector/src/hnsw/batch.rs:32` |
| `has_ram_entry_point` | `bool` | `crates/contextra-vector/src/hnsw/batch.rs:33` |
| `backlink_map` | `BacklinkTable` | `crates/contextra-vector/src/hnsw/batch.rs:34` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(core: &HnswIndexCore) -> Self` | `crates/contextra-vector/src/hnsw/batch.rs:38` |  |

#### `struct NeighborBacklink`
- Ort: `crates/contextra-vector/src/hnsw/batch.rs:22`
```rust
pub struct NeighborBacklink
```
Dokumentation: Back-link connection update for a neighbor node at a specific layer.

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `neighbor_ram_idx` | `usize` | `crates/contextra-vector/src/hnsw/batch.rs:23` |
| `layer` | `usize` | `crates/contextra-vector/src/hnsw/batch.rs:24` |
| `updated_connections` | `Vec<u32>` | `crates/contextra-vector/src/hnsw/batch.rs:25` |

#### `struct PreparedInsert`
- Ort: `crates/contextra-vector/src/hnsw/batch.rs:9`
```rust
pub struct PreparedInsert
```

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `doc_id` | `DocId` | `crates/contextra-vector/src/hnsw/batch.rs:10` |
| `vector_data` | `VectorData` | `crates/contextra-vector/src/hnsw/batch.rs:11` |
| `new_layer` | `usize` | `crates/contextra-vector/src/hnsw/batch.rs:12` |
| `new_idx` | `usize` | `crates/contextra-vector/src/hnsw/batch.rs:13` |
| `final_connections` | `Vec<Vec<u32>>` | `crates/contextra-vector/src/hnsw/batch.rs:14` |
| `neighbor_backlinks` | `Vec<NeighborBacklink>` | `crates/contextra-vector/src/hnsw/batch.rs:15` |
| `should_update_entry_point` | `bool` | `crates/contextra-vector/src/hnsw/batch.rs:16` |
| `should_update_ram_entry_point` | `bool` | `crates/contextra-vector/src/hnsw/batch.rs:17` |

### Modul `hnsw::clock`

#### `struct MonotonicCounterClock`
- Ort: `crates/contextra-vector/src/hnsw/clock.rs:17`
```rust
pub struct MonotonicCounterClock
```
Dokumentation: Deterministic monotonic counter clock for HNSW vector index. Implements [`Clock`] without any reference to system wall-clock or non-deterministic OS time sources.
- Impl-Block `crates/contextra-vector/src/hnsw/clock.rs:21`
- Impl-Block (Trait `Default`) `crates/contextra-vector/src/hnsw/clock.rs:35`
- Impl-Block (Trait `Clock`) `crates/contextra-vector/src/hnsw/clock.rs:41`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(initial_nanos: u64) -> Self` | `crates/contextra-vector/src/hnsw/clock.rs:23` | Creates a new [`MonotonicCounterClock`] starting at the given nanosecond timestamp. |
| `pub fn advance_nanos(&self, delta_nanos: u64)` | `crates/contextra-vector/src/hnsw/clock.rs:30` | Advances the clock by `delta_nanos`. |

### Modul `hnsw::config`

#### `struct HnswConfig`
- Ort: `crates/contextra-vector/src/hnsw/config.rs:17`
```rust
pub struct HnswConfig
```
Dokumentation: Konfiguration für den HNSW-Graph-Index.
- Impl-Block (Trait `Default`) `crates/contextra-vector/src/hnsw/config.rs:51`
- Impl-Block `crates/contextra-vector/src/hnsw/config.rs:79`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `dimension` | `usize` | `crates/contextra-vector/src/hnsw/config.rs:19` |
| `max_elements` | `usize` | `crates/contextra-vector/src/hnsw/config.rs:21` |
| `m` | `usize` | `crates/contextra-vector/src/hnsw/config.rs:23` |
| `ef_construction` | `usize` | `crates/contextra-vector/src/hnsw/config.rs:25` |
| `ef_search` | `usize` | `crates/contextra-vector/src/hnsw/config.rs:27` |
| `distance_metric` | `DistanceMetric` | `crates/contextra-vector/src/hnsw/config.rs:29` |
| `rebuild_threshold` | `f64` | `crates/contextra-vector/src/hnsw/config.rs:31` |
| `backoff_base_delay` | `std::time::Duration` | `crates/contextra-vector/src/hnsw/config.rs:33` |
| `backoff_max_delay` | `std::time::Duration` | `crates/contextra-vector/src/hnsw/config.rs:35` |
| `backoff_alert_threshold` | `u32` | `crates/contextra-vector/src/hnsw/config.rs:37` |
| `quantize` | `bool` | `crates/contextra-vector/src/hnsw/config.rs:39` |
| `quantizer_recalibration_sample_size` | `usize` | `crates/contextra-vector/src/hnsw/config.rs:41` |
| `quantizer_drift_threshold` | `f32` | `crates/contextra-vector/src/hnsw/config.rs:43` |
| `compute_pool` | `Option<crate::compute_pool::ComputePool>` | `crates/contextra-vector/src/hnsw/config.rs:45` |
| `partial_rebuild_config` | `crate::partial_rebuild::PartialRebuildConfig` | `crates/contextra-vector/src/hnsw/config.rs:48` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn validate(&self) -> Result<()>` | `crates/contextra-vector/src/hnsw/config.rs:82` |  |

#### `struct HnswConfigBuilder`
- Ort: `crates/contextra-vector/src/hnsw/config.rs:124`
```rust
pub struct HnswConfigBuilder
```
Dokumentation: Builder for HnswConfig with resource limit enforcements to prevent OOM.
- Impl-Block `crates/contextra-vector/src/hnsw/config.rs:128`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(dimension: usize) -> Self` | `crates/contextra-vector/src/hnsw/config.rs:130` | Creates a new builder with the chosen dimensionality. |
| `pub fn max_elements(mut self, max: usize) -> Self` | `crates/contextra-vector/src/hnsw/config.rs:140` | Set max elements with a hardcap limit to avoid OOM. |
| `pub fn m(mut self, m: usize) -> Self` | `crates/contextra-vector/src/hnsw/config.rs:146` | Set the number of connections per element (M). |
| `pub fn ef_construction(mut self, ef: usize) -> Self` | `crates/contextra-vector/src/hnsw/config.rs:152` | Set dynamic candidate list size for construction. |
| `pub fn ef_search(mut self, ef: usize) -> Self` | `crates/contextra-vector/src/hnsw/config.rs:158` | Set dynamic candidate list size for search. |
| `pub fn distance_metric(mut self, metric: DistanceMetric) -> Self` | `crates/contextra-vector/src/hnsw/config.rs:164` | Use a specific distance metric. |
| `pub fn quantize(mut self, quantize: bool) -> Self` | `crates/contextra-vector/src/hnsw/config.rs:170` | Enable or disable scalar quantization (SQ8) to reduce footprint. |
| `pub fn quantizer_recalibration_sample_size(mut self, size: usize) -> Self` | `crates/contextra-vector/src/hnsw/config.rs:176` | Sets the sample size used for ScalarQuantizer recalibration during rebuilds. |
| `pub fn quantizer_drift_threshold(mut self, threshold: f32) -> Self` | `crates/contextra-vector/src/hnsw/config.rs:182` | Sets the drift ratio threshold for ScalarQuantizer recalibration and rebuilds. |
| `pub fn rebuild_threshold(mut self, threshold: f64) -> Self` | `crates/contextra-vector/src/hnsw/config.rs:188` | Sets the rebuild threshold. |
| `pub fn backoff_base_delay(mut self, delay: std::time::Duration) -> Self` | `crates/contextra-vector/src/hnsw/config.rs:194` | Sets the base delay for rebuild exponential backoff cooldown. |
| `pub fn backoff_max_delay(mut self, delay: std::time::Duration) -> Self` | `crates/contextra-vector/src/hnsw/config.rs:200` | Sets the maximum cap for rebuild exponential backoff cooldown. |
| `pub fn backoff_alert_threshold(mut self, threshold: u32) -> Self` | `crates/contextra-vector/src/hnsw/config.rs:206` | Sets the threshold of consecutive rapid rebuilds that trigger warning alerts. |
| `pub fn partial_rebuild_config( mut self, config: crate::partial_rebuild::PartialRebuildConfig, ) -> Self` | `crates/contextra-vector/src/hnsw/config.rs:213` | Sets the partial rebuild configuration for local hot-path rebuilds (F-02). |
| `pub fn build(self) -> Result<HnswConfig>` | `crates/contextra-vector/src/hnsw/config.rs:223` |  |

### Modul `hnsw::core_rebuild`

#### `mod backoff`
- Ort: `crates/contextra-vector/src/hnsw/core_rebuild/mod.rs:12`
```rust
pub mod backoff
```

#### `mod full`
- Ort: `crates/contextra-vector/src/hnsw/core_rebuild/mod.rs:13`
```rust
pub mod full
```

#### `mod region`
- Ort: `crates/contextra-vector/src/hnsw/core_rebuild/mod.rs:14`
```rust
pub mod region
```

#### `mod seq_replay`
- Ort: `crates/contextra-vector/src/hnsw/core_rebuild/mod.rs:15`
```rust
pub mod seq_replay
```

#### `mod snapshot`
- Ort: `crates/contextra-vector/src/hnsw/core_rebuild/mod.rs:16`
```rust
pub mod snapshot
```

### Modul `hnsw::core_rebuild::backoff`

#### `struct BackoffTracker`
- Ort: `crates/contextra-vector/src/hnsw/core_rebuild/backoff.rs:30`
```rust
pub struct BackoffTracker
```
Dokumentation: Autarker Backoff-Zustandsautomat zur Ratenbegrenzung von Rebuild-Vorgängen.
- Impl-Block `crates/contextra-vector/src/hnsw/core_rebuild/backoff.rs:38`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(base_delay: Duration, max_delay: Duration, alert_threshold: u64) -> Self` | `crates/contextra-vector/src/hnsw/core_rebuild/backoff.rs:40` | Erstellt einen neuen BackoffTracker mit den konfigurierten Parametern. |
| `pub fn current_cooldown(&self) -> Duration` | `crates/contextra-vector/src/hnsw/core_rebuild/backoff.rs:51` | Berechnet die aktuelle Cooldown-Dauer anhand der Anzahl aufeinanderfolgender Rebuilds. |
| `pub fn check_and_update(&mut self, now_nanos: u64) -> Result<()>` | `crates/contextra-vector/src/hnsw/core_rebuild/backoff.rs:63` | Prüft, ob ein Rebuild zum Zeitpunkt `now_nanos` erlaubt ist, und aktualisiert den internen Zustand. |
| `pub fn record_success(&mut self, now_nanos: u64)` | `crates/contextra-vector/src/hnsw/core_rebuild/backoff.rs:95` | Markiert einen erfolgreichen Rebuild zum Zeitpunkt `now_nanos` und setzt den Zähler zurück. |
| `pub fn consecutive_rapid_rebuilds(&self) -> u64` | `crates/contextra-vector/src/hnsw/core_rebuild/backoff.rs:101` | Liefert die Anzahl der aufeinanderfolgenden schnellen Rebuild-Versuche. |
| `pub fn last_rebuild_nanos(&self) -> Option<u64>` | `crates/contextra-vector/src/hnsw/core_rebuild/backoff.rs:106` | Liefert den Zeitstempel des letzten erfolgreichen Rebuilds. |

### Modul `hnsw::deletion`

#### `struct DeletionStats`
- Ort: `crates/contextra-vector/src/hnsw/deletion.rs:50`
```rust
pub struct DeletionStats
```
Dokumentation: Statistics returned after a ghost-free node deletion and neighborhood repair.

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `doc_id` | `DocId` | `crates/contextra-vector/src/hnsw/deletion.rs:52` |
| `repaired_edges` | `usize` | `crates/contextra-vector/src/hnsw/deletion.rs:54` |
| `orphaned_replacements` | `usize` | `crates/contextra-vector/src/hnsw/deletion.rs:56` |
| `verified_no_ghost_pointers` | `bool` | `crates/contextra-vector/src/hnsw/deletion.rs:58` |

#### `trait GhostFreeVectorIndex`
- Ort: `crates/contextra-vector/src/hnsw/deletion.rs:63`
```rust
pub trait GhostFreeVectorIndex
```
Dokumentation: Trait providing ghost-free vector index deletion with synchronous neighborhood graph repair.

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn remove_with_graph_repair(&mut self, doc_id: DocId) -> Result<DeletionStats>` | `crates/contextra-vector/src/hnsw/deletion.rs:70` | Deletes `doc_id` and synchronously repairs all neighborhood pointers across all layers before returning. # Invariant `INV-DELETION-2`: After `Ok(stats)` returns |

#### `trait IntoDocId`
- Ort: `crates/contextra-vector/src/hnsw/deletion.rs:21`
```rust
pub trait IntoDocId
```
Dokumentation: Trait for types convertible to [`DocId`] for deletion APIs.

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn into_doc_id(self) -> DocId` | `crates/contextra-vector/src/hnsw/deletion.rs:23` | Converts the type into a [`DocId`]. |

### Modul `hnsw::sq8_bias`

#### `struct Sq8Bias`
- Ort: `crates/contextra-vector/src/hnsw/sq8_bias.rs:11`
```rust
pub struct Sq8Bias
```
Dokumentation: Measured quantization bias and variance statistics for SQ8 distance calculations.
- Impl-Block (Trait `Default`) `crates/contextra-vector/src/hnsw/sq8_bias.rs:20`
- Impl-Block `crates/contextra-vector/src/hnsw/sq8_bias.rs:30`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `mean_bias` | `f32` | `crates/contextra-vector/src/hnsw/sq8_bias.rs:13` |
| `variance_bias` | `f32` | `crates/contextra-vector/src/hnsw/sq8_bias.rs:15` |
| `sample_count` | `u64` | `crates/contextra-vector/src/hnsw/sq8_bias.rs:17` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(mean_bias: f32, variance_bias: f32, sample_count: u64) -> Self` | `crates/contextra-vector/src/hnsw/sq8_bias.rs:32` | Creates a new `Sq8Bias` instance with calibrated parameters. |
| `pub fn calibrate( batch: &[&[f32]], quantizer: &crate::quantize::ScalarQuantizer, metric: crate::types::DistanceMetric, ) -> Self` | `crates/contextra-vector/src/hnsw/sq8_bias.rs:41` | Calibrates quantization error statistics across a representative batch of vectors. |

### Modul `hnsw::staging`

#### `struct VectorTxBuffer`
- Ort: `crates/contextra-vector/src/hnsw/staging.rs:24`
```rust
pub struct VectorTxBuffer
```
Dokumentation: Transaction staging buffer for vector operations. Encapsulates transactional staging locally, isolating from direct coupling to storage transaction buffers.
- Impl-Block `crates/contextra-vector/src/hnsw/staging.rs:31`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(capacity: usize, timeout: Duration) -> Self` | `crates/contextra-vector/src/hnsw/staging.rs:33` | Creates a new vector staging buffer with the specified concurrency ceiling and timeout. |
| `pub fn stage(&self, tx: TxId, op: VectorStagingOp) -> crate::error::Result<()>` | `crates/contextra-vector/src/hnsw/staging.rs:42` | Stages an operation into the transaction buffer. |
| `pub fn drain(&self, tx: TxId) -> Vec<VectorStagingOp>` | `crates/contextra-vector/src/hnsw/staging.rs:55` | Drains and returns all staged operations for a given transaction. |
| `pub fn discard(&self, tx: TxId)` | `crates/contextra-vector/src/hnsw/staging.rs:60` | Discards staged operations on transaction rollback. |

#### `enum VectorStagingOp`
- Ort: `crates/contextra-vector/src/hnsw/staging.rs:12`
```rust
pub enum VectorStagingOp
```
Dokumentation: Operation within a transactional staging buffer.

Varianten:

- `Insert{ doc_id: DocId, embedding: Vec<f32> }` — `crates/contextra-vector/src/hnsw/staging.rs:14`
- `Delete{ doc_id: DocId }` — `crates/contextra-vector/src/hnsw/staging.rs:16`

### Modul `hnsw::types::core`

#### `struct HnswColdCore`
- Ort: `crates/contextra-vector/src/hnsw/types/core.rs:91`
```rust
pub struct HnswColdCore
```
Dokumentation: Kalter Kern des HNSW-Index (Konfiguration, Puffer, Metriken, Quantisierer).

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `config` | `HnswConfig` | `crates/contextra-vector/src/hnsw/types/core.rs:92` |
| `validation_error` | `Option<String>` | `crates/contextra-vector/src/hnsw/types/core.rs:93` |
| `tx_buffer` | `crate::hnsw::staging::VectorTxBuffer` | `crates/contextra-vector/src/hnsw/types/core.rs:94` |
| `quantizer` | `RwLock<Option<crate::quantize::ScalarQuantizer>>` | `crates/contextra-vector/src/hnsw/types/core.rs:95` |
| `sq8_bias` | `RwLock<Sq8Bias>` | `crates/contextra-vector/src/hnsw/types/core.rs:96` |
| `mmap_index` | `RwLock<Option<crate::persistence::MmapIndex>>` | `crates/contextra-vector/src/hnsw/types/core.rs:97` |
| `seq_log` | `RwLock<contextra_mvcc::SequenceLog>` | `crates/contextra-vector/src/hnsw/types/core.rs:98` |
| `rebuild_count` | `AtomicU64` | `crates/contextra-vector/src/hnsw/types/core.rs:99` |
| `visited_dead_nodes` | `AtomicU64` | `crates/contextra-vector/src/hnsw/types/core.rs:100` |
| `deleted_nodes` | `RwLock<RoaringTreemap>` | `crates/contextra-vector/src/hnsw/types/core.rs:101` |
| `compute_pool` | `crate::compute_pool::ComputePool` | `crates/contextra-vector/src/hnsw/types/core.rs:102` |
| `rng` | `std::sync::Arc<dyn crate::types::Rng>` | `crates/contextra-vector/src/hnsw/types/core.rs:103` |
| `clock` | `RwLock<std::sync::Arc<dyn crate::types::Clock>>` | `crates/contextra-vector/src/hnsw/types/core.rs:104` |
| `metrics_sink` | `RwLock<std::sync::Arc<dyn crate::types::MetricsSink>>` | `crates/contextra-vector/src/hnsw/types/core.rs:105` |
| `last_rebuild_nanos` | `AtomicU64` | `crates/contextra-vector/src/hnsw/types/core.rs:106` |
| `consecutive_rapid_rebuilds` | `AtomicU32` | `crates/contextra-vector/src/hnsw/types/core.rs:107` |
| `traversal_tracker` | `Option<RwLock<crate::partial_rebuild::TraversalTracker>>` | `crates/contextra-vector/src/hnsw/types/core.rs:109` |
| `fault_injection_insert_target` | `AtomicU64` | `crates/contextra-vector/src/hnsw/types/core.rs:111` |
| `fault_injection_insert_count` | `AtomicU64` | `crates/contextra-vector/src/hnsw/types/core.rs:113` |

#### `struct HnswHotCore`
- Ort: `crates/contextra-vector/src/hnsw/types/core.rs:21`
```rust
pub struct HnswHotCore
```
Dokumentation: Heisser Kern des HNSW-Index (fuer Lese- und Schreibpfade im Graph optimiert).
- Impl-Block `crates/contextra-vector/src/hnsw/types/core.rs:41`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `nodes` | `RwLock<Vec<HnswNode>>` | `crates/contextra-vector/src/hnsw/types/core.rs:22` |
| `entry_point` | `AtomicU32` | `crates/contextra-vector/src/hnsw/types/core.rs:27` |
| `ram_entry_point` | `AtomicU32` | `crates/contextra-vector/src/hnsw/types/core.rs:28` |
| `max_layer` | `AtomicU64` | `crates/contextra-vector/src/hnsw/types/core.rs:29` |
| `ml` | `f64` | `crates/contextra-vector/src/hnsw/types/core.rs:30` |
| `layer_seed` | `AtomicU64` | `crates/contextra-vector/src/hnsw/types/core.rs:31` |
| `deleted_count` | `AtomicU64` | `crates/contextra-vector/src/hnsw/types/core.rs:32` |
| `write_mutex` | `Mutex<()>` | `crates/contextra-vector/src/hnsw/types/core.rs:33` |
| `last_tx_id` | `AtomicU64` | `crates/contextra-vector/src/hnsw/types/core.rs:34` |
| `rebuilding` | `AtomicBool` | `crates/contextra-vector/src/hnsw/types/core.rs:35` |
| `rebuild_condvar` | `parking_lot::Condvar` | `crates/contextra-vector/src/hnsw/types/core.rs:36` |
| `rebuild_mutex` | `Mutex<()>` | `crates/contextra-vector/src/hnsw/types/core.rs:37` |
| `arena` | `HnswArena` | `crates/contextra-vector/src/hnsw/types/core.rs:38` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn get_entry_point(&self) -> Option<usize>` | `crates/contextra-vector/src/hnsw/types/core.rs:43` |  |
| `pub fn set_entry_point(&self, ep: Option<usize>)` | `crates/contextra-vector/src/hnsw/types/core.rs:54` |  |
| `pub fn get_ram_entry_point(&self) -> Option<usize>` | `crates/contextra-vector/src/hnsw/types/core.rs:64` |  |
| `pub fn set_ram_entry_point(&self, ep: Option<usize>)` | `crates/contextra-vector/src/hnsw/types/core.rs:75` |  |
| `pub fn get_ram_node_connections(&self, ram_idx: usize, layer: usize, m: usize) -> Vec<u32>` | `crates/contextra-vector/src/hnsw/types/core.rs:84` |  |

#### `struct HnswIndexCore`
- Ort: `crates/contextra-vector/src/hnsw/types/core.rs:118`
```rust
pub struct HnswIndexCore
```
Dokumentation: Die Kern-Implementierung des HNSW-Index aus Hot- und Cold-Strukturen.
- Impl-Block `crates/contextra-vector/src/hnsw/core_insert/apply.rs:17`
- Impl-Block `crates/contextra-vector/src/hnsw/core_insert/compute.rs:21`
- Impl-Block `crates/contextra-vector/src/hnsw/core_insert/connectivity.rs:15`
- Impl-Block `crates/contextra-vector/src/hnsw/core_rebuild/full.rs:16`
- Impl-Block `crates/contextra-vector/src/hnsw/core_rebuild/region.rs:14`
- Impl-Block `crates/contextra-vector/src/hnsw/core_rebuild/snapshot.rs:16`
- Impl-Block `crates/contextra-vector/src/hnsw/core_search/layer.rs:21`
- Impl-Block `crates/contextra-vector/src/hnsw/types/core.rs:123`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `hot` | `HnswHotCore` | `crates/contextra-vector/src/hnsw/types/core.rs:119` |
| `cold` | `HnswColdCore` | `crates/contextra-vector/src/hnsw/types/core.rs:120` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn apply_insert(&self, prepared: PreparedInsert, tx_id: u64)` | `crates/contextra-vector/src/hnsw/core_insert/apply.rs:19` |  |
| `pub fn compute_insert(&self, id: DocId, vector: &[f32]) -> Result<PreparedInsert>` | `crates/contextra-vector/src/hnsw/core_insert/compute.rs:70` |  |
| `pub fn compute_insert_with_context( &self, id: DocId, vector: &[f32], offset: usize, prior_prepared: &mut [PreparedInsert], batch_ctx: &mut BatchContext, ) -> Result<PreparedInsert>` | `crates/contextra-vector/src/hnsw/core_insert/compute.rs:76` |  |
| `pub fn connectivity_score(&self) -> f64` | `crates/contextra-vector/src/hnsw/core_insert/connectivity.rs:16` |  |
| `pub fn check_connectivity(&self) -> crate::error::Result<()>` | `crates/contextra-vector/src/hnsw/core_insert/connectivity.rs:41` |  |
| `pub fn rebuild_status(&self) -> RebuildStatus` | `crates/contextra-vector/src/hnsw/core_rebuild/full.rs:18` |  |
| `pub fn wait_for_rebuild_blocking( &self, timeout: std::time::Duration, _clock: &dyn crate::types::Clock, ) -> bool` | `crates/contextra-vector/src/hnsw/core_rebuild/full.rs:27` |  |
| `pub fn wait_for_rebuild(&self) -> bool` | `crates/contextra-vector/src/hnsw/core_rebuild/full.rs:45` |  |
| `pub fn wait_for_rebuild_with_clock(&self, clock: &dyn crate::types::Clock) -> bool` | `crates/contextra-vector/src/hnsw/core_rebuild/full.rs:50` |  |
| `pub fn wait_for_rebuild_with_timeout(&self, timeout: std::time::Duration) -> bool` | `crates/contextra-vector/src/hnsw/core_rebuild/full.rs:54` |  |
| `pub fn wait_for_rebuild_with_timeout_and_clock( &self, timeout: std::time::Duration, clock: &dyn crate::types::Clock, ) -> bool` | `crates/contextra-vector/src/hnsw/core_rebuild/full.rs:59` |  |
| `pub fn deleted_ratio(&self) -> f64` | `crates/contextra-vector/src/hnsw/core_rebuild/full.rs:158` |  |
| `pub fn rebuild_count(&self) -> u64` | `crates/contextra-vector/src/hnsw/core_rebuild/full.rs:162` |  |
| `pub fn visited_dead_nodes(&self) -> u64` | `crates/contextra-vector/src/hnsw/core_rebuild/full.rs:166` |  |
| `pub fn is_rebuild_required(&self) -> bool` | `crates/contextra-vector/src/hnsw/core_rebuild/full.rs:170` |  |
| `pub fn rebuild(&self) -> Result<()>` | `crates/contextra-vector/src/hnsw/core_rebuild/full.rs:184` |  |
| `pub fn rebuild_with_clock(&self, clock: &dyn crate::types::Clock) -> Result<()>` | `crates/contextra-vector/src/hnsw/core_rebuild/full.rs:189` |  |
| `pub fn rebuild_sync(&self) -> Result<()>` | `crates/contextra-vector/src/hnsw/core_rebuild/full.rs:193` |  |
| `pub fn rebuild_sync_with_clock(&self, clock: &dyn crate::types::Clock) -> Result<()>` | `crates/contextra-vector/src/hnsw/core_rebuild/full.rs:199` |  |
| `pub fn rebuild_region(&self, region_node_ids: Vec<u64>) -> Result<()>` | `crates/contextra-vector/src/hnsw/core_rebuild/region.rs:16` | Startet einen partiellen Rebuild für eine bestimmte Menge von Knoten. |
| `pub fn rebuild_region_sync(&self, region_node_ids: Vec<u64>) -> Result<()>` | `crates/contextra-vector/src/hnsw/core_rebuild/region.rs:21` | Führt die lokale Reorganisation einer Region synchron durch. |
| `pub fn set_layer_seed(&self, seed: u64)` | `crates/contextra-vector/src/hnsw/types/core.rs:125` | Sets the random seed used for HNSW layer selection during insertions. |
| `pub fn metrics_sink(&self) -> std::sync::Arc<dyn crate::types::MetricsSink>` | `crates/contextra-vector/src/hnsw/types/core.rs:131` | Returns the metrics sink. |
| `pub fn set_metrics_sink(&self, sink: std::sync::Arc<dyn crate::types::MetricsSink>)` | `crates/contextra-vector/src/hnsw/types/core.rs:136` | Sets the metrics sink. |
| `pub fn clock(&self) -> std::sync::Arc<dyn crate::types::Clock>` | `crates/contextra-vector/src/hnsw/types/core.rs:141` | Returns the clock instance used by this index. |
| `pub fn set_clock(&self, clock: std::sync::Arc<dyn crate::types::Clock>)` | `crates/contextra-vector/src/hnsw/types/core.rs:146` | Sets the clock instance. |

#### `enum RebuildStatus`
- Ort: `crates/contextra-vector/src/hnsw/types/core.rs:153`
```rust
pub enum RebuildStatus
```
Dokumentation: Status eines Index-Rebuilds.

Varianten:

- `Idle` — `crates/contextra-vector/src/hnsw/types/core.rs:155`
- `Running` — `crates/contextra-vector/src/hnsw/types/core.rs:157`
- `Pending` — `crates/contextra-vector/src/hnsw/types/core.rs:159`

### Modul `hnsw::types::data`

#### `struct Candidate`
- Ort: `crates/contextra-vector/src/hnsw/types/data.rs:48`
```rust
pub struct Candidate
```
Dokumentation: Search candidate.
- Impl-Block (Trait `PartialEq`) `crates/contextra-vector/src/hnsw/types/data.rs:53`
- Impl-Block (Trait `Eq`) `crates/contextra-vector/src/hnsw/types/data.rs:58`
- Impl-Block (Trait `PartialOrd`) `crates/contextra-vector/src/hnsw/types/data.rs:60`
- Impl-Block (Trait `Ord`) `crates/contextra-vector/src/hnsw/types/data.rs:66`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `index` | `usize` | `crates/contextra-vector/src/hnsw/types/data.rs:49` |
| `distance` | `f32` | `crates/contextra-vector/src/hnsw/types/data.rs:50` |

#### `struct HnswNode`
- Ort: `crates/contextra-vector/src/hnsw/types/data.rs:38`
```rust
pub struct HnswNode
```
Dokumentation: A node in the HNSW graph.

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `doc_id` | `DocId` | `crates/contextra-vector/src/hnsw/types/data.rs:39` |
| `vector` | `VectorData` | `crates/contextra-vector/src/hnsw/types/data.rs:40` |
| `max_layer` | `usize` | `crates/contextra-vector/src/hnsw/types/data.rs:41` |
| `committed_tx` | `u64` | `crates/contextra-vector/src/hnsw/types/data.rs:42` |

#### `enum VectorData`
- Ort: `crates/contextra-vector/src/hnsw/types/data.rs:28`
```rust
pub enum VectorData
```

Varianten:

- `F32(Vec<f32>)` — `crates/contextra-vector/src/hnsw/types/data.rs:30`
- `U8(Vec<u8>)` — `crates/contextra-vector/src/hnsw/types/data.rs:32`

#### `const DEFAULT_LAYER_SEED`
- Ort: `crates/contextra-vector/src/hnsw/types/data.rs:21`
```rust
pub const DEFAULT_LAYER_SEED: u64 = 100
```
Dokumentation: Default-Seed fuer die deterministische HNSW-Ebenenzuweisung (wie in hnswlib ueblich).

#### `const HNSW_REBUILD_DELETION_RATIO`
- Ort: `crates/contextra-vector/src/hnsw/types/data.rs:17`
```rust
pub const HNSW_REBUILD_DELETION_RATIO: f64 = 0.10
```
Dokumentation: Standard-Loeschanteil (0.10 = 10 % geloeschte Knoten), ab dem ein Rebuild getriggert wird.

#### `const SENTINEL_NO_ENTRY_POINT`
- Ort: `crates/contextra-vector/src/hnsw/types/data.rs:13`
```rust
pub const SENTINEL_NO_ENTRY_POINT: u32 = u32::MAX
```
Dokumentation: Sentinel-Wert fuer ungueltigen / nicht gesetzten Entry-Point in HNSW.

### Modul `hnsw::types::guards`

#### `struct RebuildGuard`
- Ort: `crates/contextra-vector/src/hnsw/types/guards.rs:12`
```rust
pub struct RebuildGuard<'a>(pub &'a AtomicBool, pub &'a parking_lot::Condvar)
```
Dokumentation: Guard, der das `rebuilding`-Flag auf `false` zuruecksetzt und wartende Threads weckt.
- Impl-Block (Trait `Drop`) `crates/contextra-vector/src/hnsw/types/guards.rs:14`

#### `struct SnapshotPinGuard`
- Ort: `crates/contextra-vector/src/hnsw/types/guards.rs:24`
```rust
pub struct SnapshotPinGuard<'a>(pub &'a HnswIndexCore, pub(crate) u64)
```
Dokumentation: Guard, der einen Snapshot im `SequenceLog` pinnt und beim Drop wieder freigibt.
- Impl-Block `crates/contextra-vector/src/hnsw/types/guards.rs:26`
- Impl-Block (Trait `Drop`) `crates/contextra-vector/src/hnsw/types/guards.rs:33`

### Modul `hnsw::types::index`

#### `struct GenericHnswIndex`
- Ort: `crates/contextra-vector/src/hnsw/types/index.rs:21`
```rust
pub struct GenericHnswIndex<K: DistanceKernel = ScalarKernel>
```
Dokumentation: Generischer HNSW (Hierarchical Navigable Small World) Vektorindex parametrisiert ueber DistanceKernel.
- Impl-Block `crates/contextra-vector/src/hnsw/types/index.rs:29`
- Impl-Block `crates/contextra-vector/src/hnsw/types/persist.rs:14`
- Impl-Block `crates/contextra-vector/src/hnsw/types/seqlog.rs:10`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `inner` | `std::sync::Arc<HnswIndexCore>` | `crates/contextra-vector/src/hnsw/types/index.rs:22` |
| `_kernel` | `std::marker::PhantomData<K>` | `crates/contextra-vector/src/hnsw/types/index.rs:23` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn entry_point(&self) -> Option<usize>` | `crates/contextra-vector/src/hnsw/types/index.rs:31` | Gibt den aktuellen globalen Entry-Point-Knotenindex zurueck (falls vorhanden). |
| `pub fn node_idx_for_doc(&self, doc_id: DocId) -> Option<usize>` | `crates/contextra-vector/src/hnsw/types/index.rs:36` | Gibt den internen Knotenindex fuer eine `DocId` zurueck (falls vorhanden und nicht geloescht). |
| `pub fn layer0_connections(&self, node_idx: usize) -> Vec<u32>` | `crates/contextra-vector/src/hnsw/types/index.rs:45` | Gibt die Layer-0-Verbindungen fuer einen Knotenindex zurueck. |
| `pub fn all_doc_ids_from_map(&self) -> Vec<DocId>` | `crates/contextra-vector/src/hnsw/types/index.rs:72` |  |
| `pub fn all_doc_ids_and_layers(&self) -> Vec<(DocId, usize)>` | `crates/contextra-vector/src/hnsw/types/index.rs:82` | Returns all non-deleted document IDs and their assigned HNSW max layers. |
| `pub fn deleted_ratio(&self) -> f64` | `crates/contextra-vector/src/hnsw/types/index.rs:95` |  |
| `pub fn rebuild_count(&self) -> u64` | `crates/contextra-vector/src/hnsw/types/index.rs:99` |  |
| `pub fn visited_dead_nodes(&self) -> u64` | `crates/contextra-vector/src/hnsw/types/index.rs:103` |  |
| `pub fn connectivity_score(&self) -> f64` | `crates/contextra-vector/src/hnsw/types/index.rs:107` |  |
| `pub fn check_connectivity(&self) -> crate::error::Result<()>` | `crates/contextra-vector/src/hnsw/types/index.rs:111` |  |
| `pub fn is_rebuild_required(&self) -> bool` | `crates/contextra-vector/src/hnsw/types/index.rs:115` |  |
| `pub fn rebuild(&self) -> Result<()>` | `crates/contextra-vector/src/hnsw/types/index.rs:119` |  |
| `pub fn rebuild_sync(&self) -> Result<()>` | `crates/contextra-vector/src/hnsw/types/index.rs:123` |  |
| `pub fn rebuild_region(&self, region_node_ids: Vec<u64>) -> Result<()>` | `crates/contextra-vector/src/hnsw/types/index.rs:127` |  |
| `pub fn rebuild_region_sync(&self, region_node_ids: Vec<u64>) -> Result<()>` | `crates/contextra-vector/src/hnsw/types/index.rs:131` |  |
| `pub fn trigger_rebuild_async(&self)` | `crates/contextra-vector/src/hnsw/types/index.rs:135` |  |
| `pub fn clock(&self) -> std::sync::Arc<dyn crate::types::Clock>` | `crates/contextra-vector/src/hnsw/types/index.rs:148` | Returns the clock instance used by this index. |
| `pub fn rebuild_status(&self) -> RebuildStatus` | `crates/contextra-vector/src/hnsw/types/index.rs:152` |  |
| `pub fn wait_for_rebuild_blocking( &self, timeout: std::time::Duration, clock: &dyn crate::types::Clock, ) -> bool` | `crates/contextra-vector/src/hnsw/types/index.rs:156` |  |
| `pub fn wait_for_rebuild(&self) -> bool` | `crates/contextra-vector/src/hnsw/types/index.rs:164` |  |
| `pub fn wait_for_rebuild_with_timeout(&self, timeout: std::time::Duration) -> bool` | `crates/contextra-vector/src/hnsw/types/index.rs:168` |  |
| `pub fn quantizer(&self) -> Option<crate::quantize::ScalarQuantizer>` | `crates/contextra-vector/src/hnsw/types/index.rs:173` | Returns a snapshot clone of the current quantizer if trained. |
| `pub fn sq8_bias(&self) -> Sq8Bias` | `crates/contextra-vector/src/hnsw/types/index.rs:178` | Returns the calibrated SQ8 quantization bias statistics. |
| `pub fn metrics_sink(&self) -> std::sync::Arc<dyn crate::types::MetricsSink>` | `crates/contextra-vector/src/hnsw/types/index.rs:183` | Returns the metrics sink. |
| `pub fn set_metrics_sink(&self, sink: std::sync::Arc<dyn crate::types::MetricsSink>)` | `crates/contextra-vector/src/hnsw/types/index.rs:188` | Sets the metrics sink. |
| `pub fn with_metrics_sink(self, sink: std::sync::Arc<dyn crate::types::MetricsSink>) -> Self` | `crates/contextra-vector/src/hnsw/types/index.rs:193` | Builder method to set the metrics sink. |
| `pub fn set_clock(&self, clock: std::sync::Arc<dyn crate::types::Clock>)` | `crates/contextra-vector/src/hnsw/types/index.rs:199` | Sets the clock instance. |
| `pub fn with_clock(self, clock: std::sync::Arc<dyn crate::types::Clock>) -> Self` | `crates/contextra-vector/src/hnsw/types/index.rs:204` | Builder method to set the clock port. |
| `pub fn set_layer_seed(&self, seed: u64)` | `crates/contextra-vector/src/hnsw/types/index.rs:210` | Sets the random seed used for HNSW layer selection during insertions. |
| `pub fn set_fault_injection_insert_target(&self, target: u64)` | `crates/contextra-vector/src/hnsw/types/index.rs:216` | Sets the target insertion count for simulating compute_insert fault injection in tests. |
| `pub fn check_and_trigger_partial_rebuild(&self)` | `crates/contextra-vector/src/hnsw/types/index.rs:228` |  |
| `pub fn load_mmap_sync(&self, path: impl AsRef<std::path::Path>) -> Result<()>` | `crates/contextra-vector/src/hnsw/types/persist.rs:16` |  |
| `pub fn load_mmap(&self, path: impl AsRef<std::path::Path>) -> Result<()>` | `crates/contextra-vector/src/hnsw/types/persist.rs:22` |  |
| `pub fn save_sync(&self, path: impl AsRef<std::path::Path>) -> Result<()>` | `crates/contextra-vector/src/hnsw/types/persist.rs:283` |  |
| `pub fn save(&self, path: impl AsRef<std::path::Path>) -> Result<()>` | `crates/contextra-vector/src/hnsw/types/persist.rs:288` |  |
| `pub fn compact_seq_log(&self, min_active_seqno: u64)` | `crates/contextra-vector/src/hnsw/types/seqlog.rs:11` |  |
| `pub fn pin_snapshot(&self, seq_no: u64)` | `crates/contextra-vector/src/hnsw/types/seqlog.rs:15` |  |
| `pub fn unpin_snapshot(&self, seq_no: u64)` | `crates/contextra-vector/src/hnsw/types/seqlog.rs:19` |  |

#### `type HnswIndex`
- Ort: `crates/contextra-vector/src/hnsw/types/index.rs:27`
```rust
pub type HnswIndex = GenericHnswIndex<ScalarKernel>
```
Dokumentation: Der Standard-HNSW-Vektorindex mit dem skalaren Referenz-Distanzkernel.

### Modul `partial_rebuild`

#### `struct PartialRebuildConfig`
- Ort: `crates/contextra-vector/src/partial_rebuild.rs:44`
```rust
pub struct PartialRebuildConfig
```
Dokumentation: Konfiguration für den Partial-Rebuild-Trigger.
- Impl-Block (Trait `Default`) `crates/contextra-vector/src/partial_rebuild.rs:53`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `critical_ratio` | `f32` | `crates/contextra-vector/src/partial_rebuild.rs:46` |
| `traversal_window` | `usize` | `crates/contextra-vector/src/partial_rebuild.rs:48` |
| `min_global_ratio` | `f32` | `crates/contextra-vector/src/partial_rebuild.rs:50` |

#### `struct TraversalTracker`
- Ort: `crates/contextra-vector/src/partial_rebuild.rs:69`
```rust
pub struct TraversalTracker
```
Dokumentation: Ringpuffer für besuchte HNSW-Knoten-IDs während ef_search-Traversierungen.
- Impl-Block `crates/contextra-vector/src/partial_rebuild.rs:74`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(config: PartialRebuildConfig) -> Self` | `crates/contextra-vector/src/partial_rebuild.rs:76` | Erstellt einen neuen TraversalTracker mit der gegebenen Konfiguration. |
| `pub fn record_traversal(&mut self, visited_node_ids: Vec<u64>)` | `crates/contextra-vector/src/partial_rebuild.rs:85` |  |
| `pub fn find_oversaturated_regions( &self, tombstone_map: &BTreeMap<u64, bool>, ) -> Vec<u64>` | `crates/contextra-vector/src/partial_rebuild.rs:98` |  |
| `pub fn hot_path_nodes(&self) -> impl Iterator<Item = u64> + '_` | `crates/contextra-vector/src/partial_rebuild.rs:138` | Gibt die N zuletzt besuchten eindeutigen Knoten-IDs zurück. |

#### `fn should_trigger_partial_rebuild`
- Ort: `crates/contextra-vector/src/partial_rebuild.rs:160`
```rust
pub fn should_trigger_partial_rebuild( tracker: &TraversalTracker, tombstone_map: &BTreeMap<u64, bool>, global_tombstone_ratio: f32, config: &PartialRebuildConfig, ) -> Option<Vec<u64>>
```
- Gate: `#[cfg(feature = "partial-index-rebuild")]`

### Modul `persistence`

#### `mod atomic`
- Ort: `crates/contextra-vector/src/persistence/mod.rs:7`
```rust
pub mod atomic
```

### Modul `persistence::atomic`

#### `fn write_atomic`
- Ort: `crates/contextra-vector/src/persistence/atomic.rs:56`
```rust
pub fn write_atomic(destination: &Path, data: &[u8]) -> Result<()>
```
Dokumentation: Writes data to `destination` atomically via a temporary file in the same parent directory, ensuring both file data and parent directory entry are durably synchronized to disk.

### Modul `persistence::header`

#### `struct HnswHeader`
- Ort: `crates/contextra-vector/src/persistence/header.rs:24`
```rust
pub struct HnswHeader
```
Dokumentation: The header of an HNSW persistent file.
- Impl-Block `crates/contextra-vector/src/persistence/header.rs:44`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new( dimension: u32, m: u32, metric: u8, quantized: u8, q_min: f32, q_max: f32, node_count: u64, entry_point: i64, nodes_offset: u64, connections_offset: u64, last_tx_id: u64, ) -> Self` | `crates/contextra-vector/src/persistence/header.rs:48` |  |
| `pub fn new_v2( dimension: u32, m: u32, metric: u8, quantized: u8, q_min: f32, q_max: f32, node_count: u64, entry_point: i64, nodes_offset: u64, connections_offset: u64, last_tx_id: u64, quant_calibration_offset: u64, quant_calibration_len: u32,` | `crates/contextra-vector/src/persistence/header.rs:79` |  |
| `pub fn new_v2_with_bias( dimension: u32, m: u32, metric: u8, quantized: u8, q_min: f32, q_max: f32, node_count: u64, entry_point: i64, nodes_offset: u64, connections_offset: u64, last_tx_id: u64, quant_calibration_offset: u64, quant_calibration_len: u32,` | `crates/contextra-vector/src/persistence/header.rs:114` |  |
| `pub fn magic(&self) -> u32` | `crates/contextra-vector/src/persistence/header.rs:152` |  |
| `pub fn version(&self) -> u16` | `crates/contextra-vector/src/persistence/header.rs:156` |  |
| `pub fn m(&self) -> u32` | `crates/contextra-vector/src/persistence/header.rs:160` |  |
| `pub fn metric(&self) -> u8` | `crates/contextra-vector/src/persistence/header.rs:164` |  |
| `pub fn quantized(&self) -> u8` | `crates/contextra-vector/src/persistence/header.rs:168` |  |
| `pub fn q_min(&self) -> f32` | `crates/contextra-vector/src/persistence/header.rs:172` |  |
| `pub fn q_max(&self) -> f32` | `crates/contextra-vector/src/persistence/header.rs:176` |  |
| `pub fn nodes_offset(&self) -> u64` | `crates/contextra-vector/src/persistence/header.rs:180` |  |
| `pub fn connections_offset(&self) -> u64` | `crates/contextra-vector/src/persistence/header.rs:184` |  |
| `pub fn set_connections_offset(&mut self, offset: u64)` | `crates/contextra-vector/src/persistence/header.rs:188` |  |
| `pub fn node_count(&self) -> u64` | `crates/contextra-vector/src/persistence/header.rs:192` |  |
| `pub fn last_tx_id(&self) -> u64` | `crates/contextra-vector/src/persistence/header.rs:196` |  |
| `pub fn quant_calibration_offset(&self) -> u64` | `crates/contextra-vector/src/persistence/header.rs:200` |  |
| `pub fn quant_calibration_len(&self) -> u32` | `crates/contextra-vector/src/persistence/header.rs:204` |  |
| `pub fn sq8_bias_mean(&self) -> f32` | `crates/contextra-vector/src/persistence/header.rs:208` |  |
| `pub fn sq8_bias_variance(&self) -> f32` | `crates/contextra-vector/src/persistence/header.rs:212` |  |
| `pub fn sq8_bias(&self) -> Sq8Bias` | `crates/contextra-vector/src/persistence/header.rs:216` |  |
| `pub fn dimension(&self) -> u32` | `crates/contextra-vector/src/persistence/header.rs:224` |  |
| `pub fn entry_point(&self) -> i64` | `crates/contextra-vector/src/persistence/header.rs:228` |  |
| `pub fn is_quantized(&self) -> bool` | `crates/contextra-vector/src/persistence/header.rs:232` |  |
| `pub fn q_range(&self) -> (f32, f32)` | `crates/contextra-vector/src/persistence/header.rs:236` |  |
| `pub fn try_from_bytes(bytes: &[u8]) -> Result<Self>` | `crates/contextra-vector/src/persistence/header.rs:241` |  |
| `pub fn checksum(&self) -> [u8` | `crates/contextra-vector/src/persistence/header.rs:424` |  |
| `pub fn verify_checksum(&self, expected: &[u8` | `crates/contextra-vector/src/persistence/header.rs:429` | Verifies that the serialized bytes match the expected BLAKE3 checksum. |
| `pub fn to_bytes(&self) -> [u8` | `crates/contextra-vector/src/persistence/header.rs:442` |  |

Assoziierte Konstanten/Typen:

- `pub const SIZE: usize = 84` — `crates/contextra-vector/src/persistence/header.rs:45`

#### `const HNSW_MAGIC`
- Ort: `crates/contextra-vector/src/persistence/header.rs:14`
```rust
pub const HNSW_MAGIC: u32 = 0x484E5357
```
Dokumentation: Magic number for HNSW files (0x484E5357 = "HNSW").

#### `const HNSW_VERSION`
- Ort: `crates/contextra-vector/src/persistence/header.rs:16`
```rust
pub const HNSW_VERSION: u16 = 2
```
Dokumentation: Current file format version.

### Modul `persistence::mmap`

#### `struct MmapIndex`
- Ort: `crates/contextra-vector/src/persistence/mmap.rs:13`
```rust
pub struct MmapIndex
```
Dokumentation: A reader for memory-mapped HNSW indices.
- Impl-Block `crates/contextra-vector/src/persistence/mmap.rs:19`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `mmap` | `std::sync::Arc<memmap2::Mmap>` | `crates/contextra-vector/src/persistence/mmap.rs:14` |
| `header` | `HnswHeader` | `crates/contextra-vector/src/persistence/mmap.rs:15` |
| `path` | `Option<std::path::PathBuf>` | `crates/contextra-vector/src/persistence/mmap.rs:16` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn open(path: impl AsRef<std::path::Path>) -> Result<Self>` | `crates/contextra-vector/src/persistence/mmap.rs:21` |  |
| `pub async fn open_async(path: impl AsRef<std::path::Path> + Send) -> Result<Self>` | `crates/contextra-vector/src/persistence/mmap.rs:134` | Opens an HNSW file without requiring external runtime blocking task spawns. |
| `pub fn get_node_record(&self, index: usize) -> Result<NodeRecord>` | `crates/contextra-vector/src/persistence/mmap.rs:138` |  |
| `pub fn get_vector(&self, record: &NodeRecord) -> Result<&[u8]>` | `crates/contextra-vector/src/persistence/mmap.rs:157` |  |
| `pub fn get_connections(&self, record: &NodeRecord, layer: usize) -> Result<Vec<u32>>` | `crates/contextra-vector/src/persistence/mmap.rs:178` |  |

### Modul `persistence::node`

#### `struct NodeRecord`
- Ort: `crates/contextra-vector/src/persistence/node.rs:11`
```rust
pub struct NodeRecord
```
Dokumentation: Represents a node's metadata in the flat file.
- Impl-Block `crates/contextra-vector/src/persistence/node.rs:21`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `doc_id` | `u64` | `crates/contextra-vector/src/persistence/node.rs:13` |
| `doc_id` | `u128` | `crates/contextra-vector/src/persistence/node.rs:15` |
| `max_layer` | `u8` | `crates/contextra-vector/src/persistence/node.rs:16` |
| `vector_offset` | `u64` | `crates/contextra-vector/src/persistence/node.rs:17` |
| `connections_offset` | `u64` | `crates/contextra-vector/src/persistence/node.rs:18` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn vector_offset(&self) -> u64` | `crates/contextra-vector/src/persistence/node.rs:28` |  |
| `pub fn connections_offset(&self) -> u64` | `crates/contextra-vector/src/persistence/node.rs:33` |  |
| `pub fn from_bytes(bytes: &[u8]) -> Result<Self>` | `crates/contextra-vector/src/persistence/node.rs:37` |  |
| `pub fn to_bytes(&self) -> Vec<u8>` | `crates/contextra-vector/src/persistence/node.rs:84` |  |

Assoziierte Konstanten/Typen:

- `pub const SIZE: usize = 8 + 1 + 8 + 8` — `crates/contextra-vector/src/persistence/node.rs:23`
- `pub const SIZE: usize = 16 + 1 + 8 + 8` — `crates/contextra-vector/src/persistence/node.rs:25`

### Modul `persistence::tests`

#### `mod atomic_tests`
- Ort: `crates/contextra-vector/src/persistence/tests/mod.rs:6`
```rust
pub mod atomic_tests
```

#### `mod header_tests`
- Ort: `crates/contextra-vector/src/persistence/tests/mod.rs:7`
```rust
pub mod header_tests
```

#### `mod mmap_tests`
- Ort: `crates/contextra-vector/src/persistence/tests/mod.rs:9`
```rust
pub mod mmap_tests
```

#### `mod node_tests`
- Ort: `crates/contextra-vector/src/persistence/tests/mod.rs:10`
```rust
pub mod node_tests
```

### Modul `quantize`

#### `mod decode`
- Ort: `crates/contextra-vector/src/quantize/mod.rs:18`
```rust
pub mod decode
```

#### `mod distance`
- Ort: `crates/contextra-vector/src/quantize/mod.rs:19`
```rust
pub mod distance
```

#### `mod encode`
- Ort: `crates/contextra-vector/src/quantize/mod.rs:20`
```rust
pub mod encode
```

#### `mod train`
- Ort: `crates/contextra-vector/src/quantize/mod.rs:21`
```rust
pub mod train
```

#### `struct ScalarQuantizer`
- Ort: `crates/contextra-vector/src/quantize/mod.rs:49`
```rust
pub struct ScalarQuantizer
```
Dokumentation: An 8-bit Scalar Quantizer (SQ8) with per-dimension scaling. Quantization reduces the memory footprint of vector storage by 4x. Per-dimension scaling improves recall by adapting to different value ranges.
- Impl-Block `crates/contextra-vector/src/quantize/decode.rs:14`
- Impl-Block `crates/contextra-vector/src/quantize/distance.rs:16`
- Impl-Block `crates/contextra-vector/src/quantize/encode.rs:15`
- Impl-Block (Trait `Clone`) `crates/contextra-vector/src/quantize/mod.rs:61`
- Impl-Block `crates/contextra-vector/src/quantize/mod.rs:81`
- Impl-Block `crates/contextra-vector/src/quantize/train.rs:15`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `mins` | `Vec<f32>` | `crates/contextra-vector/src/quantize/mod.rs:50` |
| `maxes` | `Vec<f32>` | `crates/contextra-vector/src/quantize/mod.rs:51` |
| `scales` | `Vec<f32>` | `crates/contextra-vector/src/quantize/mod.rs:52` |
| `inv_scales` | `Vec<f32>` | `crates/contextra-vector/src/quantize/mod.rs:53` |
| `dimension` | `usize` | `crates/contextra-vector/src/quantize/mod.rs:54` |
| `total_queries` | `AtomicU64` | `crates/contextra-vector/src/quantize/mod.rs:56` |
| `out_of_range_queries` | `AtomicU64` | `crates/contextra-vector/src/quantize/mod.rs:58` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn dequantize(&self, vector: &[u8]) -> Result<Vec<f32>>` | `crates/contextra-vector/src/quantize/decode.rs:17` |  |
| `pub fn asymmetric_dist( &self, query: &[f32], quantized: &[u8], metric: DistanceMetric, ) -> Result<f32>` | `crates/contextra-vector/src/quantize/distance.rs:19` |  |
| `pub fn symmetric_dist(&self, q1: &[u8], q2: &[u8], metric: DistanceMetric) -> Result<f32>` | `crates/contextra-vector/src/quantize/distance.rs:70` |  |
| `pub fn quantize(&self, vector: &[f32]) -> Result<Vec<u8>>` | `crates/contextra-vector/src/quantize/encode.rs:18` |  |
| `pub fn mins(&self) -> &[f32]` | `crates/contextra-vector/src/quantize/mod.rs:83` | Dimensionale Minima. |
| `pub fn maxes(&self) -> &[f32]` | `crates/contextra-vector/src/quantize/mod.rs:88` | Dimensionale Maxima. |
| `pub fn scales(&self) -> &[f32]` | `crates/contextra-vector/src/quantize/mod.rs:93` | Dimensionale Skalierungsfaktoren. |
| `pub fn inv_scales(&self) -> &[f32]` | `crates/contextra-vector/src/quantize/mod.rs:98` | Dimensionale inverse Skalierungsfaktoren. |
| `pub fn max_reconstruction_error_per_dim(&self, dim: usize) -> Option<f32>` | `crates/contextra-vector/src/quantize/mod.rs:103` | Maximaler Rekonstruktionsfehler für eine gegebene Dimension. |
| `pub fn dimension(&self) -> usize` | `crates/contextra-vector/src/quantize/mod.rs:113` | Vektordimension des Quantizers. |
| `pub fn try_train(batch: &[&[f32]], dimension: usize) -> Result<Self>` | `crates/contextra-vector/src/quantize/train.rs:17` | Versucht das Training eines `ScalarQuantizer` mit Standard-Perzentilen. |
| `pub fn try_train_with_percentiles( batch: &[&[f32]], dimension: usize, p_low: f32, p_high: f32, ) -> Result<Self>` | `crates/contextra-vector/src/quantize/train.rs:28` |  |
| `pub fn train_with_percentiles( batch: &[&[f32]], dimension: usize, p_low: f32, p_high: f32, ) -> Self` | `crates/contextra-vector/src/quantize/train.rs:129` | Panik-freie Trainingsmethode für Tests und Legacy-Pfade. |
| `pub fn train(batch: &[&[f32]], dimension: usize) -> Self` | `crates/contextra-vector/src/quantize/train.rs:147` | Trainiert den Quantizer mit Standard-Parametern. |
| `pub fn calculate_vector_drift(&self, vector: &[f32]) -> f32` | `crates/contextra-vector/src/quantize/train.rs:158` |  |
| `pub fn check_drift(&self, vector: &[f32]) -> f32` | `crates/contextra-vector/src/quantize/train.rs:173` | Prüft den Drift und aktualisiert interne Metriken. |
| `pub fn drift_ratio(&self) -> f32` | `crates/contextra-vector/src/quantize/train.rs:178` | Gibt das Verhältnis der Anfragen außerhalb des trainierten Bereichs zurück. |
| `pub fn is_rebuild_required(&self, threshold: f32) -> bool` | `crates/contextra-vector/src/quantize/train.rs:188` | Prüft, ob eine Neukalibrierung empfohlen wird. |

#### `const DEFAULT_P_HIGH`
- Ort: `crates/contextra-vector/src/quantize/mod.rs:35`
```rust
pub const DEFAULT_P_HIGH: f32 = 0.995
```
Dokumentation: Default high percentile for quantization scaling (99.5%).

#### `const DEFAULT_P_LOW`
- Ort: `crates/contextra-vector/src/quantize/mod.rs:33`
```rust
pub const DEFAULT_P_LOW: f32 = 0.005
```
Dokumentation: Default low percentile for quantization scaling (0.5%).

#### `const SQ8_MAX_ERROR_FACTOR_PER_DIM`
- Ort: `crates/contextra-vector/src/quantize/mod.rs:38`
```rust
pub const SQ8_MAX_ERROR_FACTOR_PER_DIM: f32 = 1.0 / 510.0
```
Dokumentation: Theoretical maximum quantization error bound factor per dimension: $1 / (2 \cdot 255) = 1 / 510$.

### Modul `quantize_rabitq`

#### `struct RaBitQQuantizer`
- Ort: `crates/contextra-vector/src/quantize_rabitq.rs:29`
```rust
pub struct RaBitQQuantizer
```
Dokumentation: RaBitQ Quantizer with random orthogonal rotation and scalar metadata.
- Impl-Block `crates/contextra-vector/src/quantize_rabitq.rs:35`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn try_train(vectors: &[Vec<f32>], dimension: usize) -> Result<Self>` | `crates/contextra-vector/src/quantize_rabitq.rs:37` | Safely trains a `RaBitQQuantizer` for vectors of the specified `dimension` with a default seed. |
| `pub fn try_train_with_seed(vectors: &[Vec<f32>], dimension: usize, seed: u64) -> Result<Self>` | `crates/contextra-vector/src/quantize_rabitq.rs:49` | Safely trains a `RaBitQQuantizer` for vectors of the specified `dimension` with an explicit injected `seed`. Generates a deterministic orthogonal matrix $R \in  |
| `pub fn dimension(&self) -> usize` | `crates/contextra-vector/src/quantize_rabitq.rs:81` | Returns the vector dimension supported by this quantizer. |
| `pub fn code_length(&self) -> usize` | `crates/contextra-vector/src/quantize_rabitq.rs:86` | Returns the expected byte length of quantized codes (`dimension.div_ceil(8) + 8` bytes). |
| `pub fn quantize(&self, vector: &[f32]) -> Result<Vec<u8>>` | `crates/contextra-vector/src/quantize_rabitq.rs:96` | Quantizes a single $f32$ vector into a RaBitQ packed bitcode buffer. The returned code consists of: - $\lceil D / 8 \rceil$ bytes of packed sign bits. - 4 bytes |
| `pub fn asymmetric_distance(&self, query: &[f32], code: &[u8]) -> Result<f32>` | `crates/contextra-vector/src/quantize_rabitq.rs:163` | Computes the asymmetric Euclidean distance between an unquantized query vector and a quantized bitcode. # Mathematical Derivation & Bias Correction Following Ra |

### Modul `types`

#### `struct ScoredDocument`
- Ort: `crates/contextra-vector/src/types.rs:40`
```rust
pub struct ScoredDocument
```
Dokumentation: Suchergebnis mit Dokument-ID und Relevanz-Score / Distanz.
- Impl-Block (Trait `From<ScoredDocument>`) `crates/contextra-vector/src/compat/types.rs:26`
- Impl-Block (Trait `From<contextra_types::ScoredDocument>`) `crates/contextra-vector/src/compat/types.rs:35`
- Impl-Block `crates/contextra-vector/src/types.rs:49`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `doc_id` | `DocId` | `crates/contextra-vector/src/types.rs:42` |
| `score` | `f32` | `crates/contextra-vector/src/types.rs:44` |
| `distance` | `f32` | `crates/contextra-vector/src/types.rs:46` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn from_score(doc_id: DocId, score: f32) -> Self` | `crates/contextra-vector/src/types.rs:51` | Erzeugt ein ScoredDocument mit abgeleiteter Distanz (`1.0 - score`). |
| `pub fn new(doc_id: DocId, score: f32, distance: f32) -> Self` | `crates/contextra-vector/src/types.rs:60` | Erzeugt ein neues ScoredDocument mit explizitem Score und Distanzwert. |

#### `enum DistanceMetric`
- Ort: `crates/contextra-vector/src/types.rs:27`
```rust
pub enum DistanceMetric
```
- Impl-Block (Trait `From<DistanceMetric>`) `crates/contextra-vector/src/compat/types.rs:5`
- Impl-Block (Trait `From<contextra_types::DistanceMetric>`) `crates/contextra-vector/src/compat/types.rs:15`

Varianten:

- `Cosine` — `crates/contextra-vector/src/types.rs:30`
- `Euclidean` — `crates/contextra-vector/src/types.rs:32`
- `DotProduct` — `crates/contextra-vector/src/types.rs:34`

#### `trait Maintain`
- Ort: `crates/contextra-vector/src/types.rs:86`
```rust
pub trait Maintain: Send + Sync
```
Dokumentation: Neutraler Wartungs-Trait für Speicherfreigaben und Kompaktierung (SPEC §6.1).

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn compact(&self) -> crate::error::Result<()>` | `crates/contextra-vector/src/types.rs:88` | Führt physische Bereinigung und Slot-Freigaben durch. |
| `fn is_rebuild_required(&self) -> bool` | `crates/contextra-vector/src/types.rs:90` | Prüft den Rebuild- bzw. Reorganisationsbedarf. |

#### `const MAX_SEARCH_K`
- Ort: `crates/contextra-vector/src/types.rs:70`
```rust
pub const MAX_SEARCH_K: usize = 10_000
```
Dokumentation: Maximal zulässiges `k` für Nachbarschaftssuchen.

## 9. Impl-Blöcke ohne Typ im selben Crate

- `impl From<VectorError> for contextra_types::ContextraError` — `crates/contextra-vector/src/compat/error.rs:7` (0 pub Methoden)
- `impl VectorIndex for HnswIndex` — `crates/contextra-vector/src/compat/hnsw_vector_index.rs:15` (0 pub Methoden)
- `impl Index<ExternalId, [f32]> for HnswIndex` — `crates/contextra-vector/src/compat/hnsw_vector_index.rs:603` (0 pub Methoden)
- `impl Maintain for HnswIndex` — `crates/contextra-vector/src/compat/hnsw_vector_index.rs:808` (0 pub Methoden)
- `impl HnswIndex` — `crates/contextra-vector/src/compat/scanner.rs:262` (1 pub Methoden)
- `impl FilteredIndex for HnswIndex` — `crates/contextra-vector/src/hnsw/acorn_filtered.rs:39` (0 pub Methoden)
- `impl HnswIndex` — `crates/contextra-vector/src/hnsw/core_rebuild/full.rs:302` (1 pub Methoden)
- `impl crate::types::Maintain for HnswIndex` — `crates/contextra-vector/src/hnsw/core_rebuild/full.rs:387` (0 pub Methoden)
- `impl HnswIndex` — `crates/contextra-vector/src/hnsw/core_rebuild/full.rs:405` (3 pub Methoden)
- `impl HnswIndex` — `crates/contextra-vector/src/hnsw/core_rebuild/full.rs:427` (1 pub Methoden)
- `impl HnswIndex` — `crates/contextra-vector/src/hnsw/core_search/adaptive.rs:21` (3 pub Methoden)
- `impl IntoDocId for DocId` — `crates/contextra-vector/src/hnsw/deletion.rs:26` (0 pub Methoden)
- `impl IntoDocId for u64` — `crates/contextra-vector/src/hnsw/deletion.rs:32` (0 pub Methoden)
- `impl IntoDocId for &u64` — `crates/contextra-vector/src/hnsw/deletion.rs:38` (0 pub Methoden)
- `impl GhostFreeVectorIndex for HnswIndex` — `crates/contextra-vector/src/hnsw/deletion.rs:73` (0 pub Methoden)
- `impl HnswIndex` — `crates/contextra-vector/src/hnsw/deletion.rs:336` (2 pub Methoden)
- `impl HnswIndex` — `crates/contextra-vector/src/hnsw/types/builder.rs:23` (5 pub Methoden)
