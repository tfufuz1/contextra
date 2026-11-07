# Dead Code & Orphan Symbols Analysis

> Generiert durch `tools/dead_code_pruner.py`.
> Untersucht Symbole, die innerhalb ihres Crates nur exakt 1x (bei Deklaration) vorkommen.

## Crate Übersicht

| Crate | Untersuchte Symbole | Potenzielle Dead-Code Kandidaten |
|---|---|---|
| `contextra` | 63 | **10** |
| `contextra-adapt` | 165 | **8** |
| `contextra-agent` | 175 | **8** |
| `contextra-audit-export` | 25 | **3** |
| `contextra-avv-generator` | 24 | **3** |
| `contextra-checkpoint` | 185 | **4** |
| `contextra-cognition` | 158 | **6** |
| `contextra-core` | 0 | **0** |
| `contextra-crypto` | 292 | **24** |
| `contextra-db` | 51 | **4** |
| `contextra-durable-fs` | 90 | **12** |
| `contextra-engine` | 436 | **22** |
| `contextra-erasure` | 653 | **66** |
| `contextra-graph` | 510 | **11** |
| `contextra-infer-candle` | 170 | **8** |
| `contextra-infer-ollama` | 110 | **3** |
| `contextra-infer-onnx` | 72 | **1** |
| `contextra-ingest` | 598 | **33** |
| `contextra-kvcache` | 216 | **10** |
| `contextra-license` | 43 | **15** |
| `contextra-mcp` | 137 | **11** |
| `contextra-mvcc` | 174 | **15** |
| `contextra-ports` | 447 | **33** |
| `contextra-privacy` | 167 | **10** |
| `contextra-py` | 72 | **1** |
| `contextra-rank` | 173 | **13** |
| `contextra-router` | 121 | **6** |
| `contextra-sandbox` | 44 | **2** |
| `contextra-simd` | 42 | **0** |
| `contextra-store` | 531 | **32** |
| `contextra-sys` | 44 | **0** |
| `contextra-testkit` | 98 | **9** |
| `contextra-text` | 233 | **7** |
| `contextra-types` | 260 | **43** |
| `contextra-vector` | 326 | **8** |
| `contextra-wire` | 168 | **1** |

## Dead Code Kandidaten (Auszug)

### `contextra`

- `/app/crates/contextra/src/query_rewriter.rs:64` [fn `with_max_subqueries`]: `pub fn with_max_subqueries(mut self, n: usize) -> Self {`
- `/app/crates/contextra/src/query_rewriter.rs:71` [fn `with_max_context_results`]: `pub fn with_max_context_results(mut self, n: usize) -> Self {`
- `/app/crates/contextra/src/query_rewriter.rs:78` [fn `with_max_snippet_chars`]: `pub fn with_max_snippet_chars(mut self, n: usize) -> Self {`
- `/app/crates/contextra/src/collection_profile.rs:123` [fn `validate_with_license`]: `pub fn validate_with_license(`
- `/app/crates/contextra/src/boot.rs:17` [enum `BootPhase`]: `pub enum BootPhase {`
- `/app/crates/contextra/src/builder.rs:227` [fn `with_embedding_backend`]: `pub fn with_embedding_backend(mut self, backend: EmbeddingBackend) -> Self {`
- `/app/crates/contextra/src/builder.rs:253` [fn `with_license_gate`]: `pub fn with_license_gate(mut self, gate: Arc<dyn LicenseGate>) -> Self {`
- `/app/crates/contextra/src/builder.rs:264` [fn `with_signed_license`]: `pub fn with_signed_license(`
- `/app/crates/contextra/src/builder.rs:294` [fn `with_performance_profile`]: `pub fn with_performance_profile(`
- `/app/crates/contextra/src/agent_memory.rs:175` [fn `new_arc`]: `pub fn new_arc(engine: Arc<Contextra>) -> Self {`
### `contextra-adapt`

- `/app/crates/contextra-adapt/src/flow_thompson.rs:198` [fn `ring3_background_task_token`]: `pub fn ring3_background_task_token() -> Ring3Token {`
- `/app/crates/contextra-adapt/src/flow_thompson.rs:335` [fn `update_with_flow`]: `pub fn update_with_flow(`
- `/app/crates/contextra-adapt/src/pid_latency_controller.rs:79` [fn `with_params`]: `pub fn with_params(kp: f64, ki: f64, kd: f64, target_latency_ms: f64) -> Self {`
- `/app/crates/contextra-adapt/src/bandit.rs:166` [fn `derive_seed`]: `pub fn derive_seed(tenant_id: &contextra_types::TenantId, base_seed: u64) -> u64 {`
- `/app/crates/contextra-adapt/src/bandit.rs:936` [fn `bench_sherman_morrison_p95_latency`]: `fn bench_sherman_morrison_p95_latency() {`
- `/app/crates/contextra-adapt/src/rie_greedy.rs:114` [fn `predict`]: `pub fn predict(&self, context: &[f32]) -> Result<f32, RieGreedyError> {`
- `/app/crates/contextra-adapt/src/lyapunov.rs:139` [fn `analyze`]: `pub fn analyze(&self) -> LyapunovResult {`
- `/app/crates/contextra-adapt/src/lyapunov.rs:151` [fn `status_str`]: `pub fn status_str(&self) -> &'static str {`
### `contextra-agent`

- `/app/crates/contextra-agent/src/engine.rs:108` [fn `try_register_tool_with_manifest`]: `pub fn try_register_tool_with_manifest(`
- `/app/crates/contextra-agent/src/event_source.rs:149` [fn `with_last_seen_seq`]: `pub fn with_last_seen_seq(mut self, last_seen_seq: u64) -> Self {`
- `/app/crates/contextra-agent/src/context.rs:167` [fn `set_cache_directive`]: `pub fn set_cache_directive(&mut self, directive: CacheDirective) {`
- `/app/crates/contextra-agent/src/context.rs:172` [fn `get_cache_directive`]: `pub fn get_cache_directive(&self) -> &CacheDirective {`
- `/app/crates/contextra-agent/src/context.rs:183` [fn `directive_for_node`]: `pub fn directive_for_node(&self, node_id: &str, step_id: StepId) -> CacheDirective {`
- `/app/crates/contextra-agent/src/graph.rs:241` [fn `try_add_edge_with_goal`]: `pub fn try_add_edge_with_goal(`
- `/app/crates/contextra-agent/src/tools/retrieve.rs:59` [fn `with_name`]: `pub fn with_name(name: impl Into<String>) -> Self {`
- `/app/crates/contextra-agent/src/tools/registry.rs:55` [fn `get_manifest`]: `pub fn get_manifest(&self, name: &str) -> Option<&ToolManifest> {`
### `contextra-audit-export`

- `/app/crates/contextra-audit-export/src/testkit.rs:34` [fn `with_sample_data_for`]: `pub fn with_sample_data_for(tenant_id: TenantId) -> Self {`
- `/app/crates/contextra-audit-export/src/testkit.rs:108` [fn `add_entry`]: `pub fn add_entry(&mut self, entry: ProcessingRegisterEntry) {`
- `/app/crates/contextra-audit-export/src/lib.rs:257` [fn `render_register_json`]: `pub fn render_register_json(`
### `contextra-avv-generator`

- `/app/crates/contextra-avv-generator/src/lib.rs:147` [fn `with_contract_date`]: `pub fn with_contract_date(mut self, contract_date: impl Into<String>) -> Self {`
- `/app/crates/contextra-avv-generator/src/lib.rs:154` [fn `with_subprocessors_allowed`]: `pub fn with_subprocessors_allowed(mut self, allowed: bool) -> Self {`
- `/app/crates/contextra-avv-generator/src/lib.rs:267` [fn `render_avv_markdown_with_date`]: `pub fn render_avv_markdown_with_date(ctx: &AvvContext, date: &str) -> Result<String, AvvError> {`
### `contextra-checkpoint`

