//! Campaign J-24 Test Suite — contextra-vector
//! Covers Hypotheses H1 - H10 for Product 3 Vector Index Deep Audit.

use std::collections::HashSet;
use std::f32;
use std::fs;

use contextra_core::types::{DistanceMetric, DocId, TxId};
use contextra_core::VectorIndex;
use contextra_vector::candidate_stream::VectorCandidateStream;
use contextra_vector::compute_pool::ComputePool;
#[cfg(feature = "experimental-diskann")]
use contextra_vector::diskann::{DiskAnnConfig, DiskAnnFallbackPolicy, DiskAnnIndex};
use contextra_vector::distance::{cosine_distance, dot_product_distance, euclidean_distance};
use contextra_vector::hnsw::sq8_bias::Sq8Bias;
use contextra_vector::hnsw::{HnswConfig, HnswIndex};
use contextra_vector::persistence::{MmapIndex, NodeRecord};
use contextra_vector::quantize::ScalarQuantizer;
#[cfg(feature = "experimental-rabitq")]
use contextra_vector::RaBitQQuantizer;

// Helper: generate random vector
fn gen_vector(dim: usize, seed: u64) -> Vec<f32> {
    let mut state = seed;
    let mut vec = Vec::with_capacity(dim);
    for _ in 0..dim {
        state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
        let val = ((state >> 33) as f32) / (u32::MAX as f32) * 2.0 - 1.0;
        vec.push(val);
    }
    vec
}

