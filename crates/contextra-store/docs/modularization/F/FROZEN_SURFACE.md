# Frozen Surface: contextra-store Modul F (LSM-Kern)

Dieses Dokument erfasst alle Typen, Funktionen, Konstanten, Re-Exporte und Methoden innerhalb des Eigentums von Modul F, die von anderen Modulen (innerhalb und außerhalb von `contextra-store`) referenziert werden. Diese Außenfläche ist strikt eingefroren.

## 1. Strukturvertrag mit Agent E (LSM-Schreibpfad)

Die folgenden Strukturen und Felder müssen absolut unverändert bleiben (Namen, Typen, Sichtbarkeiten):

- `LsmStorage` (in `src/lsm/engine.rs`):
  - `pub(super) config: Arc<LsmConfig>`
  - `pub(super) state: Arc<RwLock<LsmState>>`
  - `pub(super) sstable_builder: Arc<parking_lot::Mutex<SstableBuilder>>`
  - `pub(super) wal: Arc<RwLock<Arc<Wal>>>`
  - `pub(super) commit_mutex: Arc<tokio::sync::Mutex<()>>`
  - `pub(super) tx_buffer: Arc<[tokio::sync::Mutex<crate::memtable::TxBufferShard>; 16]>`
  - `pub(super) intent_locks: Arc<parking_lot::Mutex<std::collections::HashMap<Vec<u8>, contextra_ids::TxId>>>`
  - `pub(super) sstables: Arc<RwLock<Arc<[Arc<SstableReader>]>>>`
  - `pub(super) block_cache: Option<Arc<dyn crate::sstable::BlockCache>>`
  - `pub(super) recovery_salt: [u8; 32]`
  - `pub(super) budget: Arc<crate::budget::ResourceTracker>`
  - `pub(super) dir: PathBuf`
  - `pub(super) compaction_controller: Arc<Option<crate::compaction::AdaptiveCompactionController>>`
  - `pub(super) observer_registry: Arc<ObserverRegistry>`
  - `pub(super) pending_commit_queue: Arc<tokio::sync::Mutex<Option<crate::lsm::group_commit::PendingCommitQueue>>>`
  - `pub(super) is_shutting_down: Arc<std::sync::atomic::AtomicBool>`

- Inhärente Methoden auf `LsmStorage`, die von E aufgerufen werden:
  - `self.cleanup_intent_locks_where(...)` (aufgerufen in `src/lsm/commit.rs`)

- In `src/lsm/guard.rs`:
  - `LsmState`
  - `CommitGuard`

- In `src/lsm/mod.rs` (Re-Exporte für E):
  - `pub(super) use crate::memtable::MemTable;`
  - `pub(super) use crate::sstable::{SstableBuilder, SstableReader};`
  - `pub(super) use crate::wal::{Wal, WalOp};`
  - `pub(super) use bytes::Bytes;`
  - `pub(super) use std::path::PathBuf;`
  - `pub(super) use std::sync::atomic::Ordering;`
  - `pub(super) use guard::{CommitGuard, LsmState};`
  - `pub(super) use validate::validate_key;`

## 2. Außenfläche nach außen (andere Crates & lib.rs)

Re-exportiert in `src/lib.rs` oder `src/lsm/mod.rs` und von extern (`contextra-engine`, `contextra-db`, `tests/`, `examples/`, `benches/`) konsumiert:

- `LsmStorage` (`pub struct LsmStorage`)
  - `pub async fn open(config: LsmConfig) -> Result<Self>`
  - `pub async fn open_with_key_source(...)`
  - `pub async fn get(&self, key: &[u8]) -> Result<Option<Bytes>>`
  - `pub async fn get_at_seq(&self, key: &[u8], seq_no: u64) -> Result<Option<Bytes>>`
  - `pub async fn scan_prefix(&self, prefix: &[u8]) -> Result<Vec<(Bytes, Bytes)>>`
  - `pub async fn scan_prefix_bounded(...)`
  - `pub async fn scan_prefix_at(...)`
  - `pub async fn scan_bounded(...)`
  - `pub async fn scan(...)`
  - `pub async fn put(&self, tx_id: TxId, key: &[u8], value: &[u8]) -> Result<()>`
  - `pub async fn put_if_absent(...)`
  - `pub async fn delete_many(...)`
  - `pub async fn delete_prefix(...)`
  - `pub async fn delete(...)`
  - `pub async fn commit(&self, tx_id: TxId) -> Result<()>`
  - `pub async fn rollback(&self, tx_id: TxId) -> Result<()>`
  - `pub async fn flush(&self) -> Result<()>`
  - `pub async fn stats(&self) -> Result<crate::stats::StorageStats>`
  - `pub async fn last_seq_no(&self) -> Result<u64>`
  - `pub async fn last_tx_id(&self) -> Result<TxId>`
  - `pub async fn shutdown(&self) -> Result<()>`
  - `pub fn health(&self) -> StorageHealth`
  - `pub fn recovery_salt(&self) -> [u8; 32]`
  - `pub fn dir(&self) -> &std::path::Path`
  - `pub fn config(&self) -> &LsmConfig`
  - `pub fn block_cache(&self) -> Option<Arc<dyn crate::sstable::BlockCache>>`
  - `pub fn memtables_for_scanner(...)`
  - `pub fn sstables_for_scanner(...)`
  - `pub fn sstables_arc(...)`
  - `pub fn budget_for_test(&self) -> Arc<crate::budget::ResourceTracker>`