- `/app/crates/contextra-checkpoint/src/store.rs:466` [fn `set_metrics_sink`]: `pub fn set_metrics_sink(&mut self, sink: Arc<dyn contextra_ports::MetricsSink>) {`
- `/app/crates/contextra-checkpoint/src/orphan.rs:51` [fn `total_recovered`]: `pub fn total_recovered(&self) -> usize {`
- `/app/crates/contextra-checkpoint/src/orphan.rs:149` [fn `register_orphan`]: `pub fn register_orphan(&self, pin_id: PinId) -> std::io::Result<()> {`
- `/app/crates/contextra-checkpoint/src/orphan.rs:351` [fn `with_vfs`]: `pub fn with_vfs(mut self, vfs: Arc<dyn contextra_ports::Vfs>) -> Self {`
### `contextra-cognition`

- `/app/crates/contextra-cognition/src/maintenance_scheduler.rs:87` [fn `with_edge_reinforcement_buffer`]: `pub fn with_edge_reinforcement_buffer(`
- `/app/crates/contextra-cognition/src/consolidation_executor.rs:559` [fn `with_leanrag`]: `pub fn with_leanrag(mut self, cfg: AggregationConfig) -> Self {`
- `/app/crates/contextra-cognition/src/consolidation_executor.rs:565` [fn `with_validator`]: `pub fn with_validator(mut self, validator: Arc<dyn ResponseGroundingValidator>) -> Self {`
- `/app/crates/contextra-cognition/src/consolidation_executor.rs:571` [fn `with_rich_validator`]: `pub fn with_rich_validator(mut self, validator: Arc<dyn GroundingValidator>) -> Self {`
- `/app/crates/contextra-cognition/src/consolidation_executor.rs:584` [fn `start_worker`]: `pub fn start_worker(`
- `/app/crates/contextra-cognition/src/context_compaction/compactor.rs:133` [fn `with_rng`]: `pub fn with_rng(mut self, rng: Arc<dyn Rng>) -> Self {`
### `contextra-crypto`

- `/app/crates/contextra-crypto/src/crypto.rs:316` [fn `cipher_for_scoped`]: `pub fn cipher_for_scoped<T>(`
- `/app/crates/contextra-crypto/src/ed25519_proof.rs:75` [fn `from_signing_key`]: `pub fn from_signing_key(signing_key: ed25519_dalek::SigningKey) -> Self {`
- `/app/crates/contextra-crypto/src/audit_chain.rs:149` [fn `with_signature`]: `pub fn with_signature(`
- `/app/crates/contextra-crypto/src/audit_chain.rs:288` [fn `sign_head`]: `pub fn sign_head(`
- `/app/crates/contextra-crypto/src/kv_shredding.rs:162` [fn `get_wrapped_kek`]: `pub fn get_wrapped_kek(&self, group_id: u64) -> Option<(Vec<u8>, [u8; 12])> {`
- `/app/crates/contextra-crypto/src/kv_shredding.rs:172` [fn `get_wrapped_dek`]: `pub fn get_wrapped_dek(&self, group_id: u64, record_id: u64) -> Option<(Vec<u8>, [u8; 12])> {`
- `/app/crates/contextra-crypto/src/kv_shredding.rs:336` [fn `is_key_active`]: `pub fn is_key_active(&self, group_id: u64) -> bool {`
- `/app/crates/contextra-crypto/src/kv_shredding.rs:341` [fn `is_record_active`]: `pub fn is_record_active(&self, group_id: u64, record_id: u64) -> bool {`
- `/app/crates/contextra-crypto/src/wal_crypto.rs:392` [fn `set_last_seq_no`]: `pub fn set_last_seq_no(&mut self, seq_no: Option<u64>) {`
- `/app/crates/contextra-crypto/src/wal_crypto.rs:397` [fn `last_seq_no_snapshot`]: `pub fn last_seq_no_snapshot(&self) -> Option<u64> {`
- `/app/crates/contextra-crypto/src/kdf.rs:94` [fn `new_for_test`]: `pub fn new_for_test(m_cost_kib: u32, t_cost: u32, p_cost: u32) -> Self {`
- `/app/crates/contextra-crypto/src/kdf.rs:132` [fn `generate_default`]: `pub fn generate_default() -> Result<Self> {`
- `/app/crates/contextra-crypto/src/kv_segment/store.rs:172` [fn `set_spill_handler`]: `pub fn set_spill_handler(&self, handler: SpillHandler) {`
- `/app/crates/contextra-crypto/src/kv_segment/store.rs:243` [fn `remove_segment`]: `pub fn remove_segment(&self, tenant: TenantId, segment_id: u64) {`
- `/app/crates/contextra-crypto/src/kv_segment/store.rs:259` [fn `insert_encrypted_segment`]: `pub fn insert_encrypted_segment(`
- `/app/crates/contextra-crypto/src/deletion_proof/creation.rs:18` [fn `issue`]: `pub fn issue<'a>(`
- `/app/crates/contextra-crypto/src/deletion_proof/creation.rs:213` [fn `create`]: `pub fn create(`
- `/app/crates/contextra-crypto/src/deletion_proof/creation.rs:323` [fn `create_v3_with_audit_position`]: `pub fn create_v3_with_audit_position(`
- `/app/crates/contextra-crypto/src/deletion_proof/key_derivation.rs:14` [fn `derive_deletion_proof_key_for_tenant`]: `pub fn derive_deletion_proof_key_for_tenant(`
- `/app/crates/contextra-crypto/src/deletion_proof/verifier.rs:144` [fn `verify_standalone_for_tenant`]: `pub fn verify_standalone_for_tenant(`
- ... und 4 weitere Kandidaten.
### `contextra-db`

- `/app/crates/contextra-db/src/homeostat.rs:21` [fn `is_expired`]: `pub fn is_expired(&self) -> bool {`
- `/app/crates/contextra-db/src/homeostat.rs:25` [fn `remaining`]: `pub fn remaining(&self) -> Duration {`
- `/app/crates/contextra-db/src/pid_latency_controller.rs:47` [fn `current_scaling_factor`]: `pub fn current_scaling_factor(&self) -> f64 {`
- `/app/crates/contextra-db/src/pid_latency_controller.rs:80` [fn `reset`]: `pub fn reset(&mut self) {`
### `contextra-durable-fs`

- `/app/crates/contextra-durable-fs/src/health.rs:109` [fn `allows_writes`]: `pub fn allows_writes(self) -> bool {`
- `/app/crates/contextra-durable-fs/src/vfs.rs:64` [fn `write_atomic`]: `fn write_atomic(&self, path: &Path, data: &[u8]) -> Result<()> {`
- `/app/crates/contextra-durable-fs/src/vfs.rs:111` [fn `list_dir`]: `fn list_dir(&self, path: &Path) -> Result<Vec<String>> {`
- `/app/crates/contextra-durable-fs/src/vfs.rs:176` [fn `read_verified`]: `fn read_verified(&self, offset: u64, len: usize) -> std::io::Result<(Vec<u8>, VerifiedVia)> {`
- `/app/crates/contextra-durable-fs/src/journal.rs:91` [fn `health_state`]: `pub fn health_state(&self) -> HealthState {`
- `/app/crates/contextra-durable-fs/src/journal.rs:104` [fn `record`]: `pub fn record(&mut self, path: &Path) -> std::io::Result<()> {`
- `/app/crates/contextra-durable-fs/src/journal.rs:111` [fn `pending`]: `pub fn pending(&self) -> &[PathBuf] {`
- `/app/crates/contextra-durable-fs/src/lib.rs:158` [struct `PoisonedHandle`]: `pub struct PoisonedHandle;`
- `/app/crates/contextra-durable-fs/src/lib.rs:201` [fn `new_with_path_and_injector`]: `pub fn new_with_path_and_injector(file: File, path: PathBuf, injector: F) -> Self {`
- `/app/crates/contextra-durable-fs/src/lib.rs:256` [fn `get_ref`]: `pub fn get_ref(&self) -> Option<&File> {`
- `/app/crates/contextra-durable-fs/src/lib.rs:276` [fn `poison`]: `pub fn poison(&mut self) {`
- `/app/crates/contextra-durable-fs/src/lib.rs:342` [fn `acquire`]: `pub fn acquire(dir: &Path, mode: LockMode) -> std::io::Result<Self> {`
### `contextra-engine`

