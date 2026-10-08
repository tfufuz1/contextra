use contextra_core::{DocId, IndexOp, Result, TxId, VectorIndex};
use contextra_vector::hnsw::{HnswConfig, HnswIndex, VectorData};

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
async fn test_op_normalization_insert_delete_netto_delete() -> Result<()> {
    let dim = 4;
    let index = create_test_index(dim)?;
    let tx1 = TxId::new(1);
    let doc_a = DocId::new(100);
    let vec_a = vec![1.0, 0.0, 0.0, 0.0];

    // Stage insert(A) then delete(A) in the same transaction
    index.insert(tx1, doc_a, &vec_a).await?;
    index.delete(tx1, doc_a).await?;
    index.commit(tx1).await?;

    let all_docs = index.all_doc_ids().await?;
    assert!(
        !all_docs.contains(&doc_a),
        "Doc A should not exist after Netto-Delete in same tx"
    );

    let search_res = index.search(&vec_a, 10).await?;
    assert!(
        search_res.iter().all(|s| s.doc_id != doc_a),
        "Search should not return doc A after Netto-Delete"
    );
    assert_eq!(index.len().await, 0);

    Ok(())
}

#[tokio::test]
async fn test_op_normalization_delete_insert_replaces_old() -> Result<()> {
    let dim = 4;
    let index = create_test_index(dim)?;
    let doc_a = DocId::new(100);
    let vec_v1 = vec![1.0, 0.0, 0.0, 0.0];
    let vec_v2 = vec![0.0, 1.0, 0.0, 0.0];

    // Tx1: insert v1
    let tx1 = TxId::new(1);
    index.insert(tx1, doc_a, &vec_v1).await?;
    index.commit(tx1).await?;

    // Tx2: delete A, insert v2 in same tx
    let tx2 = TxId::new(2);
    index.delete(tx2, doc_a).await?;
    index.insert(tx2, doc_a, &vec_v2).await?;
    index.commit(tx2).await?;

    let all_docs = index.all_doc_ids().await?;
    assert_eq!(all_docs.len(), 1);
    assert_eq!(all_docs[0], doc_a);

    // Search query aligned with v2 should yield higher score than query aligned with v1
    let search_res = index.search(&vec_v2, 1).await?;
    assert_eq!(search_res.len(), 1);
    assert_eq!(search_res[0].doc_id, doc_a);
    assert!((search_res[0].score - 1.0).abs() < 1e-4);

    Ok(())
}

#[tokio::test]
async fn test_op_normalization_insert_insert_last_wins() -> Result<()> {
    let dim = 4;
    let index = create_test_index(dim)?;
    let doc_a = DocId::new(100);
    let vec_v1 = vec![1.0, 0.0, 0.0, 0.0];
    let vec_v2 = vec![0.0, 0.0, 1.0, 0.0];

    // Tx1: stage insert v1, then stage insert v2 in same tx
    let tx1 = TxId::new(1);
    index.insert(tx1, doc_a, &vec_v1).await?;
    index.insert(tx1, doc_a, &vec_v2).await?;
    index.commit(tx1).await?;

    let all_docs = index.all_doc_ids().await?;
    assert_eq!(all_docs.len(), 1);
    assert_eq!(all_docs[0], doc_a);

    let search_res = index.search(&vec_v2, 1).await?;
    assert_eq!(search_res.len(), 1);
    assert_eq!(search_res[0].doc_id, doc_a);
    assert!((search_res[0].score - 1.0).abs() < 1e-4);

    Ok(())
}

#[tokio::test]
async fn test_delete_never_inserted_doc_is_noop() -> Result<()> {
    let dim = 4;
    let index = create_test_index(dim)?;
    let doc_missing = DocId::new(999);

    let tx1 = TxId::new(1);
    index.delete(tx1, doc_missing).await?;
    let res = index.commit(tx1).await;
    assert!(res.is_ok(), "Delete on non-existing doc must succeed");

    assert_eq!(index.len().await, 0);
    assert!(index.all_doc_ids().await?.is_empty());

    Ok(())
}

#[tokio::test]
async fn test_commit_error_preparation_restages_ops() -> Result<()> {
    let dim = 4;
    let index = create_test_index(dim)?;
    let doc1 = DocId::new(10);
    let invalid_vec = vec![1.0]; // dimension 1 instead of expected 4

    let tx1 = TxId::new(1);
    // Stage op directly with invalid vector dimension
    index.inner_core().cold.tx_buffer.stage(
        tx1,
        IndexOp::Insert {
            doc_id: doc1,
            data: invalid_vec,
        },
    )?;

    let commit_res = index.commit(tx1).await;
    assert!(
        commit_res.is_err(),
        "First commit should fail due to dimension mismatch in preparation"
    );

    // Verify ops were re-staged into tx_buffer upon preparation failure
    let restaged_ops = index.inner_core().cold.tx_buffer.drain(tx1);
    assert_eq!(
        restaged_ops.len(),
        1,
        "Ops must be re-staged back into tx_buffer when commit preparation fails"
    );

    Ok(())
}

#[tokio::test]
async fn test_zeroed_ram_slot_on_safe_free_vs_retention() -> Result<()> {
    let dim = 4;
    let index = create_test_index(dim)?;
    let doc1 = DocId::new(100);
    let vec1 = vec![0.5, 0.5, 0.5, 0.5];

    // Case 1: Delete without retention -> safe to free -> vector zeroed
    let tx1 = TxId::new(1);
    index.insert(tx1, doc1, &vec1).await?;
    index.commit(tx1).await?;

    let tx2 = TxId::new(2);
    index.delete(tx2, doc1).await?;
    index.commit(tx2).await?;

    // Check RAM slot 0
    let vec_slot0 = index.ram_vector_at(0).expect("RAM node 0 should exist");
    if let VectorData::F32(v) = vec_slot0 {
        assert!(
            v.iter().all(|&x| x == 0.0),
            "Vector in safe-freed slot 0 must be zeroed, got {:?}",
            v
        );
    } else {
        panic!("Expected VectorData::F32");
    }

    // Case 2: Delete with active snapshot pin retention -> NOT safe to free -> vector retained
    let doc2 = DocId::new(200);
    let vec2 = vec![0.8, 0.1, 0.1, 0.0];

    let tx3 = TxId::new(3);
    index.insert(tx3, doc2, &vec2).await?;
    index.commit(tx3).await?;

    // Pin snapshot at seq 3
    index.pin_snapshot(3);

    let tx4 = TxId::new(4);
    index.delete(tx4, doc2).await?;
    index.commit(tx4).await?;

    // Check RAM slot 1
    let vec_slot1 = index.ram_vector_at(1).expect("RAM node 1 should exist");
    if let VectorData::F32(v) = vec_slot1 {
        assert_eq!(
            v, vec2,
            "Vector in retained node slot 1 must NOT be zeroed while pinned by snapshot"
        );
    } else {
        panic!("Expected VectorData::F32");
    }

    index.unpin_snapshot(3);

    Ok(())
}
