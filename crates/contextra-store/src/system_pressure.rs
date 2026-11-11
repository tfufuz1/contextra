//! # System Pressure Root (Ring 1)
//!
//! Observability Mandate (v17 Teil 11 & Teil 2.2):
//! SystemPressureMonitor tracks backpressure levels (Normal/Elevated/Critical) and emits gauge metric
//! `lsm_backpressure_level` via `MetricsSink` (0.0 = Normal, 1.0 = Elevated, 2.0 = Critical).

pub mod calculation;
pub mod monitor;
pub mod signal;

pub use calculation::*;
pub use monitor::*;
pub use signal::*;

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;
    use std::time::Duration;
    use tokio_util::sync::CancellationToken;

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
        let _clock = monitor.clock.clone();
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
        let start_nanos = _clock.monotonic_nanos();
        while _clock.monotonic_nanos().saturating_sub(start_nanos) < 2_000_000_000 {
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

    struct DummyPressureSignal;
    impl SystemPressureSignal for DummyPressureSignal {
        fn current_pressure(&self) -> PressureLevel {
            PressureLevel::Normal
        }
        fn advise_reduction_ratio(&self) -> f32 {
            0.0
        }
    }

    #[test]
    fn test_system_pressure_signal_contract() {
        let sig = DummyPressureSignal;
        assert_eq!(sig.current_pressure(), PressureLevel::Normal);
        assert_eq!(sig.advise_reduction_ratio(), 0.0);
    }
}