- `/app/crates/contextra-engine/src/metrics_names.rs:15` [const `CHECKPOINT_DURATION`]: `pub const CHECKPOINT_DURATION: &str = "checkpoint.duration";`
- `/app/crates/contextra-engine/src/lib.rs:281` [fn `with_tenant_policy`]: `pub fn with_tenant_policy(mut self, policy: TenantPolicy) -> Self {`
- `/app/crates/contextra-engine/src/lib.rs:412` [fn `inner_storage`]: `pub fn inner_storage(&self) -> Arc<LsmStorage> {`
- `/app/crates/contextra-engine/src/collection/mod.rs:333` [fn `with_hnsw`]: `pub fn with_hnsw(`
- `/app/crates/contextra-engine/src/collection/mod.rs:424` [fn `with_clock`]: `pub fn with_clock(self, clock: Arc<dyn Clock>) -> Self {`
- `/app/crates/contextra-engine/src/collection/mod.rs:430` [fn `set_clock`]: `pub fn set_clock(&self, clock: Arc<dyn Clock>) {`
- `/app/crates/contextra-engine/src/collection/mod.rs:440` [fn `with_config`]: `pub fn with_config(self, config: CollectionConfig) -> Self {`
- `/app/crates/contextra-engine/src/collection/mod.rs:446` [fn `set_config`]: `pub fn set_config(&self, config: CollectionConfig) {`
- `/app/crates/contextra-engine/src/collection/mod.rs:456` [fn `with_pressure_receiver`]: `pub fn with_pressure_receiver(`
- `/app/crates/contextra-engine/src/collection/mod.rs:465` [fn `set_pressure_receiver`]: `pub fn set_pressure_receiver(`
- `/app/crates/contextra-engine/src/collection/mod.rs:545` [fn `user_key_prefix`]: `pub fn user_key_prefix(&self) -> Vec<u8> {`
- `/app/crates/contextra-engine/src/collection/mod.rs:657` [fn `with_community_detection_trigger_threshold`]: `pub fn with_community_detection_trigger_threshold(self, threshold: u64) -> Self {`
- `/app/crates/contextra-engine/src/contextra_impl/wiring.rs:24` [fn `set_router`]: `pub fn set_router(&self, router: std::sync::Weak<dyn DriftStatusProvider>) {`
- `/app/crates/contextra-engine/src/contextra_impl/wiring.rs:30` [fn `set_calibrator`]: `pub fn set_calibrator(`
- `/app/crates/contextra-engine/src/contextra_impl/erasure.rs:119` [fn `with_proof_key_provider`]: `pub fn with_proof_key_provider(self, provider: Arc<dyn ProofKeyProvider>) -> Self {`
- `/app/crates/contextra-engine/src/collection/query_builder/scope.rs:38` [fn `from_allowed_ids`]: `pub fn from_allowed_ids(allowed_doc_ids: BTreeSet<DocId>) -> Self {`
- `/app/crates/contextra-engine/src/collection/query_builder/mod.rs:24` [const `METRIC_SEARCH_LATENCY_VECTOR`]: `pub const METRIC_SEARCH_LATENCY_VECTOR: &str = "search.latency.vector";`
- `/app/crates/contextra-engine/src/collection/query_builder/mod.rs:27` [const `METRIC_SEARCH_LATENCY_TEXT`]: `pub const METRIC_SEARCH_LATENCY_TEXT: &str = "search.latency.text";`
- `/app/crates/contextra-engine/src/collection/query_builder/mod.rs:30` [const `METRIC_SEARCH_LATENCY_GRAPH`]: `pub const METRIC_SEARCH_LATENCY_GRAPH: &str = "search.latency.graph";`
- `/app/crates/contextra-engine/src/collection/crud/auto_extraction.rs:86` [fn `recommended_mode_for_regulated`]: `pub fn recommended_mode_for_regulated(is_regulated: bool) -> AutoExtractionMode {`
- ... und 2 weitere Kandidaten.
### `contextra-erasure`

- `/app/crates/contextra-erasure/src/ledger.rs:67` [fn `finalize_erasure`]: `pub fn finalize_erasure(`
- `/app/crates/contextra-erasure/src/restore.rs:26` [fn `is_replay`]: `pub fn is_replay(&self, seq: Seq) -> bool {`
- `/app/crates/contextra-erasure/src/journal.rs:37` [fn `open_durable`]: `pub fn open_durable(`
- `/app/crates/contextra-erasure/src/vault.rs:68` [fn `new_in_memory_with_key`]: `pub fn new_in_memory_with_key(total_capacity: usize, mac_key: [u8; 32]) -> Self {`
- `/app/crates/contextra-erasure/src/vault.rs:114` [fn `create_dir`]: `pub fn create_dir(`
- `/app/crates/contextra-erasure/src/vault.rs:162` [fn `open_dir`]: `pub fn open_dir(`
- `/app/crates/contextra-erasure/src/vault.rs:352` [fn `replace_durable_shard`]: `pub fn replace_durable_shard(&mut self, index: usize, shard: DurableVaultShard) {`
- `/app/crates/contextra-erasure/src/io.rs:29` [fn `read_exact_at`]: `pub fn read_exact_at(f: &dyn VfsFile, buf: &mut [u8], offset: u64) -> Result<(), ErasureError> {`
- `/app/crates/contextra-erasure/src/io.rs:54` [fn `zero_at`]: `pub fn zero_at(f: &dyn VfsFile, offset: u64, len: usize) -> Result<(), ErasureError> {`
- `/app/crates/contextra-erasure/src/supersede.rs:142` [fn `verify_updated_seq`]: `pub fn verify_updated_seq(&self, latest_committed_seq: u64) -> Result<(), ErasureError> {`
- `/app/crates/contextra-erasure/src/types_local.rs:598` [fn `compute_chained`]: `pub fn compute_chained(`
- `/app/crates/contextra-erasure/src/types_local.rs:628` [fn `verify_chain_hash`]: `pub fn verify_chain_hash(&self, prev_hash: &[u8; 32]) -> bool {`
- `/app/crates/contextra-erasure/src/types_local.rs:1360` [fn `read_only`]: `pub fn read_only() -> Self {`
- `/app/crates/contextra-erasure/src/types_local.rs:1367` [fn `write_create`]: `pub fn write_create() -> Self {`
- `/app/crates/contextra-erasure/src/restricted.rs:102` [fn `allows_access_to_restricted`]: `pub fn allows_access_to_restricted(&self, is_target_restricted: bool) -> bool {`
- `/app/crates/contextra-erasure/src/restricted.rs:143` [fn `create_entry`]: `pub fn create_entry(`
- `/app/crates/contextra-erasure/src/restricted.rs:175` [fn `is_lifted`]: `pub fn is_lifted(&self) -> bool {`
- `/app/crates/contextra-erasure/src/restricted.rs:185` [fn `lift`]: `pub fn lift(&mut self, _reason: &str) -> Result<(), ErasureError> {`
- `/app/crates/contextra-erasure/src/restricted.rs:199` [fn `supersede_by_erasure`]: `pub fn supersede_by_erasure(&mut self) -> Result<(), ErasureError> {`
- `/app/crates/contextra-erasure/src/replace.rs:88` [fn `advance_to_r2_intent`]: `pub fn advance_to_r2_intent(&mut self) -> Result<(), ErasureError> {`
- ... und 46 weitere Kandidaten.
### `contextra-graph`

