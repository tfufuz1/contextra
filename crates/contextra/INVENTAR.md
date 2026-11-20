## contextra

Pfad: `crates/contextra`

Abhängigkeiten (direkt): contextra-types, contextra-ports, contextra-db, serde, serde_json, uuid, tokio, thiserror, tracing, zeroize

### Module

- `/src/agent_memory.rs`
- `/src/boot.rs`
- `/src/builder.rs`
- `/src/collection_profile.rs`
- `/src/lib.rs`
- `/src/performance_profile.rs`
- `/src/query_rewriter.rs`

### Öffentliche Typen

- **AgentMemory** (struct) – src/agent_memory.rs:158
- **BootPhase** (enum) – src/boot.rs:15
- **CollectionProfile** (struct) – src/collection_profile.rs:83
- **CollectionProfileError** (enum) – src/collection_profile.rs:93
- **ContextraBuilder** (struct) – src/builder.rs:83
- **ContextraConfigExt** (trait) – src/builder.rs:64
- **DeploymentTier** (enum) – src/collection_profile.rs:139
- **DurabilityConfigError** (enum) – src/performance_profile.rs:53
- **DurabilityMode** (enum) – src/performance_profile.rs:39
- **EncryptionMode** (enum) – src/builder.rs:46
- **KvDeleteMode** (enum) – src/collection_profile.rs:40
- **LlmQueryRewriter** (struct) – src/query_rewriter.rs:40
- **LsmTuning** (struct) – src/collection_profile.rs:71
- **Memory** (struct) – src/agent_memory.rs:124
- **MemoryId** (struct) – src/agent_memory.rs:70
- **PerformanceProfile** (enum) – src/performance_profile.rs:102
- **PerformanceProfileError** (enum) – src/performance_profile.rs:128
- **RelationId** (struct) – src/agent_memory.rs:113
- **ResolvedProfileConfig** (struct) – src/performance_profile.rs:118

### Öffentliche Funktionen

- `pub async fn build(self) -> Result<Contextra, ContextraError> `  – src/builder.rs:299
- `pub async fn create_test_db(`  – tests/common/mod.rs:22
- `pub async fn forget(&self, id: &MemoryId) -> Result<(), ContextraError> `  – src/agent_memory.rs:266
- `pub async fn open(path: impl AsRef<std::path::Path>) -> Result<Contextra, ContextraError> `  – src/lib.rs:128
- `pub async fn open_with_config(`  – src/lib.rs:160
- `pub async fn recall(&self, query: &str, k: usize) -> Result<Vec<Memory>, ContextraError> `  – src/agent_memory.rs:254
- `pub async fn relate(`  – src/agent_memory.rs:280
- `pub async fn relate_n_ary(`  – src/agent_memory.rs:313
- `pub async fn remember(`  – src/agent_memory.rs:228
- `pub async fn rewrite_query(`  – src/query_rewriter.rs:84
- `pub fn as_str(&self) -> &str `  – src/agent_memory.rs:79
- `pub fn builder(dimension: usize) -> ContextraBuilder `  – src/lib.rs:86
- `pub fn dummy_metadata(tag: &str, sequence: usize) -> Value `  – tests/common/mod.rs:41
- `pub fn dummy_vector(dimension: usize, value: f32) -> Vec<f32> `  – tests/common/mod.rs:35
- `pub fn enforce_license(&self, _gate: &dyn LicenseGate) -> Result<(), PerformanceProfileError> `  – src/performance_profile.rs:177
- `pub fn engine(&self) -> &Contextra `  – src/agent_memory.rs:200
- `pub fn execute_boot_sequence(`  – src/boot.rs:39
- `pub fn is_encrypted(&self) -> bool `  – src/builder.rs:212
- `pub fn new(dimension: usize) -> Self `  – src/builder.rs:133
- `pub fn new(engine: Contextra) -> Self `  – src/agent_memory.rs:166
- `pub fn new(generator: Arc<dyn LlmTextGenerator>) -> Self `  – src/query_rewriter.rs:53
- `pub fn new(id: impl Into<String>) -> Self `  – src/agent_memory.rs:74
- `pub fn new_arc(engine: Arc<Contextra>) -> Self `  – src/agent_memory.rs:175
- `pub fn new_with_id_gen(engine: Contextra, id_gen: Arc<dyn IdGen>) -> Self `  – src/agent_memory.rs:184
- `pub fn resolve(self) -> CollectionProfile `  – src/collection_profile.rs:151
- `pub fn resolve(self) -> ResolvedProfileConfig `  – src/performance_profile.rs:146
- `pub fn validate(&self) -> Result<(), CollectionProfileError> `  – src/collection_profile.rs:110
- `pub fn validate(&self) -> Result<(), PerformanceProfileError> `  – src/performance_profile.rs:187
- `pub fn validate_against_features(`  – src/performance_profile.rs:70
- `pub fn validate_with_license(`  – src/collection_profile.rs:126
- `pub fn with_config(mut self, mut config: ContextraConfig) -> Self `  – src/builder.rs:269
- `pub fn with_consolidation(mut self, enabled: bool, interval: Duration) -> Self `  – src/builder.rs:229
- `pub fn with_distance_metric(mut self, metric: DistanceMetric) -> Self `  – src/builder.rs:173
- `pub fn with_embedder(mut self, embedder: Arc<dyn TextEmbeddingEngine>) -> Self `  – src/builder.rs:237
- `pub fn with_embedding_backend(mut self, backend: EmbeddingBackend) -> Self `  – src/builder.rs:218
- `pub fn with_encryption(mut self, mode: EncryptionMode) -> Self `  – src/builder.rs:183
- `pub fn with_encryption_passphrase(self, passphrase: impl Into<String>) -> Self `  – src/builder.rs:205
- `pub fn with_id_gen(mut self, id_gen: Arc<dyn IdGen>) -> Self `  – src/agent_memory.rs:193
- `pub fn with_license_gate(self, _gate: Arc<dyn LicenseGate>) -> Self `  – src/builder.rs:248
- `pub fn with_max_context_results(mut self, n: usize) -> Self `  – src/query_rewriter.rs:71
- `pub fn with_max_elements(mut self, max_elements: usize) -> Self `  – src/builder.rs:163
- `pub fn with_max_snippet_chars(mut self, n: usize) -> Self `  – src/query_rewriter.rs:78
- `pub fn with_max_subqueries(mut self, n: usize) -> Self `  – src/query_rewriter.rs:64
- `pub fn with_performance_profile(`  – src/builder.rs:257
- `pub fn with_storage_path(mut self, path: impl Into<PathBuf>) -> Self `  – src/builder.rs:153

---
