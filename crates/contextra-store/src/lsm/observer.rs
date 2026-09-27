use contextra_core::TxId;
use std::sync::Arc;
use std::time::{Duration, Instant};
use crate::wal::{WalEntry, WalOp};

/// Origin tag indicating where a committed batch originated from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WriteOrigin {
    /// Regular user/application write transaction.
    UserWrite,
    /// Background consolidation from the cognition pipeline.
    CognitionConsolidation,
    /// Compaction rewrite operation.
    CompactionRewrite,
}

/// Lightweight view reference into a `WalEntry`.
#[derive(Debug, Clone, Copy)]
pub struct WalEntryRef<'a> {
    pub entry: &'a WalEntry,
}

impl<'a> WalEntryRef<'a> {
    /// Creates a new `WalEntryRef` from a `WalEntry`.
    pub fn new(entry: &'a WalEntry) -> Self {
        Self { entry }
    }

    /// Returns a reference to the operation inside the entry.
    pub fn op(&self) -> &'a WalOp {
        &self.entry.op
    }

    /// Returns the sequence number of the entry.
    pub fn seq_no(&self) -> u64 {
        self.entry.seq_no
    }

    /// Returns a reference to the entry's checksum.
    pub fn checksum(&self) -> &'a [u8; 32] {
        &self.entry.checksum
    }

    /// Returns a reference to the entry's previous HMAC checksum.
    pub fn prev_hmac(&self) -> &'a [u8; 32] {
        &self.entry.prev_hmac
    }

    /// Returns the transaction ID associated with this entry.
    pub fn tx_id(&self) -> TxId {
        self.entry.tx_id()
    }
}

/// Batch of committed WAL entries passed to observers.
pub struct CommittedBatch<'a> {
    pub entries: Vec<WalEntryRef<'a>>,
    pub origin: WriteOrigin,
}

/// Synchronous, deterministic observer interface for committed WAL batches.
pub trait WalObserver: Send + Sync {
    /// Notification callback executed synchronously upon batch commit.
    ///
    /// # Fail-Open Contract
    /// Callbacks MUST NOT block longer than `max_observer_latency` (default 1ms).
    /// If an observer exceeds this limit, it is automatically deregistered and an `ObserverTimeout`
    /// warning is logged. The commit itself is NEVER delayed or aborted.
    fn on_commit(&self, batch: &CommittedBatch<'_>, seq_no: u64, tx_id: TxId);
}

/// Configuration and handle for managing WAL observers.
pub struct ObserverRegistry {
    observers: parking_lot::RwLock<Vec<Arc<dyn WalObserver>>>,
    max_observer_latency: parking_lot::RwLock<Duration>,
}

impl Default for ObserverRegistry {
    fn default() -> Self {
        Self {
            observers: parking_lot::RwLock::new(Vec::new()),
            max_observer_latency: parking_lot::RwLock::new(Duration::from_millis(1)),
        }
    }
}

impl ObserverRegistry {
    /// Creates a new, empty `ObserverRegistry`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers a new WAL observer.
    pub fn register_observer(&self, observer: Arc<dyn WalObserver>) {
        self.observers.write().push(observer);
    }

    /// Deregisters an observer matching the provided `Arc` reference via pointer equality.
    pub fn deregister_observer(&self, observer: &Arc<dyn WalObserver>) {
        self.observers
            .write()
            .retain(|obs| !Arc::ptr_eq(obs, observer));
    }

    /// Sets the maximum latency threshold for observer callbacks.
    pub fn set_max_observer_latency(&self, latency: Duration) {
        *self.max_observer_latency.write() = latency;
    }

    /// Returns the current maximum latency threshold.
    pub fn max_observer_latency(&self) -> Duration {
        *self.max_observer_latency.read()
    }

    /// Synchronously notifies all registered observers of a committed batch.
    ///
    /// Evaluates execution time per observer against `max_observer_latency`. Timed out observers
    /// are evicted without failing the commit.
    pub fn notify(&self, entries: &[WalEntry], seq_no: u64, tx_id: TxId, origin: WriteOrigin) {
        let observers_snapshot = {
            let guard = self.observers.read();
            if guard.is_empty() {
                return;
            }
            guard.clone()
        };

        let wal_refs: Vec<WalEntryRef<'_>> = entries.iter().map(WalEntryRef::new).collect();
        let batch = CommittedBatch {
            entries: wal_refs,
            origin,
        };

        let max_latency = self.max_observer_latency();
        let mut timed_out: Vec<Arc<dyn WalObserver>> = Vec::new();

        for obs in &observers_snapshot {
            let start = Instant::now();
            obs.on_commit(&batch, seq_no, tx_id);
            let elapsed = start.elapsed();

            if elapsed > max_latency {
                tracing::warn!(
                    elapsed_us = elapsed.as_micros(),
                    max_allowed_us = max_latency.as_micros(),
                    tx_id = tx_id.inner(),
                    "WalObserver exceeded max_observer_latency; deregistering observer"
                );
                timed_out.push(Arc::clone(obs));
            }
        }

        if !timed_out.is_empty() {
            let mut guard = self.observers.write();
            guard.retain(|obs| !timed_out.iter().any(|to| Arc::ptr_eq(obs, to)));
        }
    }
}