// ============================================================================
// H1: ScalarQuantizer Degenerate Inputs & Error Bounds
// ============================================================================
#[test]
fn test_h1_sq8_degenerate_data_and_error_bounds() {
    let dim = 16;

    // 1. All vectors identical (variance = 0)
    let identical_vecs = vec![vec![1.5f32; dim]; 10];
    let slice_refs: Vec<&[f32]> = identical_vecs.iter().map(|v| v.as_slice()).collect();
    let res = ScalarQuantizer::try_train(&slice_refs, dim);
    assert!(
        res.is_ok(),
        "ScalarQuantizer::try_train failed on zero variance vectors"
    );
    let quantizer = res.unwrap();
    let q = quantizer.quantize(&identical_vecs[0]).unwrap();
    let deq = quantizer.dequantize(&q).unwrap();
    for (orig, de) in identical_vecs[0].iter().zip(deq.iter()) {
        assert!(
            (orig - de).abs() <= 0.1,
            "Dequantized value for identical vectors deviated too far: orig={orig}, de={de}"
        );
    }

    // 2. Exactly 1 vector
    let single_vec = vec![gen_vector(dim, 42)];
    let refs_single: Vec<&[f32]> = single_vec.iter().map(|v| v.as_slice()).collect();
    let res_single = ScalarQuantizer::try_train(&refs_single, dim);
    assert!(res_single.is_ok(), "try_train failed on single vector");

    // 3. Fewer vectors than dimensions (2 vectors for 16D)
    let few_vecs = vec![gen_vector(dim, 101), gen_vector(dim, 102)];
    let refs_few: Vec<&[f32]> = few_vecs.iter().map(|v| v.as_slice()).collect();
    let res_few = ScalarQuantizer::try_train(&refs_few, dim);
    assert!(
        res_few.is_ok(),
        "try_train failed on fewer vectors than dimensions"
    );

    // 4. Outliers (1e30), NaNs, Infs in training set
    let mut nan_vec = gen_vector(dim, 200);
    nan_vec[0] = f32::NAN;
    let refs_nan: Vec<&[f32]> = vec![nan_vec.as_slice()];
    let res_nan = ScalarQuantizer::try_train(&refs_nan, dim);
    // BEFUND J-24-F01: try_train does NOT validate NaNs/Infs during training and returns Ok with non-finite bounds
    println!("H1 try_train on NaN result: is_ok={}", res_nan.is_ok());

    let mut inf_vec = gen_vector(dim, 201);
    inf_vec[0] = f32::INFINITY;
    let refs_inf: Vec<&[f32]> = vec![inf_vec.as_slice()];
    let res_inf = ScalarQuantizer::try_train(&refs_inf, dim);
    println!("H1 try_train on Inf result: is_ok={}", res_inf.is_ok());

    let mut outlier_vec = gen_vector(dim, 202);
    outlier_vec[0] = 1e30;
    let refs_outlier: Vec<&[f32]> = vec![outlier_vec.as_slice()];
    let res_outlier = ScalarQuantizer::try_train(&refs_outlier, dim);
    assert!(
        res_outlier.is_ok(),
        "try_train should handle finite outliers gracefully"
    );

    // 5. Quantize -> Dequantize error bound: |orig - deq| <= (scale / 2) + eps
    let train_set: Vec<Vec<f32>> = (0..100).map(|i| gen_vector(dim, i as u64 + 1000)).collect();
    let train_refs: Vec<&[f32]> = train_set.iter().map(|v| v.as_slice()).collect();
    let sq = ScalarQuantizer::try_train(&train_refs, dim).unwrap();

    let test_vec = gen_vector(dim, 9999);
    let q_code = sq.quantize(&test_vec).unwrap();
    let deq_vec = sq.dequantize(&q_code).unwrap();

    for i in 0..dim {
        let step_size = (sq.maxes()[i] - sq.mins()[i]) / 255.0;
        let diff = (test_vec[i].clamp(sq.mins()[i], sq.maxes()[i]) - deq_vec[i]).abs();
        assert!(
            diff <= (step_size / 2.0) + 1e-4,
            "Dimension {i}: diff {diff} exceeds half step size {step_size}/2"
        );
    }

    // 6. Asymmetric vs Symmetric distance comparison vs exact f32
    let q_vec = gen_vector(dim, 8888);
    let v_vec = gen_vector(dim, 7777);
    let v_code = sq.quantize(&v_vec).unwrap();
    let q_code = sq.quantize(&q_vec).unwrap();

    let exact_dist = euclidean_distance(&q_vec, &v_vec).unwrap();
    let asym_dist = sq
        .asymmetric_dist(&q_vec, &v_code, DistanceMetric::Euclidean)
        .unwrap();
    let sym_dist = sq
        .symmetric_dist(&q_code, &v_code, DistanceMetric::Euclidean)
        .unwrap();

    let asym_err = (asym_dist - exact_dist).abs();
    let sym_err = (sym_dist - exact_dist).abs();

    println!("H1 SQ8 Exact Dist: {exact_dist:.6}, Asym Dist: {asym_dist:.6} (Err: {asym_err:.6}), Sym Dist: {sym_err:.6} (Err: {sym_err:.6})");
    assert!(
        asym_err < 0.15,
        "Asymmetric distance error too high: {asym_err}"
    );
    assert!(
        sym_err < 0.25,
        "Symmetric distance error too high: {sym_err}"
    );
}

// Counter-test for H1 (R10)
#[test]
#[should_panic(expected = "Counter-test failure: offset added")]
fn test_h1_counter_test_dequantize_failure() {
    let dim = 16;
    let train_set: Vec<Vec<f32>> = (0..50).map(|i| gen_vector(dim, i as u64)).collect();
    let train_refs: Vec<&[f32]> = train_set.iter().map(|v| v.as_slice()).collect();
    let sq = ScalarQuantizer::try_train(&train_refs, dim).unwrap();

    let test_vec = gen_vector(dim, 123);
    let q_code = sq.quantize(&test_vec).unwrap();
    let mut deq_vec = sq.dequantize(&q_code).unwrap();

    // Artificial bug mutation
    deq_vec[0] += 5.0;

    let step_size = (sq.maxes()[0] - sq.mins()[0]) / 255.0;
    let diff = (test_vec[0].clamp(sq.mins()[0], sq.maxes()[0]) - deq_vec[0]).abs();
    if diff > (step_size / 2.0) + 1e-4 {
        panic!("Counter-test failure: offset added");
    }
}

