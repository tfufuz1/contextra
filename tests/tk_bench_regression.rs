// ANCHOR[contextra-testkit:TEST-TK-BENCH-REGRESSION]
//! Integration tests for Benchmark-Harness, JSON reporting, and regression detection.

#[path = "../crates/contextra-testkit/src/bench/mod.rs"]
mod bench;

#[path = "../crates/contextra-testkit/src/report.rs"]
mod report;

use bench::harness::{BenchmarkHarness, BenchmarkResult};
use bench::regression::{check_regression, BaselineReport, RegressionConfig};
use report::{BenchmarkSummaryEntry, TestResultEntry, VerifyReport};
use std::time::Duration;
use tempfile::tempdir;

#[test]
fn test_harness_records_and_exports_json() {
    let tmp = tempdir().unwrap();
    let report_path = tmp.path().join("contextra-testkit.json");

    let mut harness = BenchmarkHarness::new("contextra-testkit", 42);
    harness.record(BenchmarkResult {
        id: "BM-TK-HARNESS-OVERHEAD".to_string(),
        p50: Duration::from_nanos(100),
        p95: Duration::from_nanos(150),
        p99: Duration::from_nanos(200),
        throughput_ops_per_sec: 10_000_000.0,
        extra_metrics: Default::default(),
    });

    harness.write_json(&report_path).unwrap();
    assert!(report_path.exists());

    let content = std::fs::read_to_string(&report_path).unwrap();
    assert!(content.contains("BM-TK-HARNESS-OVERHEAD"));
    assert!(content.contains("contextra-testkit"));
}

#[test]
fn test_regression_gate_detects_latency_regression() {
    let baseline = BaselineReport {
        benchmarks: vec![BenchmarkResult {
            id: "BM-WRITE".to_string(),
            p50: Duration::from_millis(10),
            p95: Duration::from_millis(15),
            p99: Duration::from_millis(20),
            throughput_ops_per_sec: 1000.0,
            extra_metrics: Default::default(),
        }],
    };

    // Current run with 30% latency increase (above 10% threshold)
    let current = vec![BenchmarkResult {
        id: "BM-WRITE".to_string(),
        p50: Duration::from_millis(13), // +30%
        p95: Duration::from_millis(18),
        p99: Duration::from_millis(25),
        throughput_ops_per_sec: 900.0,
        extra_metrics: Default::default(),
    }];

    let config = RegressionConfig {
        latency_threshold_percent: 10.0,
        throughput_threshold_percent: 10.0,
    };

    let result = check_regression(&baseline, &current, &config);
    assert!(
        result.is_err(),
        "Regression gate must trigger on +30% latency increase"
    );
}

#[test]
fn test_verify_report_generation() {
    let tmp = tempdir().unwrap();
    let json_path = tmp.path().join("verify-report.json");
    let md_path = tmp.path().join("verify-report.md");

    let mut report = VerifyReport::default();
    report.add_test_result(TestResultEntry {
        test_name: "mv_01_reference_model_no_sentinel".to_string(),
        passed: true,
        duration_ms: 1.5,
    });
    report.add_benchmark_summary(BenchmarkSummaryEntry {
        id: "BM-TK-HARNESS-OVERHEAD".to_string(),
        p50_ns: 100,
        p99_ns: 200,
        ops_per_sec: 10_000_000.0,
    });

    report.write_json(&json_path).unwrap();
    report.write_markdown(&md_path).unwrap();

    assert!(json_path.exists());
    assert!(md_path.exists());
}