- `/app/crates/contextra-graph/src/arc_slice.rs:81` [fn `as_arc`]: `pub fn as_arc(&self) -> &Arc<[T]> {`
- `/app/crates/contextra-graph/src/ppr_stream.rs:50` [fn `with_batch_size`]: `pub fn with_batch_size(mut self, n: usize) -> Self {`
- `/app/crates/contextra-graph/src/ppr_stream.rs:57` [fn `with_exclude_seeds`]: `pub fn with_exclude_seeds(mut self, exclude: bool) -> Self {`
- `/app/crates/contextra-graph/src/ppr_stream.rs:120` [fn `is_exhausted`]: `pub fn is_exhausted(&self) -> bool {`
- `/app/crates/contextra-graph/src/provider.rs:68` [fn `add_hyperedge`]: `pub fn add_hyperedge(&mut self, hyperedge: HyperEdge) {`
- `/app/crates/contextra-graph/src/cascade.rs:22` [const `DEFAULT_HYPEREDGE_CASCADE_FANOUT_LIMIT`]: `pub const DEFAULT_HYPEREDGE_CASCADE_FANOUT_LIMIT: usize = MAX_HYPEREDGE_CASCADE_FANOUT;`
- `/app/crates/contextra-graph/src/hyperedge.rs:248` [fn `slice_participants`]: `pub fn slice_participants(`
- `/app/crates/contextra-graph/src/apprh/selector.rs:92` [fn `with_default_monitor`]: `pub fn with_default_monitor(config: ApprhSelectorConfig) -> Result<Self, ApprhError> {`
- `/app/crates/contextra-graph/src/csr/graph_write/mod.rs:160` [fn `get_source_doc_id`]: `pub fn get_source_doc_id(&self, index: usize) -> Option<DocId> {`
- `/app/crates/contextra-graph/src/csr/graph_write/hyperedge.rs:54` [fn `pending_cascade_queue_len`]: `pub fn pending_cascade_queue_len(&self) -> usize {`
- `/app/crates/contextra-graph/src/csr/graph_write/hyperedge.rs:357` [fn `commit_super_edge_batch`]: `pub fn commit_super_edge_batch(`
### `contextra-infer-candle`

- `/app/crates/contextra-infer-candle/src/attention_exporter.rs:88` [fn `tracked_request_count`]: `pub fn tracked_request_count(&self) -> usize {`
- `/app/crates/contextra-infer-candle/src/kv_state.rs:163` [fn `layer_count`]: `pub fn layer_count(&self) -> usize {`
- `/app/crates/contextra-infer-candle/src/kv_state.rs:305` [fn `import_block_at`]: `pub fn import_block_at(&mut self, block: &KvBlock, at: usize) -> Result<(), ContextraError> {`
- `/app/crates/contextra-infer-candle/src/inference/core.rs:172` [fn `with_prefix_store`]: `pub fn with_prefix_store(mut self, store: Arc<dyn contextra_ports::kv::KvPrefixStore>) -> Self {`
- `/app/crates/contextra-infer-candle/src/inference/quantized_llama.rs:62` [fn `with_generation_config`]: `pub fn with_generation_config(mut self, config: GenerationConfig) -> Self {`
- `/app/crates/contextra-infer-candle/src/inference/config.rs:47` [fn `with_max_context_length`]: `pub fn with_max_context_length(mut self, limit: usize) -> Self {`
- `/app/crates/contextra-infer-candle/src/inference/config.rs:53` [fn `with_max_token_budget`]: `pub fn with_max_token_budget(mut self, budget: usize) -> Self {`
- `/app/crates/contextra-infer-candle/src/inference/config.rs:73` [fn `validate_token_budget`]: `pub fn validate_token_budget(&self, requested_tokens: usize) -> Result<(), ContextraError> {`
### `contextra-infer-ollama`

- `/app/crates/contextra-infer-ollama/src/importance.rs:129` [fn `with_provenance`]: `pub fn with_provenance(`
- `/app/crates/contextra-infer-ollama/src/client/core.rs:197` [fn `with_clock`]: `pub fn with_clock(mut self, clock: Arc<dyn Clock>) -> Self {`
- `/app/crates/contextra-infer-ollama/src/client/policy.rs:44` [fn `loopback_only`]: `pub fn loopback_only() -> Self {`
### `contextra-infer-onnx`

- `/app/crates/contextra-infer-onnx/src/lib.rs:738` [fn `with_expected_dimension`]: `pub fn with_expected_dimension(mut self, dim: usize) -> Self {`
### `contextra-ingest`

- `/app/crates/contextra-ingest/src/format.rs:104` [fn `extensions`]: `pub fn extensions(self) -> &'static [&'static str] {`
- `/app/crates/contextra-ingest/src/safe/archive.rs:93` [fn `file_names`]: `pub fn file_names(&mut self) -> Vec<String> {`
- `/app/crates/contextra-ingest/src/formats/ole/cfb_reader.rs:105` [fn `list_children`]: `pub fn list_children(&self, storage_path: &str) -> Result<Vec<Entry>, IngestError> {`
- `/app/crates/contextra-ingest/src/formats/web/html.rs:107` [fn `finish`]: `fn finish(self) -> Self::Output {`
- `/app/crates/contextra-ingest/src/formats/web/html.rs:111` [fn `parse_error`]: `fn parse_error(&self, _msg: Cow<'static, str>) {}`
- `/app/crates/contextra-ingest/src/formats/web/html.rs:113` [fn `get_document`]: `fn get_document(&self) -> Self::Handle {`
- `/app/crates/contextra-ingest/src/formats/web/html.rs:117` [fn `set_quirks_mode`]: `fn set_quirks_mode(&self, mode: QuirksMode) {`
- `/app/crates/contextra-ingest/src/formats/web/html.rs:121` [fn `same_node`]: `fn same_node(&self, x: &Self::Handle, y: &Self::Handle) -> bool {`
- `/app/crates/contextra-ingest/src/formats/web/html.rs:125` [fn `elem_name`]: `fn elem_name<'a>(&'a self, target: &'a Self::Handle) -> Self::ElemName<'a> {`
- `/app/crates/contextra-ingest/src/formats/web/html.rs:135` [fn `get_template_contents`]: `fn get_template_contents(&self, target: &Self::Handle) -> Self::Handle {`
- `/app/crates/contextra-ingest/src/formats/web/html.rs:166` [fn `append_based_on_parent_node`]: `fn append_based_on_parent_node(`
- `/app/crates/contextra-ingest/src/formats/web/html.rs:180` [fn `append_before_sibling`]: `fn append_before_sibling(&self, sibling: &Self::Handle, child: NodeOrText<Self::Handle>) {`
- `/app/crates/contextra-ingest/src/formats/web/html.rs:187` [fn `append_doctype_to_document`]: `fn append_doctype_to_document(`
- `/app/crates/contextra-ingest/src/formats/web/html.rs:201` [fn `add_attrs_if_missing`]: `fn add_attrs_if_missing(&self, target: &Self::Handle, attrs: Vec<Attribute>) {`
- `/app/crates/contextra-ingest/src/formats/web/html.rs:216` [fn `remove_from_parent`]: `fn remove_from_parent(&self, target: &Self::Handle) {`
- `/app/crates/contextra-ingest/src/formats/web/html.rs:224` [fn `reparent_children`]: `fn reparent_children(&self, from: &Self::Handle, to: &Self::Handle) {`
- `/app/crates/contextra-ingest/src/formats/web/html.rs:232` [fn `is_mathml_annotation_xml_integration_point`]: `fn is_mathml_annotation_xml_integration_point(&self, _handle: &Self::Handle) -> bool {`
- `/app/crates/contextra-ingest/src/formats/web/html.rs:236` [fn `create_element`]: `fn create_element(`
- `/app/crates/contextra-ingest/src/formats/web/html.rs:252` [fn `create_comment`]: `fn create_comment(&self, text: StrTendril) -> Self::Handle {`
- `/app/crates/contextra-ingest/src/formats/web/html.rs:260` [fn `create_pi`]: `fn create_pi(&self, _target: StrTendril, _data: StrTendril) -> Self::Handle {`
- ... und 13 weitere Kandidaten.
### `contextra-kvcache`

