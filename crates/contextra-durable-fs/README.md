# contextra-durable-fs

**Zweck:** Crash-sichere Dateisystemoperationen und synchrones Flushen auf Platte. Grundlage für Durability (INV-FSYNC-1). Der frühere Tracer `tools/fsync_durability_tracer.py` wurde entfernt (Historie: `git show 0ec4de767:tools/fsync_durability_tracer.py`).

**Ebene:** E0

## Öffentliche API (`src/lib.rs`, Stand dieser README)

Ist-Stand. Module oder Namen, die laut PLAN.md entfallen, können hier noch stehen.

```rust
pub mod error
pub mod fault_injector
pub mod health
pub mod journal
pub mod platform
pub mod scrub
pub mod vfs
pub use error::{DurableFsError, Error, Result}
pub use fault_injector::{FaultInjector, NoFault}
pub use health::{AtomicHealthState, HealthEvent, HealthState}
pub use journal::
pub use platform::FlushKind
pub use scrub::
pub use vfs::{RealFs, StdVfs, Vfs}
pub enum FsClass
pub fn flush_file(file: &File, kind: FlushKind) -> Result<()>
pub fn flush_dir(dir: &Path) -> Result<()>
pub fn fs_class(path: &Path) -> Result<FsClass>
pub fn rename_durable(from: &Path, to: &Path) -> Result<()>
pub struct PoisonedHandle
pub struct DurableHandle<F: FaultInjector = NoFault>
pub enum LockMode
pub struct DirLock
pub fn sync_dir(dir: &Path) -> Result<()>
pub fn sync_dir_with_injector<F: FaultInjector>(dir: &Path, injector: &F) -> Result<()>
pub fn atomic_replace(path: &Path, bytes: &[u8]) -> Result<()>
pub fn atomic_replace_with_injector<F: FaultInjector>(
pub fn durable_remove(path: &Path) -> Result<()>
pub fn durable_remove_with_injector<F: FaultInjector>(path: &Path, injector: &F) -> Result<()>
pub fn scrub_and_remove(path: &Path) -> Result<ScrubReceipt>
pub fn scrub_and_remove_with_injector<F: FaultInjector>(
```

## Features (`Cargo.toml`)

`default`, `std`, `crc`, `serde`, `tracing`, `inspect`

## Dokumentation

Implementierungsspezifikation: [`PLAN.md`](PLAN.md). Maßgeblich ist `docs/SPEC.md` v4.1.

