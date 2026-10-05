// ZWECK: WalObserver Interface, Circuit-Breaker und Bounded-Dispatcher für Fail-Open Commit-Sicherheit.

use crate::wal::{WalEntry, WalOp};
use contextra_core::TxId;
use contextra_ports::{Clock, SystemClock};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{sync_channel, RecvTimeoutError, SyncSender};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

/// Default maximum latency allowed for WAL observer callback execution (1 millisecond).
pub const DEFAULT_MAX_OBSERVER_LATENCY: Duration = Duration::from_millis(1);

/// Origin tag indicating where a committed batch originated from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum WriteOrigin {
    /// Regular user/application write transaction.
    UserWrite,
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

/// Context describing a committed transaction passed to observers.
#[derive(Debug, Clone, Copy)]
pub struct CommitContext {
    pub tx_id: TxId,
    pub origin: WriteOrigin,
    pub durable: bool,
}

/// Batch of committed WAL entries passed to observers.
pub struct CommittedBatch<'a> {
    pub entries: Vec<WalEntryRef<'a>>,
    pub origin: WriteOrigin,
}

/// Synchronous, deterministic observer interface for committed WAL batches.
pub trait WalObserver: Send + Sync {
    /// Indicates whether this observer requires durable WAL commits.
    ///
    /// Returns `false` by default. Observers setting this to `false` will also receive notifications
    /// for `DurabilityMode::MemoryOnly` commits.
    fn requires_durability(&self) -> bool {
        false
    }

    /// Notification callback executed upon batch commit.
    ///
    /// Invoked via a bounded, non-blocking dispatcher mechanism. The commit path enforces a
    /// configurable maximum latency deadline (`max_observer_latency`). Observers taking longer
    /// than this deadline will time out on the commit path, increment drop counters, and trip
    /// the circuit breaker to prevent blocking subsequent transaction commits.
    fn on_commit(&self, batch: &CommittedBatch<'_>, seq_no: u64, tx_id: TxId);
}

struct CommitEvent {
    entries: Vec<WalEntry>,
    seq_no: u64,
    tx_id: TxId,
    origin: WriteOrigin,
}

enum WorkResult {
    Completed { elapsed_nanos: u64 },
    Panicked,
}

struct WorkerTask {
    event: CommitEvent,
    done_tx: SyncSender<WorkResult>,
}

struct BoundedObserverWorker {
    inner: Arc<dyn WalObserver>,
    sender: SyncSender<WorkerTask>,
    dropped_count: Arc<AtomicU64>,
    circuit_breaker_open_until_nanos: Arc<AtomicU64>,
    consecutive_drops: Arc<AtomicU64>,
    _thread_handle: Option<thread::JoinHandle<()>>,
}

impl BoundedObserverWorker {
    fn new(observer: Arc<dyn WalObserver>, clock: Arc<dyn Clock>) -> Self {
        let (task_tx, task_rx) = sync_channel::<WorkerTask>(256);
        let dropped_count = Arc::new(AtomicU64::new(0));
        let circuit_breaker_open_until_nanos = Arc::new(AtomicU64::new(0));
        let consecutive_drops = Arc::new(AtomicU64::new(0));

        let obs_clone = Arc::clone(&observer);
        let clock_clone = Arc::clone(&clock);

        let thread_handle = thread::Builder::new()
            .name("wal-observer-worker".to_string())
            .spawn(move || {
                while let Ok(task) = task_rx.recv() {
                    let wal_refs: Vec<WalEntryRef<'_>> =
                        task.event.entries.iter().map(WalEntryRef::new).collect();
                    let batch = CommittedBatch {
                        entries: wal_refs,
                        origin: task.event.origin,
                    };

                    let start_nanos = clock_clone.monotonic_nanos();
                    let res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                        obs_clone.on_commit(&batch, task.event.seq_no, task.event.tx_id);
                    }));
                    let end_nanos = clock_clone.monotonic_nanos();
                    let elapsed_nanos = end_nanos.saturating_sub(start_nanos);

                    let result = match res {
                        Ok(()) => WorkResult::Completed { elapsed_nanos },
                        Err(_) => WorkResult::Panicked,
                    };

                    let _ = task.done_tx.try_send(result);
                }
            })
            .ok();

        Self {
            inner: observer,
            sender: task_tx,
            dropped_count,
            circuit_breaker_open_until_nanos,
            consecutive_drops,
            _thread_handle: thread_handle,
        }
    }

    fn is_circuit_breaker_open(&self, now_nanos: u64) -> bool {
        let open_until = self
            .circuit_breaker_open_until_nanos
            .load(Ordering::Relaxed);
        open_until > 0 && now_nanos < open_until
    }
}

