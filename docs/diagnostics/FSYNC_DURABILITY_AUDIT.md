# Fsync & Durability Tracer (INV-FSYNC-1 / M4)

> Generiert durch `tools/fsync_durability_tracer.py`.
> Erlaubte Low-Level I/O Crates: `['contextra-durable-fs', 'contextra-sys']`

## Crate Übersicht (Produktivpfade)

| Crate | Status | Rohe FS-Aufrufe (Prod) | Fsync Aufrufe | Swallowed Syncs (`let _ =`) |
|---|---|---|---|---|
| `contextra` | **ABWEICHUNG** | 6 | 0 | 0 |
| `contextra-adapt` | SAUBER | 0 | 0 | 0 |
| `contextra-agent` | SAUBER | 0 | 0 | 0 |
| `contextra-audit-export` | SAUBER | 0 | 0 | 0 |
| `contextra-avv-generator` | SAUBER | 0 | 0 | 0 |
| `contextra-checkpoint` | **ABWEICHUNG** | 5 | 0 | 0 |
| `contextra-cognition` | SAUBER | 0 | 0 | 0 |
| `contextra-core` | SAUBER | 0 | 0 | 0 |
| `contextra-crypto` | SAUBER | 0 | 1 | 1 |
| `contextra-db` | **ABWEICHUNG** | 6 | 0 | 0 |
| `contextra-durable-fs` | Low-Level (OK) | 13 | 0 | 0 |
| `contextra-engine` | SAUBER | 0 | 0 | 0 |
| `contextra-erasure` | SAUBER | 0 | 12 | 0 |
| `contextra-graph` | SAUBER | 0 | 0 | 0 |
| `contextra-infer-candle` | **ABWEICHUNG** | 11 | 0 | 0 |
| `contextra-infer-ollama` | SAUBER | 0 | 0 | 0 |
| `contextra-infer-onnx` | **ABWEICHUNG** | 13 | 2 | 1 |
| `contextra-ingest` | SAUBER | 0 | 0 | 0 |
| `contextra-kvcache` | SAUBER | 0 | 0 | 0 |
| `contextra-license` | **ABWEICHUNG** | 3 | 0 | 0 |
| `contextra-mcp` | **ABWEICHUNG** | 5 | 0 | 0 |
| `contextra-mvcc` | SAUBER | 0 | 0 | 0 |
| `contextra-ports` | SAUBER | 0 | 1 | 0 |
| `contextra-privacy` | SAUBER | 0 | 0 | 0 |
| `contextra-py` | SAUBER | 0 | 0 | 0 |
| `contextra-rank` | SAUBER | 0 | 0 | 0 |
| `contextra-router` | **ABWEICHUNG** | 1 | 0 | 0 |
| `contextra-sandbox` | SAUBER | 0 | 0 | 0 |
| `contextra-simd` | SAUBER | 0 | 0 | 0 |
| `contextra-store` | **ABWEICHUNG** | 105 | 20 | 0 |
| `contextra-sys` | Low-Level (OK) | 56 | 2 | 0 |
| `contextra-testkit` | SAUBER | 0 | 2 | 0 |
| `contextra-text` | SAUBER | 0 | 0 | 0 |
| `contextra-types` | SAUBER | 0 | 0 | 0 |
| `contextra-vector` | **ABWEICHUNG** | 43 | 9 | 0 |
| `contextra-wire` | **ABWEICHUNG** | 2 | 0 | 0 |

## Auffälligkeiten & Durability-Risiken

### `contextra`

- `/app/crates/contextra/src/lib.rs:187` [std_fs]: `let _ = std::fs::remove_dir_all(&base_tmp);`
- `/app/crates/contextra/src/lib.rs:205` [std_fs]: `let _ = std::fs::remove_dir_all(&base_tmp);`
- `/app/crates/contextra/src/builder.rs:516` [std_fs]: `let _ = std::fs::remove_dir_all(&tmp_path);`
- `/app/crates/contextra/src/builder.rs:527` [std_fs]: `let _ = std::fs::remove_dir_all(&tmp_path);`
- `/app/crates/contextra/src/builder.rs:537` [std_fs]: `let _ = std::fs::remove_dir_all(&tmp_path);`
- `/app/crates/contextra/src/builder.rs:546` [std_fs]: `let _ = std::fs::remove_dir_all(&tmp_path);`
### `contextra-checkpoint`

