## contextra-testkit

Pfad: `crates/contextra-testkit`

Abhängigkeiten (direkt): contextra-ports, contextra-types, bytes, parking_lot, tracing

### Module

- `/src/conformance/clock.rs`
- `/src/conformance/failure.rs`
- `/src/conformance/graph.rs`
- `/src/conformance/kv_ports.rs`
- `/src/conformance/macros.rs`
- `/src/conformance/metrics.rs`
- `/src/conformance/mod.rs`
- `/src/conformance/rng.rs`
- `/src/conformance/routing_ports.rs`
- `/src/conformance/storage.rs`
- `/src/conformance/text.rs`
- `/src/conformance/vector.rs`
- `/src/conformance/vfs.rs`
- `/src/fault_vfs.rs`
- `/src/in_memory_graph.rs`
- `/src/in_memory_store.rs`
- `/src/in_memory_text.rs`
- `/src/in_memory_vector.rs`
- `/src/lib.rs`
- `/src/manual_clock.rs`
- `/src/mocks.rs`
- `/src/reference_model.rs`

### Öffentliche Typen

- **BudgetTruncatingContextPreparer** (struct) – src/mocks.rs:557
- **ConformanceFailure** (struct) – src/conformance/failure.rs:20
- **ConformanceResult** (type) – src/conformance/failure.rs:61
- **FaultConfig** (struct) – src/fault_vfs.rs:39
- **FaultVfs** (struct) – src/fault_vfs.rs:61
- **InMemoryGraphIndex** (struct) – src/in_memory_graph.rs:48
- **InMemoryKvBridgeStorage** (struct) – src/mocks.rs:326
- **InMemoryKvPrefixStore** (struct) – src/mocks.rs:415
- **InMemoryStorageEngine** (struct) – src/in_memory_store.rs:64
- **InMemoryTextIndex** (struct) – src/in_memory_text.rs:41
- **InMemoryVectorIndex** (struct) – src/in_memory_vector.rs:51
- **ManualClock** (struct) – src/manual_clock.rs:41
- **MetricEvent** (enum) – src/mocks.rs:100
- **MockEmbedder** (struct) – src/mocks.rs:35
- **RefOp** (enum) – src/reference_model.rs:31
- **ReferenceModel** (struct) – src/reference_model.rs:50
- **ScriptedAttentionExporter** (struct) – src/mocks.rs:385
- **ScriptedEgressPolicy** (struct) – src/mocks.rs:650
- **ScriptedHybridSearchProvider** (struct) – src/mocks.rs:486
- **StaticCommunityResolver** (struct) – src/mocks.rs:529
- **StaticDriftStatusProvider** (struct) – src/mocks.rs:610
- **TestMetricsSink** (struct) – src/mocks.rs:136

### Öffentliche Funktionen