/// Configuration and handle for managing WAL observers with bounded delivery and circuit breaker.
pub struct ObserverRegistry {
    observers: parking_lot::RwLock<Vec<Arc<BoundedObserverWorker>>>,
    historical_drops: Arc<AtomicU64>,
    max_observer_latency: parking_lot::RwLock<Duration>,
    clock: parking_lot::RwLock<Arc<dyn Clock>>,
}

impl Default for ObserverRegistry {
    fn default() -> Self {
        Self {
            observers: parking_lot::RwLock::new(Vec::new()),
            historical_drops: Arc::new(AtomicU64::new(0)),
            max_observer_latency: parking_lot::RwLock::new(DEFAULT_MAX_OBSERVER_LATENCY),
            clock: parking_lot::RwLock::new(Arc::new(SystemClock::new())),
        }
    }
}

/// Non-blocking, bounded async adapter wrapper for `WalObserver` implementations.
pub struct AsyncObserverAdapter {
    inner: Arc<dyn WalObserver>,
    sender: tokio::sync::mpsc::Sender<(u64, TxId, WriteOrigin)>,
}

impl AsyncObserverAdapter {
    pub fn new(
        inner: Arc<dyn WalObserver>,
        capacity: usize,
    ) -> (Self, tokio::task::JoinHandle<()>) {
        let (tx, mut rx) = tokio::sync::mpsc::channel::<(u64, TxId, WriteOrigin)>(capacity);
        let obs_clone = Arc::clone(&inner);

        let handle = tokio::spawn(async move {
            while let Some((seq_no, tx_id, origin)) = rx.recv().await {
                let empty_batch = CommittedBatch {
                    entries: Vec::new(),
                    origin,
                };
                obs_clone.on_commit(&empty_batch, seq_no, tx_id);
            }
        });

        (Self { inner, sender: tx }, handle)
    }
}

impl WalObserver for AsyncObserverAdapter {
    fn requires_durability(&self) -> bool {
        self.inner.requires_durability()
    }

    fn on_commit(&self, _batch: &CommittedBatch<'_>, seq_no: u64, tx_id: TxId) {
        let _ = self
            .sender
            .try_send((seq_no, tx_id, WriteOrigin::UserWrite));
    }
}

impl ObserverRegistry {
    /// Returns true if no observers are registered.
    pub fn is_empty(&self) -> bool {
        self.observers.read().is_empty()
    }

    /// Creates a new, empty `ObserverRegistry`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers a new WAL observer.
    pub fn register_observer(&self, observer: Arc<dyn WalObserver>) {
        let clock = self.clock.read().clone();
        let worker = Arc::new(BoundedObserverWorker::new(observer, clock));
        self.observers.write().push(worker);
    }

    /// Deregisters an observer matching the provided `Arc` reference via pointer equality.
    pub fn deregister_observer(&self, observer: &Arc<dyn WalObserver>) {
        let mut guard = self.observers.write();
        guard.retain(|worker| {
            if Arc::ptr_eq(&worker.inner, observer) {
                self.historical_drops.fetch_add(
                    worker.dropped_count.load(Ordering::Relaxed),
                    Ordering::Relaxed,
                );
                false
            } else {
                true
            }
        });
    }

    /// Sets the maximum latency threshold for observer callbacks.
    pub fn set_max_observer_latency(&self, latency: Duration) {
        *self.max_observer_latency.write() = latency;
    }

    /// Returns the current maximum latency threshold.
    pub fn max_observer_latency(&self) -> Duration {
        *self.max_observer_latency.read()
    }

