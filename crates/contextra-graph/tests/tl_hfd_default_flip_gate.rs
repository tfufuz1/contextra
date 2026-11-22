use contextra_graph::{
    DefaultFlipGate, ShadowDiscrepancyReport, TlHfdFlipGate, MIN_AGREEMENT_THRESHOLD,
    MIN_SHADOW_SAMPLES,
};

#[test]
fn test_default_flip_gate_threshold_constants() {
    assert_eq!(MIN_SHADOW_SAMPLES, 10_000);
    assert_eq!(MIN_AGREEMENT_THRESHOLD, 0.85);
}

struct TestCase {
    name: &'static str,
    sample_count: u64,
    mean_topk_jaccard: f32,
    p99_latency_delta_us: f64,
    recall_improvement_ratio: Option<f32>,
    expected_flip: bool,
}

#[test]
fn test_tl_hfd_default_flip_gate_table() {
    let gate = TlHfdFlipGate;

    let cases = vec![
        // Baseline & exact boundaries
        TestCase {
            name: "exact_thresholds_should_flip",
            sample_count: 10_000,
            mean_topk_jaccard: 0.85,
            p99_latency_delta_us: 0.0,
            recall_improvement_ratio: None,
            expected_flip: true,
        },
        TestCase {
            name: "above_all_thresholds_should_flip",
            sample_count: 50_000,
            mean_topk_jaccard: 0.95,
            p99_latency_delta_us: -150.0,
            recall_improvement_ratio: Some(0.12),
            expected_flip: true,
        },
        // sample_count boundaries
        TestCase {
            name: "sample_count_just_below_fails",
            sample_count: 9_999,
            mean_topk_jaccard: 0.85,
            p99_latency_delta_us: 0.0,
            recall_improvement_ratio: None,
            expected_flip: false,
        },
        TestCase {
            name: "sample_count_just_above_passes",
            sample_count: 10_001,
            mean_topk_jaccard: 0.85,
            p99_latency_delta_us: 0.0,
            recall_improvement_ratio: None,
            expected_flip: true,
        },
        // mean_topk_jaccard boundaries
        TestCase {
            name: "jaccard_just_below_fails",
            sample_count: 10_000,
            mean_topk_jaccard: 0.8499,
            p99_latency_delta_us: 0.0,
            recall_improvement_ratio: None,
            expected_flip: false,
        },
        TestCase {
            name: "jaccard_just_above_passes",
            sample_count: 10_000,
            mean_topk_jaccard: 0.8501,
            p99_latency_delta_us: 0.0,
            recall_improvement_ratio: None,
            expected_flip: true,
        },
        TestCase {
            name: "jaccard_perfect_passes",
            sample_count: 10_000,
            mean_topk_jaccard: 1.0,
            p99_latency_delta_us: 0.0,
            recall_improvement_ratio: None,
            expected_flip: true,
        },
        // p99_latency_delta_us boundaries
        TestCase {
            name: "latency_delta_negative_faster_passes",
            sample_count: 10_000,
            mean_topk_jaccard: 0.85,
            p99_latency_delta_us: -0.001,
            recall_improvement_ratio: None,
            expected_flip: true,
        },
        TestCase {
            name: "latency_delta_positive_slower_fails",
            sample_count: 10_000,
            mean_topk_jaccard: 0.85,
            p99_latency_delta_us: 0.001,
            recall_improvement_ratio: None,
            expected_flip: false,
        },
        TestCase {
            name: "latency_delta_significantly_slower_fails",
            sample_count: 10_000,
            mean_topk_jaccard: 0.99,
            p99_latency_delta_us: 12.5,
            recall_improvement_ratio: Some(0.30),
            expected_flip: false,
        },
        // recall_improvement_ratio variations (should not impact flip logic)
        TestCase {
            name: "recall_ratio_some_positive",
            sample_count: 10_000,
            mean_topk_jaccard: 0.85,
            p99_latency_delta_us: 0.0,
            recall_improvement_ratio: Some(0.20),
            expected_flip: true,
        },
        TestCase {
            name: "recall_ratio_some_zero",
            sample_count: 10_000,
            mean_topk_jaccard: 0.85,
            p99_latency_delta_us: 0.0,
            recall_improvement_ratio: Some(0.0),
            expected_flip: true,
        },
        TestCase {
            name: "recall_ratio_some_negative",
            sample_count: 10_000,
            mean_topk_jaccard: 0.85,
            p99_latency_delta_us: 0.0,
            recall_improvement_ratio: Some(-0.05),
            expected_flip: true,
        },
        // Multiple failing criteria
        TestCase {
            name: "all_criteria_fail",
            sample_count: 100,
            mean_topk_jaccard: 0.50,
            p99_latency_delta_us: 100.0,
            recall_improvement_ratio: None,
            expected_flip: false,
        },
        TestCase {
            name: "sample_count_and_jaccard_fail",
            sample_count: 5_000,
            mean_topk_jaccard: 0.80,
            p99_latency_delta_us: -5.0,
            recall_improvement_ratio: None,
            expected_flip: false,
        },
    ];

    for case in cases {
        let report = ShadowDiscrepancyReport {
            sample_count: case.sample_count,
            mean_topk_jaccard: case.mean_topk_jaccard,
            p99_latency_delta_us: case.p99_latency_delta_us,
            recall_improvement_ratio: case.recall_improvement_ratio,
        };

        let actual = gate.should_flip(&report);
        assert_eq!(
            actual, case.expected_flip,
            "Test case '{}' failed: expected flip = {}, got = {} for report = {:?}",
            case.name, case.expected_flip, actual, report
        );
    }
}

struct CustomFlipGate;
impl DefaultFlipGate for CustomFlipGate {}

#[test]
fn test_custom_gate_trait_implementation() {
    let custom_gate = CustomFlipGate;
    let report = ShadowDiscrepancyReport {
        sample_count: 10_000,
        mean_topk_jaccard: 0.85,
        p99_latency_delta_us: 0.0,
        recall_improvement_ratio: None,
    };
    assert!(custom_gate.should_flip(&report));
}
