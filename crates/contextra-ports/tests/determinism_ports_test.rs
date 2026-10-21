//! Integration tests for determinism ports (Clock, EntropyRng, SeededRng, IdGen / IdGenerator).

use contextra_ports::{Clock, DeterministicTimestamp, EntropyRng, IdGen, IdGenerator, Rng, SeededRng};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

struct MockTestClock {
    wall_ms: AtomicU64,
    mono_ticks: AtomicU64,
}

impl Clock for MockTestClock {
    fn now(&self) -> DeterministicTimestamp {
        DeterministicTimestamp::new(
            self.wall_ms.load(Ordering::Relaxed),
            self.mono_ticks.fetch_add(1, Ordering::Relaxed),
        )
    }

    fn monotonic_ticks(&self) -> u64 {
        self.mono_ticks.load(Ordering::Relaxed)
    }
}

struct MockEntropyRng;
impl EntropyRng for MockEntropyRng {
    fn fill_bytes(&self, dest: &mut [u8]) {
        dest.fill(0xAA);
    }
    fn next_u64(&self) -> u64 {
        0x1234_5678_9ABC_DEF0
    }
}

struct MockSeededRng {
    state: AtomicU64,
}

impl SeededRng for MockSeededRng {
    fn next_u64(&self) -> u64 {
        self.state.fetch_add(1, Ordering::Relaxed)
    }

    fn fill_bytes(&self, dest: &mut [u8]) {
        for b in dest.iter_mut() {
            *b = self.state.fetch_add(1, Ordering::Relaxed) as u8;
        }
    }

    fn next_unit_f64(&self) -> f64 {
        0.42
    }
}

struct MockIdGen {
    counter: AtomicU64,
}

impl IdGenerator for MockIdGen {
    fn next_id(&self) -> u64 {
        self.counter.fetch_add(1, Ordering::SeqCst)
    }
}

fn assert_send_sync<T: Send + Sync + 'static>() {}

#[test]
fn test_determinism_ports_trait_bounds_and_object_safety() {
    assert_send_sync::<MockTestClock>();
    assert_send_sync::<MockEntropyRng>();
    assert_send_sync::<MockSeededRng>();
    assert_send_sync::<MockIdGen>();

    // Test dyn compatibility for Clock
    let clock: Arc<dyn Clock> = Arc::new(MockTestClock {
        wall_ms: AtomicU64::new(1700000000000),
        mono_ticks: AtomicU64::new(100),
    });
    let ts = clock.now();
    assert_eq!(ts.wall_ms, 1700000000000);
    assert_eq!(ts.mono_ctr, 100);
    assert_eq!(clock.now_unix_nanos(), 1700000000000000000);
    assert!(clock.monotonic_ticks() >= 100);

    // Test dyn compatibility for EntropyRng
    let entropy_rng: Arc<dyn EntropyRng> = Arc::new(MockEntropyRng);
    let mut key = [0u8; 16];
    entropy_rng.fill_bytes(&mut key);
    assert_eq!(key, [0xAA; 16]);

    // Test dyn compatibility for SeededRng / Rng
    let seeded_rng: Arc<dyn Rng> = Arc::new(MockSeededRng {
        state: AtomicU64::new(10),
    });
    assert_eq!(seeded_rng.next_u64(), 10);
    assert_eq!(seeded_rng.next_unit_f64(), 0.42);

    // Test dyn compatibility for IdGen / IdGenerator
    let id_gen: Arc<dyn IdGen> = Arc::new(MockIdGen {
        counter: AtomicU64::new(1),
    });
    assert_eq!(id_gen.next_id(), 1);
    assert_eq!(id_gen.next_id(), 2);
}

#[test]
fn test_clock_duration_measurement_by_ticks_difference() {
    let clock = MockTestClock {
        wall_ms: AtomicU64::new(5000),
        mono_ticks: AtomicU64::new(1000),
    };

    let start_ticks = clock.monotonic_ticks();
    clock.mono_ticks.store(1250, Ordering::Relaxed);
    let end_ticks = clock.monotonic_ticks();

    let duration_ticks = end_ticks.saturating_sub(start_ticks);
    assert_eq!(duration_ticks, 250);
}

#[test]
fn test_rng_type_separation_key_paths() {
    fn crypto_key_gen<R: EntropyRng>(rng: &R) -> [u8; 8] {
        let mut key = [0u8; 8];
        rng.fill_bytes(&mut key);
        key
    }

    let entropy_rng = MockEntropyRng;
    let key = crypto_key_gen(&entropy_rng);
    assert_eq!(key, [0xAA; 8]);
    // MockSeededRng cannot be passed here because SeededRng does not implement EntropyRng.
}
