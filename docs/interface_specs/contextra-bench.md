# Schnittstellenspezifikation: `contextra-bench`

> Erzeugt durch `tools/extract_interfaces.py` aus dem Quellcode. Keine Handpflege. Jede Angabe trägt Datei:Zeile. Maßgeblich bleibt der Quellcode.

## 1. Kopf

| Feld | Wert |
|---|---|
| Name | `contextra-bench` |
| Version | workspace |
| Pfad | `benchmarks/contextra-bench` |
| Rolle | Benchmarks |
| Bibliothekspfad | src/lib.rs |

## 2. Abhängigkeiten

| Name | Art | Quelle | optional | verwendet von (normal/dev) |
|---|---|---|---|---|
| `anyhow` | normal | registry | nein |  |
| `contextra-db` | normal | workspace | nein | contextra, contextra-agent, contextra-bench, contextra-cognition, contextra-db-fuzz, contextra-engine |
| `contextra-rank` | normal | workspace | nein | contextra-bench, contextra-cognition, contextra-db, contextra-engine, contextra-infer-candle, contextra-mcp |
| `contextra-store` | normal | workspace | nein | contextra-agent, contextra-bench, contextra-db, contextra-engine, contextra-store-fuzz |
| `contextra-sys` | dev | workspace | nein | contextra-bench, contextra-store, contextra-vault, contextra-vector |
| `contextra-text` | normal | workspace | nein | contextra-agent, contextra-bench, contextra-cognition, contextra-db, contextra-engine, contextra-text-fuzz |
| `contextra-types` | normal | workspace | nein | contextra, contextra-adapt, contextra-agent, contextra-bench, contextra-checkpoint, contextra-cognition, contextra-crypto, contextra-crypto-fuzz, contextra-db, contextra-engine, contextra-graph, contextra-graph-fuzz, contextra-infer-candle, contextra-infer-ollama, contextra-infer-onnx, contextra-ingest, contextra-kvcache, contextra-mcp, contextra-mvcc, contextra-ports, contextra-py, contextra-rank, contextra-router, contextra-simd, contextra-store, contextra-store-fuzz, contextra-testkit, contextra-text, contextra-vault, contextra-vector, contextra-vector-fuzz |
| `contextra-vector` | normal | workspace | nein | contextra-agent, contextra-bench, contextra-db, contextra-engine, contextra-vector-fuzz |
| `criterion` | dev | workspace | nein |  |
| `hdrhistogram` | normal | registry | nein |  |
| `libc` | dev | registry | nein |  |
| `rand` | normal | workspace | nein |  |
| `redb` | dev | registry | nein |  |
| `serde` | normal | workspace | nein |  |
| `serde_json` | normal | workspace | nein |  |
| `sled` | dev | registry | nein |  |
| `tempfile` | normal | workspace | nein |  |
| `tokio` | normal | workspace | nein |  |
| `tracing` | normal | workspace | nein |  |
| `tracing-subscriber` | normal | workspace | nein |  |

**Verwendet von (Crate-Ebene):** –

## 3. Features

| Feature | Aktiviert |
|---|---|
| `datasets` | – |
| `default` | – |
| `external-benchmarks` | – |

## 4. Metriken (Quellcode)

| Kennzahl | Wert |
|---|---|
| Quelldateien | 12 |
| Quellzeilen | 4309 |
| Öffentliche Items (Modul-Ebene) | 68 |
| davon `fn` | 27 |
| davon `struct` | 29 |
| davon `enum` | 1 |
| davon `trait` | 2 |
| davon `const` | 1 |
| davon `mod` | 8 |
| Öffentliche Makros (`#[macro_export]`) | 0 |
| Öffentliche Re-Exports (`pub use`) | 0 |
| `unsafe`-Vorkommen (Produktionscode) | 0 |
| `.unwrap()` (Produktionscode) | 0 |
| `.expect(` (Produktionscode) | 0 |
| `panic!` (Produktionscode) | 0 |
| `todo!/unimplemented!/unreachable!` | 0 |
| Indexierungen `x[…]` (grob, Produktionscode) | 48 |
| `#[deprecated]` | 0 |
| `#[cfg(feature…)]`-Gates | 7 |
| Parser-Abdeckung (geparste `pub`-Deklarationen / Grep-Treffer) | 71 / 71 = 100 % |

## 5. Modulbaum und Dateien

| Modul | Datei | Zeilen |
|---|---|---|
| `adapters` | `benchmarks/contextra-bench/src/adapters.rs` | 168 |
| `ann_benchmarks` | `benchmarks/contextra-bench/src/ann_benchmarks.rs` | 284 |
| `beir_eval` | `benchmarks/contextra-bench/src/beir_eval.rs` | 378 |
| `bin:compare_baseline` | `benchmarks/contextra-bench/src/bin/compare_baseline.rs` | 92 |
| `bin:scale_100k` | `benchmarks/contextra-bench/src/bin/scale_100k.rs` | 296 |
| `compare` | `benchmarks/contextra-bench/src/compare.rs` | 437 |
| `crate` | `benchmarks/contextra-bench/src/lib.rs` | 13 |
| `crate` | `benchmarks/contextra-bench/src/main.rs` | 1505 |
| `matrix` | `benchmarks/contextra-bench/src/matrix.rs` | 204 |
| `metrics` | `benchmarks/contextra-bench/src/metrics.rs` | 347 |
| `path_rag_sweep` | `benchmarks/contextra-bench/src/path_rag_sweep.rs` | 361 |
| `regression_gate` | `benchmarks/contextra-bench/src/regression_gate.rs` | 224 |

