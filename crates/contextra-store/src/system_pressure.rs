//! Observability Mandate (v17 Teil 11 & Teil 2.2):
//! SystemPressureMonitor tracks backpressure levels (Normal/Elevated/Critical) and emits gauge metric
//! `lsm_backpressure_level` via `MetricsSink` (0.0 = Normal, 1.0 = Elevated, 2.0 = Critical).
//! Measurement point scope:
//! - Commit-Latenz & Backpressure-Level: Covered by Prompt 13 (`contextra-store`)
//! - Suchlatenz je Signal: Prompt 14
//! - Rebuild-Dauer (HNSW): Prompt 15
//! - Checkpoint-Dauer & DLQ-Tiefe: Prompt 25
//!
//! INTEGRATION GUIDE:
//! 1. In LsmStorage::new(): SystemPressureMonitor::run() is spawned as a background task
//!    monitoring group-commit follower queue depth (`wal_queue_depth_fn`).
//! 2. In Collection::insert(): pressure_rx.borrow().pressure_level != Critical is checked;
//!    if Critical: tokio::time::sleep(backpressure_delay).await before insert.
//! 3. In contextra-embed/TextEmbedder: pressure_rx subscriber handles Critical state for embeddings.
//!
//! IMPLEMENTATION NOTES & LIMITATIONS:
//! - `wal_queue_depth`: Fully implemented in `LsmStorage` via AtomicUsize tracking pending
//!   group-commit followers awaiting disk write/notification.
//! - `scheduler_queue_depth`: Measures Tokio async scheduler global queue depth ratio
//!   (`metrics.global_queue_depth()`).
//! - `blocking_util`: Measures blocking thread pool utilization/saturation ratio (0.0–1.0).
//!   Note: Direct inspection of Tokio blocking pool internal thread counts/queue depth requires
//!   the `tokio_unstable` flag (unstable Tokio metrics API). To maintain standard stable toolchain
//!   compatibility without `cfg(tokio_unstable)`, `blocking_util` is measured dynamically using a
//!   lightweight probe task (`spawn_blocking`) with dispatch latency tracking and timeout probe,
//!   reflecting real execution queue contention and pool saturation.
//! - `embedding_queue_depth`: Defaults to `0/0` in `LsmStorage` because `contextra-store` is a pure
//!   KV engine decoupled from embedding crates (`contextra-embed` / `contextra-candle`), which manage
//!   their own semaphore permits.

use std::sync::Arc;
use std::time::Duration;
use tokio_util::sync::CancellationToken;

pub const WAL_QUEUE_CRITICAL_THRESHOLD: usize = 500;
pub const WAL_QUEUE_ELEVATED_THRESHOLD: usize = 100;
pub const SCHEDULER_QUEUE_CRITICAL: f32 = 0.85;
pub const SCHEDULER_QUEUE_ELEVATED: f32 = 0.60;
pub const BLOCKING_UTIL_CRITICAL: f32 = SCHEDULER_QUEUE_CRITICAL;
pub const BLOCKING_UTIL_ELEVATED: f32 = SCHEDULER_QUEUE_ELEVATED;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PressureLevel {
    Normal,
    Elevated, // Warnung: System unter Last
    Critical, // Backpressure aktiv: neue Inserts verlangsamen
}

#[derive(Debug, Clone, PartialEq)]
pub struct SystemPressure {
    pub wal_queue_depth: usize,
    pub scheduler_queue_depth: f32, // 0.0–1.0 (Tokio global scheduler queue depth ratio)
    pub blocking_util: f32,         // 0.0–1.0 (Blocking pool workload / utilization ratio)
    pub embedding_queue_depth: usize, // via Semaphore-Permits
    pub pressure_level: PressureLevel,
}

pub struct SystemPressureMonitor {
    pressure_tx: tokio::sync::watch::Sender<SystemPressure>,
    pub pressure_rx: tokio::sync::watch::Receiver<SystemPressure>,
    sampling_interval: Duration,
}

impl SystemPressureMonitor {
    pub fn new(sampling_interval: Duration) -> Self {
        let default_pressure = SystemPressure {
            wal_queue_depth: 0,
            scheduler_queue_depth: 0.0,
            blocking_util: 0.0,
            embedding_queue_depth: 0,
            pressure_level: PressureLevel::Normal,
        };
        let (pressure_tx, pressure_rx) = tokio::sync::watch::channel(default_pressure);
        Self {
            pressure_tx,
            pressure_rx,
            sampling_interval,
        }
    }

