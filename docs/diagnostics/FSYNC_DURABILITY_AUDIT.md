# Fsync & Durability Tracer (INV-FSYNC-1 / M4)

> Generiert durch `tools/fsync_durability_tracer.py`.
> Erlaubte Low-Level I/O Crates: `['contextra-durable-fs', 'contextra-sys']`

## Crate Übersicht (Produktivpfade)

| Crate | Status | Rohe FS-Aufrufe (Prod) | Fsync Aufrufe | Swallowed Syncs (`let _ =`) |
|---|---|---|---|---|
| `contextra` | **ABWEICHUNG** | 8 | 0 | 0 |
| `contextra-checkpoint` | **ABWEICHUNG** | 6 | 0 | 0 |
| `contextra-crypto` | SAUBER | 0 | 0 | 0 |
| `contextra-db` | **ABWEICHUNG** | 3 | 0 | 0 |
| `contextra-durable-fs` | Low-Level (OK) | 17 | 4 | 0 |
| `contextra-engine` | SAUBER | 0 | 0 | 0 |
| `contextra-env` | SAUBER | 0 | 0 | 0 |
| `contextra-erasure` | **ABWEICHUNG** | 5 | 21 | 4 |
| `contextra-graph` | SAUBER | 0 | 0 | 0 |
| `contextra-ids` | SAUBER | 0 | 0 | 0 |
| `contextra-mcp` | SAUBER | 0 | 0 | 0 |
| `contextra-mvcc` | SAUBER | 0 | 0 | 0 |
| `contextra-ports` | SAUBER | 0 | 3 | 0 |
| `contextra-privacy` | SAUBER | 0 | 0 | 0 |
| `contextra-rank` | SAUBER | 0 | 0 | 0 |
| `contextra-simd` | SAUBER | 0 | 0 | 0 |
| `contextra-store` | **ABWEICHUNG** | 118 | 39 | 0 |
| `contextra-sys` | Low-Level (OK) | 59 | 6 | 0 |
| `contextra-testkit` | SAUBER | 0 | 2 | 0 |
| `contextra-text` | SAUBER | 0 | 0 | 0 |
| `contextra-types` | SAUBER | 0 | 0 | 0 |
| `contextra-vault` | SAUBER | 0 | 0 | 0 |
| `contextra-vector` | **ABWEICHUNG** | 23 | 8 | 0 |

## Auffälligkeiten & Durability-Risiken

### `contextra`

- `/home/freddy/Projekte/contextra/crates/contextra/src/builder.rs:473` [std_fs]: `let _ = std::fs::remove_dir_all(&tmp_path);`
- `/home/freddy/Projekte/contextra/crates/contextra/src/builder.rs:484` [std_fs]: `let _ = std::fs::remove_dir_all(&tmp_path);`
- `/home/freddy/Projekte/contextra/crates/contextra/src/builder.rs:494` [std_fs]: `let _ = std::fs::remove_dir_all(&tmp_path);`
- `/home/freddy/Projekte/contextra/crates/contextra/src/builder.rs:503` [std_fs]: `let _ = std::fs::remove_dir_all(&tmp_path);`
- `/home/freddy/Projekte/contextra/crates/contextra/src/lib.rs:181` [std_fs]: `let _ = std::fs::remove_dir_all(&base_tmp);`
- `/home/freddy/Projekte/contextra/crates/contextra/src/lib.rs:199` [std_fs]: `let _ = std::fs::remove_dir_all(&base_tmp);`
- `/home/freddy/Projekte/contextra/crates/contextra/examples/embedded_facade.rs:15` [std_fs]: `let _ = std::fs::remove_dir_all(&tmp_dir);`
- `/home/freddy/Projekte/contextra/crates/contextra/examples/embedded_facade.rs:53` [std_fs]: `let _ = std::fs::remove_dir_all(&tmp_dir);`
### `contextra-checkpoint`

