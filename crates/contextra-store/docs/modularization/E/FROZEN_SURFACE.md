# FROZEN_SURFACE — contextra-store (Modul LSM-Schreibpfad / Gruppe E)

## 1. Angebotene Items (Außenfläche Gruppe E)

### Aus `src/lsm/ops/write.rs`:
- `pub(crate) async fn put(storage: &LsmStorage, tx_id: TxId, key: &[u8], value: &[u8]) -> Result<()>` (aufgerufen in `src/lsm/engine.rs:595`)
- `pub(crate) async fn put_if_absent(storage: &LsmStorage, tx_id: TxId, key: &[u8], value: &[u8]) -> Result<bool>` (aufgerufen in `src/lsm/engine.rs:599`)
- `pub(crate) async fn delete_many(storage: &LsmStorage, tx_id: TxId, keys: &[&[u8]]) -> Result<()>` (aufgerufen in `src/lsm/engine.rs:603`)
- `pub(crate) async fn delete_prefix(storage: &LsmStorage, tx_id: TxId, prefix: &[u8]) -> Result<()>` (aufgerufen in `src/lsm/engine.rs:607`)
- `pub(crate) async fn delete(storage: &LsmStorage, tx_id: TxId, key: &[u8]) -> Result<()>` (aufgerufen in `src/lsm/engine.rs:611`)
- `pub(crate) async fn commit(storage: &LsmStorage, tx_id: TxId) -> Result<()>` (aufgerufen in `src/lsm/engine.rs:615`)

### Aus `src/lsm/ops/maintenance.rs`:
- `pub(crate) async fn rollback(storage: &LsmStorage, tx_id: TxId) -> Result<()>` (aufgerufen in `src/lsm/engine.rs:619`)
- `pub(crate) async fn rollback_to_tx(storage: &LsmStorage, tx_id: TxId) -> Result<()>` (aufgerufen in `src/compat/engine/lsm.rs:162`)
- `pub(crate) async fn pin_checkpoint(storage: &LsmStorage, seq_no: u64) -> Result<()>` (aufgerufen in `src/lsm/engine.rs:623`)
- `pub(crate) async fn unpin_checkpoint(storage: &LsmStorage, seq_no: u64) -> Result<()>` (aufgerufen in `src/lsm/engine.rs:627`)
- `pub(crate) async fn stats(storage: &LsmStorage) -> Result<StorageStats>` (aufgerufen in `src/lsm/engine.rs:635`)
- `pub(crate) async fn last_seq_no(storage: &LsmStorage) -> Result<u64>` (aufgerufen in `src/lsm/engine.rs:639`)
- `pub(crate) async fn last_tx_id(storage: &LsmStorage) -> Result<TxId>` (aufgerufen in `src/lsm/engine.rs:643`)

### Aus `src/lsm/observer.rs`:
- `pub const DEFAULT_MAX_OBSERVER_LATENCY: Duration` (re-exportiert in `src/lsm/mod.rs` & `src/lib.rs`)
- `pub enum WriteOrigin { UserWrite }` (re-exportiert in `src/lsm/mod.rs` & `src/lib.rs`)
- `pub struct WalEntryRef<'a>` (re-exportiert in `src/lsm/mod.rs` & `src/lib.rs`)
- `pub struct CommittedBatch<'a>` (re-exportiert in `src/lsm/mod.rs` & `src/lib.rs`)
- `pub struct CommitContext` (re-exportiert in `src/lsm/mod.rs`)
- `pub trait WalObserver: Send + Sync` (re-exportiert in `src/lsm/mod.rs` & `src/lib.rs`)
- `pub struct ObserverRegistry` (re-exportiert in `src/lsm/mod.rs` & `src/lib.rs`)
- `pub struct AsyncObserverAdapter` (re-exportiert in `src/lsm/mod.rs`)

### Aus `src/lsm/validate.rs`:
- `pub fn validate_key(key: &[u8]) -> Result<()>` (re-exportiert in `src/lsm/mod.rs`)
- `pub fn validate_value(value: &[u8]) -> Result<()>`
- `pub fn derive_doc_id(key: &[u8]) -> DocId`

### Aus `src/lsm/lock_order.rs`:
- `pub const RANK_DOCUMENT_INTENT: u32 = 100;`
- `pub const RANK_COMMIT_MUTEX: u32 = 200;`
- `pub const RANK_FLUSH_MUTEX: u32 = 300;`
- `pub const RANK_STATE_RWLOCK: u32 = 400;`
- `pub const RANK_WAL_TRUNCATE: u32 = 500;`
- `pub const RANK_SSTABLES_RWLOCK: u32 = 600;`
- `pub const RANK_MEMTABLE_SHARDS: u32 = 700;`
- `pub const RANK_MANIFEST_MUTEX: u32 = 800;`
- `pub const RANK_OBSERVER_REGISTRY: u32 = 900;`
- `pub const RANK_BLOCK_CACHE: u32 = 950;`

### Aus `src/lsm/fence.rs`:
- `pub struct FlushPurgeFence`
- `pub fn publish_visibility(target: &AtomicU64, new_tx: u64)`
- `pub fn load_snapshot_tx(target: &AtomicU64) -> u64`

### Aus `src/lsm/commit.rs`:
- `LsmStorage::notify_commit_observers`
- `LsmStorage::cleanup_intent_locks_where`
- `LsmStorage::clear_intent_locks_for_tx`
- `LsmStorage::clear_intent_locks_above_tx`
- `LsmStorage::cleanup_intent_locks_for_tx`
- `LsmStorage::cleanup_intent_locks_for_txs`
- `LsmStorage::apply_backpressure`
- `LsmStorage::advance_visibility`
- `LsmStorage::apply_mem_updates`

## 2. Konsumiert von F/A/B/C:
- `LsmStorage` Strukturfelder (read-only): `commit_mutex`, `pending_commit_queue`, `state`, `intent_locks`, `budget`, `config`, `last_committed_tx`, `last_applied_seq`, `compaction_engine`, `wal`, `tx_buffer`, `observer_registry`.
- `LsmState` und `CommitGuard` aus `guard.rs` (gehören Agent F).