    pub fn compute_pressure(
        &self,
        wal_depth: usize,
        scheduler_queue_depth: f32,
        blocking_util: f32,
        embedding_queue: usize,
        max_permits: usize,
    ) -> SystemPressure {
        let level = if scheduler_queue_depth > SCHEDULER_QUEUE_CRITICAL
            || blocking_util > BLOCKING_UTIL_CRITICAL
            || wal_depth > WAL_QUEUE_CRITICAL_THRESHOLD
            || (max_permits > 0 && embedding_queue == 0)
        {
            PressureLevel::Critical
        } else if scheduler_queue_depth > SCHEDULER_QUEUE_ELEVATED
            || blocking_util > BLOCKING_UTIL_ELEVATED
            || wal_depth > WAL_QUEUE_ELEVATED_THRESHOLD
        {
            PressureLevel::Elevated
        } else {
            PressureLevel::Normal
        };

        SystemPressure {
            wal_queue_depth: wal_depth,
            scheduler_queue_depth,
            blocking_util,
            embedding_queue_depth: embedding_queue,
            pressure_level: level,
        }
    }

    pub async fn run(
        self,
        cancellation: CancellationToken,
        wal_queue_depth_fn: impl Fn() -> usize + Send + 'static,
        embedding_permits_fn: impl Fn() -> usize + Send + 'static,
        max_embedding_permits: usize,
    ) {
        self.run_with_metrics_sink(
            cancellation,
            wal_queue_depth_fn,
            embedding_permits_fn,
            max_embedding_permits,
            None,
        )
        .await
    }

