#![allow(
    clippy::panic,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::collapsible_if,
    clippy::unnecessary_map_or
)]

//! # Tests für Runtime, Governor und Metrics
//!
//! SPEC §4, §17.1-17.5, §18 (INV-API-1).

use contextra_storage::env::clock::RealClock;
use contextra_storage::env::governor::{FileQuota, Governor};
use contextra_storage::env::metrics::MetricsAdapter;
use contextra_storage::env::runtime::{Lane, Runtime};
use contextra_storage_ports::{Metrics, Pressure, PressureSource};
use std::sync::atomic::{AtomicI64, AtomicU64, Ordering};
use std::sync::Arc;

struct TestMetricsSink {
    counter_calls: AtomicU64,
    last_gauge: AtomicI64,
}

impl Metrics for TestMetricsSink {
    fn counter(&self, _n: &'static str, v: u64) {
        self.counter_calls.fetch_add(v, Ordering::SeqCst);
    }

    fn gauge(&self, _n: &'static str, v: i64) {
        self.last_gauge.store(v, Ordering::SeqCst);
    }

    fn observe(&self, _n: &'static str, _v: f64) {}
}

#[test]
fn test_metrics_adapter_inv_api_1() {
    // // INV-API-1: Metrikmethoden nehmen nur &'static str und Zahlen
    let sink = Arc::new(TestMetricsSink {
        counter_calls: AtomicU64::new(0),
        last_gauge: AtomicI64::new(0),
    });
    let adapter = MetricsAdapter::new(sink.clone());

    adapter.inc("redo.flush.count");
    adapter.count("blob.bytes.written", 4096);
    adapter.gauge("memtable.active.bytes", 1024 * 1024);

    assert_eq!(sink.counter_calls.load(Ordering::SeqCst), 4097);
    assert_eq!(sink.last_gauge.load(Ordering::SeqCst), 1024 * 1024);

    // No-Op Adapter stürzt nicht ab
    let noop = MetricsAdapter::noop();
    noop.inc("test");
    noop.gauge("test", 10);
    noop.observe("test", 1.5);
}

#[test]
fn test_runtime_manual_fifo_execution() {
    let rt = Runtime::manual();
    assert!(rt.is_manual());

    let order = Arc::new(std::sync::Mutex::new(Vec::new()));

    for i in 1..=5 {
        let order_clone = Arc::clone(&order);
        rt.spawn_fn(move || {
            let mut v = order_clone.lock().unwrap();
            v.push(i);
        });
    }

    assert_eq!(rt.pending_count(), 5);
    let ran = rt.run_pending();
    assert_eq!(ran, 5);
    assert_eq!(rt.pending_count(), 0);

    let recorded = order.lock().unwrap();
    assert_eq!(&recorded[..], &[1, 2, 3, 4, 5]);
}

#[test]
fn test_runtime_spawn_lane_max_one_job_per_instance() {
    let rt = Runtime::manual();
    let instance_1 = 42;
    let instance_2 = 43;

    // Urgent Lane für Instanz 1 belegen
    let res1 = rt.spawn_lane(instance_1, Lane::Urgent, Box::new(|| {}));
    assert!(res1.is_ok());
    assert!(rt.is_lane_busy(instance_1, Lane::Urgent));

    // Zweiter Job auf derselben Lane für dieselbe Instanz wird abgelehnt
    let res2 = rt.spawn_lane(instance_1, Lane::Urgent, Box::new(|| {}));
    assert!(res2.is_err());

    // Background Lane für dieselbe Instanz ist frei
    assert!(!rt.is_lane_busy(instance_1, Lane::Background));
    let res3 = rt.spawn_lane(instance_1, Lane::Background, Box::new(|| {}));
    assert!(res3.is_ok());

    // Andere Instanz kann Urgent Lane belegen
    let res4 = rt.spawn_lane(instance_2, Lane::Urgent, Box::new(|| {}));
    assert!(res4.is_ok());

    // Nach Ausführung werden die Slots wieder frei
    rt.run_pending();
    assert!(!rt.is_lane_busy(instance_1, Lane::Urgent));
    assert!(!rt.is_lane_busy(instance_1, Lane::Background));
    assert!(!rt.is_lane_busy(instance_2, Lane::Urgent));

    // Jetzt kann Instanz 1 wieder auf Urgent planen
    let res5 = rt.spawn_lane(instance_1, Lane::Urgent, Box::new(|| {}));
    assert!(res5.is_ok());
}

struct MockPressure(std::sync::atomic::AtomicU8);

impl PressureSource for MockPressure {
    fn level(&self) -> Pressure {
        match self.0.load(Ordering::SeqCst) {
            1 => Pressure::Soft,
            2 => Pressure::Hard,
            _ => Pressure::Normal,
        }
    }
}

#[test]
fn test_governor_quotas_and_pressure() {
    let clock = Arc::new(RealClock::new());
    let pressure = Arc::new(MockPressure(std::sync::atomic::AtomicU8::new(0)));
    let gov = Arc::new(Governor::new(
        clock,
        Some(pressure.clone()),
        10,  // max 10 open files
        100, // max 100 cache blocks
        1000,
        1000,
    ));

    gov.register_instance(1, 100, 2, 20);

    let quota = gov.file_quota(1);
    assert!(quota.acquire_file_fd());
    assert!(quota.acquire_file_fd());

    quota.release_file_fd();

    // BlockCache-Prüfung
    assert!(gov.can_admit_block(1, 10, 1));
    assert!(!gov.should_evict_block(1, 10));

    // Simuliere Soft-Pressure
    pressure.0.store(1, Ordering::SeqCst);
    assert_eq!(gov.pressure_level(), Pressure::Soft);
    assert!(gov.should_evict_block(1, 25)); // über min_blocks -> evict unter Druck
}

#[test]
fn test_governor_token_bucket() {
    let clock = Arc::new(RealClock::new());
    let gov = Governor::new(
        clock,
        None,
        512,
        1024,
        1000, // 1000 B/s
        1000,
    );

    // Initial ist das Burst-Limit verfügbar
    assert!(gov.throttle_background_read(500));
    assert!(gov.throttle_background_read(500));
    // Nun aufgebraucht
    assert!(!gov.throttle_background_read(500));
}