- `/app/crates/contextra-kvcache/src/store.rs:543` [fn `remove_segment`]: `pub fn remove_segment(&self, tenant: TenantId, segment_id: u64) {`
- `/app/crates/contextra-kvcache/src/store.rs:559` [fn `insert_encrypted_segment`]: `pub fn insert_encrypted_segment(`
- `/app/crates/contextra-kvcache/src/store.rs:593` [fn `get_decrypted_segment`]: `pub fn get_decrypted_segment(`
- `/app/crates/contextra-kvcache/src/store.rs:858` [fn `scan_units`]: `fn scan_units(`
- `/app/crates/contextra-kvcache/src/segment.rs:324` [fn `write_quantized`]: `pub fn write_quantized(`
- `/app/crates/contextra-kvcache/src/segment.rs:354` [fn `read_dequantized`]: `pub fn read_dequantized(`
- `/app/crates/contextra-kvcache/src/eviction_worker.rs:155` [fn `register_segment_request`]: `pub fn register_segment_request(`
- `/app/crates/contextra-kvcache/src/eviction_worker.rs:167` [fn `unregister_segment_request`]: `pub fn unregister_segment_request(&self, tenant_id: TenantId, segment_id: u64) {`
- `/app/crates/contextra-kvcache/src/service.rs:151` [fn `enable_kivi_quantization`]: `pub fn enable_kivi_quantization(mut self, config: Option<KiviQuantizeConfig>) -> Self {`
- `/app/crates/contextra-kvcache/src/service.rs:457` [fn `invalidate_scratchpad`]: `pub fn invalidate_scratchpad(`
### `contextra-license`

- `/app/crates/contextra-license/src/signed_gate.rs:191` [fn `create_signed`]: `pub fn create_signed(`
- `/app/crates/contextra-license/src/signed_gate.rs:496` [fn `no_activation`]: `pub fn no_activation() -> Self {`
- `/app/crates/contextra-license/src/signed_gate.rs:509` [fn `from_activation`]: `pub fn from_activation(activation: SignedActivation, verifying_key: VerifyingKey) -> Self {`
- `/app/crates/contextra-license/src/signed_gate.rs:534` [fn `from_signed_payload`]: `pub fn from_signed_payload(`
- `/app/crates/contextra-license/src/signed_gate.rs:550` [fn `from_signed_payload_with_clock`]: `pub fn from_signed_payload_with_clock(`
- `/app/crates/contextra-license/src/signed_gate.rs:621` [fn `with_local_installation_id`]: `pub fn with_local_installation_id(mut self, id_hash: [u8; 32]) -> Self {`
- `/app/crates/contextra-license/src/signed_gate.rs:644` [fn `check_step_1_fast_ring`]: `pub fn check_step_1_fast_ring(&self, ring: FeatureRing) -> bool {`
- `/app/crates/contextra-license/src/signed_gate.rs:649` [fn `check_step_2_presence`]: `pub fn check_step_2_presence(&self, ring: FeatureRing) -> Result<(), StepVerificationError> {`
- `/app/crates/contextra-license/src/signed_gate.rs:658` [fn `check_step_3_installation`]: `pub fn check_step_3_installation(`
- `/app/crates/contextra-license/src/signed_gate.rs:671` [fn `check_step_4_signature`]: `pub fn check_step_4_signature(&self) -> Result<(), StepVerificationError> {`
- `/app/crates/contextra-license/src/signed_gate.rs:680` [fn `check_step_5_expiration`]: `pub fn check_step_5_expiration(&self) -> Result<(), StepVerificationError> {`
- `/app/crates/contextra-license/src/signed_gate.rs:689` [fn `check_step_6_ring_permission`]: `pub fn check_step_6_ring_permission(`
- `/app/crates/contextra-license/src/signed_gate.rs:701` [fn `check_step_7_success`]: `pub fn check_step_7_success(&self) -> Result<(), StepVerificationError> {`
- `/app/crates/contextra-license/src/signed_gate.rs:707` [fn `verify_ring`]: `pub fn verify_ring(&self, ring: FeatureRing) -> Result<(), LicenseError> {`
- `/app/crates/contextra-license/src/signed_gate.rs:713` [fn `create_test_signed_payload`]: `pub fn create_test_signed_payload(`
### `contextra-mcp`

- `/app/crates/contextra-mcp/src/protocol.rs:29` [fn `parse_error`]: `pub fn parse_error(msg: impl Into<String>) -> Self {`
- `/app/crates/contextra-mcp/src/protocol.rs:33` [fn `invalid_request`]: `pub fn invalid_request(msg: impl Into<String>) -> Self {`
- `/app/crates/contextra-mcp/src/protocol.rs:37` [fn `method_not_found`]: `pub fn method_not_found(msg: impl Into<String>) -> Self {`
- `/app/crates/contextra-mcp/src/egress_guard.rs:14` [fn `search_text`]: `fn search_text<'a>(`
- `/app/crates/contextra-mcp/src/routing.rs:9` [fn `search_hybrid`]: `fn search_hybrid<'a>(`
- `/app/crates/contextra-mcp/src/routing.rs:39` [fn `get_community`]: `fn get_community<'a>(`
- `/app/crates/contextra-mcp/src/routing.rs:50` [fn `prepare_context`]: `fn prepare_context(`
- `/app/crates/contextra-mcp/src/server.rs:69` [fn `with_tenant_id`]: `pub fn with_tenant_id(mut self, tenant_id: Option<TenantId>) -> Self {`
- `/app/crates/contextra-mcp/src/server.rs:74` [fn `with_injection_guard`]: `pub fn with_injection_guard(mut self, injection_guard: Arc<PromptInjectionGuard>) -> Self {`
- `/app/crates/contextra-mcp/src/server.rs:79` [fn `with_egress_classifier`]: `pub fn with_egress_classifier(mut self, classifier: Arc<dyn EgressClassifier>) -> Self {`
- `/app/crates/contextra-mcp/src/server.rs:89` [fn `with_plugin_registry`]: `pub fn with_plugin_registry(`
### `contextra-mvcc`

