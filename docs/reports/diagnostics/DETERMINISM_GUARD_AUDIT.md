# Determinismus Audit Report (P28)

> Generiert durch `tools/determinism_guard.py`.
> Untersucht Nicht-Determinismus in Produktivpfaden (Instant, SystemTime, thread_rng, HashMap/HashSet).

## Crate Übersicht (Produktivpfade)

| Crate | Instant::now | SystemTime::now | thread_rng | Random HashMap/Set | env::var | Gesamt Prod |
|---|---|---|---|---|---|---|
| `contextra` | 0 | 0 | 0 | 2 | 0 | **2** |
| `contextra-adapt` | 0 | 0 | 0 | 4 | 0 | **4** |
| `contextra-agent` | 0 | 0 | 0 | 0 | 0 | **0** |
| `contextra-audit-export` | 0 | 0 | 0 | 0 | 0 | **0** |
| `contextra-avv-generator` | 0 | 0 | 0 | 0 | 0 | **0** |
| `contextra-checkpoint` | 0 | 0 | 0 | 8 | 0 | **8** |
| `contextra-cognition` | 0 | 0 | 0 | 0 | 0 | **0** |
| `contextra-core` | 0 | 0 | 0 | 0 | 0 | **0** |
| `contextra-crypto` | 0 | 0 | 0 | 1 | 0 | **1** |
| `contextra-db` | 4 | 0 | 0 | 1 | 0 | **5** |
| `contextra-durable-fs` | 0 | 0 | 0 | 0 | 0 | **0** |
| `contextra-engine` | 1 | 1 | 0 | 20 | 0 | **22** |
| `contextra-erasure` | 0 | 0 | 0 | 11 | 0 | **11** |
| `contextra-graph` | 0 | 0 | 0 | 151 | 0 | **151** |
| `contextra-infer-candle` | 1 | 0 | 0 | 3 | 0 | **4** |
| `contextra-infer-ollama` | 1 | 0 | 0 | 0 | 0 | **1** |
| `contextra-infer-onnx` | 0 | 0 | 0 | 0 | 1 | **1** |
| `contextra-ingest` | 0 | 0 | 0 | 0 | 0 | **0** |
| `contextra-kvcache` | 3 | 0 | 0 | 3 | 0 | **6** |
| `contextra-license` | 0 | 0 | 0 | 0 | 0 | **0** |
| `contextra-mcp` | 1 | 0 | 2 | 3 | 22 | **28** |
| `contextra-mvcc` | 0 | 0 | 0 | 0 | 0 | **0** |
| `contextra-ports` | 2 | 1 | 0 | 0 | 0 | **3** |
| `contextra-privacy` | 0 | 0 | 0 | 3 | 0 | **3** |
| `contextra-py` | 0 | 0 | 0 | 0 | 1 | **1** |
| `contextra-rank` | 0 | 0 | 0 | 0 | 0 | **0** |
| `contextra-router` | 0 | 0 | 0 | 0 | 0 | **0** |
| `contextra-sandbox` | 4 | 0 | 0 | 0 | 0 | **4** |
| `contextra-simd` | 0 | 0 | 0 | 0 | 0 | **0** |
| `contextra-store` | 7 | 0 | 5 | 17 | 0 | **29** |
| `contextra-sys` | 0 | 0 | 0 | 0 | 0 | **0** |
| `contextra-testkit` | 0 | 0 | 0 | 0 | 0 | **0** |
| `contextra-text` | 1 | 0 | 0 | 31 | 0 | **32** |
| `contextra-types` | 0 | 0 | 0 | 0 | 0 | **0** |
| `contextra-vector` | 0 | 0 | 0 | 17 | 0 | **17** |
| `contextra-wire` | 0 | 0 | 0 | 0 | 0 | **0** |

## Detaillierte Befunde mit Handlungsempfehlungen

### `contextra` (2 Fundstellen)

- `/app/crates/contextra/src/query_rewriter.rs:32` [random_hashset]: `use std::collections::HashSet;`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/app/crates/contextra/src/query_rewriter.rs:248` [random_hashset]: `let mut seen = HashSet::new();`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
### `contextra-adapt` (4 Fundstellen)

- `/app/crates/contextra-adapt/src/flow_thompson.rs:660` [random_hashmap]: `pub propensities: std::collections::HashMap<contextra_types::RetrievalStrategy, f32>,`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/app/crates/contextra-adapt/src/flow_thompson.rs:666` [random_hashmap]: `propensities: std::collections::HashMap<contextra_types::RetrievalStrategy, f32>,`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/app/crates/contextra-adapt/src/flow_thompson.rs:730` [random_hashmap]: `pub probabilities: std::collections::HashMap<contextra_types::RetrievalStrategy, f32>,`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/app/crates/contextra-adapt/src/flow_thompson.rs:736` [random_hashmap]: `probabilities: std::collections::HashMap<contextra_types::RetrievalStrategy, f32>,`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
### `contextra-checkpoint` (8 Fundstellen)