    /// Sets the injected clock port for deterministic execution timing (P28).
    pub fn set_clock(&self, clock: Arc<dyn Clock>) {
        *self.clock.write() = clock;
    }

    /// Returns the total number of dropped events across all observers (including deregistered ones).
    pub fn dropped_count(&self) -> u64 {
        let guard = self.observers.read();
        let active_drops: u64 = guard
            .iter()
            .map(|w| w.dropped_count.load(Ordering::Relaxed))
            .sum();
        active_drops + self.historical_drops.load(Ordering::Relaxed)
    }

    /// Returns the number of dropped events for a specific registered observer.
    // TODO(wiring): Facade-Anbindung in contextra/src/builder.rs folgt in separatem Task
    pub fn dropped_count_for(&self, observer: &Arc<dyn WalObserver>) -> u64 {
        let guard = self.observers.read();
        guard
            .iter()
            .find(|w| Arc::ptr_eq(&w.inner, observer))
            .map(|w| w.dropped_count.load(Ordering::Relaxed))
            .unwrap_or(0)
    }

    /// Returns whether the circuit breaker is currently open for a specific observer.
    pub fn is_circuit_breaker_open(&self, observer: &Arc<dyn WalObserver>) -> bool {
        let clock = self.clock.read().clone();
        let now_nanos = clock.monotonic_nanos();
        let guard = self.observers.read();
        guard
            .iter()
            .find(|w| Arc::ptr_eq(&w.inner, observer))
            .map(|w| w.is_circuit_breaker_open(now_nanos))
            .unwrap_or(false)
    }

    /// Returns whether any registered observer currently has an open circuit breaker.
    // TODO(wiring): Facade-Anbindung in contextra/src/builder.rs folgt in separatem Task
    pub fn is_any_circuit_breaker_open(&self) -> bool {
        let clock = self.clock.read().clone();
        let now_nanos = clock.monotonic_nanos();
        let guard = self.observers.read();
        guard.iter().any(|w| w.is_circuit_breaker_open(now_nanos))
    }

    /// Resets the circuit breaker state for a specific observer.
    // TODO(wiring): Facade-Anbindung in contextra/src/builder.rs folgt in separatem Task
    pub fn clear_circuit_breaker(&self, observer: &Arc<dyn WalObserver>) {
        let guard = self.observers.read();
        if let Some(worker) = guard.iter().find(|w| Arc::ptr_eq(&w.inner, observer)) {
            worker
                .circuit_breaker_open_until_nanos
                .store(0, Ordering::Relaxed);
            worker.consecutive_drops.store(0, Ordering::Relaxed);
        }
    }