    pub async fn run_with_metrics_sink(
        self,
        cancellation: CancellationToken,
        wal_queue_depth_fn: impl Fn() -> usize + Send + 'static,
        embedding_permits_fn: impl Fn() -> usize + Send + 'static,
        max_embedding_permits: usize,
        metrics_sink: Option<Arc<parking_lot::RwLock<Arc<dyn contextra_ports::MetricsSink>>>>,
    ) {
        let mut last_level = PressureLevel::Normal;

        // Emit initial gauge status for Normal pressure level (0.0)
        if let Some(ref sink_lock) = metrics_sink {
            sink_lock
                .read()
                .record_gauge("lsm_backpressure_level", 0.0, &[("level", "Normal")]);
        }

        loop {
            tokio::select! {
                _ = cancellation.cancelled() => break,
                _ = tokio::time::sleep(self.sampling_interval) => {
                    let metrics = tokio::runtime::Handle::current().metrics();
                    let active_tasks = metrics.num_workers() as f32;
                    let queue_depth = metrics.global_queue_depth() as f32;
                    let scheduler_queue_depth = if queue_depth > 0.0 {
                        (queue_depth / (active_tasks + queue_depth)).min(1.0)
                    } else {
                        0.0
                    };

                    // Sample blocking thread pool utilization using a probe task with timeout.
                    // Probe measures task execution/dispatch latency through the blocking queue.
                    let sample_start = std::time::Instant::now();
                    let blocking_probe = tokio::task::spawn_blocking(|| {
                        // Minimal work in blocking thread
                    });
                    let blocking_util = match tokio::time::timeout(Duration::from_millis(100), blocking_probe).await {
                        Ok(Ok(())) => {
                            let elapsed = sample_start.elapsed().as_secs_f32();
                            // Latency > 10ms indicates elevated blocking queue contention; > 50ms indicates critical.
                            (elapsed / 0.05).min(1.0)
                        }
                        _ => 1.0, // Timeout or join error indicates blocking pool saturation
                    };

                    let pressure = self.compute_pressure(
                        wal_queue_depth_fn(),
                        scheduler_queue_depth,
                        blocking_util,
                        embedding_permits_fn(),
                        max_embedding_permits,
                    );

                    if let Some(ref sink_lock) = metrics_sink {
                        let sink = sink_lock.read();
                        sink.record_gauge(
                            "lsm_blocking_pool_utilization",
                            pressure.blocking_util as f64,
                            &[],
                        );

                        if pressure.pressure_level != last_level {
                            let (gauge_val, level_str) = match pressure.pressure_level {
                                PressureLevel::Normal => (0.0, "Normal"),
                                PressureLevel::Elevated => (1.0, "Elevated"),
                                PressureLevel::Critical => (2.0, "Critical"),
                            };

                            sink.record_gauge(
                                "lsm_backpressure_level",
                                gauge_val,
                                &[("level", level_str)],
                            );
                        }
                    }

                    if pressure.pressure_level != last_level {
                        tracing::info!(
                            previous_level = ?last_level,
                            new_level = ?pressure.pressure_level,
                            wal_depth = pressure.wal_queue_depth,
                            scheduler_queue_depth = pressure.scheduler_queue_depth,
                            blocking_util = pressure.blocking_util,
                            embedding_queue = pressure.embedding_queue_depth,
                            "SystemPressure level transitioned"
                        );
                        last_level = pressure.pressure_level;
                    }

                    let _ = self.pressure_tx.send(pressure);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    #[test]
    fn test_compute_pressure_thresholds() {
        let monitor = SystemPressureMonitor::new(Duration::from_millis(100));

        // Normal state
        let normal = monitor.compute_pressure(50, 0.2, 0.1, 5, 10);
        assert_eq!(normal.pressure_level, PressureLevel::Normal);

        // Elevated states
        let elevated_wal = monitor.compute_pressure(150, 0.2, 0.1, 5, 10);
        assert_eq!(elevated_wal.pressure_level, PressureLevel::Elevated);

        let elevated_util = monitor.compute_pressure(50, 0.7, 0.1, 5, 10);
        assert_eq!(elevated_util.pressure_level, PressureLevel::Elevated);

        let elevated_blocking = monitor.compute_pressure(50, 0.2, 0.7, 5, 10);
        assert_eq!(elevated_blocking.pressure_level, PressureLevel::Elevated);

        // Critical states
        let critical_wal = monitor.compute_pressure(501, 0.2, 0.1, 5, 10);
        assert_eq!(critical_wal.pressure_level, PressureLevel::Critical);

        let critical_util = monitor.compute_pressure(50, 0.9, 0.1, 5, 10);
        assert_eq!(critical_util.pressure_level, PressureLevel::Critical);

        let critical_blocking = monitor.compute_pressure(50, 0.2, 0.9, 5, 10);
        assert_eq!(critical_blocking.pressure_level, PressureLevel::Critical);

        let critical_permits = monitor.compute_pressure(50, 0.2, 0.1, 0, 10);
        assert_eq!(critical_permits.pressure_level, PressureLevel::Critical);
    }

    #[tokio::test]
    async fn test_monitor_run_cancellation() {
        let monitor = SystemPressureMonitor::new(Duration::from_millis(50));
        let cancellation = CancellationToken::new();
        let cancel_token = cancellation.clone();

        let handle = tokio::spawn(async move {
            monitor.run(cancel_token, || 0, || 10, 10).await;
        });

        tokio::time::sleep(Duration::from_millis(20)).await;
        cancellation.cancel();

        let res = tokio::time::timeout(Duration::from_secs(1), handle).await;
        assert!(
            res.is_ok(),
            "Monitor run loop should terminate cleanly on cancellation"
        );
    }

    #[tokio::test]
    async fn test_watch_receiver_updates() {
        let monitor = SystemPressureMonitor::new(Duration::from_millis(50));
        let mut rx = monitor.pressure_rx.clone();
        let cancellation = CancellationToken::new();
        let cancel_token = cancellation.clone();

        let wal_depth = Arc::new(AtomicUsize::new(10));
        let wal_depth_clone = Arc::clone(&wal_depth);

        let handle = tokio::spawn(async move {
            monitor
                .run(
                    cancel_token,
                    move || wal_depth_clone.load(Ordering::Relaxed),
                    || 10,
                    10,
                )
                .await;
        });

        // Initial state
        assert_eq!(rx.borrow().wal_queue_depth, 0);

        // Update wal depth to critical
        wal_depth.store(600, Ordering::Relaxed);

        // Wait for change on watch receiver
        rx.changed().await.unwrap();
        let updated = rx.borrow().clone();
        assert_eq!(updated.wal_queue_depth, 600);
        assert_eq!(updated.pressure_level, PressureLevel::Critical);

        cancellation.cancel();
        let _ = handle.await;
    }

    #[tokio::test]
    async fn test_blocking_pool_load_triggers_pressure() {
        let monitor = SystemPressureMonitor::new(Duration::from_millis(20));
        let mut rx = monitor.pressure_rx.clone();
        let cancellation = CancellationToken::new();
        let cancel_token = cancellation.clone();

        let handle = tokio::spawn(async move {
            monitor.run(cancel_token, || 0, || 10, 10).await;
        });

        // Spawn blocking tasks to saturate default 512 max blocking pool threads
        let mut handles = Vec::new();
        for _ in 0..512 {
            handles.push(tokio::task::spawn_blocking(|| {
                std::thread::sleep(Duration::from_millis(150));
            }));
        }

        // Wait for SystemPressureMonitor to detect blocking queue delay/saturation
        let mut reacted = false;
        let start = std::time::Instant::now();
        while start.elapsed() < Duration::from_secs(2) {
            if rx.changed().await.is_ok() {
                let p = rx.borrow().clone();
                if p.blocking_util > BLOCKING_UTIL_ELEVATED
                    || p.pressure_level != PressureLevel::Normal
                {
                    reacted = true;
                    break;
                }
            }
        }

        assert!(
            reacted,
            "SystemPressureMonitor should react to simulated blocking pool saturation"
        );

        cancellation.cancel();
        let _ = handle.await;
        for h in handles {
            let _ = h.await;
        }
    }
}
