## contextra-db

Pfad: `crates/contextra-db`

Abhängigkeiten (direkt): contextra-ids, contextra-env, contextra-types, contextra-ports, contextra-privacy, contextra-erasure, contextra-crypto, contextra-store, contextra-vector, contextra-text, contextra-graph, contextra-rank, contextra-version, contextra-checkpoint, contextra-durable-fs, tokio, thiserror

### Module

- `/src/erasure.rs`
- `/src/lib.rs`
- `/src/lineage.rs`
- `/src/storage.rs`

### Öffentliche Typen

- **DerivedRef** (struct) – src/lineage.rs:111
- **Document** (struct) – src/storage.rs:15
- **ErasureWiring** (struct) – src/erasure.rs:13
- **GateAudit** (struct) – src/storage.rs:26
- **HybridCandidate** (struct) – src/storage.rs:35
- **LineageIndex** (struct) – src/lineage.rs:122
- **LineageRef** (struct) – src/lineage.rs:84
- **NonEmpty** (struct) – src/lineage.rs:12
- **SourceId** (enum) – src/lineage.rs:31
- **SourceRef** (struct) – src/lineage.rs:49
- **TenantScopedStorage** (struct) – src/storage.rs:44

### Öffentliche Funktionen

- `pub async fn erase(`  – src/erasure.rs:30
- `pub fn as_slice(&self) -> &[T] `  – src/lineage.rs:26
- `pub fn canonical(mut sources: Vec<SourceRef>) -> Result<Self> `  – src/lineage.rs:102
- `pub fn chain(&self) -> &Arc<Chain> `  – src/erasure.rs:40
- `pub fn compaction_range(&self, start: DocId, end: DocId) -> RangeGate `  – src/storage.rs:149
- `pub fn derived_of(&self, source: &SourceRef) -> Vec<DerivedRef> `  – src/lineage.rs:133
- `pub fn documents_for_subject(&self, subject: &SubjectRef) -> Vec<DocId> `  – src/storage.rs:107
- `pub fn get(&self, doc: DocId, audit: Option<GateAudit>) -> Result<Option<&Document>> `  – src/storage.rs:114
- `pub fn get_at_seq(&self, doc: DocId, _seq: u64, audit: Option<GateAudit>) -> Result<Option<&Document>> `  – src/storage.rs:120
- `pub fn hybrid_search(&self, mut candidates: Vec<HybridCandidate>, k: usize, audit: Option<GateAudit>) -> Result<Vec<HybridCandidate>> `  – src/storage.rs:134
- `pub fn insert(&mut self, lineage: &LineageRef, derived: DerivedRef) `  – src/lineage.rs:126
- `pub fn lock_for_erasure(&mut self, doc: DocId) `  – src/storage.rs:104
- `pub fn new(chain: Arc<Chain>) -> Self `  – src/erasure.rs:19
- `pub fn new(tenant: TenantId, collection: CollectionId, gate: Arc<dyn ReadGate>) -> Self `  – src/storage.rs:55
- `pub fn new(values: Vec<T>) -> Result<Self> `  – src/lineage.rs:16
- `pub fn recorded() -> Self `  – src/storage.rs:30
- `pub fn recover_before_reads(&self) -> Result<(), ErasureError> `  – src/erasure.rs:24
- `pub fn remember_d(&mut self, doc: DocId, payload: Vec<u8>, lineage: LineageRef) -> Result<()> `  – src/storage.rs:79
- `pub fn remember_k(&mut self, doc: DocId, payload: Validated<Vec<u8>>) -> Result<()> `  – src/storage.rs:67
- `pub fn remember_p(&mut self, doc: DocId, subject: &SubjectRef, payload: Vec<u8>) -> Result<()> `  – src/storage.rs:72
- `pub fn remove(&mut self, derived: DerivedRef) `  – src/lineage.rs:138
- `pub fn scan(&self, audit: Option<GateAudit>) -> Result<Vec<(DocId, &Document)>> `  – src/storage.rs:125
- `pub fn try_new(sources: Vec<SourceRef>) -> Result<Self> `  – src/lineage.rs:91
- `pub(crate) fn remember_e(&mut self, doc: DocId, payload: Vec<u8>) -> Result<()> `  – src/storage.rs:84

---
