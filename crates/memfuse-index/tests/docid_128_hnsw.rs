use memfuse_core::{DistanceMetric, DocId, MemFuseError, TxId, VectorIndex};
use memfuse_index::hnsw::{HnswConfig, HnswIndex};
use memfuse_index::persistence::MmapIndex;

#[tokio::test]
async fn test_docid_128_hnsw_basic_roundtrip() -> memfuse_core::Result<()> {
    let temp_dir = tempfile::tempdir().map_err(|e| MemFuseError::Storage(e.to_string()))?;
    let path = temp_dir.path().join("test_docid_128.hnsw");

    let config = HnswConfig {
        dimension: 4,
        m: 16,
        distance_metric: DistanceMetric::Euclidean,
        ..Default::default()
    };
    let index = HnswIndex::try_new(config.clone())?;
    let tx = TxId::new(1);

    let doc_id = DocId::from_key("test_key_128_bit").expect("valid key");
    let vec = vec![1.0, 2.0, 3.0, 4.0];

    index.insert(tx, doc_id, &vec).await?;
    index.commit(tx).await?;

    index.save(&path).await?;

    let mmap_index = MmapIndex::open(&path)?;
    assert_eq!(mmap_index.header.node_count(), 1);

    let record = mmap_index.get_node_record(0)?;
    assert_eq!(record.doc_id, doc_id.inner());

    let loaded_index = HnswIndex::try_new(config)?;
    loaded_index.load_mmap(&path).await?;

    let search_results = loaded_index.search(&vec, 1).await?;
    assert_eq!(search_results.len(), 1);
    assert_eq!(search_results[0].doc_id, doc_id);

    Ok(())
}

#[tokio::test]
async fn test_docid_format_mismatch_detection() -> memfuse_core::Result<()> {
    let temp_dir = tempfile::tempdir().map_err(|e| MemFuseError::Storage(e.to_string()))?;
    let path = temp_dir.path().join("mismatch.hnsw");

    // Manually create a file with fake NodeRecord layout
    let config = HnswConfig {
        dimension: 4,
        m: 16,
        distance_metric: DistanceMetric::Euclidean,
        ..Default::default()
    };
    let index = HnswIndex::try_new(config)?;
    let tx = TxId::new(1);
    index
        .insert(tx, DocId::from(42u64), &[1.0, 2.0, 3.0, 4.0])
        .await?;
    index.commit(tx).await?;
    index.save(&path).await?;

    let mut bytes = std::fs::read(&path).map_err(|e| MemFuseError::Storage(e.to_string()))?;

    // Corrupt record 0 vector_offset field in bytes (for NodeRecord at offset 80:
    // doc_id is 0..8 or 0..16, max_layer is at 8 or 16, vector_offset is at 9..17 or 17..25)
    let vec_offset_start = 80 + std::mem::size_of::<DocId>() + 1;
    let bad_vector_offset = 9999u64;
    bytes[vec_offset_start..vec_offset_start + 8].copy_from_slice(&bad_vector_offset.to_le_bytes());

    std::fs::write(&path, &bytes).map_err(|e| MemFuseError::Storage(e.to_string()))?;

    let res = MmapIndex::open(&path);
    assert!(res.is_err(), "Expected storage error on format mismatch");
    if let Err(MemFuseError::Storage(msg)) = res {
        assert!(
            msg.contains("mismatch") || msg.contains("size"),
            "Unexpected error message: {}",
            msg
        );
    } else {
        panic!("Expected Storage error");
    }

    Ok(())
}