## 8. Öffentliche Schnittstelle je Item

### Modul `adapters`

#### `struct BenchmarkDocument`
- Ort: `benchmarks/contextra-bench/src/adapters.rs:14`
```rust
pub struct BenchmarkDocument
```
Dokumentation: Abstraktes Dokument für den Engine-Vergleich

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `id` | `String` | `benchmarks/contextra-bench/src/adapters.rs:15` |
| `title` | `String` | `benchmarks/contextra-bench/src/adapters.rs:16` |
| `body` | `String` | `benchmarks/contextra-bench/src/adapters.rs:17` |
| `vector` | `Option<Vec<f32>>` | `benchmarks/contextra-bench/src/adapters.rs:18` |

#### `struct EngineHit`
- Ort: `benchmarks/contextra-bench/src/adapters.rs:23`
```rust
pub struct EngineHit
```
Dokumentation: Einheitliche Treffer-Struktur externer Engines

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `doc_id` | `String` | `benchmarks/contextra-bench/src/adapters.rs:24` |
| `score` | `f32` | `benchmarks/contextra-bench/src/adapters.rs:25` |

#### `struct LanceDbAdapterSketch`
- Ort: `benchmarks/contextra-bench/src/adapters.rs:112`
```rust
pub struct LanceDbAdapterSketch
```
Dokumentation: Skizze eines LanceDB-Adapters (Vorbereitung für spätere Feature-Gated Implementierung)
- Impl-Block `benchmarks/contextra-bench/src/adapters.rs:118`
- Impl-Block (Trait `VectorEngineAdapter`) `benchmarks/contextra-bench/src/adapters.rs:128`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `uri` | `String` | `benchmarks/contextra-bench/src/adapters.rs:113` |
| `table_name` | `String` | `benchmarks/contextra-bench/src/adapters.rs:114` |
| `dimension` | `usize` | `benchmarks/contextra-bench/src/adapters.rs:115` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(uri: impl Into<String>, table_name: impl Into<String>, dimension: usize) -> Self` | `benchmarks/contextra-bench/src/adapters.rs:119` |  |

#### `struct QueryLatencyStats`
- Ort: `benchmarks/contextra-bench/src/adapters.rs:30`
```rust
pub struct QueryLatencyStats
```
Dokumentation: Ausführungs-Statistik eines Query-Laufs

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `elapsed` | `Duration` | `benchmarks/contextra-bench/src/adapters.rs:31` |
| `hits_returned` | `usize` | `benchmarks/contextra-bench/src/adapters.rs:32` |

#### `struct TantivyAdapterSketch`
- Ort: `benchmarks/contextra-bench/src/adapters.rs:73`
```rust
pub struct TantivyAdapterSketch
```
Dokumentation: Skizze eines Tantivy-Adapters (Vorbereitung für spätere Feature-Gated Implementierung)
- Impl-Block `benchmarks/contextra-bench/src/adapters.rs:78`
- Impl-Block (Trait `FullTextEngineAdapter`) `benchmarks/contextra-bench/src/adapters.rs:87`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `index_path` | `String` | `benchmarks/contextra-bench/src/adapters.rs:74` |
| `is_initialized` | `bool` | `benchmarks/contextra-bench/src/adapters.rs:75` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(index_path: impl Into<String>) -> Self` | `benchmarks/contextra-bench/src/adapters.rs:79` |  |

#### `trait FullTextEngineAdapter`
- Ort: `benchmarks/contextra-bench/src/adapters.rs:36`
```rust
pub trait FullTextEngineAdapter: Send + Sync
```
Dokumentation: Generischer Trait für externe oder interne Volltext-Such-Engines (z. B. Tantivy)

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn name(&self) -> &'static str` | `benchmarks/contextra-bench/src/adapters.rs:41` | Name der Engine (z. B. "tantivy", "contextra-text") |
| `fn index_batch(&mut self, docs: &[BenchmarkDocument]) -> Result<Duration, Self::Error>` | `benchmarks/contextra-bench/src/adapters.rs:44` | Indiziert einen Batch von Textdokumenten |
| `fn search(&self, query: &str, top_k: usize) -> Result<Vec<EngineHit>, Self::Error>` | `benchmarks/contextra-bench/src/adapters.rs:47` | Führt eine Volltext-/BM25-Suche aus |
| `fn commit(&mut self) -> Result<(), Self::Error>` | `benchmarks/contextra-bench/src/adapters.rs:50` | Erzwingt einen Flush / Commit auf die Platte |

Assoziierte Konstanten/Typen:

- `type Error: std::fmt::Display` — `benchmarks/contextra-bench/src/adapters.rs:38`

