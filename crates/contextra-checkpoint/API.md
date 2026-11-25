# API: contextra-checkpoint

> Erzeugt aus dem Quellcode am 2026-11-25, Repository HEAD ce4579a56. Eine Zeile je Item: Datei:Zeile und die **erste Zeile** der Signatur. Mehrzeilige Signaturen sind nur angeschnitten. Nicht kompiliert, daher ohne Typprüfung. Testcode (`tests`, `testing`, `test_doubles`) ist ausgeschlossen.

Umfang: 128 öffentliche Items, 10 crate-interne Sichtbarkeitsgrenzen.

## Innen: Abhängigkeiten zu anderen Workspace-Crates

- keine

## Außen: öffentliche Schnittstelle (`pub`)

```rust
src/contextra_ports.rs:10  pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>
src/contextra_ports.rs:13  pub trait Clock: Send + Sync
src/contextra_ports.rs:22  pub struct SystemClock
src/contextra_ports.rs:29  pub const fn new() -> Self
src/contextra_ports.rs:51  pub struct ManualClock
src/contextra_ports.rs:57  pub fn new(initial_nanos: u64) -> Self
src/contextra_ports.rs:64  pub fn now_nanos(&self) -> u64
src/contextra_ports.rs:69  pub fn advance(&self, delta: std::time::Duration)
src/contextra_ports.rs:87  pub trait MetricsSink: Send + Sync
src/contextra_ports.rs:97  pub trait Vfs: Send + Sync
src/contextra_ports.rs:106  pub struct StorageStats
src/contextra_ports.rs:116  pub trait StorageEngine: Send + Sync
src/contextra_ports.rs:155  pub trait CheckpointCoordinator: Send + Sync
src/contextra_ports.rs:180  pub trait Checkpoint: Send + Sync
src/contextra_types.rs:8  pub enum ContextraError
src/contextra_types.rs:29  pub fn invalid_input(msg: impl Into<String>) -> Self
src/contextra_types.rs:34  pub fn cross_device_link(msg: impl Into<String>) -> Self
src/contextra_types.rs:64  pub type Result<T> = std::result::Result<T, ContextraError>
src/contextra_types.rs:68  pub struct TxId(pub u64)
src/contextra_types.rs:72  pub const INTERNAL_BASE: u64 = 1 << 62
src/contextra_types.rs:76  pub const fn new(id: u64) -> Self
src/contextra_types.rs:82  pub const fn as_u64(self) -> u64
src/contextra_types.rs:88  pub const fn inner(self) -> u64
src/contextra_types.rs:115  pub struct WorkflowState
src/guard.rs:41  pub struct PinGuard<S: crate::contextra_ports::StorageEngine>
src/guard.rs:91  pub fn clock(&self) -> &Arc<dyn crate::contextra_ports::Clock>
src/guard.rs:96  pub fn defuse(mut self)
src/guard.rs:152  pub struct CheckpointGuard<S: crate::contextra_ports::StorageEngine>
src/guard.rs:165  pub fn new(checkpoint: StateCheckpoint, storage: Arc<S>, namespace: impl Into<String>) -> Self
src/guard.rs:171  pub fn with_clock(
src/guard.rs:192  pub fn with_registry(
src/guard.rs:211  pub fn with_registry_and_counter(
src/guard.rs:230  pub fn with_registry_counter_and_clock(
src/guard.rs:252  pub fn with_metrics_sink(mut self, sink: Arc<dyn crate::contextra_ports::MetricsSink>) -> Self
src/guard.rs:258  pub fn with_metrics_sink_opt(
src/guard.rs:267  pub fn clock(&self) -> &Arc<dyn crate::contextra_ports::Clock>
src/guard.rs:317  pub fn checkpoint(&self) -> Result<&StateCheckpoint>
src/guard.rs:331  pub fn commit(mut self) -> Result<StateCheckpoint>
src/guard.rs:417  pub fn rollback_blocking(mut self) -> Result<()>
src/hardlink_cloner.rs:30  pub trait SnapshotPinner: Send + Sync
src/hardlink_cloner.rs:98  pub struct LoomReadDir
src/hardlink_cloner.rs:105  pub struct LoomDirEntry
src/hardlink_cloner.rs:107  pub fn path(&self) -> std::path::PathBuf
src/hardlink_cloner.rs:112  pub struct LoomMetadata
src/hardlink_cloner.rs:127  pub struct HardlinkCloneResult
src/hardlink_cloner.rs:150  pub trait CheckpointHardlinkCloner: Send + Sync
src/hardlink_cloner.rs:177  pub struct DefaultHardlinkCloner
src/hardlink_cloner.rs:181  pub fn new() -> Self
src/lib.rs:35  pub mod contextra_ports
src/lib.rs:36  pub mod contextra_types
src/lib.rs:38  pub use crate::contextra_types::{ContextraError, Result, TxId, WorkflowState}
src/lib.rs:50  pub use guard::{CheckpointGuard, PinGuard}
src/lib.rs:51  pub use hardlink_cloner::
src/lib.rs:54  pub use manifest::CheckpointManifest
src/lib.rs:55  pub use meta::{CheckpointMeta, StateCheckpoint}
src/lib.rs:57  pub use orphan::
src/lib.rs:64  pub use store::{CheckpointRegistry, PersistentCheckpointStore}
src/manifest.rs:32  pub struct CheckpointManifest
src/manifest.rs:56  pub fn new(meta: CheckpointMeta, components: Vec<String>) -> Result<Self>
src/manifest.rs:88  pub fn verify(&self) -> Result<()>
src/meta.rs:64  pub struct CheckpointMeta
src/meta.rs:79  pub fn into_workflow_state(&self) -> WorkflowState
src/meta.rs:90  pub fn to_workflow_state(&self) -> WorkflowState
src/meta.rs:111  pub struct StateCheckpoint
src/orphan.rs:28  pub type PinId = u64
src/orphan.rs:37  pub struct RecoveryReport
src/orphan.rs:46  pub fn is_empty(&self) -> bool
src/orphan.rs:51  pub fn total_recovered(&self) -> usize
src/orphan.rs:77  pub fn global_orphan_registry() -> &'static OrphanRegistry
src/orphan.rs:98  pub fn monotonic_timestamp_ms_with(clock: &dyn crate::contextra_ports::Clock) -> u64
src/orphan.rs:105  pub fn monotonic_timestamp_ms() -> u64
src/orphan.rs:128  pub struct OrphanRegistry
src/orphan.rs:142  pub fn new(file_path: impl Into<std::path::PathBuf>) -> Self
src/orphan.rs:149  pub fn register_orphan(&self, pin_id: PinId) -> std::io::Result<()>
src/orphan.rs:159  pub fn get_orphans(&self) -> Vec<PinId>
src/orphan.rs:168  pub fn clear_all(&self)
src/orphan.rs:212  pub struct PinnedSeqNoOrphan
src/orphan.rs:223  pub fn register_pinned_seq_no_orphan(orphan: PinnedSeqNoOrphan)
src/orphan.rs:238  pub struct OrphanState
src/orphan.rs:246  pub fn persist_sync(&self) -> std::io::Result<()>
src/orphan.rs:255  pub fn persist_vfs(&self, vfs: &dyn crate::contextra_ports::Vfs) -> std::io::Result<()>
src/orphan.rs:265  pub fn load_sync(path: &std::path::Path) -> Self
src/orphan.rs:279  pub fn load_vfs(vfs: &dyn crate::contextra_ports::Vfs, path: &std::path::Path) -> Self
src/orphan.rs:301  pub struct InstanceOrphanRegistry
src/orphan.rs:330  pub fn new(persist_path: impl Into<std::path::PathBuf>) -> Self
src/orphan.rs:336  pub fn new_with_vfs(
src/orphan.rs:345  pub fn with_clock(mut self, clock: Arc<dyn crate::contextra_ports::Clock>) -> Self
src/orphan.rs:351  pub fn with_vfs(mut self, vfs: Arc<dyn crate::contextra_ports::Vfs>) -> Self
src/orphan.rs:357  pub fn set_clock(&mut self, clock: Arc<dyn crate::contextra_ports::Clock>)
src/orphan.rs:362  pub fn clock(&self) -> &Arc<dyn crate::contextra_ports::Clock>
src/orphan.rs:367  pub fn vfs(&self) -> Option<&Arc<dyn crate::contextra_ports::Vfs>>
src/orphan.rs:372  pub fn load_sync(path: &std::path::Path) -> Self
src/orphan.rs:412  pub fn persist_sync(&self) -> std::io::Result<()>
src/orphan.rs:436  pub fn register_orphan_sync(&self, orphan: PinnedSeqNoOrphan)
src/orphan.rs:457  pub fn register_checkpoint_sync(&self, cp: StateCheckpoint)
src/orphan.rs:507  pub fn drain_orphan_pins(&self) -> Vec<PinnedSeqNoOrphan>
src/orphan.rs:524  pub fn get_orphan_pins(&self) -> Vec<PinnedSeqNoOrphan>
src/orphan.rs:529  pub fn clear_orphan_pin(&self, seq_no: u64)
src/orphan.rs:546  pub fn drain_orphaned_checkpoints(&self) -> Vec<StateCheckpoint>
src/orphan.rs:563  pub fn get_orphaned_checkpoints(&self) -> Vec<StateCheckpoint>
src/orphan.rs:568  pub fn clear_orphaned_checkpoint(&self, tx_id: TxId)
src/orphan.rs:581  pub fn clear_all(&self)
src/orphan.rs:599  pub fn register_orphaned_checkpoint(cp: StateCheckpoint)
src/orphan.rs:612  pub fn get_orphaned_checkpoints() -> Vec<StateCheckpoint>
src/orphan.rs:618  pub fn get_orphaned_checkpoints_for_namespace(ns: &str) -> Vec<StateCheckpoint>
src/orphan.rs:631  pub fn clear_orphaned_checkpoint(_tx_id: TxId)
src/orphan.rs:639  pub fn clear_all_orphaned_checkpoints()
src/orphan.rs:645  pub fn pending_rollback_count() -> usize
src/orphan.rs:651  pub fn orphaned_checkpoint_count() -> usize
src/store.rs:44  pub trait CheckpointRegistry: crate::contextra_ports::Checkpoint + Send + Sync
src/store.rs:232  pub struct PersistentCheckpointStore<S: crate::contextra_ports::StorageEngine>
src/store.rs:332  pub fn new(storage: Arc<S>, namespace: impl Into<String>) -> Result<Self>
src/store.rs:343  pub fn new_with_orphan_registry(
src/store.rs:440  pub fn skipped_rollback_count(&self) -> u64
src/store.rs:445  pub fn checkpoint_guard_skipped_rollback_count(&self) -> u64
src/store.rs:450  pub fn checkpoint_counter(&self) -> u64
src/store.rs:458  pub fn with_clock(mut self, clock: Arc<dyn crate::contextra_ports::Clock>) -> Self
src/store.rs:467  pub fn clock(&self) -> &Arc<dyn crate::contextra_ports::Clock>
src/store.rs:472  pub fn set_metrics_sink(&mut self, sink: Arc<dyn crate::contextra_ports::MetricsSink>)
src/store.rs:477  pub fn with_metrics_sink(mut self, sink: Arc<dyn crate::contextra_ports::MetricsSink>) -> Self
src/store.rs:483  pub fn monotonic_timestamp_ms(&self) -> u64
src/store.rs:494  pub fn create_guard(&self, tx_id: TxId) -> Result<CheckpointGuard<S>>
src/store.rs:791  pub fn orphan_registry(&self) -> &Arc<InstanceOrphanRegistry>
src/store.rs:814  pub fn register_pinned_seq_no_orphan(&self, orphan: PinnedSeqNoOrphan)
src/store.rs:819  pub fn register_orphaned_checkpoint(&self, cp: StateCheckpoint)
src/store.rs:824  pub fn get_orphaned_checkpoints(&self) -> Vec<StateCheckpoint>
src/store.rs:829  pub fn clear_orphaned_checkpoint(&self, tx_id: TxId)
src/store.rs:834  pub fn clear_all_orphaned_checkpoints(&self)
```

## Innen: crate-interne Grenzen (`pub(crate)`, `pub(super)`, `pub(in …)`)

```rust
src/guard.rs:153  pub(crate) checkpoint: Option<StateCheckpoint>,
src/guard.rs:154  pub(crate) storage: Arc<S>,
src/guard.rs:155  pub(crate) namespace: String,
src/guard.rs:156  pub(crate) orphan_registry: Arc<InstanceOrphanRegistry>,
src/guard.rs:157  pub(crate) skipped_rollbacks: Arc<AtomicU64>,
src/guard.rs:158  pub(crate) clock: Arc<dyn crate::contextra_ports::Clock>,
src/guard.rs:159  pub(crate) metrics_sink: Option<Arc<dyn crate::contextra_ports::MetricsSink>>,
src/guard.rs:160  pub(crate) created_at_nanos: u64,
src/meta.rs:32  pub(crate) fn validate_identifier(field_name: &str, value: &str) -> Result<()>
src/orphan.rs:111  pub(crate) fn clock_timestamp_ms(clock: &dyn crate::contextra_ports::Clock) -> u64
```
