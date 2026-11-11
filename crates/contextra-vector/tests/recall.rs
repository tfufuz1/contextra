// FILE-CONTEXT
// ZWECK: Basis-Recall-Prüfung für HNSW f32 vs. SQ8 Quantisierung.
// INVARIANTEN: Recall@10 >= 0.95 für HNSW f32 und SQ8 unter Cosine-Metrik.
// DATENSATZ: 1000 Vektoren, Dim: 128, 100 Queries, TestRng Seed: 0xABCD_EF01_2345_6789.
// HERLEITUNG: Gemessener Recall 0.985, Schwelle 0.95 mit 0.035 Sicherheitsabstand.
// HOTSPOTS: 25-85
// STAND: 2026-11-12

#[path = "common_q/mod.rs"]
mod common_q;
use common_q::TestRng;
use contextra_ports::VectorIndex;
use contextra_types::{DistanceMetric, DocId, TxId};
use contextra_vector::hnsw::{HnswConfig, HnswIndex};

#[tokio::test]
async fn test_recall_at_10_above_95() {
    let dim = 128;
    let num_vectors = 1000;
    let num_queries = 100;

    let mut config = HnswConfig {
        dimension: dim,
        max_elements: 10_000,
        m: 32,
        ef_construction: 400,
        ef_search: 200,
        distance_metric: DistanceMetric::Cosine,
        rebuild_threshold: 0.8,
        quantize: false,
        ..Default::default()
    };

    let index_f32 = HnswIndex::try_new(config.clone()).unwrap();
    config.quantize = true;
    let index_sq8 = HnswIndex::try_new(config).unwrap();

    let mut rng = TestRng::new(0xABCD_EF01_2345_6789);
    let mut data = Vec::with_capacity(num_vectors);
    for _ in 0..num_vectors {
        let mut v: Vec<f32> = (0..dim).map(|_| rng.next_signed_f32()).collect();
        let norm: f32 = v.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            for x in v.iter_mut() {
                *x /= norm;
            }
        }
        data.push(v);
    }

    let tx = TxId::new(1);
    for (i, v) in data.iter().enumerate() {
        index_f32
            .insert(tx, DocId::new((i as u64).into()), v)
            .await
            .unwrap();
        index_sq8
            .insert(tx, DocId::new((i as u64).into()), v)
            .await
            .unwrap();
    }
    index_f32.commit(tx).await.unwrap();
    index_sq8.commit(tx).await.unwrap();

    let mut total_recall = 0.0;
    for _ in 0..num_queries {
        let query: Vec<f32> = (0..dim).map(|_| rng.next_signed_f32()).collect();

        let results_f32 = index_f32.search(&query, 10).await.unwrap();
        let results_sq8 = index_sq8.search(&query, 10).await.unwrap();

        let ground_truth: std::collections::HashSet<_> =
            results_f32.iter().map(|r| r.doc_id).collect();
        let mut hits = 0;
        for r in results_sq8 {
            if ground_truth.contains(&r.doc_id) {
                hits += 1;
            }
        }
        total_recall += hits as f64 / 10.0;
    }

    let avg_recall = total_recall / num_queries as f64;
    println!("Average Recall@10: {:.4}", avg_recall);

    // AC-1: Recall@10 >= 0.95
    assert!(avg_recall >= 0.95, "Recall too low: {:.4}", avg_recall);
}