#### `trait VectorEngineAdapter`
- Ort: `benchmarks/contextra-bench/src/adapters.rs:54`
```rust
pub trait VectorEngineAdapter: Send + Sync
```
Dokumentation: Generischer Trait für externe oder interne Vektor-Engines (z. B. LanceDB)

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn name(&self) -> &'static str` | `benchmarks/contextra-bench/src/adapters.rs:59` | Name der Engine (z. B. "lancedb", "contextra-vector") |
| `fn dimension(&self) -> usize` | `benchmarks/contextra-bench/src/adapters.rs:62` | Dimension der Vektoren |
| `fn insert_vectors(&mut self, docs: &[BenchmarkDocument]) -> Result<Duration, Self::Error>` | `benchmarks/contextra-bench/src/adapters.rs:65` | Fügt Vektoren mit Dokument-IDs ein |
| `fn search_knn(&self, query_vector: &[f32], top_k: usize) -> Result<Vec<EngineHit>, Self::Error>` | `benchmarks/contextra-bench/src/adapters.rs:68` | Führt k-NN Ähnlichkeitssuche aus |

Assoziierte Konstanten/Typen:

- `type Error: std::fmt::Display` — `benchmarks/contextra-bench/src/adapters.rs:56`

### Modul `ann_benchmarks`

#### `struct AnnBenchResult`
- Ort: `benchmarks/contextra-bench/src/ann_benchmarks.rs:19`
```rust
pub struct AnnBenchResult
```

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `dataset_name` | `String` | `benchmarks/contextra-bench/src/ann_benchmarks.rs:20` |
| `n_vectors` | `usize` | `benchmarks/contextra-bench/src/ann_benchmarks.rs:21` |
| `dimension` | `usize` | `benchmarks/contextra-bench/src/ann_benchmarks.rs:22` |
| `k` | `usize` | `benchmarks/contextra-bench/src/ann_benchmarks.rs:23` |
| `qps_at_recall_90` | `f64` | `benchmarks/contextra-bench/src/ann_benchmarks.rs:25` |
| `qps_at_recall_95` | `f64` | `benchmarks/contextra-bench/src/ann_benchmarks.rs:26` |
| `qps_at_recall_99` | `f64` | `benchmarks/contextra-bench/src/ann_benchmarks.rs:27` |
| `max_recall_at_10` | `f64` | `benchmarks/contextra-bench/src/ann_benchmarks.rs:29` |
| `p50_latency_ms` | `f64` | `benchmarks/contextra-bench/src/ann_benchmarks.rs:30` |
| `p99_latency_ms` | `f64` | `benchmarks/contextra-bench/src/ann_benchmarks.rs:31` |

#### `fn compute_ground_truth`
- Ort: `benchmarks/contextra-bench/src/ann_benchmarks.rs:80`
```rust
pub fn compute_ground_truth( base: &[Vec<f32>], queries: &[Vec<f32>], k: usize, metric: DistanceMetric, ) -> Vec<Vec<usize>>
```
Dokumentation: Misst Ground-Truth Nearest Neighbors via Brute-Force

#### `fn generate_synthetic_vectors`
- Ort: `benchmarks/contextra-bench/src/ann_benchmarks.rs:35`
```rust
pub fn generate_synthetic_vectors(n: usize, dim: usize, seed: u64) -> Vec<Vec<f32>>
```
Dokumentation: Synthetische Vektoren (SIFT-ähnlich, uniform-random in [0..127]^dim für CI ohne HDF5)

#### `fn recall_at_k`
- Ort: `benchmarks/contextra-bench/src/ann_benchmarks.rs:112`
```rust
pub fn recall_at_k(retrieved: &[u64], ground_truth: &[usize], k: usize) -> f64
```
Dokumentation: Berechnet Recall@k: |retrieved ∩ ground_truth| / k

#### `fn run_ann_benchmark`
- Ort: `benchmarks/contextra-bench/src/ann_benchmarks.rs:136`
```rust
pub async fn run_ann_benchmark( base_vectors: Vec<Vec<f32>>, query_vectors: Vec<Vec<f32>>, ground_truth: Vec<Vec<usize>>, dataset_name: &str, m: usize, ef_construction: usize, ef_search_values: &[usize], ) -> Result<AnnBenchResult>
```
Dokumentation: Führt ANN-Benchmark durch: variiert ef_search, misst QPS vs Recall Trade-off

#### `fn run_ann_benchmark_synthetic_ci`
- Ort: `benchmarks/contextra-bench/src/ann_benchmarks.rs:249`
```rust
pub async fn run_ann_benchmark_synthetic_ci() -> Result<AnnBenchResult>
```
Dokumentation: Schnelle CI-Version: 1k Vektoren, D=128 (läuft in <10s)

### Modul `beir_eval`

#### `struct BeirDocument`
- Ort: `benchmarks/contextra-bench/src/beir_eval.rs:17`
```rust
pub struct BeirDocument
```

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `id` | `String` | `benchmarks/contextra-bench/src/beir_eval.rs:18` |
| `title` | `String` | `benchmarks/contextra-bench/src/beir_eval.rs:19` |
| `text` | `String` | `benchmarks/contextra-bench/src/beir_eval.rs:20` |

