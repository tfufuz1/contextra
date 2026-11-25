// ZWECK: Testet, dass Observer-Benachrichtigungen Tokio Runtime-Threads in current_thread-Runtimes nicht blockieren.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use contextra_core::TxId;
use contextra_store::{CommittedBatch, ObserverRegistry, WalObserver, WriteOrigin};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

struct SlowObserver {
    sleep_duration: Duration,
    count: AtomicUsize,
}

impl SlowObserver {
    fn new(sleep_duration: Duration) -> Self {
        Self {
            sleep_duration,
            count: AtomicUsize::new(0),
        }
    }
}

impl WalObserver for SlowObserver {
    fn on_commit(&self, _batch: &CommittedBatch<'_>, _seq_no: u64, _tx_id: TxId) {
        self.count.fetch_add(1, Ordering::SeqCst);
        std::thread::sleep(self.sleep_duration);
    }
}

#[tokio::test(flavor = "current_thread")]
async fn test_observer_notify_does_not_block_tokio_runtime_thread() {
    let registry = ObserverRegistry::new();
    registry.set_max_observer_latency(Duration::from_millis(2));

    let slow_obs = Arc::new(SlowObserver::new(Duration::from_millis(50)));
    registry.register_observer(slow_obs.clone());

    // Spawn a parallel timer task on the same current_thread Tokio runtime
    let timer_handle = tokio::spawn(async move {
        let mut max_delay = Duration::ZERO;
        let iterations = 10;
        for _ in 0..iterations {
            let start = Instant::now();
            tokio::time::sleep(Duration::from_millis(1)).await;
            let elapsed = start.elapsed();
            let delay = elapsed.saturating_sub(Duration::from_millis(1));
            if delay > max_delay {
                max_delay = delay;
            }
        }
        max_delay
    });

    // Yield to let the timer task start its first sleep cycle
    tokio::task::yield_now().await;

    let obs_trait: Arc<dyn WalObserver> = slow_obs.clone();

    // Clear circuit breaker if open so notify() will invoke observer
    registry.clear_circuit_breaker(&obs_trait);

    // Notify observers while the timer task is sleeping
    for i in 1..=5u64 {
        registry.clear_circuit_breaker(&obs_trait);
        registry.notify(&[], i, TxId::new(i), WriteOrigin::UserWrite);
        tokio::time::sleep(Duration::from_millis(1)).await;
    }

    let max_timer_delay = timer_handle.await.expect("timer handle");

    // If notify() blocked the Tokio runtime thread during the slow observer execution (50ms),
    // the timer task would experience at least 50ms of delay.
    // With non-blocking notification, max_timer_delay must remain under 15ms.
    assert!(
        max_timer_delay < Duration::from_millis(15),
        "Timer task experienced max delay of {:?}, expected < 15ms. Observer notification blocked Tokio runtime thread!",
        max_timer_delay
    );
}
