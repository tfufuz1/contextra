use contextra_core::{DistanceMetric, DocId, Result, TxId, VectorIndex};
use contextra_vector::hnsw::{HnswConfig, HnswIndex};

#[tokio::test]
async fn test_hnsw_index_node_deletion_frees_arena_slot() -> Result<()> {
    let dim = 8;
    let config = HnswConfig {
        dimension: dim,
        max_elements: 100,
        m: 16,
        ef_construction: 200,
        ef_search: 100,
        distance_metric: DistanceMetric::Euclidean,
        ..Default::default()
    };
    let index = HnswIndex::try_new(config)?;

    let doc_id = DocId::from(1001u64);
    let vector = vec![1.0f32; dim];

    // Initial insert and commit
    let tx1 = TxId::new(1);
    index.insert(tx1, doc_id, &vector).await?;
    index.commit(tx1).await?;

    // Perform initial 10 insert/delete cycles to stabilize max capacity
    for i in 2..10 {
        let tx_del = TxId::new(i * 2);
        index.delete(tx_del, doc_id).await?;
        index.commit(tx_del).await?;

        let tx_ins = TxId::new(i * 2 + 1);
        index.insert(tx_ins, doc_id, &vector).await?;
        index.commit(tx_ins).await?;
    }

    let mem_after_10_cycles = index.stats().await?.memory_usage_bytes;
    assert_eq!(index.len().await, 1);

    // Perform another 100 delete and re-insert cycles
    for i in 10..110 {
        let tx_del = TxId::new(i * 2);
        index.delete(tx_del, doc_id).await?;
        index.commit(tx_del).await?;

        let tx_ins = TxId::new(i * 2 + 1);
        index.insert(tx_ins, doc_id, &vector).await?;
        index.commit(tx_ins).await?;
    }

    // Verify index length remains 1 and memory usage size is bounded (no growth from 10 to 110 cycles)
    assert_eq!(index.len().await, 1);
    let mem_after_110_cycles = index.stats().await?.memory_usage_bytes;
    assert_eq!(
        mem_after_110_cycles, mem_after_10_cycles,
        "HNSW index memory must remain bounded under repeated insert and delete cycles due to slot reuse via free_node"
    );

    Ok(())
}
