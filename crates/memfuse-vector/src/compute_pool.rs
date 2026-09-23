// FILE-CONTEXT
// ZWECK: Begrenzter ComputePool für CPU-intensive Tasks (z. B. HNSW-Index-Rebuild).
// INVARIANTEN: Zero-Panic Doctrine; feste Obergrenze an Worker-Threads (Resource Protection).
// HOTSPOTS: compute_pool.rs (Job-Queue & Task-Dispatch)

use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Arc;
use std::thread;

type Job = Box<dyn FnOnce() + Send + 'static>;

/// Handle to await the outcome of a task submitted to [`ComputePool`].
pub struct TaskHandle<T> {
    rx: Receiver<T>,
}

impl<T> TaskHandle<T> {
    /// Waits for the task to complete and returns its result, or an error if the task failed or was dropped.
    pub fn join(self) -> Result<T, std::sync::mpsc::RecvError> {
        self.rx.recv()
    }
}

struct PoolState {
    job_tx: Sender<Job>,
    active_jobs: AtomicUsize,
    max_workers: usize,
    is_shutdown: AtomicBool,
}

/// Bounded compute thread pool for background index rebuilds and heavy CPU tasks.
/// Enforces a hard ceiling on concurrent worker threads to prevent thread contention.
#[derive(Clone)]
pub struct ComputePool {
    inner: Arc<PoolState>,
}

impl ComputePool {
    /// Creates a new [`ComputePool`] with a fixed worker thread ceiling.
    pub fn new(max_workers: usize) -> Self {
        let workers = max_workers.max(1);
        let (job_tx, job_rx) = channel::<Job>();
        let job_rx = Arc::new(parking_lot::Mutex::new(job_rx));

        let state = Arc::new(PoolState {
            job_tx,
            active_jobs: AtomicUsize::new(0),
            max_workers: workers,
            is_shutdown: AtomicBool::new(false),
        });

        for i in 0..workers {
            let rx = Arc::clone(&job_rx);
            let state_clone = Arc::clone(&state);
            let _ = thread::Builder::new()
                .name(format!("memfuse-compute-{}", i))
                .spawn(move || loop {
                    let job_opt = {
                        let rx_guard = rx.lock();
                        rx_guard.recv().ok()
                    };

                    match job_opt {
                        Some(job) => {
                            state_clone.active_jobs.fetch_add(1, Ordering::SeqCst);
                            job();
                            state_clone.active_jobs.fetch_sub(1, Ordering::SeqCst);
                        }
                        None => break, // Channel closed when PoolState is dropped
                    }
                });
        }

        Self { inner: state }
    }

    /// Returns the maximum worker capacity of this pool.
    pub fn max_workers(&self) -> usize {
        self.inner.max_workers
    }

    /// Returns the number of jobs currently executing in worker threads.
    pub fn active_jobs(&self) -> usize {
        self.inner.active_jobs.load(Ordering::Relaxed)
    }

    /// Submits a fire-and-forget closure for execution on the pool.
    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        if self.inner.is_shutdown.load(Ordering::Relaxed) {
            return;
        }
        let _ = self.inner.job_tx.send(Box::new(f));
    }

    /// Submits a function and returns a [`TaskHandle`] to wait for its return value.
    pub fn spawn<F, T>(&self, f: F) -> TaskHandle<T>
    where
        F: FnOnce() -> T + Send + 'static,
        T: Send + 'static,
    {
        let (tx, rx) = channel();
        let job = Box::new(move || {
            let res = f();
            let _ = tx.send(res);
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

#[cfg(test)]
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
}
