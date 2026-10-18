use contextra_core::{DistanceMetric, DocId, TxId, VectorIndex};
use contextra_vector::hnsw::{HnswConfig, HnswIndex};

#[tokio::test]
async fn test_ram_reduction_sq8_vs_f32() -> Result<(), Box<dyn std::error::Error>> {
    let dim = 128;
    let num_vectors = 1000;

    // 1. Build F32 Index
    let config_f32 = HnswConfig {
        dimension: dim,
        m: 16,
        ef_construction: 64,
        quantize: false,
        distance_metric: DistanceMetric::Cosine,
        ..Default::default()
    };
    let index_f32 = HnswIndex::try_new(config_f32)?;

    let tx = TxId::new(1);
    for i in 1..=num_vectors {
        let doc_id = DocId::new(i as u64);
        let vector = vec![(i as f32) / (num_vectors as f32); dim];
        index_f32.insert(tx, doc_id, &vector).await?;
    }
    index_f32.commit(tx).await?;

    let ram_f32 = index_f32.estimated_ram_bytes();

    // 2. Build SQ8 Quantized Index
    let config_sq8 = HnswConfig {
        dimension: dim,
        m: 16,
        ef_construction: 64,
        quantize: true,
        distance_metric: DistanceMetric::Cosine,
        ..Default::default()
    };
    let index_sq8 = HnswIndex::try_new(config_sq8)?;

    for i in 1..=num_vectors {
        let doc_id = DocId::new(i as u64);
        let vector = vec![(i as f32) / (num_vectors as f32); dim];
        index_sq8.insert(tx, doc_id, &vector).await?;
    }
    index_sq8.commit(tx).await?;

    let ram_sq8 = index_sq8.estimated_ram_bytes();

    // 3. Verify RAM Reduction (SQ8 should consume ~75% less vector data RAM)
    assert!(
        ram_sq8 < ram_f32,
        "SQ8 RAM usage ({ram_sq8}) must be strictly lower than F32 RAM usage ({ram_f32})"
    );

    let reduction_ratio = (ram_f32 - ram_sq8) as f64 / ram_f32 as f64;
    println!("RAM Reduction Ratio with SQ8: {:.2}%", reduction_ratio * 100.0);

    assert!(
        reduction_ratio >= 0.50,
        "SQ8 must achieve at least 50% overall index RAM reduction, got {:.2}%",
        reduction_ratio * 100.0
    );

    Ok(())
}
