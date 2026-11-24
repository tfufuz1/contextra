// FILE-CONTEXT
// ZWECK: Testen des Rebuild-Backoff- und Alarm-Automaten ohne Wanduhr (P28, Monotonic Counter Clock).
// INVARIANTEN: Zeitabfragen ausschließlich über Clock-Abstraktion, saturierende Arithmetik, deterministische Übergänge.
// STAND: TS:2026-11-11T02:00:00Z

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

use contextra_vector::hnsw::{HnswConfig, HnswIndex};
use contextra_vector::types::{Clock, SeededRng};

struct MonotonicCounterClock {
    nanos: AtomicU64,
}

impl MonotonicCounterClock {
    fn new(start_nanos: u64) -> Self {
        Self {
            nanos: AtomicU64::new(start_nanos),
        }
    }

    fn advance(&self, duration: Duration) {
        let delta = duration.as_nanos().min(u64::MAX as u128) as u64;
        self.nanos.fetch_add(delta, Ordering::SeqCst);
    }
}

impl Clock for MonotonicCounterClock {
    fn now_unix_nanos(&self) -> u64 {
        self.nanos.load(Ordering::SeqCst)
    }

    fn monotonic_nanos(&self) -> u64 {
        self.nanos.load(Ordering::SeqCst)
    }
}

#[test]
fn test_standalone_backoff_tracker_via_index() {
    let clock = Arc::new(MonotonicCounterClock::new(1_000_000_000));
    let mut config = HnswConfig::default();
    config.dimension = 4;
    config.backoff_base_delay = Duration::from_millis(100);
    config.backoff_max_delay = Duration::from_millis(800);
    config.backoff_alert_threshold = 4;

    let rng = Arc::new(SeededRng::new(42));
    let index =
        HnswIndex::try_new_with_rng_and_clock(config, rng, clock).expect("index creation succeeds");

    let mut tracker = index.create_backoff_tracker();
    assert_eq!(tracker.current_cooldown(), Duration::from_millis(100));
    assert!(tracker.check_and_update(1_000_000_000).is_ok());
    tracker.record_success(1_000_000_000);
    assert_eq!(tracker.consecutive_rapid_rebuilds(), 0);

    // Rapid attempt fails
    assert!(tracker.check_and_update(1_050_000_000).is_err());
    assert_eq!(tracker.consecutive_rapid_rebuilds(), 1);

    // After backoff cooldown succeeds
    assert!(tracker.check_and_update(1_250_000_000).is_ok());
    tracker.record_success(1_250_000_000);
    assert_eq!(tracker.consecutive_rapid_rebuilds(), 0);
}

#[test]
fn test_backoff_transitions_and_exponential_extension() {
    let clock = Arc::new(MonotonicCounterClock::new(1_000_000_000));
    let mut config = HnswConfig::default();
    config.dimension = 4;
    config.backoff_base_delay = Duration::from_millis(100);
    config.backoff_max_delay = Duration::from_millis(800);
    config.backoff_alert_threshold = 4;

    let rng = Arc::new(SeededRng::new(42));
    let index = HnswIndex::try_new_with_rng_and_clock(config, rng, clock.clone())
        .expect("index creation succeeds");

    // Initial State: allowed
    let res0 = index.rebuild_sync_with_clock(clock.as_ref());
    assert!(res0.is_ok());

    // Attempt 1: 50ms later (< 100ms) -> suppressed, consecutive = 1
    clock.advance(Duration::from_millis(50));
    let err1 = index.rebuild_sync_with_clock(clock.as_ref()).unwrap_err();
    assert!(err1.to_string().contains("rate limited"));

    // Attempt 2: 10ms later (< 100ms) -> suppressed, consecutive = 2
    clock.advance(Duration::from_millis(10));
    let err2 = index.rebuild_sync_with_clock(clock.as_ref()).unwrap_err();
    assert!(err2.to_string().contains("rate limited"));

    // Attempt 3: 50ms later (< 200ms) -> suppressed, consecutive = 3
    clock.advance(Duration::from_millis(50));
    let err3 = index.rebuild_sync_with_clock(clock.as_ref()).unwrap_err();
    assert!(err3.to_string().contains("rate limited"));

    // Attempt 4: reaches alert_threshold = 4 -> suppressed with alert
    clock.advance(Duration::from_millis(50));
    let err4 = index.rebuild_sync_with_clock(clock.as_ref()).unwrap_err();
    assert!(err4.to_string().contains("rate limited"));

    // Advance clock past cooldown: 850ms later (> 800ms) -> rebuild succeeds!
    clock.advance(Duration::from_millis(850));
    let res_ok = index.rebuild_sync_with_clock(clock.as_ref());
    assert!(res_ok.is_ok());
}

#[test]
fn test_backoff_cooldown_reset_after_idle_window() {
    let clock = Arc::new(MonotonicCounterClock::new(1_000_000_000));
    let mut config = HnswConfig::default();
    config.dimension = 4;
    config.backoff_base_delay = Duration::from_millis(100);
    config.backoff_max_delay = Duration::from_millis(500);

    let rng = Arc::new(SeededRng::new(42));
    let index = HnswIndex::try_new_with_rng_and_clock(config, rng, clock.clone())
        .expect("index creation succeeds");

    assert!(index.rebuild_sync_with_clock(clock.as_ref()).is_ok());

    // Trigger one suppressed attempt to increment consecutive
    clock.advance(Duration::from_millis(50));
    assert!(index.rebuild_sync_with_clock(clock.as_ref()).is_err());

    // Advance time past 2 * max_delay (1100ms > 1000ms)
    clock.advance(Duration::from_millis(1100));
    let res = index.rebuild_sync_with_clock(clock.as_ref());
    assert!(res.is_ok());
}

#[test]
fn test_backoff_saturating_arithmetic_overflow_safety() {
    let clock = Arc::new(MonotonicCounterClock::new(u64::MAX - 1000));
    let mut config = HnswConfig::default();
    config.dimension = 4;
    config.backoff_base_delay = Duration::from_secs(10);
    config.backoff_max_delay = Duration::from_secs(3600);

    let rng = Arc::new(SeededRng::new(42));
    let index = HnswIndex::try_new_with_rng_and_clock(config, rng, clock.clone())
        .expect("index creation succeeds");

    for _ in 0..50 {
        let _ = index.rebuild_sync_with_clock(clock.as_ref());
    }
}
