use contextra_ports::VectorIndex;
use contextra_types::{DocId, TxId};
use contextra_vector::hnsw::{HnswConfigBuilder, HnswIndex, VectorSeededRng};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::sync::Arc;

#[tokio::test]
async fn recall_stable_after_atomic_save_roundtrip() -> contextra_types::Result<()> {
    let temp_dir = tempfile::tempdir().unwrap();
    let index_path = temp_dir.path().join("recall_test.hnsw");

    let seed = 12345u64;
    let clock = Arc::new(contextra_ports::SystemClock::new());
    let config = HnswConfigBuilder::new(8).build().unwrap();
    let rng = std::sync::Arc::new(VectorSeededRng::new(seed));

    let index = HnswIndex::try_new_with_clock_and_rng(config, clock.clone(), rng)?;

    let tx = TxId::internal();
    let mut rng_data = StdRng::seed_from_u64(seed);

    let num_docs = 50;
    for i in 1..=num_docs {
        let vec: Vec<f32> = (0..8).map(|_| rng_data.gen::<f32>()).collect();
        let doc_id = DocId::new(i as u64);
        index.insert(tx, doc_id, &vec).await?;
    }
    index.commit(tx).await?;

    // Perform reference searches before save
    let queries: Vec<Vec<f32>> = (0..5)
        .map(|_| (0..8).map(|_| rng_data.gen::<f32>()).collect())
        .collect();

    let mut pre_save_results = Vec::new();
    for query in &queries {
        let res = index.search(query, 5).await?;
        let doc_scores: Vec<(DocId, f32)> = res.iter().map(|r| (r.doc_id, r.score)).collect();
        pre_save_results.push(doc_scores);
    }

    // Save index
    index.save(&index_path).await?;
    assert!(index_path.exists());

    // Load into new index instance
    let loaded_config = HnswConfigBuilder::new(8).build().unwrap();
    let loaded_rng = std::sync::Arc::new(VectorSeededRng::new(seed));
    let loaded_index = HnswIndex::try_new_with_clock_and_rng(loaded_config, clock, loaded_rng)?;
    loaded_index.load_mmap(&index_path).await?;

    // Perform searches after save and verify bit-identical results
    for (idx, query) in queries.iter().enumerate() {
        let res = loaded_index.search(query, 5).await?;
        let post_save_doc_scores: Vec<(DocId, f32)> = res.iter().map(|r| (r.doc_id, r.score)).collect();
        assert_eq!(
            post_save_doc_scores, pre_save_results[idx],
            "Query {} yielded different top-k recall/scores after save + load roundtrip",
            idx
        );
    }

    Ok(())
}
