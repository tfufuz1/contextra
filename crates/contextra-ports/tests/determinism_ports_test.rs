//! Integration tests for determinism ports (Clock, Rng, IdGen / IdGenerator).

use contextra_ports::{
    Clock, IdGen, IdGenerator, ManualClock, Rng, SeededRng, SequentialIdGen, SequentialIds,
    SystemClock, SystemIdGen, SystemIdGenerator, SystemRng,
};
use std::sync::Arc;
use std::thread;

fn assert_send_sync<T: Send + Sync + 'static>() {}

#[test]
fn test_determinism_ports_trait_bounds_and_object_safety() {
    assert_send_sync::<SystemClock>();
    assert_send_sync::<ManualClock>();
    assert_send_sync::<SystemRng>();
    assert_send_sync::<SeededRng>();
    assert_send_sync::<SystemIdGen>();
    assert_send_sync::<SequentialIdGen>();

    // Test dyn compatibility
    let clock: Arc<dyn Clock> = Arc::new(ManualClock::new(1000));
    assert_eq!(clock.now_unix_nanos(), 1000);

    let rng: Arc<dyn Rng> = Arc::new(SeededRng::new(42));
    let val1 = rng.next_u64();
    assert!(val1 > 0);

    let id_gen: Arc<dyn IdGen> = Arc::new(SequentialIdGen::new(1));
    assert_eq!(id_gen.next_id(), 1);

    let id_generator: Arc<dyn IdGenerator> = Arc::new(SystemIdGenerator::new());
    assert!(id_generator.next_id() > 0);
}

#[test]
fn test_manual_clock_deterministic_advancement() {
    let clock = ManualClock::new(500);
    assert_eq!(clock.now_unix_nanos(), 500);
    assert_eq!(clock.monotonic_nanos(), 500);

    clock.advance(std::time::Duration::from_nanos(250));
    assert_eq!(clock.now_unix_nanos(), 750);

    clock.set_nanos(2000);
    assert_eq!(clock.monotonic_nanos(), 2000);
}

#[test]
fn test_seeded_rng_reproducibility_across_threads() {
    let seed = 9876543210u64;

    let handle1 = thread::spawn(move || {
        let rng = SeededRng::new(seed);
        (0..100).map(|_| rng.next_u64()).collect::<Vec<_>>()
    });

    let handle2 = thread::spawn(move || {
        let rng = SeededRng::new(seed);
        (0..100).map(|_| rng.next_u64()).collect::<Vec<_>>()
    });

    let seq1 = handle1.join().expect("thread 1 failed");
    let seq2 = handle2.join().expect("thread 2 failed");

    assert_eq!(
        seq1, seq2,
        "SeededRng generated different outputs for identical seeds"
    );
}

#[test]
fn test_sequential_id_gen_monotonic_and_wrapping() {
    let gen = SequentialIds::new(100);
    assert_eq!(gen.next_id(), 100);
    assert_eq!(gen.next_id(), 101);
    assert_eq!(gen.next_id(), 102);

    let max_gen = SequentialIdGen::new(u64::MAX);
    assert_eq!(max_gen.next_id(), u64::MAX);
    assert_eq!(max_gen.next_id(), 0); // Wraps safely
}

#[test]
fn test_production_ports_basic_functionality() {
    let sys_clock = SystemClock::new();
    let sys_rng = SystemRng::new();
    let sys_id = SystemIdGen::new();

    let t1 = sys_clock.now_unix_nanos();
    let m1 = sys_clock.monotonic_nanos();
    assert!(t1 > 0);

    let r1 = sys_rng.next_u64();
    let f1 = sys_rng.next_unit_f64();
    assert!((0.0..1.0).contains(&f1));
    assert!(r1 > 0 || r1 == 0);

    let id1 = sys_id.next_id();
    let id2 = sys_id.next_id();
    assert_eq!(id2, id1 + 1);

    assert!(sys_clock.monotonic_nanos() >= m1);
}