- `/app/crates/contextra-mvcc/src/tx_buffer.rs:59` [const `DEFAULT_MAX_READ_SET_KEYS`]: `pub const DEFAULT_MAX_READ_SET_KEYS: usize = 100_000;`
- `/app/crates/contextra-mvcc/src/tx_buffer.rs:770` [fn `drain_kv`]: `pub fn drain_kv(&self, tx: TxId) -> Vec<IndexOp<(Vec<u8>, Vec<u8>)>> {`
- `/app/crates/contextra-mvcc/src/tx_buffer.rs:779` [fn `discard_kv`]: `pub fn discard_kv(&self, tx: TxId) {`
- `/app/crates/contextra-mvcc/src/tx_buffer.rs:807` [fn `staged_status`]: `pub fn staged_status(&self, key: &[u8]) -> Option<bool> {`
- `/app/crates/contextra-mvcc/src/snapshot.rs:219` [fn `pin_for_tenant`]: `pub fn pin_for_tenant(&self, tenant: TenantId, seq_no: u64) {`
- `/app/crates/contextra-mvcc/src/snapshot.rs:327` [fn `longest_active_pin`]: `pub fn longest_active_pin(&self) -> Option<(u64, Duration)> {`
- `/app/crates/contextra-mvcc/src/ssi.rs:166` [fn `keys_map`]: `pub fn keys_map(&self) -> &AHashMap<Vec<u8>, u64> {`
- `/app/crates/contextra-mvcc/src/ssi.rs:171` [fn `prefixes_map`]: `pub fn prefixes_map(&self) -> &AHashMap<Vec<u8>, u64> {`
- `/app/crates/contextra-mvcc/src/ssi.rs:593` [fn `with_snapshot_registry`]: `pub fn with_snapshot_registry(mut self, registry: Arc<SnapshotRegistry>) -> Self {`
- `/app/crates/contextra-mvcc/src/ssi.rs:599` [fn `set_snapshot_registry`]: `pub fn set_snapshot_registry(&mut self, registry: Arc<SnapshotRegistry>) {`
- `/app/crates/contextra-mvcc/src/ssi.rs:604` [fn `with_metrics_sink`]: `pub fn with_metrics_sink(`
- `/app/crates/contextra-mvcc/src/ssi.rs:613` [fn `set_metrics_sink`]: `pub fn set_metrics_sink(`
- `/app/crates/contextra-mvcc/src/ssi.rs:651` [fn `forget_from`]: `pub fn forget_from(&self, first_seq: u64) -> usize {`
- `/app/crates/contextra-mvcc/src/ssi.rs:733` [fn `diagnose_pruning_blocker`]: `pub fn diagnose_pruning_blocker(&self) -> Option<PruningBlockerInfo> {`
- `/app/crates/contextra-mvcc/src/ssi.rs:993` [fn `validate_and_record`]: `pub fn validate_and_record<'a>(`
### `contextra-ports`

- `/app/crates/contextra-ports/src/ledger.rs:95` [fn `compute_chained`]: `pub fn compute_chained(`
- `/app/crates/contextra-ports/src/ledger.rs:125` [fn `verify_chain_hash`]: `pub fn verify_chain_hash(&self, prev_hash: &[u8; 32]) -> bool {`
- `/app/crates/contextra-ports/src/ledger.rs:169` [fn `read_all`]: `fn read_all(&self) -> Result<Vec<LedgerEntry>, String>;`
- `/app/crates/contextra-ports/src/ledger.rs:178` [fn `prune`]: `fn prune(&self, older_than: Timestamp) -> Result<usize, String>;`
- `/app/crates/contextra-ports/src/keys.rs:136` [fn `seal`]: `fn seal<'a>(`
- `/app/crates/contextra-ports/src/keys.rs:169` [fn `make_slots_durable`]: `fn make_slots_durable(&self) -> BoxFuture<'_, Result<(), String>>;`
- `/app/crates/contextra-ports/src/erasure.rs:253` [fn `resolve`]: `fn resolve(&self, t: &EraseTarget) -> BoxFuture<'_, Result<LayerScope, String>>;`
- `/app/crates/contextra-ports/src/erasure.rs:262` [fn `on_gate`]: `fn on_gate(&self, s: &LayerScope) -> BoxFuture<'_, Result<(), String>>;`
- `/app/crates/contextra-ports/src/erasure.rs:295` [fn `key_coverage`]: `fn key_coverage(&self) -> KeyCoverage;`
- `/app/crates/contextra-ports/src/vfs.rs:66` [fn `write_create`]: `pub fn write_create() -> Self {`
- `/app/crates/contextra-ports/src/vfs.rs:111` [fn `write_at`]: `fn write_at(&self, buf: &[u8], offset: u64) -> std::io::Result<usize>;`
- `/app/crates/contextra-ports/src/vfs.rs:120` [fn `set_len`]: `fn set_len(&self, len: u64) -> std::io::Result<()>;`
- `/app/crates/contextra-ports/src/gate.rs:145` [fn `check_bound`]: `fn check_bound(&self, b: &KeyBound) -> GateDecision;`
- `/app/crates/contextra-ports/src/gate.rs:154` [fn `variant_for_range`]: `fn variant_for_range(&self, r: &KeyRange) -> RangeGate;`
- `/app/crates/contextra-ports/src/gate.rs:168` [fn `gate_epoch`]: `fn gate_epoch(&self) -> u64 {`
- `/app/crates/contextra-ports/src/gate.rs:175` [fn `covers`]: `fn covers(&self, k: &KeyBuf) -> bool {`
- `/app/crates/contextra-ports/src/gate.rs:188` [fn `covers_raw`]: `fn covers_raw(&self, raw: &[u8; 78]) -> bool {`
- `/app/crates/contextra-ports/src/gate.rs:196` [fn `gated_seq_raw`]: `fn gated_seq_raw(&self, _raw: &[u8; 78]) -> Option<Seq> {`
- `/app/crates/contextra-ports/src/conformance.rs:38` [fn `assert_checkpoint_contract`]: `pub fn assert_checkpoint_contract<T: Checkpoint>(t: &T, tx: TxId, state: &WorkflowState) {`
- `/app/crates/contextra-ports/src/conformance.rs:72` [fn `assert_attention_exporter_contract`]: `pub fn assert_attention_exporter_contract<T: AttentionExporter>(t: &T, req: RequestId) {`
- ... und 13 weitere Kandidaten.
### `contextra-privacy`

- `/app/crates/contextra-privacy/src/entity_recognizer.rs:92` [fn `with_gazetteer`]: `pub fn with_gazetteer(gazetteer: Gazetteer) -> Self {`
- `/app/crates/contextra-privacy/src/entity_recognizer.rs:101` [fn `with_max_input_length`]: `pub fn with_max_input_length(mut self, limit: usize) -> Self {`
- `/app/crates/contextra-privacy/src/entity_recognizer.rs:114` [fn `add_gazetteer_term`]: `pub fn add_gazetteer_term(mut self, term: impl Into<String>) -> Self {`
- `/app/crates/contextra-privacy/src/entity_recognizer.rs:127` [fn `recognize_bounded`]: `pub fn recognize_bounded(`
- `/app/crates/contextra-privacy/src/error.rs:94` [fn `internal_error`]: `pub fn internal_error(msg: impl Into<String>) -> Self {`
- `/app/crates/contextra-privacy/src/processing_registry.rs:109` [fn `append_record`]: `pub fn append_record(`
- `/app/crates/contextra-privacy/src/egress_gateway.rs:109` [fn `with_classifier`]: `pub fn with_classifier(mut self, classifier: Arc<dyn EgressClassifier>) -> Self {`
- `/app/crates/contextra-privacy/src/egress_gateway.rs:124` [fn `with_egress_vault`]: `pub fn with_egress_vault(mut self, vault: Arc<EgressVault>) -> Self {`
- `/app/crates/contextra-privacy/src/egress_gateway.rs:134` [fn `with_bulk_detector`]: `pub fn with_bulk_detector(`
- `/app/crates/contextra-privacy/src/egress_gateway.rs:389` [fn `resolve_effective_kv_delete_mode`]: `pub fn resolve_effective_kv_delete_mode<D, K>(`
### `contextra-py`

- `/app/crates/contextra-py/src/lib.rs:141` [const `VERSION`]: `pub const VERSION: &str = "0.1.0";`
### `contextra-rank`

