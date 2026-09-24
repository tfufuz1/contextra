use contextra_core::{ContextraError, DistanceMetric, DocId, Result, ScoredDocument, TxId, VectorIndex, VectorIndexStats};
use contextra_vector::hnsw::{HnswConfig, HnswIndex};
use contextra_vector::{VectorCandidateStream, DEFAULT_VECTOR_STREAM_BATCH_SIZE};
use rand::Rng;
use std::collections::HashSet;

fn create_hnsw_index(dim: usize, max_elements: usize) -> Result<HnswIndex> {
    let config = HnswConfig {
        dimension: dim,
        max_elements,
        m: 16,
        ef_construction: 200,
        ef_search: 100,
        distance_metric: DistanceMetric::Cosine,
        ..Default::default()
    };
    HnswIndex::try_new(config)
}

fn generate_random_vector(dim: usize, rng: &mut impl Rng) -> Vec<f32> {
    let mut v: Vec<f32> = (0..dim).map(|_| rng.gen_range(-1.0..1.0)).collect();
    let norm: f32 = v.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm > 0.0 {
        for x in v.iter_mut() {
            *x /= norm;
        }
    } else {
        v[0] = 1.0;
    }
    v
}

struct UnsupportedSearchAtMockIndex;

impl VectorIndex for UnsupportedSearchAtMockIndex {
    async fn insert(&self, _: TxId, _: DocId, _: &[f32]) -> Result<()> {
        Ok(())
    }
    async fn search(&self, _: &[f32], _: usize) -> Result<Vec<ScoredDocument>> {
        Ok(vec![])
    }
    async fn delete(&self, _: TxId, _: DocId) -> Result<()> {
        Ok(())
    }
    async fn commit(&self, _: TxId) -> Result<()> {
        Ok(())
    }
    async fn rollback(&self, _: TxId) -> Result<()> {
        Ok(())
    }
    async fn rollback_to_tx(&self, _: TxId) -> Result<()> {
        Ok(())
    }
    async fn last_tx_id(&self) -> Result<TxId> {
        Ok(TxId(0))
    }
    async fn len(&self) -> usize {
        0
    }
    async fn stats(&self) -> Result<VectorIndexStats> {
        Ok(VectorIndexStats::default())
    }
}

#[tokio::test]
async fn test_first_batch_lazy() -> Result<()> {
    let dim = 16;
    let index = create_hnsw_index(dim, 100)?;
    let mut rng = rand::thread_rng();

    let tx = TxId::new(1);
    for i in 0..50 {
        let v = generate_random_vector(dim, &mut rng);
        index.insert(tx, DocId::from(i as u64), &v).await?;
    }
    index.commit(tx).await?;

    let query = generate_random_vector(dim, &mut rng);
    let mut stream = VectorCandidateStream::new(&index, query, None);

    assert_eq!(DEFAULT_VECTOR_STREAM_BATCH_SIZE, 16);
    let batch = stream.next_batch().await?;
    assert!(batch.len() <= 16);
    assert_eq!(stream.fetched_depth(), 16);
    Ok(())
}

#[tokio::test]
async fn test_full_exhaustion_200_vectors() -> Result<()> {
    let dim = 16;
    let total_vectors = 200;
    let index = create_hnsw_index(dim, 300)?;
    let mut rng = rand::thread_rng();

    let tx = TxId::new(1);
    for i in 0..total_vectors {
        let v = generate_random_vector(dim, &mut rng);
        index.insert(tx, DocId::from(i as u64), &v).await?;
    }
    index.commit(tx).await?;

    let query = generate_random_vector(dim, &mut rng);
    let mut stream = VectorCandidateStream::new(&index, query, None);

    let mut collected = HashSet::new();
    let mut total_yielded = 0;

    while !stream.is_exhausted() {
        let batch = stream.next_batch().await?;
        if batch.is_empty() {
            break;
        }
        for doc in batch {
            assert!(
                collected.insert(doc.doc_id),
                "Duplicate DocId found"
            );
            total_yielded += 1;
        }
    }

    assert_eq!(collected.len(), total_vectors);
    assert_eq!(total_yielded, total_vectors);
    assert_eq!(stream.yielded(), total_vectors);
    assert!(stream.is_exhausted());
    assert!(stream.fetched_depth() <= VectorCandidateStream::<HnswIndex>::max_depth());
    Ok(())
}

