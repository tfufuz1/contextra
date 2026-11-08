# Cherry-Pick-Katalog: Historisch entfernte und verworfene Funktionen

Dieser Katalog dokumentiert alle Funktionen, Methoden und Algorithmen, die im reifsten, funktionsreichen Zustand (Stand 16. Oktober 2026 / Tags `archiv/v0`, `park/EP1..EP6/v1`) implementiert waren, jedoch im Zuge der Baukasten-Transformation, Entkopplung oder Vereinfachung aus dem aktuellen HEAD entfernt oder modifiziert wurden.

> **Gesamtanzahl erfasster entfernter Funktionen:** 304
> **Zweck:** Präzise Referenz für gezielte Cherry-Picks und Wiederinbetriebnahme hochentwickelter Algorithmen.

## Übersicht nach Themenbereichen & Domänen

| Domäne / Kategorie | Anzahl Funktionen | Primäre Cherry-Pick-Referenz |
|---|---|---|
| **Agent & Workflow (Scratchpad, DLQ, GoalCondition, Replay)** | 16 | `05a6a9af (archiv/v0) / 8412f0862` |
| **Allgemeine Hilfsfunktionen & Benchmarks** | 62 | `7fddc8504 (Stand 16. Oktober 2026)` |
| **Engine: Transaktionen & Background Maintenance** | 44 | `7fddc8504 / 8412f0862` |
| **Graph & Hyperedges (Wissensgraph, PPR, Lovasz, Cascade)** | 126 | `05a6a9af (archiv/v0) / 7fddc8504` |
| **Kryptographie & DeletionProof (Löschnachweise, RevocationLog)** | 31 | `05a6a9af (archiv/v0) / 7fddc8504` |
| **MCP-Protokoll & Tools (Parking Tools, Egress Gateway)** | 12 | `7fddc8504 (Stand 16. Oktober 2026)` |
| **Privacy & Egress Schutz** | 2 | `7fddc8504 (Stand 16. Oktober 2026)` |
| **Ranking & Kalibrierung (DiBud, Conformal Prediction)** | 1 | `c27326391 / 5d349c188` |
| **WASM Sandbox & Isolation** | 10 | `7fddc8504 (Stand 16. Oktober 2026)` |

## Übersicht nach Crates

| Crate | Entfernte Funktionen |
|---|---|
| `(root)` | 21 |
| `contextra-agent` | 16 |
| `contextra-avv-generator` | 1 |
| `contextra-bench` | 24 |
| `contextra-cognition` | 4 |
| `contextra-crypto` | 27 |
| `contextra-db` | 4 |
| `contextra-engine` | 44 |
| `contextra-graph` | 126 |
| `contextra-infer-onnx` | 2 |
| `contextra-mcp` | 12 |
| `contextra-ports` | 4 |
| `contextra-privacy` | 2 |
| `contextra-rank` | 1 |
| `contextra-sandbox` | 10 |
| `contextra-store` | 6 |

## Detail-Katalog der entfernten Funktionen (nach Domänen gegliedert)

### Agent & Workflow (Scratchpad, DLQ, GoalCondition, Replay)

| Crate | Historische Datei & Zeile | Funktion / Methode | Kontext | Signatur | Cherry-Pick-Quelle |
|---|---|---|---|---|---|
| `contextra-agent` | `crates/contextra-agent/src/audit.rs:570` | `InMemoryStorageEngine::get` | `impl StorageEngine for InMemoryStorageEngine` | `fn get<'a>(&'a self, key: &'a [u8]) -> BoxFuture<'a, Result<Option<bytes::Bytes>>>` | `05a6a9af (archiv/v0) / 8412f0862` |
| `contextra-agent` | `crates/contextra-agent/src/audit.rs:579` | `InMemoryStorageEngine::get_at_seq` | `impl StorageEngine for InMemoryStorageEngine` | `fn get_at_seq<'a>( &'a self, key: &'a [u8], _seq: u64, ) -> BoxFuture<'a, Result<Option<bytes::Bytes>>>` | `05a6a9af (archiv/v0) / 8412f0862` |
| `contextra-agent` | `crates/contextra-agent/src/audit.rs:587` | `InMemoryStorageEngine::put` | `impl StorageEngine for InMemoryStorageEngine` | `fn put<'a>( &'a self, _tx_id: contextra_types::TxId, key: &'a [u8], value: &'a [u8], ) -> BoxFuture<'a, Result<()>>` | `05a6a9af (archiv/v0) / 8412f0862` |
| `contextra-agent` | `crates/contextra-agent/src/audit.rs:602` | `InMemoryStorageEngine::put_if_absent` | `impl StorageEngine for InMemoryStorageEngine` | `fn put_if_absent<'a>( &'a self, _tx_id: contextra_types::TxId, key: &'a [u8], value: &'a [u8], ) -> BoxFuture<'a, Result<bool>>` | `05a6a9af (archiv/v0) / 8412f0862` |
| `contextra-agent` | `crates/contextra-agent/src/audit.rs:621` | `InMemoryStorageEngine::delete` | `impl StorageEngine for InMemoryStorageEngine` | `fn delete<'a>( &'a self, _tx_id: contextra_types::TxId, key: &'a [u8], ) -> BoxFuture<'a, Result<()>>` | `05a6a9af (archiv/v0) / 8412f0862` |
| `contextra-agent` | `crates/contextra-agent/src/audit.rs:635` | `InMemoryStorageEngine::commit` | `impl StorageEngine for InMemoryStorageEngine` | `fn commit<'a>(&'a self, _tx_id: contextra_types::TxId) -> BoxFuture<'a, Result<()>>` | `05a6a9af (archiv/v0) / 8412f0862` |
| `contextra-agent` | `crates/contextra-agent/src/audit.rs:639` | `InMemoryStorageEngine::rollback` | `impl StorageEngine for InMemoryStorageEngine` | `fn rollback<'a>(&'a self, _tx_id: contextra_types::TxId) -> BoxFuture<'a, Result<()>>` | `05a6a9af (archiv/v0) / 8412f0862` |
| `contextra-agent` | `crates/contextra-agent/src/audit.rs:643` | `InMemoryStorageEngine::rollback_to_tx` | `impl StorageEngine for InMemoryStorageEngine` | `fn rollback_to_tx<'a>(&'a self, _tx_id: contextra_types::TxId) -> BoxFuture<'a, Result<()>>` | `05a6a9af (archiv/v0) / 8412f0862` |
| `contextra-agent` | `crates/contextra-agent/src/audit.rs:647` | `InMemoryStorageEngine::flush` | `impl StorageEngine for InMemoryStorageEngine` | `fn flush<'a>(&'a self) -> BoxFuture<'a, Result<()>>` | `05a6a9af (archiv/v0) / 8412f0862` |
| `contextra-agent` | `crates/contextra-agent/src/audit.rs:651` | `InMemoryStorageEngine::stats` | `impl StorageEngine for InMemoryStorageEngine` | `fn stats<'a>(&'a self) -> BoxFuture<'a, Result<contextra_ports::StorageStats>>` | `05a6a9af (archiv/v0) / 8412f0862` |
| `contextra-agent` | `crates/contextra-agent/src/audit.rs:661` | `InMemoryStorageEngine::last_seq_no` | `impl StorageEngine for InMemoryStorageEngine` | `fn last_seq_no<'a>(&'a self) -> BoxFuture<'a, Result<u64>>` | `05a6a9af (archiv/v0) / 8412f0862` |
| `contextra-agent` | `crates/contextra-agent/src/audit.rs:665` | `InMemoryStorageEngine::last_tx_id` | `impl StorageEngine for InMemoryStorageEngine` | `fn last_tx_id<'a>(&'a self) -> BoxFuture<'a, Result<contextra_types::TxId>>` | `05a6a9af (archiv/v0) / 8412f0862` |
| `contextra-agent` | `crates/contextra-agent/src/audit.rs:669` | `InMemoryStorageEngine::pin_checkpoint` | `impl StorageEngine for InMemoryStorageEngine` | `fn pin_checkpoint<'a>(&'a self, _seq_no: u64) -> BoxFuture<'a, Result<()>>` | `05a6a9af (archiv/v0) / 8412f0862` |
| `contextra-agent` | `crates/contextra-agent/src/audit.rs:673` | `InMemoryStorageEngine::unpin_checkpoint` | `impl StorageEngine for InMemoryStorageEngine` | `fn unpin_checkpoint<'a>(&'a self, _seq_no: u64) -> BoxFuture<'a, Result<()>>` | `05a6a9af (archiv/v0) / 8412f0862` |
| `contextra-agent` | `crates/contextra-agent/src/audit.rs:677` | `InMemoryStorageEngine::scan_prefix` | `impl StorageEngine for InMemoryStorageEngine` | `fn scan_prefix<'a>( &'a self, prefix: &'a [u8], ) -> BoxFuture<'a, Result<Vec<(Vec<u8>, Vec<u8>)>>>` | `05a6a9af (archiv/v0) / 8412f0862` |
| `contextra-agent` | `crates/contextra-agent/src/audit.rs:694` | `InMemoryStorageEngine::scan` | `impl StorageEngine for InMemoryStorageEngine` | `fn scan<'a>( &'a self, start: std::ops::Bound<&'a [u8]>, end: std::ops::Bound<&'a [u8]>, limit: Option<usize>, ) -> BoxFuture<'a, Result<Vec<(Vec<u8>, Vec<u8>)>>>` | `05a6a9af (archiv/v0) / 8412f0862` |

### Allgemeine Hilfsfunktionen & Benchmarks

