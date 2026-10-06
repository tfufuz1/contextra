//! Integration test for DiskANN pending.wal crash idempotency (P03 F-04 HIGH).

#![cfg(feature = "experimental-diskann")]

use contextra_core::{DistanceMetric, DocId, Result, VectorIndex};
use contextra_vector::diskann::{DiskAnnConfig, DiskAnnIndex};

#[tokio::test]
async fn test_p03d_diskann_pending_wal_crash_idempotency() -> Result<()> {
    let temp_dir = tempfile::tempdir().map_err(contextra_core::ContextraError::Io)?;
    let index_path = temp_dir.path().join("p03d_test.idx");

    let config = DiskAnnConfig {
        index_path: index_path.clone(),
        dimension: 4,
        max_degree: 16,
        beam_width: 32,
        sector_size: 4096,
        distance_metric: DistanceMetric::Euclidean,
        quantize: false,
        pending_flush_threshold: Some(100), // high threshold so auto-persist doesn't trigger automatically
        ..DiskAnnConfig::default()
    };

    let index = DiskAnnIndex::try_new(config.clone())?;

    // 1. Initial build with base vectors (DocIds 1, 2)
    let base_vecs = vec![vec![1.0, 0.0, 0.0, 0.0], vec![2.0, 0.0, 0.0, 0.0]];
    let base_ids = vec![DocId::from(1u64), DocId::from(2u64)];
    index.build(&base_vecs, &base_ids).await?;

    // 2. Insert pending vector (DocId 3)
    let new_vec = vec![3.0, 0.0, 0.0, 0.0];
    let new_id = DocId::from(3u64);
    index
        .insert(contextra_core::TxId(1), new_id, &new_vec)
        .await?;

    // Verify pending.wal exists
    let pending_wal_path = index_path.with_extension("pending.wal");
    assert!(
        pending_wal_path.exists(),
        "pending.wal should exist after insert"
    );

    // 3. Persist delta synchronously
    index.persist_delta().await?;

    // Now index_path contains base_vecs + new_vec (DocIds 1, 2, 3).
    // And pending.wal was removed by persist_delta().

    // 4. Fault Injection: Simulate crash after rename but before remove_file
    // Re-create pending.wal containing DocId 3 using insert on a fresh un-loaded index instance
    let dummy_index = DiskAnnIndex::try_new(config.clone())?;
    dummy_index
        .insert(contextra_core::TxId(2), new_id, &new_vec)
        .await?;
    assert!(
        pending_wal_path.exists(),
        "pending.wal recreated for fault injection"
    );

    // 5. Reload index from disk (simulating process restart)
    let reloaded_index = DiskAnnIndex::try_new(config.clone())?;
    reloaded_index.load().await?;

    // Force persist_delta on reloaded_index
    reloaded_index.persist_delta().await?;

    // 6. Assertions: Check doc_ids and count
    let all_ids = reloaded_index.all_doc_ids().await?;
    assert_eq!(
        all_ids.len(),
        3,
        "Expected exactly 3 unique vectors, got {}: {:?}",
        all_ids.len(),
        all_ids
    );

    let doc3_count = all_ids.iter().filter(|&&id| id == new_id).count();
    assert_eq!(doc3_count, 1, "DocId 3 should be present exactly once");

    // Verify search returns 3 documents
    let results = reloaded_index.search(&vec![3.0, 0.0, 0.0, 0.0], 10).await?;
    assert_eq!(results.len(), 3, "Search should return exactly 3 results");

    // Verify pending.wal is deleted after persist
    assert!(
        !pending_wal_path.exists(),
        "pending.wal should be cleaned up after recovery and persist"
    );

    Ok(())
}
