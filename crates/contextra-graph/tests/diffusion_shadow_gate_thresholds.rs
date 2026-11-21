//! Integration tests for diffusion shadow mode gate thresholds (TL-HFD & APPRH).

#![cfg(feature = "apprh-diffusion")]
#![allow(clippy::expect_used, clippy::unwrap_used)]

use contextra_graph::apprh::gate_monitor::{ApprhGateMonitor, ApprhGateMonitorConfig};
use contextra_graph::apprh::shadow::ApprhShadowComparison;
use contextra_graph::tl_hfd::shadow::{
    DefaultFlipGate, ShadowDiscrepancyReport, TlHfdFlipGate, MIN_AGREEMENT_THRESHOLD,
    MIN_SHADOW_SAMPLES,
};
use contextra_types::EntityId;

#[test]
fn test_tl_hfd_gate_threshold_boundaries() {
    let gate = TlHfdFlipGate;

    // 1. Gate remains closed at 9,999 samples (< 10,000) even with perfect agreement and lower latency
    let report_9999 = ShadowDiscrepancyReport {
        sample_count: MIN_SHADOW_SAMPLES - 1,
        mean_topk_jaccard: 1.0,
        p99_latency_delta_us: -100.0,
        recall_improvement_ratio: None,
    };
    assert!(!gate.should_flip(&report_9999));

    // 2. Gate remains closed at Jaccard 0.849 (< 0.85) even with 10,000 samples and lower latency
    let report_jaccard_low = ShadowDiscrepancyReport {
        sample_count: MIN_SHADOW_SAMPLES,
        mean_topk_jaccard: MIN_AGREEMENT_THRESHOLD - 0.001,
        p99_latency_delta_us: -100.0,
        recall_improvement_ratio: None,
    };
    assert!(!gate.should_flip(&report_jaccard_low));

    // 3. Gate remains closed if latency delta is positive (> 0.0 us)
    let report_latency_high = ShadowDiscrepancyReport {
        sample_count: MIN_SHADOW_SAMPLES,
        mean_topk_jaccard: MIN_AGREEMENT_THRESHOLD,
        p99_latency_delta_us: 0.1,
        recall_improvement_ratio: None,
    };
    assert!(!gate.should_flip(&report_latency_high));

    // 4. Gate opens exactly at boundary values: 10,000 samples, Jaccard 0.85, latency delta <= 0.0
    let report_exact_boundary = ShadowDiscrepancyReport {
        sample_count: MIN_SHADOW_SAMPLES,
        mean_topk_jaccard: MIN_AGREEMENT_THRESHOLD,
        p99_latency_delta_us: 0.0,
        recall_improvement_ratio: None,
    };
    assert!(gate.should_flip(&report_exact_boundary));
}

#[test]
fn test_apprh_gate_monitor_consecutive_passes_boundary() {
    let config = ApprhGateMonitorConfig {
        window_size: 20,
        required_consecutive_passes: 10,
    };
    let monitor = ApprhGateMonitor::new(config).expect("Valid config");

    let clean_pass = ApprhShadowComparison {
        max_abs_diff: 0.01,
        top_k_overlap: 0.90,
        discrepancy: false,
        forward_push_results: vec![(EntityId(1), 1.0)],
        apprh_results: vec![(EntityId(1), 1.0)],
    };

    let outlier = ApprhShadowComparison {
        max_abs_diff: 0.5,
        top_k_overlap: 0.50,
        discrepancy: true,
        forward_push_results: vec![(EntityId(1), 1.0)],
        apprh_results: vec![(EntityId(1), 1.0)],
    };

    // Observe 9 clean passes
    for _ in 0..9 {
        monitor.observe(&clean_pass);
    }
    // Gate remains closed at 9 passes
    assert!(!monitor.production_ready());

    // Observe 1 outlier -> streak resets to 0
    monitor.observe(&outlier);
    assert!(!monitor.production_ready());

    // Observe 9 clean passes again -> still not ready
    for _ in 0..9 {
        monitor.observe(&clean_pass);
    }
    assert!(!monitor.production_ready());

    // Observe 10th clean pass -> production_ready becomes true
    monitor.observe(&clean_pass);
    assert!(monitor.production_ready());
}
