use contextra_core::{ContextraError, DocId, IndexOp, Result, TxId, VectorIndex};
use contextra_vector::hnsw::{HnswConfig, HnswIndex};

fn create_test_index(dim: usize) -> Result<HnswIndex> {
    let config = HnswConfig {
        dimension: dim,
        m: 16,
        ef_construction: 64,
        ef_search: 64,
        ..Default::default()
    };
    HnswIndex::try_new(config)
}

#[tokio::test]
async fn test_hnsw_delete_and_commit_clean_propagation() -> Result<()> {
    let dim = 4;
    let index = create_test_index(dim)?;
    let tx1 = TxId::new(1);
    let doc1 = DocId::new(42);
    let vec1 = vec![1.0, 0.0, 0.0, 0.0];

    // Insert doc1, commit
    index.insert(tx1, doc1, &vec1).await?;
    index.commit(tx1).await?;
    assert_eq!(index.len().await, 1);

    // Delete doc1, commit
    let tx2 = TxId::new(2);
    index.delete(tx2, doc1).await?;
    index.commit(tx2).await?;
    assert_eq!(index.len().await, 0);

    let search_res = index.search(&vec1, 10).await?;
    assert!(search_res.is_empty());

    Ok(())
}

#[tokio::test]
async fn test_hnsw_delete_error_propagation_on_invalid_config() -> Result<()> {
    // Construct index with invalid configuration (dimension = 0)
    let invalid_config = HnswConfig {
        dimension: 0,
        m: 16,
        ef_construction: 64,
        ef_search: 64,
        ..Default::default()
    };

    #[allow(deprecated)]
    let index = HnswIndex::new(invalid_config);
    let tx1 = TxId::new(1);
    let doc1 = DocId::new(100);

    // Attempting delete on an index with a validation error must return Err without panicking
    let delete_res = index.delete(tx1, doc1).await;
    assert!(
        delete_res.is_err(),
        "delete() on invalid index config must propagate error"
    );

    let commit_res = index.commit(tx1).await;
    assert!(
        commit_res.is_err(),
        "commit() on invalid index config must propagate error"
    );

    Ok(())
}

#[tokio::test]
async fn test_hnsw_commit_handles_phase1_validation_error_without_panic() -> Result<()> {
    let dim = 4;
    let index = create_test_index(dim)?;
    let tx1 = TxId::new(1);
    let doc1 = DocId::new(200);

    // Directly stage an Insert operation with wrong vector dimension (dim 2 instead of 4)
    index.inner_core().cold.tx_buffer.stage(
        tx1,
        IndexOp::Insert {
            doc_id: doc1,
            data: vec![0.5, 0.5],
        },
    )?;

    // Commit must fail gracefully during Phase 1 normalization/validation
    let commit_res = index.commit(tx1).await;
    assert!(
        commit_res.is_err(),
        "commit() with dimension mismatch in staged op must fail with Err"
    );

    if let Err(err) = commit_res {
        assert!(
            matches!(err, ContextraError::InvalidInput(_)),
            "Expected InvalidInput error, got: {:?}",
            err
        );
    }

    // Verify operation was re-staged after validation error
    let restaged = index.inner_core().cold.tx_buffer.drain(tx1);
    assert_eq!(
        restaged.len(),
        1,
        "Failed transaction operations must be re-staged on commit validation failure"
    );

    Ok(())
}

#[tokio::test]
async fn test_hnsw_commit_handles_phase2_preparation_error_without_panic() -> Result<()> {
    let dim = 4;
    let index = create_test_index(dim)?;
    let tx1 = TxId::new(1);
    let doc1 = DocId::new(300);

    // Directly stage an Insert operation containing non-finite NaN values
    index.inner_core().cold.tx_buffer.stage(
        tx1,
        IndexOp::Insert {
            doc_id: doc1,
            data: vec![f32::NAN, 0.0, 0.0, 0.0],
        },
    )?;

    // Commit must fail gracefully during validation
    let commit_res = index.commit(tx1).await;
    assert!(
        commit_res.is_err(),
        "commit() with NaN vector must fail with Err"
    );

    Ok(())
}
