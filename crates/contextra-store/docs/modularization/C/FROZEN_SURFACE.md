# Eingefrorene Außenfläche: STORE-MOD-C (Manifest & Memtable)

Die Außenfläche umfasst alle Typen, Funktionen, Konstanten und Signaturen, die außerhalb der Module `manifest` und `memtable` referenziert werden. Diese Schnittstellen bleiben 100% kompatibel.

## 1. Modul `crate::manifest`

### Re-Exporte in `crates/contextra-store/src/lib.rs`
- `pub use manifest::{Manifest, ManifestEntry};`

### Typen und Signaturen (`src/manifest/mod.rs` & `src/manifest/core.rs`)
- `pub struct Manifest { ... }`
  - `pub async fn open(path: impl AsRef<Path>) -> StoreResult<Self>`
  - `pub async fn open_with_dir_lock(path: impl AsRef<Path>, dir_lock: Option<Arc<DirLock>>) -> StoreResult<Self>`
  - `pub async fn append(&self, entry: &ManifestEntry) -> StoreResult<()>`
  - `pub async fn append_batch(&self, entries: &[ManifestEntry]) -> StoreResult<()>`
  - `pub async fn sync(&self) -> StoreResult<()>`
  - `pub fn is_poisoned(&self) -> bool`
  - `pub fn poison(&self)`
  - `pub fn path(&self) -> &Path`
  - `pub async fn load(path: impl AsRef<Path>) -> StoreResult<Vec<ManifestEntry>>`
  - `pub fn reconstruct_valid_sstables(entries: &[ManifestEntry]) -> Vec<(PathBuf, u64)>`
  - `pub fn reconstruct_dead_sstables(entries: &[ManifestEntry]) -> HashSet<PathBuf>`
  - `pub fn extract_high_water_mark(entries: &[ManifestEntry]) -> u64`
- `pub enum ManifestEntry { ... }`
  - `Add { path: PathBuf, size_bytes: u64 }`
  - `Remove { path: PathBuf }`
  - `Replace { removed: Vec<PathBuf>, added: Vec<(PathBuf, u64)> }`
  - `WalCheckpoint { hmac: [u8; 32] }`
  - `RollbackComplete { tx_id: u64, restored_ssts: Vec<PathBuf> }`
- Konstanten:
  - `pub const CURRENT_MANIFEST_VERSION: u32 = 1;`
  - `pub const DEFAULT_ROLLOVER_THRESHOLD_BYTES: u64 = 10 * 1024 * 1024;`

---

## 2. Modul `crate::memtable`

### Re-Exporte in `crates/contextra-store/src/lib.rs`
- Modul `pub mod memtable;`

### Typen und Signaturen (`src/memtable.rs`)
- `pub struct MemTable { ... }`
  - `pub fn new() -> Self`
  - `pub fn put(&self, key: Bytes, value: Bytes, seq_no: u64, tx_id: u64)`
  - `pub fn tx_range(&self) -> (u64, u64)`
  - `pub fn rollback(&self, tx_id: u64)`
  - `pub fn get(&self, key: &[u8]) -> Option<(Bytes, u64)>`
  - `pub fn get_at_seq(&self, key: &[u8], seq_no: u64, max_tx: u64) -> Option<(Bytes, u64, u64)>`
  - `pub fn size(&self) -> usize`
  - `pub fn is_empty(&self) -> bool`
  - `pub fn iter(&self) -> Vec<(Bytes, Bytes, u64, u64)>`
  - `pub fn scan_prefix_into(&self, prefix: &[u8], target: &mut BTreeMap<Bytes, (Bytes, u64)>, seq_no: u64, max_tx: TxId)`
  - `pub fn scan_prefix_into_matching<F>(&self, prefix: &[u8], seq_no: u64, max_tx: TxId, target: &mut BTreeMap<Bytes, (Bytes, u64)>, predicate: F) where F: Fn(&[u8], u64, u64) -> bool`
  - `pub fn scan_range_into(&self, start: Bound<&[u8]>, end: Bound<&[u8]>, seq_no: u64, max_tx: TxId, target: &mut BTreeMap<Bytes, (Bytes, u64)>)`
  - `pub fn scan_range_into_matching<F>(&self, start: Bound<&[u8]>, end: Bound<&[u8]>, seq_no: u64, max_tx: TxId, target: &mut BTreeMap<Bytes, (Bytes, u64)>, predicate: F) where F: Fn(&[u8], u64, u64) -> bool`
  - `pub fn iter_latest(&self) -> Vec<(Bytes, Bytes, u64, u64)>`
  - `pub fn shard_entry_counts(&self) -> [usize; SHARD_COUNT]`
  - `pub fn shard_for(key: &[u8]) -> usize`
- `pub const TOMBSTONE_BIT: u64 = 1u64 << 63;`
- `pub fn is_tombstone(seq: u64) -> bool`
- `pub fn is_valid_range_bounds(start: Bound<&[u8]>, end: Bound<&[u8]>) -> bool`