- `/app/crates/contextra-checkpoint/src/hardlink_cloner.rs:62` [tokio_fs]: `use tokio::fs;`
- `/app/crates/contextra-checkpoint/src/orphan.rs:251` [std_fs]: `std::fs::write(&self.persist_path, data)`
- `/app/crates/contextra-checkpoint/src/orphan.rs:266` [std_fs]: `if let Ok(data) = std::fs::read(path) {`
- `/app/crates/contextra-checkpoint/src/orphan.rs:375` [std_fs]: `if let Ok(data) = std::fs::read(path) {`
- `/app/crates/contextra-checkpoint/src/orphan.rs:425` [std_fs]: `std::fs::write(&self.persist_path, data)`
### `contextra-crypto`

- `/app/crates/contextra-crypto/src/revocation_log.rs:529` [sync_dir] [SWALLOWED ERROR!]: `let _ = vfs.sync_dir(p);`
### `contextra-db`

- `/app/crates/contextra-db/examples/hybrid_search.rs:104` [tokio_fs]: `let _ = tokio::fs::remove_dir_all("./hybrid_data").await;`
- `/app/crates/contextra-db/examples/quickstart.rs:72` [tokio_fs]: `let _ = tokio::fs::remove_dir_all("./example_data").await;`
- `/app/crates/contextra-db/examples/collections.rs:129` [tokio_fs]: `let _ = tokio::fs::remove_dir_all("./collections_data").await;`
- `/app/crates/contextra-db/src/lib.rs:691` [std_fs]: `std::fs::create_dir_all(&model_dir).expect("create model dir");`
- `/app/crates/contextra-db/src/lib.rs:700` [std_fs]: `std::fs::copy(fixture_dir.join("model.onnx"), model_dir.join("model.onnx"))`
- `/app/crates/contextra-db/src/lib.rs:702` [std_fs]: `std::fs::copy(`
### `contextra-infer-candle`

- `/app/crates/contextra-infer-candle/src/gguf_loader.rs:8` [std_fs]: `use std::fs::File;`
- `/app/crates/contextra-infer-candle/src/gguf_loader.rs:50` [std_file]: `let mut file = File::open(model_path).map_err(|e| {`
- `/app/crates/contextra-infer-candle/src/model_registry.rs:9` [std_fs]: `use std::fs::File;`
- `/app/crates/contextra-infer-candle/src/model_registry.rs:61` [std_file]: `let file = File::open(model_path).map_err(|e| {`
- `/app/crates/contextra-infer-candle/src/embedding.rs:128` [std_fs]: `} else if let Ok(entries) = std::fs::read_dir(model_dir) {`
- `/app/crates/contextra-infer-candle/src/embedding.rs:186` [std_fs]: `let weights_bytes = std::fs::read(weights_path).map_err(|e| {`
- `/app/crates/contextra-infer-candle/src/inference/core.rs:26` [std_fs]: `use std::fs::File;`
- `/app/crates/contextra-infer-candle/src/inference/core.rs:385` [std_fs]: `let entries = std::fs::read_dir(model_dir).map_err(|e| ContextraError::ModelLoad {`
- `/app/crates/contextra-infer-candle/src/inference/core.rs:452` [std_file]: `let mut file = File::open(model_path).map_err(|e| {`
- `/app/crates/contextra-infer-candle/src/inference/quantized_llama.rs:15` [std_fs]: `use std::fs::File;`
- `/app/crates/contextra-infer-candle/src/inference/quantized_llama.rs:36` [std_file]: `let mut file = File::open(model_path).map_err(|e| {`
### `contextra-infer-onnx`

