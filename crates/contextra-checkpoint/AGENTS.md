# AGENTS.md — `contextra-checkpoint`

> Werkzeug/Crate-Kontext für Agenten. Der obere Teil ist generiert; manuelle Notizen unterhalb ergänzen.

<!-- BEGIN GENERATED: crate_inventory.py -->

## Inventar (generiert)

- **Pfad:** `crates/contextra-checkpoint` · **Ring:** 1 · **Version:** 0.1.0
- **Beschreibung:** Backup and snapshot management for Contextra storage
- **src:** 20 Dateien, 4017 LOC · **unsafe:** 0 Vorkommen, Policy `forbid`
- **Interne Abhängigkeiten (normal, 4):** `contextra-env`, `contextra-ids`, `contextra-ports`, `contextra-types`
- **Interne Dev/Build-Abhängigkeiten (1):** `contextra-testkit`
- **Externe Abhängigkeiten (normal, 8):** `blake3`, `bytes`, `parking_lot`, `serde`, `serde_json`, `thiserror`, `tokio`, `tracing`
- **Reverse-Abhängigkeiten (2):** `contextra-agent`, `contextra-db`
- **Kopplung an Hub-Crates:** ausgehend `contextra-ports`, `contextra-types`; eingehend `contextra-db`
- **Features (5):** `default`, `docid-128`, `ports-compat`, `types-compat`, `workspace`

### Öffentliche API (grob, aus src/)
- struct (19): `CheckpointGuard`, `CheckpointManifest`, `CheckpointMeta`, `DefaultHardlinkCloner`, `HardlinkCloneResult`, `InMemorySnapshotSource`, `InstanceOrphanRegistry`, `LoomDirEntry`, `LoomMetadata`, `LoomReadDir`, `OrphanRegistry`, `OrphanState`, `PersistentCheckpointStore`, `PinGuard`, `PinnedSeqNoOrphan` … (+4)
- enum (1): `CheckpointError`
- trait (6): `CheckpointHardlinkCloner`, `CheckpointRegistry`, `CheckpointVfs`, `FsSeam`, `SnapshotPinner`, `SnapshotSource`
- fn (91): `allocate_tx`, `await_pending_rollbacks`, `checkpoint`, `checkpoint_counter`, `checkpoint_guard_skipped_rollback_count`, `clear_all`, `clear_all_orphaned_checkpoints`, `clear_orphan_pin`, `clear_orphaned_checkpoint`, `clock`, `close`, `commit`, `copy`, `create_checkpoint`, `create_dir_all` … (+76)
- type (5): `BoxFuture`, `CheckpointResult`, `PinId`, `Result`, `VersionedKeyMap`
- const (1): `CURRENT_ID_WIDTH_BITS`
- mod (5): `compat`, `contract`, `error`, `ports_impl`, `snapshot_bridge`
- pub use: `crate::error::{CheckpointError, CheckpointResult}`, `crate::fs_seam::{FsSeam, StdFsSeam}`, `crate::meta::WorkflowState`, `crate::snapshot_source::{BoxFuture, SnapshotSource}`, `crate::vfs::CheckpointVfs`, `error::{CheckpointError, CheckpointResult}`, `fs_seam::{FsSeam, StdFsSeam}`, `guard::{CheckpointGuard, PinGuard}`, `hardlink_cloner::{ CheckpointHardlinkCloner, DefaultHardlinkCloner, HardlinkCloneResult, SnapshotPinner, }`, `manifest::CheckpointManifest` … (+5)

Regeneriert mit `python3 tools/crate_inventory.py agents`. Manuelle Notizen stehen außerhalb dieses Blocks.

<!-- END GENERATED: crate_inventory.py -->