| Crate | Historische Datei & Zeile | Funktion / Methode | Kontext | Signatur | Cherry-Pick-Quelle |
|---|---|---|---|---|---|
| `(root)` | `benches/competitive_bench.rs:13` | `generate_embedding` | `-` | `fn generate_embedding(doc_idx: usize) -> Vec<f32>` | `7fddc8504 (Stand 16. Oktober 2026)` |
| `(root)` | `benches/competitive_bench.rs:24` | `generate_content` | `-` | `fn generate_content(doc_idx: usize) -> String` | `7fddc8504 (Stand 16. Oktober 2026)` |
| `(root)` | `benches/competitive_bench.rs:31` | `bench_write_throughput` | `-` | `fn bench_write_throughput(c: &mut Criterion)` | `7fddc8504 (Stand 16. Oktober 2026)` |
| `(root)` | `benches/competitive_bench.rs:71` | `bench_hybrid_search_latency` | `-` | `fn bench_hybrid_search_latency(c: &mut Criterion)` | `7fddc8504 (Stand 16. Oktober 2026)` |
| `(root)` | `benches/competitive_bench.rs:114` | `bench_compaction_throughput` | `-` | `fn bench_compaction_throughput(c: &mut Criterion)` | `7fddc8504 (Stand 16. Oktober 2026)` |
| `(root)` | `benches/migration_benchmarks.rs:12` | `bench_hybrid_search` | `-` | `fn bench_hybrid_search(c: &mut Criterion)` | `7fddc8504 (Stand 16. Oktober 2026)` |
| `(root)` | `benches/migration_benchmarks.rs:44` | `bench_agent_state_checkpoint` | `-` | `fn bench_agent_state_checkpoint(c: &mut Criterion)` | `7fddc8504 (Stand 16. Oktober 2026)` |
| `(root)` | `benches/migration_benchmarks.rs:61` | `bench_rerun_cost` | `-` | `fn bench_rerun_cost(c: &mut Criterion)` | `7fddc8504 (Stand 16. Oktober 2026)` |
| `(root)` | `benches/migration_benchmarks.rs:90` | `bench_snapshot_overhead` | `-` | `fn bench_snapshot_overhead(c: &mut Criterion)` | `7fddc8504 (Stand 16. Oktober 2026)` |
| `(root)` | `benches/migration_benchmarks.rs:123` | `bench_staged_stats_commit` | `-` | `fn bench_staged_stats_commit(c: &mut Criterion)` | `7fddc8504 (Stand 16. Oktober 2026)` |
| `(root)` | `benches/relate_bench.rs:10` | `bench_relate_performance` | `-` | `fn bench_relate_performance(c: &mut Criterion)` | `7fddc8504 (Stand 16. Oktober 2026)` |
| `(root)` | `benches/rrf_scale_bench.rs:10` | `get_vm_rss_kb` | `-` | `fn get_vm_rss_kb() -> Option<u64>` | `7fddc8504 (Stand 16. Oktober 2026)` |
| `(root)` | `benches/rrf_scale_bench.rs:24` | `log_rss_measurement` | `-` | `fn log_rss_measurement( stage: &str, hits_per_signal: usize, total_hits: usize, rss_kb: u64, latency_micros: f64, )` | `7fddc8504 (Stand 16. Oktober 2026)` |
| `(root)` | `benches/rrf_scale_bench.rs:64` | `generate_signal_results` | `-` | `fn generate_signal_results( signal_name: &str, count: usize, overlap_factor: usize, ) -> Vec<SearchResult>` | `7fddc8504 (Stand 16. Oktober 2026)` |
| `(root)` | `benches/rrf_scale_bench.rs:92` | `bench_rrf_scaling` | `-` | `fn bench_rrf_scaling(c: &mut Criterion)` | `7fddc8504 (Stand 16. Oktober 2026)` |
| `(root)` | `benches/scale_bench.rs:17` | `get_vm_rss_kb` | `-` | `fn get_vm_rss_kb() -> Option<u64>` | `7fddc8504 (Stand 16. Oktober 2026)` |
| `(root)` | `benches/scale_bench.rs:31` | `log_rss_measurement` | `-` | `fn log_rss_measurement(stage: &str, num_chunks: usize, rss_kb: u64)` | `7fddc8504 (Stand 16. Oktober 2026)` |
| `(root)` | `benches/scale_bench.rs:70` | `insert_batch_adaptive` | `-` | `fn insert_batch_adaptive<'a>( db: &'a Contextra, batch: &'a [(String, Vec<f32>, Option<serde_json::Value>)], batch_size: &'a mut usize, ) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send + 'a>>` | `7fddc8504 (Stand 16. Oktober 2026)` |
| `(root)` | `benches/scale_bench.rs:104` | `generate_embedding` | `-` | `fn generate_embedding(doc_idx: usize) -> Vec<f32>` | `7fddc8504 (Stand 16. Oktober 2026)` |
| `(root)` | `benches/scale_bench.rs:116` | `generate_content` | `-` | `fn generate_content(doc_idx: usize) -> String` | `7fddc8504 (Stand 16. Oktober 2026)` |
| `(root)` | `benches/scale_bench.rs:134` | `bench_scale_inserts_and_search` | `-` | `fn bench_scale_inserts_and_search(c: &mut Criterion)` | `7fddc8504 (Stand 16. Oktober 2026)` |
| `contextra-bench` | `benchmarks/contextra-bench/benches/locomo.rs:11` | `get_fixture_path` | `-` | `fn get_fixture_path(relative: &str) -> PathBuf` | `7fddc8504 (Stand 16. Oktober 2026)` |
| `contextra-bench` | `benchmarks/contextra-bench/benches/locomo.rs:26` | `ensure_fd_limit` | `-` | `fn ensure_fd_limit(min_fds: u64)` | `7fddc8504 (Stand 16. Oktober 2026)` |
| `contextra-bench` | `benchmarks/contextra-bench/benches/locomo.rs:45` | `bench_locomo_eval` | `-` | `fn bench_locomo_eval(c: &mut Criterion)` | `7fddc8504 (Stand 16. Oktober 2026)` |
| `contextra-bench` | `benchmarks/contextra-bench/benches/long_mem_eval.rs:10` | `get_fixture_path` | `-` | `fn get_fixture_path(relative: &str) -> PathBuf` | `7fddc8504 (Stand 16. Oktober 2026)` |
| `contextra-bench` | `benchmarks/contextra-bench/benches/long_mem_eval.rs:25` | `ensure_fd_limit` | `-` | `fn ensure_fd_limit(min_fds: u64)` | `7fddc8504 (Stand 16. Oktober 2026)` |
| `contextra-bench` | `benchmarks/contextra-bench/benches/long_mem_eval.rs:44` | `bench_long_mem_eval` | `-` | `fn bench_long_mem_eval(c: &mut Criterion)` | `7fddc8504 (Stand 16. Oktober 2026)` |
| `contextra-bench` | `benchmarks/contextra-bench/src/locomo.rs:25` | `LocomoQuestionCategory::from_u8` | `impl LocomoQuestionCategory` | `pub fn from_u8(val: u8) -> Option<Self>` | `7fddc8504 (Stand 16. Oktober 2026)` |
| `contextra-bench` | `benchmarks/contextra-bench/src/locomo.rs:38` | `LocomoQuestionCategory::fmt` | `impl Display for LocomoQuestionCategory` | `fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result` | `7fddc8504 (Stand 16. Oktober 2026)` |
| `contextra-bench` | `benchmarks/contextra-bench/src/locomo.rs:85` | `LocomoDataset::deref` | `impl Deref for LocomoDataset` | `fn deref(&self) -> &Self::Target` | `7fddc8504 (Stand 16. Oktober 2026)` |
| `contextra-bench` | `benchmarks/contextra-bench/src/locomo.rs:129` | `extract_turns` | `-` | `fn extract_turns(sample_id: &str, turns: &[RawLocomoTurn], index: &mut LocomoConversationIndex)` | `7fddc8504 (Stand 16. Oktober 2026)` |
| `contextra-bench` | `benchmarks/contextra-bench/src/locomo.rs:144` | `load_locomo_dataset` | `-` | `pub fn load_locomo_dataset(path: &Path) -> Result<LocomoDataset>` | `7fddc8504 (Stand 16. Oktober 2026)` |
| `contextra-bench` | `benchmarks/contextra-bench/src/locomo.rs:240` | `run_locomo_eval` | `-` | `pub async fn run_locomo_eval<'a, F>( cases: &[LocomoCase], conversation_index: &LocomoConversationIndex, search_fn: F, ) -> Result<LocomoReport> where F: Fn(&str) -> BoxFuture<'a, Result<Vec<ScoredChunk>>>,` | `7fddc8504 (Stand 16. Oktober 2026)` |
| `contextra-bench` | `benchmarks/contextra-bench/src/long_mem_eval.rs:40` | `LongMemEvalQuestionType::fmt` | `impl Display for LongMemEvalQuestionType` | `fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result` | `7fddc8504 (Stand 16. Oktober 2026)` |
| `contextra-bench` | `benchmarks/contextra-bench/src/long_mem_eval.rs:68` | `LongMemEvalCase::answer_str` | `impl LongMemEvalCase` | `pub fn answer_str(&self) -> String` | `7fddc8504 (Stand 16. Oktober 2026)` |
| `contextra-bench` | `benchmarks/contextra-bench/src/long_mem_eval.rs:80` | `LongMemEvalCase::expected_answer` | `impl LongMemEvalCase` | `pub fn expected_answer(&self) -> String` | `7fddc8504 (Stand 16. Oktober 2026)` |
| `contextra-bench` | `benchmarks/contextra-bench/src/long_mem_eval.rs:143` | `check_regression` | `-` | `pub fn check_regression( current: &RegressionReport, baseline_path: &Path, ) -> std::result::Result<(), String>` | `7fddc8504 (Stand 16. Oktober 2026)` |
| `contextra-bench` | `benchmarks/contextra-bench/src/long_mem_eval.rs:191` | `pad_vector` | `-` | `fn pad_vector(v: &[f32], target_dim: usize) -> Vec<f32>` | `7fddc8504 (Stand 16. Oktober 2026)` |
| `contextra-bench` | `benchmarks/contextra-bench/src/long_mem_eval.rs:201` | `RegressionSuite::baseline` | `impl RegressionSuite` | `pub fn baseline() -> Self` | `7fddc8504 (Stand 16. Oktober 2026)` |
| `contextra-bench` | `benchmarks/contextra-bench/src/long_mem_eval.rs:992` | `RegressionSuite::run_against_collection` | `impl RegressionSuite` | `pub async fn run_against_collection<S: StorageEngine, V: VectorIndex>( &self, collection: &Collection<S, V>, ) -> Result<RegressionReport>` | `7fddc8504 (Stand 16. Oktober 2026)` |
| `contextra-bench` | `benchmarks/contextra-bench/src/long_mem_eval.rs:1125` | `load_from_jsonl` | `-` | `pub fn load_from_jsonl(path: &Path) -> Result<Vec<LongMemEvalCase>>` | `7fddc8504 (Stand 16. Oktober 2026)` |
| `contextra-bench` | `benchmarks/contextra-bench/src/long_mem_eval.rs:1195` | `parse_question_type` | `-` | `fn parse_question_type(qtype_str: &str, qid: &str) -> LongMemEvalQuestionType` | `7fddc8504 (Stand 16. Oktober 2026)` |
| `contextra-bench` | `benchmarks/contextra-bench/src/long_mem_eval.rs:1215` | `json_val_to_string` | `-` | `fn json_val_to_string(val: Option<serde_json::Value>) -> Option<String>` | `7fddc8504 (Stand 16. Oktober 2026)` |
| `contextra-bench` | `benchmarks/contextra-bench/src/long_mem_eval.rs:1236` | `parse_raw_item` | `-` | `fn parse_raw_item(item: RawLongMemEvalItem) -> Option<LongMemEvalCase>` | `7fddc8504 (Stand 16. Oktober 2026)` |
| `contextra-bench` | `benchmarks/contextra-bench/src/long_mem_eval.rs:1304` | `run_long_mem_eval` | `-` | `pub async fn run_long_mem_eval<'a, F>( cases: &[LongMemEvalCase], search_fn: F, ) -> Result<LongMemEvalReport> where F: Fn(&str) -> BoxFuture<'a, Result<Vec<ScoredChunk>>>,` | `7fddc8504 (Stand 16. Oktober 2026)` |
| `contextra-avv-generator` | `crates/contextra-avv-generator/src/template.rs:152` | `format_technical_measure` | `-` | `fn format_technical_measure( out: &mut String, measure: &TechnicalMeasure, ) -> Result<(), AvvGeneratorError>` | `7fddc8504 (Stand 16. Oktober 2026)` |
| `contextra-cognition` | `crates/contextra-cognition/src/aggregation_phase.rs:178` | `SuperEdgeSink::tombstone_edge` | `trait SuperEdgeSink` | `fn tombstone_edge(&mut self, id: HyperEdgeId) -> Result<()>` | `7fddc8504 (Stand 16. Oktober 2026)` |
| `contextra-cognition` | `crates/contextra-cognition/src/graph_sink.rs:38` | `CsrGraphSuperEdgeSink::tombstone_edge` | `impl SuperEdgeSink for CsrGraphSuperEdgeSink` | `fn tombstone_edge(&mut self, id: HyperEdgeId) -> Result<()>` | `7fddc8504 (Stand 16. Oktober 2026)` |
| `contextra-cognition` | `crates/contextra-cognition/src/maintenance_scheduler.rs:291` | `write_tick_intent` | `-` | `async fn write_tick_intent<S: StorageEngine, V: VectorIndex>( collection: &Collection<S, V>, ) -> Result<TxId, ContextraError>` | `7fddc8504 (Stand 16. Oktober 2026)` |
| `contextra-cognition` | `crates/contextra-cognition/src/maintenance_scheduler.rs:315` | `complete_tick_intent` | `-` | `async fn complete_tick_intent<S: StorageEngine, V: VectorIndex>( collection: &Collection<S, V>, ) -> Result<(), ContextraError>` | `7fddc8504 (Stand 16. Oktober 2026)` |
| `contextra-infer-onnx` | `crates/contextra-infer-onnx/src/lib.rs:123` | `download_onnx_file` | `-` | `async fn download_onnx_file( client: &reqwest::Client, url: &str, dest_path: &Path, file_name: &str, target_dir: &Path, ) -> Result<()>` | `7fddc8504 (Stand 16. Oktober 2026)` |
| `contextra-infer-onnx` | `crates/contextra-infer-onnx/src/lib.rs:195` | `create_candle_embedder` | `-` | `pub fn create_candle_embedder( client: CandleEmbedClient, ) -> Box<dyn contextra_ports::EmbeddingProvider>` | `7fddc8504 (Stand 16. Oktober 2026)` |
| `contextra-ports` | `crates/contextra-ports/src/checkpoint.rs:56` | `Snapshot::seq_no` | `trait Snapshot` | `fn seq_no(&self) -> u64` | `7fddc8504 (Stand 16. Oktober 2026)` |
| `contextra-ports` | `crates/contextra-ports/src/embedding.rs:189` | `MockEmbedder::with_fixed_output` | `impl MockEmbedder` | `pub fn with_fixed_output(fixed_output: Vec<f32>) -> Self` | `7fddc8504 (Stand 16. Oktober 2026)` |
| `contextra-ports` | `crates/contextra-ports/src/metrics.rs:83` | `TestMetricsSink::events` | `impl TestMetricsSink` | `pub fn events(&self) -> Vec<MetricEvent>` | `7fddc8504 (Stand 16. Oktober 2026)` |
| `contextra-ports` | `crates/contextra-ports/src/metrics.rs:91` | `TestMetricsSink::clear` | `impl TestMetricsSink` | `pub fn clear(&self)` | `7fddc8504 (Stand 16. Oktober 2026)` |
| `contextra-store` | `crates/contextra-store/src/util.rs:60` | `fsync_parent_dir` | `-` | `pub(crate) async fn fsync_parent_dir(path: &Path) -> Result<()>` | `7fddc8504 (Stand 16. Oktober 2026)` |
| `contextra-store` | `crates/contextra-store/src/wal/mod.rs:251` | `KeyManager::try_new` | `impl KeyManager` | `pub fn try_new(_passphrase: &str, _salt: &[u8]) -> Result<Self>` | `7fddc8504 (Stand 16. Oktober 2026)` |
| `contextra-store` | `crates/contextra-store/src/wal/mod.rs:254` | `KeyManager::derive_file_key` | `impl KeyManager` | `pub fn derive_file_key(&self, _file_id: &[u8]) -> Result<Self>` | `7fddc8504 (Stand 16. Oktober 2026)` |
| `contextra-store` | `crates/contextra-store/src/wal/mod.rs:257` | `KeyManager::encrypt_auto_nonce` | `impl KeyManager` | `pub fn encrypt_auto_nonce(&self, _plaintext: &[u8]) -> Result<(Vec<u8>, [u8; 12])>` | `7fddc8504 (Stand 16. Oktober 2026)` |
| `contextra-store` | `crates/contextra-store/src/wal/mod.rs:260` | `KeyManager::decrypt_auto_nonce` | `impl KeyManager` | `pub fn decrypt_auto_nonce(&self, _ciphertext: &[u8], _nonce: &[u8; 12]) -> Result<Vec<u8>>` | `7fddc8504 (Stand 16. Oktober 2026)` |
| `contextra-store` | `crates/contextra-store/src/wal/mod.rs:263` | `KeyManager::integrity_key` | `impl KeyManager` | `pub fn integrity_key(&self) -> Result<[u8; 32]>` | `7fddc8504 (Stand 16. Oktober 2026)` |

