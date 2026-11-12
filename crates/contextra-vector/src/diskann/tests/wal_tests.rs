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
async fn test_pending_wal_roundtrip_hmac() -> Result<()> {
    let temp_dir = tempfile::tempdir().map_err(VectorError::Io)?;
    let wal_path = temp_dir.path().join("test_roundtrip.pending.wal");

    let doc_id = DocId::from(42u64);
    let embedding = vec![1.0f32, 2.0, 3.0, 4.0];

    DiskAnnIndex::append_to_pending_wal(&wal_path, doc_id, &embedding).await?;
    let recovered = DiskAnnIndex::read_pending_wal(&wal_path)?;

    assert_eq!(recovered.len(), 1);
    assert_eq!(recovered[0].0, doc_id);
    assert_eq!(recovered[0].1, embedding);

    Ok(())
}

#[tokio::test]
async fn test_pending_wal_corrupt_payload_skips_entry() -> Result<()> {
    let temp_dir = tempfile::tempdir().map_err(VectorError::Io)?;
    let wal_path = temp_dir.path().join("test_corrupt.pending.wal");

    let doc1 = DocId::from(100u64);
    let vec1 = vec![1.0f32, 1.1, 1.2, 1.3];
    let doc2 = DocId::from(200u64);
    let vec2 = vec![2.0f32, 2.1, 2.2, 2.3];

    DiskAnnIndex::append_to_pending_wal(&wal_path, doc1, &vec1).await?;
    DiskAnnIndex::append_to_pending_wal(&wal_path, doc2, &vec2).await?;

    // Corrupt a byte in the embedding payload of entry 1
    let mut data = tokio::fs::read(&wal_path).await.map_err(VectorError::Io)?;
    // Header is 5 bytes. Entry 1 starts at byte 5.
    // ID: 8 bytes, Dim: 4 bytes -> Embedding starts at byte 5 + 12 = 17.
    data[18] ^= 0xFF;
    tokio::fs::write(&wal_path, &data)
        .await
        .map_err(VectorError::Io)?;

    let recovered = DiskAnnIndex::read_pending_wal(&wal_path)?;

    // Entry 1 should be skipped due to HMAC mismatch, Entry 2 recovered
    assert_eq!(recovered.len(), 1);
    assert_eq!(recovered[0].0, doc2);
    assert_eq!(recovered[0].1, vec2);

    Ok(())
}

#[tokio::test]
async fn test_pending_wal_absurd_dim_prevents_allocation() -> Result<()> {
    let temp_dir = tempfile::tempdir().map_err(VectorError::Io)?;
    let wal_path = temp_dir.path().join("test_absurd_dim.pending.wal");

    let mut data = Vec::new();
    data.extend_from_slice(PENDING_WAL_MAGIC);
    data.push(PENDING_WAL_VERSION);

    // Entry with ID=1, Dim=0xFFFFFFF0 (~17GB allocation attempt without check)
    data.extend_from_slice(&1u64.to_le_bytes());
    let absurd_dim: u32 = 0xFFFFFFF0;
    data.extend_from_slice(&absurd_dim.to_le_bytes());

    tokio::fs::write(&wal_path, &data)
        .await
        .map_err(VectorError::Io)?;

    let recovered = DiskAnnIndex::read_pending_wal(&wal_path)?;

    // Must break gracefully without OOM or panic and return empty list
    assert!(recovered.is_empty());

    Ok(())
}

#[tokio::test]
async fn test_pending_wal_rejects_legacy_unversioned_file() -> Result<()> {
    let temp_dir = tempfile::tempdir().map_err(VectorError::Io)?;
    let wal_path = temp_dir.path().join("legacy.pending.wal");

    // Legacy format: raw u64 id, u32 dim, f32s without PWAL magic header
    let mut data = Vec::new();
    data.extend_from_slice(&1u64.to_le_bytes());
    data.extend_from_slice(&2u32.to_le_bytes());
    data.extend_from_slice(&1.0f32.to_le_bytes());
    data.extend_from_slice(&2.0f32.to_le_bytes());

    tokio::fs::write(&wal_path, &data)
        .await
        .map_err(VectorError::Io)?;

    let result = DiskAnnIndex::read_pending_wal(&wal_path);
    assert!(result.is_err());
    let err_str = result.err().unwrap().to_string();
    assert!(
        err_str.contains("Unbekanntes oder veraltetes pending.wal-Format"),
        "Error string was: {}",
        err_str
    );

    Ok(())
}
