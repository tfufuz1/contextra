//! Local port traits decoupled from the removed `contextra-ports` crate.

use crate::contextra_types::{Result, TxId, WorkflowState};
use std::future::Future;
use std::path::Path;
use std::pin::Pin;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

/// Boxed `Send` future alias.
pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// Monotonic / wall-clock time source.
pub trait Clock: Send + Sync {
    /// Current wall-clock timestamp in nanoseconds since UNIX epoch.
    fn now_unix_nanos(&self) -> u64;
    /// Monotonic timestamp in nanoseconds.
    fn monotonic_nanos(&self) -> u64;
}

/// Default system clock implementation.
#[derive(Debug, Clone, Copy, Default)]
pub struct SystemClock {
    _private: (),
}

impl SystemClock {
    /// Creates a new `SystemClock`.
    #[inline]
    pub const fn new() -> Self {
        Self { _private: () }
    }
}

impl Clock for SystemClock {
    fn now_unix_nanos(&self) -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| u64::try_from(d.as_nanos()).unwrap_or(u64::MAX))
            .unwrap_or(0)
    }

    fn monotonic_nanos(&self) -> u64 {
        static START: std::sync::OnceLock<Instant> = std::sync::OnceLock::new();
        let start = START.get_or_init(Instant::now);
        u64::try_from(start.elapsed().as_nanos()).unwrap_or(u64::MAX)
    }
}

/// Deterministic manual clock for tests.
#[derive(Debug, Default)]
pub struct ManualClock {
    nanos: std::sync::atomic::AtomicU64,
}

impl ManualClock {
    /// Creates a new `ManualClock` initialized to `initial_nanos`.
    pub fn new(initial_nanos: u64) -> Self {
        Self {
            nanos: std::sync::atomic::AtomicU64::new(initial_nanos),
        }
    }

    /// Returns the current timestamp in nanoseconds.
    pub fn now_nanos(&self) -> u64 {
        self.nanos.load(std::sync::atomic::Ordering::SeqCst)
    }

    /// Advances the clock by `delta`.
    pub fn advance(&self, delta: std::time::Duration) {
        let delta_nanos = u64::try_from(delta.as_nanos()).unwrap_or(u64::MAX);
        self.nanos
            .fetch_add(delta_nanos, std::sync::atomic::Ordering::SeqCst);
    }
}

impl Clock for ManualClock {
    fn now_unix_nanos(&self) -> u64 {
        self.nanos.load(std::sync::atomic::Ordering::SeqCst)
    }

    fn monotonic_nanos(&self) -> u64 {
        self.nanos.load(std::sync::atomic::Ordering::SeqCst)
    }
}

/// Metrics sink trait for checkpoint observability.
pub trait MetricsSink: Send + Sync {
    /// Records a counter increment.
    fn increment_counter(&self, name: &str, value: u64, labels: &[(&str, &str)]);
    /// Records a gauge value.
    fn record_gauge(&self, name: &str, value: f64, labels: &[(&str, &str)]);
    /// Records a histogram observation.
    fn record_histogram(&self, _name: &str, _value: f64, _labels: &[(&str, &str)]) {}
}

/// Minimal virtual filesystem abstraction for crash-consistent persistence.
pub trait Vfs: Send + Sync {
    /// Atomically writes `data` to `path`.
    fn write_atomic(&self, path: &Path, data: &[u8]) -> Result<()>;
    /// Reads all bytes from `path`.
    fn read(&self, path: &Path) -> Result<Vec<u8>>;
}

/// Storage engine statistics.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct StorageStats {
    /// Number of segments.
    pub num_segments: usize,
    /// Total size in bytes.
    pub total_size_bytes: u64,
    /// Memtable size in bytes.
    pub memtable_size_bytes: u64,
}

/// Storage engine port required by `PersistentCheckpointStore` and `CheckpointGuard`.
pub trait StorageEngine: Send + Sync {
    /// Fetches the latest value for `key`.
    fn get<'a>(&'a self, key: &'a [u8]) -> BoxFuture<'a, Result<Option<bytes::Bytes>>>;
    /// Fetches the value for `key` at sequence `seq`.
    fn get_at_seq<'a>(
        &'a self,
        key: &'a [u8],
        seq: u64,
    ) -> BoxFuture<'a, Result<Option<bytes::Bytes>>>;
    /// Inserts or updates `key` within `tx_id`.
    fn put<'a>(&'a self, tx_id: TxId, key: &'a [u8], value: &'a [u8]) -> BoxFuture<'a, Result<()>>;
    /// Deletes `key` within `tx_id`.
    fn delete<'a>(&'a self, tx_id: TxId, key: &'a [u8]) -> BoxFuture<'a, Result<()>>;
    /// Commits `tx_id`.
    fn commit<'a>(&'a self, tx_id: TxId) -> BoxFuture<'a, Result<()>>;
    /// Rolls back `tx_id`.
    fn rollback<'a>(&'a self, tx_id: TxId) -> BoxFuture<'a, Result<()>>;
    /// Rolls back storage state to `tx_id`.
    fn rollback_to_tx<'a>(&'a self, tx_id: TxId) -> BoxFuture<'a, Result<()>>;
    /// Returns the last committed sequence number.
    fn last_seq_no<'a>(&'a self) -> BoxFuture<'a, Result<u64>>;
    /// Returns the last committed transaction ID.
    fn last_tx_id<'a>(&'a self) -> BoxFuture<'a, Result<TxId>>;
    /// Flushes pending writes to durable storage.
    fn flush<'a>(&'a self) -> BoxFuture<'a, Result<()>>;
    /// Returns storage statistics.
    fn stats<'a>(&'a self) -> BoxFuture<'a, Result<StorageStats>>;
    /// Pins a checkpoint at `seq_no` against compaction/GC.
    fn pin_checkpoint<'a>(&'a self, seq_no: u64) -> BoxFuture<'a, Result<()>>;
    /// Unpins a checkpoint at `seq_no`.
    fn unpin_checkpoint<'a>(&'a self, seq_no: u64) -> BoxFuture<'a, Result<()>>;
    /// Scans all key-value pairs matching `prefix`.
    fn scan_prefix<'a>(
        &'a self,
        prefix: &'a [u8],
    ) -> BoxFuture<'a, Result<Vec<(Vec<u8>, Vec<u8>)>>>;
}

/// Named checkpoint coordinator port.
pub trait CheckpointCoordinator: Send + Sync {
    /// Checkpoint metadata type.
    type Meta: Send + Sync;

    /// Creates a named checkpoint.
    fn create_named_checkpoint(
        &self,
        name: &str,
        collection_id: &str,
        seq_no: u64,
        tx_id: TxId,
        metadata: serde_json::Value,
    ) -> impl Future<Output = Result<Self::Meta>> + Send;

    /// Restores a named checkpoint.
    fn restore_named_checkpoint(&self, name: &str) -> impl Future<Output = Result<Self::Meta>> + Send;

    /// Drops a named checkpoint.
    fn drop_named_checkpoint(&self, name: &str) -> impl Future<Output = Result<()>> + Send;

    /// Lists all named checkpoints.
    fn list_named_checkpoints(&self) -> impl Future<Output = Result<Vec<Self::Meta>>> + Send;
}

/// Low-level snapshot/restore port.
pub trait Checkpoint: Send + Sync {
    /// Takes a snapshot for `tx`.
    fn take_snapshot<'a>(&'a self, tx: TxId) -> BoxFuture<'a, Result<WorkflowState>>;
    /// Restores state from `state`.
    fn restore<'a>(&'a self, state: &'a WorkflowState) -> BoxFuture<'a, Result<()>>;
}
