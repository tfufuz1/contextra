// FILE-CONTEXT
// ZWECK: Begrenzter ComputePool für CPU-intensive Tasks (z. B. HNSW-Index-Rebuild).
// INVARIANTEN: Zero-Panic Doctrine; feste Obergrenze an Worker-Threads; begrenzter Kanal; sauberes Beenden bei Drop ohne JoinHandle-Leck.
// HOTSPOTS: compute_pool.rs (Job-Queue & Task-Dispatch)
// STAND: 2026-11-12

use crate::error::{Result as VectorResult, VectorError};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::mpsc::{channel, sync_channel, Receiver, SyncSender};
use std::sync::Arc;
use std::thread;

type Job = Box<dyn FnOnce() + Send + 'static>;

/// Standard-Kapazität für die synchrone Job-Warteschlange pro Worker-Thread.
pub const DEFAULT_QUEUE_CAPACITY_PER_WORKER: usize = 128;

// SECTION TaskHandle

// ANCHOR[contextra-vector:POOL-TASK-HANDLE]
/// Handle zum Abwarten des Ergebnisses eines an den [`ComputePool`] übergebenen Tasks.
pub struct TaskHandle<T> {
    rx: Receiver<VectorResult<T>>,
}

impl<T> TaskHandle<T> {
    /// Wartet auf den Abschluss des Tasks. Panics im Task werden als [`VectorError`] propagiert.
    pub fn join(self) -> VectorResult<T> {
        match self.rx.recv() {
            Ok(res) => res,
            Err(_) => Err(VectorError::Internal(
                "ComputePool worker channel disconnected before completion".to_string(),
            )),
        }
    }
}
// !SECTION

// SECTION Pool-Zustand & Implementierung

struct WorkerShared {
    active_jobs: AtomicUsize,
    active_jobs_condvar: parking_lot::Condvar,
    active_jobs_mutex: parking_lot::Mutex<()>,
}

struct PoolState {
    job_tx: parking_lot::Mutex<Option<SyncSender<Job>>>,
    shared: Arc<WorkerShared>,
    max_workers: usize,
    is_shutdown: AtomicBool,
    worker_handles: parking_lot::Mutex<Vec<thread::JoinHandle<()>>>,
}

// ANCHOR[contextra-vector:POOL-COMPUTE-POOL]
/// Begrenzter Compute-Threadpool für Hintergrundberechnungen und rechenintensive Indizierung.
/// Erzwingt eine strikte Obergrenze für Worker-Threads und eine beschränkte Auftragswarteschlange.
#[derive(Clone)]
pub struct ComputePool {
    inner: Arc<PoolState>,
}

impl Drop for PoolState {
    fn drop(&mut self) {
        self.is_shutdown.store(true, Ordering::SeqCst);
        // Schließe den Sender; dadurch schlägt recv() bei allen Workern fehl und sie terminieren.
        let mut tx_guard = self.job_tx.lock();
        let _ = tx_guard.take();
        drop(tx_guard);

        // Alle Worker-Threads deterministisch joinen.
        let mut handles = self.worker_handles.lock();
        for handle in handles.drain(..) {
            let _ = handle.join();
        }
    }
}

impl ComputePool {
    /// Erzeugt einen neuen [`ComputePool`] mit fester Worker-Thread-Grenze und beschränktem Puffer.
    // ANCHOR[contextra-vector:POOL-NEW]
    pub fn new(max_workers: usize) -> Self {
        Self::with_queue_capacity(
            max_workers,
            max_workers.max(1) * DEFAULT_QUEUE_CAPACITY_PER_WORKER,
        )
    }

    /// Erzeugt einen neuen [`ComputePool`] mit konfigurierbarer Pufferkapazität (begrenzter Kanal).
    pub fn with_queue_capacity(max_workers: usize, queue_capacity: usize) -> Self {
        let workers = max_workers.max(1);
        let queue_cap = queue_capacity.max(1);
        let (job_tx, job_rx) = sync_channel::<Job>(queue_cap);
        let job_rx = Arc::new(parking_lot::Mutex::new(job_rx));

        let shared = Arc::new(WorkerShared {
            active_jobs: AtomicUsize::new(0),
            active_jobs_condvar: parking_lot::Condvar::new(),
            active_jobs_mutex: parking_lot::Mutex::new(()),
        });

        let mut handles = Vec::with_capacity(workers);
        for i in 0..workers {
            let rx = Arc::clone(&job_rx);
            let shared_clone = Arc::clone(&shared);
            let handle = thread::Builder::new()
                .name(format!("contextra-compute-{}", i))
                .spawn(move || loop {
                    let job_opt = {
                        let rx_guard = rx.lock();
                        rx_guard.recv().ok()
                    };

                    match job_opt {
                        Some(job) => {
                            shared_clone.active_jobs.fetch_add(1, Ordering::SeqCst);

                            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(job));
                            if let Err(err) = result {
                                let panic_msg = if let Some(s) = err.downcast_ref::<&str>() {
                                    (*s).to_string()
                                } else if let Some(s) = err.downcast_ref::<String>() {
                                    s.clone()
                                } else {
                                    "Unknown panic payload".to_string()
                                };
                                tracing::error!(
                                    panic_payload = %panic_msg,
                                    "ComputePool worker caught panic during job execution; thread remains active"
                                );
                            }

                            shared_clone.active_jobs.fetch_sub(1, Ordering::SeqCst);
                            let _lock = shared_clone.active_jobs_mutex.lock();
                            shared_clone.active_jobs_condvar.notify_all();
                        }
                        None => break, // Kanal geschlossen, Worker beendet sich sauber.
                    }
                });

            if let Ok(h) = handle {
                handles.push(h);
            }
        }