- `/home/freddy/Projekte/contextra/crates/contextra-checkpoint/src/orphan.rs:257` [std_fs]: `std::fs::write(&self.persist_path, data)`
- `/home/freddy/Projekte/contextra/crates/contextra-checkpoint/src/orphan.rs:271` [std_fs]: `if let Ok(data) = std::fs::read(path) {`
- `/home/freddy/Projekte/contextra/crates/contextra-checkpoint/src/orphan.rs:380` [std_fs]: `if let Ok(data) = std::fs::read(path) {`
- `/home/freddy/Projekte/contextra/crates/contextra-checkpoint/src/orphan.rs:430` [std_fs]: `std::fs::write(&self.persist_path, data)`
- `/home/freddy/Projekte/contextra/crates/contextra-checkpoint/src/hardlink_cloner.rs:63` [tokio_fs]: `use tokio::fs;`
- `/home/freddy/Projekte/contextra/crates/contextra-checkpoint/src/fs_seam.rs:6` [std_fs]: `use std::fs;`
### `contextra-db`

- `/home/freddy/Projekte/contextra/crates/contextra-db/examples/collections.rs:129` [tokio_fs]: `let _ = tokio::fs::remove_dir_all("./collections_data").await;`
- `/home/freddy/Projekte/contextra/crates/contextra-db/examples/hybrid_search.rs:104` [tokio_fs]: `let _ = tokio::fs::remove_dir_all("./hybrid_data").await;`
- `/home/freddy/Projekte/contextra/crates/contextra-db/examples/quickstart.rs:72` [tokio_fs]: `let _ = tokio::fs::remove_dir_all("./example_data").await;`
### `contextra-erasure`

- `/home/freddy/Projekte/contextra/crates/contextra-erasure/src/scope/file.rs:115` [sync_dir] [SWALLOWED ERROR!]: `let _ = vfs.sync_dir(parent);`
- `/home/freddy/Projekte/contextra/crates/contextra-erasure/src/scope/single_flight.rs:230` [sync_dir] [SWALLOWED ERROR!]: `let _ = self.vfs.sync_dir(parent);`
- `/home/freddy/Projekte/contextra/crates/contextra-erasure/src/scope/single_flight.rs:330` [sync_dir] [SWALLOWED ERROR!]: `let _ = self.vfs.sync_dir(parent);`
- `/home/freddy/Projekte/contextra/crates/contextra-erasure/src/ledger/storage.rs:5` [std_fs]: `use std::fs::{File, OpenOptions};`
- `/home/freddy/Projekte/contextra/crates/contextra-erasure/src/ledger/storage.rs:116` [std_file]: `let mut file = File::open(&self.path).map_err(LedgerError::from)?;`
- `/home/freddy/Projekte/contextra/crates/contextra-erasure/src/ledger/storage.rs:125` [std_fs]: `std::fs::create_dir_all(parent).map_err(LedgerError::from)?;`
- `/home/freddy/Projekte/contextra/crates/contextra-erasure/src/ledger/storage.rs:157` [std_file]: `if let Ok(parent_file) = File::open(parent) {`
- `/home/freddy/Projekte/contextra/crates/contextra-erasure/src/ledger/storage.rs:158` [sync_all] [SWALLOWED ERROR!]: `let _ = parent_file.sync_all();`
- `/home/freddy/Projekte/contextra/crates/contextra-erasure/src/ledger/storage.rs:181` [std_fs]: `let meta = std::fs::metadata(&self.path).map_err(LedgerError::from)?;`
### `contextra-store`

