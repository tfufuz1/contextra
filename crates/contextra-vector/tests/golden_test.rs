use std::fs;
use std::path::PathBuf;
use contextra_ports::VectorIndex;
use contextra_types::{DocId, TxId};
use contextra_vector::types::DistanceMetric;
use contextra_vector::distance::{compute_distance, validate_vector};
use contextra_vector::hnsw::{HnswConfig, HnswIndex};
use contextra_vector::persistence::MmapIndex;

fn golden_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("golden")
}

fn deterministic_vector(doc_id: u64, dim: usize) -> Vec<f32> {
    let mut vec = Vec::with_capacity(dim);
    for i in 0..dim {
        let angle = ((doc_id.wrapping_mul(31) + i as u64) % 1000) as f32 / 1000.0 * std::f32::consts::TAU;
        vec.push(angle.sin());
    }
    vec
}

#[tokio::test]
#[ignore]
async fn generate_golden_files() {
    let dir = golden_dir();
    fs::create_dir_all(&dir).unwrap();

    let dim = 16;
    let config_raw = HnswConfig {
        dimension: dim,
        max_elements: 100,
        m: 8,
        ef_construction: 16,
        ef_search: 16,
        distance_metric: DistanceMetric::Cosine,
        rebuild_threshold: 0.8,
        quantize: false,
        ..Default::default()
    };
    let index_raw = HnswIndex::try_new(config_raw).unwrap();
    let tx = TxId::new(1);
    for id in 1..=20u64 {
        let v = deterministic_vector(id, dim);
        index_raw.insert(tx, DocId::new(id), &v).await.unwrap();
    }
    index_raw.commit(tx).await.unwrap();

    let raw_file = dir.join("hnsw_raw.bin");
    index_raw.save(&raw_file).unwrap();

    let config_sq8 = HnswConfig {
        dimension: dim,
        max_elements: 100,
        m: 8,
        ef_construction: 16,
        ef_search: 16,
        distance_metric: DistanceMetric::Cosine,
        rebuild_threshold: 0.8,
        quantize: true,
        ..Default::default()
    };
    let index_sq8 = HnswIndex::try_new(config_sq8).unwrap();
    for id in 1..=20u64 {
        let v = deterministic_vector(id, dim);
        index_sq8.insert(tx, DocId::new(id), &v).await.unwrap();
    }
    index_sq8.commit(tx).await.unwrap();

    let sq8_file = dir.join("hnsw_sq8.bin");
    index_sq8.save(&sq8_file).unwrap();

    let query = deterministic_vector(999, dim);
    let results = index_raw.search(&query, 5).await.unwrap();
    let ranking_data: Vec<(u64, f32)> = results.into_iter().map(|s| (s.doc_id.inner() as u64, s.score)).collect();
    let ranking_json = serde_json::to_string_pretty(&ranking_data).unwrap();
    fs::write(dir.join("ranking_golden.json"), ranking_json).unwrap();

    let dims = [4, 17, 128, 768];
    let metrics = [DistanceMetric::Cosine, DistanceMetric::Euclidean, DistanceMetric::DotProduct];
    let mut oracle_entries = Vec::new();

    for &d in &dims {
        let v1 = deterministic_vector(101, d);
        let v2 = deterministic_vector(202, d);
        let v_zero = vec![0.0f32; d];
        let mut v_denorm = vec![1.0e-38f32; d];
        v_denorm[0] = 1.0e-40f32;

        for &metric in &metrics {
            let metric_name = format!("{metric:?}");
            let d_norm = compute_distance(&v1, &v2, metric).unwrap();
            oracle_entries.push(serde_json::json!({
                "dim": d,
                "metric": metric_name,
                "case": "normal",
                "val": d_norm,
                "bits": d_norm.to_bits(),
            }));
            let d_zero = compute_distance(&v1, &v_zero, metric).unwrap();
            oracle_entries.push(serde_json::json!({
                "dim": d,
                "metric": metric_name,
                "case": "zero",
                "val": d_zero,
                "bits": d_zero.to_bits(),
            }));
            let d_denorm = compute_distance(&v1, &v_denorm, metric).unwrap();
            oracle_entries.push(serde_json::json!({
                "dim": d,
                "metric": metric_name,
                "case": "denormal",
                "val": d_denorm,
                "bits": d_denorm.to_bits(),
            }));
        }
    }
    let oracle_json = serde_json::to_string_pretty(&oracle_entries).unwrap();
    fs::write(dir.join("distance_oracle.json"), oracle_json).unwrap();
}

