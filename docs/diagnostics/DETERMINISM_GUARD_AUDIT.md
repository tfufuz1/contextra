# Determinismus Audit Report (P28)

> Generiert durch `tools/determinism_guard.py`.
> Untersucht Nicht-Determinismus in Produktivpfaden (Instant, SystemTime, thread_rng, HashMap/HashSet).

## Crate Übersicht (Produktivpfade)

| Crate | Instant::now | SystemTime::now | thread_rng | Random HashMap/Set | env::var | Gesamt Prod |
|---|---|---|---|---|---|---|
| `contextra-store` | 2 | 0 | 0 | 15 | 0 | **17** |

## Detaillierte Befunde mit Handlungsempfehlungen

### `contextra-store` (17 Fundstellen)

- `/app/crates/contextra-store/src/lsm/engine.rs:18` [random_hashmap]: `use std::collections::HashMap;`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/app/crates/contextra-store/src/lsm/engine.rs:67` [random_hashmap]: `pub(super) intent_locks: std::sync::Mutex<HashMap<Vec<u8>, TxId>>,`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/app/crates/contextra-store/src/lsm/recovery.rs:251` [random_hashset]: `Option<std::collections::HashSet<std::path::PathBuf>>,`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/app/crates/contextra-store/src/lsm/recovery.rs:252` [random_hashset]: `Option<std::collections::HashSet<std::path::PathBuf>>,`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/app/crates/contextra-store/src/lsm/recovery.rs:271` [random_hashmap]: `let mut pending_tx_map: std::collections::HashMap<u64, Vec<PendingTxOp>> =`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/app/crates/contextra-store/src/lsm/recovery.rs:272` [random_hashmap]: `std::collections::HashMap::new();`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/app/crates/contextra-store/src/lsm/recovery.rs:275` [random_hashset]: `let mut replayed_hmac_set = std::collections::HashSet::new();`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/app/crates/contextra-store/src/lsm/recovery.rs:620` [random_hashmap]: `intent_locks: std::sync::Mutex::new(std::collections::HashMap::new()),`
  - **Empfehlung:** `BTreeMap oder HashMap mit festem BuildHasher`
- `/app/crates/contextra-store/src/lsm/recovery.rs:742` [random_hashset]: `let txs_to_rollback: std::collections::HashSet<u64> = memtable`
  - **Empfehlung:** `BTreeSet oder HashSet mit festem BuildHasher`
- `/app/crates/contextra-store/src/wal/flusher.rs:242` [instant_now]: `let deadline = tokio::time::Instant::now()`
  - **Empfehlung:** `contextra_ports::Clock::now_monotonic()`
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
