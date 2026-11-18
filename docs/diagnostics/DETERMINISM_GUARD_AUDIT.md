# Determinismus Audit Report (P28)

> Generiert durch `tools/determinism_guard.py`.
> Untersucht Nicht-Determinismus in Produktivpfaden (Instant, SystemTime, thread_rng, HashMap/HashSet).

## Crate Übersicht (Produktivpfade)

| Crate | Instant::now | SystemTime::now | thread_rng | Random HashMap/Set | env::var | Gesamt Prod |
|---|---|---|---|---|---|---|
| `contextra` | 0 | 0 | 0 | 2 | 0 | **2** |
| `contextra-checkpoint` | 0 | 0 | 0 | 6 | 0 | **6** |
| `contextra-crypto` | 0 | 0 | 0 | 0 | 0 | **0** |
| `contextra-db` | 4 | 1 | 0 | 18 | 0 | **23** |
| `contextra-durable-fs` | 1 | 0 | 0 | 5 | 0 | **6** |
| `contextra-engine` | 0 | 0 | 0 | 0 | 0 | **0** |
| `contextra-env` | 2 | 1 | 0 | 0 | 0 | **3** |
| `contextra-erasure` | 2 | 1 | 0 | 0 | 0 | **3** |
| `contextra-graph` | 0 | 0 | 0 | 153 | 0 | **153** |
| `contextra-ids` | 0 | 0 | 0 | 0 | 0 | **0** |
| `contextra-mcp` | 0 | 0 | 2 | 3 | 15 | **20** |
| `contextra-mvcc` | 0 | 0 | 0 | 0 | 0 | **0** |
| `contextra-ports` | 0 | 0 | 0 | 0 | 0 | **0** |
| `contextra-privacy` | 0 | 0 | 0 | 0 | 0 | **0** |
| `contextra-rank` | 0 | 0 | 0 | 0 | 0 | **0** |
| `contextra-simd` | 0 | 0 | 0 | 0 | 0 | **0** |
| `contextra-store` | 2 | 0 | 0 | 7 | 0 | **9** |
| `contextra-sys` | 0 | 0 | 0 | 0 | 0 | **0** |
| `contextra-testkit` | 0 | 0 | 0 | 0 | 0 | **0** |
| `contextra-text` | 1 | 0 | 0 | 24 | 0 | **25** |
| `contextra-types` | 0 | 0 | 0 | 0 | 0 | **0** |
| `contextra-vault` | 1 | 0 | 0 | 0 | 0 | **1** |
| `contextra-vector` | 0 | 0 | 0 | 6 | 0 | **6** |

## Detaillierte Befunde mit Handlungsempfehlungen

### `contextra` (2 Fundstellen)

- `/home/freddy/Projekte/contextra/crates/contextra/src/query_rewriter.rs:32` [random_hashset]: `use std::collections::HashSet;`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra/src/query_rewriter.rs:248` [random_hashset]: `let mut seen = HashSet::new();`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
### `contextra-checkpoint` (6 Fundstellen)

- `/home/freddy/Projekte/contextra/crates/contextra-checkpoint/src/guard/tests.rs:8` [random_hashmap]: `use std::collections::{HashMap, HashSet};`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-checkpoint/src/guard/tests.rs:8` [random_hashset]: `use std::collections::{HashMap, HashSet};`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-checkpoint/src/guard/tests.rs:11` [random_hashmap]: `data: Mutex<HashMap<Vec<u8>, Vec<u8>>>,`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-checkpoint/src/guard/tests.rs:12` [random_hashset]: `pinned: Mutex<HashSet<u64>>,`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-checkpoint/src/guard/tests.rs:19` [random_hashmap]: `data: Mutex::new(HashMap::new()),`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-checkpoint/src/guard/tests.rs:20` [random_hashset]: `pinned: Mutex::new(HashSet::new()),`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
### `contextra-db` (23 Fundstellen)

