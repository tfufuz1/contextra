use contextra_ports::VectorIndex;
use contextra_types::DocId;
use contextra_vector::hnsw::{HnswConfigBuilder, HnswIndex, VectorSeededRng};
use std::sync::Arc;

async fn run_single_index_search(seed: u64, clock: Arc<dyn contextra_ports::Clock>) -> Vec<contextra_types::ScoredDocument> {
    let config = HnswConfigBuilder::new(4).build().unwrap();
    let rng = std::sync::Arc::new(VectorSeededRng::new(seed));
    let index = HnswIndex::try_new_with_clock_and_rng(config, clock, rng).unwrap();

    let tx = contextra_types::TxId::internal();
    let vecs = vec![
        (DocId::new(1), vec![1.0, 0.0, 0.0, 0.0]),
        (DocId::new(2), vec![0.9, 0.1, 0.0, 0.0]),
        (DocId::new(3), vec![0.8, 0.2, 0.0, 0.0]),
        (DocId::new(4), vec![0.7, 0.3, 0.0, 0.0]),
        (DocId::new(5), vec![0.6, 0.4, 0.0, 0.0]),
    ];

    for (id, vec) in vecs {
        index.insert(tx, id, &vec).await.unwrap();
    }
    index.commit(tx).await.unwrap();

    let query = vec![1.0, 0.0, 0.0, 0.0];
    index.search(&query, 5).await.unwrap()
}

#[tokio::test]
async fn hnsw_search_deterministic_with_fixed_seed() -> contextra_types::Result<()> {
    let seed = 42u64;
    let clock: Arc<dyn contextra_ports::Clock> = Arc::new(contextra_ports::SystemClock::new());

    let first_results = run_single_index_search(seed, clock.clone()).await;
    let first_ids_scores: Vec<(DocId, f32)> = first_results.iter().map(|r| (r.doc_id, r.score)).collect();

    for run in 1..20 {
        let results = run_single_index_search(seed, clock.clone()).await;
        let ids_scores: Vec<(DocId, f32)> = results.iter().map(|r| (r.doc_id, r.score)).collect();
        assert_eq!(
            ids_scores, first_ids_scores,
            "Run {} with fixed seed yielded non-deterministic search results",
            run
        );
    }

    Ok(())
}