### Engine: Transaktionen & Background Maintenance

| Crate | Historische Datei & Zeile | Funktion / Methode | Kontext | Signatur | Cherry-Pick-Quelle |
|---|---|---|---|---|---|
| `contextra-engine` | `crates/contextra-engine/src/chunker.rs:63` | `estimate_tokens` | `-` | `pub fn estimate_tokens(text: &str) -> usize` | `7fddc8504 / 8412f0862` |
| `contextra-engine` | `crates/contextra-engine/src/chunker.rs:142` | `ChunkerConfig::default` | `impl Default for ChunkerConfig` | `fn default() -> Self` | `7fddc8504 / 8412f0862` |
| `contextra-engine` | `crates/contextra-engine/src/chunker.rs:183` | `MarkdownBlock::to_text` | `impl MarkdownBlock` | `fn to_text(&self) -> String` | `7fddc8504 / 8412f0862` |
| `contextra-engine` | `crates/contextra-engine/src/chunker.rs:207` | `MarkdownBlock::estimate_tokens` | `impl MarkdownBlock` | `fn estimate_tokens(&self) -> usize` | `7fddc8504 / 8412f0862` |
| `contextra-engine` | `crates/contextra-engine/src/chunker.rs:212` | `parse_markdown_blocks` | `-` | `fn parse_markdown_blocks(lines: &[String]) -> Vec<MarkdownBlock>` | `7fddc8504 / 8412f0862` |
| `contextra-engine` | `crates/contextra-engine/src/chunker.rs:278` | `is_table_row` | `-` | `fn is_table_row(line: &str) -> bool` | `7fddc8504 / 8412f0862` |
| `contextra-engine` | `crates/contextra-engine/src/chunker.rs:286` | `is_table_delimiter_row` | `-` | `fn is_table_delimiter_row(line: &str) -> bool` | `7fddc8504 / 8412f0862` |
| `contextra-engine` | `crates/contextra-engine/src/chunker.rs:291` | `split_raw_section` | `-` | `fn split_raw_section( sec: RawSection, hard_limit: usize, window_chars: usize, overlap_chars: usize, ) -> Vec<RawSection>` | `7fddc8504 / 8412f0862` |
| `contextra-engine` | `crates/contextra-engine/src/chunker.rs:483` | `MarkdownChunker::new` | `impl MarkdownChunker` | `pub fn new(config: ChunkerConfig) -> Self` | `7fddc8504 / 8412f0862` |
| `contextra-engine` | `crates/contextra-engine/src/chunker.rs:488` | `MarkdownChunker::with_defaults` | `impl MarkdownChunker` | `pub fn with_defaults() -> Self` | `7fddc8504 / 8412f0862` |
| `contextra-engine` | `crates/contextra-engine/src/chunker.rs:493` | `MarkdownChunker::chunk` | `impl MarkdownChunker` | `pub fn chunk(&self, doc_id: DocId, markdown: &str) -> Vec<ContextChunk>` | `7fddc8504 / 8412f0862` |
| `contextra-engine` | `crates/contextra-engine/src/chunker.rs:624` | `parts_first` | `-` | `fn parts_first(_line: &str, h_level: u8) -> String` | `7fddc8504 / 8412f0862` |
| `contextra-engine` | `crates/contextra-engine/src/chunker.rs:632` | `chunk_text_with_overlap` | `-` | `pub fn chunk_text_with_overlap(text: &str, window_chars: usize, overlap_chars: usize) -> Vec<&str>` | `7fddc8504 / 8412f0862` |
| `contextra-engine` | `crates/contextra-engine/src/chunker.rs:667` | `chunk_text` | `-` | `pub fn chunk_text(text: &str, chunk_size: usize) -> Vec<&str>` | `7fddc8504 / 8412f0862` |
| `contextra-engine` | `crates/contextra-engine/src/collection/maintenance.rs:139` | `Collection::reap_by_thermostat` | `impl Collection` | `pub async fn reap_by_thermostat( &self, decay_controller: &AdaptiveDecayController, max_per_tick: usize, ) -> Result<usize>` | `7fddc8504 / 8412f0862` |
| `contextra-engine` | `crates/contextra-engine/src/collection/maintenance.rs:655` | `Collection::trigger_reaper` | `impl Collection` | `pub async fn trigger_reaper(&self) -> Result<usize>` | `7fddc8504 / 8412f0862` |
| `contextra-engine` | `crates/contextra-engine/src/collection/mod.rs:482` | `Collection::with_kv_store` | `impl Collection` | `pub fn with_kv_store(mut self, kv_store: Arc<contextra_crypto::TenantIsolatedKvStore>) -> Self` | `7fddc8504 / 8412f0862` |
| `contextra-engine` | `crates/contextra-engine/src/collection/mod.rs:489` | `Collection::set_kv_store` | `impl Collection` | `pub fn set_kv_store(&mut self, kv_store: Arc<contextra_crypto::TenantIsolatedKvStore>)` | `7fddc8504 / 8412f0862` |
| `contextra-engine` | `crates/contextra-engine/src/collection/mod.rs:494` | `Collection::set_kv_store` | `impl Collection` | `pub fn set_kv_store(&mut self, _kv_store: Arc<contextra_crypto::TenantIsolatedKvStore>) {}` | `7fddc8504 / 8412f0862` |
| `contextra-engine` | `crates/contextra-engine/src/collection/mod.rs:497` | `Collection::with_kv_store` | `impl Collection` | `pub fn with_kv_store(self, _kv_store: Arc<contextra_crypto::TenantIsolatedKvStore>) -> Self` | `7fddc8504 / 8412f0862` |
| `contextra-engine` | `crates/contextra-engine/src/collection/mod.rs:503` | `Collection::kv_store` | `impl Collection` | `pub fn kv_store(&self) -> Option<&Arc<contextra_crypto::TenantIsolatedKvStore>>` | `7fddc8504 / 8412f0862` |
| `contextra-engine` | `crates/contextra-engine/src/collection/mod.rs:509` | `Collection::kv_store` | `impl Collection` | `pub fn kv_store(&self) -> Option<&Arc<crate::TenantIsolatedKvStore>>` | `7fddc8504 / 8412f0862` |
| `contextra-engine` | `crates/contextra-engine/src/contextra_impl/wiring.rs:26` | `Contextra::set_pid_controller` | `impl Contextra` | `pub fn set_pid_controller( &self, pid_controller: std::sync::Weak<parking_lot::Mutex<contextra_adapt::PidController>>, )` | `7fddc8504 / 8412f0862` |
| `contextra-engine` | `crates/contextra-engine/src/kv_cache_integration/mod.rs:26` | `AccessCounterAttentionExporter::new` | `impl AccessCounterAttentionExporter` | `pub fn new() -> Self` | `7fddc8504 / 8412f0862` |
| `contextra-engine` | `crates/contextra-engine/src/kv_cache_integration/mod.rs:31` | `AccessCounterAttentionExporter::record_access` | `impl AccessCounterAttentionExporter` | `pub fn record_access(&self, request_id: RequestId)` | `7fddc8504 / 8412f0862` |
| `contextra-engine` | `crates/contextra-engine/src/kv_cache_integration/mod.rs:37` | `AccessCounterAttentionExporter::get_access_count` | `impl AccessCounterAttentionExporter` | `pub fn get_access_count(&self, request_id: RequestId) -> u64` | `7fddc8504 / 8412f0862` |
| `contextra-engine` | `crates/contextra-engine/src/kv_cache_integration/mod.rs:47` | `AccessCounterAttentionExporter::export_attention_weights` | `impl AttentionExporter for AccessCounterAttentionExporter` | `fn export_attention_weights(&self, request_id: RequestId) -> Option<Vec<f32>>` | `7fddc8504 / 8412f0862` |
| `contextra-engine` | `crates/contextra-engine/src/kv_cache_integration/mod.rs:57` | `build_eviction_worker` | `-` | `pub fn build_eviction_worker(store: Arc<TenantIsolatedKvStore>) -> EvictionWorker` | `7fddc8504 / 8412f0862` |
| `contextra-engine` | `crates/contextra-engine/src/kv_cache_integration/mod.rs:65` | `build_eviction_worker_with_exporter` | `-` | `pub fn build_eviction_worker_with_exporter( store: Arc<TenantIsolatedKvStore>, exporter: Arc<dyn AttentionExporter>, ) -> EvictionWorker` | `7fddc8504 / 8412f0862` |
| `contextra-engine` | `crates/contextra-engine/src/kv_cache_integration/mod.rs:76` | `start_kv_cache_eviction` | `-` | `pub fn start_kv_cache_eviction(store: Arc<TenantIsolatedKvStore>) -> EvictionWorker` | `7fddc8504 / 8412f0862` |
| `contextra-engine` | `crates/contextra-engine/src/lib.rs:50` | `LayerCleanupProof::new_after_verified_empty` | `impl LayerCleanupProof` | `pub fn new_after_verified_empty( layer: DeletionLayer, remaining_count: usize, ) -> contextra_types::Result<Self>` | `7fddc8504 / 8412f0862` |
| `contextra-engine` | `crates/contextra-engine/src/lib.rs:60` | `LayerCleanupProof::verify_and_create` | `impl LayerCleanupProof` | `pub fn verify_and_create<F>( layer: DeletionLayer, verifier: F, ) -> contextra_types::Result<Self> where F: FnOnce() -> contextra_types::Result<bool>,` | `7fddc8504 / 8412f0862` |
| `contextra-engine` | `crates/contextra-engine/src/lib.rs:103` | `default_signature_version` | `-` | `const fn default_signature_version() -> u8` | `7fddc8504 / 8412f0862` |
| `contextra-engine` | `crates/contextra-engine/src/lib.rs:107` | `default_tx_id` | `-` | `const fn default_tx_id() -> TxId` | `7fddc8504 / 8412f0862` |
| `contextra-engine` | `crates/contextra-engine/src/lib.rs:115` | `TenantIsolatedKvStore::new` | `impl TenantIsolatedKvStore` | `pub fn new() -> Self` | `7fddc8504 / 8412f0862` |
| `contextra-engine` | `crates/contextra-engine/src/lib.rs:119` | `TenantIsolatedKvStore::purge_tenant_segments` | `impl TenantIsolatedKvStore` | `pub fn purge_tenant_segments(&self, _tenant: TenantId) {}` | `7fddc8504 / 8412f0862` |
| `contextra-engine` | `crates/contextra-engine/src/lib.rs:121` | `TenantIsolatedKvStore::on_rollback` | `impl TenantIsolatedKvStore` | `pub fn on_rollback(&self, _tenant: TenantId, _chunk_ids: &[u64]) {}` | `7fddc8504 / 8412f0862` |
| `contextra-engine` | `crates/contextra-engine/src/lib.rs:123` | `TenantIsolatedKvStore::remove_tenant_segment` | `impl TenantIsolatedKvStore` | `pub fn remove_tenant_segment(&self, _tenant: TenantId, _doc_id: contextra_types::DocId) {}` | `7fddc8504 / 8412f0862` |
| `contextra-engine` | `crates/contextra-engine/src/lib.rs:127` | `DeletionProof::create` | `impl DeletionProof` | `pub fn create( scope: DeletionScope, mut deleted_keys: Vec<Vec<u8>>, tx_id: TxId, layer_proofs: Vec<LayerCleanupProof>, _cas_proofs: Vec<Vec<u8>>, proof_key: &[u8], ) -> contextra_types::Result<Self>` | `7fddc8504 / 8412f0862` |
| `contextra-engine` | `crates/contextra-engine/src/lib.rs:182` | `DeletionProof::tenant_id` | `impl DeletionProof` | `pub fn tenant_id(&self) -> TenantId` | `7fddc8504 / 8412f0862` |
| `contextra-engine` | `crates/contextra-engine/src/lib.rs:190` | `DeletionProof::verify` | `impl DeletionProof` | `pub fn verify(&self, _key: &[u8]) -> contextra_types::Result<bool>` | `7fddc8504 / 8412f0862` |
| `contextra-engine` | `crates/contextra-engine/src/lib.rs:194` | `DeletionProof::export_for_audit` | `impl DeletionProof` | `pub fn export_for_audit(&self) -> contextra_types::Result<String>` | `7fddc8504 / 8412f0862` |
| `contextra-engine` | `crates/contextra-engine/src/lib.rs:574` | `Contextra::kv_eviction_attention_source` | `impl Contextra` | `pub fn kv_eviction_attention_source( &self, ) -> Option<Arc<dyn contextra_kvcache::AttentionScoreSource>>` | `7fddc8504 / 8412f0862` |
| `contextra-engine` | `crates/contextra-engine/src/transaction/db_transaction_rollback.rs:10` | `DbTransaction::trigger_kv_store_rollback` | `impl DbTransaction` | `pub(super) fn trigger_kv_store_rollback(&self, doc_ids: &[DocId]) -> Result<()>` | `7fddc8504 / 8412f0862` |