- `/app/crates/contextra-infer-onnx/src/lib.rs:336` [std_fs]: `let mut f = std::fs::File::create(&tmp_path).map_err(|e| {`
- `/app/crates/contextra-infer-onnx/src/lib.rs:336` [std_file]: `let mut f = std::fs::File::create(&tmp_path).map_err(|e| {`
- `/app/crates/contextra-infer-onnx/src/lib.rs:353` [std_fs]: `std::fs::rename(&tmp_path, &dest_path_buf).map_err(|e| {`
- `/app/crates/contextra-infer-onnx/src/lib.rs:378` [std_fs]: `std::fs::create_dir_all(&d).map_err(|e| {`
- `/app/crates/contextra-infer-onnx/src/lib.rs:930` [std_fs]: `use std::fs::File;`
- `/app/crates/contextra-infer-onnx/src/lib.rs:948` [std_file]: `File::create(dir.path().join("tokenizer.json"))?;`
- `/app/crates/contextra-infer-onnx/src/lib.rs:996` [std_file]: `File::create(dir.path().join("model.onnx"))?.write_all(b"invalid")?;`
- `/app/crates/contextra-infer-onnx/src/lib.rs:997` [std_file]: `File::create(dir.path().join("tokenizer.json"))?.write_all(b"invalid")?;`
- `/app/crates/contextra-infer-onnx/src/lib.rs:1035` [std_fs]: `std::fs::create_dir_all(&model_dir).unwrap();`
- `/app/crates/contextra-infer-onnx/src/lib.rs:1036` [std_fs]: `std::fs::File::create(model_dir.join("model.onnx")).unwrap();`
- `/app/crates/contextra-infer-onnx/src/lib.rs:1036` [std_file]: `std::fs::File::create(model_dir.join("model.onnx")).unwrap();`
- `/app/crates/contextra-infer-onnx/src/lib.rs:1037` [std_fs]: `std::fs::File::create(model_dir.join("tokenizer.json")).unwrap();`
- `/app/crates/contextra-infer-onnx/src/lib.rs:1037` [std_file]: `std::fs::File::create(model_dir.join("tokenizer.json")).unwrap();`
- `/app/crates/contextra-infer-onnx/src/lib.rs:1270` [sync_dir] [SWALLOWED ERROR!]: `let _ = vfs.sync_dir(base);`
### `contextra-license`

- `/app/crates/contextra-license/src/signed_gate.rs:239` [std_fs]: `if let Ok(content) = std::fs::read_to_string(&id_file) {`
- `/app/crates/contextra-license/src/signed_gate.rs:249` [std_fs]: `if std::fs::create_dir_all(path).is_ok() {`
- `/app/crates/contextra-license/src/signed_gate.rs:250` [std_fs]: `if let Ok(()) = std::fs::write(&id_file, &generated) {}`
### `contextra-mcp`

- `/app/crates/contextra-mcp/src/config.rs:507` [std_fs]: `if let Ok(bytes) = std::fs::read(path) {`
- `/app/crates/contextra-mcp/src/prompt_injection/guard.rs:189` [std_fs]: `let content = std::fs::read_to_string(path)`
- `/app/crates/contextra-mcp/src/prompt_injection/guard.rs:233` [std_fs]: `if let Ok(content) = std::fs::read_to_string(&patterns_file) {`
- `/app/crates/contextra-mcp/src/prompt_injection/audit.rs:2` [std_fs]: `use std::fs::OpenOptions;`
- `/app/crates/contextra-mcp/src/bin/contextra-mcp-server.rs:62` [std_fs]: `if let Ok(bytes) = std::fs::read(&path) {`
### `contextra-router`

- `/app/crates/contextra-router/src/router/lifecycle.rs:67` [std_fs]: `std::fs::read(path).ok()`
### `contextra-store`

