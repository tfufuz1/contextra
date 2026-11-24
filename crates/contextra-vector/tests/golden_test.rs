use contextra_ports::{Rng as PortsRng, VectorIndex};
use contextra_types::{DocId, TxId};
use contextra_vector::distance::{compute_distance, validate_vector};
use contextra_vector::hnsw::{HnswConfig, HnswIndex};
use contextra_vector::persistence::MmapIndex;
use contextra_vector::types::DistanceMetric;
use std::fs;
use std::path::PathBuf;

#[cfg(feature = "experimental-diskann")]
use contextra_vector::diskann::{DiskAnnConfig, DiskAnnFallbackPolicy, DiskAnnIndex};

fn golden_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("golden")
}

fn deterministic_vector(doc_id: u64, dim: usize) -> Vec<f32> {
    let mut vec = Vec::with_capacity(dim);
    for i in 0..dim {
        let angle =
            ((doc_id.wrapping_mul(31) + i as u64) % 1000) as f32 / 1000.0 * std::f32::consts::TAU;
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

    // Golden for HNSW with tombstones / soft-deleted documents
    let config_tombstones = HnswConfig {
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
    let index_tombstones = HnswIndex::try_new(config_tombstones).unwrap();
    for id in 1..=20u64 {
        let v = deterministic_vector(id, dim);
        index_tombstones
            .insert(tx, DocId::new(id), &v)
            .await
            .unwrap();
    }
    index_tombstones.commit(tx).await.unwrap();
    let tx_del = TxId::new(2);
    index_tombstones
        .delete(tx_del, DocId::new(5))
        .await
        .unwrap();
    index_tombstones
        .delete(tx_del, DocId::new(15))
        .await
        .unwrap();
    index_tombstones.commit(tx_del).await.unwrap();
    let tombstones_file = dir.join("hnsw_tombstones.bin");
    index_tombstones.save(&tombstones_file).unwrap();

    // Level & RNG Golden (Seed determinism layer assignments)
    let mut level_entries = Vec::new();
    for seed in [42u64, 100u64, 999u64] {
        let mut layers = Vec::new();
        let rng = contextra_ports::SeededRng::new(seed);
        for _ in 0..100 {
            let r = rng.next_unit_f64() as f32;
            let r_clamped = r.max(f32::EPSILON);
            let ml = 1.0 / (8.0f64).ln(); // for m=8
            let layer = (-(r_clamped.ln()) as f64 * ml) as usize;
            layers.push(layer.min(32));
        }
        level_entries.push(serde_json::json!({
            "seed": seed,
            "layers": layers,
        }));
    }
    fs::write(
        dir.join("level_rng_golden.json"),
        serde_json::to_string_pretty(&level_entries).unwrap(),
    )
    .unwrap();

    let query = deterministic_vector(999, dim);
    let results = index_raw.search(&query, 5).await.unwrap();
    let ranking_data: Vec<(u64, f32)> = results
        .into_iter()
        .map(|s| (s.doc_id.inner() as u64, s.score))
        .collect();
    let ranking_json = serde_json::to_string_pretty(&ranking_data).unwrap();
    fs::write(dir.join("ranking_golden.json"), ranking_json).unwrap();

    let dims = [4, 17, 128, 768];
    let metrics = [
        DistanceMetric::Cosine,
        DistanceMetric::Euclidean,
        DistanceMetric::DotProduct,
    ];
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

    #[cfg(feature = "experimental-diskann")]
    {
        let diskann_file = dir.join("diskann.dann");
        let pwal_file = dir.join("diskann.pending.wal");
        let twal_file = dir.join("diskann.tombstone.wal");

        let _ = fs::remove_file(&diskann_file);
        let _ = fs::remove_file(&pwal_file);
        let _ = fs::remove_file(&twal_file);

        let diskann_config = DiskAnnConfig {
            dimension: dim,
            max_degree: 8,
            beam_width: 16,
            index_path: diskann_file.clone(),
            distance_metric: DistanceMetric::Cosine,
            quantize: false,
            fallback_policy: DiskAnnFallbackPolicy::FailFast,
            ..Default::default()
        };
        let diskann = DiskAnnIndex::try_new(diskann_config).unwrap();
        let mut vecs = Vec::new();
        let mut ids = Vec::new();
        for id in 1..=20u64 {
            vecs.push(deterministic_vector(id, dim));
            ids.push(DocId::new(id));
        }
        diskann.build(&vecs, &ids).await.unwrap();

        let p_vec = deterministic_vector(21, dim);
        diskann
            .insert(TxId::new(10), DocId::new(21), &p_vec)
            .await
            .unwrap();
        diskann.delete(TxId::new(10), DocId::new(5)).await.unwrap();
    }
}

#[tokio::test]
async fn test_golden_hnsw_persistence() {
    let dir = golden_dir();
    let raw_file = dir.join("hnsw_raw.bin");
    let sq8_file = dir.join("hnsw_sq8.bin");

    assert!(
        raw_file.exists(),
        "Raw golden file must exist: {:?}",
        raw_file
    );
    assert!(
        sq8_file.exists(),
        "SQ8 golden file must exist: {:?}",
        sq8_file
    );

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
async fn test_golden_tombstones_persistence() {
    let dir = golden_dir();
    let tombstones_file = dir.join("hnsw_tombstones.bin");
    if tombstones_file.exists() {
        let mmap_tomb = MmapIndex::open(&tombstones_file).expect("open tombstones golden");
        assert_eq!(mmap_tomb.header.node_count(), 20);
        assert_eq!(mmap_tomb.header.dimension(), 16);
    }
}

#[tokio::test]
async fn test_golden_level_rng_determinism() {
    let dir = golden_dir();
    let level_path = dir.join("level_rng_golden.json");
    if level_path.exists() {
        let content = fs::read_to_string(level_path).expect("read level golden");
        let entries: Vec<serde_json::Value> =
            serde_json::from_str(&content).expect("parse level golden");
        for entry in entries {
            let seed = entry["seed"].as_u64().unwrap();
            let expected_layers: Vec<usize> = entry["layers"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_u64().unwrap() as usize)
                .collect();
            let rng = contextra_ports::SeededRng::new(seed);
            let mut actual_layers = Vec::new();
            for _ in 0..expected_layers.len() {
                let r = rng.next_unit_f64() as f32;
                let r_clamped = r.max(f32::EPSILON);
                let ml = 1.0 / (8.0f64).ln();
                let layer = (-(r_clamped.ln()) as f64 * ml) as usize;
                actual_layers.push(layer.min(32));
            }
            assert_eq!(actual_layers, expected_layers, "Mismatch for seed {}", seed);
        }
    }
}

#[cfg(feature = "experimental-diskann")]
#[tokio::test]
async fn test_golden_diskann_persistence() {
    let dir = golden_dir();
    let diskann_file = dir.join("diskann.dann");
    if diskann_file.exists() {
        let bytes = fs::read(&diskann_file).expect("read diskann golden");
        assert!(
            bytes.len() >= 40 + 36,
            "DiskANN file must have at least header + footer"
        );
        assert_eq!(&bytes[0..4], b"DANN", "Invalid DISKANN magic");
        assert_eq!(
            &bytes[bytes.len() - 36..bytes.len() - 32],
            b"DANF",
            "Invalid DISKANN footer magic"
        );
    }
}

#[tokio::test]
async fn test_golden_ranking_and_oracle() {
    let dir = golden_dir();
    let ranking_path = dir.join("ranking_golden.json");
    let oracle_path = dir.join("distance_oracle.json");

    assert!(ranking_path.exists());
    assert!(oracle_path.exists());

    let ranking_str = fs::read_to_string(ranking_path).expect("read ranking");
    let expected_ranking: Vec<(u64, f32)> =
        serde_json::from_str(&ranking_str).expect("parse ranking");

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
    let results = index_raw
        .search(&query, expected_ranking.len())
        .await
        .unwrap();
    let actual_ranking: Vec<(u64, f32)> = results
        .into_iter()
        .map(|s| (s.doc_id.inner() as u64, s.score))
        .collect();

    assert_eq!(actual_ranking.len(), expected_ranking.len());
    for (act, exp) in actual_ranking.iter().zip(expected_ranking.iter()) {
        assert_eq!(act.0, exp.0, "DocId mismatch in ranking");
        assert!(
            (act.1 - exp.1).abs() < 1e-6,
            "Score mismatch: {} vs {}",
            act.1,
            exp.1
        );
    }

    let oracle_str = fs::read_to_string(oracle_path).expect("read oracle");
    let oracle_entries: Vec<serde_json::Value> =
        serde_json::from_str(&oracle_str).expect("parse oracle");
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