### Graph & Hyperedges (Wissensgraph, PPR, Lovasz, Cascade)

| Crate | Historische Datei & Zeile | Funktion / Methode | Kontext | Signatur | Cherry-Pick-Quelle |
|---|---|---|---|---|---|
| `contextra-graph` | `crates/contextra-graph/src/csr/graph_index.rs:19` | `CsrGraph::remove_entity` | `impl GraphIndexExt for CsrGraph` | `fn remove_entity<'a>(&'a self, tx: TxId, entity: EntityId) -> BoxFuture<'a, Result<()>>` | `05a6a9af (archiv/v0) / 7fddc8504` |
| `contextra-graph` | `crates/contextra-graph/src/csr/graph_index.rs:125` | `CsrGraph::default` | `impl Default for CsrGraph` | `fn default() -> Self` | `05a6a9af (archiv/v0) / 7fddc8504` |
| `contextra-graph` | `crates/contextra-graph/src/csr/graph_index.rs:131` | `CsrGraph::add_entity` | `impl GraphIndex for CsrGraph` | `fn add_entity<'a>(&'a self, tx: TxId, entity: Entity) -> BoxFuture<'a, Result<()>>` | `05a6a9af (archiv/v0) / 7fddc8504` |
| `contextra-graph` | `crates/contextra-graph/src/csr/graph_index.rs:164` | `CsrGraph::add_edge` | `impl GraphIndex for CsrGraph` | `fn add_edge<'a>(&'a self, tx: TxId, edge: contextra_types::Edge) -> BoxFuture<'a, Result<()>>` | `05a6a9af (archiv/v0) / 7fddc8504` |
| `contextra-graph` | `crates/contextra-graph/src/csr/graph_index.rs:256` | `CsrGraph::personalized_page_rank` | `impl GraphIndex for CsrGraph` | `fn personalized_page_rank<'a>( &'a self, seed_nodes: &'a [EntityId], config: &'a contextra_types::PprConfig, ) -> BoxFuture<'a, Result<Vec<(EntityId, f32)>>>` | `05a6a9af (archiv/v0) / 7fddc8504` |
| `contextra-graph` | `crates/contextra-graph/src/csr/graph_index.rs:281` | `CsrGraph::personalized_page_rank_at` | `impl GraphIndex for CsrGraph` | `fn personalized_page_rank_at<'a>( &'a self, seed_nodes: &'a [EntityId], config: &'a contextra_types::PprConfig, seq_no: u64, ) -> BoxFuture<'a, Result<Vec<(EntityId, f32)>>>` | `05a6a9af (archiv/v0) / 7fddc8504` |
| `contextra-graph` | `crates/contextra-graph/src/csr/graph_index.rs:318` | `CsrGraph::path_rag_at` | `impl GraphIndex for CsrGraph` | `fn path_rag_at<'a>( &'a self, start_nodes: &'a [EntityId], max_hops: usize, seq_no: u64, ) -> BoxFuture<'a, Result<Vec<(EntityId, f32)>>>` | `05a6a9af (archiv/v0) / 7fddc8504` |
| `contextra-graph` | `crates/contextra-graph/src/csr/graph_index.rs:327` | `CsrGraph::traverse_at` | `impl GraphIndex for CsrGraph` | `fn traverse_at<'a>( &'a self, start_node: EntityId, max_hops: usize, seq_no: u64, ) -> BoxFuture<'a, Result<Vec<(EntityId, f32)>>>` | `05a6a9af (archiv/v0) / 7fddc8504` |
| `contextra-graph` | `crates/contextra-graph/src/csr/graph_index.rs:339` | `CsrGraph::traverse_at_time` | `impl GraphIndex for CsrGraph` | `fn traverse_at_time<'a>( &'a self, start: EntityId, max_hops: usize, as_of: TxId, ) -> BoxFuture<'a, Result<Vec<(EntityId, f32)>>>` | `05a6a9af (archiv/v0) / 7fddc8504` |
| `contextra-graph` | `crates/contextra-graph/src/csr/graph_index.rs:351` | `CsrGraph::traverse_at_bitemporal` | `impl GraphIndex for CsrGraph` | `fn traverse_at_bitemporal<'a>( &'a self, start: EntityId, max_hops: usize, as_of_tx: TxId, as_of_business: Option<i64>, ) -> BoxFuture<'a, Result<Vec<(EntityId, f32)>>>` | `05a6a9af (archiv/v0) / 7fddc8504` |
| `contextra-graph` | `crates/contextra-graph/src/csr/graph_index.rs:508` | `CsrGraph::traverse` | `impl GraphIndex for CsrGraph` | `fn traverse<'a>( &'a self, start: EntityId, max_hops: usize, ) -> BoxFuture<'a, Result<Vec<(EntityId, f32)>>>` | `05a6a9af (archiv/v0) / 7fddc8504` |
| `contextra-graph` | `crates/contextra-graph/src/csr/graph_index.rs:648` | `CsrGraph::commit` | `impl GraphIndex for CsrGraph` | `fn commit<'a>(&'a self, tx: TxId) -> BoxFuture<'a, Result<()>>` | `05a6a9af (archiv/v0) / 7fddc8504` |
| `contextra-graph` | `crates/contextra-graph/src/csr/graph_index.rs:765` | `CsrGraph::remove_edge` | `impl GraphIndex for CsrGraph` | `fn remove_edge<'a>( &'a self, tx: TxId, from: EntityId, to: EntityId, ) -> BoxFuture<'a, Result<()>>` | `05a6a9af (archiv/v0) / 7fddc8504` |
| `contextra-graph` | `crates/contextra-graph/src/csr/graph_index.rs:801` | `CsrGraph::add_bidirectional` | `impl GraphIndex for CsrGraph` | `fn add_bidirectional<'a>( &'a self, tx: TxId, from: EntityId, to: EntityId, label: &'a str, ) -> BoxFuture<'a, Result<()>>` | `05a6a9af (archiv/v0) / 7fddc8504` |
| `contextra-graph` | `crates/contextra-graph/src/csr/graph_index.rs:817` | `CsrGraph::neighbors` | `impl GraphIndex for CsrGraph` | `fn neighbors<'a>(&'a self, start: EntityId) -> BoxFuture<'a, Result<Vec<EntityId>>>` | `05a6a9af (archiv/v0) / 7fddc8504` |
| `contextra-graph` | `crates/contextra-graph/src/csr/graph_index.rs:821` | `CsrGraph::rollback` | `impl GraphIndex for CsrGraph` | `fn rollback<'a>(&'a self, tx: TxId) -> BoxFuture<'a, Result<()>>` | `05a6a9af (archiv/v0) / 7fddc8504` |
| `contextra-graph` | `crates/contextra-graph/src/csr/graph_index.rs:831` | `CsrGraph::rollback_to_tx` | `impl GraphIndex for CsrGraph` | `fn rollback_to_tx<'a>(&'a self, _tx_id: TxId) -> BoxFuture<'a, Result<()>>` | `05a6a9af (archiv/v0) / 7fddc8504` |
| `contextra-graph` | `crates/contextra-graph/src/csr/graph_index.rs:839` | `CsrGraph::last_tx_id` | `impl GraphIndex for CsrGraph` | `fn last_tx_id<'a>(&'a self) -> BoxFuture<'a, Result<TxId>>` | `05a6a9af (archiv/v0) / 7fddc8504` |
| `contextra-graph` | `crates/contextra-graph/src/csr/graph_index.rs:843` | `CsrGraph::len` | `impl GraphIndex for CsrGraph` | `fn len<'a>(&'a self) -> BoxFuture<'a, usize>` | `05a6a9af (archiv/v0) / 7fddc8504` |
| `contextra-graph` | `crates/contextra-graph/src/csr/graph_index.rs:847` | `CsrGraph::stats` | `impl GraphIndex for CsrGraph` | `fn stats<'a>(&'a self) -> BoxFuture<'a, Result<GraphIndexStats>>` | `05a6a9af (archiv/v0) / 7fddc8504` |
| `contextra-graph` | `crates/contextra-graph/src/csr/graph_read.rs:12` | `CsrGraph::compact` | `impl CsrGraph` | `pub fn compact(&self)` | `05a6a9af (archiv/v0) / 7fddc8504` |
| `contextra-graph` | `crates/contextra-graph/src/csr/graph_read.rs:37` | `CsrGraph::compact_async` | `impl CsrGraph` | `pub async fn compact_async(&self) -> Result<()>` | `05a6a9af (archiv/v0) / 7fddc8504` |
| `contextra-graph` | `crates/contextra-graph/src/csr/graph_read.rs:79` | `CsrGraph::set_communities_batch` | `impl CsrGraph` | `pub fn set_communities_batch(&self, assignments: &[crate::CommunityAssignment])` | `05a6a9af (archiv/v0) / 7fddc8504` |
| `contextra-graph` | `crates/contextra-graph/src/csr/graph_read.rs:88` | `CsrGraph::get_communities_batch` | `impl CsrGraph` | `pub async fn get_communities_batch( &self, entity_ids: &[EntityId], ) -> Result<HashMap<EntityId, u64>>` | `05a6a9af (archiv/v0) / 7fddc8504` |
| `contextra-graph` | `crates/contextra-graph/src/csr/graph_read.rs:135` | `CsrGraph::neighbors` | `impl CsrGraph` | `pub async fn neighbors(&self, start: EntityId) -> Result<Vec<EntityId>>` | `05a6a9af (archiv/v0) / 7fddc8504` |
| `contextra-graph` | `crates/contextra-graph/src/csr/graph_read.rs:186` | `CsrGraph::pagerank` | `impl CsrGraph` | `pub async fn pagerank( &self, damping_factor: f32, max_iterations: usize, tolerance: f32, ) -> HashMap<EntityId, f32>` | `05a6a9af (archiv/v0) / 7fddc8504` |
| `contextra-graph` | `crates/contextra-graph/src/csr/graph_read.rs:268` | `CsrGraph::get_deleted_node_indices` | `impl CsrGraph` | `pub async fn get_deleted_node_indices(&self) -> HashSet<usize>` | `05a6a9af (archiv/v0) / 7fddc8504` |
| `contextra-graph` | `crates/contextra-graph/src/csr/graph_read.rs:286` | `CsrGraph::deleted_view` | `impl CsrGraph` | `pub async fn deleted_view(&self) -> crate::DeletedView` | `05a6a9af (archiv/v0) / 7fddc8504` |
| `contextra-graph` | `crates/contextra-graph/src/csr/graph_read.rs:291` | `CsrGraph::personalized_page_rank_with_context_async` | `impl CsrGraph` | `pub async fn personalized_page_rank_with_context_async( &self, seed_nodes: &[EntityId], config: &contextra_types::PprConfig, ctx: &mut crate::PprContext, ) -> Vec<(EntityId, f32)>` | `05a6a9af (archiv/v0) / 7fddc8504` |
| `contextra-graph` | `crates/contextra-graph/src/csr/graph_read.rs:312` | `CsrGraph::entity_count` | `impl CsrGraph` | `pub fn entity_count(&self) -> usize` | `05a6a9af (archiv/v0) / 7fddc8504` |
| `contextra-graph` | `crates/contextra-graph/src/csr/graph_read.rs:321` | `CsrGraph::entity_exists` | `impl CsrGraph` | `pub fn entity_exists(&self, id: EntityId) -> bool` | `05a6a9af (archiv/v0) / 7fddc8504` |
| `contextra-graph` | `crates/contextra-graph/src/csr/graph_read.rs:332` | `CsrGraph::is_entity_deleted` | `impl CsrGraph` | `pub async fn is_entity_deleted(&self, entity: EntityId) -> bool` | `05a6a9af (archiv/v0) / 7fddc8504` |
| `contextra-graph` | `crates/contextra-graph/src/csr/graph_read.rs:341` | `CsrGraph::edge_count` | `impl CsrGraph` | `pub fn edge_count(&self) -> usize` | `05a6a9af (archiv/v0) / 7fddc8504` |
| `contextra-graph` | `crates/contextra-graph/src/csr/graph_read.rs:349` | `CsrGraph::remove_entity` | `impl CsrGraph` | `pub async fn remove_entity(&self, tx: TxId, entity: EntityId) -> Result<()>` | `05a6a9af (archiv/v0) / 7fddc8504` |
| `contextra-graph` | `crates/contextra-graph/src/csr/graph_read.rs:357` | `CsrGraph::path_rag_at` | `impl CsrGraph` | `pub fn path_rag_at<'a>( &'a self, start_nodes: &'a [EntityId], max_hops: usize, seq_no: u64, ) -> contextra_ports::BoxFuture<'a, Result<Vec<(EntityId, f32)>>>` | `05a6a9af (archiv/v0) / 7fddc8504` |
| `contextra-graph` | `crates/contextra-graph/src/csr/graph_write.rs:36` | `CsrGraph::new` | `impl CsrGraph` | `pub fn new() -> Self` | `05a6a9af (archiv/v0) / 7fddc8504` |
| `contextra-graph` | `crates/contextra-graph/src/csr/graph_write.rs:41` | `CsrGraph::with_config` | `impl CsrGraph` | `pub fn with_config(config: CsrGraphConfig) -> Self` | `05a6a9af (archiv/v0) / 7fddc8504` |
| `contextra-graph` | `crates/contextra-graph/src/csr/graph_write.rs:56` | `CsrGraph::with_storage` | `impl CsrGraph` | `pub fn with_storage(storage: Arc<dyn StorageEngine>) -> Self` | `05a6a9af (archiv/v0) / 7fddc8504` |
| `contextra-graph` | `crates/contextra-graph/src/csr/graph_write.rs:61` | `CsrGraph::with_config_and_storage` | `impl CsrGraph` | `pub fn with_config_and_storage( config: CsrGraphConfig, storage: Arc<dyn StorageEngine>, ) -> Self` | `05a6a9af (archiv/v0) / 7fddc8504` |
| `contextra-graph` | `crates/contextra-graph/src/csr/graph_write.rs:79` | `CsrGraph::with_consistency_enforcer` | `impl CsrGraph` | `pub fn with_consistency_enforcer(suppression_threshold: u32) -> Self` | `05a6a9af (archiv/v0) / 7fddc8504` |
| `contextra-graph` | `crates/contextra-graph/src/csr/graph_write.rs:96` | `CsrGraph::tombstone_edge` | `impl CsrGraph` | `pub async fn tombstone_edge( &self, edge_id: crate::consistency_enforcement::EdgeId, tx: TxId, ) -> Result<()>` | `05a6a9af (archiv/v0) / 7fddc8504` |
| `contextra-graph` | `crates/contextra-graph/src/csr/graph_write.rs:114` | `CsrGraph::inner_read` | `impl CsrGraph` | `pub fn inner_read(&self) -> arc_swap::Guard<Arc<GraphInner>>` | `05a6a9af (archiv/v0) / 7fddc8504` |
| `contextra-graph` | `crates/contextra-graph/src/csr/graph_write.rs:118` | `CsrGraph::inner_write` | `impl CsrGraph` | `pub(crate) fn inner_write(&self) -> InnerWriteGuard<'_>` | `05a6a9af (archiv/v0) / 7fddc8504` |
| `contextra-graph` | `crates/contextra-graph/src/csr/graph_write.rs:126` | `CsrGraph::set_storage` | `impl CsrGraph` | `pub fn set_storage(&mut self, storage: Arc<dyn StorageEngine>)` | `05a6a9af (archiv/v0) / 7fddc8504` |
| `contextra-graph` | `crates/contextra-graph/src/csr/graph_write.rs:131` | `CsrGraph::storage` | `impl CsrGraph` | `pub fn storage(&self) -> Option<Arc<dyn StorageEngine>>` | `05a6a9af (archiv/v0) / 7fddc8504` |
| `contextra-graph` | `crates/contextra-graph/src/csr/graph_write.rs:136` | `CsrGraph::enqueue_cascade_deferred` | `impl CsrGraph` | `pub fn enqueue_cascade_deferred( &self, doc_id: DocId, hyperedge_ids: &[crate::hyperedge::HyperEdgeId], )` | `05a6a9af (archiv/v0) / 7fddc8504` |
| `contextra-graph` | `crates/contextra-graph/src/csr/graph_write.rs:148` | `CsrGraph::pending_cascade_queue_len` | `impl CsrGraph` | `pub fn pending_cascade_queue_len(&self) -> usize` | `05a6a9af (archiv/v0) / 7fddc8504` |
| `contextra-graph` | `crates/contextra-graph/src/csr/graph_write.rs:156` | `CsrGraph::process_cascade_queue` | `impl CsrGraph` | `pub async fn process_cascade_queue(&self, batch_size: usize, wal_tx: TxId) -> Result<usize>` | `05a6a9af (archiv/v0) / 7fddc8504` |
| `contextra-graph` | `crates/contextra-graph/src/csr/graph_write.rs:178` | `CsrGraph::dfs_upward` | `impl CsrGraph` | `fn dfs_upward( graph: &CsrGraph, node: crate::hyperedge::HyperEdgeId, visited: &mut HashSet<crate::hyperedge::HyperEdgeId>, on_stack: &mut HashSet<crate::hyperedge::HyperEdgeId>, closure_nodes: &mut Vec<crate::hyperedge::HyperEdgeId>, )` | `05a6a9af (archiv/v0) / 7fddc8504` |
| `contextra-graph` | `crates/contextra-graph/src/csr/graph_write.rs:232` | `CsrGraph::estimate_memory` | `impl CsrGraph` | `pub fn estimate_memory(&self, presized: bool) -> MemoryEstimate` | `05a6a9af (archiv/v0) / 7fddc8504` |
| `contextra-graph` | `crates/contextra-graph/src/csr/graph_write.rs:238` | `CsrGraph::estimate_memory_bytes` | `impl CsrGraph` | `pub fn estimate_memory_bytes(&self) -> usize` | `05a6a9af (archiv/v0) / 7fddc8504` |
| `contextra-graph` | `crates/contextra-graph/src/csr/graph_write.rs:245` | `CsrGraph::estimate_compaction_peak_bytes` | `impl CsrGraph` | `pub fn estimate_compaction_peak_bytes(&self) -> usize` | `05a6a9af (archiv/v0) / 7fddc8504` |
| `contextra-graph` | `crates/contextra-graph/src/csr/graph_write.rs:251` | `CsrGraph::get_source_doc_id` | `impl CsrGraph` | `pub fn get_source_doc_id(&self, index: usize) -> Option<DocId>` | `05a6a9af (archiv/v0) / 7fddc8504` |
| `contextra-graph` | `crates/contextra-graph/src/csr/graph_write.rs:257` | `CsrGraph::source_doc_id_at` | `impl CsrGraph` | `pub fn source_doc_id_at(&self, from: EntityId, to: EntityId) -> Option<DocId>` | `05a6a9af (archiv/v0) / 7fddc8504` |
| `contextra-graph` | `crates/contextra-graph/src/csr/graph_write.rs:295` | `CsrGraph::tombstone_edges_direct` | `impl CsrGraph` | `pub(crate) fn tombstone_edges_direct( &self, edges: &[(EntityId, EntityId)], wal_tx: TxId, ) -> Result<(usize, Vec<(EntityId, EntityId)>, Vec<EntityId>)>` | `05a6a9af (archiv/v0) / 7fddc8504` |
| `contextra-graph` | `crates/contextra-graph/src/csr/graph_write.rs:349` | `CsrGraph::relate_n_ary` | `impl CsrGraph` | `pub fn relate_n_ary( &self, id: crate::hyperedge::HyperEdgeId, predicate: crate::csr::EdgeType, participants: Vec<crate::hyperedge::RoleBinding>, weight: f32, doc_id: Option<DocId>, ) -> std::result::Result<crate::hyperedge::HyperEdge, GraphMutationError>` | `05a6a9af (archiv/v0) / 7fddc8504` |
| `contextra-graph` | `crates/contextra-graph/src/csr/graph_write.rs:372` | `CsrGraph::persist_hyperedge` | `impl CsrGraph` | `pub async fn persist_hyperedge( &self, tx: TxId, hyperedge: &crate::hyperedge::HyperEdge, ) -> Result<()>` | `05a6a9af (archiv/v0) / 7fddc8504` |
| `contextra-graph` | `crates/contextra-graph/src/csr/graph_write.rs:399` | `CsrGraph::insert_hyperedge_direct` | `impl CsrGraph` | `pub fn insert_hyperedge_direct(&self, hyperedge: crate::hyperedge::HyperEdge)` | `05a6a9af (archiv/v0) / 7fddc8504` |
| `contextra-graph` | `crates/contextra-graph/src/csr/graph_write.rs:430` | `CsrGraph::parent_hyperedges_of` | `impl CsrGraph` | `pub fn parent_hyperedges_of( &self, child: crate::hyperedge::HyperEdgeId, ) -> Vec<crate::hyperedge::HyperEdgeId>` | `05a6a9af (archiv/v0) / 7fddc8504` |
| `contextra-graph` | `crates/contextra-graph/src/csr/graph_write.rs:455` | `CsrGraph::hyperedges_for_doc` | `impl CsrGraph` | `pub fn hyperedges_for_doc(&self, doc_id: DocId) -> Vec<crate::hyperedge::HyperEdgeId>` | `05a6a9af (archiv/v0) / 7fddc8504` |
| `contextra-graph` | `crates/contextra-graph/src/csr/graph_write.rs:475` | `CsrGraph::max_hyperedge_id` | `impl CsrGraph` | `pub fn max_hyperedge_id(&self) -> u64` | `05a6a9af (archiv/v0) / 7fddc8504` |
| `contextra-graph` | `crates/contextra-graph/src/csr/graph_write.rs:486` | `CsrGraph::get_hyperedge` | `impl CsrGraph` | `pub fn get_hyperedge( &self, id: crate::hyperedge::HyperEdgeId, ) -> Option<Arc<crate::hyperedge::HyperEdge>>` | `05a6a9af (archiv/v0) / 7fddc8504` |
| `contextra-graph` | `crates/contextra-graph/src/csr/graph_write.rs:499` | `CsrGraph::hyperedges_for_entity` | `impl CsrGraph` | `pub fn hyperedges_for_entity(&self, entity_id: EntityId) -> Vec<crate::hyperedge::HyperEdgeId>` | `05a6a9af (archiv/v0) / 7fddc8504` |
| `contextra-graph` | `crates/contextra-graph/src/csr/graph_write.rs:526` | `CsrGraph::tombstone_hyperedge` | `impl CsrGraph` | `pub fn tombstone_hyperedge(&self, id: crate::hyperedge::HyperEdgeId, wal_tx: TxId) -> bool` | `05a6a9af (archiv/v0) / 7fddc8504` |
| `contextra-graph` | `crates/contextra-graph/src/csr/graph_write.rs:583` | `CsrGraph::commit_super_edge_batch` | `impl CsrGraph` | `pub fn commit_super_edge_batch( &self, tombstone_ids: &[crate::hyperedge::HyperEdgeId], new_edges: Vec<crate::hyperedge::HyperEdge>, wal_tx: TxId, ) -> std::result::Result<(), GraphMutationError>` | `05a6a9af (archiv/v0) / 7fddc8504` |
| `contextra-graph` | `crates/contextra-graph/src/csr/graph_write.rs:679` | `CsrGraph::edges_for_doc` | `impl CsrGraph` | `pub fn edges_for_doc(&self, doc_id: DocId) -> Vec<(EntityId, EntityId)>` | `05a6a9af (archiv/v0) / 7fddc8504` |
| `contextra-graph` | `crates/contextra-graph/src/csr/graph_write.rs:690` | `CsrGraph::insert_hyperedge` | `impl CsrGraph` | `pub fn insert_hyperedge(&self, edge: crate::hyperedge::HyperEdge)` | `05a6a9af (archiv/v0) / 7fddc8504` |
| `contextra-graph` | `crates/contextra-graph/src/csr/graph_write.rs:695` | `CsrGraph::insert_entity_direct` | `impl CsrGraph` | `pub fn insert_entity_direct(&self, entity: Entity) -> Result<()>` | `05a6a9af (archiv/v0) / 7fddc8504` |
| `contextra-graph` | `crates/contextra-graph/src/csr/graph_write.rs:707` | `CsrGraph::add_edge` | `impl CsrGraph` | `pub async fn add_edge( self: &Arc<Self>, from: EntityId, to: EntityId, weight: f32, tx_valid_from: Option<TxId>, tx_valid_to: Option<TxId>, business_valid_from: Option<i64>, business_valid_to: Option<i64>, source_doc_id: Option<DocId>, predicate_hash: Option<[u8; 32]>,` | `05a6a9af (archiv/v0) / 7fddc8504` |
| `contextra-graph` | `crates/contextra-graph/src/csr/graph_write.rs:799` | `CsrGraph::insert_edge_direct` | `impl CsrGraph` | `pub async fn insert_edge_direct( self: &Arc<Self>, from: EntityId, to: EntityId, weight: f32, ) -> Result<()>` | `05a6a9af (archiv/v0) / 7fddc8504` |
| `contextra-graph` | `crates/contextra-graph/src/csr/graph_write.rs:815` | `CsrGraph::insert_edge_direct_with_validity` | `impl CsrGraph` | `pub async fn insert_edge_direct_with_validity( self: &Arc<Self>, from: EntityId, to: EntityId, weight: f32, tx_valid_from: Option<TxId>, tx_valid_to: Option<TxId>, ) -> Result<()>` | `05a6a9af (archiv/v0) / 7fddc8504` |
| `contextra-graph` | `crates/contextra-graph/src/csr/graph_write.rs:840` | `CsrGraph::insert_edge_direct_with_bitemporal_validity` | `impl CsrGraph` | `pub async fn insert_edge_direct_with_bitemporal_validity( self: &Arc<Self>, from: EntityId, to: EntityId, weight: f32, tx_valid_from: Option<TxId>, tx_valid_to: Option<TxId>, business_valid_from: Option<i64>, business_valid_to: Option<i64>, source_doc_id: Option<DocId>, ) -> Result<()>` | `05a6a9af (archiv/v0) / 7fddc8504` |
| `contextra-graph` | `crates/contextra-graph/src/csr/graph_write.rs:867` | `CsrGraph::load_entity_direct` | `impl CsrGraph` | `pub fn load_entity_direct(&self, entity: Entity) -> Result<()>` | `05a6a9af (archiv/v0) / 7fddc8504` |
| `contextra-graph` | `crates/contextra-graph/src/csr/graph_write.rs:880` | `CsrGraph::load_edge_direct` | `impl CsrGraph` | `pub fn load_edge_direct( &self, from: EntityId, to: EntityId, weight: f32, tx_valid_from: Option<TxId>, tx_valid_to: Option<TxId>, business_valid_from: Option<i64>, business_valid_to: Option<i64>, source_doc_id: Option<DocId>, ) -> Result<()>` | `05a6a9af (archiv/v0) / 7fddc8504` |
| `contextra-graph` | `crates/contextra-graph/src/csr/graph_write.rs:927` | `CsrGraph::relate_n_ary` | `impl GraphCollectionMutation for CsrGraph` | `fn relate_n_ary( &self, predicate_tag: u32, participants: &[contextra_ports::graph::RoleBinding], doc_id: DocId, ) -> std::result::Result< contextra_ports::graph::HyperEdgeId, contextra_ports::graph::GraphMutationError, >` | `05a6a9af (archiv/v0) / 7fddc8504` |
| `contextra-graph` | `crates/contextra-graph/src/csr/inner.rs:91` | `MemoryEstimate::total_bytes` | `impl MemoryEstimate` | `pub fn total_bytes(&self) -> usize` | `05a6a9af (archiv/v0) / 7fddc8504` |
| `contextra-graph` | `crates/contextra-graph/src/csr/inner.rs:97` | `table_bytes` | `-` | `fn table_bytes<K, V>(capacity: usize) -> usize` | `05a6a9af (archiv/v0) / 7fddc8504` |
| `contextra-graph` | `crates/contextra-graph/src/csr/inner.rs:102` | `vec_bytes` | `-` | `fn vec_bytes<T>(capacity: usize) -> usize` | `05a6a9af (archiv/v0) / 7fddc8504` |
| `contextra-graph` | `crates/contextra-graph/src/csr/inner.rs:107` | `sentinel_entity` | `-` | `pub(crate) fn sentinel_entity() -> Entity` | `05a6a9af (archiv/v0) / 7fddc8504` |
| `contextra-graph` | `crates/contextra-graph/src/csr/inner.rs:112` | `GraphInner::default` | `impl Default for GraphInner` | `fn default() -> Self` | `05a6a9af (archiv/v0) / 7fddc8504` |
| `contextra-graph` | `crates/contextra-graph/src/csr/inner.rs:118` | `GraphInner::new` | `impl GraphInner` | `pub fn new() -> Self` | `05a6a9af (archiv/v0) / 7fddc8504` |
| `contextra-graph` | `crates/contextra-graph/src/csr/inner.rs:152` | `GraphInner::entity_at` | `impl GraphInner` | `pub(crate) fn entity_at(&self, idx: usize) -> Option<&Entity>` | `05a6a9af (archiv/v0) / 7fddc8504` |
| `contextra-graph` | `crates/contextra-graph/src/csr/inner.rs:157` | `GraphInner::tx_valid_from_at` | `impl GraphInner` | `pub(crate) fn tx_valid_from_at(&self, idx: usize) -> Option<TxId>` | `05a6a9af (archiv/v0) / 7fddc8504` |
| `contextra-graph` | `crates/contextra-graph/src/csr/inner.rs:165` | `GraphInner::tx_valid_to_at` | `impl GraphInner` | `pub(crate) fn tx_valid_to_at(&self, idx: usize) -> Option<TxId>` | `05a6a9af (archiv/v0) / 7fddc8504` |
| `contextra-graph` | `crates/contextra-graph/src/csr/inner.rs:173` | `GraphInner::business_valid_from_at` | `impl GraphInner` | `pub(crate) fn business_valid_from_at(&self, idx: usize) -> Option<i64>` | `05a6a9af (archiv/v0) / 7fddc8504` |
| `contextra-graph` | `crates/contextra-graph/src/csr/inner.rs:181` | `GraphInner::business_valid_to_at` | `impl GraphInner` | `pub(crate) fn business_valid_to_at(&self, idx: usize) -> Option<i64>` | `05a6a9af (archiv/v0) / 7fddc8504` |
| `contextra-graph` | `crates/contextra-graph/src/csr/inner.rs:189` | `GraphInner::source_doc_id_at` | `impl GraphInner` | `pub(crate) fn source_doc_id_at(&self, idx: usize) -> Option<DocId>` | `05a6a9af (archiv/v0) / 7fddc8504` |
| `contextra-graph` | `crates/contextra-graph/src/csr/inner.rs:196` | `GraphInner::estimate_memory` | `impl GraphInner` | `pub(crate) fn estimate_memory(&self, presized: bool) -> MemoryEstimate` | `05a6a9af (archiv/v0) / 7fddc8504` |
| `contextra-graph` | `crates/contextra-graph/src/csr/inner.rs:403` | `GraphInner::estimate_memory_bytes` | `impl GraphInner` | `pub(crate) fn estimate_memory_bytes(&self) -> usize` | `05a6a9af (archiv/v0) / 7fddc8504` |
| `contextra-graph` | `crates/contextra-graph/src/csr/inner.rs:407` | `GraphInner::estimate_compaction_peak_bytes` | `impl GraphInner` | `pub(crate) fn estimate_compaction_peak_bytes(&self) -> usize` | `05a6a9af (archiv/v0) / 7fddc8504` |
| `contextra-graph` | `crates/contextra-graph/src/csr/inner.rs:417` | `GraphInner::hyperedges_for_entity` | `impl GraphInner` | `pub(crate) fn hyperedges_for_entity(&self, id: EntityId) -> Vec<crate::hyperedge::HyperEdgeId>` | `05a6a9af (archiv/v0) / 7fddc8504` |
| `contextra-graph` | `crates/contextra-graph/src/csr/inner.rs:428` | `GraphInner::insert_hyperedge` | `impl GraphInner` | `pub(crate) fn insert_hyperedge(&mut self, edge: crate::hyperedge::HyperEdge)` | `05a6a9af (archiv/v0) / 7fddc8504` |
| `contextra-graph` | `crates/contextra-graph/src/csr/inner.rs:445` | `GraphInner::get_or_create_index` | `impl GraphInner` | `pub fn get_or_create_index(&mut self, id: EntityId) -> InternalIndex` | `05a6a9af (archiv/v0) / 7fddc8504` |
| `contextra-graph` | `crates/contextra-graph/src/csr/inner.rs:461` | `GraphInner::add_to_out_weight_sum` | `impl GraphInner` | `pub(crate) fn add_to_out_weight_sum(&mut self, node_idx: usize, weight: f32)` | `05a6a9af (archiv/v0) / 7fddc8504` |
| `contextra-graph` | `crates/contextra-graph/src/csr/inner.rs:473` | `GraphInner::recompute_node_out_weight_sum` | `impl GraphInner` | `pub(crate) fn recompute_node_out_weight_sum(&mut self, node_idx: usize)` | `05a6a9af (archiv/v0) / 7fddc8504` |
| `contextra-graph` | `crates/contextra-graph/src/csr/inner.rs:519` | `GraphInner::compact` | `impl GraphInner` | `pub fn compact(&mut self)` | `05a6a9af (archiv/v0) / 7fddc8504` |
| `contextra-graph` | `crates/contextra-graph/src/csr/inner.rs:651` | `GraphInner::find_edge_mut_by_entities` | `impl GraphInner` | `pub(crate) fn find_edge_mut_by_entities( &mut self, from: EntityId, to: EntityId, ) -> Option<&mut Edge>` | `05a6a9af (archiv/v0) / 7fddc8504` |
| `contextra-graph` | `crates/contextra-graph/src/csr/inner.rs:708` | `GraphInner::all_entity_ids` | `impl GraphInner` | `pub(crate) fn all_entity_ids(&self) -> Vec<EntityId>` | `05a6a9af (archiv/v0) / 7fddc8504` |
| `contextra-graph` | `crates/contextra-graph/src/csr/inner.rs:712` | `GraphInner::outgoing_edges_mut` | `impl GraphInner` | `pub(crate) fn outgoing_edges_mut(&mut self, entity_id: EntityId) -> &mut [Edge]` | `05a6a9af (archiv/v0) / 7fddc8504` |
| `contextra-graph` | `crates/contextra-graph/src/csr/inner.rs:747` | `GraphInner::sync_edge_reinforcement_weights` | `impl GraphInner` | `pub(crate) fn sync_edge_reinforcement_weights( &mut self, config: &crate::edge_reinforcement::EdgeReinforcementConfig, )` | `05a6a9af (archiv/v0) / 7fddc8504` |
| `contextra-graph` | `crates/contextra-graph/src/csr/inner.rs:799` | `InnerWriteGuard::deref` | `impl Deref for InnerWriteGuard` | `fn deref(&self) -> &Self::Target` | `05a6a9af (archiv/v0) / 7fddc8504` |
| `contextra-graph` | `crates/contextra-graph/src/csr/inner.rs:805` | `InnerWriteGuard::deref_mut` | `impl DerefMut for InnerWriteGuard` | `fn deref_mut(&mut self) -> &mut Self::Target` | `05a6a9af (archiv/v0) / 7fddc8504` |
| `contextra-graph` | `crates/contextra-graph/src/csr/inner.rs:811` | `InnerWriteGuard::drop` | `impl Drop for InnerWriteGuard` | `fn drop(&mut self)` | `05a6a9af (archiv/v0) / 7fddc8504` |
| `contextra-graph` | `crates/contextra-graph/src/error.rs:55` | `ContextraError::from` | `impl From for ContextraError` | `fn from(err: GraphMutationError) -> Self` | `05a6a9af (archiv/v0) / 7fddc8504` |
| `contextra-graph` | `crates/contextra-graph/src/hyperedge.rs:51` | `HyperEdgeId::inner` | `impl HyperEdgeId` | `pub const fn inner(self) -> u64` | `05a6a9af (archiv/v0) / 7fddc8504` |
| `contextra-graph` | `crates/contextra-graph/src/hyperedge.rs:57` | `HyperEdgeId::from` | `impl From for HyperEdgeId` | `fn from(id: u64) -> Self` | `05a6a9af (archiv/v0) / 7fddc8504` |
| `contextra-graph` | `crates/contextra-graph/src/hyperedge.rs:63` | `HyperEdgeId::fmt` | `impl Display for HyperEdgeId` | `fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result` | `05a6a9af (archiv/v0) / 7fddc8504` |
| `contextra-graph` | `crates/contextra-graph/src/hyperedge.rs:84` | `RoleId::inner` | `impl RoleId` | `pub const fn inner(self) -> u32` | `05a6a9af (archiv/v0) / 7fddc8504` |
| `contextra-graph` | `crates/contextra-graph/src/hyperedge.rs:90` | `RoleId::from` | `impl From for RoleId` | `fn from(id: u32) -> Self` | `05a6a9af (archiv/v0) / 7fddc8504` |
| `contextra-graph` | `crates/contextra-graph/src/hyperedge.rs:96` | `RoleId::fmt` | `impl Display for RoleId` | `fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result` | `05a6a9af (archiv/v0) / 7fddc8504` |
| `contextra-graph` | `crates/contextra-graph/src/ppr.rs:35` | `DeletedView::from_nodes` | `impl DeletedView` | `pub(crate) fn from_nodes(deleted_nodes: HashSet<usize>) -> Self` | `05a6a9af (archiv/v0) / 7fddc8504` |
| `contextra-graph` | `crates/contextra-graph/src/ppr.rs:49` | `DeletedView::contains` | `impl DeletedView` | `pub fn contains(&self, idx: usize) -> bool` | `05a6a9af (archiv/v0) / 7fddc8504` |
| `contextra-graph` | `crates/contextra-graph/src/ppr.rs:55` | `DeletedView::len` | `impl DeletedView` | `pub fn len(&self) -> usize` | `05a6a9af (archiv/v0) / 7fddc8504` |
| `contextra-graph` | `crates/contextra-graph/src/ppr.rs:61` | `DeletedView::is_empty` | `impl DeletedView` | `pub fn is_empty(&self) -> bool` | `05a6a9af (archiv/v0) / 7fddc8504` |
| `contextra-graph` | `crates/contextra-graph/src/ppr.rs:80` | `PprContext::new` | `impl PprContext` | `pub fn new() -> Self` | `05a6a9af (archiv/v0) / 7fddc8504` |
| `contextra-graph` | `crates/contextra-graph/src/ppr.rs:85` | `PprContext::prepare` | `impl PprContext` | `fn prepare(&mut self, n: usize)` | `05a6a9af (archiv/v0) / 7fddc8504` |
| `contextra-graph` | `crates/contextra-graph/src/ppr.rs:100` | `compute_ppr` | `-` | `pub(crate) fn compute_ppr( inner: &GraphInner, seed_nodes: &[EntityId], config: &PprConfig, deleted_nodes: &DeletedView, ) -> Vec<(EntityId, f32)>` | `05a6a9af (archiv/v0) / 7fddc8504` |
| `contextra-graph` | `crates/contextra-graph/src/ppr.rs:112` | `compute_ppr_with_apprh_selector` | `-` | `pub fn compute_ppr_with_apprh_selector( inner: &GraphInner, seed_nodes: &[EntityId], config: &PprConfig, deleted_nodes: &DeletedView, ctx: &mut PprContext, selector: &crate::apprh::ApprhSelector, apprh_params: &crate::apprh::ApprhParams, ) -> Vec<(EntityId, f32)>` | `05a6a9af (archiv/v0) / 7fddc8504` |
| `contextra-graph` | `crates/contextra-graph/src/ppr.rs:158` | `select_auto_ppr_algorithm` | `-` | `pub(crate) fn select_auto_ppr_algorithm( inner: &GraphInner, seed_count: usize, config: &PprConfig, ) -> PprAlgorithm` | `05a6a9af (archiv/v0) / 7fddc8504` |
| `contextra-graph` | `crates/contextra-graph/src/ppr.rs:175` | `compute_ppr_with_context` | `-` | `pub(crate) fn compute_ppr_with_context( inner: &GraphInner, seed_nodes: &[EntityId], config: &PprConfig, deleted_nodes: &DeletedView, ctx: &mut PprContext, ) -> Vec<(EntityId, f32)>` | `05a6a9af (archiv/v0) / 7fddc8504` |
| `contextra-graph` | `crates/contextra-graph/src/ppr.rs:334` | `GraphInner::neighbors_with_weights` | `impl PathGraph for GraphInner` | `fn neighbors_with_weights(&self, node: EntityId) -> Vec<(EntityId, f32)>` | `05a6a9af (archiv/v0) / 7fddc8504` |
| `contextra-graph` | `crates/contextra-graph/src/ppr.rs:381` | `GraphInner::predecessors_with_weights` | `impl PathGraph for GraphInner` | `fn predecessors_with_weights(&self, node: EntityId) -> Vec<(EntityId, f32)>` | `05a6a9af (archiv/v0) / 7fddc8504` |
| `contextra-graph` | `crates/contextra-graph/src/ppr.rs:431` | `GraphInner::hyperedges_for_entity` | `impl PathGraph for GraphInner` | `fn hyperedges_for_entity(&self, node: EntityId) -> Vec<crate::hyperedge::HyperEdgeId>` | `05a6a9af (archiv/v0) / 7fddc8504` |
| `contextra-graph` | `crates/contextra-graph/src/ppr.rs:438` | `GraphInner::get_hyperedge` | `impl PathGraph for GraphInner` | `fn get_hyperedge( &self, id: crate::hyperedge::HyperEdgeId, ) -> Option<std::sync::Arc<crate::hyperedge::HyperEdge>>` | `05a6a9af (archiv/v0) / 7fddc8504` |
| `contextra-graph` | `crates/contextra-graph/src/ppr.rs:449` | `forward_push_ppr` | `-` | `pub(crate) fn forward_push_ppr( inner: &GraphInner, seed_nodes: &[EntityId], config: &PprConfig, deleted_nodes: &DeletedView, ctx: &mut PprContext, ) -> Vec<(EntityId, f32)>` | `05a6a9af (archiv/v0) / 7fddc8504` |
| `contextra-graph` | `crates/contextra-graph/src/ppr.rs:766` | `compute_ppr_dense` | `-` | `pub(crate) fn compute_ppr_dense( inner: &GraphInner, seed_nodes: &[EntityId], config: &PprConfig, deleted_nodes: &DeletedView, ctx: &mut PprContext, ) -> Vec<(EntityId, f32)>` | `05a6a9af (archiv/v0) / 7fddc8504` |

