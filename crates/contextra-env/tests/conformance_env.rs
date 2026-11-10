//! Conformance suite for contextra-env traits (Clock, Rng, IdGen, MetricsSink)

use contextra_env::*;

#[test]
fn test_clock_contract() {
    let clock = SystemClock::default();
    let t1 = clock.now_unix_nanos();
    assert!(t1 > 0);
    let m1 = clock.monotonic_nanos();
    let m2 = clock.monotonic_nanos();
    assert!(m2 >= m1);
    assert_eq!(unix_ms(&clock), t1 / 1_000_000);
}

#[test]
fn test_seeded_rng_determinism() {
    let rng1 = SeededRng::new(42);
    let rng2 = SeededRng::new(42);
    for _ in 0..100 {
        assert_eq!(rng1.next_u64(), rng2.next_u64());
    }
}

#[test]
fn test_sequential_id_gen() {
    let gen = SequentialIdGen::new(100);
    assert_eq!(gen.next_id(), 100);
    assert_eq!(gen.next_id(), 101);
}

#[test]
fn test_metrics_sink_noop() {
    let sink = NoopMetricsSink;
    let _key = MetricKey::new("test_metric", &[]);
    sink.record_counter("test", 1, &[]);
    sink.record_gauge("test", 1.0, &[]);
    sink.record_histogram("test", 1.0, &[]);
}