    /// Synchronously notifies all registered observers of a committed batch with explicit CommitContext.
    ///
    /// When invoked within a multi-threaded Tokio async runtime context, blocking execution is offloaded via
    /// `tokio::task::block_in_place` to ensure transaction commit paths on Tokio runtime worker threads
    /// remain non-blocking.
    pub fn notify_with_context(&self, entries: &[WalEntry], seq_no: u64, ctx: CommitContext) {
        let observers_snapshot = {
            let guard = self.observers.read();
            if guard.is_empty() {
                return;
            }
            guard.clone()
        };

        let clock = self.clock.read().clone();
        let max_latency = self.max_observer_latency();
        let max_latency_nanos = max_latency.as_nanos() as u64;
        let now_nanos = clock.monotonic_nanos();

        // Circuit breaker cooldown: 1 second
        let cooldown_nanos = 1_000_000_000u64;

        let mut panicking: Vec<Arc<dyn WalObserver>> = Vec::new();

        let mut run_notify = || {
            for worker in &observers_snapshot {
                if !ctx.durable && worker.inner.requires_durability() {
                    continue;
                }

                if worker.is_circuit_breaker_open(now_nanos) {
                    worker.dropped_count.fetch_add(1, Ordering::Relaxed);
                    continue;
                }

                let event = CommitEvent {
                    entries: entries.to_vec(),
                    seq_no,
                    tx_id: ctx.tx_id,
                    origin: ctx.origin,
                };

                let (done_tx, done_rx) = sync_channel(1);
                let task = WorkerTask { event, done_tx };

                if worker.sender.try_send(task).is_err() {
                    worker.dropped_count.fetch_add(1, Ordering::Relaxed);
                    worker.consecutive_drops.fetch_add(1, Ordering::Relaxed);
                    worker
                        .circuit_breaker_open_until_nanos
                        .store(now_nanos.saturating_add(cooldown_nanos), Ordering::Relaxed);
                    tracing::warn!(
                        tx_id = ctx.tx_id.inner(),
                        "WalObserver queue full; dropped event and tripped circuit breaker"
                    );
                    continue;
                }

                match done_rx.recv_timeout(max_latency) {
                    Ok(WorkResult::Completed { elapsed_nanos }) => {
                        if elapsed_nanos > max_latency_nanos {
                            worker.dropped_count.fetch_add(1, Ordering::Relaxed);
                            worker.consecutive_drops.fetch_add(1, Ordering::Relaxed);
                            worker
                                .circuit_breaker_open_until_nanos
                                .store(now_nanos.saturating_add(cooldown_nanos), Ordering::Relaxed);
                            tracing::warn!(
                                elapsed_us = elapsed_nanos / 1_000,
                                max_allowed_us = max_latency_nanos / 1_000,
                                tx_id = ctx.tx_id.inner(),
                                "WalObserver exceeded max_observer_latency; tripped circuit breaker"
                            );
                        } else {
                            worker.consecutive_drops.store(0, Ordering::Relaxed);
                            worker
                                .circuit_breaker_open_until_nanos
                                .store(0, Ordering::Relaxed);
                        }
                    }
                    Ok(WorkResult::Panicked) => {
                        worker.dropped_count.fetch_add(1, Ordering::Relaxed);
                        worker
                            .circuit_breaker_open_until_nanos
                            .store(now_nanos.saturating_add(cooldown_nanos), Ordering::Relaxed);
                        tracing::error!(
                            tx_id = ctx.tx_id.inner(),
                            "WalObserver panicked during on_commit execution; deregistering observer"
                        );
                        panicking.push(Arc::clone(&worker.inner));
                    }
                    Err(RecvTimeoutError::Timeout) => {
                        worker.dropped_count.fetch_add(1, Ordering::Relaxed);
                        worker.consecutive_drops.fetch_add(1, Ordering::Relaxed);
                        worker
                            .circuit_breaker_open_until_nanos
                            .store(now_nanos.saturating_add(cooldown_nanos), Ordering::Relaxed);
                        tracing::warn!(
                            max_allowed_us = max_latency_nanos / 1_000,
                            tx_id = ctx.tx_id.inner(),
                            "WalObserver timed out on commit path; tripped circuit breaker and continuing"
                        );
                    }
                    Err(RecvTimeoutError::Disconnected) => {
                        worker.dropped_count.fetch_add(1, Ordering::Relaxed);
                        worker
                            .circuit_breaker_open_until_nanos
                            .store(now_nanos.saturating_add(cooldown_nanos), Ordering::Relaxed);
                    }
                }
            }
        };

        if let Ok(handle) = tokio::runtime::Handle::try_current() {
            if handle.runtime_flavor() == tokio::runtime::RuntimeFlavor::MultiThread {
                tokio::task::block_in_place(run_notify);
            } else {
                run_notify();
            }
        } else {
            run_notify();
        }

        if !panicking.is_empty() {
            let mut guard = self.observers.write();
            guard.retain(|w| !panicking.iter().any(|p| Arc::ptr_eq(&w.inner, p)));
        }
    }

    /// Synchronously notifies all registered observers of a committed batch.
    pub fn notify(&self, entries: &[WalEntry], seq_no: u64, tx_id: TxId, origin: WriteOrigin) {
        self.notify_with_context(
            entries,
            seq_no,
            CommitContext {
                tx_id,
                origin,
                durable: true,
            },
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_write_origin_user_write() {
        let origin = WriteOrigin::UserWrite;
        assert_eq!(origin, WriteOrigin::UserWrite);
    }
}
