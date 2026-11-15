//! Conformance suite for acta-env traits (Clock, Rng, IdGen, MetricsSink)

use acta_env::*;

// =========================================================================
// SECTION: Conformance Test Suite
// ANCHOR[id=conformance-tests,epic=env-core,seq=5]
// NOTE: Verifiziert alle Grundverträge der vier Environment-Kernports.
// LINK crates/acta-env/SPEC.md#4-bindender-vertrag-42-s
// =========================================================================

#[test]
fn test_clock_contract() {
    // NOTE: Verifiziert, dass SystemClock gültige Nanosekunden und konsistente ms liefert
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
    // NOTE: Verifiziert deterministische Bitgleichheit zweier Instanzen mit Seed 42
    let rng1 = SeededRng::new(42);
    let rng2 = SeededRng::new(42);
    for _ in 0..100 {
        assert_eq!(rng1.next_u64(), rng2.next_u64());
    }
}

#[test]
fn test_sequential_id_gen() {
    // NOTE: Verifiziert Monotonie des ID-Generators
    let gen = SequentialIdGen::new(100);
    assert_eq!(gen.next_id(), 100);
    assert_eq!(gen.next_id(), 101);
}

#[test]
fn test_metrics_sink_noop() {
    // NOTE: Verifiziert die Noop-Implementierung auf fehlerfreie Ausführung
    let sink = NoopMetricsSink;
    let _key = MetricKey::new("test_metric", &[]);
    sink.record_counter("test", 1, &[]);
    sink.record_gauge("test", 1.0, &[]);
    sink.record_histogram("test", 1.0, &[]);
}

// !SECTION: Conformance Test Suite