### Kryptographie & DeletionProof (Löschnachweise, RevocationLog)

| Crate | Historische Datei & Zeile | Funktion / Methode | Kontext | Signatur | Cherry-Pick-Quelle |
|---|---|---|---|---|---|
| `contextra-crypto` | `crates/contextra-crypto/src/deletion_proof.rs:52` | `hash_deleted_keys_length_prefixed` | `-` | `pub fn hash_deleted_keys_length_prefixed(deleted_keys: &[Vec<u8>]) -> [u8; 32]` | `a43391fa5 / 8412f0862` |
| `contextra-crypto` | `crates/contextra-crypto/src/deletion_proof.rs:70` | `DeletionProofKeyPair::generate` | `impl DeletionProofKeyPair` | `pub fn generate() -> Self` | `a43391fa5 / 8412f0862` |
| `contextra-crypto` | `crates/contextra-crypto/src/deletion_proof.rs:81` | `DeletionProofKeyPair::verifying_key_bytes` | `impl DeletionProofKeyPair` | `pub fn verifying_key_bytes(&self) -> [u8; 32]` | `a43391fa5 / 8412f0862` |
| `contextra-crypto` | `crates/contextra-crypto/src/deletion_proof.rs:86` | `DeletionProofKeyPair::signing_key` | `impl DeletionProofKeyPair` | `pub fn signing_key(&self) -> &ed25519_dalek::SigningKey` | `a43391fa5 / 8412f0862` |
| `contextra-crypto` | `crates/contextra-crypto/src/deletion_proof.rs:101` | `VerificationKey::from` | `impl From for VerificationKey` | `fn from(key: &'a [u8]) -> Self` | `a43391fa5 / 8412f0862` |
| `contextra-crypto` | `crates/contextra-crypto/src/deletion_proof.rs:107` | `VerificationKey::from` | `impl From for VerificationKey` | `fn from(key: &'a [u8; N]) -> Self` | `a43391fa5 / 8412f0862` |
| `contextra-crypto` | `crates/contextra-crypto/src/deletion_proof.rs:113` | `From::from` | `impl From` | `fn from(key: &'a Vec<u8>) -> Self` | `a43391fa5 / 8412f0862` |
| `contextra-crypto` | `crates/contextra-crypto/src/deletion_proof.rs:119` | `VerificationKey::from` | `impl From for VerificationKey` | `fn from(key: &'a ed25519_dalek::VerifyingKey) -> Self` | `a43391fa5 / 8412f0862` |
| `contextra-crypto` | `crates/contextra-crypto/src/deletion_proof.rs:144` | `LayerCleanupProof::new_after_verified_empty` | `impl LayerCleanupProof` | `pub fn new_after_verified_empty( layer: DeletionLayer, remaining_live_entries: usize, ) -> Result<Self>` | `a43391fa5 / 8412f0862` |
| `contextra-crypto` | `crates/contextra-crypto/src/deletion_proof.rs:171` | `LayerCleanupProof::verify_and_create` | `impl LayerCleanupProof` | `pub fn verify_and_create<F>(layer: DeletionLayer, verification: F) -> Result<Self> where F: FnOnce() -> Result<bool>,` | `a43391fa5 / 8412f0862` |
| `contextra-crypto` | `crates/contextra-crypto/src/deletion_proof.rs:190` | `LayerCleanupProof::layer` | `impl LayerCleanupProof` | `pub fn layer(&self) -> &DeletionLayer` | `a43391fa5 / 8412f0862` |
| `contextra-crypto` | `crates/contextra-crypto/src/deletion_proof.rs:302` | `default_signature_version` | `-` | `const fn default_signature_version() -> u8` | `a43391fa5 / 8412f0862` |
| `contextra-crypto` | `crates/contextra-crypto/src/deletion_proof.rs:313` | `DeletionProof::create` | `impl DeletionProof` | `pub fn create( scope: DeletionScope, deleted_keys: Vec<Vec<u8>>, deleted_after_tx: TxId, covered_layers: Vec<LayerCleanupProof>, excluded_scopes: Vec<ExcludedScope>, proof_key: &[u8], ) -> Result<Self>` | `a43391fa5 / 8412f0862` |
| `contextra-crypto` | `crates/contextra-crypto/src/deletion_proof.rs:333` | `DeletionProof::create_with_wal_receipt` | `impl DeletionProof` | `pub fn create_with_wal_receipt( scope: DeletionScope, mut deleted_keys: Vec<Vec<u8>>, deleted_after_tx: TxId, covered_layers: Vec<LayerCleanupProof>, excluded_scopes: Vec<ExcludedScope>, wal_chain_receipt: Option<[u8; 32]>,` | `a43391fa5 / 8412f0862` |
| `contextra-crypto` | `crates/contextra-crypto/src/deletion_proof.rs:396` | `DeletionProof::create_v3` | `impl DeletionProof` | `pub fn create_v3( scope: DeletionScope, deleted_keys: Vec<Vec<u8>>, deleted_after_tx: TxId, covered_layers: Vec<LayerCleanupProof>, excluded_scopes: Vec<ExcludedScope>, attested_at: i64, graph_repair: &[GraphRepairAttestation], signing_key: &ed25519_dalek::SigningKey, ) -> Result<Self>` | `a43391fa5 / 8412f0862` |
| `contextra-crypto` | `crates/contextra-crypto/src/deletion_proof.rs:421` | `DeletionProof::create_v3_with_audit_position` | `impl DeletionProof` | `pub fn create_v3_with_audit_position( scope: DeletionScope, deleted_keys: Vec<Vec<u8>>, deleted_after_tx: TxId, covered_layers: Vec<LayerCleanupProof>, excluded_scopes: Vec<ExcludedScope>, audit_chain_position: Option<u64>, attested_at: i64, graph_repair: &[GraphRepairAttestation], signing_key: &ed25519_dalek::SigningKey, ) -> Result<Self>` | `a43391fa5 / 8412f0862` |
| `contextra-crypto` | `crates/contextra-crypto/src/deletion_proof.rs:448` | `DeletionProof::create_with_wal_receipt_v3` | `impl DeletionProof` | `pub fn create_with_wal_receipt_v3( scope: DeletionScope, deleted_keys: Vec<Vec<u8>>, deleted_after_tx: TxId, covered_layers: Vec<LayerCleanupProof>, excluded_scopes: Vec<ExcludedScope>, wal_chain_receipt: Option<[u8; 32]>,` | `a43391fa5 / 8412f0862` |
| `contextra-crypto` | `crates/contextra-crypto/src/deletion_proof.rs:475` | `DeletionProof::create_full_v3` | `impl DeletionProof` | `pub fn create_full_v3( scope: DeletionScope, mut deleted_keys: Vec<Vec<u8>>, deleted_after_tx: TxId, covered_layers: Vec<LayerCleanupProof>, excluded_scopes: Vec<ExcludedScope>, wal_chain_receipt: Option<[u8; 32]>,` | `a43391fa5 / 8412f0862` |
| `contextra-crypto` | `crates/contextra-crypto/src/deletion_proof.rs:539` | `DeletionProof::signature_version_typed` | `impl DeletionProof` | `pub fn signature_version_typed( &self, ) -> std::result::Result<SignatureVersion, DeletionProofError>` | `a43391fa5 / 8412f0862` |
| `contextra-crypto` | `crates/contextra-crypto/src/deletion_proof.rs:548` | `DeletionProof::verify` | `impl DeletionProof` | `pub fn verify<'a>(&self, key: impl Into<VerificationKey<'a>>) -> Result<bool>` | `a43391fa5 / 8412f0862` |
| `contextra-crypto` | `crates/contextra-crypto/src/deletion_proof.rs:633` | `DeletionProof::construct_v3_payload` | `impl DeletionProof` | `fn construct_v3_payload(&self) -> std::result::Result<Vec<u8>, CryptoError>` | `a43391fa5 / 8412f0862` |
| `contextra-crypto` | `crates/contextra-crypto/src/deletion_proof.rs:712` | `DeletionProof::verify_external` | `impl DeletionProof` | `pub fn verify_external( &self, public_key: &ed25519_dalek::VerifyingKey, ) -> std::result::Result<(), CryptoError>` | `a43391fa5 / 8412f0862` |
| `contextra-crypto` | `crates/contextra-crypto/src/deletion_proof.rs:739` | `DeletionProof::export_for_audit` | `impl DeletionProof` | `pub fn export_for_audit(&self) -> Result<String>` | `a43391fa5 / 8412f0862` |
| `contextra-crypto` | `crates/contextra-crypto/src/deletion_proof.rs:760` | `DeletionProof::tenant_id` | `impl DeletionProof` | `pub fn tenant_id(&self) -> TenantId` | `a43391fa5 / 8412f0862` |
| `contextra-crypto` | `crates/contextra-crypto/src/deletion_proof.rs:771` | `compute_wal_delete_receipt` | `-` | `pub fn compute_wal_delete_receipt( prev_hmac: &[u8; 32],` | `a43391fa5 / 8412f0862` |
| `contextra-crypto` | `crates/contextra-crypto/src/deletion_proof.rs:783` | `verify_wal_delete_receipt` | `-` | `pub fn verify_wal_delete_receipt( receipt: &[u8; 32],` | `a43391fa5 / 8412f0862` |
| `contextra-crypto` | `crates/contextra-crypto/src/deletion_proof.rs:794` | `compute_hmac_sha256` | `-` | `fn compute_hmac_sha256(key: &[u8], data_parts: &[&[u8]]) -> Result<[u8; 32]>` | `a43391fa5 / 8412f0862` |
| `contextra-db` | `crates/contextra-db/examples/verify_deletion_proof_external.rs:41` | `verify_external` | `-` | `fn verify_external(proof: &DeletionProof, verifying_key: &VerifyingKey) -> Result<(), CryptoError>` | `7fddc8504 (Stand 16. Oktober 2026)` |
| `contextra-db` | `crates/contextra-db/examples/verify_deletion_proof_external.rs:61` | `run_operator_role` | `-` | `async fn run_operator_role( proof_file_path: &Path, pubkey_file_path: &Path, ) -> contextra_types::Result<()>` | `7fddc8504 (Stand 16. Oktober 2026)` |
| `contextra-db` | `crates/contextra-db/examples/verify_deletion_proof_external.rs:162` | `run_external_verifier_role` | `-` | `fn run_external_verifier_role( proof_file_path: &Path, pubkey_file_path: &Path, label: &str, ) -> Result<(), CryptoError>` | `7fddc8504 (Stand 16. Oktober 2026)` |
| `contextra-db` | `crates/contextra-db/examples/verify_deletion_proof_external.rs:219` | `main` | `-` | `async fn main() -> contextra_types::Result<()>` | `7fddc8504 (Stand 16. Oktober 2026)` |