- `/home/freddy/Projekte/contextra/crates/contextra-db/src/multistep.rs:434` [random_hashset]: `let mut seen = std::collections::HashSet::new();`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/homeostat.rs:17` [instant_now]: `deadline: Instant::now() + timeout,`
  - **Empfehlung:** `contextra_ports::Clock::now_monotonic()`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/homeostat.rs:22` [instant_now]: `Instant::now() >= self.deadline`
  - **Empfehlung:** `contextra_ports::Clock::now_monotonic()`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/homeostat.rs:26` [instant_now]: `self.deadline.saturating_duration_since(Instant::now())`
  - **Empfehlung:** `contextra_ports::Clock::now_monotonic()`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/background_workers/tests.rs:538` [system_time_now]: `let now_ms = std::time::SystemTime::now()`
  - **Empfehlung:** `contextra_ports::Clock::now_wall_clock()`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/contextra_impl/collections.rs:249` [random_hashset]: `let mut names = std::collections::HashSet::new();`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/contextra_impl/collections.rs:280` [random_hashset]: `let mut names = std::collections::HashSet::new();`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/contextra_impl/lifecycle.rs:240` [instant_now]: `let start_time = std::time::Instant::now();`
  - **Empfehlung:** `contextra_ports::Clock::now_monotonic()`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/orchestration/mod.rs:59` [random_hashmap]: `data: std::collections::HashMap<String, String>,`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/orchestration/mod.rs:66` [random_hashmap]: `data: std::collections::HashMap::new(),`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/collection/maintenance.rs:172` [random_hashset]: `let indexed_ids: std::collections::HashSet<DocId> =`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/collection/maintenance.rs:209` [random_hashset]: `indexed_ids: &std::collections::HashSet<DocId>,`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/collection/maintenance.rs:816` [random_hashmap]: `) -> Result<std::collections::HashMap<EntityId, u64>> {`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/collection/maintenance.rs:818` [random_hashmap]: `return Ok(std::collections::HashMap::new());`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/collection/maintenance.rs:830` [random_hashset]: `let target_set: std::collections::HashSet<&EntityId> = entity_ids.iter().collect();`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/collection/maintenance.rs:831` [random_hashmap]: `let mut map = std::collections::HashMap::new();`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/collection/maintenance.rs:914` [random_hashmap]: `let mut embeddings = std::collections::HashMap::new();`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/collection/maintenance.rs:915` [random_hashmap]: `let mut id_map = std::collections::HashMap::new();`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/collection/query_builder/builder_exec.rs:274` [random_hashset]: `let mut evaluated_indices = std::collections::HashSet::new();`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/collection/search/filtered.rs:91` [random_hashset]: `) -> Result<std::collections::HashSet<DocId>> {`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/collection/search/filtered.rs:101` [random_hashset]: `let mut matched = std::collections::HashSet::new();`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/collection/search/filtered.rs:127` [random_hashset]: `) -> Result<Option<std::collections::HashSet<DocId>>> {`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/collection/search/filtered.rs:141` [random_hashset]: `let mut matched = std::collections::HashSet::new();`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
### `contextra-durable-fs` (6 Fundstellen)

- `/home/freddy/Projekte/contextra/crates/contextra-durable-fs/src/fault_injector.rs:3` [random_hashmap]: `use std::collections::HashMap;`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-durable-fs/src/fault_injector.rs:101` [random_hashmap]: `synced_snapshots: Mutex<HashMap<PathBuf, Vec<u8>>>,`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-durable-fs/src/fault_injector.rs:102` [random_hashmap]: `dirty_snapshots: Mutex<HashMap<PathBuf, Vec<u8>>>,`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-durable-fs/src/fault_injector.rs:114` [random_hashmap]: `synced_snapshots: Mutex::new(HashMap::new()),`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-durable-fs/src/fault_injector.rs:115` [random_hashmap]: `dirty_snapshots: Mutex::new(HashMap::new()),`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-durable-fs/src/scrub.rs:103` [instant_now]: `let start = Instant::now();`
  - **Empfehlung:** `contextra_ports::Clock::now_monotonic()`
### `contextra-env` (3 Fundstellen)

- `/home/freddy/Projekte/contextra/crates/contextra-env/src/clock.rs:70` [instant_now]: `start_instant: Instant::now(),`
  - **Empfehlung:** `contextra_ports::Clock::now_monotonic()`
- `/home/freddy/Projekte/contextra/crates/contextra-env/src/clock.rs:90` [system_time_now]: `SystemTime::now()`
  - **Empfehlung:** `contextra_ports::Clock::now_wall_clock()`
- `/home/freddy/Projekte/contextra/crates/contextra-env/src/clock.rs:103` [instant_now]: `let nanos = Instant::now()`
  - **Empfehlung:** `contextra_ports::Clock::now_monotonic()`
### `contextra-erasure` (3 Fundstellen)

- `/home/freddy/Projekte/contextra/crates/contextra-erasure/src/types_local.rs:2114` [instant_now]: `start_instant: std::time::Instant::now(),`
  - **Empfehlung:** `contextra_ports::Clock::now_monotonic()`
- `/home/freddy/Projekte/contextra/crates/contextra-erasure/src/types_local.rs:2129` [system_time_now]: `std::time::SystemTime::now()`
  - **Empfehlung:** `contextra_ports::Clock::now_wall_clock()`
- `/home/freddy/Projekte/contextra/crates/contextra-erasure/src/types_local.rs:2138` [instant_now]: `let nanos = std::time::Instant::now()`
  - **Empfehlung:** `contextra_ports::Clock::now_monotonic()`
### `contextra-graph` (153 Fundstellen)

- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/hyperedge.rs:17` [random_hashmap]: `use scc::HashMap;`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/hyperedge.rs:281` [random_hashmap]: `forward: HashMap<String, RoleId>,`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/hyperedge.rs:282` [random_hashmap]: `backward: HashMap<RoleId, String>,`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/hyperedge.rs:363` [random_hashmap]: `forward: HashMap::new(),`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/hyperedge.rs:364` [random_hashmap]: `backward: HashMap::new(),`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/community.rs:26` [random_hashmap]: `use std::collections::HashMap;`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/community.rs:240` [random_hashmap]: `let mut adj_raw: HashMap<usize, HashMap<usize, f32>> = HashMap::new();`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/community.rs:268` [random_hashmap]: `let mut virtual_map: HashMap<crate::hyperedge::HyperEdgeId, usize> = HashMap::new();`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/community.rs:333` [random_hashmap]: `let mut global_to_local: HashMap<usize, usize> = HashMap::with_capacity(num_total_nodes);`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/community.rs:351` [random_hashmap]: `let entity_to_local: HashMap<EntityId, usize> = local_entity_ids`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/community.rs:364` [random_hashmap]: `let mut vnode_to_local: HashMap<crate::hyperedge::HyperEdgeId, usize> = HashMap::new();`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/community.rs:421` [random_hashmap]: `let mut community_degrees: HashMap<u64, f32> = HashMap::new();`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/community.rs:448` [random_hashmap]: `let mut comm_weights: HashMap<u64, f32> = HashMap::new();`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/community.rs:494` [random_hashmap]: `let mut comm_members: HashMap<u64, Vec<usize>> = HashMap::new();`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/community.rs:508` [random_hashmap]: `let mut sub_comm: HashMap<usize, u64> = HashMap::new();`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/community.rs:509` [random_hashmap]: `let mut sub_degrees: HashMap<u64, f32> = HashMap::new();`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/community.rs:532` [random_hashmap]: `let mut sub_weights: HashMap<u64, f32> = HashMap::new();`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/community.rs:581` [random_hashmap]: `let mut sub_members: HashMap<u64, Vec<usize>> = HashMap::new();`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/percolation.rs:15` [random_hashmap]: `use std::collections::HashMap;`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/percolation.rs:89` [random_hashmap]: `embeddings: &HashMap<EntityId, Vec<f32>>,`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/percolation.rs:101` [random_hashmap]: `let mut neighbors_cache: HashMap<EntityId, std::collections::HashSet<EntityId>> =`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/percolation.rs:101` [random_hashset]: `let mut neighbors_cache: HashMap<EntityId, std::collections::HashSet<EntityId>> =`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/percolation.rs:102` [random_hashmap]: `HashMap::new();`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/percolation.rs:122` [random_hashset]: `e.insert(std::collections::HashSet::new());`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/percolation.rs:134` [random_hashset]: `e.insert(std::collections::HashSet::new());`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/ppr_stream.rs:24` [random_hashset]: `use std::collections::{HashSet, VecDeque};`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/ppr_stream.rs:156` [random_hashset]: `let seed_set: HashSet<EntityId> = self.seeds.iter().copied().collect();`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/consistency_enforcement.rs:33` [random_hashmap]: `use std::collections::HashMap;`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/consistency_enforcement.rs:105` [random_hashmap]: `patterns: HashMap<[u8; 32], ConflictPattern>,`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/consistency_enforcement.rs:118` [random_hashmap]: `patterns: HashMap::new(),`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/cascade.rs:198` [random_hashset]: `let node_set: std::collections::HashSet<_> = closure_nodes.iter().copied().collect();`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/cascade.rs:200` [random_hashmap]: `let mut in_degree = std::collections::HashMap::new();`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/cascade.rs:201` [random_hashmap]: `let mut children_map: std::collections::HashMap<_, Vec<_>> = std::collections::HashMap::new();`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/cascade.rs:242` [random_hashset]: `let placed: std::collections::HashSet<_> = topo_order.iter().copied().collect();`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/cascade.rs:283` [random_hashset]: `let mut visited = std::collections::HashSet::new();`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/cascade.rs:284` [random_hashset]: `let mut on_stack = std::collections::HashSet::new();`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/cascade.rs:289` [random_hashset]: `visited: &mut std::collections::HashSet<crate::hyperedge::HyperEdgeId>,`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/cascade.rs:290` [random_hashset]: `on_stack: &mut std::collections::HashSet<crate::hyperedge::HyperEdgeId>,`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/cascade.rs:410` [random_hashset]: `let mut visited = std::collections::HashSet::new();`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/session_dag.rs:19` [random_hashmap]: `use std::collections::HashMap;`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/session_dag.rs:37` [random_hashmap]: `_nodes_guard: RwLockReadGuard<'a, HashMap<NodeIdx, AgentStateNode>>,`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/session_dag.rs:42` [random_hashmap]: `pub fn nodes(&self) -> &HashMap<NodeIdx, AgentStateNode> {`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/session_dag.rs:74` [random_hashmap]: `_nodes_guard: RwLockWriteGuard<'a, HashMap<NodeIdx, AgentStateNode>>,`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/session_dag.rs:79` [random_hashmap]: `pub fn nodes(&self) -> &HashMap<NodeIdx, AgentStateNode> {`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/session_dag.rs:84` [random_hashmap]: `pub fn nodes_mut(&mut self) -> &mut HashMap<NodeIdx, AgentStateNode> {`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/session_dag.rs:167` [random_hashmap]: `nodes: RwLock<HashMap<NodeIdx, AgentStateNode>>,`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/session_dag.rs:185` [random_hashmap]: `let mut nodes = HashMap::new();`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/session_dag.rs:350` [random_hashset]: `let mut visited = std::collections::HashSet::new();`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/session_dag.rs:475` [random_hashmap]: `let mut nodes = HashMap::new();`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/community/tests.rs:131` [random_hashmap]: `let map: HashMap<u64, u64> = assignments`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/community/tests.rs:226` [random_hashmap]: `let map: HashMap<u64, u64> = assignments`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/community/tests.rs:329` [random_hashset]: `let expected_ids: std::collections::HashSet<_> = (0..node_count)`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/community/tests.rs:347` [random_hashset]: `let assigned_ids: std::collections::HashSet<_> = assignments`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/community/tests.rs:806` [random_hashmap]: `let map_false: HashMap<u64, u64> = assignments_false`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/community/tests.rs:820` [random_hashmap]: `let map_true: HashMap<u64, u64> = assignments_true`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/compat/provider.rs:155` [random_hashmap]: `) -> HashMap<EntityId, f32> {`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/compat/provider.rs:159` [random_hashmap]: `return HashMap::new();`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/compat/provider.rs:161` [random_hashmap]: `let node_to_idx: HashMap<EntityId, usize> = nodes`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/compat/provider.rs:224` [random_hashmap]: `let node_to_idx: HashMap<EntityId, usize> =`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/compat/provider.rs:251` [random_hashmap]: `let mut comm_degs: HashMap<u64, f32> = communities`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/compat/provider.rs:264` [random_hashmap]: `let mut weights: HashMap<u64, f32> = HashMap::new();`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/ppr/snapshot.rs:19` [random_hashmap]: `use std::collections::HashMap;`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/ppr/snapshot.rs:126` [random_hashmap]: `let mut snapshot_pending: HashMap<InternalIndex, Vec<EdgePayload>> = HashMap::new();`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/ppr/shadow_hook.rs:41` [random_hashset]: `let mut all_keys = std::collections::HashSet::new();`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/ppr/types.rs:13` [random_hashset]: `use std::collections::HashSet;`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/ppr/types.rs:20` [random_hashset]: `deleted_nodes: HashSet<usize>,`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/ppr/types.rs:24` [random_hashset]: `pub(crate) fn from_nodes(deleted_nodes: HashSet<usize>) -> Self {`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/ppr/dense.rs:17` [random_hashset]: `use std::collections::HashSet;`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/ppr/dense.rs:29` [random_hashset]: `let mut seen_seeds = HashSet::new();`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/ppr/mod.rs:275` [random_hashset]: `let mut seen = std::collections::HashSet::new();`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/ppr/mod.rs:325` [random_hashset]: `let mut seen = std::collections::HashSet::new();`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/path_rag/snapshot.rs:9` [random_hashset]: `use std::collections::HashSet;`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/path_rag/snapshot.rs:47` [random_hashset]: `let mut seen = HashSet::new();`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/path_rag/snapshot.rs:122` [random_hashset]: `let mut seen = HashSet::new();`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/path_rag/tests.rs:19` [random_hashmap]: `edges: HashMap<EntityId, Vec<(EntityId, f32)>>,`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/path_rag/tests.rs:24` [random_hashmap]: `let mut map: HashMap<EntityId, Vec<(EntityId, f32)>> = HashMap::new();`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/path_rag/tests.rs:202` [random_hashmap]: `edges: HashMap<EntityId, Vec<(EntityId, f32)>>,`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/path_rag/tests.rs:203` [random_hashmap]: `hyperedges: HashMap<EntityId, Vec<HyperEdgeId>>,`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/path_rag/tests.rs:204` [random_hashmap]: `hyperedge_store: HashMap<HyperEdgeId, HyperEdge>,`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/path_rag/tests.rs:209` [random_hashmap]: `let mut map: HashMap<EntityId, Vec<(EntityId, f32)>> = HashMap::new();`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/path_rag/tests.rs:214` [random_hashmap]: `let mut he_map: HashMap<EntityId, Vec<HyperEdgeId>> = HashMap::new();`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/path_rag/tests.rs:215` [random_hashmap]: `let mut he_store = HashMap::new();`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/path_rag/k_path.rs:18` [random_hashmap]: `use std::collections::{BinaryHeap, HashMap};`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/path_rag/k_path.rs:154` [random_hashmap]: `let mut fwd_paths: HashMap<EntityId, Vec<Vec<EntityId>>> = HashMap::new();`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/path_rag/k_path.rs:155` [random_hashmap]: `let mut bwd_paths: HashMap<EntityId, Vec<Vec<EntityId>>> = HashMap::new();`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/path_rag/mod.rs:36` [random_hashmap]: `use std::collections::{BinaryHeap, HashMap};`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/path_rag/mod.rs:299` [random_hashmap]: `let mut dist_fwd: HashMap<EntityId, f32> = HashMap::new();`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/path_rag/mod.rs:300` [random_hashmap]: `let mut dist_bwd: HashMap<EntityId, f32> = HashMap::new();`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/path_rag/mod.rs:301` [random_hashmap]: `let mut prev_fwd: HashMap<EntityId, (EntityId, f32)> = HashMap::new();`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/path_rag/mod.rs:302` [random_hashmap]: `let mut prev_bwd: HashMap<EntityId, (EntityId, f32)> = HashMap::new();`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/path_rag/mod.rs:421` [random_hashset]: `let mut targets = std::collections::HashSet::new();`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/path_rag/mod.rs:461` [random_hashmap]: `let mut scored: HashMap<u64, f32> = HashMap::new();`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/csr/path_graph.rs:76` [random_hashset]: `let mut seen = std::collections::HashSet::new();`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/csr/path_graph.rs:124` [random_hashset]: `let mut seen = std::collections::HashSet::new();`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/csr/graph_persist.rs:18` [random_hashset]: `use std::collections::HashSet;`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/csr/graph_persist.rs:93` [random_hashset]: `let mut deleted_entity_ids = HashSet::new();`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/csr/inner/estimate.rs:13` [random_hashset]: `use std::collections::HashSet;`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/csr/inner/estimate.rs:175` [random_hashset]: `p += table_bytes::<DocId, HashSet<(EntityId, EntityId)>>(cap);`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/csr/inner/estimate.rs:196` [random_hashset]: `p += table_bytes::<DocId, HashSet<crate::hyperedge::HyperEdgeId>>(cap);`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/csr/inner/estimate.rs:210` [random_hashset]: `p += table_bytes::<crate::hyperedge::HyperEdgeId, HashSet<crate::hyperedge::HyperEdgeId>>(`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/csr/inner/estimate.rs:226` [random_hashset]: `p += table_bytes::<EntityId, HashSet<crate::hyperedge::HyperEdgeId>>(cap);`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/csr/inner/mod.rs:20` [random_hashmap]: `use std::collections::{HashMap, HashSet};`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/csr/inner/mod.rs:20` [random_hashset]: `use std::collections::{HashMap, HashSet};`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/csr/inner/mod.rs:34` [random_hashmap]: `pub(crate) id_map: HashMap<EntityId, InternalIndex>,`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/csr/inner/mod.rs:37` [random_hashmap]: `pub(crate) communities: HashMap<EntityId, u64>,`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/csr/inner/mod.rs:50` [random_hashset]: `pub(crate) doc_to_edges: ahash::AHashMap<DocId, HashSet<(EntityId, EntityId)>>,`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/csr/inner/mod.rs:55` [random_hashmap]: `pub(crate) pending_edges: HashMap<InternalIndex, Vec<EdgePayload>>,`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/csr/inner/mod.rs:56` [random_hashset]: `pub(crate) tombstoned_edges: HashSet<(InternalIndex, InternalIndex)>,`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/csr/inner/mod.rs:60` [random_hashmap]: `pub(crate) edge_store: HashMap<EntityId, Vec<CsrEdge>>,`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/csr/inner/mod.rs:62` [random_hashmap]: `pub(crate) hyperedges: HashMap<crate::hyperedge::HyperEdgeId, Arc<crate::hyperedge::HyperEdge>>,`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/csr/inner/mod.rs:63` [random_hashset]: `pub(crate) doc_to_hyperedges: ahash::AHashMap<DocId, HashSet<crate::hyperedge::HyperEdgeId>>,`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/csr/inner/mod.rs:64` [random_hashset]: `pub(crate) hyperedge_index: ahash::AHashMap<EntityId, HashSet<crate::hyperedge::HyperEdgeId>>,`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/csr/inner/mod.rs:66` [random_hashset]: `ahash::AHashMap<crate::hyperedge::HyperEdgeId, HashSet<crate::hyperedge::HyperEdgeId>>,`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/csr/inner/mod.rs:84` [random_hashmap]: `id_map: HashMap::new(),`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/csr/inner/mod.rs:87` [random_hashmap]: `communities: HashMap::new(),`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/csr/inner/mod.rs:102` [random_hashmap]: `pending_edges: HashMap::new(),`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/csr/inner/mod.rs:103` [random_hashset]: `tombstoned_edges: HashSet::new(),`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/csr/inner/mod.rs:107` [random_hashmap]: `edge_store: HashMap::new(),`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/csr/inner/mod.rs:108` [random_hashmap]: `hyperedges: HashMap::new(),`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/csr/graph_write/hyperedge.rs:16` [random_hashset]: `use std::collections::HashSet;`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/csr/graph_write/hyperedge.rs:24` [random_hashset]: `visited: &mut HashSet<crate::hyperedge::HyperEdgeId>,`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/csr/graph_write/hyperedge.rs:25` [random_hashset]: `on_stack: &mut HashSet<crate::hyperedge::HyperEdgeId>,`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/csr/graph_write/hyperedge.rs:85` [random_hashset]: `let mut visited = HashSet::new();`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/csr/graph_write/hyperedge.rs:86` [random_hashset]: `let mut on_stack = HashSet::new();`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/csr/graph_write/hyperedge.rs:378` [random_hashset]: `let mut seen_ids = HashSet::with_capacity(new_edges.len());`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/csr/graph_write/tombstone.rs:18` [random_hashset]: `use std::collections::HashSet;`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/csr/graph_write/tombstone.rs:43` [random_hashset]: `let mut affected_nodes_set = HashSet::new();`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/csr/graph_write/mod.rs:22` [random_hashset]: `use std::collections::{HashSet, VecDeque};`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/csr/graph_write/mod.rs:199` [random_hashset]: `let mut edges: HashSet<(EntityId, EntityId)> =`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/csr/graph_read/communities.rs:17` [random_hashmap]: `use std::collections::HashMap;`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/csr/graph_read/communities.rs:34` [random_hashmap]: `) -> Result<HashMap<EntityId, u64>> {`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/csr/graph_read/communities.rs:37` [random_hashmap]: `let mut map = HashMap::with_capacity(entity_ids.len());`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/csr/graph_read/communities.rs:70` [random_hashmap]: `let mut map = HashMap::with_capacity(entity_ids.len());`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/csr/graph_read/communities.rs:78` [random_hashmap]: `Ok(HashMap::new())`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/csr/graph_read/communities.rs:83` [random_hashmap]: `Ok(HashMap::new())`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/csr/graph_read/mod.rs:100` [random_hashset]: `let mut seen: std::collections::HashSet<EntityId> = std::collections::HashSet::new();`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/csr/graph_read/pagerank.rs:16` [random_hashmap]: `use std::collections::{HashMap, HashSet};`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/csr/graph_read/pagerank.rs:16` [random_hashset]: `use std::collections::{HashMap, HashSet};`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/csr/graph_read/pagerank.rs:26` [random_hashmap]: `) -> HashMap<EntityId, f32> {`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/csr/graph_read/pagerank.rs:33` [random_hashmap]: `return HashMap::new();`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/csr/graph_read/pagerank.rs:90` [random_hashmap]: `let mut result = HashMap::new();`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/csr/graph_read/pagerank.rs:101` [random_hashset]: `pub async fn get_deleted_node_indices(&self) -> HashSet<usize> {`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/csr/graph_read/pagerank.rs:105` [random_hashset]: `return HashSet::new();`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/csr/graph_read/pagerank.rs:111` [random_hashset]: `let mut deleted_indices = HashSet::new();`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/csr/graph_read/pagerank.rs:121` [random_hashset]: `HashSet::new()`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/compat/graph_index/traversal.rs:14` [random_hashmap]: `use std::collections::{HashMap, VecDeque};`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/compat/graph_index/traversal.rs:82` [random_hashmap]: `mut visited: HashMap<InternalIndex, f32>,`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/compat/graph_index/traversal.rs:105` [random_hashmap]: `visited: &mut HashMap<InternalIndex, f32>,`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/compat/graph_index/traversal.rs:176` [random_hashmap]: `) -> HashMap<InternalIndex, f32> {`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/compat/graph_index/traversal.rs:177` [random_hashmap]: `let mut visited: HashMap<InternalIndex, f32> = HashMap::new();`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/compat/graph_index/traversal.rs:220` [random_hashmap]: `visited: &mut HashMap<InternalIndex, f32>,`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/compat/graph_index/traversal.rs:268` [random_hashmap]: `) -> HashMap<InternalIndex, f32> {`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/compat/graph_index/traversal.rs:269` [random_hashmap]: `let mut visited: HashMap<InternalIndex, f32> = HashMap::new();`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
### `contextra-mcp` (20 Fundstellen)

- `/home/freddy/Projekte/contextra/crates/contextra-mcp/src/server.rs:111` [env_var]: `let timeout_secs = std::env::var("CONTEXTRA_MCP_IDLE_TIMEOUT_SECS")`
  - **Empfehlung:** `Explizite Konfigurationsübergabe (Config-Struct)`
- `/home/freddy/Projekte/contextra/crates/contextra-mcp/src/validation.rs:9` [env_var]: `std::env::var("CONTEXTRA_MCP_ALLOW_WRITE")`
  - **Empfehlung:** `Explizite Konfigurationsübergabe (Config-Struct)`
- `/home/freddy/Projekte/contextra/crates/contextra-mcp/src/sandbox.rs:19` [random_hashmap]: `use std::collections::HashMap;`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-mcp/src/sandbox.rs:271` [random_hashmap]: `volatile_results: Mutex<HashMap<String, VolatileToolResult>>,`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-mcp/src/sandbox.rs:289` [random_hashmap]: `volatile_results: Mutex::new(HashMap::new()),`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-mcp/src/sandbox.rs:447` [thread_rng]: `rand::thread_rng().fill_bytes(&mut salt);`
  - **Empfehlung:** `contextra_ports::RngSeed / deterministischer ChaCha20Rng`
- `/home/freddy/Projekte/contextra/crates/contextra-mcp/src/sandbox.rs:450` [thread_rng]: `rand::thread_rng().fill_bytes(&mut passphrase);`
  - **Empfehlung:** `contextra_ports::RngSeed / deterministischer ChaCha20Rng`
- `/home/freddy/Projekte/contextra/crates/contextra-mcp/src/config.rs:200` [env_var]: `let prefixed = std::env::var("CONTEXTRA_EMBEDDING_PROVIDER").ok();`
  - **Empfehlung:** `Explizite Konfigurationsübergabe (Config-Struct)`
- `/home/freddy/Projekte/contextra/crates/contextra-mcp/src/config.rs:201` [env_var]: `let unprefixed = std::env::var("EMBEDDING_PROVIDER").ok();`
  - **Empfehlung:** `Explizite Konfigurationsübergabe (Config-Struct)`
- `/home/freddy/Projekte/contextra/crates/contextra-mcp/src/config.rs:214` [env_var]: `let ollama_url = std::env::var("CONTEXTRA_OLLAMA_URL")`
  - **Empfehlung:** `Explizite Konfigurationsübergabe (Config-Struct)`
- `/home/freddy/Projekte/contextra/crates/contextra-mcp/src/config.rs:217` [env_var]: `let embed_model = std::env::var("CONTEXTRA_EMBED_MODEL")`
  - **Empfehlung:** `Explizite Konfigurationsübergabe (Config-Struct)`
- `/home/freddy/Projekte/contextra/crates/contextra-mcp/src/config.rs:220` [env_var]: `let onnx_model_path = std::env::var("CONTEXTRA_ONNX_MODEL_PATH")`
  - **Empfehlung:** `Explizite Konfigurationsübergabe (Config-Struct)`
- `/home/freddy/Projekte/contextra/crates/contextra-mcp/src/config.rs:224` [env_var]: `let candle_model_dir = std::env::var("CONTEXTRA_CANDLE_MODEL_DIR")`
  - **Empfehlung:** `Explizite Konfigurationsübergabe (Config-Struct)`
- `/home/freddy/Projekte/contextra/crates/contextra-mcp/src/config.rs:325` [env_var]: `let prefixed = std::env::var("CONTEXTRA_LLM_PROVIDER").ok();`
  - **Empfehlung:** `Explizite Konfigurationsübergabe (Config-Struct)`
- `/home/freddy/Projekte/contextra/crates/contextra-mcp/src/config.rs:326` [env_var]: `let unprefixed = std::env::var("LLM_PROVIDER").ok();`
  - **Empfehlung:** `Explizite Konfigurationsübergabe (Config-Struct)`
- `/home/freddy/Projekte/contextra/crates/contextra-mcp/src/config.rs:339` [env_var]: `let ollama_url = std::env::var("CONTEXTRA_OLLAMA_URL")`
  - **Empfehlung:** `Explizite Konfigurationsübergabe (Config-Struct)`
- `/home/freddy/Projekte/contextra/crates/contextra-mcp/src/config.rs:343` [env_var]: `std::env::var("CONTEXTRA_LLM_MODEL").unwrap_or_else(|_| "llama3.2:3b".to_string());`
  - **Empfehlung:** `Explizite Konfigurationsübergabe (Config-Struct)`
- `/home/freddy/Projekte/contextra/crates/contextra-mcp/src/config.rs:345` [env_var]: `let candle_model_dir = std::env::var("CONTEXTRA_CANDLE_MODEL_DIR")`
  - **Empfehlung:** `Explizite Konfigurationsübergabe (Config-Struct)`
- `/home/freddy/Projekte/contextra/crates/contextra-mcp/src/config.rs:421` [env_var]: `let profiles_path = std::env::var("CONTEXTRA_ROUTER_PROFILES_PATH")`
  - **Empfehlung:** `Explizite Konfigurationsübergabe (Config-Struct)`
- `/home/freddy/Projekte/contextra/crates/contextra-mcp/src/config.rs:425` [env_var]: `let calibration_store_path = std::env::var("CONTEXTRA_ROUTER_CALIBRATION_PATH")`
  - **Empfehlung:** `Explizite Konfigurationsübergabe (Config-Struct)`
### `contextra-store` (9 Fundstellen)

- `/home/freddy/Projekte/contextra/crates/contextra-store/fuzz/fuzz_targets/fuzz_compaction_interleave.rs:8` [random_hashmap]: `use std::collections::HashMap;`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-store/fuzz/fuzz_targets/fuzz_compaction_interleave.rs:59` [random_hashmap]: `let mut oracle: HashMap<Vec<u8>, Option<Vec<u8>>> = HashMap::new();`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-store/fuzz/fuzz_targets/fuzz_memtable_concurrent.rs:99` [instant_now]: `let start_time = std::time::Instant::now();`
  - **Empfehlung:** `contextra_ports::Clock::now_monotonic()`
- `/home/freddy/Projekte/contextra/crates/contextra-store/src/manifest/cleanup.rs:64` [random_hashset]: `pub fn reconstruct_dead_sstables(entries: &[ManifestEntry]) -> HashSet<PathBuf> {`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-store/src/wal/flusher.rs:184` [instant_now]: `let deadline = tokio::time::Instant::now()`
  - **Empfehlung:** `contextra_ports::Clock::now_monotonic()`
- `/home/freddy/Projekte/contextra/crates/contextra-store/src/lsm/engine.rs:22` [random_hashmap]: `use std::collections::HashMap;`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-store/src/lsm/engine.rs:73` [random_hashmap]: `pub(super) intent_locks: std::sync::Mutex<HashMap<Vec<u8>, TxId>>,`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-store/src/lsm/recovery/startup.rs:144` [random_hashset]: `Option<std::collections::HashSet<std::path::PathBuf>>,`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-store/src/lsm/recovery/startup.rs:514` [random_hashmap]: `intent_locks: std::sync::Mutex::new(std::collections::HashMap::new()),`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
### `contextra-text` (25 Fundstellen)

- `/home/freddy/Projekte/contextra/crates/contextra-text/src/tokenizer.rs:20` [random_hashset]: `use std::collections::HashSet;`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-text/src/tokenizer.rs:25` [random_hashset]: `static STOPWORDS: OnceLock<HashSet<String>> = OnceLock::new();`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-text/src/tokenizer.rs:116` [random_hashset]: `fn get_stopwords() -> &'static HashSet<String> {`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-text/src/tokenizer.rs:457` [random_hashset]: `stopwords: Option<HashSet<String>>,`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-text/src/tokenizer.rs:489` [random_hashset]: `pub fn with_stopwords(mut self, stopwords: HashSet<String>) -> Self {`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-text/src/inverted/index_struct.rs:4` [random_hashmap]: `use std::collections::HashMap;`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-text/src/inverted/index_struct.rs:27` [random_hashmap]: `pub(crate) staged_stats: Arc<parking_lot::Mutex<HashMap<TxId, StagedStatsChange>>>,`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-text/src/inverted/index_struct.rs:28` [random_hashmap]: `pub(crate) staged_terms: Arc<parking_lot::Mutex<HashMap<TxId, Vec<String>>>>,`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-text/src/inverted/index_struct.rs:86` [random_hashmap]: `staged_stats: Arc::new(parking_lot::Mutex::new(HashMap::new())),`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-text/src/inverted/index_struct.rs:87` [random_hashmap]: `staged_terms: Arc::new(parking_lot::Mutex::new(HashMap::new())),`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-text/src/inverted/index_struct.rs:268` [random_hashmap]: `let mut tfs = HashMap::with_capacity(tokens.len());`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-text/src/compat/purge.rs:25` [random_hashset]: `use std::collections::HashSet;`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-text/src/compat/purge.rs:176` [random_hashset]: `let mut terms_to_check: HashSet<String> = HashSet::new();`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-text/src/morphology/tests.rs:8` [random_hashset]: `use std::collections::HashSet;`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-text/src/morphology/tests.rs:635` [random_hashset]: `let mut custom_words = HashSet::new();`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-text/src/morphology/tests.rs:738` [instant_now]: `let start = std::time::Instant::now();`
  - **Empfehlung:** `contextra_ports::Clock::now_monotonic()`
- `/home/freddy/Projekte/contextra/crates/contextra-text/src/morphology/stopwords.rs:3` [random_hashset]: `use std::collections::HashSet;`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-text/src/morphology/stopwords.rs:6` [random_hashset]: `static GERMAN_STOPWORDS: OnceLock<HashSet<String>> = OnceLock::new();`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-text/src/morphology/stopwords.rs:13` [random_hashset]: `pub fn get_german_stopwords() -> &'static HashSet<String> {`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-text/src/morphology/stopwords.rs:22` [random_hashset]: `let mut set = HashSet::with_capacity(words.len());`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-text/src/morphology/trie.rs:10` [random_hashmap]: `pub children: std::collections::HashMap<char, TrieNode>,`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-text/src/morphology/compound_splitter.rs:6` [random_hashset]: `use std::collections::HashSet;`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-text/src/morphology/compound_splitter.rs:81` [random_hashset]: `pub fn with_dictionary(min_len: usize, custom_words: HashSet<String>) -> Self {`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-text/src/domain/mod.rs:18` [random_hashset]: `use std::collections::HashSet;`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-text/src/domain/mod.rs:74` [random_hashset]: `let protected_set: HashSet<&'static str> = protected_vec.iter().copied().collect();`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
### `contextra-vault` (1 Fundstellen)

- `/home/freddy/Projekte/contextra/crates/contextra-vault/src/vault.rs:305` [instant_now]: `let purged_at = Instant::now();`
  - **Empfehlung:** `contextra_ports::Clock::now_monotonic()`
### `contextra-vector` (6 Fundstellen)

- `/home/freddy/Projekte/contextra/crates/contextra-vector/src/fixed_hasher.rs:12` [random_hashmap]: `use std::collections::{HashMap, HashSet};`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-vector/src/fixed_hasher.rs:12` [random_hashset]: `use std::collections::{HashMap, HashSet};`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-vector/src/fixed_hasher.rs:35` [random_hashset]: `pub type FixedHashSet<T> = HashSet<T, RandomState>;`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-vector/src/fixed_hasher.rs:38` [random_hashmap]: `pub type FixedHashMap<K, V> = HashMap<K, V, RandomState>;`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-vector/src/fixed_hasher.rs:42` [random_hashset]: `HashSet::with_capacity_and_hasher(capacity, fixed_hasher())`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/home/freddy/Projekte/contextra/crates/contextra-vector/src/fixed_hasher.rs:47` [random_hashmap]: `HashMap::with_capacity_and_hasher(capacity, fixed_hasher())`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