- `/app/crates/contextra-rank/src/drift.rs:90` [fn `set_baseline`]: `pub fn set_baseline(&mut self, baseline: f32) {`
- `/app/crates/contextra-rank/src/drift.rs:97` [fn `observe`]: `pub fn observe(&mut self, score: f32) -> DriftStatus {`
- `/app/crates/contextra-rank/src/drift.rs:136` [fn `overall_drift_status`]: `pub fn overall_drift_status(&self) -> &'static str {`
- `/app/crates/contextra-rank/src/fusion/types.rs:110` [fn `synthesized_from`]: `pub fn synthesized_from(source_doc_ids: &[contextra_types::DocId]) -> Self {`
- `/app/crates/contextra-rank/src/calibration/platt.rs:54` [fn `is_fitted`]: `pub fn is_fitted(&self) -> bool {`
- `/app/crates/contextra-rank/src/calibration/platt.rs:59` [fn `params`]: `pub fn params(&self) -> (f32, f32) {`
- `/app/crates/contextra-rank/src/calibration/platt.rs:64` [fn `predict`]: `pub fn predict(&self, raw_score: f32) -> f32 {`
- `/app/crates/contextra-rank/src/calibration/platt.rs:69` [fn `apply`]: `pub fn apply(&self, logit: f32) -> f32 {`
- `/app/crates/contextra-rank/src/calibration/platt.rs:105` [fn `fit`]: `pub fn fit(observations: &[(f32, bool)]) -> Self {`
- `/app/crates/contextra-rank/src/calibration/conformal.rs:140` [fn `with_bounds`]: `pub fn with_bounds(`
- `/app/crates/contextra-rank/src/calibration/isotonic.rs:136` [fn `calibrated_probability`]: `pub fn calibrated_probability(&mut self, raw_score: f32) -> Option<f32> {`
- `/app/crates/contextra-rank/src/calibration/isotonic.rs:274` [fn `expected_calibration_error`]: `pub fn expected_calibration_error(&mut self) -> Option<f32> {`
- `/app/crates/contextra-rank/src/dibud/state.rs:83` [fn `provenance_of`]: `pub fn provenance_of(&self, doc_id: &DocId) -> Option<&ProvenanceRecord> {`
### `contextra-router`

- `/app/crates/contextra-router/src/router/outcomes.rs:24` [fn `bandit_decision_propensity`]: `pub fn bandit_decision_propensity(&self, id: DecisionId) -> Option<f32> {`
- `/app/crates/contextra-router/src/router/lifecycle.rs:107` [fn `with_vfs`]: `pub fn with_vfs(mut self, vfs: Arc<dyn Vfs>) -> Self {`
- `/app/crates/contextra-router/src/router/lifecycle.rs:123` [fn `with_initial_decision_id`]: `pub fn with_initial_decision_id(mut self, start: u64) -> Self {`
- `/app/crates/contextra-router/src/router/lifecycle.rs:132` [fn `with_routing_strategy`]: `pub fn with_routing_strategy(`
- `/app/crates/contextra-router/src/tests/fixtures.rs:29` [fn `advance`]: `pub fn advance(&self, duration: std::time::Duration) {`
- `/app/crates/contextra-router/src/tests/fixtures.rs:94` [fn `set_community`]: `pub fn set_community(&self, entity_id: EntityId, comm_id: u64) {`
### `contextra-sandbox`

- `/app/crates/contextra-sandbox/src/capabilities.rs:68` [fn `pure_merge_operator`]: `pub fn pure_merge_operator() -> Self {`
- `/app/crates/contextra-sandbox/src/merge.rs:43` [fn `new_pure`]: `pub fn new_pure(wasm_bytes: impl Into<Arc<[u8]>>) -> Result<Self, SandboxError> {`
### `contextra-store`

- `/app/crates/contextra-store/src/memtable.rs:598` [fn `shard_entry_counts`]: `pub fn shard_entry_counts(&self) -> [usize; SHARD_COUNT] {`
- `/app/crates/contextra-store/src/tenant_codec.rs:60` [fn `encode_graph_key`]: `pub fn encode_graph_key(&self, collection: &CollectionId, entity_id: u64) -> Vec<u8> {`
- `/app/crates/contextra-store/src/tenant_codec.rs:78` [fn `collection_prefix`]: `pub fn collection_prefix(&self, collection: &CollectionId) -> Vec<u8> {`
- `/app/crates/contextra-store/src/lsm/engine.rs:121` [fn `spawn_tracked`]: `pub fn spawn_tracked<F>(&self, future: F)`
- `/app/crates/contextra-store/src/lsm/engine.rs:158` [const `ROLLBACK_INLINE_THRESHOLD_ENTRIES`]: `pub const ROLLBACK_INLINE_THRESHOLD_ENTRIES: usize = 1;`
- `/app/crates/contextra-store/src/lsm/engine.rs:186` [fn `set_metrics_sink`]: `pub fn set_metrics_sink(&self, sink: Arc<dyn contextra_ports::MetricsSink>) {`
- `/app/crates/contextra-store/src/lsm/engine.rs:223` [fn `with_ssi_validator`]: `pub fn with_ssi_validator(`
- `/app/crates/contextra-store/src/lsm/engine.rs:299` [fn `next_seq_no_for_test`]: `pub fn next_seq_no_for_test(&self) -> u64 {`
- `/app/crates/contextra-store/src/lsm/engine.rs:313` [fn `sstables_for_test`]: `pub fn sstables_for_test(&self) -> Arc<RwLock<Vec<Arc<SstableReader>>>> {`
- `/app/crates/contextra-store/src/lsm/engine.rs:319` [fn `block_cache_for_test`]: `pub fn block_cache_for_test(&self) -> Arc<BlockCache> {`
- `/app/crates/contextra-store/src/lsm/engine.rs:325` [fn `budget_for_test`]: `pub fn budget_for_test(&self) -> Arc<ResourceTracker> {`
- `/app/crates/contextra-store/src/lsm/engine.rs:331` [fn `manifest_for_test`]: `pub fn manifest_for_test(&self) -> Arc<crate::manifest::Manifest> {`
- `/app/crates/contextra-store/src/lsm/config.rs:41` [fn `validate_against_features`]: `pub fn validate_against_features(`
- `/app/crates/contextra-store/src/lsm/mod.rs:113` [const `MAX_INTERNAL_MERGE_ENTRIES_FACTOR`]: `pub const MAX_INTERNAL_MERGE_ENTRIES_FACTOR: usize = 8;`
- `/app/crates/contextra-store/src/lsm/observer.rs:299` [fn `dropped_count_for`]: `pub fn dropped_count_for(&self, observer: &Arc<dyn WalObserver>) -> u64 {`
- `/app/crates/contextra-store/src/lsm/observer.rs:321` [fn `is_any_circuit_breaker_open`]: `pub fn is_any_circuit_breaker_open(&self) -> bool {`
- `/app/crates/contextra-store/src/lsm/observer.rs:329` [fn `clear_circuit_breaker`]: `pub fn clear_circuit_breaker(&self, observer: &Arc<dyn WalObserver>) {`
- `/app/crates/contextra-store/src/compaction/engine.rs:97` [fn `with_adaptive_planner`]: `pub fn with_adaptive_planner(mut self, planner: Arc<dyn AdaptiveCompactionPlanner>) -> Self {`
- `/app/crates/contextra-store/src/kv/segment.rs:72` [fn `write_segment`]: `pub fn write_segment(`
- `/app/crates/contextra-store/src/kv/segment.rs:113` [fn `read_segment`]: `pub fn read_segment(&self, segment: &KvSegmentPayload) -> Result<Vec<u8>, ContextraError> {`
- ... und 12 weitere Kandidaten.
### `contextra-testkit`