- `/home/freddy/Projekte/contextra/crates/contextra-store/src/fsync.rs:17` [std_file]: `let file = crate::wal::fs::File::open(path).await.map_err(|e| {`
- `/home/freddy/Projekte/contextra/crates/contextra-store/src/fsync.rs:56` [std_file]: `let dir = crate::wal::fs::File::open(dir_path).await.map_err(|e| {`
- `/home/freddy/Projekte/contextra/crates/contextra-store/src/fsync.rs:97` [tokio_fs]: `tokio::fs::write(&file_path, b"test payload")`
- `/home/freddy/Projekte/contextra/crates/contextra-store/src/io_seam.rs:54` [std_fs]: `std::fs::read(path).map_err(|e| {`
- `/home/freddy/Projekte/contextra/crates/contextra-store/src/io_seam.rs:70` [std_fs]: `let mut std_opts = std::fs::OpenOptions::new();`
- `/home/freddy/Projekte/contextra/crates/contextra-store/src/io_seam.rs:89` [std_fs]: `let mut f = std::fs::OpenOptions::new()`
- `/home/freddy/Projekte/contextra/crates/contextra-store/src/io_seam.rs:101` [std_fs]: `if let Err(e) = std::fs::rename(&tmp_path, path) {`
- `/home/freddy/Projekte/contextra/crates/contextra-store/src/io_seam.rs:102` [std_fs]: `let _ = std::fs::remove_file(&tmp_path);`
- `/home/freddy/Projekte/contextra/crates/contextra-store/src/io_seam.rs:112` [std_fs]: `std::fs::create_dir_all(path).map_err(StoreError::Io)`
- `/home/freddy/Projekte/contextra/crates/contextra-store/src/io_seam.rs:116` [std_fs]: `std::fs::remove_file(path).map_err(|e| {`
- `/home/freddy/Projekte/contextra/crates/contextra-store/src/io_seam.rs:126` [std_fs]: `std::fs::rename(from, to).map_err(StoreError::Io)`
- `/home/freddy/Projekte/contextra/crates/contextra-store/src/io_seam.rs:130` [std_fs]: `let entries = std::fs::read_dir(path).map_err(StoreError::Io)?;`
- `/home/freddy/Projekte/contextra/crates/contextra-store/src/io_seam.rs:143` [std_fs]: `std::fs::metadata(path).map(|m| m.len()).map_err(|e| {`
- `/home/freddy/Projekte/contextra/crates/contextra-store/src/io_seam.rs:153` [std_fs]: `let entries = std::fs::read_dir(path).map_err(StoreError::Io)?;`
- `/home/freddy/Projekte/contextra/crates/contextra-store/src/io_seam.rs:166` [std_fs]: `let f = std::fs::File::open(path).map_err(|e| {`
- `/home/freddy/Projekte/contextra/crates/contextra-store/src/io_seam.rs:166` [std_file]: `let f = std::fs::File::open(path).map_err(|e| {`
- `/home/freddy/Projekte/contextra/crates/contextra-store/src/io_seam.rs:180` [std_fs]: `file: std::fs::File,`
- `/home/freddy/Projekte/contextra/crates/contextra-store/src/io_seam.rs:185` [std_fs]: `pub fn new(file: std::fs::File) -> Self {`
- `/home/freddy/Projekte/contextra/crates/contextra-store/src/io_seam.rs:190` [std_fs]: `pub fn file(&self) -> &std::fs::File {`
- `/home/freddy/Projekte/contextra/crates/contextra-store/src/util.rs:16` [std_fs]: `use std::fs::{File, TryLockError};`
- `/home/freddy/Projekte/contextra/crates/contextra-store/src/util.rs:90` [std_file]: `let file = File::create(&lock_path).map_err(|e| {`
- `/home/freddy/Projekte/contextra/crates/contextra-store/examples/store_standalone.rs:40` [std_fs]: `let _ = std::fs::remove_dir_all(&tmp_dir);`
- `/home/freddy/Projekte/contextra/crates/contextra-store/examples/chaos_writer.rs:10` [std_fs]: `use std::fs::OpenOptions;`
- `/home/freddy/Projekte/contextra/crates/contextra-store/examples/standalone.rs:26` [std_fs]: `let _ = std::fs::remove_dir_all(&tmp_dir);`
- `/home/freddy/Projekte/contextra/crates/contextra-store/fuzz/fuzz_targets/fuzz_wal_replay.rs:23` [tokio_fs]: `if tokio::fs::write(&wal_path, data).await.is_err() {`
- `/home/freddy/Projekte/contextra/crates/contextra-store/fuzz/fuzz_targets/fuzz_wal_replay.rs:33` [tokio_fs]: `let file_size = tokio::fs::metadata(&wal_path)`
- `/home/freddy/Projekte/contextra/crates/contextra-store/fuzz/fuzz_targets/fuzz_manifest_load.rs:23` [tokio_fs]: `if tokio::fs::write(&manifest_path, data).await.is_err() {`
- `/home/freddy/Projekte/contextra/crates/contextra-store/fuzz/fuzz_targets/fuzz_wal_hmac_chain.rs:22` [tokio_fs]: `if tokio::fs::write(&wal_path, data).await.is_err() {`
- `/home/freddy/Projekte/contextra/crates/contextra-store/fuzz/fuzz_targets/fuzz_recovery_arbitrary_state.rs:32` [tokio_fs]: `if tokio::fs::write(path.join("MANIFEST"), manifest_bytes).await.is_err() {`
- `/home/freddy/Projekte/contextra/crates/contextra-store/fuzz/fuzz_targets/fuzz_recovery_arbitrary_state.rs:38` [tokio_fs]: `if tokio::fs::write(path.join("wal-0.log"), wal_bytes).await.is_err() {`
- `/home/freddy/Projekte/contextra/crates/contextra-store/fuzz/fuzz_targets/fuzz_recovery_arbitrary_state.rs:44` [tokio_fs]: `if tokio::fs::write(path.join("000001.sst"), sstable_bytes).await.is_err() {`
- `/home/freddy/Projekte/contextra/crates/contextra-store/fuzz/fuzz_targets/wal_mutation_chaos.rs:233` [tokio_fs]: `let mut file_bytes = match tokio::fs::read(&wal_path).await {`
- `/home/freddy/Projekte/contextra/crates/contextra-store/fuzz/fuzz_targets/wal_mutation_chaos.rs:275` [tokio_fs]: `if tokio::fs::write(&wal_path, &file_bytes).await.is_err() {`
- `/home/freddy/Projekte/contextra/crates/contextra-store/fuzz/fuzz_targets/wal_roundtrip.rs:23` [tokio_fs]: `if tokio::fs::write(&wal_path, data).await.is_err() {`
- `/home/freddy/Projekte/contextra/crates/contextra-store/fuzz/fuzz_targets/fuzz_sstable_legacy_footer.rs:23` [tokio_fs]: `if tokio::fs::write(&sst_path, data).await.is_err() {`
- `/home/freddy/Projekte/contextra/crates/contextra-store/src/manifest/rollover.rs:80` [tokio_fs]: `match tokio::fs::OpenOptions::new()`
- `/home/freddy/Projekte/contextra/crates/contextra-store/src/manifest/rollover.rs:141` [tokio_fs]: `tokio::fs::rename(&tmp_path, &self.path)`
- `/home/freddy/Projekte/contextra/crates/contextra-store/src/manifest/rollover.rs:152` [tokio_fs]: `let reopened_file = tokio::fs::OpenOptions::new()`
- `/home/freddy/Projekte/contextra/crates/contextra-store/src/manifest/rollover.rs:177` [tokio_fs]: `if let Err(clean_err) = tokio::fs::remove_file(&tmp_path).await {`
- `/home/freddy/Projekte/contextra/crates/contextra-store/src/manifest/core.rs:28` [tokio_fs]: `if !tokio::fs::try_exists(path).await.unwrap_or(false) {`
- `/home/freddy/Projekte/contextra/crates/contextra-store/src/manifest/core.rs:32` [std_file]: `let mut file = match tokio::fs::File::open(path).await {`
- `/home/freddy/Projekte/contextra/crates/contextra-store/src/manifest/core.rs:32` [tokio_fs]: `let mut file = match tokio::fs::File::open(path).await {`
- `/home/freddy/Projekte/contextra/crates/contextra-store/src/manifest/handle.rs:17` [tokio_fs]: `pub(super) file: tokio::sync::Mutex<tokio::fs::File>,`
- `/home/freddy/Projekte/contextra/crates/contextra-store/src/manifest/handle.rs:42` [tokio_fs]: `let (mut file, is_new) = match tokio::fs::OpenOptions::new()`
- `/home/freddy/Projekte/contextra/crates/contextra-store/src/manifest/handle.rs:51` [tokio_fs]: `let file = tokio::fs::OpenOptions::new()`
- `/home/freddy/Projekte/contextra/crates/contextra-store/src/wal/replay.rs:152` [std_fs]: `let std_file = std::fs::File::open(&self.path)`
- `/home/freddy/Projekte/contextra/crates/contextra-store/src/wal/replay.rs:152` [std_file]: `let std_file = std::fs::File::open(&self.path)`
- `/home/freddy/Projekte/contextra/crates/contextra-store/src/wal/replay.rs:212` [std_fs]: `let std_file = std::fs::File::open(&self.path).map_err(|e| {`
- `/home/freddy/Projekte/contextra/crates/contextra-store/src/wal/replay.rs:212` [std_file]: `let std_file = std::fs::File::open(&self.path).map_err(|e| {`
- `/home/freddy/Projekte/contextra/crates/contextra-store/src/wal/fs.rs:6` [tokio_fs]: `pub use tokio::fs::*;`
- `/home/freddy/Projekte/contextra/crates/contextra-store/src/wal/fs.rs:18` [std_fs]: `std::fs::read(path)`
- `/home/freddy/Projekte/contextra/crates/contextra-store/src/wal/fs.rs:24` [std_fs]: `std::fs::write(path, contents)`
- `/home/freddy/Projekte/contextra/crates/contextra-store/src/wal/fs.rs:48` [std_fs]: `perm: std::fs::Permissions,`
- `/home/freddy/Projekte/contextra/crates/contextra-store/src/wal/fs.rs:64` [std_fs]: `pub fn permissions(&self) -> std::fs::Permissions {`
- `/home/freddy/Projekte/contextra/crates/contextra-store/src/wal/fs.rs:68` [std_fs]: `std::fs::Permissions::from_mode(0o644)`
- `/home/freddy/Projekte/contextra/crates/contextra-store/src/wal/fs.rs:72` [std_fs]: `let mut perm_res = std::fs::metadata(".").map(|m| m.permissions());`
- `/home/freddy/Projekte/contextra/crates/contextra-store/src/wal/fs.rs:74` [std_fs]: `perm_res = std::fs::metadata(std::env::temp_dir()).map(|m| m.permissions());`
- `/home/freddy/Projekte/contextra/crates/contextra-store/src/wal/fs.rs:77` [std_fs]: `perm_res = std::fs::metadata("NUL").map(|m| m.permissions());`
- `/home/freddy/Projekte/contextra/crates/contextra-store/src/wal/fs.rs:83` [std_fs]: `let p = std::fs::File::create(&tmp)`
- `/home/freddy/Projekte/contextra/crates/contextra-store/src/wal/fs.rs:83` [std_file]: `let p = std::fs::File::create(&tmp)`
- `/home/freddy/Projekte/contextra/crates/contextra-store/src/wal/fs.rs:86` [std_fs]: `let _ = std::fs::remove_file(tmp);`
- `/home/freddy/Projekte/contextra/crates/contextra-store/src/wal/fs.rs:88` [std_fs]: `std::fs::metadata(std::env::current_dir().unwrap_or_default())`
- `/home/freddy/Projekte/contextra/crates/contextra-store/src/wal/fs.rs:91` [std_fs]: `std::fs::metadata(".")`
- `/home/freddy/Projekte/contextra/crates/contextra-store/src/wal/fs.rs:94` [std_fs]: `std::fs::metadata(std::env::temp_dir())`
- `/home/freddy/Projekte/contextra/crates/contextra-store/src/wal/fs.rs:97` [std_fs]: `std::fs::metadata(`
- `/home/freddy/Projekte/contextra/crates/contextra-store/src/wal/fs.rs:105` [std_fs]: `let p2 = std::fs::File::create(&local_tmp)`
- `/home/freddy/Projekte/contextra/crates/contextra-store/src/wal/fs.rs:105` [std_file]: `let p2 = std::fs::File::create(&local_tmp)`
- `/home/freddy/Projekte/contextra/crates/contextra-store/src/wal/fs.rs:108` [std_fs]: `let _ = std::fs::remove_file(local_tmp);`
- `/home/freddy/Projekte/contextra/crates/contextra-store/src/wal/fs.rs:110` [std_fs]: `if let Ok(m) = std::fs::metadata(".") {`
- `/home/freddy/Projekte/contextra/crates/contextra-store/src/io_seam/types.rs:83` [std_fs]: `std::fs::read(path).map_err(|e| {`
- `/home/freddy/Projekte/contextra/crates/contextra-store/src/io_seam/real_fs.rs:6` [std_fs]: `use std::fs::{File, OpenOptions};`
- `/home/freddy/Projekte/contextra/crates/contextra-store/src/io_seam/real_fs.rs:30` [std_fs]: `std::fs::write(&tmp, data)?;`
- `/home/freddy/Projekte/contextra/crates/contextra-store/src/io_seam/real_fs.rs:31` [std_file]: `let f = File::open(&tmp)?;`
- `/home/freddy/Projekte/contextra/crates/contextra-store/src/io_seam/real_fs.rs:34` [std_fs]: `std::fs::rename(&tmp, path)?;`
- `/home/freddy/Projekte/contextra/crates/contextra-store/src/io_seam/real_fs.rs:42` [std_fs]: `std::fs::create_dir_all(path)?;`
- `/home/freddy/Projekte/contextra/crates/contextra-store/src/io_seam/real_fs.rs:47` [std_fs]: `std::fs::remove_file(path)?;`
- `/home/freddy/Projekte/contextra/crates/contextra-store/src/io_seam/real_fs.rs:52` [std_fs]: `std::fs::rename(from, to)?;`
- `/home/freddy/Projekte/contextra/crates/contextra-store/src/io_seam/real_fs.rs:61` [std_fs]: `for entry in std::fs::read_dir(path)? {`
- `/home/freddy/Projekte/contextra/crates/contextra-store/src/io_seam/real_fs.rs:72` [std_fs]: `Ok(std::fs::metadata(path)?.len())`
- `/home/freddy/Projekte/contextra/crates/contextra-store/src/io_seam/real_fs.rs:77` [std_fs]: `for entry in std::fs::read_dir(path)? {`
- `/home/freddy/Projekte/contextra/crates/contextra-store/src/io_seam/real_fs.rs:87` [std_file]: `let d = File::open(path)?;`
- `/home/freddy/Projekte/contextra/crates/contextra-store/src/lsm/engine/migration.rs:15` [tokio_fs]: `let mut entries = match tokio::fs::read_dir(path).await {`
- `/home/freddy/Projekte/contextra/crates/contextra-store/src/lsm/engine/migration.rs:70` [tokio_fs]: `let mut entries = match tokio::fs::read_dir(&config.path).await {`
- `/home/freddy/Projekte/contextra/crates/contextra-store/src/lsm/engine/migration.rs:86` [tokio_fs]: `let salt = tokio::fs::read(&salt_path).await.map_err(|e| {`
- `/home/freddy/Projekte/contextra/crates/contextra-store/src/lsm/recovery/salt.rs:53` [tokio_fs]: `match tokio::fs::OpenOptions::new()`
- `/home/freddy/Projekte/contextra/crates/contextra-store/src/lsm/recovery/salt.rs:95` [tokio_fs]: `tokio::fs::rename(&tmp_path, salt_path).await.map_err(|e| {`
- `/home/freddy/Projekte/contextra/crates/contextra-store/src/lsm/recovery/salt.rs:105` [tokio_fs]: `if let Err(e) = tokio::fs::remove_file(&tmp_path).await {`
- `/home/freddy/Projekte/contextra/crates/contextra-store/src/lsm/recovery/salt.rs:117` [tokio_fs]: `let mut entries = tokio::fs::read_dir(path).await.map_err(|e| {`
- `/home/freddy/Projekte/contextra/crates/contextra-store/src/lsm/recovery/startup.rs:102` [tokio_fs]: `let mut entries = tokio::fs::read_dir(&config.path)`
- `/home/freddy/Projekte/contextra/crates/contextra-store/src/lsm/recovery/startup.rs:291` [tokio_fs]: `let mut entries = tokio::fs::read_dir(&config.path)`
- `/home/freddy/Projekte/contextra/crates/contextra-store/src/lsm/recovery/startup.rs:318` [tokio_fs]: `let mut entries = tokio::fs::read_dir(&config.path)`
- `/home/freddy/Projekte/contextra/crates/contextra-store/src/lsm/recovery/startup.rs:334` [tokio_fs]: `if let Err(e) = tokio::fs::remove_file(&path).await {`
- `/home/freddy/Projekte/contextra/crates/contextra-store/src/lsm/recovery/startup.rs:347` [tokio_fs]: `if let Err(e) = tokio::fs::remove_file(&path).await {`
- `/home/freddy/Projekte/contextra/crates/contextra-store/src/lsm/recovery/startup.rs:540` [tokio_fs]: `if let Err(e) = tokio::fs::remove_file(old_wal_path).await {`
- `/home/freddy/Projekte/contextra/crates/contextra-store/src/lsm/recovery/startup.rs:546` [tokio_fs]: `if let Err(e) = tokio::fs::remove_file(&uuid_sidecar).await {`
- `/home/freddy/Projekte/contextra/crates/contextra-store/src/lsm/recovery/rollback.rs:47` [tokio_fs]: `tokio::fs::write(&intent_path, &intent_bytes)`
- `/home/freddy/Projekte/contextra/crates/contextra-store/src/lsm/recovery/rollback.rs:55` [std_fs]: `std::fs::File::open(&parent)`
- `/home/freddy/Projekte/contextra/crates/contextra-store/src/lsm/recovery/rollback.rs:55` [std_file]: `std::fs::File::open(&parent)`
- `/home/freddy/Projekte/contextra/crates/contextra-store/src/lsm/recovery/rollback.rs:181` [tokio_fs]: `if let Err(e) = tokio::fs::remove_file(&path).await {`
- `/home/freddy/Projekte/contextra/crates/contextra-store/src/lsm/recovery/rollback.rs:199` [tokio_fs]: `if let Err(e) = tokio::fs::remove_file(&intent_path).await {`
- `/home/freddy/Projekte/contextra/crates/contextra-store/src/compat/engine/memory_only.rs:360` [std_fs]: `let entries = std::fs::read_dir(&path).unwrap().count();`
- `/home/freddy/Projekte/contextra/crates/contextra-store/src/compat/scanner/sstable.rs:82` [tokio_fs]: `let _ = tokio::fs::remove_file(&canary_path).await;`
- `/home/freddy/Projekte/contextra/crates/contextra-store/src/compat/scanner/sstable.rs:150` [tokio_fs]: `if let Ok(mut dir) = tokio::fs::read_dir(data_dir).await {`
- `/home/freddy/Projekte/contextra/crates/contextra-store/src/compat/scanner/purge.rs:141` [tokio_fs]: `if let Ok(metadata) = tokio::fs::metadata(&old_path).await {`
- `/home/freddy/Projekte/contextra/crates/contextra-store/src/compat/scanner/purge.rs:144` [tokio_fs]: `let _ = tokio::fs::write(&old_path, zeros).await;`
- `/home/freddy/Projekte/contextra/crates/contextra-store/src/compat/scanner/purge.rs:145` [tokio_fs]: `if tokio::fs::remove_file(&old_path).await.is_ok() {`
- `/home/freddy/Projekte/contextra/crates/contextra-store/src/compat/scanner/purge.rs:214` [tokio_fs]: `let metadata = tokio::fs::metadata(&wal_path)`
- `/home/freddy/Projekte/contextra/crates/contextra-store/src/compat/scanner/purge.rs:219` [tokio_fs]: `tokio::fs::write(&wal_path, &zeros)`
- `/home/freddy/Projekte/contextra/crates/contextra-store/src/compat/scanner/purge.rs:222` [tokio_fs]: `tokio::fs::remove_file(&wal_path)`
- `/home/freddy/Projekte/contextra/crates/contextra-store/src/compat/scanner/purge.rs:247` [tokio_fs]: `let mut tmp_file = tokio::fs::OpenOptions::new()`
- `/home/freddy/Projekte/contextra/crates/contextra-store/src/compat/scanner/purge.rs:264` [tokio_fs]: `tokio::fs::rename(&wal_path, &trash_wal_path)`
- `/home/freddy/Projekte/contextra/crates/contextra-store/src/compat/scanner/purge.rs:267` [tokio_fs]: `tokio::fs::rename(&tmp_wal_path, &wal_path)`
- `/home/freddy/Projekte/contextra/crates/contextra-store/src/compat/scanner/purge.rs:271` [tokio_fs]: `if let Ok(metadata) = tokio::fs::metadata(&trash_wal_path).await {`
- `/home/freddy/Projekte/contextra/crates/contextra-store/src/compat/scanner/purge.rs:274` [tokio_fs]: `let _ = tokio::fs::write(&trash_wal_path, &zeros).await;`
- `/home/freddy/Projekte/contextra/crates/contextra-store/src/compat/scanner/purge.rs:275` [tokio_fs]: `let _ = tokio::fs::remove_file(&trash_wal_path).await;`
- `/home/freddy/Projekte/contextra/crates/contextra-store/src/compat/scanner/wal.rs:74` [tokio_fs]: `if tokio::fs::write(&canary_path, file_bytes).await.is_ok() {`
- `/home/freddy/Projekte/contextra/crates/contextra-store/src/compat/scanner/wal.rs:95` [tokio_fs]: `let _ = tokio::fs::remove_file(&canary_path).await;`
- `/home/freddy/Projekte/contextra/crates/contextra-store/src/wal/io/acl.rs:25` [std_file]: `let mut file = crate::wal::fs::File::create(&file_path)`
### `contextra-vector`

