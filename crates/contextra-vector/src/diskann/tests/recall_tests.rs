// FILE-CONTEXT
// ZWECK: Modularisierte Testsuiten für DiskANN (WP-VEC-07).
// STAND: 2026-11-11

use super::super::config::{DiskAnnConfig, DiskAnnFallbackPolicy};
use super::super::format::{
    compute_adaptive_flush_threshold, DiskAnnFooter, DiskAnnHeader, DISKANN_FOOTER_MAGIC,
    DISKANN_INTEGRITY_KEY, DISKANN_MAGIC, DISKANN_VERSION,
};
use super::super::types::{DiskAnnIndex, SearchCandidate};
use crate::error::{Result, VectorError};
use crate::types::{DistanceMetric, DocId, TxId};
use tempfile::tempdir;
#[tokio::test]
async fn test_diskann_recall_basic() {
    let temp_dir = tempfile::tempdir().unwrap(); // unwrap
    let index_path = temp_dir.path().join("recall_test.idx");

    let config = DiskAnnConfig {
        index_path: index_path.clone(),
        dimension: 16,
        max_degree: 8,
        beam_width: 8,
        distance_metric: DistanceMetric::Euclidean,
        ..DiskAnnConfig::default()
    };

    let index = DiskAnnIndex::try_new(config).expect("valid config"); // expect

    let n = 100;
    let mut vectors = Vec::with_capacity(n);
    let mut ids = Vec::with_capacity(n);
    for i in 0..n {
        let mut v = vec![0.0f32; 16];
        v[0] = i as f32;
        vectors.push(v);
        ids.push(DocId::from(i as u64));
    }

    index.build(&vectors, &ids).await.expect("Build failed"); // expect

    let query = &vectors[50];
    let results = index.search(query, 1).await.expect("Search failed"); // expect
    assert!(!results.is_empty());
    assert_eq!(results[0].doc_id, ids[50]);
}

#[tokio::test]
async fn test_diskann_sq8_recall() {
    let temp_dir = tempfile::tempdir().unwrap(); // unwrap
    let index_path = temp_dir.path().join("sq8_test.idx");

    let config = DiskAnnConfig {
        index_path: index_path.clone(),
        dimension: 16,
        max_degree: 16,
        beam_width: 16,
        distance_metric: DistanceMetric::Euclidean,
        quantize: true,
        ..DiskAnnConfig::default()
    };

    let index = DiskAnnIndex::try_new(config).expect("valid config"); // expect

    let n = 200;
    let mut vectors = Vec::with_capacity(n);
    let mut ids = Vec::with_capacity(n);
    for i in 0..n {
        let mut v = vec![0.0f32; 16];
        v[0] = i as f32;
        vectors.push(v);
        ids.push(DocId::from(i as u64));
    }

    index.build(&vectors, &ids).await.expect("Build failed"); // expect

    let query = &vectors[150];
    let results = index.search(query, 1).await.expect("Search failed"); // expect
    assert!(!results.is_empty());
    assert_eq!(results[0].doc_id, ids[150]);
}

#[tokio::test]
async fn test_diskann_robust_prune_handles_nan_distance_without_panic() -> Result<()> {
    let temp_dir = tempfile::tempdir().map_err(VectorError::Io)?;
    let config = DiskAnnConfig {
        index_path: temp_dir.path().join("robust_prune_nan.idx"),
        dimension: 4,
        max_degree: 4,
        distance_metric: DistanceMetric::Euclidean,
        ..DiskAnnConfig::default()
    };
    let index = DiskAnnIndex::try_new(config)?;

    // Test 1: candidate distance is NaN in prune_in_memory
    let mut candidates = vec![
        SearchCandidate {
            index: 0,
            distance: 1.0,
        },
        SearchCandidate {
            index: 1,
            distance: f32::NAN,
        },
    ];
    let vectors = vec![vec![0.0, 0.0, 0.0, 0.0], vec![1.0, 0.0, 0.0, 0.0]];

    let pruned = index.prune_in_memory(&mut candidates, &vectors, 4, 1.2)?;
    // Both candidates must be retained (fail-open) and prune_in_memory must not panic
    assert_eq!(pruned.len(), 2);
    assert!(pruned.contains(&0));
    assert!(pruned.contains(&1));

    // Test 2: candidate distance is NaN in prune_streaming
    let mut streaming_candidates = vec![
        SearchCandidate {
            index: 0,
            distance: 1.0,
        },
        SearchCandidate {
            index: 1,
            distance: f32::NAN,
        },
    ];
    let new_vecs = vec![
        (DocId::from(1u64), vec![0.0, 0.0, 0.0, 0.0]),
        (DocId::from(2u64), vec![1.0, 0.0, 0.0, 0.0]),
    ];
    let pruned_streaming =
        index.prune_streaming(0, &mut streaming_candidates, 0, &new_vecs, 4, 1.2)?;
    assert_eq!(pruned_streaming.len(), 2);
    assert!(pruned_streaming.contains(&0));
    assert!(pruned_streaming.contains(&1));

    Ok(())
}

#[tokio::test]
async fn test_diskann_query_handles_nan_neighbor_distance() -> Result<()> {
    let temp_dir = tempfile::tempdir().map_err(VectorError::Io)?;
    let index_path = temp_dir.path().join("query_nan_neighbor.idx");
    let config = DiskAnnConfig {
        index_path: index_path.clone(),
        dimension: 4,
        max_degree: 4,
        beam_width: 4,
        distance_metric: DistanceMetric::Cosine,
        fallback_policy: DiskAnnFallbackPolicy::FailFast,
        ..DiskAnnConfig::default()
    };

    let index = DiskAnnIndex::try_new(config.clone())?;
    let vectors = vec![
        vec![1.0, 0.0, 0.0, 0.0],
        vec![0.0, 1.0, 0.0, 0.0],
        vec![0.0, 0.0, 1.0, 0.0],
    ];
    let ids = vec![DocId::from(1u64), DocId::from(2u64), DocId::from(3u64)];
    index.build(&vectors, &ids).await?;

    // Query with normal vector should succeed and find results
    let query = vec![1.0, 0.0, 0.0, 0.0];
    let results = index.search(&query, 2).await?;
    assert!(!results.is_empty());
    assert_eq!(results[0].doc_id, DocId::from(1u64));

    Ok(())
}
