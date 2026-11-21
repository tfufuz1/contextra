# contextra-checkpoint

**Zweck:** Backup und Snapshots: Hartlink-Snapshot und Restore. Ausgeschlossen laut Spec: `keys/` und `erasure/` (ADR-42). Datensicherung und Wiederherstellung: SPEC §7.11.

**Ebene:** E1

## Öffentliche API (`src/lib.rs`, Stand dieser README)

Ist-Stand. Module oder Namen, die laut PLAN.md entfallen, können hier noch stehen.

```rust
pub mod compat
pub mod contract
pub use vfs::CheckpointVfs
pub use error::{CheckpointError, CheckpointResult}
pub use fs_seam::{FsSeam, StdFsSeam}
pub use guard::{CheckpointGuard, PinGuard}
pub use hardlink_cloner::
pub use manifest::CheckpointManifest
pub use meta::{CheckpointMeta, StateCheckpoint, CURRENT_ID_WIDTH_BITS}
pub use orphan::
pub use snapshot_source::{BoxFuture as LocalBoxFuture, SnapshotSource}
pub use store::{CheckpointRegistry, PersistentCheckpointStore}
```

## Features (`Cargo.toml`)

`default`, `workspace`, `types-compat`, `ports-compat`, `docid-128`

## Dokumentation

Implementierungsspezifikation: [`PLAN.md`](PLAN.md). Maßgeblich ist `docs/SPEC.md` v4.1.

