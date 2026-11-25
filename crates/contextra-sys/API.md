# API: contextra-sys

> Erzeugt aus dem Quellcode am 2026-11-25, Repository HEAD ce4579a56. Eine Zeile je Item: Datei:Zeile und die **erste Zeile** der Signatur. Mehrzeilige Signaturen sind nur angeschnitten. Nicht kompiliert, daher ohne Typprüfung. Testcode (`tests`, `testing`, `test_doubles`) ist ausgeschlossen.

Umfang: 84 öffentliche Items, 2 crate-interne Sichtbarkeitsgrenzen.

## Innen: Abhängigkeiten zu anderen Workspace-Crates

- keine

## Außen: öffentliche Schnittstelle (`pub`)

```rust
src/acl_win32.rs:60  pub fn set_restrictive_file_acl(path: &Path) -> std::io::Result<()>
src/acl_win32.rs:83  pub fn verify_file_acl_owner_only(path: &Path) -> std::io::Result<()>
src/acl_win32.rs:197  pub fn set_restrictive_file_acl(path: &Path) -> std::io::Result<()>
src/acl_win32.rs:281  pub fn verify_file_acl_owner_only(path: &Path) -> std::io::Result<()>
src/acl_win32.rs:404  pub fn set_restrictive_file_acl(path: &Path) -> std::io::Result<()>
src/acl_win32.rs:412  pub fn verify_file_acl_owner_only(path: &Path) -> std::io::Result<()>
src/durability.rs:44  pub fn set_fail_fsync(fail: bool, errno: i32)
src/durability.rs:49  pub fn should_fail_fsync() -> Option<i32>
src/durability.rs:163  pub fn durable_flush_file(file: &File, metadata_changed: bool) -> IoResult<()>
src/durability.rs:169  pub fn durable_flush_dir(path: &Path) -> IoResult<()>
src/durability.rs:194  pub fn write_all_durable(file: &mut File, mut buf: &[u8]) -> IoResult<()>
src/durability.rs:220  pub struct DurableFile
src/durability.rs:227  pub fn new(file: File) -> Self
src/durability.rs:235  pub fn is_poisoned(&self) -> bool
src/durability.rs:240  pub fn get_ref(&self) -> Result<&File>
src/durability.rs:249  pub fn write_all(&mut self, buf: &[u8]) -> Result<()>
src/durability.rs:257  pub fn sync_all(&mut self) -> Result<()>
src/durability.rs:271  pub fn sync_data(&mut self) -> Result<()>
src/error.rs:17  pub enum Error
src/error.rs:54  pub type Result<T> = std::result::Result<T, Error>
src/lib.rs:33  pub mod acl_win32
src/lib.rs:34  pub mod durability
src/lib.rs:35  pub mod error
src/lib.rs:36  pub mod mlock
src/lib.rs:37  pub mod mmap
src/lib.rs:38  pub mod posix
src/lib.rs:39  pub mod vault
src/lib.rs:42  pub mod mem
src/lib.rs:43  pub use crate::mlock::
src/lib.rs:46  pub use crate::vault::{LockedRegions, VaultBuffer}
src/lib.rs:50  pub mod fsmeta
src/lib.rs:51  pub use crate::acl_win32::{set_restrictive_file_acl, verify_file_acl_owner_only}
src/lib.rs:55  pub mod map
src/lib.rs:56  pub use crate::mmap::{mmap_readonly, SealedFile, SealedMap, SealedReadOnly, Unsealed}
src/lib.rs:61  pub use acl_win32::{set_restrictive_file_acl, verify_file_acl_owner_only}
src/lib.rs:62  pub use durability::{durable_flush_dir, durable_flush_file, write_all_durable, DurableFile}
src/lib.rs:63  pub use error::{Error, Result}
src/lib.rs:64  pub use mlock::
src/lib.rs:67  pub use mmap::{mmap_readonly, SealedFile, SealedMap, SealedReadOnly, Unsealed}
src/lib.rs:68  pub use posix::{ensure_fd_limit, reopen_and_dup2, set_fd_limit}
src/lib.rs:69  pub use vault::{LockedRegions, VaultBuffer}
src/mlock.rs:36  pub fn set_fail_mlock(fail: bool, errno: i32)
src/mlock.rs:41  pub fn should_fail_mlock() -> Option<i32>
src/mlock.rs:53  pub enum MemLockPolicy
src/mlock.rs:66  pub fn apply_dont_dump(ptr: *const u8, len: usize) -> Result<()>
src/mlock.rs:103  pub fn mem_lock_typed(ptr: *const u8, len: usize, policy: MemLockPolicy) -> Result<bool>
src/mlock.rs:206  pub fn mem_lock_with_policy(ptr: *const u8, len: usize, policy: MemLockPolicy) -> IoResult<bool>
src/mlock.rs:212  pub fn mem_unlock(addr: usize, len: usize)
src/mlock.rs:241  pub fn mem_lock(ptr: *const u8, len: usize) -> bool
src/mmap.rs:30  pub fn mmap_readonly(file: &File) -> io::Result<memmap2::Mmap>
src/mmap.rs:39  pub struct SealedReadOnly
src/mmap.rs:43  pub struct Unsealed
src/mmap.rs:48  pub struct SealedFile<State = SealedReadOnly>
src/mmap.rs:56  pub fn new_verified(file: File, expected_len: u64) -> io::Result<Self>
src/mmap.rs:76  pub fn as_file(&self) -> &File
src/mmap.rs:81  pub fn verified_len(&self) -> u64
src/mmap.rs:86  pub fn map_readonly(&self) -> io::Result<SealedMap>
src/mmap.rs:94  pub struct SealedMap
src/mmap.rs:100  pub fn map_sealed(file: &SealedFile<SealedReadOnly>) -> io::Result<Self>
src/mmap.rs:117  pub fn get_subslice(&self, offset: usize, len: usize) -> Result<&[u8]>
src/mmap.rs:127  pub fn as_slice(&self) -> &[u8]
src/mmap.rs:132  pub fn len(&self) -> usize
src/mmap.rs:137  pub fn is_empty(&self) -> bool
src/posix.rs:24  pub fn reopen_and_dup2(path: &Path, target_fd: i32, flags: i32) -> std::io::Result<()>
src/posix.rs:56  pub fn reopen_and_dup2(_path: &Path, _target_fd: i32, _flags: i32) -> std::io::Result<()>
src/posix.rs:65  pub fn ensure_fd_limit(min_fds: u64) -> bool
src/posix.rs:99  pub fn set_fd_limit(limit: u64) -> bool
src/vault.rs:34  pub struct LockedRegions
src/vault.rs:41  pub fn new() -> Self
src/vault.rs:49  pub fn lock_slice(&mut self, slice: &[u8], attempt_mlock: bool) -> bool
src/vault.rs:56  pub fn lock_slice_with_policy(
src/vault.rs:82  pub fn zeroize_slice(slice: &mut [u8])
src/vault.rs:94  pub fn unlock_all(&mut self)
src/vault.rs:101  pub fn len(&self) -> usize
src/vault.rs:106  pub fn is_empty(&self) -> bool
src/vault.rs:120  pub struct VaultBuffer
src/vault.rs:131  pub fn allocate(size: usize) -> Result<Self>
src/vault.rs:136  pub fn allocate_with_policy(size: usize, policy: MemLockPolicy) -> Result<Self>
src/vault.rs:155  pub fn from_slice(source: &[u8]) -> Result<Self>
src/vault.rs:162  pub fn is_locked(&self) -> bool
src/vault.rs:167  pub fn as_slice(&self) -> &[u8]
src/vault.rs:172  pub fn as_mut_slice(&mut self) -> &mut [u8]
src/vault.rs:177  pub fn len(&self) -> usize
src/vault.rs:182  pub fn is_empty(&self) -> bool
```

## Innen: crate-interne Grenzen (`pub(crate)`, `pub(super)`, `pub(in …)`)

```rust
src/durability.rs:38  pub(crate) mod fault_injection
src/mlock.rs:30  pub(crate) mod fault_injection
```
