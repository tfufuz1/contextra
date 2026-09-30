#![forbid(unsafe_code)]

use contextra_core::{DocId, Result, VectorIndex};
use contextra_vector::diskann::{DiskAnnConfig, DiskAnnIndex};
use tempfile::tempdir;

#[tokio::test]
async fn test_diskann_docid128_tombstones_handling() -> Result<()> {
    let dir = tempdir().map_err(contextra_core::ContextraError::Io)?;
    let index_path = dir.path().join("docid128_tombstones.idx");

    let config = DiskAnnConfig {
        index_path: index_path.clone(),
        dimension: 4,
        max_degree: 8,
        beam_width: 16,
        ..Default::default()
    };

    let index = DiskAnnIndex::try_new(config)?;

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
    index.delete(contextra_core::TxId(1), id1).await?;

    let all_after = index.all_doc_ids().await?;
    assert_eq!(all_after.len(), 1);
    assert!(!all_after.contains(&id1));
    assert!(all_after.contains(&id2));

    Ok(())
}
