## contextra-sys

Pfad: `crates/contextra-sys`

Abhängigkeiten (direkt): memmap2, thiserror, tracing

### Module

- `/src/acl_win32.rs`
- `/src/contract.rs`
- `/src/durability.rs`
- `/src/error.rs`
- `/src/lib.rs`
- `/src/mlock.rs`
- `/src/mmap.rs`
- `/src/posix.rs`
- `/src/vault.rs`

### Öffentliche Typen

- **DurableFile** (struct) – src/durability.rs:234
- **Error** (enum) – src/error.rs:24
- **LockedRegions** (struct) – src/vault.rs:40
- **MemLockPolicy** (enum) – src/mlock.rs:61
- **MemoryLocker** (trait) – src/contract.rs:11
- **NoopMemoryLocker** (struct) – src/contract.rs:21
- **OsMemoryLocker** (struct) – src/contract.rs:33
- **Result** (type) – src/error.rs:68
- **SealedFile** (struct) – src/mmap.rs:64
- **SealedMap** (struct) – src/mmap.rs:116
- **SealedReadOnly** (struct) – src/mmap.rs:55
- **SysError** (type) – src/error.rs:65
- **Unsealed** (struct) – src/mmap.rs:59
- **VaultBuffer** (struct) – src/vault.rs:125

### Öffentliche Funktionen

- `pub fn allocate(size: usize) -> Result<Self> `  – src/vault.rs:136
- `pub fn allocate_with_policy(size: usize, policy: MemLockPolicy) -> Result<Self> `  – src/vault.rs:141
- `pub fn apply_dont_dump(ptr: *const u8, len: usize) -> Result<()> `  – src/mlock.rs:81
- `pub fn as_file(&self) -> &File `  – src/mmap.rs:95
- `pub fn as_mut_slice(&mut self) -> &mut [u8] `  – src/vault.rs:177
- `pub fn as_slice(&self) -> &[u8] `  – src/mmap.rs:155
- `pub fn as_slice(&self) -> &[u8] `  – src/vault.rs:172
- `pub fn durable_flush_dir(path: &Path) -> IoResult<()> `  – src/durability.rs:179
- `pub fn durable_flush_file(file: &File, metadata_changed: bool) -> IoResult<()> `  – src/durability.rs:174
- `pub fn ensure_fd_limit(min_fds: u64) -> bool `  – src/posix.rs:84
- `pub fn from_slice(source: &[u8]) -> Result<Self> `  – src/vault.rs:160
- `pub fn get_ref(&self) -> Result<&File> `  – src/durability.rs:254
- `pub fn get_subslice(&self, offset: usize, len: usize) -> Result<&[u8]> `  – src/mmap.rs:145
- `pub fn is_empty(&self) -> bool `  – src/mmap.rs:165
- `pub fn is_empty(&self) -> bool `  – src/vault.rs:108
- `pub fn is_empty(&self) -> bool `  – src/vault.rs:187
- `pub fn is_locked(&self) -> bool `  – src/vault.rs:167
- `pub fn is_poisoned(&self) -> bool `  – src/durability.rs:249
- `pub fn len(&self) -> usize `  – src/mmap.rs:160
- `pub fn len(&self) -> usize `  – src/vault.rs:103
- `pub fn len(&self) -> usize `  – src/vault.rs:182
- `pub fn lock_slice(&mut self, slice: &[u8], attempt_mlock: bool) -> bool `  – src/vault.rs:53
- `pub fn lock_slice_with_policy(`  – src/vault.rs:59
- `pub fn map_readonly(&self) -> io::Result<SealedMap> `  – src/mmap.rs:105
- `pub fn map_sealed(file: &SealedFile<SealedReadOnly>) -> io::Result<Self> `  – src/mmap.rs:125
- `pub fn mem_lock(ptr: *const u8, len: usize) -> bool `  – src/mlock.rs:260
- `pub fn mem_lock_typed(ptr: *const u8, len: usize, policy: MemLockPolicy) -> Result<bool> `  – src/mlock.rs:123
- `pub fn mem_lock_with_policy(ptr: *const u8, len: usize, policy: MemLockPolicy) -> IoResult<bool> `  – src/mlock.rs:227
- `pub fn mem_unlock(addr: usize, len: usize) `  – src/mlock.rs:232
- `pub fn mmap_readonly(file: &File) -> io::Result<memmap2::Mmap> `  – src/mmap.rs:41
- `pub fn new() -> Self `  – src/vault.rs:46
- `pub fn new(file: File) -> Self `  – src/durability.rs:241
- `pub fn new_verified(file: File, expected_len: u64) -> io::Result<Self> `  – src/mmap.rs:75
- `pub fn reopen_and_dup2(_path: &Path, _target_fd: i32, _flags: i32) -> std::io::Result<()> `  – src/posix.rs:69
- `pub fn reopen_and_dup2(path: &Path, target_fd: i32, flags: i32) -> std::io::Result<()> `  – src/posix.rs:37
- `pub fn set_fail_fsync(fail: bool, errno: i32) `  – src/durability.rs:49
- `pub fn set_fail_mlock(fail: bool, errno: i32) `  – src/mlock.rs:41
- `pub fn set_fd_limit(limit: u64) -> bool `  – src/posix.rs:119
- `pub fn set_restrictive_file_acl(path: &Path) -> std::io::Result<()> `  – src/acl_win32.rs:201
- `pub fn set_restrictive_file_acl(path: &Path) -> std::io::Result<()> `  – src/acl_win32.rs:409
- `pub fn set_restrictive_file_acl(path: &Path) -> std::io::Result<()> `  – src/acl_win32.rs:64
- `pub fn should_fail_fsync() -> Option<i32> `  – src/durability.rs:54
- `pub fn should_fail_mlock() -> Option<i32> `  – src/mlock.rs:46
- `pub fn sync_all(&mut self) -> Result<()> `  – src/durability.rs:271
- `pub fn sync_data(&mut self) -> Result<()> `  – src/durability.rs:285
- `pub fn unlock_all(&mut self) `  – src/vault.rs:96
- `pub fn verified_len(&self) -> u64 `  – src/mmap.rs:100
- `pub fn verify_file_acl_owner_only(path: &Path) -> std::io::Result<()> `  – src/acl_win32.rs:284
- `pub fn verify_file_acl_owner_only(path: &Path) -> std::io::Result<()> `  – src/acl_win32.rs:416
- `pub fn verify_file_acl_owner_only(path: &Path) -> std::io::Result<()> `  – src/acl_win32.rs:86
- `pub fn write_all(&mut self, buf: &[u8]) -> Result<()> `  – src/durability.rs:263
- `pub fn write_all_durable(file: &mut File, mut buf: &[u8]) -> IoResult<()> `  – src/durability.rs:205
- `pub fn zeroize_slice(slice: &mut [u8]) `  – src/vault.rs:84

---