- `StorageHealth` (`pub enum StorageHealth`)
  - Varianten: `Healthy`, `Degraded`, `Critical`

- `LsmConfig` (`pub struct LsmConfig`)
  - Alle Felder und Methoden: `new`, `with_dir`, `validate`, etc.

- `DurabilityMode` (`pub enum DurabilityMode`)
  - Varianten: `Full`, `WalNoHmac`, `MemoryOnly`
  - Methode: `validate_durability`

- `DurabilityConfigError` (`pub enum DurabilityConfigError`)

- Konstanten in `src/lsm/mod.rs`:
  - `MAX_KEY_SIZE: usize = 65_535`
  - `MAX_VALUE_SIZE: usize = 134_217_728`
  - `MAX_BATCH_SIZE: usize = 10_000`
  - `MAX_INTERNAL_MERGE_ENTRIES_FACTOR: usize = 8`
  - `MAX_GROUP_COMMIT_BATCH_SIZE: usize = 1_000`
  - `MIN_ENTRIES_FOR_SSTABLE_REBUILD: usize = 8`

- Observer-Re-Exporte (Quelle E, weitergeleitet durch `mod.rs`):
  - `AsyncObserverAdapter`, `CommitContext`, `CommittedBatch`, `ObserverRegistry`, `WalEntryRef`, `WalObserver`, `WriteOrigin`, `DEFAULT_MAX_OBSERVER_LATENCY`

- Freie Funktionen aus `recovery.rs` genutzt in `src/scanner/`, `src/wal/`, `src/manifest/`:
  - `crate::lsm::recovery::next_temp_file_counter() -> u64`
  - `crate::lsm::recovery::generate_crypto_salt() -> Result<[u8; 32]>`
  - `crate::lsm::recovery::write_salt_atomically(path: &Path, salt: &[u8; 32]) -> Result<()>`

- Schnittstelle in `src/lsm/ops/read.rs` genutzt in `src/compat/engine/lsm.rs`:
  - `crate::lsm::ops::read::get(storage, key)`
  - `crate::lsm::ops::read::get_tracked(storage, tx_id, key)`
  - `crate::lsm::ops::read::get_at_seq(storage, key, seq_no)`
  - `crate::lsm::ops::read::get_at_seq_tracked(storage, tx_id, key, seq)`
  - `crate::lsm::ops::read::scan_prefix(storage, prefix)`
  - `crate::lsm::ops::read::scan_prefix_tracked(storage, tx_id, prefix)`
  - `crate::lsm::ops::read::scan_prefix_bounded(storage, prefix, limit, cursor)`
  - `crate::lsm::ops::read::scan_prefix_at(storage, prefix, seq_no)`
  - `crate::lsm::ops::read::scan_bounded(storage, start, end, limit, cursor)`
  - `crate::lsm::ops::read::scan(storage, start, end, limit)`

## 3. Konsumierte Module außerhalb des Eigentums F

- `crate::compaction`: `CompactionConfig`, `CompactionEngine`, `AdaptiveCompactionController`
- `crate::memtable`: `MemTable`, `TxBufferShard`
- `crate::sstable`: `SstableBuilder`, `SstableReader`, `BlockCache`
- `crate::wal`: `Wal`, `WalOp`, `WalEntry`, `PreparedBatch`
- `crate::manifest`: `ManifestCore`, `ManifestEntry`
- `crate::error`: `StoreError`, `StoreResult`
- `crate::budget`: `ResourceBudget`, `ResourceTracker`
- `crate::stats`: `StorageStats`
- `contextra_ids`: `TxId`
