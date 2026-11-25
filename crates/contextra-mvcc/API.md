# API: contextra-mvcc

> Erzeugt aus dem Quellcode am 2026-11-25, Repository HEAD ce4579a56. Eine Zeile je Item: Datei:Zeile und die **erste Zeile** der Signatur. Mehrzeilige Signaturen sind nur angeschnitten. Nicht kompiliert, daher ohne Typprüfung. Testcode (`tests`, `testing`, `test_doubles`) ist ausgeschlossen.

Umfang: 204 öffentliche Items, 3 crate-interne Sichtbarkeitsgrenzen.

## Innen: Abhängigkeiten zu anderen Workspace-Crates

- keine

## Außen: öffentliche Schnittstelle (`pub`)

```rust
src/contextra_ports.rs:6  pub trait Clock: Send + Sync
src/contextra_ports.rs:15  pub struct SystemClock
src/contextra_ports.rs:28  pub const fn new() -> Self
src/contextra_ports.rs:49  pub trait MetricsSink: Send + Sync
src/contextra_ports.rs:58  pub struct ManualClock
src/contextra_ports.rs:64  pub fn new(initial_nanos: u64) -> Self
src/contextra_ports.rs:71  pub fn now_nanos(&self) -> u64
src/contextra_ports.rs:76  pub fn advance(&self, delta: std::time::Duration)
src/contextra_types.rs:4  pub mod error
src/contextra_types.rs:9  pub enum ContextraError
src/contextra_types.rs:35  pub fn conflict(msg: impl Into<String>) -> Self
src/contextra_types.rs:40  pub fn invalid_input(msg: impl Into<String>) -> Self
src/contextra_types.rs:45  pub fn limit_exceeded(_limit: usize, msg: impl Into<String>) -> Self
src/contextra_types.rs:51  pub type Result<T> = std::result::Result<T, ContextraError>
src/contextra_types.rs:55  pub mod tombstone
src/contextra_types.rs:57  pub trait SeqBitTombstone
src/contextra_types.rs:65  pub trait TombstoneSemanticsCheck
src/contextra_types.rs:81  pub mod types
src/contextra_types.rs:86  pub const TOMBSTONE_BIT: u64 = 1 << 63
src/contextra_types.rs:90  pub struct DocId(pub u64)
src/contextra_types.rs:95  pub const fn new(id: u64) -> Self
src/contextra_types.rs:101  pub const fn as_u64(self) -> u64
src/contextra_types.rs:107  pub const fn inner(self) -> u64
src/contextra_types.rs:112  pub fn from_key(s: &str) -> super::error::Result<Self>
src/contextra_types.rs:151  pub struct TenantId(pub u64)
src/contextra_types.rs:156  pub const fn new(id: u64) -> Self
src/contextra_types.rs:162  pub const fn as_u64(self) -> u64
src/contextra_types.rs:168  pub const fn inner(self) -> u64
src/contextra_types.rs:173  pub fn from_key(s: &str) -> super::error::Result<Self>
src/contextra_types.rs:212  pub struct TxId(pub u64)
src/contextra_types.rs:216  pub const INTERNAL_BASE: u64 = 1 << 62
src/contextra_types.rs:220  pub const fn new(id: u64) -> Self
src/contextra_types.rs:226  pub const fn as_u64(self) -> u64
src/contextra_types.rs:232  pub const fn inner(self) -> u64
src/contextra_types.rs:237  pub fn from_key(s: &str) -> super::error::Result<Self>
src/lib.rs:80  pub mod contextra_ports
src/lib.rs:81  pub mod contextra_types
src/lib.rs:83  pub use contextra_types::
src/lib.rs:89  pub mod seq_log
src/lib.rs:90  pub mod snapshot
src/lib.rs:91  pub mod ssi
src/lib.rs:92  pub mod tx_buffer
src/lib.rs:94  pub use seq_log::{SeqLogChange, SeqLogEntry, SequenceLog}
src/lib.rs:95  pub use snapshot::{SnapshotGuard, SnapshotRegistry}
src/lib.rs:96  pub use ssi::{PruningBlockerInfo, ReadSet, SequenceLogSsiValidator, SsiValidator}
src/lib.rs:97  pub use tx_buffer::{IndexOp, TxBuffer}
src/seq_log.rs:32  pub const DEFAULT_MAX_PIN_DURATION: Duration = Duration::from_secs(300)
src/seq_log.rs:49  pub struct SeqLogEntry
src/seq_log.rs:79  pub fn is_visible(&self, as_of: u64) -> bool
src/seq_log.rs:93  pub enum SeqLogChange
src/seq_log.rs:112  pub fn seq(&self) -> u64
src/seq_log.rs:119  pub fn doc_id(&self) -> DocId
src/seq_log.rs:137  pub struct SequenceLog
src/seq_log.rs:173  pub fn new() -> Self
src/seq_log.rs:179  pub fn with_clock(clock: Arc<dyn Clock>) -> Self
src/seq_log.rs:191  pub fn with_time_fn<F>(time_fn: F) -> Self
src/seq_log.rs:208  pub fn clock(&self) -> &Arc<dyn Clock>
src/seq_log.rs:213  pub fn set_clock(&mut self, clock: Arc<dyn Clock>)
src/seq_log.rs:218  pub fn with_max_pin_duration(mut self, duration: Duration) -> Self
src/seq_log.rs:224  pub fn set_max_pin_duration(&mut self, duration: Duration)
src/seq_log.rs:229  pub fn max_pin_duration(&self) -> Duration
src/seq_log.rs:239  pub fn pin_snapshot(&mut self, seq_no: u64)
src/seq_log.rs:245  pub fn pin_snapshot_at(&mut self, seq_no: u64, at_nanos: u64)
src/seq_log.rs:259  pub fn unpin_snapshot(&mut self, seq_no: u64)
src/seq_log.rs:276  pub fn expired_pins_at(&self, now_nanos: u64) -> Vec<u64>
src/seq_log.rs:292  pub fn expired_pins(&self) -> Vec<u64>
src/seq_log.rs:302  pub fn reap_expired_pins_at(&mut self, now_nanos: u64) -> Vec<u64>
src/seq_log.rs:311  pub fn reap_expired_pins(&mut self) -> Vec<u64>
src/seq_log.rs:320  pub fn min_retention_seq_at(&self, now_nanos: u64) -> Option<u64>
src/seq_log.rs:343  pub fn min_retention_seq(&self) -> Option<u64>
src/seq_log.rs:348  pub fn deletion_seq(&self, doc_id: DocId) -> Option<u64>
src/seq_log.rs:360  pub fn record_insert(&mut self, doc_id: DocId, seq: u64)
src/seq_log.rs:383  pub fn record_delete(&mut self, doc_id: DocId, seq: u64)
src/seq_log.rs:399  pub fn is_visible(&self, doc_id: DocId, seq_no: u64) -> bool
src/seq_log.rs:430  pub fn compact(&mut self, min_active_seqno: u64)
src/seq_log.rs:449  pub fn len(&self) -> usize
src/seq_log.rs:454  pub fn is_empty(&self) -> bool
src/seq_log.rs:463  pub fn changes_since(&self, snapshot_seq: u64) -> Vec<SeqLogChange>
src/snapshot.rs:44  pub struct SnapshotRegistryConfig
src/snapshot.rs:76  pub struct SnapshotRegistry
src/snapshot.rs:100  pub fn new() -> Self
src/snapshot.rs:105  pub fn with_clock(clock: Arc<dyn Clock>) -> Self
src/snapshot.rs:110  pub fn with_clock_and_config(clock: Arc<dyn Clock>, config: SnapshotRegistryConfig) -> Self
src/snapshot.rs:120  pub fn with_time_fn<F>(time_fn: F) -> Self
src/snapshot.rs:137  pub fn clock(&self) -> &Arc<dyn Clock>
src/snapshot.rs:142  pub fn config(&self) -> SnapshotRegistryConfig
src/snapshot.rs:147  pub fn set_config(&mut self, config: SnapshotRegistryConfig)
src/snapshot.rs:154  pub fn oldest_active_snapshot(&self) -> Option<u64>
src/snapshot.rs:162  pub fn try_register(self: &Arc<Self>, seq_no: u64) -> Result<SnapshotGuard>
src/snapshot.rs:168  pub fn try_register_at(self: &Arc<Self>, seq_no: u64, at_nanos: u64) -> Result<SnapshotGuard>
src/snapshot.rs:173  pub fn try_register_for_tenant(
src/snapshot.rs:183  pub fn try_register_for_tenant_at(
src/snapshot.rs:224  pub fn register(self: &Arc<Self>, seq_no: u64) -> SnapshotGuard
src/snapshot.rs:230  pub fn register_for_tenant(self: &Arc<Self>, tenant: TenantId, seq_no: u64) -> SnapshotGuard
src/snapshot.rs:236  pub fn register_for_tenant_at(
src/snapshot.rs:258  pub fn register_at(self: &Arc<Self>, seq_no: u64, at_nanos: u64) -> SnapshotGuard
src/snapshot.rs:276  pub fn min_active_seqno(&self) -> u64
src/snapshot.rs:282  pub fn min_read_snapshot(&self) -> Option<u64>
src/snapshot.rs:293  pub fn min_for_compaction(&self, current_seq: u64) -> u64
src/snapshot.rs:298  pub fn revoke_covering(
src/snapshot.rs:331  pub fn pin(&self, seq_no: u64)
src/snapshot.rs:338  pub fn pin_for_tenant(&self, tenant: TenantId, seq_no: u64)
src/snapshot.rs:344  pub fn pin_for_tenant_at(&self, tenant: TenantId, seq_no: u64, at_nanos: u64)
src/snapshot.rs:355  pub fn pin_at(&self, seq_no: u64, at_nanos: u64)
src/snapshot.rs:366  pub fn unpin(&self, seq_no: u64)
src/snapshot.rs:425  pub fn longest_active_pin_at(&self, now_nanos: u64) -> Option<(u64, Duration)>
src/snapshot.rs:446  pub fn longest_active_pin(&self) -> Option<(u64, Duration)>
src/snapshot.rs:458  pub fn reap_expired_pins_at(&self, max_pin_duration: Duration, now_nanos: u64) -> Vec<u64>
src/snapshot.rs:489  pub fn reap_expired_pins(&self, max_pin_duration: Duration) -> Vec<u64>
src/snapshot.rs:508  pub struct SnapshotGuard
src/snapshot.rs:517  pub fn seq_no(&self) -> u64
src/snapshot.rs:522  pub fn tenant(&self) -> Option<TenantId>
src/ssi.rs:44  pub const DEFAULT_MAX_TRACKED_COMMIT_KEYS: usize = 1_000_000
src/ssi.rs:57  pub struct ReadSet
src/ssi.rs:68  pub fn new() -> Self
src/ssi.rs:83  pub fn record_read(&mut self, key: impl Into<Vec<u8>>, snapshot_seq: u64)
src/ssi.rs:105  pub fn record_prefix(&mut self, prefix: impl Into<Vec<u8>>, snapshot_seq: u64)
src/ssi.rs:124  pub fn register_read(&mut self, key: impl Into<Vec<u8>>, snapshot_seq: u64)
src/ssi.rs:129  pub fn get(&self, key: &[u8]) -> Option<u64>
src/ssi.rs:134  pub fn is_empty(&self) -> bool
src/ssi.rs:139  pub fn len(&self) -> usize
src/ssi.rs:144  pub fn clear(&mut self)
src/ssi.rs:151  pub fn min_snapshot_seq(&self) -> Option<u64>
src/ssi.rs:156  pub fn iter(&self) -> impl Iterator<Item = (&Vec<u8>, &u64)>
src/ssi.rs:161  pub fn prefixes_iter(&self) -> impl Iterator<Item = (&Vec<u8>, &u64)>
src/ssi.rs:166  pub fn keys_map(&self) -> &AHashMap<Vec<u8>, u64>
src/ssi.rs:171  pub fn prefixes_map(&self) -> &AHashMap<Vec<u8>, u64>
src/ssi.rs:180  pub trait SsiValidator
src/ssi.rs:210  pub struct PruningBlockerInfo
src/ssi.rs:505  pub struct SequenceLogSsiValidator
src/ssi.rs:539  pub fn new() -> Self
src/ssi.rs:544  pub fn new_with_bounds(max_tracked_keys: usize) -> Self
src/ssi.rs:549  pub fn new_with_clock_and_bounds(clock: Arc<dyn Clock>, max_tracked_keys: usize) -> Self
src/ssi.rs:563  pub fn new_with_time_fn_and_bounds<F>(time_fn: F, max_tracked_keys: usize) -> Self
src/ssi.rs:580  pub fn with_clock(mut self, clock: Arc<dyn Clock>) -> Self
src/ssi.rs:586  pub fn with_time_fn<F>(self, time_fn: F) -> Self
src/ssi.rs:603  pub fn set_clock(&mut self, clock: Arc<dyn Clock>)
src/ssi.rs:608  pub fn clock(&self) -> &Arc<dyn Clock>
src/ssi.rs:613  pub fn with_sequence_log(seq_log: Arc<RwLock<SequenceLog>>) -> Self
src/ssi.rs:627  pub fn with_snapshot_registry(mut self, registry: Arc<SnapshotRegistry>) -> Self
src/ssi.rs:633  pub fn set_snapshot_registry(&mut self, registry: Arc<SnapshotRegistry>)
src/ssi.rs:638  pub fn with_metrics_sink(
src/ssi.rs:647  pub fn set_metrics_sink(
src/ssi.rs:655  pub fn with_max_pin_duration(mut self, duration: Duration) -> Self
src/ssi.rs:671  pub fn prune_through(&self, bound_seq: u64) -> usize
src/ssi.rs:689  pub fn prune_through_checked(&self, bound: Option<u64>) -> Result<usize>
src/ssi.rs:714  pub fn forget_from(&self, first_seq: u64) -> usize
src/ssi.rs:721  pub fn pruned_through(&self) -> u64
src/ssi.rs:726  pub fn tracked_commit_keys(&self) -> usize
src/ssi.rs:731  pub fn max_tracked_keys(&self) -> usize
src/ssi.rs:796  pub fn diagnose_pruning_blocker(&self) -> Option<PruningBlockerInfo>
src/ssi.rs:801  pub fn diagnose_pruning_blocker_at(&self, now_nanos: u64) -> Option<PruningBlockerInfo>
src/ssi.rs:848  pub fn record_commit_key(&self, key: &[u8], commit_seq: u64)
src/ssi.rs:856  pub fn record_commit_keys<'a>(
src/ssi.rs:1056  pub fn validate_and_record<'a>(
src/tx_buffer.rs:49  pub const STAGING_ENTRY_OVERHEAD_BYTES: usize = 32
src/tx_buffer.rs:52  pub const DEFAULT_SHARD_COUNT: usize = 64
src/tx_buffer.rs:56  pub const DEFAULT_MAX_OPS_PER_TX: usize = 10_000
src/tx_buffer.rs:59  pub const DEFAULT_MAX_READ_SET_KEYS: usize = 100_000
src/tx_buffer.rs:62  pub const DEFAULT_MAX_TX_STAGED_BYTES: usize = 16 * 1024 * 1024
src/tx_buffer.rs:65  pub const DEFAULT_MAX_TOTAL_STAGED_BYTES: usize = 256 * 1024 * 1024
src/tx_buffer.rs:69  pub struct TxBufferConfig
src/tx_buffer.rs:101  pub enum IndexOp<T: Clone>
src/tx_buffer.rs:120  pub fn doc_id(&self) -> DocId
src/tx_buffer.rs:131  pub trait StagedOpSize
src/tx_buffer.rs:174  pub fn estimated_bytes(&self) -> usize
src/tx_buffer.rs:220  pub struct TxBuffer<T: Clone>
src/tx_buffer.rs:242  pub fn new() -> Self
src/tx_buffer.rs:247  pub fn new_with_config(shard_count: usize, tx_timeout: Duration) -> Self
src/tx_buffer.rs:256  pub fn new_with_config_ext(
src/tx_buffer.rs:265  pub fn new_with_clock_and_config(
src/tx_buffer.rs:288  pub fn new_with_time_fn_and_config<F>(
src/tx_buffer.rs:309  pub fn with_clock(mut self, clock: Arc<dyn Clock>) -> Self
src/tx_buffer.rs:315  pub fn with_time_fn<F>(self, time_fn: F) -> Self
src/tx_buffer.rs:332  pub fn clock(&self) -> &Arc<dyn Clock>
src/tx_buffer.rs:337  pub fn staged_bytes(&self) -> usize
src/tx_buffer.rs:365  pub fn has_tx(&self, tx: TxId) -> bool
src/tx_buffer.rs:380  pub fn begin_at(&self, tx: TxId, at_nanos: u64)
src/tx_buffer.rs:394  pub fn begin(&self, tx: TxId)
src/tx_buffer.rs:400  pub fn register_read(&self, tx: TxId, key: impl Into<Vec<u8>>, snapshot_seq: u64)
src/tx_buffer.rs:419  pub fn try_register_read(
src/tx_buffer.rs:447  pub fn record_read(&self, tx: TxId, key: impl Into<Vec<u8>>, snapshot_seq: u64)
src/tx_buffer.rs:455  pub fn min_read_snapshot(&self) -> Option<u64>
src/tx_buffer.rs:469  pub fn get_read_set(&self, tx: TxId) -> Option<ReadSet>
src/tx_buffer.rs:477  pub fn read_set(&self, tx: TxId) -> Option<ReadSet>
src/tx_buffer.rs:482  pub fn stage(&self, tx: TxId, op: IndexOp<T>) -> Result<()>
src/tx_buffer.rs:498  pub fn stage_bounded(&self, tx: TxId, op: IndexOp<T>) -> Result<()>
src/tx_buffer.rs:548  pub fn stage_many(&self, tx: TxId, ops: impl IntoIterator<Item = IndexOp<T>>) -> Result<()>
src/tx_buffer.rs:603  pub fn validate_pending_ops(&self, tx: TxId) -> Result<()>
src/tx_buffer.rs:621  pub fn drain(&self, tx: TxId) -> Vec<IndexOp<T>>
src/tx_buffer.rs:642  pub fn discard(&self, tx: TxId)
src/tx_buffer.rs:657  pub fn len(&self) -> usize
src/tx_buffer.rs:662  pub fn is_empty(&self) -> bool
src/tx_buffer.rs:667  pub fn reap_orphans_at(&self, now_nanos: u64) -> Vec<TxId>
src/tx_buffer.rs:672  pub fn reap_orphans(&self) -> Vec<TxId>
src/tx_buffer.rs:685  pub fn reap_orphans_bounded_at(&self, max: usize, now_nanos: u64) -> Vec<TxId>
src/tx_buffer.rs:721  pub fn reap_orphans_bounded(&self, max: usize) -> Vec<TxId>
src/tx_buffer.rs:727  pub fn get_ops(&self, tx: TxId) -> Option<Vec<IndexOp<T>>>
src/tx_buffer.rs:735  pub fn tx_timeout(&self) -> Duration
src/tx_buffer.rs:778  pub fn stage_kv(&self, tx: TxId, op: IndexOp<(Vec<u8>, Vec<u8>)>) -> Result<()>
src/tx_buffer.rs:808  pub fn drain_kv(&self, tx: TxId) -> Vec<IndexOp<(Vec<u8>, Vec<u8>)>>
src/tx_buffer.rs:817  pub fn discard_kv(&self, tx: TxId)
src/tx_buffer.rs:825  pub fn is_key_staged_for_tx(&self, tx_id: TxId, key: &[u8]) -> bool
src/tx_buffer.rs:845  pub fn staged_status(&self, key: &[u8]) -> Option<bool>
```

## Innen: crate-interne Grenzen (`pub(crate)`, `pub(super)`, `pub(in …)`)

```rust
src/snapshot.rs:371  pub(crate) fn release(&self, seq_no: u64)
src/snapshot.rs:381  pub(crate) fn release_at(&self, seq_no: u64, created_at: Option<u64>)
src/snapshot.rs:385  pub(crate) fn release_for_tenant_at(
```