#[tokio::test]
async fn test_golden_hnsw_persistence() {
    let dir = golden_dir();
    let raw_file = dir.join("hnsw_raw.bin");
    let sq8_file = dir.join("hnsw_sq8.bin");

    assert!(raw_file.exists(), "Raw golden file must exist: {:?}", raw_file);
    assert!(sq8_file.exists(), "SQ8 golden file must exist: {:?}", sq8_file);

    let mmap_raw = MmapIndex::open(&raw_file).expect("open raw golden");
    assert_eq!(mmap_raw.header.node_count(), 20);
    assert_eq!(mmap_raw.header.dimension(), 16);
    assert!(!mmap_raw.header.is_quantized());

    let mmap_sq8 = MmapIndex::open(&sq8_file).expect("open sq8 golden");
    assert_eq!(mmap_sq8.header.node_count(), 20);
    assert_eq!(mmap_sq8.header.dimension(), 16);
    assert!(mmap_sq8.header.is_quantized());
}

#[tokio::test]
async fn test_golden_ranking_and_oracle() {
    let dir = golden_dir();
    let ranking_path = dir.join("ranking_golden.json");
    let oracle_path = dir.join("distance_oracle.json");

    assert!(ranking_path.exists());
    assert!(oracle_path.exists());

    let ranking_str = fs::read_to_string(ranking_path).expect("read ranking");
    let expected_ranking: Vec<(u64, f32)> = serde_json::from_str(&ranking_str).expect("parse ranking");

    let dim = 16;
    let config_raw = HnswConfig {
        dimension: dim,
        max_elements: 100,
        m: 8,
        ef_construction: 16,
        ef_search: 16,
        distance_metric: DistanceMetric::Cosine,
        rebuild_threshold: 0.8,
        quantize: false,
        ..Default::default()
    };
    let index_raw = HnswIndex::try_new(config_raw).unwrap();
    let tx = TxId::new(1);
    for id in 1..=20u64 {
        let v = deterministic_vector(id, dim);
        index_raw.insert(tx, DocId::new(id), &v).await.unwrap();
    }
    index_raw.commit(tx).await.unwrap();

    let query = deterministic_vector(999, dim);
    let results = index_raw.search(&query, 5).await.unwrap();
    let actual_ranking: Vec<(u64, f32)> = results.into_iter().map(|s| (s.doc_id.inner() as u64, s.score)).collect();

    assert_eq!(actual_ranking.len(), expected_ranking.len());
    for (act, exp) in actual_ranking.iter().zip(expected_ranking.iter()) {
        assert_eq!(act.0, exp.0, "DocId mismatch in ranking");
        assert!((act.1 - exp.1).abs() < 1e-6, "Score mismatch: {} vs {}", act.1, exp.1);
    }

    let oracle_str = fs::read_to_string(oracle_path).expect("read oracle");
    let oracle_entries: Vec<serde_json::Value> = serde_json::from_str(&oracle_str).expect("parse oracle");
    for entry in oracle_entries {
        let d = entry["dim"].as_u64().unwrap() as usize;
        let metric_str = entry["metric"].as_str().unwrap();
        let metric = match metric_str {
            "Cosine" => DistanceMetric::Cosine,
            "Euclidean" => DistanceMetric::Euclidean,
            "DotProduct" => DistanceMetric::DotProduct,
            _ => panic!("unknown metric"),
        };
        let case = entry["case"].as_str().unwrap();
        let expected_val = entry["val"].as_f64().unwrap() as f32;

        let v1 = deterministic_vector(101, d);
        let v2 = match case {
            "normal" => deterministic_vector(202, d),
            "zero" => vec![0.0f32; d],
            "denormal" => {
                let mut v = vec![1.0e-38f32; d];
                v[0] = 1.0e-40f32;
                v
            }
            _ => panic!("unknown case"),
        };

        let actual_val = compute_distance(&v1, &v2, metric).unwrap();
        let diff = (actual_val - expected_val).abs();
        let rel = if expected_val.abs() > 1e-3 {
            diff / expected_val.abs()
        } else {
            diff
        };
        assert!(
            diff <= 1e-4 || rel <= 1e-4,
            "Distance mismatch for dim {d}, metric {metric:?}, case {case}: {actual_val} vs {expected_val}"
        );
    }

    let invalid_nan = vec![f32::NAN, 1.0, 2.0, 3.0];
    let invalid_inf = vec![f32::INFINITY, 1.0, 2.0, 3.0];
    let valid = vec![1.0, 2.0, 3.0, 4.0];
    assert!(validate_vector(&invalid_nan).is_err());
    assert!(validate_vector(&invalid_inf).is_err());
    assert!(compute_distance(&invalid_nan, &valid, DistanceMetric::Cosine).is_err());
    assert!(compute_distance(&valid, &invalid_inf, DistanceMetric::Euclidean).is_err());
}
