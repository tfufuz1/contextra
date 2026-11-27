//! Engine orchestration building blocks for durable erasure (Spezifikation 10).
//!
//! Layout: [`config`] (options), [`registry`] (layer order), [`resolver`] (scope planning),
//! [`driver`] (saga), [`budget`] (I/O throttling), [`caches`] (RAM observers),
//! [`adapters`] (port glue), [`api`] (`impl Contextra`), [`types`] (result types).

pub mod adapters;
pub mod api;
pub mod budget;
pub mod caches;
pub mod config;
pub mod driver;
pub mod registry;
pub mod resolver;
pub mod types;

pub use adapters::{KernelProgress, TokenCancel};
pub use budget::EngineIoBudget;
pub use caches::{EpochCache, ObserverFn};
pub use config::{EraseOptions, ErasureConfig};
pub use driver::{DriveOutcome, JobSched};
pub use registry::{LayerOrder, LayerRegistry};
pub use resolver::ScopeResolverImpl;
pub use types::{EraseHandle, ErasePlanSummary};

use contextra_ports::{Clock, ErasureKernel, ErasureObserver, Rng};
use contextra_store::LsmStorage;
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::Notify;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;
use tokio_util::task::TaskTracker;

/// Shared state of the erasure subsystem. Held behind an `Arc` by the worker and the API.
pub struct ErasureRuntimeInner {
    /// Kernel port of the store (K-24).
    pub kernel: Arc<dyn ErasureKernel>,
    /// Registered secondary layers.
    pub registry: parking_lot::RwLock<LayerRegistry>,
    /// RAM-cache observers (never blocking, errors only counted).
    pub observers: parking_lot::RwLock<Vec<Arc<dyn ErasureObserver>>>,
    /// Static configuration.
    pub cfg: ErasureConfig,
    /// Deterministic clock (only time source).
    pub clock: Arc<dyn Clock>,
    /// Jitter / id randomness source (K-12).
    pub rng: Arc<dyn Rng>,
    /// I/O budget for G3 work. G1 is never throttled.
    pub budget: Arc<EngineIoBudget>,
    /// Wakes the worker (new job, boot, pending-delete, pressure change).
    pub notify: Arc<Notify>,
    /// Signalled after every worker pass; waiters (`AwaitLevel::Purged`) re-check status.
    pub progress: Arc<Notify>,
    /// Cancellation shared with the engine.
    pub cancel: CancellationToken,
    /// Set once shutdown begins; `erase()` then fails.
    pub shutting_down: AtomicBool,
    /// Per-job scheduling state.
    pub sched: parking_lot::Mutex<BTreeMap<contextra_types::ErasureId, JobSched>>,
    /// Number of failed observer deliveries.
    pub observer_failures: AtomicU64,
    /// Number of jobs that exceeded their SLA.
    pub sla_breaches: AtomicU64,
    /// Number of `purge_store` rounds started by the worker (coalescing metric).
    pub purge_rounds: AtomicU64,
    /// Snapshot registry for pin reaping while jobs are open (04 §4).
    pub snapshots: Option<Arc<contextra_mvcc::SnapshotRegistry>>,
}

/// Owner handle: starts and stops the worker.
pub struct ErasureRuntime {
    inner: Arc<ErasureRuntimeInner>,
    worker: parking_lot::Mutex<Option<JoinHandle<()>>>,
}

impl ErasureRuntime {
    /// Builds and starts the runtime (Spezifikation §11).
    ///
    /// 1. Kernel from the store. 2. Observers are registered by the caller via
    /// [`Self::inner`]. 3. Open jobs get `on_gate` on all registered layers **before** traffic.
    /// 4. The worker starts and is woken if jobs are open.
    pub async fn start(
        storage: &Arc<LsmStorage>,
        registry: LayerRegistry,
        cfg: ErasureConfig,
        rng: Option<Arc<dyn Rng>>,
        cancel: CancellationToken,
        tracker: &TaskTracker,
    ) -> Self {
        // Child token: `stop()` ends only the erasure tasks, engine shutdown still cascades.
        let cancel = cancel.child_token();
        let clock = storage.clock();
        let rng: Arc<dyn Rng> = rng.unwrap_or_else(|| {
            let seed = clock.now_unix_nanos() ^ clock.monotonic_nanos().rotate_left(32);
            Arc::new(contextra_ports::SeededRng::new(seed))
        });
        let budget = Arc::new(EngineIoBudget::new(
            0,
            cfg.erase_io_share,
            Arc::clone(&clock),
            storage.pressure_receiver(),
        ));
        let inner = Arc::new(ErasureRuntimeInner {
            kernel: storage.erasure_kernel(),
            registry: parking_lot::RwLock::new(registry),
            observers: parking_lot::RwLock::new(Vec::new()),
            cfg,
            clock,
            rng,
            budget,
            notify: Arc::new(Notify::new()),
            progress: Arc::new(Notify::new()),
            cancel: cancel.clone(),
            shutting_down: AtomicBool::new(false),
            sched: parking_lot::Mutex::new(BTreeMap::new()),
            observer_failures: AtomicU64::new(0),
            sla_breaches: AtomicU64::new(0),
            purge_rounds: AtomicU64::new(0),
            snapshots: Some(Arc::clone(&storage.snapshot_registry)),
        });

        // Boot: RAM masking for every open job before the first read (Index start order 6).
        let open = inner.kernel.open_jobs();
        for job in &open {
            inner.mask_layers(&job.scope).await;
        }
        if !open.is_empty() {
            inner.notify.notify_one();
        }

        // Wake the worker whenever pending deletes remain (03 §8).
        let pending = storage.erasure_pending_delete_notify();
        {
            let notify = Arc::clone(&inner.notify);
            let cancel = cancel.clone();
            tracker.spawn(async move {
                loop {
                    tokio::select! {
                        _ = pending.notified() => notify.notify_one(),
                        _ = cancel.cancelled() => break,
                    }
                }
            });
        }

        let handle = {
            let rt = Arc::clone(&inner);
            let c = cancel;
            tracker.spawn(crate::background_workers::erasure_worker_loop(rt, c))
        };
        Self {
            inner,
            worker: parking_lot::Mutex::new(Some(handle)),
        }
    }

    /// Shared state (API, tests, layer registration).
    pub fn inner(&self) -> &Arc<ErasureRuntimeInner> {
        &self.inner
    }

    /// Stops accepting jobs and ends the worker. Progress is durable; unfinished jobs resume
    /// on the next start.
    pub async fn stop(&self) {
        self.inner.shutting_down.store(true, Ordering::Release);
        self.inner.cancel.cancel();
        self.inner.notify.notify_one();
        let handle = self.worker.lock().take();
        if let Some(h) = handle {
            let _ = h.await;
        }
    }
}
