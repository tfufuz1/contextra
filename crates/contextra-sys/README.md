# contextra-sys

**Zweck:** Low-Level-Systemabstraktion und FFI, **eine von zwei erlaubten `unsafe`-Inseln** (zusammen mit `contextra-simd`). Enthält laut Spec: mlock, Direct-IO, Dumpable, Win32-ACL (in Übergangsphase noch in `store`, siehe `tests/unsafe_islands.rs`).

**Ebene:** E0 (unsafe-Insel)

## Öffentliche API (`src/lib.rs`, Stand dieser README)

Ist-Stand. Module oder Namen, die laut PLAN.md entfallen, können hier noch stehen.

```rust
pub mod acl_win32
pub mod contract
pub mod durability
pub mod error
pub mod mlock
pub mod mmap
pub mod posix
pub mod vault
pub mod mem
pub mod fsmeta
pub mod map
pub use acl_win32::{set_restrictive_file_acl, verify_file_acl_owner_only}
pub use contract::{MemoryLocker, NoopMemoryLocker, OsMemoryLocker}
pub use durability::{durable_flush_dir, durable_flush_file, write_all_durable, DurableFile}
pub use error::{Error, Result, SysError}
pub use mlock::
pub use mmap::{mmap_readonly, SealedFile, SealedMap, SealedReadOnly, Unsealed}
pub use posix::{ensure_fd_limit, reopen_and_dup2, set_fd_limit}
pub use vault::{LockedRegions, VaultBuffer}
```

## Features (`Cargo.toml`)

keine

## Dokumentation

Implementierungsspezifikation: [`PLAN.md`](PLAN.md). Maßgeblich ist `docs/SPEC.md` v4.1.

