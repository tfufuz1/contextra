//! Integration tests for `PprCostCalibrator::recommended_multiplier` and `log_tl_hfd_vs_baseline_discrepancy`.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use ahash::AHashMap;
use contextra_adapt::{ShadowDiscrepancy, ShadowSink};
use contextra_graph::ppr::cost::{PprCostCalibrator, PprCostSample};
use contextra_graph::ppr::shadow_hook::log_tl_hfd_vs_baseline_discrepancy;
use contextra_types::{EntityId, PprAlgorithm};
use std::sync::{Arc, Mutex};

#[derive(Default)]
struct TestShadowSink {
    recorded: Arc<Mutex<Vec<ShadowDiscrepancy<f64>>>>,
}

impl ShadowSink<f64> for TestShadowSink {
    fn record(&self, discrepancy: ShadowDiscrepancy<f64>) {
        if let Ok(mut lock) = self.recorded.lock() {
            lock.push(discrepancy);
        }
    }
}

#[test]
fn test_ppr_cost_calibrator_recommended_multiplier_integration() {
    let calibrator = PprCostCalibrator::new();
    assert_eq!(calibrator.recommended_multiplier(), None);

    calibrator.record(PprCostSample {
        algorithm: PprAlgorithm::ForwardPush,
        estimated_cost: 100.0,
        measured_edge_accesses: 120,
    });
    calibrator.record(PprCostSample {
        algorithm: PprAlgorithm::DensePowerIteration,
        estimated_cost: 200.0,
        measured_edge_accesses: 160,
    });

    let mult = calibrator
        .recommended_multiplier()
        .expect("multiplier calculated");
    // (1.2 + 0.8) / 2 = 1.0
    assert!((mult - 1.0).abs() < 1e-6);
}

#[test]
fn test_log_tl_hfd_vs_baseline_discrepancy_integration() {
    let sink = TestShadowSink::default();

    let mut baseline = AHashMap::default();
    baseline.insert(EntityId::new(1), 0.5f32);
    baseline.insert(EntityId::new(2), 0.5f32);

    let mut candidate = AHashMap::default();
    candidate.insert(EntityId::new(1), 0.4f32);
    candidate.insert(EntityId::new(2), 0.6f32);

    log_tl_hfd_vs_baseline_discrepancy(&baseline, &candidate, 101, &sink);

    let records = sink.recorded.lock().unwrap();
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].context_id, 101);
    assert!((records[0].candidate - 0.1).abs() < 1e-6);
}
