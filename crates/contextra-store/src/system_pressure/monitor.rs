//! # System Pressure Monitor (Ring 1)
//!
//! Hintergrund-Monitor für die periodische Druckberechnung und Verteilung über Watch-Channels.

use super::calculation::{compute_pressure, SystemPressure};
use super::signal::PressureLevel;
use std::sync::Arc;
use std::time::Duration;
use tokio_util::sync::CancellationToken;

/// Periodischer Monitor zur Erfassung von Systemdruck-Signalen.
pub struct SystemPressureMonitor {
    pressure_tx: tokio::sync::watch::Sender<SystemPressure>,
    /// Empfänger-Kanal für Druckaktualisierungen.
    pub pressure_rx: tokio::sync::watch::Receiver<SystemPressure>,
    sampling_interval: Duration,
    pub(crate) clock: Arc<dyn contextra_env::Clock>,
}

impl SystemPressureMonitor {
    /// Erstellt einen neuen Monitor mit SystemClock.
    pub fn new(sampling_interval: Duration) -> Self {
        Self::new_with_clock(
            sampling_interval,
            Arc::new(contextra_env::SystemClock::new()),
        )
    }

    /// Erstellt einen neuen Monitor mit einer injizierten Clock.
    pub fn new_with_clock(
        sampling_interval: Duration,
        clock: Arc<dyn contextra_env::Clock>,
    ) -> Self {
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
            clock,
        }
    }

    /// Ersetzt die Clock durch eine übergebene Instanz.
    pub fn with_clock(mut self, clock: Arc<dyn contextra_env::Clock>) -> Self {
        self.clock = clock;
        self
    }

    /// Berechnet den aktuellen Druckzustand.
    pub fn compute_pressure(
        &self,
        wal_depth: usize,
        scheduler_queue_depth: f32,
        blocking_util: f32,
        embedding_queue: usize,
        max_permits: usize,
    ) -> SystemPressure {
        compute_pressure(
            wal_depth,
            scheduler_queue_depth,
            blocking_util,
            embedding_queue,
            max_permits,
        )
    }

    /// Startet die periodische Hintergrundüberwachung.
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

    /// Startet die periodische Hintergrundüberwachung mit optionalem MetricsSink.
    pub async fn run_with_metrics_sink(
        self,
        cancellation: CancellationToken,
        wal_queue_depth_fn: impl Fn() -> usize + Send + 'static,
        embedding_permits_fn: impl Fn() -> usize + Send + 'static,
        max_embedding_permits: usize,
        metrics_sink: Option<Arc<parking_lot::RwLock<Arc<dyn contextra_env::MetricsSink>>>>,
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
                    let sample_start_nanos = self.clock.monotonic_nanos();
                    let blocking_probe = tokio::task::spawn_blocking(|| {
                        // Minimal work in blocking thread
                    });
                    let blocking_util = match tokio::time::timeout(Duration::from_millis(100), blocking_probe).await {
                        Ok(Ok(())) => {
                            let elapsed_nanos = self.clock.monotonic_nanos().saturating_sub(sample_start_nanos);
                            let elapsed_secs = (elapsed_nanos as f32) / 1_000_000_000.0;
                            // Latency > 10ms indicates elevated blocking queue contention; > 50ms indicates critical.
                            (elapsed_secs / 0.05).min(1.0)
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
