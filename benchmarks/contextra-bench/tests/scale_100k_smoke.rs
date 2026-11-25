// FILE-CONTEXT
// STAND: 2026-09-30
// ZWECK: Integrationstest / Smoke Test fuer scale-100k Benchmark Binary

use serde_json::Value;
use std::process::Command;
use tempfile::NamedTempFile;

#[test]
fn test_scale_100k_smoke() {
    let temp_file = NamedTempFile::new().expect("Failed to create temp file");
    let out_path = temp_file.path().to_path_buf();

    let exe_path = env!("CARGO_BIN_EXE_scale-100k");

    let status = Command::new(exe_path)
        .arg("--docs")
        .arg("300")
        .arg("--step")
        .arg("100")
        .arg("--dim")
        .arg("8")
        .arg("--seed")
        .arg("42")
        .arg("--out")
        .arg(&out_path)
        .status()
        .expect("Failed to execute scale-100k binary");

    assert!(status.success(), "scale-100k process did not succeed");
    assert!(out_path.exists(), "Output JSON file does not exist");

    let content = std::fs::read_to_string(&out_path).expect("Failed to read output JSON");
    let json: Value = serde_json::from_str(&content).expect("Invalid JSON output");

    // 1. Disclaimer field check
    let disclaimer = json
        .get("disclaimer")
        .and_then(|v| v.as_str())
        .expect("Missing 'disclaimer' field");
    assert!(
        disclaimer.contains("messen nur Speicher- und Indexlatenz"),
        "Disclaimer does not contain expected substring"
    );
    assert!(
        disclaimer.contains("300 Chunks"),
        "Disclaimer does not contain requested chunk count"
    );

    // 2. Interval count check
    let intervals = json
        .get("intervals")
        .and_then(|v| v.as_array())
        .expect("Missing or invalid 'intervals' array");
    assert_eq!(intervals.len(), 3, "Expected exactly 3 intervals");

    // 3. Latency values > 0 check
    for (idx, interval) in intervals.iter().enumerate() {
        let docs = interval
            .get("docs_inserted")
            .and_then(|v| v.as_u64())
            .expect("Missing docs_inserted");
        assert_eq!(docs, (idx as u64 + 1) * 100);

        let p50 = interval
            .get("search_p50_ns")
            .and_then(|v| v.as_u64())
            .expect("Missing search_p50_ns");
        let p95 = interval
            .get("search_p95_ns")
            .and_then(|v| v.as_u64())
            .expect("Missing search_p95_ns");
        let p99 = interval
            .get("search_p99_ns")
            .and_then(|v| v.as_u64())
            .expect("Missing search_p99_ns");

        assert!(p50 > 0, "p50 latency must be > 0 at interval {}", idx);
        assert!(p95 > 0, "p95 latency must be > 0 at interval {}", idx);
        assert!(p99 > 0, "p99 latency must be > 0 at interval {}", idx);
    }

    // 4. Repeat run with same seed yields same interval count
    let temp_file_2 = NamedTempFile::new().expect("Failed to create second temp file");
    let out_path_2 = temp_file_2.path().to_path_buf();

    let status_2 = Command::new(exe_path)
        .arg("--docs")
        .arg("300")
        .arg("--step")
        .arg("100")
        .arg("--dim")
        .arg("8")
        .arg("--seed")
        .arg("42")
        .arg("--out")
        .arg(&out_path_2)
        .status()
        .expect("Failed to execute scale-100k binary second time");

    assert!(
        status_2.success(),
        "Second scale-100k process did not succeed"
    );
    let content_2 =
        std::fs::read_to_string(&out_path_2).expect("Failed to read second output JSON");
    let json_2: Value = serde_json::from_str(&content_2).expect("Invalid second JSON output");

    let intervals_2 = json_2
        .get("intervals")
        .and_then(|v| v.as_array())
        .expect("Missing 'intervals' array in second run");
    assert_eq!(
        intervals.len(),
        intervals_2.len(),
        "Both runs with same seed must return same number of intervals"
    );
}