### MCP-Protokoll & Tools (Parking Tools, Egress Gateway)

| Crate | Historische Datei & Zeile | Funktion / Methode | Kontext | Signatur | Cherry-Pick-Quelle |
|---|---|---|---|---|---|
| `contextra-mcp` | `crates/contextra-mcp/src/egress_guard.rs:32` | `EgressGuard::new` | `impl EgressGuard` | `pub fn new(collection: Arc<Collection>, threshold: f32, min_bytes: usize) -> Self` | `7fddc8504 (Stand 16. Oktober 2026)` |
| `contextra-mcp` | `crates/contextra-mcp/src/egress_guard.rs:42` | `EgressGuard::with_timeout` | `impl EgressGuard` | `pub fn with_timeout(mut self, timeout: Duration) -> Self` | `7fddc8504 (Stand 16. Oktober 2026)` |
| `contextra-mcp` | `crates/contextra-mcp/src/egress_guard.rs:48` | `EgressGuard::threshold` | `impl EgressGuard` | `pub fn threshold(&self) -> f32` | `7fddc8504 (Stand 16. Oktober 2026)` |
| `contextra-mcp` | `crates/contextra-mcp/src/egress_guard.rs:53` | `EgressGuard::min_bytes` | `impl EgressGuard` | `pub fn min_bytes(&self) -> usize` | `7fddc8504 (Stand 16. Oktober 2026)` |
| `contextra-mcp` | `crates/contextra-mcp/src/egress_guard.rs:58` | `EgressGuard::timeout` | `impl EgressGuard` | `pub fn timeout(&self) -> Duration` | `7fddc8504 (Stand 16. Oktober 2026)` |
| `contextra-mcp` | `crates/contextra-mcp/src/egress_guard.rs:73` | `EgressGuard::check` | `impl EgressGuard` | `pub async fn check(&self, payload: &str) -> EgressClassification` | `7fddc8504 (Stand 16. Oktober 2026)` |
| `contextra-mcp` | `crates/contextra-mcp/src/plugin_status.rs:25` | `PluginStatusEntry::from` | `impl From for PluginStatusEntry` | `fn from(cap: contextra_ports::plugin::PluginCapability) -> Self` | `7fddc8504 (Stand 16. Oktober 2026)` |
| `contextra-mcp` | `crates/contextra-mcp/src/plugin_status.rs:36` | `handle_plugin_status` | `-` | `pub async fn handle_plugin_status( registry: &contextra_ports::plugin::PluginRegistry, ) -> Result<PluginStatusResponse, McpError>` | `7fddc8504 (Stand 16. Oktober 2026)` |
| `contextra-mcp` | `crates/contextra-mcp/src/proof_key.rs:46` | `resolve_deletion_proof_key` | `-` | `pub(crate) fn resolve_deletion_proof_key( primary: Option<&str>, alias: Option<&str>, ) -> Result<ResolvedProofKey, ProofKeyError>` | `7fddc8504 (Stand 16. Oktober 2026)` |
| `contextra-mcp` | `crates/contextra-mcp/src/proof_key.rs:86` | `deletion_proof_key_from_env` | `-` | `pub fn deletion_proof_key_from_env() -> Result<Zeroizing<String>, McpError>` | `7fddc8504 (Stand 16. Oktober 2026)` |
| `contextra-mcp` | `crates/contextra-mcp/src/routing.rs:130` | `setup_kv_bridge` | `-` | `pub fn setup_kv_bridge( _db: &Arc<Contextra>, ) -> Option<Arc<contextra_infer_candle::KvBridgeAdapter>>` | `7fddc8504 (Stand 16. Oktober 2026)` |
| `contextra-mcp` | `crates/contextra-mcp/src/server.rs:76` | `McpServer::with_kv_bridge` | `impl McpServer` | `pub fn with_kv_bridge( mut self, kv_bridge: Option<Arc<contextra_infer_candle::KvBridgeAdapter>>, ) -> Self` | `7fddc8504 (Stand 16. Oktober 2026)` |

