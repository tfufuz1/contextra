# API: contextra-durable-fs

> Erzeugt aus dem Quellcode am 2026-11-25, Repository HEAD ce4579a56. Eine Zeile je Item: Datei:Zeile und die **erste Zeile** der Signatur. Mehrzeilige Signaturen sind nur angeschnitten. Nicht kompiliert, daher ohne Typprüfung. Testcode (`tests`, `testing`, `test_doubles`) ist ausgeschlossen.

Umfang: 102 öffentliche Items, 2 crate-interne Sichtbarkeitsgrenzen.

## Innen: Abhängigkeiten zu anderen Workspace-Crates

- keine

## Außen: öffentliche Schnittstelle (`pub`)

```rust
src/error.rs:11  pub enum DurableFsError
src/error.rs:45  pub type Error = DurableFsError
src/error.rs:50  pub fn kind(&self) -> std::io::ErrorKind
src/error.rs:73  pub type Result<T> = std::result::Result<T, DurableFsError>
src/fault_injector.rs:10  pub trait FaultInjector: Send + Sync
src/fault_injector.rs:34  pub struct NoFault
src/fault_injector.rs:82  pub enum InjectedFault
src/fault_injector.rs:95  pub struct FaultInjectingVfs<V: Vfs>
src/fault_injector.rs:107  pub fn new(inner: V) -> Self
src/fault_injector.rs:120  pub fn inject_write_disk_full(&self)
src/fault_injector.rs:125  pub fn inject_fsync_failure(&self)
src/fault_injector.rs:130  pub fn inject_rename_failure(&self)
src/fault_injector.rs:135  pub fn inject_dir_fsync_failure(&self)
src/fault_injector.rs:141  pub fn simulate_crash(&self) -> Result<()>
src/health.rs:18  pub enum HealthState
src/health.rs:33  pub const fn new() -> Self
src/health.rs:39  pub fn transition(self, event: HealthEvent) -> Self
src/health.rs:60  pub fn allows_writes(self) -> bool
src/health.rs:66  pub fn is_poisoned(self) -> bool
src/health.rs:73  pub enum HealthEvent
src/health.rs:86  pub struct AtomicHealthState
src/health.rs:99  pub fn new() -> Self
src/health.rs:107  pub fn load(&self) -> HealthState
src/health.rs:117  pub fn record_event(&self, event: HealthEvent) -> HealthState
src/journal.rs:12  pub enum JournalSyncPolicy
src/journal.rs:28  pub enum RecordRecoveryPolicy
src/journal.rs:37  pub const JOURNAL_MAGIC: [u8; 4] = [0x43, 0x54, 0x58, 0x4A]
src/journal.rs:41  pub const RECORD_HEADER_SIZE: usize = 20
src/journal.rs:45  pub fn calculate_crc32(data: &[u8]) -> u32
src/journal.rs:62  pub struct JournalRecord
src/journal.rs:71  pub struct AppendOnlyJournal
src/journal.rs:82  pub fn open(path: impl Into<PathBuf>) -> Result<Self>
src/journal.rs:91  pub fn open_without_recovery(
src/journal.rs:116  pub fn open_with_policy(
src/journal.rs:131  pub fn path(&self) -> &std::path::Path
src/journal.rs:136  pub fn append(&mut self, payload: &[u8]) -> Result<u64>
src/journal.rs:179  pub fn flush(&mut self) -> Result<()>
src/journal.rs:194  pub fn recover(&mut self, recovery_policy: RecordRecoveryPolicy) -> Result<Vec<JournalRecord>>
src/journal.rs:300  pub fn health_state(&self) -> HealthState
src/journal.rs:307  pub struct PendingDeleteJournal
src/journal.rs:315  pub fn open(journal_path: impl Into<PathBuf>) -> Result<Self>
src/journal.rs:320  pub fn open_with_health(
src/journal.rs:346  pub fn health_state(&self) -> HealthState
src/journal.rs:351  pub fn record(&mut self, path: &std::path::Path) -> Result<()>
src/journal.rs:358  pub fn pending(&self) -> &[PathBuf]
src/journal.rs:363  pub fn process<F>(&mut self, mut remove_fn: F) -> Result<usize>
src/lib.rs:18  pub mod error
src/lib.rs:20  pub mod fault_injector
src/lib.rs:22  pub mod health
src/lib.rs:24  pub mod journal
src/lib.rs:26  pub mod platform
src/lib.rs:28  pub mod scrub
src/lib.rs:30  pub mod vfs
src/lib.rs:32  pub use error::{DurableFsError, Error, Result}
src/lib.rs:33  pub use fault_injector::{FaultInjector, NoFault}
src/lib.rs:34  pub use health::{AtomicHealthState, HealthEvent, HealthState}
src/lib.rs:35  pub use journal::
src/lib.rs:38  pub use platform::FlushKind
src/lib.rs:39  pub use scrub::
src/lib.rs:43  pub use vfs::{RealFs, StdVfs, Vfs}
src/lib.rs:53  pub enum FsClass
src/lib.rs:69  pub const fn is_supported_local(&self) -> bool
src/lib.rs:75  pub fn flush_file(file: &File, kind: FlushKind) -> Result<()>
src/lib.rs:80  pub fn flush_dir(dir: &Path) -> Result<()>
src/lib.rs:98  pub fn fs_class(path: &Path) -> Result<FsClass>
src/lib.rs:113  pub fn rename_durable(from: &Path, to: &Path) -> Result<()>
src/lib.rs:125  pub struct PoisonedHandle
src/lib.rs:129  pub struct DurableHandle<F: FaultInjector = NoFault>
src/lib.rs:138  pub fn new(file: File) -> Self
src/lib.rs:145  pub fn new_with_injector(file: File, injector: F) -> Self
src/lib.rs:155  pub fn new_with_path_and_injector(file: File, path: PathBuf, injector: F) -> Self
src/lib.rs:166  pub fn is_poisoned(&self) -> bool
src/lib.rs:171  pub fn flush(&mut self, kind: FlushKind) -> Result<()>
src/lib.rs:197  pub fn get_ref(&self) -> Option<&File>
src/lib.rs:206  pub fn get_mut(&mut self) -> Option<&mut File>
src/lib.rs:215  pub fn poison(&mut self)
src/lib.rs:246  pub enum LockMode
src/lib.rs:256  pub struct DirLock
src/lib.rs:264  pub fn acquire(dir: &Path, mode: LockMode) -> Result<Self>
src/lib.rs:303  pub fn mode(&self) -> LockMode
src/lib.rs:319  pub fn sync_dir(dir: &Path) -> Result<()>
src/lib.rs:324  pub fn sync_dir_with_injector<F: FaultInjector>(dir: &Path, injector: &F) -> Result<()>
src/lib.rs:330  pub fn atomic_replace(path: &Path, bytes: &[u8]) -> Result<()>
src/lib.rs:335  pub fn atomic_replace_with_injector<F: FaultInjector>(
src/lib.rs:382  pub fn durable_remove(path: &Path) -> Result<()>
src/lib.rs:387  pub fn durable_remove_with_injector<F: FaultInjector>(path: &Path, injector: &F) -> Result<()>
src/lib.rs:399  pub fn scrub_and_remove(path: &Path) -> Result<ScrubReceipt>
src/lib.rs:404  pub fn scrub_and_remove_with_injector<F: FaultInjector>(
src/platform.rs:12  pub enum FlushKind
src/platform.rs:23  pub fn sync_file(file: &File, kind: FlushKind) -> io::Result<()>
src/platform.rs:37  pub fn sync_dir_path(dir: &Path) -> io::Result<()>
src/scrub.rs:21  pub struct ScrubReceipt
src/scrub.rs:28  pub enum ScrubPolicy
src/scrub.rs:37  pub enum ScrubRefusal
src/scrub.rs:54  pub enum ScrubOutcome
src/scrub.rs:65  pub struct ScrubProgress
src/scrub.rs:75  pub fn scrub_file_with_rate_limit<F>(
src/scrub.rs:133  pub fn scrub_if_safe(
src/scrub.rs:143  pub fn scrub_if_safe_with_injector<F: FaultInjector>(
src/vfs.rs:12  pub trait Vfs: Send + Sync
src/vfs.rs:57  pub struct StdVfs
src/vfs.rs:60  pub type RealFs = StdVfs
```

## Innen: crate-interne Grenzen (`pub(crate)`, `pub(super)`, `pub(in …)`)

```rust
src/vfs.rs:123  pub(crate) struct TmpFileGuard<'a>(pub &'a Path)
src/vfs.rs:132  pub(crate) fn generate_tmp_path(target: &Path) -> Result<PathBuf>
```
