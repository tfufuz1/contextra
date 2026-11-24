//! Golden-Tests und Korpus-Prüfungen für contextra-rank (RK3.1).

#![allow(clippy::indexing_slicing, clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::fs;
use std::path::Path;

use contextra_rank::calibration::{Clock, IsotonicCalibrator};
use contextra_rank::drift::DriftDetector;
use contextra_rank::fusion::{weighted_reciprocal_rank_fusion, SearchResult};
use contextra_rank::DocId;

struct DeterministicClock(u64);
impl Clock for DeterministicClock {
    fn now_unix_nanos(&self) -> u64 {
        self.0
    }
}

fn canonical_json<T: serde::Serialize>(val: &T) -> String {
    let mut v = serde_json::to_value(val).expect("serialize value");
    sort_json(&mut v);
    serde_json::to_string_pretty(&v).expect("serialize canonical json")
}

fn sort_json(val: &mut serde_json::Value) {
    match val {
        serde_json::Value::Array(arr) => {
            for item in arr {
                sort_json(item);
            }
        }
        serde_json::Value::Object(map) => {
            let mut btree = std::collections::BTreeMap::new();
            for (k, mut v) in std::mem::take(map) {
                sort_json(&mut v);
                btree.insert(k, v);
            }
            for (k, v) in btree {
                map.insert(k, v);
            }
        }
        _ => {}
    }
}

fn generate_rrf_golden() -> String {
    let list1 = vec![
        SearchResult {
            id: "doc-1".to_string(),
            score: 0.9,
            metadata: None,
            matched_signals: vec!["vector".to_string()],
            provenance: None,
        },
        SearchResult {
            id: "doc-2".to_string(),
            score: 0.8,
            metadata: None,
            matched_signals: vec!["vector".to_string()],
            provenance: None,
        },
        SearchResult {
            id: "doc-tie-b".to_string(),
            score: 0.7,
            metadata: None,
            matched_signals: vec!["vector".to_string()],
            provenance: None,
        },
    ];

    let list2 = vec![
        SearchResult {
            id: "doc-tie-a".to_string(),
            score: 0.9,
            metadata: None,
            matched_signals: vec!["text".to_string()],
            provenance: None,
        },
        SearchResult {
            id: "doc-2".to_string(),
            score: 0.85,
            metadata: None,
            matched_signals: vec!["text".to_string()],
            provenance: None,
        },
        SearchResult {
            id: "doc-1".to_string(),
            score: 0.6,
            metadata: None,
            matched_signals: vec!["text".to_string()],
            provenance: None,
        },
    ];

    let weighted_inputs = vec![
        ("vector".to_string(), list1, 1.0_f32),
        ("text".to_string(), list2, 1.0_f32),
    ];
    let fused = weighted_reciprocal_rank_fusion(weighted_inputs, 10);

    canonical_json(&fused)
}

fn generate_isotonic_golden() -> String {
    let mut cal = IsotonicCalibrator::new(5, 100).with_clock(std::sync::Arc::new(DeterministicClock(1_700_000_000_000_000_000)));
    let observations = [
        (0.1_f32, false),
        (0.2, false),
        (0.35, true),
        (0.4, false),
        (0.55, true),
        (0.65, true),
        (0.8, true),
        (0.95, true),
    ];
    for (s, o) in observations {
        cal.record_outcome(s, o);
    }
    cal.force_rebuild();

    let query_points = [0.05_f32, 0.2, 0.3, 0.5, 0.7, 0.9, 1.0];
    let predictions: Vec<(f32, Option<f32>)> = query_points.iter().map(|&p| (p, cal.calibrated_probability(p))).collect();

    canonical_json(&predictions)
}

fn generate_drift_golden() -> String {
    let mut detector = DriftDetector::new(50, 0.05);
    for i in 0..100 {
        let score = (i as f32 % 10.0) / 10.0;
        detector.observe(score);
    }
    let status = detector.analyze();
    canonical_json(&status)
}

fn generate_id_serde_golden() -> String {
    let ids: Vec<DocId> = (0..50).map(DocId).collect();
    canonical_json(&ids)
}

#[test]
fn test_golden_fixtures_verify() {
    let golden_dir = Path::new("tests/golden");
    assert!(golden_dir.exists(), "tests/golden must exist");

    let rrf_actual = generate_rrf_golden();
    let rrf_path = golden_dir.join("rrf_fusion.json");
    let rrf_expected = fs::read_to_string(&rrf_path).expect("read rrf_fusion.json");
    assert_eq!(rrf_actual.trim(), rrf_expected.trim(), "RRF golden mismatch");

    let iso_actual = generate_isotonic_golden();
    let iso_path = golden_dir.join("isotonic_calibration.json");
    let iso_expected = fs::read_to_string(&iso_path).expect("read isotonic_calibration.json");
    assert_eq!(iso_actual.trim(), iso_expected.trim(), "Isotonic golden mismatch");

    let drift_actual = generate_drift_golden();
    let drift_path = golden_dir.join("drift_status.json");
    let drift_expected = fs::read_to_string(&drift_path).expect("read drift_status.json");
    assert_eq!(drift_actual.trim(), drift_expected.trim(), "Drift golden mismatch");

    let id_actual = generate_id_serde_golden();
    let id_path = golden_dir.join("id_serde.json");
    let id_expected = fs::read_to_string(&id_path).expect("read id_serde.json");
    assert_eq!(id_actual.trim(), id_expected.trim(), "ID Serde golden mismatch");
}

#[test]
fn test_golden_backward_read_no_panic() {
    let golden_dir = Path::new("tests/golden");
    for entry in fs::read_dir(golden_dir).expect("read golden dir") {
        let entry = entry.expect("entry");
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("json") {
            let content = fs::read_to_string(&path).expect("read content");
            let _val: serde_json::Value = serde_json::from_str(&content).expect("parse json");
        }
    }
}

#[test]
#[ignore]
fn regen_fixtures() {
    let golden_dir = Path::new("tests/golden");
    fs::create_dir_all(golden_dir).unwrap();
    fs::write(golden_dir.join("rrf_fusion.json"), generate_rrf_golden()).unwrap();
    fs::write(golden_dir.join("isotonic_calibration.json"), generate_isotonic_golden()).unwrap();
    fs::write(golden_dir.join("drift_status.json"), generate_drift_golden()).unwrap();
    fs::write(golden_dir.join("id_serde.json"), generate_id_serde_golden()).unwrap();
}