#### `struct BeirMetrics`
- Ort: `benchmarks/contextra-bench/src/beir_eval.rs:31`
```rust
pub struct BeirMetrics
```

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `dataset_name` | `String` | `benchmarks/contextra-bench/src/beir_eval.rs:32` |
| `num_docs` | `usize` | `benchmarks/contextra-bench/src/beir_eval.rs:33` |
| `num_queries` | `usize` | `benchmarks/contextra-bench/src/beir_eval.rs:34` |
| `ndcg_at_10` | `f64` | `benchmarks/contextra-bench/src/beir_eval.rs:35` |
| `recall_at_10` | `f64` | `benchmarks/contextra-bench/src/beir_eval.rs:36` |
| `map_at_100` | `f64` | `benchmarks/contextra-bench/src/beir_eval.rs:37` |
| `latency_p50_ms` | `f64` | `benchmarks/contextra-bench/src/beir_eval.rs:38` |
| `latency_p99_ms` | `f64` | `benchmarks/contextra-bench/src/beir_eval.rs:39` |

#### `struct BeirQuery`
- Ort: `benchmarks/contextra-bench/src/beir_eval.rs:24`
```rust
pub struct BeirQuery
```

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `id` | `String` | `benchmarks/contextra-bench/src/beir_eval.rs:25` |
| `query` | `String` | `benchmarks/contextra-bench/src/beir_eval.rs:26` |
| `relevant_docs` | `Vec<String>` | `benchmarks/contextra-bench/src/beir_eval.rs:27` |

#### `fn load_beir_corpus`
- Ort: `benchmarks/contextra-bench/src/beir_eval.rs:43`
```rust
pub fn load_beir_corpus(path: &Path) -> Result<Vec<BeirDocument>>
```
Dokumentation: Lädt BEIR-Corpus aus JSONL-Datei

#### `fn load_beir_queries`
- Ort: `benchmarks/contextra-bench/src/beir_eval.rs:81`
```rust
pub fn load_beir_queries(queries_path: &Path, qrels_path: &Path) -> Result<Vec<BeirQuery>>
```
Dokumentation: Lädt BEIR-Queries + QRels aus JSONL / TSV-Dateien

#### `fn map_at_k`
- Ort: `benchmarks/contextra-bench/src/beir_eval.rs:214`
```rust
pub fn map_at_k(ranked: &[String], relevant: &HashSet<String>, k: usize) -> f64
```
Dokumentation: Berechnet MAP@k

#### `fn ndcg_at_k`
- Ort: `benchmarks/contextra-bench/src/beir_eval.rs:185`
```rust
pub fn ndcg_at_k(ranked: &[String], relevant: &HashSet<String>, k: usize) -> f64
```
Dokumentation: Berechnet NDCG@k aus Ranked Result List und Relevance Set (strikt log2-Standard)

#### `fn recall_at_k`
- Ort: `benchmarks/contextra-bench/src/beir_eval.rs:239`
```rust
pub fn recall_at_k(ranked: &[String], relevant: &HashSet<String>, k: usize) -> f64
```
Dokumentation: Berechnet Recall@k

#### `fn run_beir_eval`
- Ort: `benchmarks/contextra-bench/src/beir_eval.rs:261`
```rust
pub async fn run_beir_eval( db_path: &Path, corpus: &[BeirDocument], queries: &[BeirQuery], dataset_name: &str, use_hybrid: bool, ) -> Result<BeirMetrics>
```
Dokumentation: Führt vollständige BEIR-Evaluation auf einer Contextra-Collection durch

### Modul `bin:scale_100k`

#### `struct IntervalResult`
- Ort: `benchmarks/contextra-bench/src/bin/scale_100k.rs:13`
```rust
pub struct IntervalResult
```

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `docs_inserted` | `u64` | `benchmarks/contextra-bench/src/bin/scale_100k.rs:14` |
| `insert_rate_docs_per_sec` | `f64` | `benchmarks/contextra-bench/src/bin/scale_100k.rs:15` |
| `search_p50_ns` | `u64` | `benchmarks/contextra-bench/src/bin/scale_100k.rs:16` |
| `search_p95_ns` | `u64` | `benchmarks/contextra-bench/src/bin/scale_100k.rs:17` |
| `search_p99_ns` | `u64` | `benchmarks/contextra-bench/src/bin/scale_100k.rs:18` |
| `rss_bytes` | `u64` | `benchmarks/contextra-bench/src/bin/scale_100k.rs:19` |

#### `struct ScaleBenchmarkOutput`
- Ort: `benchmarks/contextra-bench/src/bin/scale_100k.rs:23`
```rust
pub struct ScaleBenchmarkOutput
```

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `disclaimer` | `String` | `benchmarks/contextra-bench/src/bin/scale_100k.rs:24` |
| `total_docs_requested` | `u64` | `benchmarks/contextra-bench/src/bin/scale_100k.rs:25` |
| `step_size` | `u64` | `benchmarks/contextra-bench/src/bin/scale_100k.rs:26` |
| `dimension` | `usize` | `benchmarks/contextra-bench/src/bin/scale_100k.rs:27` |
| `seed` | `u64` | `benchmarks/contextra-bench/src/bin/scale_100k.rs:28` |
| `aborted_at_docs` | `Option<u64>` | `benchmarks/contextra-bench/src/bin/scale_100k.rs:29` |
| `error` | `Option<String>` | `benchmarks/contextra-bench/src/bin/scale_100k.rs:30` |
| `intervals` | `Vec<IntervalResult>` | `benchmarks/contextra-bench/src/bin/scale_100k.rs:31` |

### Modul `compare`

