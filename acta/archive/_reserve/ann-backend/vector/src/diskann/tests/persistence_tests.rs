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
async fn test_load_node_rejects_corrupt_neighbor_count() {
    let temp_dir = tempfile::tempdir().unwrap(); // unwrap
    let index_path = temp_dir.path().join("corrupt_test.idx");

    let max_degree = 8;
    let dimension = 16;
    let config = DiskAnnConfig {
        index_path: index_path.clone(),
        dimension,
        max_degree,
        beam_width: 8,
        distance_metric: DistanceMetric::Euclidean,
        quantize: false,
        ..DiskAnnConfig::default()
    };

    let index = DiskAnnIndex::try_new(config.clone()).expect("valid config"); // expect
    let vectors = vec![vec![1.0f32; dimension]];
    let ids = vec![DocId::from(1u64)];

    index.build(&vectors, &ids).await.expect("Build failed"); // expect

    // Mutate neighbor_count of node 0 in the binary index file to be > max_degree
    let mut data = tokio::fs::read(&index_path).await.expect("read file"); // expect

    // Offset layout: sector_size (4096) + dimension * 4 bytes (64)
    let neighbor_count_offset = config.sector_size + (dimension * 4);
    let corrupt_count: u32 = (max_degree + 5) as u32;
    data[neighbor_count_offset..neighbor_count_offset + 4]
        .copy_from_slice(&corrupt_count.to_le_bytes());

    // Recompute HMAC for the modified payload so header & footer integrity passes, allowing load_node to test node parsing
    let mut hmac = crate::diskann::integrity::DiskAnnHmac::new(DISKANN_INTEGRITY_KEY).unwrap();
    let footer_start = data.len() - DiskAnnFooter::SIZE;
    hmac.update(&data[..footer_start]);
    let computed = hmac.finalize();
    data[footer_start + 4..].copy_from_slice(&computed);

    tokio::fs::write(&index_path, &data)
        .await
        .expect("write corrupt file"); // expect

    let reloaded_config = DiskAnnConfig {
        fallback_policy: DiskAnnFallbackPolicy::FailFast,
        ..config
    };
    let reloaded_index = DiskAnnIndex::try_new(reloaded_config).expect("valid config"); // expect
    reloaded_index.load().await.expect("Load header & mmap"); // expect

    let result = reloaded_index.load_node(0);
    assert!(result.is_err());
    let err_msg = result.err().unwrap().to_string(); // unwrap allowed (AGENT:03)
    assert!(
        err_msg.contains("Corrupt DiskANN node: neighbor_count 13 > max_degree 8"),
        "Unexpected error message: {}",
        err_msg
    );
}

#[tokio::test]
async fn test_load_rejects_bad_magic() {
    let temp_dir = tempfile::tempdir().unwrap(); // unwrap
    let index_path = temp_dir.path().join("bad_magic_test.idx");

    let config = DiskAnnConfig {
        index_path: index_path.clone(),
        dimension: 8,
        max_degree: 4,
        distance_metric: DistanceMetric::Euclidean,
        fallback_policy: DiskAnnFallbackPolicy::FailFast,
        ..DiskAnnConfig::default()
    };

    let index = DiskAnnIndex::try_new(config.clone()).expect("valid config"); // expect
    let vectors = vec![vec![1.0; 8]];
    let ids = vec![DocId::from(1u64)];
    index.build(&vectors, &ids).await.expect("build"); // expect

    // Mutate magic bytes
    let mut data = tokio::fs::read(&index_path).await.expect("read file"); // expect
    data[0..4].copy_from_slice(b"BADM");
    tokio::fs::write(&index_path, &data)
        .await
        .expect("write bad magic file"); // expect

    let reloaded_index = DiskAnnIndex::try_new(config).expect("valid config"); // expect
    let load_res = reloaded_index.load().await;
    assert!(load_res.is_err());
    let err_msg = load_res.err().unwrap().to_string(); // unwrap allowed (AGENT:03)
    assert!(
        err_msg.contains("Invalid DiskANN file: bad magic"),
        "Unexpected error message: {}",
        err_msg
    );
}

#[tokio::test]
async fn test_load_rejects_version_mismatch() {
    let temp_dir = tempfile::tempdir().unwrap(); // unwrap
    let index_path = temp_dir.path().join("version_mismatch_test.idx");

    let config = DiskAnnConfig {
        index_path: index_path.clone(),
        dimension: 8,
        max_degree: 4,
        distance_metric: DistanceMetric::Euclidean,
        fallback_policy: DiskAnnFallbackPolicy::FailFast,
        ..DiskAnnConfig::default()
    };

    let index = DiskAnnIndex::try_new(config.clone()).expect("valid config"); // expect
    let vectors = vec![vec![1.0; 8]];
    let ids = vec![DocId::from(1u64)];
    index.build(&vectors, &ids).await.expect("build"); // expect

    // Mutate version to 99
    let mut data = tokio::fs::read(&index_path).await.expect("read file"); // expect
    data[4..6].copy_from_slice(&99u16.to_le_bytes());
    tokio::fs::write(&index_path, &data)
        .await
        .expect("write bad version file"); // expect

    let reloaded_index = DiskAnnIndex::try_new(config).expect("valid config"); // expect
    let load_res = reloaded_index.load().await;
    assert!(load_res.is_err());
    let err_msg = load_res.err().unwrap().to_string(); // unwrap allowed (AGENT:03)
    assert!(
        err_msg.contains("DiskANN version mismatch"),
        "Unexpected error message: {}",
        err_msg
    );
}