### Privacy & Egress Schutz

| Crate | Historische Datei & Zeile | Funktion / Methode | Kontext | Signatur | Cherry-Pick-Quelle |
|---|---|---|---|---|---|
| `contextra-privacy` | `crates/contextra-privacy/src/avv_generator.rs:30` | `generate_avv_draft` | `-` | `pub fn generate_avv_draft(ctx: &AvvContext) -> String` | `7fddc8504 (Stand 16. Oktober 2026)` |
| `contextra-privacy` | `crates/contextra-privacy/src/bulk_exfiltration_detector.rs:151` | `BulkExfiltrationDetector::record_and_check_at` | `impl BulkExfiltrationDetector` | `fn record_and_check_at( &self, session: SessionId, payload_len: usize, now: Instant, ) -> BulkExfiltrationOutcome` | `7fddc8504 (Stand 16. Oktober 2026)` |

### Ranking & Kalibrierung (DiBud, Conformal Prediction)

| Crate | Historische Datei & Zeile | Funktion / Methode | Kontext | Signatur | Cherry-Pick-Quelle |
|---|---|---|---|---|---|
| `contextra-rank` | `crates/contextra-rank/src/explain.rs:25` | `SignalKind::name_de` | `impl SignalKind` | `pub fn name_de(&self) -> &'static str` | `c27326391 / 5d349c188` |