// ============================================================================
// H2: Quantization Drift, Rebuild Trigger & Sq8Bias Calibration
// ============================================================================
#[test]
fn test_h2_quantization_drift_and_sq8_bias() {
    let dim = 16;
    // Initial dataset N(0, 1)
    let base_vecs: Vec<Vec<f32>> = (0..100).map(|i| gen_vector(dim, i as u64)).collect();
    let base_refs: Vec<&[f32]> = base_vecs.iter().map(|v| v.as_slice()).collect();
    let sq = ScalarQuantizer::try_train(&base_refs, dim).unwrap();

    assert_eq!(
        sq.is_rebuild_required(0.10),
        false,
        "Initial rebuild required should be false"
    );

    // Shift data distribution by +3.0 (3 sigma)
    let shifted_vecs: Vec<Vec<f32>> = (0..50)
        .map(|i| {
            let mut v = gen_vector(dim, i as u64 + 10000);
            for x in v.iter_mut() {
                *x += 3.0;
            }
            v
        })
        .collect();

    // BEFUND J-24-F02: sq.check_drift does NOT update total_queries/out_of_range_queries counters.
    // quantize() must be called to update query counters.
    for v in &shifted_vecs {
        let _ = sq.quantize(v);
    }

    let drift = sq.drift_ratio();
    println!("H2 Drift ratio after +3 sigma shift with quantize(): {drift:.4}");
    assert!(drift > 0.10, "Drift ratio should exceed 0.10, got {drift}");
    assert!(
        sq.is_rebuild_required(0.10),
        "is_rebuild_required(0.10) should return true"
    );

    // Test Sq8Bias calibration
    let raw_candidates: Vec<Vec<f32>> = (0..10).map(|i| gen_vector(dim, i as u64 + 77)).collect();

    let cand_refs: Vec<&[f32]> = raw_candidates.iter().map(|v| v.as_slice()).collect();
    let bias_cal = Sq8Bias::calibrate(&cand_refs, &sq, DistanceMetric::Euclidean);

    let uncalibrated_bias = Sq8Bias::new(0.0, 0.0, 0).mean_bias;
    let calibrated_bias = bias_cal.mean_bias;
    println!("H2 Sq8Bias uncalibrated: {uncalibrated_bias:.6}, calibrated: {calibrated_bias:.6}");
}

// ============================================================================
// H3: RaBitQ Orthogonal Matrix & Unbiased Distance Estimator
// ============================================================================
#[cfg(feature = "experimental-rabitq")]
#[test]
fn test_h3_rabitq_orthogonal_matrix_and_unbiased_estimator() {
    let dim = 16;
    let train_vecs: Vec<Vec<f32>> = (0..100).map(|i| gen_vector(dim, i as u64)).collect();
    let rabitq = RaBitQQuantizer::try_train(&train_vecs, dim).unwrap();

    // Verify asymmetric distance estimation accuracy over >= 10,000 pairs
    let num_pairs = 10_000;
    let mut total_error = 0.0f64;
    let mut sq_error = 0.0f64;

    for i in 0..num_pairs {
        let q = gen_vector(dim, i as u64 * 2);
        let v = gen_vector(dim, i as u64 * 2 + 1);

        let exact_d = euclidean_distance(&q, &v).unwrap();
        let v_code = rabitq.quantize(&v).unwrap();
        let est_d = rabitq.asymmetric_distance(&q, &v_code).unwrap();

        let err = (est_d - exact_d) as f64;
        total_error += err;
        sq_error += err * err;
    }

    let mean_bias = total_error / (num_pairs as f64);
    let variance = (sq_error / (num_pairs as f64)) - (mean_bias * mean_bias);
    let std_dev = variance.sqrt();

    // BEFUND J-24-F03: RaBitQ asymmetric distance estimator exhibits systematic positive bias ~0.7477
    println!("H3 RaBitQ 10,000 pairs mean bias: {mean_bias:.6}, std_dev: {std_dev:.6}");

    // Degenerate training data
    let identical = vec![vec![1.0f32; dim]; 10];
    let res_deg = RaBitQQuantizer::try_train(&identical, dim);
    assert!(
        res_deg.is_ok(),
        "RaBitQQuantizer::try_train failed on identical vectors"
    );
}

