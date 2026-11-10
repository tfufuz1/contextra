# Öffentliche Oberfläche und API-Konsumenten: contextra-sys

Erstellt gemäß WP-Y3 (AUFTRAG sys).
Eingabe und Referenz für spätere Entkopplungs- und Konsumenten-Aufträge (u. a. `contextra-store`, `contextra-vector`, `contextra-vault`).

## 1. Öffentliche Oberfläche (Public API) von `contextra-sys`

`contextra-sys` stellt autarke, sichere Systemabstraktionen bereit. Die öffentliche API gliedert sich wie folgt:

### 1.1 Re-Exports in `crates/contextra-sys/src/lib.rs`
- **ACL / Dateisystem-Sicherheit:**
  - `set_restrictive_file_acl(path: impl AsRef<Path>) -> Result<()>`
  - `verify_file_acl_owner_only(path: impl AsRef<Path>) -> Result<bool>`
- **Durability / Synchronisation:**
  - `durable_flush_file(file: &mut std::fs::File, sync_metadata: bool) -> Result<()>`
  - `durable_flush_dir(dir_path: impl AsRef<Path>) -> Result<()>`
  - `write_all_durable(file: &mut std::fs::File, data: &[u8]) -> Result<()>`
  - `DurableFile` (Struct mit automatischer Vergiftung bei I/O-Fehlern)
- **Fehlermodell:**
  - `Error` (Strukturiertes Enum mit OS-Errno, `#[non_exhaustive]`)
  - `Result<T>` (Alias für `std::result::Result<T, Error>`)
- **Memory-Locking & Paging-Schutz (`mlock`):**
  - `mem_lock(ptr: *const u8, len: usize) -> std::io::Result<bool>`
  - `mem_lock_typed(ptr: *const u8, len: usize, policy: MemLockPolicy) -> Result<bool>`
  - `mem_lock_with_policy(ptr: *const u8, len: usize, policy: MemLockPolicy) -> std::io::Result<bool>`
  - `mem_unlock(addr: usize, len: usize)`
  - `MemLockPolicy` (`FailClosed`, `Tolerant`)
  - `apply_dont_dump(ptr: *const u8, len: usize) -> Result<()>`
- **Memory-Mapping (`mmap`):**
  - `mmap_readonly(file: &std::fs::File) -> Result<memmap2::Mmap>`
  - `SealedFile` / `SealedMap` / `SealedReadOnly` / `Unsealed` (Typzustands-versiegeltes Mmap)
- **POSIX-Ressourcen:**
  - `ensure_fd_limit(min_limit: u64) -> Result<u64>`
  - `set_fd_limit(limit: u64) -> Result<u64>`
  - `reopen_and_dup2(path: impl AsRef<Path>, target_fd: i32, flags: i32) -> Result<()>`
- **Tresor-Speicherverwaltung (`vault`):**
  - `LockedRegions` (Tracking gesperrter Speicherbereiche und Zeroization)
  - `VaultBuffer` (RAII-geschützter Speicher für sensible Schlüssel)

---

## 2. API-Konsumenten im Workspace (Inventar)

Nachfolgend sind alle tatsächlichen Nutzungen öffentlicher Symbole von `contextra-sys` außerhalb des eigenen Crates dokumentiert.

### 2.1 `contextra-store`
- **`mmap_readonly`**
  - `crates/contextra-store/src/wal/replay.rs:163` (`let mmap = contextra_sys::mmap_readonly(&std_file)`)
  - `crates/contextra-store/src/wal/replay.rs:215` (`let mmap = contextra_sys::mmap_readonly(&std_file).map_err(|e| ... )`)
- **`set_restrictive_file_acl`**
  - `crates/contextra-store/src/wal/io.rs:953` (`contextra_sys::set_restrictive_file_acl(path)`)
- **`verify_file_acl_owner_only`**
  - `crates/contextra-store/src/wal/tests/hmac_tests.rs:176` (`contextra_sys::verify_file_acl_owner_only(&key_path).expect(...)`)
- **`reopen_and_dup2`** (Chaos- und Fehlersimulation)
  - `crates/contextra-store/tests/chaos_dropped_write.rs:66` (`contextra_sys::reopen_and_dup2(&wal_path, fd_num, 0)`)
  - `crates/contextra-store/tests/chaos_dropped_write.rs:123` (`contextra_sys::reopen_and_dup2(&wal_path, fd_num, 2 | 1024)`)
- **`set_fd_limit`**
  - `crates/contextra-store/tests/recovery_fd_exhaustion_regression.rs:70` (`contextra_sys::set_fd_limit(64)`)

### 2.2 `contextra-vector`
- **`mmap_readonly`**
  - `crates/contextra-vector/src/diskann/persistence.rs:291` (`let mmap = contextra_sys::mmap_readonly(&file).map_err(ContextraError::Io)?;`)
  - `crates/contextra-vector/src/persistence/mmap.rs:24` (`let mmap = contextra_sys::mmap_readonly(&file)`)

### 2.3 `contextra-bench` (Benchmarks)
- **`ensure_fd_limit`**
  - `benchmarks/contextra-bench/benches/competitive_kv.rs:500` (`contextra_sys::ensure_fd_limit(min_fds)`)

### 2.4 Künftige / Geplante Konsumenten
- **`contextra-vault` (geplant gemäß SPEC §6, B16, D-07, S-22):**
  - Wird auf `sys::vault` (`LockedRegions`, `VaultBuffer`) aufbauen.
  - Dokumentarischer Verweis in `crates/contextra-db/src/volatile_vault.rs:16`.

---

## 3. Quellkompatibilität und Signaturstabilität (INV-2, R3)
- Keine Signaturen oder Typen wurden geändert.
- Alle öffentlichen Re-Exports in `crates/contextra-sys/src/lib.rs` bleiben vollständig erhalten und 100% quellkompatibel.