- `/home/freddy/Projekte/contextra/crates/contextra-vector/fuzz/fuzz_targets/fuzz_diskann_formats.rs:24` [std_fs]: `let _ = std::fs::write(&wal_pwal, data);`
- `/home/freddy/Projekte/contextra/crates/contextra-vector/fuzz/fuzz_targets/fuzz_diskann_formats.rs:25` [std_fs]: `let _ = std::fs::write(&wal_twal, data);`
- `/home/freddy/Projekte/contextra/crates/contextra-vector/fuzz/fuzz_targets/fuzz_hnsw_persistence.rs:17` [std_fs]: `use std::fs::OpenOptions;`
- `/home/freddy/Projekte/contextra/crates/contextra-vector/fuzz/fuzz_targets/fuzz_hnsw_persistence.rs:115` [std_fs]: `let _ = std::fs::remove_file(&path);`
- `/home/freddy/Projekte/contextra/crates/contextra-vector/fuzz/fuzz_targets/fuzz_hnsw_persistence.rs:137` [std_fs]: `let _ = std::fs::remove_file(&path);`
- `/home/freddy/Projekte/contextra/crates/contextra-vector/fuzz/fuzz_targets/fuzz_hnsw_persistence.rs:148` [std_fs]: `let _ = std::fs::remove_file(&path);`
- `/home/freddy/Projekte/contextra/crates/contextra-vector/src/diskann/types.rs:13` [memmap]: `use memmap2::Mmap;`
- `/home/freddy/Projekte/contextra/crates/contextra-vector/src/diskann/io.rs:12` [std_fs]: `use std::fs::{File, OpenOptions};`
- `/home/freddy/Projekte/contextra/crates/contextra-vector/src/diskann/io.rs:28` [std_file]: `File::open(tmp).map_err(|e| VectorError::Storage(format!("fsync tmp: {e}")))?;`
- `/home/freddy/Projekte/contextra/crates/contextra-vector/src/diskann/io.rs:35` [std_fs]: `std::fs::rename(tmp, dest).map_err(|e| VectorError::Storage(format!("rename: {e}")))?;`
- `/home/freddy/Projekte/contextra/crates/contextra-vector/src/diskann/io.rs:38` [std_file]: `if let Ok(dir) = File::open(parent) {`
- `/home/freddy/Projekte/contextra/crates/contextra-vector/src/diskann/io.rs:52` [std_fs]: `let _ = std::fs::remove_file(path);`
- `/home/freddy/Projekte/contextra/crates/contextra-vector/src/diskann/io.rs:63` [std_file]: `let mut file = match File::open(path) {`
- `/home/freddy/Projekte/contextra/crates/contextra-vector/src/diskann/io.rs:81` [std_fs]: `let is_new = match std::fs::metadata(path) {`
- `/home/freddy/Projekte/contextra/crates/contextra-vector/src/diskann/io.rs:106` [std_file]: `File::open(path).map_err(VectorError::from)`
- `/home/freddy/Projekte/contextra/crates/contextra-vector/src/diskann/io.rs:122` [memmap]: `pub(crate) fn open_mmap_readonly(file: &File) -> Result<memmap2::Mmap> {`
- `/home/freddy/Projekte/contextra/crates/contextra-vector/src/persistence/mmap.rs:14` [memmap]: `pub mmap: std::sync::Arc<memmap2::Mmap>,`
- `/home/freddy/Projekte/contextra/crates/contextra-vector/src/persistence/atomic.rs:7` [std_fs]: `use std::fs::File;`
- `/home/freddy/Projekte/contextra/crates/contextra-vector/src/persistence/io.rs:7` [std_fs]: `use std::fs::{File, OpenOptions};`
- `/home/freddy/Projekte/contextra/crates/contextra-vector/src/persistence/io.rs:14` [std_file]: `File::open(path)`
- `/home/freddy/Projekte/contextra/crates/contextra-vector/src/persistence/io.rs:43` [std_fs]: `std::fs::rename(from, to)`
- `/home/freddy/Projekte/contextra/crates/contextra-vector/src/persistence/io.rs:49` [std_fs]: `std::fs::remove_file(path)`
- `/home/freddy/Projekte/contextra/crates/contextra-vector/src/persistence/io.rs:56` [std_file]: `let dir = File::open(parent)?;`