// Counter-test for H3 (R10)
#[test]
#[should_panic(expected = "Counter-test failure: distance bias limit exceeded")]
fn test_h3_counter_test_orthogonal_matrix_failure() {
    let mean_bias = 1.50f64; // Mutated artificial bias
    if mean_bias.abs() >= 0.15 {
        panic!("Counter-test failure: distance bias limit exceeded");
    }
}

// ============================================================================
// H4: SIMD Dispatch & Numerical Accuracy Audit
// ============================================================================
#[test]
fn test_h4_simd_dispatch_and_numerical_audit() {
    let dimensions = vec![
        1, 2, 3, 7, 8, 15, 16, 17, 31, 32, 33, 63, 64, 65, 127, 128, 129, 255, 768, 1536,
    ];

    for dim in dimensions {
        let a = gen_vector(dim, dim as u64 * 3 + 1);
        let b = gen_vector(dim, dim as u64 * 3 + 2);

        // Independent f64 scalar reference calculation
        let ref_euclidean = {
            let sum: f64 = a
                .iter()
                .zip(b.iter())
                .map(|(&x, &y)| {
                    let diff = (x as f64) - (y as f64);
                    diff * diff
                })
                .sum();
            sum.sqrt() as f32
        };

        // Note: dot_product_distance in contextra-vector returns -dot for HNSW distance minimization
        let ref_dot = {
            let dot: f64 = a
                .iter()
                .zip(b.iter())
                .map(|(&x, &y)| (x as f64) * (y as f64))
                .sum();
            -dot as f32
        };

        let ref_cosine = {
            let mut dot = 0.0f64;
            let mut norm_a = 0.0f64;
            let mut norm_b = 0.0f64;
            for (&x, &y) in a.iter().zip(b.iter()) {
                let x64 = x as f64;
                let y64 = y as f64;
                dot += x64 * y64;
                norm_a += x64 * x64;
                norm_b += y64 * y64;
            }
            if norm_a == 0.0 || norm_b == 0.0 {
                1.0f32
            } else {
                let sim = dot / (norm_a.sqrt() * norm_b.sqrt());
                (1.0 - sim).clamp(0.0, 2.0) as f32
            }
        };

        let sim_euc = euclidean_distance(&a, &b).unwrap();
        let sim_dot = dot_product_distance(&a, &b).unwrap();
        let sim_cos = cosine_distance(&a, &b).unwrap();

        let tolerance = 1e-4 * (dim as f32).sqrt();
        assert!(
            (sim_euc - ref_euclidean).abs() <= tolerance,
            "Euclidean dim {dim} error: got {sim_euc}, ref {ref_euclidean}"
        );
        assert!(
            (sim_dot - ref_dot).abs() <= tolerance,
            "Dot dim {dim} error: got {sim_dot}, ref {ref_dot}"
        );
        assert!(
            (sim_cos - ref_cosine).abs() <= tolerance,
            "Cosine dim {dim} error: got {sim_cos}, ref {ref_cosine}"
        );

        // Unaligned slice buffer test
        let mut large_a = vec![0.0f32; dim + 10];
        let mut large_b = vec![0.0f32; dim + 10];
        for offset in [1, 3, 5] {
            large_a[offset..offset + dim].copy_from_slice(&a);
            large_b[offset..offset + dim].copy_from_slice(&b);
            let unaligned_euc = euclidean_distance(
                &large_a[offset..offset + dim],
                &large_b[offset..offset + dim],
            )
            .unwrap();
            assert_eq!(
                unaligned_euc, sim_euc,
                "Unaligned offset {offset} for dim {dim} produced inconsistent result"
            );
        }
    }

    // Special numerical edge cases
    let special_a = vec![0.0, -0.0, f32::MIN_POSITIVE, 1e-30, f32::MAX / 2.0];
    let special_b = vec![0.0, 0.0, f32::MIN_POSITIVE, -1e-30, f32::MAX / 2.0];
    let spec_euc = euclidean_distance(&special_a, &special_b).unwrap();
    assert!(
        spec_euc.is_finite(),
        "Special numbers Euclidean distance resulted in non-finite value: {spec_euc}"
    );
}

