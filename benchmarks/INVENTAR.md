## contextra-bench

Pfad: `benchmarks/contextra-bench`

Abhängigkeiten (direkt): contextra-store, contextra-vector, contextra-text, contextra-db, tokio, serde, serde_json, tempfile, tracing, tracing-subscriber, anyhow, hdrhistogram, rand, contextra-rank, contextra-types

### Module

- `/src/adapters.rs`
- `/src/ann_benchmarks.rs`
- `/src/beir_eval.rs`
- `/src/bin/compare_baseline.rs`
- `/src/bin/scale_100k.rs`
- `/src/compare.rs`
- `/src/lib.rs`
- `/src/main.rs`
- `/src/matrix.rs`
- `/src/metrics.rs`
- `/src/path_rag_sweep.rs`
- `/src/regression_gate.rs`

### Öffentliche Typen

- **AggregateMetrics** (struct) – src/metrics.rs:40
- **AnnBenchResult** (struct) – src/ann_benchmarks.rs:19
- **BeirDocument** (struct) – src/beir_eval.rs:17
- **BeirMetrics** (struct) – src/beir_eval.rs:31
- **BeirQuery** (struct) – src/beir_eval.rs:24
- **BenchmarkDocument** (struct) – src/adapters.rs:14
- **BenchmarkMetrics** (struct) – src/main.rs:41
- **BenchmarkReport** (struct) – src/main.rs:64
- **CombinedMetrics** (struct) – src/compare.rs:24
- **ComparisonMatrixReport** (struct) – src/matrix.rs:60
- **EngineHit** (struct) – src/adapters.rs:23
- **EvaluatedStrategy** (enum) – src/matrix.rs:27
- **FullTextEngineAdapter** (trait) – src/adapters.rs:36
- **GroundTruthQuery** (struct) – src/main.rs:22
- **IntervalResult** (struct) – src/bin/scale_100k.rs:13
- **LanceDbAdapterSketch** (struct) – src/adapters.rs:112
- **LocomoMetricsSummary** (struct) – src/compare.rs:17
- **LongMemEvalMetricsSummary** (struct) – src/compare.rs:11
- **MetricComparisonResult** (struct) – src/compare.rs:32
- **PathRagSweepMetric** (struct) – src/path_rag_sweep.rs:23
- **PathRagSweepReport** (struct) – src/path_rag_sweep.rs:33
- **QueryEvaluation** (struct) – src/metrics.rs:31
- **QueryLatencyStats** (struct) – src/adapters.rs:30
- **RankedResult** (struct) – src/metrics.rs:24
- **RegressionGateResult** (struct) – src/regression_gate.rs:18
- **RelevanceGrade** (struct) – src/metrics.rs:20
- **ScaleBenchmarkOutput** (struct) – src/bin/scale_100k.rs:23
- **ScenarioComparison** (struct) – src/main.rs:53
- **StrategyMatrixRow** (struct) – src/matrix.rs:51
- **SyntheticDocument** (struct) – src/main.rs:31
- **TantivyAdapterSketch** (struct) – src/adapters.rs:73
- **VectorEngineAdapter** (trait) – src/adapters.rs:54

### Öffentliche Funktionen

- `pub async fn run_ann_benchmark(`  – src/ann_benchmarks.rs:136
- `pub async fn run_ann_benchmark_synthetic_ci() -> Result<AnnBenchResult> `  – src/ann_benchmarks.rs:249
- `pub async fn run_beir_eval(`  – src/beir_eval.rs:261
- `pub async fn run_pathrag_sweep_locomo(`  – src/path_rag_sweep.rs:198
- `pub async fn run_pathrag_sweep_long_mem_eval(`  – src/path_rag_sweep.rs:39
- `pub fn calculate_latency_percentiles(mut latencies: Vec<Duration>) -> (u64, u64, u64) `  – src/metrics.rs:153
- `pub fn calculate_mrr_at_k(`  – src/metrics.rs:92
- `pub fn calculate_ndcg_at_k(`  – src/metrics.rs:112
- `pub fn calculate_recall_at_k(`  – src/metrics.rs:56
- `pub fn compare_metrics(`  – src/compare.rs:41
- `pub fn compare_metrics_files(`  – src/compare.rs:195
- `pub fn compute_ground_truth(`  – src/ann_benchmarks.rs:80
- `pub fn from_rank_items(items: &[ScoredItem]) -> Vec<RankedResult> `  – src/matrix.rs:77
- `pub fn fuse_via_rank_rrf(`  – src/matrix.rs:102
- `pub fn fuse_via_rank_score_normalized(`  – src/matrix.rs:118
- `pub fn fuse_via_rank_weighted_rrf(`  – src/matrix.rs:110
- `pub fn generate_synthetic_vectors(n: usize, dim: usize, seed: u64) -> Vec<Vec<f32>> `  – src/ann_benchmarks.rs:35
- `pub fn load_beir_corpus(path: &Path) -> Result<Vec<BeirDocument>> `  – src/beir_eval.rs:43
- `pub fn load_beir_queries(queries_path: &Path, qrels_path: &Path) -> Result<Vec<BeirQuery>> `  – src/beir_eval.rs:81
- `pub fn map_at_k(ranked: &[String], relevant: &HashSet<String>, k: usize) -> f64 `  – src/beir_eval.rs:214
- `pub fn name(&self) -> &'static str `  – src/matrix.rs:37
- `pub fn ndcg_at_k(ranked: &[String], relevant: &HashSet<String>, k: usize) -> f64 `  – src/beir_eval.rs:185
- `pub fn new(index_path: impl Into<String>) -> Self `  – src/adapters.rs:79
- `pub fn new(uri: impl Into<String>, table_name: impl Into<String>, dimension: usize) -> Self `  – src/adapters.rs:119
- `pub fn read_process_rss_bytes() -> u64 `  – src/metrics.rs:178
- `pub fn recall_at_k(ranked: &[String], relevant: &HashSet<String>, k: usize) -> f64 `  – src/beir_eval.rs:239
- `pub fn recall_at_k(retrieved: &[u64], ground_truth: &[usize], k: usize) -> f64 `  – src/ann_benchmarks.rs:112
- `pub fn run_regression_gate(`  – src/regression_gate.rs:26
- `pub fn to_rank_items(results: &[RankedResult]) -> Vec<ScoredItem> `  – src/matrix.rs:69
- `pub fn to_search_results(results: &[RankedResult]) -> Vec<SearchResult> `  – src/matrix.rs:88