#### `struct CombinedMetrics`
- Ort: `benchmarks/contextra-bench/src/compare.rs:24`
```rust
pub struct CombinedMetrics
```

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `long_mem_eval` | `Option<LongMemEvalMetricsSummary>` | `benchmarks/contextra-bench/src/compare.rs:26` |
| `locomo` | `Option<LocomoMetricsSummary>` | `benchmarks/contextra-bench/src/compare.rs:28` |

#### `struct LocomoMetricsSummary`
- Ort: `benchmarks/contextra-bench/src/compare.rs:17`
```rust
pub struct LocomoMetricsSummary
```

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `overall_recall_at_5` | `f64` | `benchmarks/contextra-bench/src/compare.rs:18` |
| `overall_mrr` | `f64` | `benchmarks/contextra-bench/src/compare.rs:19` |
| `total_eval_cases` | `usize` | `benchmarks/contextra-bench/src/compare.rs:20` |

#### `struct LongMemEvalMetricsSummary`
- Ort: `benchmarks/contextra-bench/src/compare.rs:11`
```rust
pub struct LongMemEvalMetricsSummary
```

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `overall_accuracy` | `f64` | `benchmarks/contextra-bench/src/compare.rs:12` |
| `total_cases` | `usize` | `benchmarks/contextra-bench/src/compare.rs:13` |

#### `struct MetricComparisonResult`
- Ort: `benchmarks/contextra-bench/src/compare.rs:32`
```rust
pub struct MetricComparisonResult
```

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `has_regression` | `bool` | `benchmarks/contextra-bench/src/compare.rs:33` |
| `errors` | `Vec<String>` | `benchmarks/contextra-bench/src/compare.rs:34` |
| `info_hints` | `Vec<String>` | `benchmarks/contextra-bench/src/compare.rs:35` |
| `pass_messages` | `Vec<String>` | `benchmarks/contextra-bench/src/compare.rs:36` |

#### `fn compare_metrics`
- Ort: `benchmarks/contextra-bench/src/compare.rs:41`
```rust
pub fn compare_metrics( current: &CombinedMetrics, baseline: &CombinedMetrics, threshold: f64, ) -> MetricComparisonResult
```
Dokumentation: Compares current evaluation metrics against baseline metrics using relative tolerance threshold. `threshold` is a relative fraction (e.g. 0.05 = 5%).

#### `fn compare_metrics_files`
- Ort: `benchmarks/contextra-bench/src/compare.rs:195`
```rust
pub fn compare_metrics_files( current_path: &Path, baseline_path: &Path, threshold: f64, ) -> Result<MetricComparisonResult, String>
```

### Modul `crate`

#### `mod adapters`
- Ort: `benchmarks/contextra-bench/src/lib.rs:3`
```rust
pub mod adapters
```

#### `mod ann_benchmarks`
- Ort: `benchmarks/contextra-bench/src/lib.rs:5`
```rust
pub mod ann_benchmarks
```
- Gate: `#[cfg(feature = "external-benchmarks")]`

#### `mod beir_eval`
- Ort: `benchmarks/contextra-bench/src/lib.rs:7`
```rust
pub mod beir_eval
```
- Gate: `#[cfg(feature = "external-benchmarks")]`

#### `mod compare`
- Ort: `benchmarks/contextra-bench/src/lib.rs:8`
```rust
pub mod compare
```

#### `mod matrix`
- Ort: `benchmarks/contextra-bench/src/lib.rs:9`
```rust
pub mod matrix
```

#### `mod metrics`
- Ort: `benchmarks/contextra-bench/src/lib.rs:10`
```rust
pub mod metrics
```

#### `mod path_rag_sweep`
- Ort: `benchmarks/contextra-bench/src/lib.rs:12`
```rust
pub mod path_rag_sweep
```
- Gate: `#[cfg(feature = "datasets")]`

#### `mod regression_gate`
- Ort: `benchmarks/contextra-bench/src/lib.rs:13`
```rust
pub mod regression_gate
```

#### `struct BenchmarkMetrics`
- Ort: `benchmarks/contextra-bench/src/main.rs:43`
```rust
pub struct BenchmarkMetrics
```
Dokumentation: Metrics recorded for a specific evaluation configuration.

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `total_queries` | `usize` | `benchmarks/contextra-bench/src/main.rs:44` |
| `recall_at_1` | `f64` | `benchmarks/contextra-bench/src/main.rs:45` |
| `recall_at_3` | `f64` | `benchmarks/contextra-bench/src/main.rs:46` |
| `recall_at_5` | `f64` | `benchmarks/contextra-bench/src/main.rs:47` |
| `mrr` | `f64` | `benchmarks/contextra-bench/src/main.rs:48` |
| `error_rate_at_1` | `f64` | `benchmarks/contextra-bench/src/main.rs:49` |
| `error_rate_at_5` | `f64` | `benchmarks/contextra-bench/src/main.rs:50` |

#### `struct BenchmarkReport`
- Ort: `benchmarks/contextra-bench/src/main.rs:66`
```rust
pub struct BenchmarkReport
```
Dokumentation: Complete benchmark report.

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `timestamp` | `String` | `benchmarks/contextra-bench/src/main.rs:67` |
| `corpus_size_docs` | `usize` | `benchmarks/contextra-bench/src/main.rs:68` |
| `total_test_queries` | `usize` | `benchmarks/contextra-bench/src/main.rs:69` |
| `scenario_a_context_prefix` | `ScenarioComparison` | `benchmarks/contextra-bench/src/main.rs:70` |
| `scenario_b_reranking` | `ScenarioComparison` | `benchmarks/contextra-bench/src/main.rs:71` |

