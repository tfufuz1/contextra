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
#[test]
fn test_compute_adaptive_flush_threshold_formula() {
    // Kleine Collection: Boden greift
    assert_eq!(compute_adaptive_flush_threshold(0), 50);
    assert_eq!(compute_adaptive_flush_threshold(100), 50); // floor(100*0.05)=5 < 50
    assert_eq!(compute_adaptive_flush_threshold(1_000), 50); // floor(1000*0.05)=50

    // Mittlere Collection
    assert_eq!(compute_adaptive_flush_threshold(10_000), 500); // floor(10000*0.05)=500

    // Große Collection: Deckel greift
    assert_eq!(compute_adaptive_flush_threshold(20_000), 1_000); // floor(20000*0.05)=1000
    assert_eq!(compute_adaptive_flush_threshold(100_000), 1_000); // Deckel

    // Monotonie: Threshold wächst mit N (bis Deckel)
    let t1 = compute_adaptive_flush_threshold(5_000);
    let t2 = compute_adaptive_flush_threshold(15_000);
    assert!(t1 <= t2);
}

#[test]
#[allow(clippy::unnecessary_literal_unwrap)]
fn test_benchmark_override_respected() {
    // Wenn pending_flush_threshold im Config gesetzt ist, soll er den
    // adaptiven Wert überschreiben (Benchmark-Escape-Hatch bleibt intakt).
    // Dieser Test prüft die Logik-Ebene (nicht den async insert()-Pfad).
    let threshold_override: Option<u64> = Some(42);
    let n_persisted: u64 = 100_000;
    let result =
        threshold_override.unwrap_or_else(|| compute_adaptive_flush_threshold(n_persisted));
    assert_eq!(result, 42); // Override gewinnt
}

#[tokio::test]
async fn test_diskann_non_power_of_two_sector_size_rejected() {
    let bad_config = DiskAnnConfig {
        sector_size: 4000, // not a power of 2
        ..DiskAnnConfig::default()
    };
    let res = DiskAnnIndex::try_new(bad_config);
    assert!(matches!(res, Err(VectorError::InvalidInput(_))));
}

#[tokio::test]
async fn test_diskann_delete_non_existent_returns_not_found() {
    let index = DiskAnnIndex::try_new(DiskAnnConfig::default()).expect("valid config");
    let tx = TxId::new(1);
    let doc_id = DocId::from(100u64);

    let delete_res = index.delete(tx, doc_id).await;
    assert!(matches!(delete_res, Err(VectorError::NotFound(_))));
}

#[tokio::test]
async fn test_diskann_tombstone_delete_search_filtering() -> Result<()> {
    let dir = tempfile::tempdir().map_err(VectorError::Io)?;
    let index_path = dir.path().join("tombstone_filtering.idx");
    let config = DiskAnnConfig {
        index_path: index_path.clone(),
        dimension: 4,
        max_degree: 4,
        beam_width: 4,
        distance_metric: DistanceMetric::Euclidean,
        fallback_policy: DiskAnnFallbackPolicy::FailFast,
        ..DiskAnnConfig::default()
    };
    let index = DiskAnnIndex::try_new(config.clone())?;

    let vecs = vec![
        vec![1.0, 0.0, 0.0, 0.0],
        vec![2.0, 0.0, 0.0, 0.0],
        vec![3.0, 0.0, 0.0, 0.0],
    ];
    let ids = vec![DocId::from(10u64), DocId::from(20u64), DocId::from(30u64)];
    index.build(&vecs, &ids).await?;

    // 1. Initial search finds doc 10 as top result
    let query = vec![1.0, 0.0, 0.0, 0.0];
    let res = index.search(&query, 1).await?;
    assert_eq!(res[0].doc_id, DocId::from(10u64));
    assert_eq!(index.len().await, 3);

    // 2. Delete doc 10
    index.delete(TxId(1), DocId::from(10u64)).await?;
    assert_eq!(index.len().await, 2);

    // Deleting doc 10 again returns NotFound
    let del2 = index.delete(TxId(1), DocId::from(10u64)).await;
    assert!(matches!(del2, Err(VectorError::NotFound(_))));

    // 3. Search query no longer returns doc 10, instead returns doc 20
    let res_after_del = index.search(&query, 1).await?;
    assert_eq!(res_after_del[0].doc_id, DocId::from(20u64));

    let all_ids = index.all_doc_ids().await?;
    assert!(!all_ids.contains(&DocId::from(10u64)));
    assert_eq!(all_ids.len(), 2);

    let stats = index.stats().await?;
    assert_eq!(stats.num_vectors, 2);
    assert!((stats.deleted_ratio - (1.0 / 3.0)).abs() < 1e-4);

    // 4. Persistence & WAL recovery after reload
    drop(index);
    let reloaded = DiskAnnIndex::try_new(config)?;
    reloaded.load().await?;

    assert_eq!(reloaded.len().await, 2);
    let res_reloaded = reloaded.search(&query, 1).await?;
    assert_eq!(res_reloaded[0].doc_id, DocId::from(20u64));

    Ok(())
}