#[tokio::test]
async fn test_determinism_identical_streams() -> Result<()> {
    let dim = 16;
    let index = create_hnsw_index(dim, 100)?;
    let mut rng = rand::thread_rng();

    let tx = TxId::new(1);
    for i in 0..50 {
        let v = generate_random_vector(dim, &mut rng);
        index.insert(tx, DocId::from(i as u64), &v).await?;
    }
    index.commit(tx).await?;

    let query = generate_random_vector(dim, &mut rng);

    let mut stream1 = VectorCandidateStream::new(&index, query.clone(), None);
    let mut stream2 = VectorCandidateStream::new(&index, query, None);

    let mut results1 = Vec::new();
    let mut results2 = Vec::new();

    while let Some(doc_id) = stream1.next_doc().await? {
        results1.push(doc_id);
    }
    while let Some(doc_id) = stream2.next_doc().await? {
        results2.push(doc_id);
    }

    assert_eq!(results1, results2);
    Ok(())
}

#[tokio::test]
async fn test_empty_index() -> Result<()> {
    let dim = 16;
    let index = create_hnsw_index(dim, 100)?;
    let query = vec![0.5f32; dim];

    let mut stream = VectorCandidateStream::new(&index, query, None);

    let batch = stream.next_batch().await?;
    assert!(batch.is_empty());
    assert!(stream.is_exhausted());

    let single_doc = stream.next_doc().await?;
    assert!(single_doc.is_none());
    Ok(())
}

#[tokio::test]
async fn test_seq_no_snapshot() -> Result<()> {
    let dim = 16;
    let index = create_hnsw_index(dim, 100)?;
    let mut rng = rand::thread_rng();

    // Commit 20 vectors at tx = 1
    let tx1 = TxId::new(1);
    for i in 0..20 {
        let v = generate_random_vector(dim, &mut rng);
        index.insert(tx1, DocId::from(i as u64), &v).await?;
    }
    index.commit(tx1).await?;

    // Commit 10 more vectors at tx = 2
    let tx2 = TxId::new(2);
    for i in 20..30 {
        let v = generate_random_vector(dim, &mut rng);
        index.insert(tx2, DocId::from(i as u64), &v).await?;
    }
    index.commit(tx2).await?;

    let query = generate_random_vector(dim, &mut rng);

    // Stream at seq_no = 1
    let mut stream_tx1 = VectorCandidateStream::new(&index, query.clone(), Some(1));
    let mut count_tx1 = 0;
    while stream_tx1.next_doc().await?.is_some() {
        count_tx1 += 1;
    }
    assert_eq!(count_tx1, 20);

    // Stream at seq_no = 2
    let mut stream_tx2 = VectorCandidateStream::new(&index, query, Some(2));
    let mut count_tx2 = 0;
    while stream_tx2.next_doc().await?.is_some() {
        count_tx2 += 1;
    }
    assert_eq!(count_tx2, 30);

    // Unsupported search_at mock index
    let mock_index = UnsupportedSearchAtMockIndex;
    let mut stream_mock = VectorCandidateStream::new(&mock_index, vec![1.0; dim], Some(1));
    let res = stream_mock.next_batch().await;
    match res {
        Err(ContextraError::CapabilityUnsupported { capability, .. }) => {
            assert_eq!(capability, "snapshot_read_at");
        }
        _ => {
            return Err(ContextraError::Index(
                "Expected CapabilityUnsupported error from UnsupportedSearchAtMockIndex".into(),
            ));
        }
    }
    Ok(())
}

#[tokio::test]
async fn test_with_batch_size_zero() -> Result<()> {
    let dim = 16;
    let index = create_hnsw_index(dim, 100)?;
    let mut rng = rand::thread_rng();

    let tx = TxId::new(1);
    for i in 0..10 {
        let v = generate_random_vector(dim, &mut rng);
        index.insert(tx, DocId::from(i as u64), &v).await?;
    }
    index.commit(tx).await?;

    let query = generate_random_vector(dim, &mut rng);
    let mut stream = VectorCandidateStream::new(&index, query, None).with_batch_size(0);

    let batch = stream.next_batch().await?;
    assert_eq!(batch.len(), 1);
    assert_eq!(stream.fetched_depth(), 1);
    Ok(())
}