#### `struct GroundTruthQuery`
- Ort: `benchmarks/contextra-bench/src/main.rs:24`
```rust
pub struct GroundTruthQuery
```
Dokumentation: Represents a benchmark query with ground truth relevant document IDs.

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `query_id` | `String` | `benchmarks/contextra-bench/src/main.rs:25` |
| `query_text` | `String` | `benchmarks/contextra-bench/src/main.rs:26` |
| `query_embedding` | `Vec<f32>` | `benchmarks/contextra-bench/src/main.rs:27` |
| `relevant_doc_ids` | `Vec<String>` | `benchmarks/contextra-bench/src/main.rs:28` |

#### `struct ScenarioComparison`
- Ort: `benchmarks/contextra-bench/src/main.rs:55`
```rust
pub struct ScenarioComparison
```
Dokumentation: Detailed benchmark results comparing baseline vs feature.

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `scenario_name` | `String` | `benchmarks/contextra-bench/src/main.rs:56` |
| `baseline_metrics` | `BenchmarkMetrics` | `benchmarks/contextra-bench/src/main.rs:57` |
| `feature_metrics` | `BenchmarkMetrics` | `benchmarks/contextra-bench/src/main.rs:58` |
| `recall_at_1_delta_pct` | `f64` | `benchmarks/contextra-bench/src/main.rs:59` |
| `recall_at_5_delta_pct` | `f64` | `benchmarks/contextra-bench/src/main.rs:60` |
| `error_rate_reduction_pct` | `f64` | `benchmarks/contextra-bench/src/main.rs:61` |

#### `struct SyntheticDocument`
- Ort: `benchmarks/contextra-bench/src/main.rs:33`
```rust
pub struct SyntheticDocument
```
Dokumentation: Raw document chunk item in the synthetic corpus.

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `id` | `String` | `benchmarks/contextra-bench/src/main.rs:34` |
| `title` | `String` | `benchmarks/contextra-bench/src/main.rs:35` |
| `raw_content` | `String` | `benchmarks/contextra-bench/src/main.rs:36` |
| `context_prefix` | `String` | `benchmarks/contextra-bench/src/main.rs:37` |
| `embedding` | `Vec<f32>` | `benchmarks/contextra-bench/src/main.rs:38` |

### Modul `matrix`

#### `struct ComparisonMatrixReport`
- Ort: `benchmarks/contextra-bench/src/matrix.rs:60`
```rust
pub struct ComparisonMatrixReport
```
Dokumentation: Gesamte Vergleichsmatrix über alle Evaluierungsmodi

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `date` | `String` | `benchmarks/contextra-bench/src/matrix.rs:61` |
| `dataset_name` | `String` | `benchmarks/contextra-bench/src/matrix.rs:62` |
| `total_queries` | `usize` | `benchmarks/contextra-bench/src/matrix.rs:63` |
| `total_documents` | `usize` | `benchmarks/contextra-bench/src/matrix.rs:64` |
| `rows` | `Vec<StrategyMatrixRow>` | `benchmarks/contextra-bench/src/matrix.rs:65` |

#### `struct StrategyMatrixRow`
- Ort: `benchmarks/contextra-bench/src/matrix.rs:51`
```rust
pub struct StrategyMatrixRow
```
Dokumentation: Ergebnis für eine einzelne Strategie in der Matrix

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `strategy` | `EvaluatedStrategy` | `benchmarks/contextra-bench/src/matrix.rs:52` |
| `name` | `String` | `benchmarks/contextra-bench/src/matrix.rs:53` |
| `metrics` | `Option<AggregateMetrics>` | `benchmarks/contextra-bench/src/matrix.rs:54` |
| `status` | `String` | `benchmarks/contextra-bench/src/matrix.rs:55` |

#### `enum EvaluatedStrategy`
- Ort: `benchmarks/contextra-bench/src/matrix.rs:27`
```rust
pub enum EvaluatedStrategy
```
Dokumentation: Getestete Retrieval-Strategie in der Vergleichsmatrix
- Impl-Block `benchmarks/contextra-bench/src/matrix.rs:36`

Varianten:

- `VectorOnly` — `benchmarks/contextra-bench/src/matrix.rs:28`
- `Bm25Only` — `benchmarks/contextra-bench/src/matrix.rs:29`
- `RrfVectorText` — `benchmarks/contextra-bench/src/matrix.rs:30`
- `RrfVectorTextGraph` — `benchmarks/contextra-bench/src/matrix.rs:31`
- `WeightedRrf` — `benchmarks/contextra-bench/src/matrix.rs:32`
- `ScoreNormalizedLinear` — `benchmarks/contextra-bench/src/matrix.rs:33`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn name(&self) -> &'static str` | `benchmarks/contextra-bench/src/matrix.rs:37` |  |

#### `fn from_rank_items`
- Ort: `benchmarks/contextra-bench/src/matrix.rs:77`
```rust
pub fn from_rank_items(items: &[ScoredItem]) -> Vec<RankedResult>
```
Dokumentation: Konvertiert `contextra_rank::ScoredItem` zurück in `RankedResult`