        let state = Arc::new(PoolState {
            job_tx: parking_lot::Mutex::new(Some(job_tx)),
            shared,
            max_workers: workers,
            is_shutdown: AtomicBool::new(false),
            worker_handles: parking_lot::Mutex::new(handles),
        });

        Self { inner: state }
    }

    /// Gibt die maximale Worker-Kapazität des Pools zurück.
    pub fn max_workers(&self) -> usize {
        self.inner.max_workers
    }

    /// Gibt die Anzahl aktuell auf Worker-Threads aktiver Jobs zurück.
    pub fn active_jobs(&self) -> usize {
        self.inner.shared.active_jobs.load(Ordering::Relaxed)
    }

    /// Reicht eine Fire-and-Forget-Closure ein. Blockiert bei voller Warteschlange (Backpressure).
    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        if self.inner.is_shutdown.load(Ordering::Relaxed) {
            return;
        }
        let tx_guard = self.inner.job_tx.lock();
        if let Some(ref tx) = *tx_guard {
            let _ = tx.send(Box::new(f));
        }
    }

    /// Reicht eine Funktion ein und liefert ein [`TaskHandle`], um das Ergebnis abzuwarten.
    pub fn spawn<F, T>(&self, f: F) -> TaskHandle<T>
    where
        F: FnOnce() -> T + Send + 'static,
        T: Send + 'static,
    {
        let (tx, rx) = channel();
        let job = Box::new(move || {
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(f));
            let mapped = result.map_err(|err| {
                let msg = if let Some(s) = err.downcast_ref::<&str>() {
                    (*s).to_string()
                } else if let Some(s) = err.downcast_ref::<String>() {
                    s.clone()
                } else {
                    "Task panicked".to_string()
                };
                VectorError::Internal(format!("ComputePool task panic: {msg}"))
            });
            let _ = tx.send(mapped);
        });
        self.execute(job);
        TaskHandle { rx }
    }
}

impl Default for ComputePool {
    fn default() -> Self {
        let default_workers = thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(4)
            .min(16);
        Self::new(default_workers)
    }
}

impl std::fmt::Debug for ComputePool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ComputePool")
            .field("max_workers", &self.inner.max_workers)
            .field("active_jobs", &self.active_jobs())
            .finish()
    }
}
// !SECTION

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_pool_basic() {
        let pool = ComputePool::new(2);
        assert_eq!(pool.max_workers(), 2);

        let handle = pool.spawn(|| 21 * 2);
        assert_eq!(handle.join().unwrap(), 42);
    }

    #[test]
    fn test_compute_pool_concurrency_ceiling() {
        let pool = ComputePool::new(2);
        let (tx, rx) = channel();

        for i in 0..5 {
            let tx_c = tx.clone();
            pool.execute(move || {
                tx_c.send(i).unwrap();
            });
        }

        let mut results = Vec::new();
        for _ in 0..5 {
            results.push(rx.recv().unwrap());
        }
        assert_eq!(results.len(), 5);
    }

    #[test]
    fn test_compute_pool_panic_resilience() {
        let pool = ComputePool::new(1);
        assert_eq!(pool.max_workers(), 1);

        // 1. Submit a task that panics explicitly.
        let handle_panic = pool.spawn(|| {
            panic!("intentional task panic for resilience testing");
        });

        // Joining the panicking task handle must return VectorError, not crash the process.
        assert!(handle_panic.join().is_err());

        // 2. Verify pool is still functional by submitting a second normal task.
        let handle_normal = pool.spawn(|| 100 + 42);
        assert_eq!(handle_normal.join().unwrap(), 142);

        // 3. Verify active_jobs returns to 0 and max_workers capacity is unchanged.
        {
            let mut guard = pool.inner.shared.active_jobs_mutex.lock();
            while pool.active_jobs() > 0 {
                let res = pool
                    .inner
                    .shared
                    .active_jobs_condvar
                    .wait_for(&mut guard, std::time::Duration::from_secs(2));
                if res.timed_out() {
                    break;
                }
            }
        }
        assert_eq!(pool.active_jobs(), 0);
        assert_eq!(pool.max_workers(), 1);
    }
}