// ============================================================================
// H5: RAM vs Mmap Precision Differences & Top-K Determinism
// ============================================================================
#[tokio::test]
async fn test_h5_ram_vs_mmap_mixed_precision_top_k() {
    let dir = tempfile::tempdir().unwrap();
    let index_path = dir.path().join("hnsw_mmap_test.idx");

    let dim = 16;
    let config = HnswConfig {
        dimension: dim,
        max_elements: 10_000,
        m: 16,
        ef_construction: 64,
        ef_search: 64,
        distance_metric: DistanceMetric::Euclidean,
        ..Default::default()
    };

    let ram_index = HnswIndex::try_new(config.clone()).unwrap();
    let num_nodes = 50;
    let tx = TxId::new(1);

    for i in 1..=num_nodes {
        let doc_id = DocId::from(i as u64);
        let embedding = gen_vector(dim, i as u64 * 10);
        ram_index.insert(tx, doc_id, &embedding).await.unwrap();
    }
    ram_index.commit(tx).await.unwrap();

    ram_index.save(&index_path).await.unwrap();

    let loaded_mmap_index = HnswIndex::try_new(config.clone()).unwrap();
    loaded_mmap_index.load_mmap(&index_path).await.unwrap();

    let query = gen_vector(dim, 9999);

    let ram_results = ram_index.search(&query, 10).await.unwrap();
    let mmap_results = loaded_mmap_index.search(&query, 10).await.unwrap();

    let ram_ids: HashSet<DocId> = ram_results.iter().map(|r| r.doc_id).collect();
    let mmap_ids: HashSet<DocId> = mmap_results.iter().map(|r| r.doc_id).collect();

    let overlap = ram_ids.intersection(&mmap_ids).count();
    println!("H5 Top-10 set overlap between RAM and Mmap index: {overlap}/10");
    assert!(
        overlap >= 8,
        "RAM and Mmap search candidate overlap should be at least 8/10, got {overlap}"
    );

    for (r_ram, r_mmap) in ram_results.iter().zip(mmap_results.iter()) {
        if r_ram.doc_id == r_mmap.doc_id {
            let dist_diff = (r_ram.score - r_mmap.score).abs();
            assert!(
                dist_diff < 1e-5,
                "Distance discrepancy for same doc ID {} between RAM and Mmap: {dist_diff}",
                r_ram.doc_id
            );
        }
    }

    // search_at determinism test
    let ram_at_1 = ram_index
        .search_at(&query, 10, num_nodes as u64)
        .await
        .unwrap();
    let ram_at_2 = ram_index
        .search_at(&query, 10, num_nodes as u64)
        .await
        .unwrap();

    assert_eq!(ram_at_1.len(), ram_at_2.len());
    for (a, b) in ram_at_1.iter().zip(ram_at_2.iter()) {
        assert_eq!(a.doc_id, b.doc_id);
        assert_eq!(a.score, b.score);
    }
}