- `/app/crates/contextra-store/examples/chaos_writer.rs:10` [std_fs]: `use std::fs::OpenOptions;`
- `/app/crates/contextra-store/examples/store_standalone.rs:40` [std_fs]: `let _ = std::fs::remove_dir_all(&tmp_dir);`
- `/app/crates/contextra-store/src/util.rs:15` [std_fs]: `use std::fs::{File, TryLockError};`
- `/app/crates/contextra-store/src/util.rs:33` [std_file]: `let file = File::create(&lock_path).map_err(|e| {`
- `/app/crates/contextra-store/src/fsync.rs:14` [std_file]: `let file = crate::wal::fs::File::open(path).await.map_err(|e| {`
- `/app/crates/contextra-store/src/fsync.rs:53` [std_file]: `let dir = crate::wal::fs::File::open(dir_path).await.map_err(|e| {`
- `/app/crates/contextra-store/src/fsync.rs:94` [tokio_fs]: `tokio::fs::write(&file_path, b"test payload")`
- `/app/crates/contextra-store/src/scanner/sstable.rs:98` [tokio_fs]: `let _ = tokio::fs::remove_file(&canary_path).await;`
- `/app/crates/contextra-store/src/scanner/sstable.rs:151` [tokio_fs]: `if let Ok(mut dir) = tokio::fs::read_dir(data_dir).await {`
- `/app/crates/contextra-store/src/scanner/purge.rs:117` [tokio_fs]: `if let Ok(mut dir) = tokio::fs::read_dir(data_dir).await {`
- `/app/crates/contextra-store/src/scanner/purge.rs:129` [tokio_fs]: `if let Ok(metadata) = tokio::fs::metadata(&old_path).await {`
- `/app/crates/contextra-store/src/scanner/purge.rs:132` [tokio_fs]: `let _ = tokio::fs::write(&old_path, zeros).await;`
- `/app/crates/contextra-store/src/scanner/purge.rs:133` [tokio_fs]: `if tokio::fs::remove_file(&old_path).await.is_ok() {`
- `/app/crates/contextra-store/src/scanner/purge.rs:141` [tokio_fs]: `if let Ok(mut dir) = tokio::fs::read_dir(data_dir).await {`
- `/app/crates/contextra-store/src/scanner/purge.rs:201` [tokio_fs]: `if let Ok(metadata) = tokio::fs::metadata(&wal_path).await {`
- `/app/crates/contextra-store/src/scanner/purge.rs:204` [tokio_fs]: `let _ = tokio::fs::write(&wal_path, zeros).await;`
- `/app/crates/contextra-store/src/scanner/purge.rs:205` [tokio_fs]: `let _ = tokio::fs::remove_file(&wal_path).await;`
- `/app/crates/contextra-store/src/scanner/purge.rs:220` [tokio_fs]: `if let Ok(metadata) = tokio::fs::metadata(&wal_path).await {`
- `/app/crates/contextra-store/src/scanner/purge.rs:223` [tokio_fs]: `let _ = tokio::fs::write(&wal_path, zeros).await;`
- `/app/crates/contextra-store/src/scanner/purge.rs:225` [tokio_fs]: `let _ = tokio::fs::rename(&tmp_wal_path, &wal_path).await;`
- `/app/crates/contextra-store/src/scanner/wal.rs:68` [tokio_fs]: `let _ = tokio::fs::remove_file(&canary_path).await;`
- `/app/crates/contextra-store/src/scanner/wal.rs:88` [tokio_fs]: `if let Ok(mut dir) = tokio::fs::read_dir(data_dir).await {`
- `/app/crates/contextra-store/src/lsm/engine.rs:349` [tokio_fs]: `let mut entries = match tokio::fs::read_dir(path).await {`
- `/app/crates/contextra-store/src/lsm/engine.rs:414` [tokio_fs]: `let mut entries = match tokio::fs::read_dir(&config.path).await {`
- `/app/crates/contextra-store/src/lsm/engine.rs:430` [tokio_fs]: `let salt = tokio::fs::read(&salt_path).await.map_err(|e| {`
- `/app/crates/contextra-store/src/lsm/recovery.rs:56` [tokio_fs]: `match tokio::fs::OpenOptions::new()`
- `/app/crates/contextra-store/src/lsm/recovery.rs:98` [tokio_fs]: `tokio::fs::rename(&tmp_path, salt_path).await.map_err(|e| {`
- `/app/crates/contextra-store/src/lsm/recovery.rs:108` [tokio_fs]: `if let Err(e) = tokio::fs::remove_file(&tmp_path).await {`
- `/app/crates/contextra-store/src/lsm/recovery.rs:119` [tokio_fs]: `let mut entries = tokio::fs::read_dir(path).await.map_err(|e| {`
- `/app/crates/contextra-store/src/lsm/recovery.rs:163` [tokio_fs]: `tokio::fs::create_dir_all(&config.path)`
- `/app/crates/contextra-store/src/lsm/recovery.rs:172` [tokio_fs]: `let salt = match tokio::fs::read(&salt_path).await {`
- `/app/crates/contextra-store/src/lsm/recovery.rs:205` [tokio_fs]: `let mut entries = tokio::fs::read_dir(&config.path)`
- `/app/crates/contextra-store/src/lsm/recovery.rs:394` [tokio_fs]: `let mut entries = tokio::fs::read_dir(&config.path)`
- `/app/crates/contextra-store/src/lsm/recovery.rs:421` [tokio_fs]: `let mut entries = tokio::fs::read_dir(&config.path)`
- `/app/crates/contextra-store/src/lsm/recovery.rs:437` [tokio_fs]: `if let Err(e) = tokio::fs::remove_file(&path).await {`
- `/app/crates/contextra-store/src/lsm/recovery.rs:450` [tokio_fs]: `if let Err(e) = tokio::fs::remove_file(&path).await {`
- `/app/crates/contextra-store/src/lsm/recovery.rs:638` [tokio_fs]: `if let Err(e) = tokio::fs::remove_file(old_wal_path).await {`
- `/app/crates/contextra-store/src/lsm/recovery.rs:644` [tokio_fs]: `if let Err(e) = tokio::fs::remove_file(&uuid_sidecar).await {`
- `/app/crates/contextra-store/src/lsm/recovery.rs:707` [tokio_fs]: `tokio::fs::write(&intent_path, &intent_bytes)`
- `/app/crates/contextra-store/src/lsm/recovery.rs:715` [std_fs]: `std::fs::File::open(&parent)`
- `/app/crates/contextra-store/src/lsm/recovery.rs:715` [std_file]: `std::fs::File::open(&parent)`
- `/app/crates/contextra-store/src/lsm/recovery.rs:842` [tokio_fs]: `if let Err(e) = tokio::fs::remove_file(&path).await {`
- `/app/crates/contextra-store/src/lsm/recovery.rs:860` [tokio_fs]: `if let Err(e) = tokio::fs::remove_file(&intent_path).await {`
- `/app/crates/contextra-store/src/compaction/engine.rs:262` [tokio_fs]: `if let Err(e) = tokio::fs::remove_file(&output_path).await {`
- `/app/crates/contextra-store/src/compaction/engine.rs:312` [tokio_fs]: `if let Err(e) = tokio::fs::remove_file(path).await {`
- `/app/crates/contextra-store/src/compaction/engine.rs:316` [tokio_fs]: `if matches!(tokio::fs::try_exists(&uuid_sidecar).await, Ok(true)) {`
- `/app/crates/contextra-store/src/compaction/engine.rs:317` [tokio_fs]: `if let Err(e) = tokio::fs::remove_file(&uuid_sidecar).await {`
- `/app/crates/contextra-store/src/compaction/engine.rs:433` [tokio_fs]: `if let Err(e) = tokio::fs::remove_file(output_path).await {`
- `/app/crates/contextra-store/src/wal/replay.rs:149` [std_fs]: `let std_file = std::fs::File::open(&self.path)`
- `/app/crates/contextra-store/src/wal/replay.rs:149` [std_file]: `let std_file = std::fs::File::open(&self.path)`
- `/app/crates/contextra-store/src/wal/replay.rs:209` [std_fs]: `let std_file = std::fs::File::open(&self.path).map_err(|e| {`
- `/app/crates/contextra-store/src/wal/replay.rs:209` [std_file]: `let std_file = std::fs::File::open(&self.path).map_err(|e| {`
- `/app/crates/contextra-store/src/wal/io.rs:969` [std_file]: `let mut file = super::super::fs::File::create(&file_path)`
- `/app/crates/contextra-store/src/wal/mod.rs:22` [tokio_fs]: `pub use tokio::fs::*;`
- `/app/crates/contextra-store/src/wal/mod.rs:32` [std_fs]: `std::fs::read(path)`
- `/app/crates/contextra-store/src/wal/mod.rs:40` [std_fs]: `std::fs::write(path, contents)`
- `/app/crates/contextra-store/src/wal/mod.rs:65` [std_fs]: `perm: std::fs::Permissions,`
- `/app/crates/contextra-store/src/wal/mod.rs:82` [std_fs]: `pub fn permissions(&self) -> std::fs::Permissions {`
- `/app/crates/contextra-store/src/wal/mod.rs:87` [std_fs]: `std::fs::Permissions::from_mode(0o644)`
- `/app/crates/contextra-store/src/wal/mod.rs:92` [std_fs]: `let mut perm_res = std::fs::metadata(".").map(|m| m.permissions());`
- `/app/crates/contextra-store/src/wal/mod.rs:94` [std_fs]: `perm_res = std::fs::metadata(std::env::temp_dir()).map(|m| m.permissions());`
- `/app/crates/contextra-store/src/wal/mod.rs:97` [std_fs]: `perm_res = std::fs::metadata("NUL").map(|m| m.permissions());`
- `/app/crates/contextra-store/src/wal/mod.rs:103` [std_fs]: `let p = std::fs::File::create(&tmp)`
- `/app/crates/contextra-store/src/wal/mod.rs:103` [std_file]: `let p = std::fs::File::create(&tmp)`
- `/app/crates/contextra-store/src/wal/mod.rs:106` [std_fs]: `let _ = std::fs::remove_file(tmp);`
- `/app/crates/contextra-store/src/wal/mod.rs:108` [std_fs]: `std::fs::metadata(std::env::current_dir().unwrap_or_default())`
- `/app/crates/contextra-store/src/wal/mod.rs:111` [std_fs]: `std::fs::metadata(".")`
- `/app/crates/contextra-store/src/wal/mod.rs:114` [std_fs]: `std::fs::metadata(std::env::temp_dir())`
- `/app/crates/contextra-store/src/wal/mod.rs:117` [std_fs]: `std::fs::metadata(`
- `/app/crates/contextra-store/src/wal/mod.rs:125` [std_fs]: `let p2 = std::fs::File::create(&local_tmp)`
- `/app/crates/contextra-store/src/wal/mod.rs:125` [std_file]: `let p2 = std::fs::File::create(&local_tmp)`
- `/app/crates/contextra-store/src/wal/mod.rs:128` [std_fs]: `let _ = std::fs::remove_file(local_tmp);`
- `/app/crates/contextra-store/src/wal/mod.rs:130` [std_fs]: `if let Ok(m) = std::fs::metadata(".") {`
- `/app/crates/contextra-store/src/manifest/core.rs:29` [tokio_fs]: `pub(super) file: tokio::sync::Mutex<tokio::fs::File>,`
- `/app/crates/contextra-store/src/manifest/core.rs:53` [tokio_fs]: `let (mut file, is_new) = match tokio::fs::OpenOptions::new()`
- `/app/crates/contextra-store/src/manifest/core.rs:62` [tokio_fs]: `let file = tokio::fs::OpenOptions::new()`
- `/app/crates/contextra-store/src/manifest/core.rs:152` [tokio_fs]: `if !tokio::fs::try_exists(path).await.unwrap_or(false) {`
- `/app/crates/contextra-store/src/manifest/core.rs:156` [std_file]: `let mut file = match tokio::fs::File::open(path).await {`
- `/app/crates/contextra-store/src/manifest/core.rs:156` [tokio_fs]: `let mut file = match tokio::fs::File::open(path).await {`
- `/app/crates/contextra-store/src/manifest/rollover.rs:80` [tokio_fs]: `match tokio::fs::OpenOptions::new()`
- `/app/crates/contextra-store/src/manifest/rollover.rs:141` [tokio_fs]: `tokio::fs::rename(&tmp_path, &self.path)`
- `/app/crates/contextra-store/src/manifest/rollover.rs:152` [tokio_fs]: `let reopened_file = tokio::fs::OpenOptions::new()`
- `/app/crates/contextra-store/src/manifest/rollover.rs:177` [tokio_fs]: `if let Err(clean_err) = tokio::fs::remove_file(&tmp_path).await {`
- `/app/crates/contextra-store/src/sstable/io.rs:7` [std_fs]: `file: &std::fs::File,`
- `/app/crates/contextra-store/src/sstable/io.rs:17` [std_fs]: `file: &std::fs::File,`
- `/app/crates/contextra-store/src/sstable/builder.rs:13` [tokio_fs]: `use tokio::fs::File;`
- `/app/crates/contextra-store/src/sstable/builder.rs:246` [std_file]: `let file = File::create(path_ref)`
- `/app/crates/contextra-store/src/sstable/reader.rs:224` [std_fs]: `pub(super) file: Arc<std::fs::File>,`
- `/app/crates/contextra-store/src/sstable/reader.rs:288` [std_fs]: `tokio::task::spawn_blocking(move || -> std::io::Result<(std::fs::File, u64)> {`
- `/app/crates/contextra-store/src/sstable/reader.rs:289` [std_fs]: `let file = std::fs::File::open(&path_for_open)?;`
- `/app/crates/contextra-store/src/sstable/reader.rs:289` [std_file]: `let file = std::fs::File::open(&path_for_open)?;`
- `/app/crates/contextra-store/src/sstable/reader.rs:682` [std_fs]: `file: Arc<std::fs::File>,`
- `/app/crates/contextra-store/src/engine/memory_only.rs:360` [std_fs]: `let entries = std::fs::read_dir(&path).unwrap().count();`
- `/app/crates/contextra-store/src/lsm/ops/compaction.rs:197` [tokio_fs]: `if let Err(e) = tokio::fs::remove_file(&wal_path).await {`
- `/app/crates/contextra-store/src/lsm/ops/compaction.rs:202` [tokio_fs]: `if let Err(e) = tokio::fs::remove_file(&uuid_sidecar).await {`
- `/app/crates/contextra-store/src/lsm/ops/compaction.rs:241` [tokio_fs]: `if let Err(rm_err) = tokio::fs::remove_file(&sst_path).await {`
- `/app/crates/contextra-store/fuzz/fuzz_targets/wal_roundtrip.rs:23` [tokio_fs]: `if tokio::fs::write(&wal_path, data).await.is_err() {`
- `/app/crates/contextra-store/fuzz/fuzz_targets/fuzz_manifest_load.rs:23` [tokio_fs]: `if tokio::fs::write(&manifest_path, data).await.is_err() {`
- `/app/crates/contextra-store/fuzz/fuzz_targets/fuzz_wal_replay.rs:23` [tokio_fs]: `if tokio::fs::write(&wal_path, data).await.is_err() {`
- `/app/crates/contextra-store/fuzz/fuzz_targets/fuzz_wal_replay.rs:35` [tokio_fs]: `let file_size = tokio::fs::metadata(&wal_path)`
- `/app/crates/contextra-store/fuzz/fuzz_targets/wal_mutation_chaos.rs:235` [tokio_fs]: `let mut file_bytes = match tokio::fs::read(&wal_path).await {`
- `/app/crates/contextra-store/fuzz/fuzz_targets/wal_mutation_chaos.rs:277` [tokio_fs]: `if tokio::fs::write(&wal_path, &file_bytes).await.is_err() {`
- `/app/crates/contextra-store/fuzz/fuzz_targets/fuzz_recovery_arbitrary_state.rs:32` [tokio_fs]: `if tokio::fs::write(path.join("MANIFEST"), manifest_bytes).await.is_err() {`
- `/app/crates/contextra-store/fuzz/fuzz_targets/fuzz_recovery_arbitrary_state.rs:38` [tokio_fs]: `if tokio::fs::write(path.join("wal-0.log"), wal_bytes).await.is_err() {`
- `/app/crates/contextra-store/fuzz/fuzz_targets/fuzz_recovery_arbitrary_state.rs:44` [tokio_fs]: `if tokio::fs::write(path.join("000001.sst"), sstable_bytes).await.is_err() {`
### `contextra-vector`