- `/app/crates/contextra-testkit/src/fault_vfs.rs:97` [fn `trigger_crash`]: `pub fn trigger_crash(&self) {`
- `/app/crates/contextra-testkit/src/manual_clock.rs:101` [fn `now_secs_f64`]: `pub fn now_secs_f64(&self) -> f64 {`
- `/app/crates/contextra-testkit/src/reference_model.rs:133` [fn `commit_batch`]: `pub fn commit_batch(&mut self, ops: Vec<RefOp>) -> u64 {`
- `/app/crates/contextra-testkit/src/reference_model.rs:140` [fn `snapshot_seq`]: `pub fn snapshot_seq(&self) -> u64 {`
- `/app/crates/contextra-testkit/src/reference_model.rs:189` [fn `scan_range_at`]: `pub fn scan_range_at(`
- `/app/crates/contextra-testkit/src/reference_model.rs:216` [fn `get_latest`]: `pub fn get_latest(&self, key: &[u8]) -> Option<Vec<u8>> {`
- `/app/crates/contextra-testkit/src/reference_model.rs:222` [fn `scan_prefix_latest`]: `pub fn scan_prefix_latest(&self, prefix: &[u8]) -> Vec<(Vec<u8>, Vec<u8>)> {`
- `/app/crates/contextra-testkit/src/reference_model.rs:228` [fn `snapshot_map_at`]: `pub fn snapshot_map_at(&self, seq: u64) -> BTreeMap<Vec<u8>, Vec<u8>> {`
- `/app/crates/contextra-testkit/src/in_memory_store.rs:223` [fn `flush`]: `fn flush<'a>(&'a self) -> BoxFuture<'a, Result<()>> {`
### `contextra-text`

- `/app/crates/contextra-text/src/stream.rs:82` [fn `with_batch_size`]: `pub fn with_batch_size(mut self, n: usize) -> Self {`
- `/app/crates/contextra-text/src/stream.rs:135` [fn `is_exhausted`]: `pub fn is_exhausted(&self) -> bool {`
- `/app/crates/contextra-text/src/inverted/scanner.rs:111` [fn `new_with_target`]: `pub fn new_with_target(index: Arc<InvertedIndex<S>>, target: ScanTarget) -> Self {`
- `/app/crates/contextra-text/src/inverted/scanner.rs:120` [fn `with_target`]: `pub fn with_target(self, target: ScanTarget) -> Self {`
- `/app/crates/contextra-text/src/inverted/scanner.rs:127` [fn `set_target`]: `pub fn set_target(&self, target: ScanTarget) {`
- `/app/crates/contextra-text/src/morphology/compound_splitter.rs:115` [fn `new_with_all_domains`]: `pub fn new_with_all_domains() -> Self {`
- `/app/crates/contextra-text/src/morphology/compound_splitter.rs:123` [fn `extend_vocabulary`]: `pub fn extend_vocabulary(&mut self, words: impl IntoIterator<Item = impl AsRef<str>>) {`
### `contextra-types`

- `/app/crates/contextra-types/src/tenant_scope.rs:51` [fn `into_inner_checked`]: `pub fn into_inner_checked(self, expected: &TenantId) -> Result<T, TenantScopeViolation> {`
- `/app/crates/contextra-types/src/erasure.rs:116` [struct `EraseRequest`]: `pub struct EraseRequest {`
- `/app/crates/contextra-types/src/erasure.rs:133` [struct `EraseReceipt`]: `pub struct EraseReceipt {`
- `/app/crates/contextra-types/src/erasure.rs:269` [enum `KeyCoverage`]: `pub enum KeyCoverage {`
- `/app/crates/contextra-types/src/erasure.rs:284` [enum `GateVariant`]: `pub enum GateVariant {`
- `/app/crates/contextra-types/src/erasure.rs:409` [fn `condition`]: `pub fn condition(&self) -> RiskCondition {`
- `/app/crates/contextra-types/src/error.rs:384` [fn `limit_exceeded`]: `pub fn limit_exceeded(limit: usize, context: impl Into<String>) -> Self {`
- `/app/crates/contextra-types/src/error.rs:402` [fn `snapshot_unsupported_for_signal`]: `pub fn snapshot_unsupported_for_signal(strategy: impl Into<String>) -> Self {`
- `/app/crates/contextra-types/src/error.rs:455` [fn `kv_quantization`]: `pub fn kv_quantization(msg: impl Into<String>) -> Self {`
- `/app/crates/contextra-types/src/error.rs:461` [fn `durability_config`]: `pub fn durability_config(msg: impl Into<String>) -> Self {`
- `/app/crates/contextra-types/src/error.rs:485` [fn `wal_truncation_detected`]: `pub fn wal_truncation_detected(expected_hmac: [u8; 32], actual_hmac: [u8; 32]) -> Self {`
- `/app/crates/contextra-types/src/error.rs:511` [fn `contextra_error_display_no_panic`]: `fn contextra_error_display_no_panic() {`
- `/app/crates/contextra-types/src/ids.rs:38` [fn `into_inner`]: `pub fn into_inner(self) -> Vec<u8> {`
- `/app/crates/contextra-types/src/schema.rs:78` [fn `as_u8`]: `pub fn as_u8(self) -> u8 {`
- `/app/crates/contextra-types/src/schema.rs:99` [fn `is_compatible_with_current_build`]: `pub fn is_compatible_with_current_build(self) -> bool {`
- `/app/crates/contextra-types/src/types/budget.rs:262` [fn `try_reserve`]: `pub fn try_reserve(&self, tokens: usize) -> Result<()> {`
- `/app/crates/contextra-types/src/types/saos.rs:79` [fn `strategy_for_signal`]: `pub fn strategy_for_signal(&self, signal_name: &str) -> FusionStrategy {`
- `/app/crates/contextra-types/src/types/saos.rs:388` [fn `text_only`]: `pub fn text_only(text: impl Into<String>) -> Self {`
- `/app/crates/contextra-types/src/types/saos.rs:410` [struct `ScoredEntry`]: `pub struct ScoredEntry {`
- `/app/crates/contextra-types/src/types/saos.rs:549` [fn `with_fusion_strategy`]: `pub fn with_fusion_strategy(mut self, strategy: impl Into<SignalFusionStrategies>) -> Self {`
- ... und 23 weitere Kandidaten.
### `contextra-vector`

- `/app/crates/contextra-vector/src/candidate_stream.rs:69` [fn `with_batch_size`]: `pub fn with_batch_size(mut self, n: usize) -> Self {`
- `/app/crates/contextra-vector/src/candidate_stream.rs:113` [fn `fetched_depth`]: `pub fn fetched_depth(&self) -> usize {`
- `/app/crates/contextra-vector/src/candidate_stream.rs:118` [fn `is_exhausted`]: `pub fn is_exhausted(&self) -> bool {`
- `/app/crates/contextra-vector/src/persistence/header.rs:227` [fn `q_range`]: `pub fn q_range(&self) -> (f32, f32) {`
- `/app/crates/contextra-vector/src/quantize/mod.rs:257` [fn `check_drift`]: `pub fn check_drift(&self, vector: &[f32]) -> f32 {`
- `/app/crates/contextra-vector/src/acorn/naive_reference.rs:35` [fn `from_vectors`]: `pub fn from_vectors(vectors: Vec<(DocId, Vec<f32>)>) -> Self {`
- `/app/crates/contextra-vector/src/hnsw/types.rs:355` [fn `set_fault_injection_insert_target`]: `pub fn set_fault_injection_insert_target(&self, target: u64) {`
- `/app/crates/contextra-vector/src/hnsw/types.rs:478` [fn `all_doc_ids_from_map`]: `pub fn all_doc_ids_from_map(&self) -> Vec<DocId> {`
### `contextra-wire`

- `/app/crates/contextra-wire/src/adapter.rs:71` [fn `into_inner`]: `pub fn into_inner(self) -> Bytes {`
