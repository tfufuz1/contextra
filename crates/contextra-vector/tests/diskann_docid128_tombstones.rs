#![forbid(unsafe_code)]

#[cfg(feature = "experimental-diskann")]
use contextra_ports::vector_index::VectorIndex;
#[cfg(feature = "experimental-diskann")]
use contextra_types::{ContextraError, DocId, Result, TxId};
#[cfg(feature = "experimental-diskann")]
use contextra_vector::diskann::{DiskAnnConfig, DiskAnnIndex};
#[cfg(feature = "experimental-diskann")]
use tempfile::tempdir;

#[cfg(feature = "experimental-diskann")]
#[tokio::test]
async fn test_diskann_docid128_tombstones_handling() -> Result<()> {
    let dir = tempdir().map_err(contextra_types::ContextraError::Io)?;
    let index_path = dir.path().join("docid128_tombstones.idx");

    let config = DiskAnnConfig {
        index_path: index_path.clone(),
        dimension: 4,
        max_degree: 8,
        beam_width: 16,
        ..Default::default()
    };

    let index = DiskAnnIndex::try_new(config.clone())?;

    // Construct two 128-bit IDs that differ strictly in the upper 64 bits
    #[cfg(feature = "docid-128")]
    let id1 = DocId::from(0x1_0000_0000_0000_0000_u128 | 42_u128);
    #[cfg(feature = "docid-128")]
    let id2 = DocId::from(0x2_0000_0000_0000_0000_u128 | 42_u128);

    #[cfg(not(feature = "docid-128"))]
    let id1 = DocId::from(42_u64);
    #[cfg(not(feature = "docid-128"))]
    let id2 = DocId::from(1042_u64);

    let v1 = vec![1.0, 0.0, 0.0, 0.0];
    let v2 = vec![0.0, 1.0, 0.0, 0.0];

    index.build(&[v1, v2], &[id1, id2]).await?;

    let all_before = index.all_doc_ids().await?;
    assert_eq!(all_before.len(), 2);
    assert!(all_before.contains(&id1));
    assert!(all_before.contains(&id2));

    // Delete id1 only
    index.delete(TxId(1), id1).await?;

    let all_after = index.all_doc_ids().await?;
    assert_eq!(all_after.len(), 1);
    assert!(!all_after.contains(&id1));
    assert!(all_after.contains(&id2));

    // Repeating delete on id1 returns NotFound (idempotent / already deleted)
    let re_delete_res = index.delete(TxId(2), id1).await;
    assert!(matches!(re_delete_res, Err(ContextraError::NotFound(_))));

    // Deleting unknown ID returns NotFound
    #[cfg(feature = "docid-128")]
    let unknown_id = DocId::from(0x9_0000_0000_0000_0000_u128 | 999_u128);
    #[cfg(not(feature = "docid-128"))]
    let unknown_id = DocId::from(9999_u64);

    let delete_unknown_res = index.delete(TxId(3), unknown_id).await;
    assert!(matches!(delete_unknown_res, Err(ContextraError::NotFound(_))));

    // Tombstone persistence after delta persist & reload
    index.persist_delta().await?;

    let reloaded_index = DiskAnnIndex::try_new(config)?;
    reloaded_index.load().await?;

    let all_reloaded = reloaded_index.all_doc_ids().await?;
    assert_eq!(all_reloaded.len(), 1);
    assert!(!all_reloaded.contains(&id1));
    assert!(all_reloaded.contains(&id2));

    Ok(())
}