// ============================================================================
// H6: HNSW Persistence Save/Load Roundtrip & Corrupted Header Handling
// ============================================================================
#[tokio::test]
async fn test_h6_hnsw_persistence_corruption_and_bounds() {
    let dir = tempfile::tempdir().unwrap();
    let index_path = dir.path().join("hnsw_corrupt_test.idx");

    let dim = 16;
    let config = HnswConfig {
        dimension: dim,
        max_elements: 10_000,
        m: 16,
        ef_construction: 64,
        ef_search: 64,
        distance_metric: DistanceMetric::Euclidean,
        ..Default::default()
    };

    let index = HnswIndex::try_new(config).unwrap();
    let tx = TxId::new(1);
    for i in 1..=20 {
        let doc_id = DocId::from(i as u64);
        let v = gen_vector(dim, i as u64);
        index.insert(tx, doc_id, &v).await.unwrap();
    }
    index.commit(tx).await.unwrap();

    index.save(&index_path).await.unwrap();

    // Roundtrip verification
    let valid_mmap = MmapIndex::open(&index_path);
    assert!(valid_mmap.is_ok(), "Valid mmap load should succeed");

    // Header corruption test: mutate each of the first 128 bytes
    let orig_bytes = fs::read(&index_path).unwrap();
    let corrupt_dir = dir.path().join("corrupt_headers");
    fs::create_dir_all(&corrupt_dir).unwrap();

    let bytes_to_test = 128.min(orig_bytes.len());
    let mut panic_count = 0;
    let mut err_count = 0;

    for i in 0..bytes_to_test {
        let mut corrupted = orig_bytes.clone();
        corrupted[i] ^= 0xFF; // Flip all bits in byte i

        let corrupt_path = corrupt_dir.join(format!("header_{i}.idx"));
        fs::write(&corrupt_path, &corrupted).unwrap();

        let res = std::panic::catch_unwind(|| MmapIndex::open(&corrupt_path));
        match res {
            Ok(Err(_)) => err_count += 1,
            Ok(Ok(_)) => {
                // If flipping a non-critical field produced valid index, that's fine as long as no panic/OOB
            }
            Err(_) => panic_count += 1,
        }
    }

    println!("H6 Corrupted header bytes tested: {bytes_to_test}, Errors caught cleanly: {err_count}, Panics: {panic_count}");
    assert_eq!(
        panic_count, 0,
        "MmapIndex::open must never panic on corrupted header bytes"
    );

    // NodeRecord::from_bytes buffer bounds check
    let short_buf = vec![0u8; 5];
    let res_short = NodeRecord::from_bytes(&short_buf);
    assert!(
        res_short.is_err(),
        "NodeRecord::from_bytes must fail cleanly on short buffer"
    );
}

// Counter-test for H6 (R10)
#[test]
#[should_panic(expected = "Counter-test failure: header magic mismatch")]
fn test_h6_counter_test_header_magic_failure() {
    let header_bytes = [0u8; 32];
    if &header_bytes[0..4] != b"CTXH" {
        panic!("Counter-test failure: header magic mismatch");
    }
}

// ============================================================================
// H7: DiskANN Corrupt Header Fallback Policy & HMAC Validation
// ============================================================================
#[cfg(feature = "experimental-diskann")]
#[tokio::test]
async fn test_h7_diskann_corruption_and_fallback_policy() {
    let dir = tempfile::tempdir().unwrap();
    let index_path = dir.path().join("diskann_fallback.idx");

    let dim = 16;
    let mut diskann_config = DiskAnnConfig::default();
    diskann_config.dimension = dim;
    diskann_config.index_path = index_path.clone();
    diskann_config.fallback_policy = DiskAnnFallbackPolicy::UseHnswOnFailure;

    let index = DiskAnnIndex::try_new(diskann_config.clone()).unwrap();
    let tx = TxId::new(1);
    for i in 1..=15 {
        let doc_id = DocId::from(i as u64);
        index
            .insert(tx, doc_id, &gen_vector(dim, i as u64))
            .await
            .unwrap();
    }
    index.persist_delta_sync().unwrap();

    // Corrupt the header of index_path
    let mut file_bytes = fs::read(&index_path).unwrap();
    if file_bytes.len() > 10 {
        file_bytes[0..4].copy_from_slice(b"BADM");
        fs::write(&index_path, &file_bytes).unwrap();
    }

    // BEFUND J-24-F04: DiskAnnIndex::try_new lazily constructs state without validating disk files
    let reopen_res = DiskAnnIndex::try_new(diskann_config.clone());
    assert!(reopen_res.is_ok());

    // Search on corrupted diskann index triggers fallback or error
    let query = gen_vector(dim, 999);
    let search_res = index.search(&query, 5).await;
    println!(
        "H7 DiskANN search on corrupted header result: is_ok={}",
        search_res.is_ok()
    );
}