### WASM Sandbox & Isolation

| Crate | Historische Datei & Zeile | Funktion / Methode | Kontext | Signatur | Cherry-Pick-Quelle |
|---|---|---|---|---|---|
| `contextra-sandbox` | `crates/contextra-sandbox/src/executor.rs:32` | `CapabilityViolationError::fmt` | `impl Display for CapabilityViolationError` | `fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result` | `7fddc8504 (Stand 16. Oktober 2026)` |
| `contextra-sandbox` | `crates/contextra-sandbox/src/executor.rs:56` | `SandboxState::memory_growing` | `impl ResourceLimiter for SandboxState` | `fn memory_growing( &mut self, _current: usize, desired: usize, _max: Option<usize>, ) -> anyhow::Result<bool>` | `7fddc8504 (Stand 16. Oktober 2026)` |
| `contextra-sandbox` | `crates/contextra-sandbox/src/executor.rs:67` | `SandboxState::table_growing` | `impl ResourceLimiter for SandboxState` | `fn table_growing( &mut self, _current: u32, desired: u32, _max: Option<u32>, ) -> anyhow::Result<bool>` | `7fddc8504 (Stand 16. Oktober 2026)` |
| `contextra-sandbox` | `crates/contextra-sandbox/src/executor.rs:89` | `WasmExecutor::fmt` | `impl Debug for WasmExecutor` | `fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result` | `7fddc8504 (Stand 16. Oktober 2026)` |
| `contextra-sandbox` | `crates/contextra-sandbox/src/executor.rs:99` | `WasmExecutor::new` | `impl WasmExecutor` | `pub fn new() -> Result<Self, SandboxError>` | `7fddc8504 (Stand 16. Oktober 2026)` |
| `contextra-sandbox` | `crates/contextra-sandbox/src/executor.rs:112` | `WasmExecutor::validate_module` | `impl WasmExecutor` | `pub fn validate_module( &self, wasm_bytes: &[u8], max_module_size_bytes: usize, ) -> Result<(), SandboxError>` | `7fddc8504 (Stand 16. Oktober 2026)` |
| `contextra-sandbox` | `crates/contextra-sandbox/src/executor.rs:142` | `WasmExecutor::execute` | `impl WasmExecutor` | `pub async fn execute( &self, wasm_bytes: &[u8], input: &[u8], capabilities: &WasmCapabilities, timeout: Duration, ) -> Result<WasmOutput, SandboxError>` | `7fddc8504 (Stand 16. Oktober 2026)` |
| `contextra-sandbox` | `crates/contextra-sandbox/src/wasi.rs:21` | `ProcessExitError::fmt` | `impl Display for ProcessExitError` | `fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result` | `7fddc8504 (Stand 16. Oktober 2026)` |
| `contextra-sandbox` | `crates/contextra-sandbox/src/wasi.rs:36` | `OutputLimitExceededError::fmt` | `impl Display for OutputLimitExceededError` | `fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result` | `7fddc8504 (Stand 16. Oktober 2026)` |
| `contextra-sandbox` | `crates/contextra-sandbox/src/wasi.rs:55` | `register` | `-` | `pub(crate) fn register(linker: &mut Linker<SandboxState>) -> Result<(), anyhow::Error>` | `7fddc8504 (Stand 16. Oktober 2026)` |