#### `fn fuse_via_rank_rrf`
- Ort: `benchmarks/contextra-bench/src/matrix.rs:102`
```rust
pub fn fuse_via_rank_rrf( lists: &[Vec<ScoredItem>], k: usize, ) -> Result<Vec<ScoredItem>, contextra_rank::RankError>
```
Dokumentation: Führt RRF-Fusion über die öffentliche `contextra_rank::rrf_fuse` API aus

#### `fn fuse_via_rank_score_normalized`
- Ort: `benchmarks/contextra-bench/src/matrix.rs:118`
```rust
pub fn fuse_via_rank_score_normalized( signal_sets: Vec<(String, Vec<SearchResult>, f32)>, max_results: usize, ) -> Vec<SearchResult>
```
Dokumentation: Führt normalisierte Linearkombination über `contextra_rank::score_normalized_fusion_with_options` aus

#### `fn fuse_via_rank_weighted_rrf`
- Ort: `benchmarks/contextra-bench/src/matrix.rs:110`
```rust
pub fn fuse_via_rank_weighted_rrf( signal_sets: Vec<(String, Vec<SearchResult>, f32)>, max_results: usize, ) -> Vec<SearchResult>
```
Dokumentation: Führt gewichtete RRF-Fusion über die öffentliche `contextra_rank::weighted_reciprocal_rank_fusion` API aus

#### `fn to_rank_items`
- Ort: `benchmarks/contextra-bench/src/matrix.rs:69`
```rust
pub fn to_rank_items(results: &[RankedResult]) -> Vec<ScoredItem>
```
Dokumentation: Konvertiert interne `RankedResult` in `contextra_rank::ScoredItem`

#### `fn to_search_results`
- Ort: `benchmarks/contextra-bench/src/matrix.rs:88`
```rust
pub fn to_search_results(results: &[RankedResult]) -> Vec<SearchResult>
```
Dokumentation: Konvertiert interne `RankedResult` in `contextra_rank::SearchResult`

### Modul `metrics`

#### `struct AggregateMetrics`
- Ort: `benchmarks/contextra-bench/src/metrics.rs:40`
```rust
pub struct AggregateMetrics
```
Dokumentation: Aggregierte Gesamtmetriken

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `total_queries` | `usize` | `benchmarks/contextra-bench/src/metrics.rs:41` |
| `recall_at_10` | `f64` | `benchmarks/contextra-bench/src/metrics.rs:42` |
| `ndcg_at_10` | `f64` | `benchmarks/contextra-bench/src/metrics.rs:43` |
| `mrr_at_10` | `f64` | `benchmarks/contextra-bench/src/metrics.rs:44` |
| `latency_p50_us` | `u64` | `benchmarks/contextra-bench/src/metrics.rs:45` |
| `latency_p95_us` | `u64` | `benchmarks/contextra-bench/src/metrics.rs:46` |
| `latency_p99_us` | `u64` | `benchmarks/contextra-bench/src/metrics.rs:47` |
| `peak_rss_bytes` | `u64` | `benchmarks/contextra-bench/src/metrics.rs:48` |
| `index_build_duration_ms` | `u64` | `benchmarks/contextra-bench/src/metrics.rs:49` |

#### `struct QueryEvaluation`
- Ort: `benchmarks/contextra-bench/src/metrics.rs:31`
```rust
pub struct QueryEvaluation
```
Dokumentation: Evaluierungsbericht für eine einzelne Abfrage

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `query_id` | `String` | `benchmarks/contextra-bench/src/metrics.rs:32` |
| `recall_at_10` | `f64` | `benchmarks/contextra-bench/src/metrics.rs:33` |
| `ndcg_at_10` | `f64` | `benchmarks/contextra-bench/src/metrics.rs:34` |
| `mrr_at_10` | `f64` | `benchmarks/contextra-bench/src/metrics.rs:35` |

#### `struct RankedResult`
- Ort: `benchmarks/contextra-bench/src/metrics.rs:24`
```rust
pub struct RankedResult
```
Dokumentation: Ein einzelnes abgerufenes Dokument mit Rang

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `doc_id` | `String` | `benchmarks/contextra-bench/src/metrics.rs:25` |
| `score` | `f64` | `benchmarks/contextra-bench/src/metrics.rs:26` |

#### `struct RelevanceGrade`
- Ort: `benchmarks/contextra-bench/src/metrics.rs:20`
```rust
pub struct RelevanceGrade(pub f64)
```
Dokumentation: Relevanzurteil für ein Dokument

#### `fn calculate_latency_percentiles`
- Ort: `benchmarks/contextra-bench/src/metrics.rs:153`
```rust
pub fn calculate_latency_percentiles(mut latencies: Vec<Duration>) -> (u64, u64, u64)
```
Dokumentation: Berechnet Perzentile p50, p95, p99 einer Latenzreihe in Mikrosekunden

#### `fn calculate_mrr_at_k`
- Ort: `benchmarks/contextra-bench/src/metrics.rs:92`
```rust
pub fn calculate_mrr_at_k( retrieved: &[RankedResult], ground_truth: &HashMap<String, f64>, k: usize, relevance_threshold: f64, ) -> f64
```
Dokumentation: Berechnet Mean Reciprocal Rank (MRR@k) für eine Abfrage MRR@k = 1 / erster relevanter Rang (1-basiert) innerhalb von Top-k. Falls kein relevantes Dokument in Top-k ist: 0.0.