- `pub async fn assert_community_resolver_conformance<R: CommunityResolver>(`  – src/conformance/routing_ports.rs:434
- `pub async fn assert_hybrid_search_provider_conformance<P: HybridSearchProvider>(provider: &P) `  – src/conformance/routing_ports.rs:425
- `pub async fn assert_kv_bridge_storage_conformance<F, Fut, S>(factory: F)`  – src/conformance/kv_ports.rs:472
- `pub async fn run_community_resolver_suite<R: CommunityResolver>(`  – src/conformance/routing_ports.rs:163
- `pub async fn run_graph_index_suite<F, Fut, G>(factory: F) -> ConformanceResult`  – src/conformance/graph.rs:30
- `pub async fn run_hybrid_search_provider_suite<P: HybridSearchProvider>(`  – src/conformance/routing_ports.rs:36
- `pub async fn run_kv_bridge_storage_suite<F, Fut, S>(factory: F) -> ConformanceResult`  – src/conformance/kv_ports.rs:227
- `pub async fn run_storage_engine_suite<F, Fut, S>(factory: F) -> ConformanceResult`  – src/conformance/storage.rs:38
- `pub async fn run_text_index_suite<F, Fut, T>(factory: F) -> ConformanceResult`  – src/conformance/text.rs:30
- `pub async fn run_vector_index_suite<F, Fut, V>(factory: F) -> ConformanceResult`  – src/conformance/vector.rs:36
- `pub fn add_deny_pattern(&self, pattern: impl Into<String>) `  – src/mocks.rs:662
- `pub fn add_redact_pattern(&self, pattern: impl Into<String>) `  – src/mocks.rs:667
- `pub fn advance(&self, duration: Duration) `  – src/manual_clock.rs:78
- `pub fn assert_attention_exporter_conformance<E: AttentionExporter>(`  – src/conformance/kv_ports.rs:487
- `pub fn assert_context_preparer_conformance<P: ContextPreparer>(preparer: &P) `  – src/conformance/routing_ports.rs:447
- `pub fn assert_drift_status_provider_conformance<D: DriftStatusProvider>(provider: &D) `  – src/conformance/routing_ports.rs:457
- `pub fn assert_egress_policy_conformance<P: EgressPolicy>(`  – src/conformance/routing_ports.rs:467
- `pub fn assert_kv_prefix_store_conformance<S: KvPrefixStore>(store: &S) `  – src/conformance/kv_ports.rs:463
- `pub fn clear(&self) `  – src/mocks.rs:161
- `pub fn commit(&mut self) -> u64 `  – src/reference_model.rs:103
- `pub fn commit_batch(&mut self, ops: Vec<RefOp>) -> u64 `  – src/reference_model.rs:133
- `pub fn delete(&mut self, key: &[u8]) `  – src/reference_model.rs:82
- `pub fn events(&self) -> Vec<MetricEvent> `  – src/mocks.rs:152
- `pub fn get_at(&self, key: &[u8], seq: u64) -> Option<Vec<u8>> `  – src/reference_model.rs:153
- `pub fn get_latest(&self, key: &[u8]) -> Option<Vec<u8>> `  – src/reference_model.rs:216
- `pub fn insert(&self, entity_id: EntityId, community_id: u64) `  – src/mocks.rs:540
- `pub fn is_empty(&self) -> bool `  – src/mocks.rs:343
- `pub fn len(&self) -> usize `  – src/mocks.rs:338
- `pub fn mint_witness_for_tests(`  – src/lib.rs:131
- `pub fn new() -> Self `  – src/fault_vfs.rs:79
- `pub fn new() -> Self `  – src/in_memory_graph.rs:69
- `pub fn new() -> Self `  – src/in_memory_store.rs:89
- `pub fn new() -> Self `  – src/in_memory_text.rs:63
- `pub fn new() -> Self `  – src/mocks.rs:144
- `pub fn new() -> Self `  – src/mocks.rs:333
- `pub fn new() -> Self `  – src/mocks.rs:391
- `pub fn new() -> Self `  – src/mocks.rs:421
- `pub fn new() -> Self `  – src/mocks.rs:492
- `pub fn new() -> Self `  – src/mocks.rs:535
- `pub fn new() -> Self `  – src/mocks.rs:561
- `pub fn new() -> Self `  – src/mocks.rs:624
- `pub fn new() -> Self `  – src/mocks.rs:657
- `pub fn new() -> Self `  – src/reference_model.rs:63
- `pub fn new(`  – src/conformance/failure.rs:32
- `pub fn new(dim: usize) -> Self `  – src/in_memory_vector.rs:65
- `pub fn new(dim: usize) -> Self `  – src/mocks.rs:47
- `pub fn new(initial_nanos: u64) -> Self `  – src/manual_clock.rs:62
- `pub fn now_nanos(&self) -> u64 `  – src/manual_clock.rs:95
- `pub fn now_secs_f64(&self) -> f64 `  – src/manual_clock.rs:101
- `pub fn now_system_time(&self) -> SystemTime `  – src/manual_clock.rs:107
- `pub fn put(&mut self, key: &[u8], value: &[u8]) `  – src/reference_model.rs:73
- `pub fn read_count(&self) -> usize `  – src/fault_vfs.rs:229
- `pub fn read_file(&self, path: &Path) -> io::Result<Vec<u8>> `  – src/fault_vfs.rs:188
- `pub fn remove_weights(&self, request_id: RequestId) -> Option<Vec<f32>> `  – src/mocks.rs:401
- `pub fn reset_counters(&self) `  – src/fault_vfs.rs:105
- `pub fn rollback(&mut self) `  – src/reference_model.rs:89
- `pub fn run_attention_exporter_suite<E: AttentionExporter>(`  – src/conformance/kv_ports.rs:405
- `pub fn run_clock_suite<C: Clock>(clock: &C) -> ConformanceResult `  – src/conformance/clock.rs:24
- `pub fn run_context_preparer_suite<P: ContextPreparer>(preparer: &P) -> ConformanceResult `  – src/conformance/routing_ports.rs:220
- `pub fn run_drift_status_provider_suite<D: DriftStatusProvider>(provider: &D) -> ConformanceResult `  – src/conformance/routing_ports.rs:344
- `pub fn run_egress_policy_suite<P: EgressPolicy>(`  – src/conformance/routing_ports.rs:370
- `pub fn run_kv_prefix_store_suite<S: KvPrefixStore>(store: &S) -> ConformanceResult `  – src/conformance/kv_ports.rs:63
- `pub fn run_metrics_sink_suite<M: MetricsSink>(sink: &M) -> ConformanceResult `  – src/conformance/metrics.rs:22
- `pub fn run_rng_suite<R: Rng>(rng: &R) -> ConformanceResult `  – src/conformance/rng.rs:26
- `pub fn run_vfs_suite<V: Vfs>(vfs: &V) -> ConformanceResult `  – src/conformance/vfs.rs:24
- `pub fn scan_prefix_at(&self, prefix: &[u8], seq: u64) -> Vec<(Vec<u8>, Vec<u8>)> `  – src/reference_model.rs:170
- `pub fn scan_prefix_latest(&self, prefix: &[u8]) -> Vec<(Vec<u8>, Vec<u8>)> `  – src/reference_model.rs:222
- `pub fn scan_range_at(`  – src/reference_model.rs:189
- `pub fn set_chunks(&self, mut chunks: Vec<ContextChunk>) `  – src/mocks.rs:497
- `pub fn set_config(&self, config: FaultConfig) `  – src/fault_vfs.rs:88
- `pub fn set_nanos(&self, nanos: u64) `  – src/manual_clock.rs:89
- `pub fn set_status(&self, status: impl Into<String>) `  – src/mocks.rs:636
- `pub fn set_weights(&self, request_id: RequestId, weights: Vec<f32>) `  – src/mocks.rs:396
- `pub fn snapshot_map_at(&self, seq: u64) -> BTreeMap<Vec<u8>, Vec<u8>> `  – src/reference_model.rs:228
- `pub fn snapshot_seq(&self) -> u64 `  – src/reference_model.rs:140
- `pub fn sync(&self) -> io::Result<()> `  – src/fault_vfs.rs:205
- `pub fn sync_count(&self) -> usize `  – src/fault_vfs.rs:234
- `pub fn trigger_crash(&self) `  – src/fault_vfs.rs:97
- `pub fn with_fixed_output(fixed_output: Vec<f32>) -> Self `  – src/mocks.rs:58
- `pub fn with_status(status: impl Into<String>) -> Self `  – src/mocks.rs:629
- `pub fn write_count(&self) -> usize `  – src/fault_vfs.rs:224
- `pub fn write_file(&self, path: &Path, data: &[u8]) -> io::Result<()> `  – src/fault_vfs.rs:176

---
