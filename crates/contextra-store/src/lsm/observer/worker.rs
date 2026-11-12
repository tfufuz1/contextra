//! # Bounded Observer Worker & Hintergrund-Thread (Ring 1)
//!
//! // FILE-CONTEXT
//! // STAND: 2026-11-11
//! // ZWECK: Bounded Channel & Circuit Breaker fuer WalObserver
//! // INVARIANTEN: Fail-open Semantik, Panic-Isolation

use super::types::{CommittedBatch, WalEntryRef, WalObserver, WriteOrigin};
use crate::wal::WalEntry;
use contextra_env::Clock;
use contextra_ids::TxId;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{sync_channel, SyncSender};
use std::sync::Arc;
use std::thread;

pub(super) struct CommitEvent {
    pub(super) entries: Vec<WalEntry>,
    pub(super) seq_no: u64,
    pub(super) tx_id: TxId,
    pub(super) origin: WriteOrigin,
}

pub(super) enum WorkResult {
    Completed { elapsed_nanos: u64 },
    Panicked,
}

pub(super) struct WorkerTask {
    pub(super) event: CommitEvent,
    pub(super) done_tx: SyncSender<WorkResult>,
}

pub(super) struct BoundedObserverWorker {
    pub(super) inner: Arc<dyn WalObserver>,
    pub(super) sender: SyncSender<WorkerTask>,
    pub(super) dropped_count: Arc<AtomicU64>,
    pub(super) circuit_breaker_open_until_nanos: Arc<AtomicU64>,
    pub(super) consecutive_drops: Arc<AtomicU64>,
    pub(super) _thread_handle: Option<thread::JoinHandle<()>>,
}

impl BoundedObserverWorker {
    pub(super) fn new(observer: Arc<dyn WalObserver>, clock: Arc<dyn Clock>) -> Self {
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

    pub(super) fn is_circuit_breaker_open(&self, now_nanos: u64) -> bool {
        let open_until = self
            .circuit_breaker_open_until_nanos
            .load(Ordering::Relaxed);
        open_until > 0 && now_nanos < open_until
    }
}