#### `fn calculate_ndcg_at_k`
- Ort: `benchmarks/contextra-bench/src/metrics.rs:112`
```rust
pub fn calculate_ndcg_at_k( retrieved: &[RankedResult], ground_truth: &HashMap<String, f64>, k: usize, ) -> f64
```
Dokumentation: Berechnet Normalized Discounted Cumulative Gain (nDCG@k) DCG@k = \sum_{i=1}^k \frac{2^{rel_i} - 1}{\log_2(i + 1)} nDCG@k = DCG@k / IDCG@k

#### `fn calculate_recall_at_k`
- Ort: `benchmarks/contextra-bench/src/metrics.rs:56`
```rust
pub fn calculate_recall_at_k( retrieved: &[RankedResult], ground_truth: &HashMap<String, f64>, k: usize, relevance_threshold: f64, ) -> f64
```
Dokumentation: Berechnet Recall@k Recall@k = |relevante Dokumente in Top-k| / |alle relevanten Dokumente| Wenn es keine relevanten Dokumente gibt, wird 0.0 zurückgegeben.

#### `fn read_process_rss_bytes`
- Ort: `benchmarks/contextra-bench/src/metrics.rs:178`
```rust
pub fn read_process_rss_bytes() -> u64
```
Dokumentation: Liest den aktuellen VmRSS (Resident Set Size) in Bytes aus /proc/self/status (Linux)

### Modul `path_rag_sweep`

#### `struct PathRagSweepMetric`
- Ort: `benchmarks/contextra-bench/src/path_rag_sweep.rs:23`
```rust
pub struct PathRagSweepMetric
```

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `threshold` | `f64` | `benchmarks/contextra-bench/src/path_rag_sweep.rs:24` |
| `recall_at_5` | `f64` | `benchmarks/contextra-bench/src/path_rag_sweep.rs:25` |
| `recall_at_10` | `f64` | `benchmarks/contextra-bench/src/path_rag_sweep.rs:26` |
| `precision_at_5` | `f64` | `benchmarks/contextra-bench/src/path_rag_sweep.rs:27` |
| `precision_at_10` | `f64` | `benchmarks/contextra-bench/src/path_rag_sweep.rs:28` |
| `total_queries` | `usize` | `benchmarks/contextra-bench/src/path_rag_sweep.rs:29` |

#### `struct PathRagSweepReport`
- Ort: `benchmarks/contextra-bench/src/path_rag_sweep.rs:33`
```rust
pub struct PathRagSweepReport
```

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `timestamp` | `String` | `benchmarks/contextra-bench/src/path_rag_sweep.rs:34` |
| `long_mem_eval_sweep` | `Vec<PathRagSweepMetric>` | `benchmarks/contextra-bench/src/path_rag_sweep.rs:35` |
| `locomo_sweep` | `Vec<PathRagSweepMetric>` | `benchmarks/contextra-bench/src/path_rag_sweep.rs:36` |

#### `fn run_pathrag_sweep_locomo`
- Ort: `benchmarks/contextra-bench/src/path_rag_sweep.rs:198`
```rust
pub async fn run_pathrag_sweep_locomo( locomo_dataset_path: &Path, thresholds: &[f64], ) -> Result<Vec<PathRagSweepMetric>>
```

#### `fn run_pathrag_sweep_long_mem_eval`
- Ort: `benchmarks/contextra-bench/src/path_rag_sweep.rs:39`
```rust
pub async fn run_pathrag_sweep_long_mem_eval( thresholds: &[f64], ) -> Result<Vec<PathRagSweepMetric>>
```

### Modul `regression_gate`

#### `struct RegressionGateResult`
- Ort: `benchmarks/contextra-bench/src/regression_gate.rs:18`
```rust
pub struct RegressionGateResult
```

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `passed` | `bool` | `benchmarks/contextra-bench/src/regression_gate.rs:19` |
| `updated` | `bool` | `benchmarks/contextra-bench/src/regression_gate.rs:20` |
| `message` | `String` | `benchmarks/contextra-bench/src/regression_gate.rs:21` |

#### `fn run_regression_gate`
- Ort: `benchmarks/contextra-bench/src/regression_gate.rs:26`
```rust
pub fn run_regression_gate( results_path: &Path, baseline_path: &Path, threshold: f64, update_baseline: bool, ) -> Result<RegressionGateResult, String>
```
Dokumentation: Evaluates current metrics file against baseline file using specified tolerance threshold. If `update_baseline` is true or if baseline does not exist, writes/updates baseline file.

#### `const DEFAULT_TOLERANCE_THRESHOLD`
- Ort: `benchmarks/contextra-bench/src/regression_gate.rs:15`
```rust
pub const DEFAULT_TOLERANCE_THRESHOLD: f64 = 0.05
```
Dokumentation: Standard tolerance threshold justification: Empirical evaluation over multiple runs on synthetic and LongMemEval benchmark cases shows a run-to-run retrieval variance of ~1.5% to 2.5% due to hybrid score tie-breaking and vector float rounding. A relative tolerance threshold of 5.0% (0.05) provides a statistically robust gate that flags true quality degradation while avoiding false-positive CI failures caused by sample noise.
