use contextra_core::{DistanceMetric, DocId, Result, TxId, VectorIndex};
use contextra_vector::hnsw::{HnswConfig, HnswIndex};
use contextra_vector::VectorCandidateStream;

#[tokio::test]
async fn test_candidate_stream_fetched_depth_tracking() -> Result<()> {
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

    let tx = TxId::new(1);
    for i in 0..50 {
        let v = vec![i as f32; dim];
        index.insert(tx, DocId::from(i as u64), &v).await?;
    }
    index.commit(tx).await?;

    let query = vec![0.0f32; dim];
    let mut stream = VectorCandidateStream::new(&index, query, None).with_batch_size(10);

    // Initial state: no fetch yet
    assert_eq!(stream.fetched_depth(), 0);

    // First batch fetch -> target_k = max(batch_size=10, 0*2) = 10
    let batch1 = stream.next_batch().await?;
    assert_eq!(batch1.len(), 10);
    assert_eq!(stream.fetched_depth(), 10);

    // Second batch fetch -> target_k = max(10, 10*2) = 20
    let batch2 = stream.next_batch().await?;
    assert_eq!(batch2.len(), 10);
    assert_eq!(stream.fetched_depth(), 20);

    Ok(())
}