- `/app/crates/contextra-vector/src/diskann/types.rs:12` [memmap]: `use memmap2::Mmap;`
- `/app/crates/contextra-vector/src/diskann/types.rs:83` [std_fs]: `std::fs::write(`
- `/app/crates/contextra-vector/src/diskann/types.rs:114` [std_fs]: `std::fs::write(&path, b"DANN_small").unwrap();`
- `/app/crates/contextra-vector/src/diskann/types.rs:206` [std_fs]: `let mut file = std::fs::File::open(&config.index_path).map_err(ContextraError::Io)?;`
- `/app/crates/contextra-vector/src/diskann/types.rs:206` [std_file]: `let mut file = std::fs::File::open(&config.index_path).map_err(ContextraError::Io)?;`
- `/app/crates/contextra-vector/src/diskann/persistence.rs:50` [std_fs]: `let _ = std::fs::remove_file(&pending_wal);`
- `/app/crates/contextra-vector/src/diskann/persistence.rs:75` [std_fs]: `let tmp_file = std::fs::File::open(&tmp_path)?;`
- `/app/crates/contextra-vector/src/diskann/persistence.rs:75` [std_file]: `let tmp_file = std::fs::File::open(&tmp_path)?;`
- `/app/crates/contextra-vector/src/diskann/persistence.rs:80` [std_fs]: `std::fs::rename(&tmp_path, &index_path)`
- `/app/crates/contextra-vector/src/diskann/persistence.rs:83` [std_fs]: `if let Ok(dir) = std::fs::File::open(parent) {`
- `/app/crates/contextra-vector/src/diskann/persistence.rs:83` [std_file]: `if let Ok(dir) = std::fs::File::open(parent) {`
- `/app/crates/contextra-vector/src/diskann/persistence.rs:91` [std_fs]: `let _ = std::fs::remove_file(&pending_wal);`
- `/app/crates/contextra-vector/src/diskann/persistence.rs:136` [std_fs]: `use std::fs::OpenOptions;`
- `/app/crates/contextra-vector/src/diskann/persistence.rs:272` [std_fs]: `if let Err(e) = std::fs::remove_file(&delta_tmp) {`
- `/app/crates/contextra-vector/src/diskann/persistence.rs:281` [std_fs]: `if let Err(e) = std::fs::remove_file(&idx_tmp) {`
- `/app/crates/contextra-vector/src/diskann/persistence.rs:290` [std_fs]: `std::fs::File::open(&inner.config.index_path).map_err(ContextraError::Io)?;`
- `/app/crates/contextra-vector/src/diskann/persistence.rs:290` [std_file]: `std::fs::File::open(&inner.config.index_path).map_err(ContextraError::Io)?;`
- `/app/crates/contextra-vector/src/diskann/build.rs:109` [std_fs]: `let is_new = match std::fs::metadata(path) {`
- `/app/crates/contextra-vector/src/diskann/build.rs:114` [std_fs]: `let mut file = std::fs::OpenOptions::new()`
- `/app/crates/contextra-vector/src/diskann/build.rs:159` [std_fs]: `let mut file = match std::fs::File::open(path) {`
- `/app/crates/contextra-vector/src/diskann/build.rs:159` [std_file]: `let mut file = match std::fs::File::open(path) {`
- `/app/crates/contextra-vector/src/diskann/build.rs:250` [std_fs]: `let is_new = match std::fs::metadata(path) {`
- `/app/crates/contextra-vector/src/diskann/build.rs:255` [std_fs]: `let mut file = std::fs::OpenOptions::new()`
- `/app/crates/contextra-vector/src/diskann/build.rs:321` [std_fs]: `let mut file = match std::fs::File::open(path) {`
- `/app/crates/contextra-vector/src/diskann/build.rs:321` [std_file]: `let mut file = match std::fs::File::open(path) {`
- `/app/crates/contextra-vector/src/diskann/build.rs:455` [std_fs]: `let _ = std::fs::remove_file(&pending_wal);`
- `/app/crates/contextra-vector/src/diskann/build.rs:722` [std_fs]: `std::fs::rename(&tmp_path, &self.inner.config.index_path).map_err(ContextraError::Io)?;`
- `/app/crates/contextra-vector/src/diskann/build.rs:726` [std_fs]: `let parent_dir = std::fs::File::open(parent).map_err(ContextraError::Io)?;`
- `/app/crates/contextra-vector/src/diskann/build.rs:726` [std_file]: `let parent_dir = std::fs::File::open(parent).map_err(ContextraError::Io)?;`
- `/app/crates/contextra-vector/src/diskann/build.rs:732` [std_fs]: `let _ = std::fs::remove_file(&pending_wal);`
- `/app/crates/contextra-vector/src/diskann/build.rs:737` [std_fs]: `let _ = std::fs::remove_file(&tombstone_wal);`
- `/app/crates/contextra-vector/src/persistence/mmap.rs:13` [memmap]: `pub mmap: std::sync::Arc<memmap2::Mmap>,`
- `/app/crates/contextra-vector/src/persistence/mmap.rs:21` [std_fs]: `let file = std::fs::File::open(&path_buf)`
- `/app/crates/contextra-vector/src/persistence/mmap.rs:21` [std_file]: `let file = std::fs::File::open(&path_buf)`
- `/app/crates/contextra-vector/src/hnsw/types.rs:637` [std_fs]: `let file = std::fs::File::create(&temp_path).map_err(|e| {`
- `/app/crates/contextra-vector/src/hnsw/types.rs:637` [std_file]: `let file = std::fs::File::create(&temp_path).map_err(|e| {`
- `/app/crates/contextra-vector/src/hnsw/types.rs:811` [std_fs]: `std::fs::rename(&temp_path, &path_buf).map_err(|e| {`
- `/app/crates/contextra-vector/src/hnsw/types.rs:816` [std_fs]: `if let Ok(parent_dir) = std::fs::File::open(parent) {`
- `/app/crates/contextra-vector/src/hnsw/types.rs:816` [std_file]: `if let Ok(parent_dir) = std::fs::File::open(parent) {`
- `/app/crates/contextra-vector/fuzz/fuzz_targets/fuzz_hnsw_persistence.rs:8` [std_fs]: `use std::fs::OpenOptions;`
- `/app/crates/contextra-vector/fuzz/fuzz_targets/fuzz_hnsw_persistence.rs:102` [std_fs]: `let _ = std::fs::remove_file(&path);`
- `/app/crates/contextra-vector/fuzz/fuzz_targets/fuzz_hnsw_persistence.rs:124` [std_fs]: `let _ = std::fs::remove_file(&path);`
- `/app/crates/contextra-vector/fuzz/fuzz_targets/fuzz_hnsw_persistence.rs:135` [std_fs]: `let _ = std::fs::remove_file(&path);`
### `contextra-wire`

- `/app/crates/contextra-wire/build.rs:132` [std_fs]: `let content = match std::fs::read_to_string(output_file) {`
- `/app/crates/contextra-wire/build.rs:154` [std_fs]: `if let Err(e) = std::fs::write(output_file, new_content) {`
