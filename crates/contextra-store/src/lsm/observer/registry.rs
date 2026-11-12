//! # Observer-Registry & Benachrichtigung (Ring 1)
//!
//! // FILE-CONTEXT
//! // STAND: 2026-11-11
//! // ZWECK: Zentrale Verwaltung und synchrone Benachrichtigung registrierter Observer
//! // INVARIANTEN: Fail-Open, Circuit-Breaker, INV-LOCK-ORDER-1

use super::types::{CommitContext, WalObserver, WriteOrigin, DEFAULT_MAX_OBSERVER_LATENCY};
use super::worker::{BoundedObserverWorker, CommitEvent, WorkResult, WorkerTask};
use crate::wal::WalEntry;
use contextra_env::{Clock, SystemClock};
use contextra_ids::TxId;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{sync_channel, RecvTimeoutError};
use std::sync::Arc;
use std::time::Duration;

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
    pub fn is_any_circuit_breaker_open(&self) -> bool {
        let clock = self.clock.read().clone();
        let now_nanos = clock.monotonic_nanos();
        let guard = self.observers.read();
        guard.iter().any(|w| w.is_circuit_breaker_open(now_nanos))
    }

    /// Resets the circuit breaker state for a specific observer.
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