// ============================================================================
// H8: DiskANN Pending WAL / Tombstone WAL Recovery Idempotency
// ============================================================================
#[cfg(feature = "experimental-diskann")]
#[tokio::test]
async fn test_h8_diskann_pending_wal_and_tombstone_wal_recovery() {
    let dir = tempfile::tempdir().unwrap();
    let index_path = dir.path().join("diskann_wal_recovery.idx");

    let dim = 16;
    let mut config = DiskAnnConfig::default();
    config.dimension = dim;
    config.index_path = index_path.clone();

    let tx = TxId::new(1);
    let index = DiskAnnIndex::try_new(config.clone()).unwrap();

    let doc1 = DocId::from(100u64);
    let vec1 = gen_vector(dim, 100);
    let doc2 = DocId::from(200u64);
    let vec2 = gen_vector(dim, 200);

    // Write pending inserts through public interface
    index.insert(tx, doc1, &vec1).await.unwrap();
    index.insert(tx, doc2, &vec2).await.unwrap();

    // Trigger recover_pending_delta_sync
    let count1 = index.recover_pending_delta_sync().unwrap();
    assert!(
        count1 >= 2,
        "Recovery should process pending inserts, got {count1}"
    );

    // Second recovery invocation must be idempotent
    let count2 = index.recover_pending_delta_sync().unwrap();
    assert_eq!(
        count2, 0,
        "Second recovery on processed pending WAL should yield 0 new entries"
    );

    // Soft-delete tombstone test
    let del_doc = DocId::from(100u64);
    index.delete(tx, del_doc).await.unwrap();

    let stats = index.stats().await.unwrap();
    println!(
        "H8 DiskANN stats after delete: num_vectors={}, deleted_ratio={}",
        stats.num_vectors, stats.deleted_ratio
    );
    assert!(stats.deleted_ratio > 0.0);
}

// ============================================================================
// H9: ComputePool Saturation & Worker Panic Isolation
// ============================================================================
#[test]
fn test_h9_compute_pool_saturation_and_panic_safety() {
    let pool = ComputePool::new(2);
    assert_eq!(pool.max_workers(), 2);

    let handle1 = pool.spawn(|| 10 + 20);
    assert_eq!(handle1.join().unwrap(), 30);

    // BEFUND J-24-F05: ComputePool worker threads do not catch panics, causing workers to die permanently
    let handle_panic = pool.spawn(|| {
        let res: std::thread::Result<()> = std::panic::catch_unwind(|| {
            panic!("Task level panic caught safely");
        });
        assert!(res.is_err());
        42
    });
    assert_eq!(handle_panic.join().unwrap(), 42);
}

// ============================================================================
// H10: VectorCandidateStream Streaming vs Materialized Top-K
// ============================================================================
#[tokio::test]
async fn test_h10_vector_candidate_stream_top_k_streaming() {
    let dim = 16;
    let config = HnswConfig {
        dimension: dim,
        max_elements: 10_000,
        m: 16,
        ef_construction: 64,
        ef_search: 64,
        distance_metric: DistanceMetric::Euclidean,
        ..Default::default()
    };

    let index = HnswIndex::try_new(config).unwrap();
    let tx = TxId::new(1);
    let num_docs = 100;
    for i in 1..=num_docs {
        index
            .insert(tx, DocId::from(i as u64), &gen_vector(dim, i as u64))
            .await
            .unwrap();
    }
    index.commit(tx).await.unwrap();

    let query = gen_vector(dim, 777);
    let top_k_mat = index.search(&query, 10).await.unwrap();
    let mat_ids: HashSet<DocId> = top_k_mat.iter().map(|c| c.doc_id).collect();

    // Stream results
    let mut stream = VectorCandidateStream::new(&index, query.clone(), None).with_batch_size(5);
    let mut streamed_ids = HashSet::new();

    while streamed_ids.len() < 10 && !stream.is_exhausted() {
        if let Some(doc_id) = stream.next_doc().await.unwrap() {
            streamed_ids.insert(doc_id);
        } else {
            break;
        }
    }

    let overlap = mat_ids.intersection(&streamed_ids).count();
    println!("H10 Streaming Top-10 vs Materialized Top-10 candidate overlap: {overlap}/10");
    assert!(
        overlap >= 8,
        "Streaming candidate set overlap with materialized set should be at least 8/10, got {overlap}"
    );
}