- `/app/crates/contextra-checkpoint/src/guard/tests.rs:8` [random_hashmap]: `use std::collections::{HashMap, HashSet};`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/app/crates/contextra-checkpoint/src/guard/tests.rs:8` [random_hashset]: `use std::collections::{HashMap, HashSet};`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/app/crates/contextra-checkpoint/src/guard/tests.rs:11` [random_hashmap]: `data: Mutex<HashMap<Vec<u8>, Vec<u8>>>,`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/app/crates/contextra-checkpoint/src/guard/tests.rs:12` [random_hashset]: `pinned: Mutex<HashSet<u64>>,`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/app/crates/contextra-checkpoint/src/guard/tests.rs:19` [random_hashmap]: `data: Mutex::new(HashMap::new()),`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/app/crates/contextra-checkpoint/src/guard/tests.rs:20` [random_hashset]: `pinned: Mutex::new(HashSet::new()),`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/app/crates/contextra-checkpoint/src/store/tests.rs:22` [random_hashset]: `pinned: Mutex<HashSet<u64>>,`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/app/crates/contextra-checkpoint/src/store/tests.rs:31` [random_hashset]: `pinned: Mutex::new(HashSet::new()),`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
### `contextra-crypto` (1 Fundstellen)

- `/app/crates/contextra-crypto/src/deletion_proof/creation.rs:65` [random_hashset]: `let mut seen_layers = std::collections::HashSet::new();`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
### `contextra-db` (5 Fundstellen)

- `/app/crates/contextra-db/src/homeostat.rs:17` [instant_now]: `deadline: Instant::now() + timeout,`
  - **Empfehlung:** `contextra_ports::Clock::now_monotonic()`
- `/app/crates/contextra-db/src/homeostat.rs:22` [instant_now]: `Instant::now() >= self.deadline`
  - **Empfehlung:** `contextra_ports::Clock::now_monotonic()`
- `/app/crates/contextra-db/src/homeostat.rs:26` [instant_now]: `self.deadline.saturating_duration_since(Instant::now())`
  - **Empfehlung:** `contextra_ports::Clock::now_monotonic()`
- `/app/crates/contextra-db/src/volatile_vault.rs:289` [instant_now]: `purged_at: Instant::now(),`
  - **Empfehlung:** `contextra_ports::Clock::now_monotonic()`
- `/app/crates/contextra-db/src/multistep.rs:434` [random_hashset]: `let mut seen = std::collections::HashSet::new();`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
### `contextra-engine` (22 Fundstellen)

- `/app/crates/contextra-engine/src/collection/maintenance.rs:172` [random_hashset]: `let indexed_ids: std::collections::HashSet<DocId> =`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/app/crates/contextra-engine/src/collection/maintenance.rs:209` [random_hashset]: `indexed_ids: &std::collections::HashSet<DocId>,`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/app/crates/contextra-engine/src/collection/maintenance.rs:816` [random_hashmap]: `) -> Result<std::collections::HashMap<EntityId, u64>> {`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/app/crates/contextra-engine/src/collection/maintenance.rs:818` [random_hashmap]: `return Ok(std::collections::HashMap::new());`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/app/crates/contextra-engine/src/collection/maintenance.rs:830` [random_hashset]: `let target_set: std::collections::HashSet<&EntityId> = entity_ids.iter().collect();`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/app/crates/contextra-engine/src/collection/maintenance.rs:831` [random_hashmap]: `let mut map = std::collections::HashMap::new();`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/app/crates/contextra-engine/src/collection/maintenance.rs:914` [random_hashmap]: `let mut embeddings = std::collections::HashMap::new();`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/app/crates/contextra-engine/src/collection/maintenance.rs:915` [random_hashmap]: `let mut id_map = std::collections::HashMap::new();`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/app/crates/contextra-engine/src/contextra_impl/lifecycle.rs:251` [instant_now]: `let start_time = std::time::Instant::now();`
  - **Empfehlung:** `contextra_ports::Clock::now_monotonic()`
- `/app/crates/contextra-engine/src/contextra_impl/collections.rs:249` [random_hashset]: `let mut names = std::collections::HashSet::new();`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/app/crates/contextra-engine/src/contextra_impl/collections.rs:280` [random_hashset]: `let mut names = std::collections::HashSet::new();`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/app/crates/contextra-engine/src/background_workers/tests.rs:538` [system_time_now]: `let now_ms = std::time::SystemTime::now()`
  - **Empfehlung:** `contextra_ports::Clock::now_wall_clock()`
- `/app/crates/contextra-engine/src/orchestration/mod.rs:59` [random_hashmap]: `data: std::collections::HashMap<String, String>,`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/app/crates/contextra-engine/src/orchestration/mod.rs:66` [random_hashmap]: `data: std::collections::HashMap::new(),`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/app/crates/contextra-engine/src/collection/search/filtered.rs:91` [random_hashset]: `) -> Result<std::collections::HashSet<DocId>> {`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/app/crates/contextra-engine/src/collection/search/filtered.rs:101` [random_hashset]: `let mut matched = std::collections::HashSet::new();`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/app/crates/contextra-engine/src/collection/search/filtered.rs:127` [random_hashset]: `) -> Result<Option<std::collections::HashSet<DocId>>> {`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/app/crates/contextra-engine/src/collection/search/filtered.rs:141` [random_hashset]: `let mut matched = std::collections::HashSet::new();`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/app/crates/contextra-engine/src/collection/query_builder/builder_exec.rs:288` [random_hashset]: `let mut evaluated_indices = std::collections::HashSet::new();`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/app/crates/contextra-engine/src/collection/crud/links.rs:112` [random_hashset]: `let mut visited = std::collections::HashSet::new();`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/app/crates/contextra-engine/src/collection/search/hybrid/query.rs:559` [random_hashset]: `let mut superseded_targets = std::collections::HashSet::new();`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/app/crates/contextra-engine/src/collection/search/hybrid/mod.rs:351` [random_hashset]: `let mut visited = std::collections::HashSet::new();`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
### `contextra-erasure` (11 Fundstellen)

- `/app/crates/contextra-erasure/src/deletion_proof.rs:546` [random_hashset]: `let mut seen_layers = std::collections::HashSet::new();`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/app/crates/contextra-erasure/src/saga/core.rs:41` [random_hashmap]: `use std::collections::HashMap;`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/app/crates/contextra-erasure/src/saga/core.rs:92` [random_hashmap]: `pub layer_priorities: HashMap<[u8; 16], PurgePriority>,`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/app/crates/contextra-erasure/src/saga/core.rs:172` [random_hashmap]: `active_sagas: Arc<RwLock<HashMap<ErasureId, ActiveEntry>>>,`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/app/crates/contextra-erasure/src/saga/core.rs:173` [random_hashmap]: `open_keys: Arc<RwLock<HashMap<[u8; 32], ErasureId>>>,`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/app/crates/contextra-erasure/src/saga/core.rs:182` [random_hashmap]: `active_sagas: Arc::new(RwLock::new(HashMap::new())),`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/app/crates/contextra-erasure/src/saga/core.rs:183` [random_hashmap]: `open_keys: Arc::new(RwLock::new(HashMap::new())),`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/app/crates/contextra-erasure/src/saga/core.rs:330` [random_hashmap]: `active_sagas: Arc<RwLock<HashMap<ErasureId, ActiveEntry>>>,`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/app/crates/contextra-erasure/src/saga/core.rs:331` [random_hashmap]: `open_keys: Arc<RwLock<HashMap<[u8; 32], ErasureId>>>,`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/app/crates/contextra-erasure/src/saga/core.rs:587` [random_hashmap]: `active_sagas: Arc<RwLock<HashMap<ErasureId, ActiveEntry>>>,`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/app/crates/contextra-erasure/src/saga/core.rs:588` [random_hashmap]: `open_keys: Arc<RwLock<HashMap<[u8; 32], ErasureId>>>,`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
### `contextra-graph` (151 Fundstellen)

- `/app/crates/contextra-graph/src/community.rs:24` [random_hashmap]: `use std::collections::HashMap;`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/app/crates/contextra-graph/src/community.rs:236` [random_hashmap]: `let mut adj_raw: HashMap<usize, HashMap<usize, f32>> = HashMap::new();`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/app/crates/contextra-graph/src/community.rs:264` [random_hashmap]: `let mut virtual_map: HashMap<crate::hyperedge::HyperEdgeId, usize> = HashMap::new();`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/app/crates/contextra-graph/src/community.rs:329` [random_hashmap]: `let mut global_to_local: HashMap<usize, usize> = HashMap::with_capacity(num_total_nodes);`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/app/crates/contextra-graph/src/community.rs:347` [random_hashmap]: `let entity_to_local: HashMap<EntityId, usize> = local_entity_ids`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/app/crates/contextra-graph/src/community.rs:360` [random_hashmap]: `let mut vnode_to_local: HashMap<crate::hyperedge::HyperEdgeId, usize> = HashMap::new();`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/app/crates/contextra-graph/src/community.rs:417` [random_hashmap]: `let mut community_degrees: HashMap<u64, f32> = HashMap::new();`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/app/crates/contextra-graph/src/community.rs:444` [random_hashmap]: `let mut comm_weights: HashMap<u64, f32> = HashMap::new();`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/app/crates/contextra-graph/src/community.rs:490` [random_hashmap]: `let mut comm_members: HashMap<u64, Vec<usize>> = HashMap::new();`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/app/crates/contextra-graph/src/community.rs:504` [random_hashmap]: `let mut sub_comm: HashMap<usize, u64> = HashMap::new();`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/app/crates/contextra-graph/src/community.rs:505` [random_hashmap]: `let mut sub_degrees: HashMap<u64, f32> = HashMap::new();`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/app/crates/contextra-graph/src/community.rs:528` [random_hashmap]: `let mut sub_weights: HashMap<u64, f32> = HashMap::new();`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/app/crates/contextra-graph/src/community.rs:577` [random_hashmap]: `let mut sub_members: HashMap<u64, Vec<usize>> = HashMap::new();`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/app/crates/contextra-graph/src/percolation.rs:8` [random_hashmap]: `use std::collections::HashMap;`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/app/crates/contextra-graph/src/percolation.rs:81` [random_hashmap]: `embeddings: &HashMap<EntityId, Vec<f32>>,`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/app/crates/contextra-graph/src/percolation.rs:93` [random_hashmap]: `let mut neighbors_cache: HashMap<EntityId, std::collections::HashSet<EntityId>> =`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/app/crates/contextra-graph/src/percolation.rs:93` [random_hashset]: `let mut neighbors_cache: HashMap<EntityId, std::collections::HashSet<EntityId>> =`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/app/crates/contextra-graph/src/percolation.rs:94` [random_hashmap]: `HashMap::new();`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/app/crates/contextra-graph/src/percolation.rs:114` [random_hashset]: `e.insert(std::collections::HashSet::new());`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/app/crates/contextra-graph/src/percolation.rs:126` [random_hashset]: `e.insert(std::collections::HashSet::new());`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/app/crates/contextra-graph/src/session_dag.rs:17` [random_hashmap]: `use std::collections::HashMap;`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/app/crates/contextra-graph/src/session_dag.rs:33` [random_hashmap]: `_nodes_guard: RwLockReadGuard<'a, HashMap<NodeIdx, AgentStateNode>>,`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/app/crates/contextra-graph/src/session_dag.rs:38` [random_hashmap]: `pub fn nodes(&self) -> &HashMap<NodeIdx, AgentStateNode> {`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/app/crates/contextra-graph/src/session_dag.rs:70` [random_hashmap]: `_nodes_guard: RwLockWriteGuard<'a, HashMap<NodeIdx, AgentStateNode>>,`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/app/crates/contextra-graph/src/session_dag.rs:75` [random_hashmap]: `pub fn nodes(&self) -> &HashMap<NodeIdx, AgentStateNode> {`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/app/crates/contextra-graph/src/session_dag.rs:80` [random_hashmap]: `pub fn nodes_mut(&mut self) -> &mut HashMap<NodeIdx, AgentStateNode> {`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/app/crates/contextra-graph/src/session_dag.rs:163` [random_hashmap]: `nodes: RwLock<HashMap<NodeIdx, AgentStateNode>>,`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/app/crates/contextra-graph/src/session_dag.rs:181` [random_hashmap]: `let mut nodes = HashMap::new();`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/app/crates/contextra-graph/src/session_dag.rs:346` [random_hashset]: `let mut visited = std::collections::HashSet::new();`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/app/crates/contextra-graph/src/session_dag.rs:469` [random_hashmap]: `let mut nodes = HashMap::new();`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/app/crates/contextra-graph/src/consistency_enforcement.rs:22` [random_hashmap]: `use std::collections::HashMap;`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/app/crates/contextra-graph/src/consistency_enforcement.rs:94` [random_hashmap]: `patterns: HashMap<[u8; 32], ConflictPattern>,`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/app/crates/contextra-graph/src/consistency_enforcement.rs:107` [random_hashmap]: `patterns: HashMap::new(),`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/app/crates/contextra-graph/src/ppr_stream.rs:15` [random_hashset]: `use std::collections::{HashSet, VecDeque};`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/app/crates/contextra-graph/src/ppr_stream.rs:145` [random_hashset]: `let seed_set: HashSet<EntityId> = self.seeds.iter().copied().collect();`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/app/crates/contextra-graph/src/provider.rs:146` [random_hashmap]: `) -> HashMap<EntityId, f32> {`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/app/crates/contextra-graph/src/provider.rs:150` [random_hashmap]: `return HashMap::new();`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/app/crates/contextra-graph/src/provider.rs:152` [random_hashmap]: `let node_to_idx: HashMap<EntityId, usize> = nodes`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/app/crates/contextra-graph/src/provider.rs:215` [random_hashmap]: `let node_to_idx: HashMap<EntityId, usize> =`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/app/crates/contextra-graph/src/provider.rs:242` [random_hashmap]: `let mut comm_degs: HashMap<u64, f32> = communities`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/app/crates/contextra-graph/src/provider.rs:255` [random_hashmap]: `let mut weights: HashMap<u64, f32> = HashMap::new();`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/app/crates/contextra-graph/src/cascade.rs:194` [random_hashset]: `let node_set: std::collections::HashSet<_> = closure_nodes.iter().copied().collect();`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/app/crates/contextra-graph/src/cascade.rs:196` [random_hashmap]: `let mut in_degree = std::collections::HashMap::new();`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/app/crates/contextra-graph/src/cascade.rs:197` [random_hashmap]: `let mut children_map: std::collections::HashMap<_, Vec<_>> = std::collections::HashMap::new();`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/app/crates/contextra-graph/src/cascade.rs:238` [random_hashset]: `let placed: std::collections::HashSet<_> = topo_order.iter().copied().collect();`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/app/crates/contextra-graph/src/cascade.rs:279` [random_hashset]: `let mut visited = std::collections::HashSet::new();`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/app/crates/contextra-graph/src/cascade.rs:280` [random_hashset]: `let mut on_stack = std::collections::HashSet::new();`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/app/crates/contextra-graph/src/cascade.rs:285` [random_hashset]: `visited: &mut std::collections::HashSet<crate::hyperedge::HyperEdgeId>,`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/app/crates/contextra-graph/src/cascade.rs:286` [random_hashset]: `on_stack: &mut std::collections::HashSet<crate::hyperedge::HyperEdgeId>,`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/app/crates/contextra-graph/src/cascade.rs:405` [random_hashset]: `let mut visited = std::collections::HashSet::new();`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/app/crates/contextra-graph/src/hyperedge.rs:10` [random_hashmap]: `use scc::HashMap;`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/app/crates/contextra-graph/src/hyperedge.rs:272` [random_hashmap]: `forward: HashMap<String, RoleId>,`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/app/crates/contextra-graph/src/hyperedge.rs:273` [random_hashmap]: `backward: HashMap<RoleId, String>,`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/app/crates/contextra-graph/src/hyperedge.rs:357` [random_hashmap]: `forward: HashMap::new(),`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/app/crates/contextra-graph/src/hyperedge.rs:358` [random_hashmap]: `backward: HashMap::new(),`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/app/crates/contextra-graph/src/path_rag/snapshot.rs:9` [random_hashset]: `use std::collections::HashSet;`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/app/crates/contextra-graph/src/path_rag/snapshot.rs:45` [random_hashset]: `let mut seen = HashSet::new();`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/app/crates/contextra-graph/src/path_rag/snapshot.rs:120` [random_hashset]: `let mut seen = HashSet::new();`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/app/crates/contextra-graph/src/path_rag/k_path.rs:10` [random_hashmap]: `use std::collections::{BinaryHeap, HashMap};`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/app/crates/contextra-graph/src/path_rag/k_path.rs:144` [random_hashmap]: `let mut fwd_paths: HashMap<EntityId, Vec<Vec<EntityId>>> = HashMap::new();`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/app/crates/contextra-graph/src/path_rag/k_path.rs:145` [random_hashmap]: `let mut bwd_paths: HashMap<EntityId, Vec<Vec<EntityId>>> = HashMap::new();`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/app/crates/contextra-graph/src/path_rag/mod.rs:27` [random_hashmap]: `use std::collections::{BinaryHeap, HashMap};`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/app/crates/contextra-graph/src/path_rag/mod.rs:290` [random_hashmap]: `let mut dist_fwd: HashMap<EntityId, f32> = HashMap::new();`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/app/crates/contextra-graph/src/path_rag/mod.rs:291` [random_hashmap]: `let mut dist_bwd: HashMap<EntityId, f32> = HashMap::new();`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/app/crates/contextra-graph/src/path_rag/mod.rs:292` [random_hashmap]: `let mut prev_fwd: HashMap<EntityId, (EntityId, f32)> = HashMap::new();`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/app/crates/contextra-graph/src/path_rag/mod.rs:293` [random_hashmap]: `let mut prev_bwd: HashMap<EntityId, (EntityId, f32)> = HashMap::new();`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/app/crates/contextra-graph/src/path_rag/mod.rs:412` [random_hashset]: `let mut targets = std::collections::HashSet::new();`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/app/crates/contextra-graph/src/path_rag/mod.rs:452` [random_hashmap]: `let mut scored: HashMap<u64, f32> = HashMap::new();`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/app/crates/contextra-graph/src/path_rag/tests.rs:10` [random_hashmap]: `edges: HashMap<EntityId, Vec<(EntityId, f32)>>,`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/app/crates/contextra-graph/src/path_rag/tests.rs:15` [random_hashmap]: `let mut map: HashMap<EntityId, Vec<(EntityId, f32)>> = HashMap::new();`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/app/crates/contextra-graph/src/path_rag/tests.rs:193` [random_hashmap]: `edges: HashMap<EntityId, Vec<(EntityId, f32)>>,`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/app/crates/contextra-graph/src/path_rag/tests.rs:194` [random_hashmap]: `hyperedges: HashMap<EntityId, Vec<HyperEdgeId>>,`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/app/crates/contextra-graph/src/path_rag/tests.rs:195` [random_hashmap]: `hyperedge_store: HashMap<HyperEdgeId, HyperEdge>,`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/app/crates/contextra-graph/src/path_rag/tests.rs:200` [random_hashmap]: `let mut map: HashMap<EntityId, Vec<(EntityId, f32)>> = HashMap::new();`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/app/crates/contextra-graph/src/path_rag/tests.rs:205` [random_hashmap]: `let mut he_map: HashMap<EntityId, Vec<HyperEdgeId>> = HashMap::new();`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/app/crates/contextra-graph/src/path_rag/tests.rs:206` [random_hashmap]: `let mut he_store = HashMap::new();`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/app/crates/contextra-graph/src/community/tests.rs:121` [random_hashmap]: `let map: HashMap<u64, u64> = assignments`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/app/crates/contextra-graph/src/community/tests.rs:216` [random_hashmap]: `let map: HashMap<u64, u64> = assignments`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/app/crates/contextra-graph/src/community/tests.rs:319` [random_hashset]: `let expected_ids: std::collections::HashSet<_> = (0..node_count)`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/app/crates/contextra-graph/src/community/tests.rs:337` [random_hashset]: `let assigned_ids: std::collections::HashSet<_> = assignments`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/app/crates/contextra-graph/src/community/tests.rs:796` [random_hashmap]: `let map_false: HashMap<u64, u64> = assignments_false`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/app/crates/contextra-graph/src/community/tests.rs:810` [random_hashmap]: `let map_true: HashMap<u64, u64> = assignments_true`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/app/crates/contextra-graph/src/ppr/dense.rs:10` [random_hashset]: `use std::collections::HashSet;`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/app/crates/contextra-graph/src/ppr/dense.rs:20` [random_hashset]: `let mut seen_seeds = HashSet::new();`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/app/crates/contextra-graph/src/ppr/types.rs:6` [random_hashset]: `use std::collections::HashSet;`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/app/crates/contextra-graph/src/ppr/types.rs:11` [random_hashset]: `deleted_nodes: HashSet<usize>,`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/app/crates/contextra-graph/src/ppr/types.rs:15` [random_hashset]: `pub(crate) fn from_nodes(deleted_nodes: HashSet<usize>) -> Self {`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/app/crates/contextra-graph/src/ppr/snapshot.rs:12` [random_hashmap]: `use std::collections::HashMap;`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/app/crates/contextra-graph/src/ppr/snapshot.rs:117` [random_hashmap]: `let mut snapshot_pending: HashMap<InternalIndex, Vec<EdgePayload>> = HashMap::new();`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/app/crates/contextra-graph/src/ppr/mod.rs:265` [random_hashset]: `let mut seen = std::collections::HashSet::new();`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/app/crates/contextra-graph/src/ppr/mod.rs:315` [random_hashset]: `let mut seen = std::collections::HashSet::new();`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/app/crates/contextra-graph/src/ppr/shadow_hook.rs:32` [random_hashset]: `let mut all_keys = std::collections::HashSet::new();`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/app/crates/contextra-graph/src/csr/graph_persist.rs:10` [random_hashset]: `use std::collections::HashSet;`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/app/crates/contextra-graph/src/csr/graph_persist.rs:81` [random_hashset]: `let mut deleted_entity_ids = HashSet::new();`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/app/crates/contextra-graph/src/csr/path_graph.rs:66` [random_hashset]: `let mut seen = std::collections::HashSet::new();`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/app/crates/contextra-graph/src/csr/path_graph.rs:114` [random_hashset]: `let mut seen = std::collections::HashSet::new();`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/app/crates/contextra-graph/src/csr/graph_write/tombstone.rs:9` [random_hashset]: `use std::collections::HashSet;`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/app/crates/contextra-graph/src/csr/graph_write/tombstone.rs:31` [random_hashset]: `let mut affected_nodes_set = HashSet::new();`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/app/crates/contextra-graph/src/csr/graph_write/mod.rs:13` [random_hashset]: `use std::collections::{HashSet, VecDeque};`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/app/crates/contextra-graph/src/csr/graph_write/mod.rs:204` [random_hashset]: `let mut edges: HashSet<(EntityId, EntityId)> =`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/app/crates/contextra-graph/src/csr/graph_write/hyperedge.rs:9` [random_hashset]: `use std::collections::HashSet;`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/app/crates/contextra-graph/src/csr/graph_write/hyperedge.rs:15` [random_hashset]: `visited: &mut HashSet<crate::hyperedge::HyperEdgeId>,`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/app/crates/contextra-graph/src/csr/graph_write/hyperedge.rs:16` [random_hashset]: `on_stack: &mut HashSet<crate::hyperedge::HyperEdgeId>,`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/app/crates/contextra-graph/src/csr/graph_write/hyperedge.rs:76` [random_hashset]: `let mut visited = HashSet::new();`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/app/crates/contextra-graph/src/csr/graph_write/hyperedge.rs:77` [random_hashset]: `let mut on_stack = HashSet::new();`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/app/crates/contextra-graph/src/csr/graph_write/hyperedge.rs:366` [random_hashset]: `let mut seen_ids = HashSet::with_capacity(new_edges.len());`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/app/crates/contextra-graph/src/csr/inner/estimate.rs:6` [random_hashset]: `use std::collections::HashSet;`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/app/crates/contextra-graph/src/csr/inner/estimate.rs:166` [random_hashset]: `p += table_bytes::<DocId, HashSet<(EntityId, EntityId)>>(cap);`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/app/crates/contextra-graph/src/csr/inner/estimate.rs:187` [random_hashset]: `p += table_bytes::<DocId, HashSet<crate::hyperedge::HyperEdgeId>>(cap);`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/app/crates/contextra-graph/src/csr/inner/estimate.rs:201` [random_hashset]: `p += table_bytes::<crate::hyperedge::HyperEdgeId, HashSet<crate::hyperedge::HyperEdgeId>>(`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/app/crates/contextra-graph/src/csr/inner/estimate.rs:217` [random_hashset]: `p += table_bytes::<EntityId, HashSet<crate::hyperedge::HyperEdgeId>>(cap);`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/app/crates/contextra-graph/src/csr/inner/mod.rs:11` [random_hashmap]: `use std::collections::{HashMap, HashSet};`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/app/crates/contextra-graph/src/csr/inner/mod.rs:11` [random_hashset]: `use std::collections::{HashMap, HashSet};`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/app/crates/contextra-graph/src/csr/inner/mod.rs:25` [random_hashmap]: `pub(crate) id_map: HashMap<EntityId, InternalIndex>,`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/app/crates/contextra-graph/src/csr/inner/mod.rs:28` [random_hashmap]: `pub(crate) communities: HashMap<EntityId, u64>,`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/app/crates/contextra-graph/src/csr/inner/mod.rs:41` [random_hashset]: `pub(crate) doc_to_edges: ahash::AHashMap<DocId, HashSet<(EntityId, EntityId)>>,`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/app/crates/contextra-graph/src/csr/inner/mod.rs:46` [random_hashmap]: `pub(crate) pending_edges: HashMap<InternalIndex, Vec<EdgePayload>>,`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/app/crates/contextra-graph/src/csr/inner/mod.rs:47` [random_hashset]: `pub(crate) tombstoned_edges: HashSet<(InternalIndex, InternalIndex)>,`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/app/crates/contextra-graph/src/csr/inner/mod.rs:51` [random_hashmap]: `pub(crate) edge_store: HashMap<EntityId, Vec<CsrEdge>>,`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/app/crates/contextra-graph/src/csr/inner/mod.rs:53` [random_hashmap]: `pub(crate) hyperedges: HashMap<crate::hyperedge::HyperEdgeId, Arc<crate::hyperedge::HyperEdge>>,`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/app/crates/contextra-graph/src/csr/inner/mod.rs:54` [random_hashset]: `pub(crate) doc_to_hyperedges: ahash::AHashMap<DocId, HashSet<crate::hyperedge::HyperEdgeId>>,`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/app/crates/contextra-graph/src/csr/inner/mod.rs:55` [random_hashset]: `pub(crate) hyperedge_index: ahash::AHashMap<EntityId, HashSet<crate::hyperedge::HyperEdgeId>>,`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/app/crates/contextra-graph/src/csr/inner/mod.rs:57` [random_hashset]: `ahash::AHashMap<crate::hyperedge::HyperEdgeId, HashSet<crate::hyperedge::HyperEdgeId>>,`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/app/crates/contextra-graph/src/csr/inner/mod.rs:75` [random_hashmap]: `id_map: HashMap::new(),`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/app/crates/contextra-graph/src/csr/inner/mod.rs:78` [random_hashmap]: `communities: HashMap::new(),`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/app/crates/contextra-graph/src/csr/inner/mod.rs:93` [random_hashmap]: `pending_edges: HashMap::new(),`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/app/crates/contextra-graph/src/csr/inner/mod.rs:94` [random_hashset]: `tombstoned_edges: HashSet::new(),`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/app/crates/contextra-graph/src/csr/inner/mod.rs:98` [random_hashmap]: `edge_store: HashMap::new(),`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/app/crates/contextra-graph/src/csr/inner/mod.rs:99` [random_hashmap]: `hyperedges: HashMap::new(),`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/app/crates/contextra-graph/src/csr/graph_read/communities.rs:9` [random_hashmap]: `use std::collections::HashMap;`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/app/crates/contextra-graph/src/csr/graph_read/communities.rs:24` [random_hashmap]: `) -> Result<HashMap<EntityId, u64>> {`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/app/crates/contextra-graph/src/csr/graph_read/communities.rs:27` [random_hashmap]: `let mut map = HashMap::with_capacity(entity_ids.len());`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/app/crates/contextra-graph/src/csr/graph_read/communities.rs:55` [random_hashmap]: `let mut map = HashMap::with_capacity(entity_ids.len());`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/app/crates/contextra-graph/src/csr/graph_read/communities.rs:63` [random_hashmap]: `Ok(HashMap::new())`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/app/crates/contextra-graph/src/csr/graph_read/pagerank.rs:9` [random_hashmap]: `use std::collections::{HashMap, HashSet};`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/app/crates/contextra-graph/src/csr/graph_read/pagerank.rs:9` [random_hashset]: `use std::collections::{HashMap, HashSet};`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/app/crates/contextra-graph/src/csr/graph_read/pagerank.rs:17` [random_hashmap]: `) -> HashMap<EntityId, f32> {`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/app/crates/contextra-graph/src/csr/graph_read/pagerank.rs:24` [random_hashmap]: `return HashMap::new();`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/app/crates/contextra-graph/src/csr/graph_read/pagerank.rs:81` [random_hashmap]: `let mut result = HashMap::new();`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/app/crates/contextra-graph/src/csr/graph_read/pagerank.rs:92` [random_hashset]: `pub async fn get_deleted_node_indices(&self) -> HashSet<usize> {`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/app/crates/contextra-graph/src/csr/graph_read/pagerank.rs:94` [random_hashset]: `return HashSet::new();`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/app/crates/contextra-graph/src/csr/graph_read/pagerank.rs:100` [random_hashset]: `let mut deleted_indices = HashSet::new();`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/app/crates/contextra-graph/src/csr/graph_read/mod.rs:89` [random_hashset]: `let mut seen: std::collections::HashSet<EntityId> = std::collections::HashSet::new();`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/app/crates/contextra-graph/src/csr/graph_index/traversal.rs:7` [random_hashmap]: `use std::collections::{HashMap, VecDeque};`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/app/crates/contextra-graph/src/csr/graph_index/traversal.rs:73` [random_hashmap]: `mut visited: HashMap<InternalIndex, f32>,`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/app/crates/contextra-graph/src/csr/graph_index/traversal.rs:96` [random_hashmap]: `visited: &mut HashMap<InternalIndex, f32>,`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/app/crates/contextra-graph/src/csr/graph_index/traversal.rs:167` [random_hashmap]: `) -> HashMap<InternalIndex, f32> {`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/app/crates/contextra-graph/src/csr/graph_index/traversal.rs:168` [random_hashmap]: `let mut visited: HashMap<InternalIndex, f32> = HashMap::new();`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/app/crates/contextra-graph/src/csr/graph_index/traversal.rs:211` [random_hashmap]: `visited: &mut HashMap<InternalIndex, f32>,`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/app/crates/contextra-graph/src/csr/graph_index/traversal.rs:259` [random_hashmap]: `) -> HashMap<InternalIndex, f32> {`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/app/crates/contextra-graph/src/csr/graph_index/traversal.rs:260` [random_hashmap]: `let mut visited: HashMap<InternalIndex, f32> = HashMap::new();`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
### `contextra-infer-candle` (4 Fundstellen)

- `/app/crates/contextra-infer-candle/src/inference/tests.rs:638` [instant_now]: `let start = std::time::Instant::now();`
  - **Empfehlung:** `contextra_ports::Clock::now_monotonic()`
- `/app/crates/contextra-infer-candle/src/model/quantized_llama.rs:21` [random_hashmap]: `use std::collections::HashMap;`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/app/crates/contextra-infer-candle/src/model/quantized_llama.rs:304` [random_hashmap]: `pub masks: HashMap<(usize, usize), Tensor>,`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/app/crates/contextra-infer-candle/src/model/quantized_llama.rs:439` [random_hashmap]: `masks: HashMap::new(),`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
### `contextra-infer-ollama` (1 Fundstellen)

- `/app/crates/contextra-infer-ollama/src/client/tests.rs:927` [instant_now]: `let start = tokio::time::Instant::now();`
  - **Empfehlung:** `contextra_ports::Clock::now_monotonic()`
### `contextra-infer-onnx` (1 Fundstellen)

- `/app/crates/contextra-infer-onnx/src/lib.rs:71` [env_var]: `if let Ok(home) = std::env::var("HOME").or_else(|_| std::env::var("USERPROFILE")) {`
  - **Empfehlung:** `Explizite Konfigurationsübergabe (Config-Struct)`
### `contextra-kvcache` (6 Fundstellen)

- `/app/crates/contextra-kvcache/src/store.rs:139` [instant_now]: `pinned_at: std::time::Instant::now(),`
  - **Empfehlung:** `contextra_ports::Clock::now_monotonic()`
- `/app/crates/contextra-kvcache/src/store.rs:207` [instant_now]: `let now = std::time::Instant::now();`
  - **Empfehlung:** `contextra_ports::Clock::now_monotonic()`
- `/app/crates/contextra-kvcache/src/store.rs:240` [instant_now]: `let base_time = std::time::Instant::now();`
  - **Empfehlung:** `contextra_ports::Clock::now_monotonic()`
- `/app/crates/contextra-kvcache/src/eviction_worker.rs:21` [random_hashmap]: `use std::collections::HashMap;`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/app/crates/contextra-kvcache/src/eviction_worker.rs:35` [random_hashmap]: `segment_map: RwLock<HashMap<(TenantId, u64), RequestId>>,`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/app/crates/contextra-kvcache/src/eviction_worker.rs:42` [random_hashmap]: `segment_map: RwLock::new(HashMap::new()),`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
### `contextra-mcp` (28 Fundstellen)

- `/app/crates/contextra-mcp/src/validation.rs:5` [env_var]: `std::env::var("CONTEXTRA_MCP_ALLOW_WRITE")`
  - **Empfehlung:** `Explizite Konfigurationsübergabe (Config-Struct)`
- `/app/crates/contextra-mcp/src/proof_key_env.rs:95` [env_var]: `let primary = std::env::var("CONTEXTRA_CANONICAL_PROOF_KEY").ok();`
  - **Empfehlung:** `Explizite Konfigurationsübergabe (Config-Struct)`
- `/app/crates/contextra-mcp/src/proof_key_env.rs:96` [env_var]: `let legacy = std::env::var("CONTEXTRA_PROOF_KEY").ok();`
  - **Empfehlung:** `Explizite Konfigurationsübergabe (Config-Struct)`
- `/app/crates/contextra-mcp/src/config.rs:205` [env_var]: `let prefixed = std::env::var("CONTEXTRA_EMBEDDING_PROVIDER").ok();`
  - **Empfehlung:** `Explizite Konfigurationsübergabe (Config-Struct)`
- `/app/crates/contextra-mcp/src/config.rs:206` [env_var]: `let unprefixed = std::env::var("EMBEDDING_PROVIDER").ok();`
  - **Empfehlung:** `Explizite Konfigurationsübergabe (Config-Struct)`
- `/app/crates/contextra-mcp/src/config.rs:219` [env_var]: `let ollama_url = std::env::var("CONTEXTRA_OLLAMA_URL")`
  - **Empfehlung:** `Explizite Konfigurationsübergabe (Config-Struct)`
- `/app/crates/contextra-mcp/src/config.rs:222` [env_var]: `let embed_model = std::env::var("CONTEXTRA_EMBED_MODEL")`
  - **Empfehlung:** `Explizite Konfigurationsübergabe (Config-Struct)`
- `/app/crates/contextra-mcp/src/config.rs:225` [env_var]: `let onnx_model_path = std::env::var("CONTEXTRA_ONNX_MODEL_PATH")`
  - **Empfehlung:** `Explizite Konfigurationsübergabe (Config-Struct)`
- `/app/crates/contextra-mcp/src/config.rs:229` [env_var]: `let candle_model_dir = std::env::var("CONTEXTRA_CANDLE_MODEL_DIR")`
  - **Empfehlung:** `Explizite Konfigurationsübergabe (Config-Struct)`
- `/app/crates/contextra-mcp/src/config.rs:369` [env_var]: `let prefixed = std::env::var("CONTEXTRA_LLM_PROVIDER").ok();`
  - **Empfehlung:** `Explizite Konfigurationsübergabe (Config-Struct)`
- `/app/crates/contextra-mcp/src/config.rs:370` [env_var]: `let unprefixed = std::env::var("LLM_PROVIDER").ok();`
  - **Empfehlung:** `Explizite Konfigurationsübergabe (Config-Struct)`
- `/app/crates/contextra-mcp/src/config.rs:383` [env_var]: `let ollama_url = std::env::var("CONTEXTRA_OLLAMA_URL")`
  - **Empfehlung:** `Explizite Konfigurationsübergabe (Config-Struct)`
- `/app/crates/contextra-mcp/src/config.rs:387` [env_var]: `std::env::var("CONTEXTRA_LLM_MODEL").unwrap_or_else(|_| "llama3.2:3b".to_string());`
  - **Empfehlung:** `Explizite Konfigurationsübergabe (Config-Struct)`
- `/app/crates/contextra-mcp/src/config.rs:389` [env_var]: `let candle_model_dir = std::env::var("CONTEXTRA_CANDLE_MODEL_DIR")`
  - **Empfehlung:** `Explizite Konfigurationsübergabe (Config-Struct)`
- `/app/crates/contextra-mcp/src/config.rs:497` [env_var]: `let profiles_path = std::env::var("CONTEXTRA_ROUTER_PROFILES_PATH")`
  - **Empfehlung:** `Explizite Konfigurationsübergabe (Config-Struct)`
- `/app/crates/contextra-mcp/src/config.rs:501` [env_var]: `let calibration_store_path = std::env::var("CONTEXTRA_ROUTER_CALIBRATION_PATH")`
  - **Empfehlung:** `Explizite Konfigurationsübergabe (Config-Struct)`
- `/app/crates/contextra-mcp/src/config.rs:514` [env_var]: `} else if let Ok(json_str) = std::env::var("CONTEXTRA_ROUTER_PROFILES_JSON") {`
  - **Empfehlung:** `Explizite Konfigurationsübergabe (Config-Struct)`
- `/app/crates/contextra-mcp/src/sandbox.rs:15` [random_hashmap]: `use std::collections::HashMap;`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/app/crates/contextra-mcp/src/sandbox.rs:268` [random_hashmap]: `volatile_results: Mutex<HashMap<String, VolatileToolResult>>,`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/app/crates/contextra-mcp/src/sandbox.rs:286` [random_hashmap]: `volatile_results: Mutex::new(HashMap::new()),`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/app/crates/contextra-mcp/src/sandbox.rs:444` [thread_rng]: `rand::thread_rng().fill_bytes(&mut salt);`
  - **Empfehlung:** `contextra_ports::RngSeed / deterministischer ChaCha20Rng`
- `/app/crates/contextra-mcp/src/sandbox.rs:447` [thread_rng]: `rand::thread_rng().fill_bytes(&mut passphrase);`
  - **Empfehlung:** `contextra_ports::RngSeed / deterministischer ChaCha20Rng`
- `/app/crates/contextra-mcp/src/server.rs:107` [env_var]: `let timeout_secs = std::env::var("CONTEXTRA_MCP_IDLE_TIMEOUT_SECS")`
  - **Empfehlung:** `Explizite Konfigurationsübergabe (Config-Struct)`
- `/app/crates/contextra-mcp/src/prompt_injection/guard.rs:220` [env_var]: `let log_path = std::env::var("CONTEXTRA_MCP_SECURITY_LOG")`
  - **Empfehlung:** `Explizite Konfigurationsübergabe (Config-Struct)`
- `/app/crates/contextra-mcp/src/prompt_injection/guard.rs:225` [env_var]: `if let Ok(config_file) = std::env::var("CONTEXTRA_MCP_INJECTION_CONFIG") {`
  - **Empfehlung:** `Explizite Konfigurationsübergabe (Config-Struct)`
- `/app/crates/contextra-mcp/src/prompt_injection/guard.rs:232` [env_var]: `if let Ok(patterns_file) = std::env::var("CONTEXTRA_MCP_PATTERNS_FILE") {`
  - **Empfehlung:** `Explizite Konfigurationsübergabe (Config-Struct)`
- `/app/crates/contextra-mcp/src/prompt_injection/policy.rs:34` [env_var]: `if let Ok(val) = std::env::var("CONTEXTRA_MCP_QUARANTINE_POLICY") {`
  - **Empfehlung:** `Explizite Konfigurationsübergabe (Config-Struct)`
- `/app/crates/contextra-mcp/src/prompt_injection/tests.rs:226` [instant_now]: `let start = std::time::Instant::now();`
  - **Empfehlung:** `contextra_ports::Clock::now_monotonic()`
### `contextra-ports` (3 Fundstellen)

- `/app/crates/contextra-ports/src/clock.rs:30` [instant_now]: `start_instant: Instant::now(),`
  - **Empfehlung:** `contextra_ports::Clock::now_monotonic()`
- `/app/crates/contextra-ports/src/clock.rs:45` [system_time_now]: `SystemTime::now()`
  - **Empfehlung:** `contextra_ports::Clock::now_wall_clock()`
- `/app/crates/contextra-ports/src/clock.rs:54` [instant_now]: `let nanos = Instant::now()`
  - **Empfehlung:** `contextra_ports::Clock::now_monotonic()`
### `contextra-privacy` (3 Fundstellen)

- `/app/crates/contextra-privacy/src/bulk_exfiltration_detector.rs:19` [random_hashmap]: `use scc::HashMap;`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/app/crates/contextra-privacy/src/bulk_exfiltration_detector.rs:132` [random_hashmap]: `sessions: HashMap<SessionId, Arc<Mutex<SlidingWindowCounter>>>,`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/app/crates/contextra-privacy/src/bulk_exfiltration_detector.rs:171` [random_hashmap]: `sessions: HashMap::new(),`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
### `contextra-py` (1 Fundstellen)

- `/app/crates/contextra-py/src/bindings/runtime_state.rs:42` [env_var]: `std::env::var("CONTEXTRA_WORKER_THREADS")`
  - **Empfehlung:** `Explizite Konfigurationsübergabe (Config-Struct)`
### `contextra-sandbox` (4 Fundstellen)

- `/app/crates/contextra-sandbox/src/executor/mod.rs:89` [instant_now]: `let deadline = tokio::time::Instant::now() + effective_timeout;`
  - **Empfehlung:** `contextra_ports::Clock::now_monotonic()`
- `/app/crates/contextra-sandbox/src/executor/mod.rs:121` [instant_now]: `start_instant: tokio::time::Instant::now(),`
  - **Empfehlung:** `contextra_ports::Clock::now_monotonic()`
- `/app/crates/contextra-sandbox/src/executor/tests.rs:266` [instant_now]: `start_instant: tokio::time::Instant::now(),`
  - **Empfehlung:** `contextra_ports::Clock::now_monotonic()`
- `/app/crates/contextra-sandbox/src/executor/tests.rs:295` [instant_now]: `start_instant: tokio::time::Instant::now(),`
  - **Empfehlung:** `contextra_ports::Clock::now_monotonic()`
### `contextra-store` (29 Fundstellen)

- `/app/crates/contextra-store/src/system_pressure.rs:163` [instant_now]: `let sample_start = std::time::Instant::now();`
  - **Empfehlung:** `contextra_ports::Clock::now_monotonic()`
- `/app/crates/contextra-store/src/scanner/sstable.rs:17` [random_hashset]: `use std::collections::HashSet;`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/app/crates/contextra-store/src/scanner/sstable.rs:117` [random_hashset]: `let mut active_paths = HashSet::new();`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/app/crates/contextra-store/src/lsm/engine.rs:18` [random_hashmap]: `use std::collections::HashMap;`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/app/crates/contextra-store/src/lsm/engine.rs:67` [random_hashmap]: `pub(super) intent_locks: std::sync::Mutex<HashMap<Vec<u8>, TxId>>,`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/app/crates/contextra-store/src/lsm/recovery.rs:34` [thread_rng]: `rand::thread_rng().fill(&mut buf);`
  - **Empfehlung:** `contextra_ports::RngSeed / deterministischer ChaCha20Rng`
- `/app/crates/contextra-store/src/lsm/recovery.rs:246` [random_hashset]: `Option<std::collections::HashSet<std::path::PathBuf>>,`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/app/crates/contextra-store/src/lsm/recovery.rs:247` [random_hashset]: `Option<std::collections::HashSet<std::path::PathBuf>>,`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/app/crates/contextra-store/src/lsm/recovery.rs:266` [random_hashmap]: `let mut pending_tx_map: std::collections::HashMap<u64, Vec<PendingTxOp>> =`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/app/crates/contextra-store/src/lsm/recovery.rs:267` [random_hashmap]: `std::collections::HashMap::new();`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/app/crates/contextra-store/src/lsm/recovery.rs:270` [random_hashset]: `let mut replayed_hmac_set = std::collections::HashSet::new();`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/app/crates/contextra-store/src/lsm/recovery.rs:612` [random_hashmap]: `intent_locks: std::sync::Mutex::new(std::collections::HashMap::new()),`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/app/crates/contextra-store/src/lsm/recovery.rs:734` [random_hashset]: `let txs_to_rollback: std::collections::HashSet<u64> = memtable`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/app/crates/contextra-store/src/compaction/engine.rs:532` [instant_now]: `let mut io_token_last_reset = std::time::Instant::now();`
  - **Empfehlung:** `contextra_ports::Clock::now_monotonic()`
- `/app/crates/contextra-store/src/compaction/engine.rs:561` [instant_now]: `let wait_start = std::time::Instant::now();`
  - **Empfehlung:** `contextra_ports::Clock::now_monotonic()`
- `/app/crates/contextra-store/src/compaction/engine.rs:686` [instant_now]: `io_token_last_reset = std::time::Instant::now();`
  - **Empfehlung:** `contextra_ports::Clock::now_monotonic()`
- `/app/crates/contextra-store/src/compaction/engine.rs:764` [instant_now]: `io_token_last_reset = std::time::Instant::now();`
  - **Empfehlung:** `contextra_ports::Clock::now_monotonic()`
- `/app/crates/contextra-store/src/wal/flusher.rs:242` [instant_now]: `let deadline = tokio::time::Instant::now()`
  - **Empfehlung:** `contextra_ports::Clock::now_monotonic()`
- `/app/crates/contextra-store/src/wal/hmac.rs:268` [thread_rng]: `rand::thread_rng().fill_bytes(&mut key);`
  - **Empfehlung:** `contextra_ports::RngSeed / deterministischer ChaCha20Rng`
- `/app/crates/contextra-store/src/wal/hmac.rs:273` [thread_rng]: `rand::thread_rng().next_u64()`
  - **Empfehlung:** `contextra_ports::RngSeed / deterministischer ChaCha20Rng`
- `/app/crates/contextra-store/src/wal/hmac.rs:389` [thread_rng]: `rand::thread_rng().next_u64()`
  - **Empfehlung:** `contextra_ports::RngSeed / deterministischer ChaCha20Rng`
- `/app/crates/contextra-store/src/wal/hmac.rs:477` [thread_rng]: `rand::thread_rng().next_u64()`
  - **Empfehlung:** `contextra_ports::RngSeed / deterministischer ChaCha20Rng`
- `/app/crates/contextra-store/src/manifest/core.rs:284` [random_hashmap]: `let mut valid_map = std::collections::HashMap::new();`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/app/crates/contextra-store/src/manifest/core.rs:338` [random_hashset]: `) -> std::collections::HashSet<PathBuf> {`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/app/crates/contextra-store/src/manifest/core.rs:340` [random_hashset]: `let valid_set: std::collections::HashSet<PathBuf> =`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/app/crates/contextra-store/src/manifest/core.rs:342` [random_hashset]: `let mut dead_set = std::collections::HashSet::new();`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/app/crates/contextra-store/fuzz/fuzz_targets/fuzz_compaction_interleave.rs:8` [random_hashmap]: `use std::collections::HashMap;`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/app/crates/contextra-store/fuzz/fuzz_targets/fuzz_compaction_interleave.rs:59` [random_hashmap]: `let mut oracle: HashMap<Vec<u8>, Option<Vec<u8>>> = HashMap::new();`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/app/crates/contextra-store/fuzz/fuzz_targets/fuzz_memtable_concurrent.rs:99` [instant_now]: `let start_time = std::time::Instant::now();`
  - **Empfehlung:** `contextra_ports::Clock::now_monotonic()`
### `contextra-text` (32 Fundstellen)

- `/app/crates/contextra-text/src/stream.rs:19` [random_hashset]: `use std::collections::{HashSet, VecDeque};`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/app/crates/contextra-text/src/stream.rs:48` [random_hashset]: `seen: HashSet<DocId>,`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/app/crates/contextra-text/src/stream.rs:73` [random_hashset]: `seen: HashSet::new(),`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/app/crates/contextra-text/src/wand.rs:20` [random_hashmap]: `use std::collections::{BinaryHeap, HashMap};`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/app/crates/contextra-text/src/wand.rs:290` [random_hashmap]: `let mut doc_len_cache: HashMap<DocId, Option<u32>> = HashMap::new();`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/app/crates/contextra-text/src/wand.rs:291` [random_hashmap]: `let mut tbs_cache: HashMap<Vec<u8>, bool> = HashMap::new();`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/app/crates/contextra-text/src/tokenizer.rs:20` [random_hashset]: `use std::collections::HashSet;`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/app/crates/contextra-text/src/tokenizer.rs:25` [random_hashset]: `static STOPWORDS: OnceLock<HashSet<String>> = OnceLock::new();`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/app/crates/contextra-text/src/tokenizer.rs:117` [random_hashset]: `fn get_stopwords() -> &'static HashSet<String> {`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/app/crates/contextra-text/src/posting_list.rs:24` [random_hashmap]: `use std::collections::HashMap;`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/app/crates/contextra-text/src/posting_list.rs:322` [random_hashmap]: `index: RwLock<HashMap<String, Arc<PostingList>>>,`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/app/crates/contextra-text/src/posting_list.rs:330` [random_hashmap]: `index: RwLock::new(HashMap::new()),`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/app/crates/contextra-text/src/domain/mod.rs:18` [random_hashset]: `use std::collections::HashSet;`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/app/crates/contextra-text/src/domain/mod.rs:74` [random_hashset]: `let protected_set: HashSet<&'static str> = protected_vec.iter().copied().collect();`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/app/crates/contextra-text/src/inverted/purge.rs:25` [random_hashset]: `use std::collections::HashSet;`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/app/crates/contextra-text/src/inverted/purge.rs:176` [random_hashset]: `let mut terms_to_check: HashSet<String> = HashSet::new();`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/app/crates/contextra-text/src/inverted/index_struct.rs:4` [random_hashmap]: `use std::collections::HashMap;`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/app/crates/contextra-text/src/inverted/index_struct.rs:25` [random_hashmap]: `pub(crate) staged_stats: Arc<parking_lot::Mutex<HashMap<TxId, StagedStatsChange>>>,`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/app/crates/contextra-text/src/inverted/index_struct.rs:26` [random_hashmap]: `pub(crate) staged_terms: Arc<parking_lot::Mutex<HashMap<TxId, Vec<String>>>>,`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/app/crates/contextra-text/src/inverted/index_struct.rs:84` [random_hashmap]: `staged_stats: Arc::new(parking_lot::Mutex::new(HashMap::new())),`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/app/crates/contextra-text/src/inverted/index_struct.rs:85` [random_hashmap]: `staged_terms: Arc::new(parking_lot::Mutex::new(HashMap::new())),`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/app/crates/contextra-text/src/inverted/index_struct.rs:283` [random_hashmap]: `let mut tfs = HashMap::with_capacity(tokens.len());`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/app/crates/contextra-text/src/morphology/stopwords.rs:3` [random_hashset]: `use std::collections::HashSet;`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/app/crates/contextra-text/src/morphology/stopwords.rs:6` [random_hashset]: `static GERMAN_STOPWORDS: OnceLock<HashSet<String>> = OnceLock::new();`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/app/crates/contextra-text/src/morphology/stopwords.rs:13` [random_hashset]: `pub fn get_german_stopwords() -> &'static HashSet<String> {`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/app/crates/contextra-text/src/morphology/stopwords.rs:22` [random_hashset]: `let mut set = HashSet::with_capacity(words.len());`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/app/crates/contextra-text/src/morphology/trie.rs:10` [random_hashmap]: `pub children: std::collections::HashMap<char, TrieNode>,`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/app/crates/contextra-text/src/morphology/compound_splitter.rs:6` [random_hashset]: `use std::collections::HashSet;`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/app/crates/contextra-text/src/morphology/compound_splitter.rs:78` [random_hashset]: `pub fn with_dictionary(min_len: usize, custom_words: HashSet<String>) -> Self {`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/app/crates/contextra-text/src/morphology/tests.rs:8` [random_hashset]: `use std::collections::HashSet;`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/app/crates/contextra-text/src/morphology/tests.rs:635` [random_hashset]: `let mut custom_words = HashSet::new();`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/app/crates/contextra-text/src/morphology/tests.rs:738` [instant_now]: `let start = std::time::Instant::now();`
  - **Empfehlung:** `contextra_ports::Clock::now_monotonic()`
### `contextra-vector` (17 Fundstellen)

- `/app/crates/contextra-vector/src/candidate_stream.rs:8` [random_hashset]: `use std::collections::{HashSet, VecDeque};`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/app/crates/contextra-vector/src/candidate_stream.rs:46` [random_hashset]: `seen: HashSet<DocId>,`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/app/crates/contextra-vector/src/candidate_stream.rs:61` [random_hashset]: `seen: HashSet::new(),`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/app/crates/contextra-vector/src/partial_rebuild.rs:35` [random_hashmap]: `use std::collections::{HashMap, VecDeque};`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/app/crates/contextra-vector/src/partial_rebuild.rs:89` [random_hashmap]: `tombstone_map: &HashMap<u64, bool>, // node_id → is_tombstoned`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/app/crates/contextra-vector/src/partial_rebuild.rs:128` [random_hashset]: `let mut seen = std::collections::HashSet::new();`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/app/crates/contextra-vector/src/partial_rebuild.rs:150` [random_hashmap]: `tombstone_map: &HashMap<u64, bool>,`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/app/crates/contextra-vector/src/diskann/predicate_augmented.rs:10` [random_hashset]: `use std::collections::{BinaryHeap, HashSet};`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/app/crates/contextra-vector/src/diskann/predicate_augmented.rs:59` [random_hashset]: `let mut visited = HashSet::new();`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/app/crates/contextra-vector/src/diskann/predicate_augmented.rs:151` [random_hashmap]: `let mut best_matching: std::collections::HashMap<DocId, f32> =`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/app/crates/contextra-vector/src/diskann/predicate_augmented.rs:152` [random_hashmap]: `std::collections::HashMap::new();`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/app/crates/contextra-vector/src/diskann/search.rs:14` [random_hashset]: `use std::collections::{BinaryHeap, HashSet};`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/app/crates/contextra-vector/src/diskann/search.rs:41` [random_hashset]: `let mut visited = HashSet::new();`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/app/crates/contextra-vector/src/diskann/search.rs:163` [random_hashset]: `let mut visited = HashSet::new();`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/app/crates/contextra-vector/src/diskann/search.rs:299` [random_hashset]: `let mut visited = HashSet::new();`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/app/crates/contextra-vector/src/diskann/build.rs:20` [random_hashset]: `pub(crate) set: std::collections::HashSet<u128>,`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/app/crates/contextra-vector/src/diskann/build.rs:27` [random_hashset]: `set: std::collections::HashSet::new(),`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