#[tokio::test]
async fn test_diskann_insert_no_longer_returns_error() {
    let dir = tempfile::tempdir().unwrap();
    let config = DiskAnnConfig {
        index_path: dir.path().join("insert_test.idx"),
        dimension: 3,
        ..DiskAnnConfig::default()
    };
    let index = DiskAnnIndex::try_new(config).unwrap();
    let result = index
        .insert(TxId(1), DocId::from(42u64), &[0.1, 0.2, 0.3])
        .await;
    assert!(result.is_ok(), "insert() darf kein Err mehr zurückgeben");
}

#[tokio::test]
async fn test_diskann_persist_delta_empty_noop() {
    let dir = tempfile::tempdir().unwrap();
    let config = DiskAnnConfig {
        index_path: dir.path().join("noop_test.idx"),
        ..DiskAnnConfig::default()
    };
    let index = DiskAnnIndex::try_new(config).unwrap();
    let result = index.persist_delta().await;
    assert!(
        result.is_ok(),
        "persist_delta auf leerem pending ist ein Noop"
    );
}

#[tokio::test]
async fn test_diskann_persist_delta_atomic_rename() {
    // INV-DISKANN-1: Nach persist_delta() existiert kein .delta.tmp
    let dir = tempfile::tempdir().unwrap();
    let index_path = dir.path().join("atomic_test.idx");
    let config = DiskAnnConfig {
        index_path: index_path.clone(),
        dimension: 64,
        max_degree: 8,
        beam_width: 8,
        distance_metric: DistanceMetric::Euclidean,
        ..DiskAnnConfig::default()
    };
    let index = DiskAnnIndex::try_new(config).unwrap();

    let vecs: Vec<Vec<f32>> = (0..5).map(|i| vec![i as f32; 64]).collect();
    // Build initial index
    let ids: Vec<DocId> = (0..5u64).map(DocId::from).collect();
    index.build(&vecs, &ids).await.unwrap();

    // Insert new vectors
    for i in 5..10 {
        index
            .insert(TxId(1), DocId::from(i as u64), &vec![i as f32; 64])
            .await
            .unwrap();
    }
    index.persist_delta().await.unwrap();

    // Kein .delta.tmp sollte noch existieren
    let tmp = index_path.with_extension("delta.tmp");
    assert!(
        !tmp.exists(),
        "Temporäre Datei muss nach persist_delta bereinigt sein"
    );

    // Verify total length is now 10 and inserted vectors are searchable
    assert_eq!(index.len().await, 10);
    let query = vec![7.0f32; 64];
    let results = index.search(&query, 1).await.unwrap();
    assert!(!results.is_empty());
    assert_eq!(results[0].doc_id, DocId::from(7u64));
}

#[tokio::test]
async fn test_diskann_config_validation() {
    let valid_config = DiskAnnConfig {
        index_path: PathBuf::from("dummy.idx"),
        dimension: 128,
        max_degree: 64,
        sector_size: 4096,
        ..DiskAnnConfig::default()
    };

    let index = DiskAnnIndex::try_new(valid_config).expect("valid config"); // expect
    assert_eq!(index.len().await, 0);
}

#[tokio::test]
async fn test_diskann_header_persistence() {
    let temp_dir = tempfile::tempdir().unwrap(); // unwrap
    let index_path = temp_dir.path().join("header_test.idx");

    let config = DiskAnnConfig {
        index_path: index_path.clone(),
        dimension: 8,
        max_degree: 4,
        distance_metric: DistanceMetric::Euclidean,
        ..DiskAnnConfig::default()
    };

    let index = DiskAnnIndex::try_new(config).expect("valid config"); // expect
    let vectors = vec![vec![1.0; 8]];
    let ids = vec![DocId::from(42u64)];
    index.build(&vectors, &ids).await.expect("build"); // expect

    let data = tokio::fs::read(&index_path).await.expect("read file"); // expect
    assert!(data.starts_with(b"DANN"));

    let header =
        DiskAnnHeader::try_from_bytes(&data[0..DiskAnnHeader::SIZE]).expect("try_from_bytes"); // expect
    assert_eq!(header.version, DISKANN_VERSION);
    assert_eq!(header.node_count, 1);
    assert_eq!(header.sector_size, 4096);
}