#[tokio::test]
async fn test_write_to_file_uses_tmp_and_atomic_rename() {
    let temp_dir = tempfile::tempdir().unwrap(); // unwrap
    let index_path = temp_dir.path().join("atomic_save_test.idx");
    let tmp_path = index_path.with_extension("idx.tmp");

    let config = DiskAnnConfig {
        index_path: index_path.clone(),
        dimension: 8,
        max_degree: 4,
        sector_size: 4096,
        distance_metric: DistanceMetric::Euclidean,
        ..DiskAnnConfig::default()
    };

    let index = DiskAnnIndex::try_new(config).expect("valid config"); // expect
    let vectors = vec![vec![1.0; 8]];
    let ids = vec![DocId::from(1u64)];
    index.build(&vectors, &ids).await.expect("build"); // expect

    // After build completes, index_path must exist and .tmp must NOT exist
    assert!(
        index_path.exists(),
        "Final index file must exist after atomic rename"
    );
    assert!(
        !tmp_path.exists(),
        "Temporary file .tmp must be cleaned up / renamed"
    );
}

#[tokio::test]
async fn test_load_rejects_sector_size_mismatch() {
    let temp_dir = tempfile::tempdir().unwrap(); // unwrap allowed (AGENT:03)
    let index_path = temp_dir.path().join("sector_mismatch_test.idx");

    let build_config = DiskAnnConfig {
        index_path: index_path.clone(),
        dimension: 8,
        max_degree: 4,
        sector_size: 4096,
        distance_metric: DistanceMetric::Euclidean,
        ..DiskAnnConfig::default()
    };

    let index = DiskAnnIndex::try_new(build_config).expect("valid config"); // expect
    let vectors = vec![vec![1.0; 8]];
    let ids = vec![DocId::from(1u64)];
    index.build(&vectors, &ids).await.expect("build"); // expect

    let load_config = DiskAnnConfig {
        index_path,
        dimension: 8,
        max_degree: 4,
        sector_size: 2048,
        distance_metric: DistanceMetric::Euclidean,
        fallback_policy: DiskAnnFallbackPolicy::FailFast,
        ..DiskAnnConfig::default()
    };

    let reloaded_index = DiskAnnIndex::try_new(load_config).expect("valid config"); // expect
    let load_res = reloaded_index.load().await;
    assert!(load_res.is_err());
    let err_msg = load_res.err().unwrap().to_string(); // unwrap allowed (AGENT:03)
    assert!(
            err_msg.contains("DiskANN-Index inkompatibel: Config-sector_size=2048 stimmt nicht mit Header-sector_size=4096 überein"),
            "Unexpected error message: {}",
            err_msg
        );
}

#[tokio::test]
async fn test_diskann_footer_roundtrip_hmac_integrity() {
    let temp_dir = tempfile::tempdir().unwrap();
    let index_path = temp_dir.path().join("footer_roundtrip.idx");

    let config = DiskAnnConfig {
        index_path: index_path.clone(),
        dimension: 8,
        max_degree: 4,
        sector_size: 4096,
        distance_metric: DistanceMetric::Euclidean,
        fallback_policy: DiskAnnFallbackPolicy::FailFast,
        ..DiskAnnConfig::default()
    };

    let index = DiskAnnIndex::try_new(config.clone()).unwrap();
    let vectors = vec![vec![1.0; 8]];
    let ids = vec![DocId::from(42u64)];
    index.build(&vectors, &ids).await.unwrap();

    // 1. Verify index file contains a valid DiskAnnFooter at the end
    let file_bytes = tokio::fs::read(&index_path).await.unwrap();
    assert!(file_bytes.len() >= DiskAnnFooter::SIZE);
    let footer_slice = &file_bytes[file_bytes.len() - DiskAnnFooter::SIZE..];
    let footer = DiskAnnFooter::try_from_bytes(footer_slice).expect("footer parsing");
    assert_eq!(&footer.magic, DISKANN_FOOTER_MAGIC);

    // Verify HMAC calculation over payload
    let mut hmac = crate::diskann::integrity::DiskAnnHmac::new(DISKANN_INTEGRITY_KEY).unwrap();
    let payload = &file_bytes[..file_bytes.len() - DiskAnnFooter::SIZE];
    hmac.update(payload);
    let expected_hmac = hmac.finalize();
    assert_eq!(footer.hmac, expected_hmac);

    // 2. Corrupt one byte of HMAC in footer, write to disk, and verify load returns integrity error
    let mut corrupt_bytes = file_bytes.clone();
    let last_idx = corrupt_bytes.len() - 1;
    corrupt_bytes[last_idx] ^= 0xFF;
    tokio::fs::write(&index_path, &corrupt_bytes)
        .await
        .expect("write corrupt footer file");

    let reloaded_index = DiskAnnIndex::try_new(config).expect("valid config");
    let load_res = reloaded_index.load().await;
    assert!(
        load_res.is_err(),
        "Loading corrupted HMAC footer must return Err"
    );
    let err_msg = load_res.err().unwrap().to_string();
    assert!(
        err_msg.contains("HMAC integrity validation failed"),
        "Unexpected error message: {}",
        err_msg
    );
}
