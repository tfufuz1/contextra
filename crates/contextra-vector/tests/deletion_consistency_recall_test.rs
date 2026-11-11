// FILE-CONTEXT
// ZWECK: Testet den vollständigen Lösch-Konsistenz-Zyklus für HNSW (WP-41).
// INVARIANTEN:
// 1. Gelöschte IDs tauchen in Suchergebnissen niemals wieder auf.
// 2. Recall der Restmenge bleibt >= 0.95 gegenüber dem Brute-Force-Orakel.
// 3. Konnektivität und Graphintegrität bleiben nach Löschung und Rebuild vollständig gewahrt.

use std::collections::HashSet;
use std::error::Error;

use contextra_ports::VectorIndex;
use contextra_types::{DocId, TxId};
use contextra_vector::hnsw::{HnswConfig, HnswIndex};
use contextra_vector::types::DistanceMetric;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

fn compute_distance_scalar(a: &[f32], b: &[f32], metric: DistanceMetric) -> f32 {
    match metric {
        DistanceMetric::Cosine => {
            let mut dot = 0.0f32;
            let mut norm_a = 0.0f32;
            let mut norm_b = 0.0f32;
            for (&x, &y) in a.iter().zip(b.iter()) {
                dot += x * y;
                norm_a += x * x;
                norm_b += y * y;
            }
            if norm_a == 0.0 || norm_b == 0.0 {
                1.0
            } else {
                1.0 - (dot / (norm_a.sqrt() * norm_b.sqrt()))
            }
        }
        DistanceMetric::Euclidean => {
            let mut sum_sq = 0.0f32;
            for (&x, &y) in a.iter().zip(b.iter()) {
                let diff = x - y;
                sum_sq += diff * diff;
            }
            sum_sq.sqrt()
        }
        _ => panic!("Unsupported metric for brute force reference"),
    }
}

/// Brute Force k-NN Scan auf einer gefilterten aktiven Datenmenge.
fn brute_force_knn_filtered(
    data: &[(u64, Vec<f32>)],
    query: &[f32],
    k: usize,
    metric: DistanceMetric,
) -> Vec<u64> {
    let mut scored: Vec<(u64, f32)> = data
        .iter()
        .map(|(id, v)| (*id, compute_distance_scalar(query, v, metric)))
        .collect();
    scored.sort_by(|a, b| a.1.total_cmp(&b.1));
    scored.into_iter().take(k).map(|(id, _)| id).collect()
}

#[tokio::test]
async fn test_deletion_consistency_and_surviving_recall() -> Result<(), Box<dyn Error>> {
    let num_vectors = 2_000;
    let dim = 64;
    let seed = 12345u64;
    let delete_count = 400; // 20% Löschung
    let k = 10;
    let num_queries = 50;

    let config = HnswConfig {
        dimension: dim,
        max_elements: num_vectors + 500,
        m: 32,
        ef_construction: 200,
        ef_search: 120,
        distance_metric: DistanceMetric::Cosine,
        rebuild_threshold: 0.0,
        quantize: false,
        ..Default::default()
    };

    let index = HnswIndex::try_new(config)?;
    let mut rng = StdRng::seed_from_u64(seed);

    let mut all_data = Vec::with_capacity(num_vectors);
    for i in 0..num_vectors {
        let v: Vec<f32> = (0..dim).map(|_| rng.gen_range(-1.0..1.0)).collect();
        all_data.push((i as u64, v));
    }

    // 1. Alle n Vektoren einfügen
    let tx_insert = TxId::new(1);
    for (id, v) in &all_data {
        index.insert(tx_insert, DocId::new(*id), v).await?;
    }
    index.commit(tx_insert).await?;

    // 2. Teilmenge auswählen und über delete entfernen
    let mut deleted_ids = HashSet::new();
    let tx_delete = TxId::new(2);
    for i in 0..delete_count {
        let del_id = (i * 3 + 7) as u64 % (num_vectors as u64);
        if deleted_ids.insert(del_id) {
            index.delete(tx_delete, DocId::new(del_id)).await?;
        }
    }
    index.commit(tx_delete).await?;

    // Überlebende Datenmenge für das Brute-Force-Orakel bilden
    let surviving_data: Vec<(u64, Vec<f32>)> = all_data
        .into_iter()
        .filter(|(id, _)| !deleted_ids.contains(id))
        .collect();

    // 3. Suchprüfungen:
    // Invariante 1: Gelöschte IDs tauchen in Suchergebnissen NIEMALS auf.
    // Invariante 2: Recall der Restmenge bleibt >= 0.95 gegenüber dem gefilterten Brute-Force-Orakel.
    let mut total_recall = 0.0;
    for _ in 0..num_queries {
        let query: Vec<f32> = (0..dim).map(|_| rng.gen_range(-1.0..1.0)).collect();

        let oracle_top_k =
            brute_force_knn_filtered(&surviving_data, &query, k, DistanceMetric::Cosine);
        let oracle_set: HashSet<u64> = oracle_top_k.into_iter().collect();

        let results = index.search(&query, k).await?;

        for res in &results {
            let res_id = res.doc_id.inner() as u64;
            assert!(
                !deleted_ids.contains(&res_id),
                "FATAL: Gelöschte DocId {} taucht in Suchergebnissen auf!",
                res_id
            );
        }

        let hits = results
            .iter()
            .filter(|r| oracle_set.contains(&(r.doc_id.inner() as u64)))
            .count();

        total_recall += hits as f64 / k as f64;
    }

    let avg_recall_tombstoned = total_recall / num_queries as f64;
    println!(
        "Recall der Restmenge vor Rebuild: {:.4}",
        avg_recall_tombstoned
    );
    assert!(
        avg_recall_tombstoned >= 0.95,
        "Recall nach Löschung vor Rebuild zu niedrig: {:.4} (erwartet >= 0.95)",
        avg_recall_tombstoned
    );

    // 4. Synchronen Rebuild ausführen (Konnektivität und Slot-Bereinigung)
    index.rebuild_sync()?;
    assert!(
        index.check_connectivity().is_ok(),
        "check_connectivity() muss nach Rebuild Ok sein!"
    );

    // 5. Recall nach Rebuild erneut gegen gefiltertes Orakel prüfen
    let mut total_recall_post_rebuild = 0.0;
    for _ in 0..num_queries {
        let query: Vec<f32> = (0..dim).map(|_| rng.gen_range(-1.0..1.0)).collect();

        let oracle_top_k =
            brute_force_knn_filtered(&surviving_data, &query, k, DistanceMetric::Cosine);
        let oracle_set: HashSet<u64> = oracle_top_k.into_iter().collect();

        let results = index.search(&query, k).await?;

        for res in &results {
            let res_id = res.doc_id.inner() as u64;
            assert!(
                !deleted_ids.contains(&res_id),
                "FATAL: Gelöschte DocId {} taucht nach Rebuild auf!",
                res_id
            );
        }

        let hits = results
            .iter()
            .filter(|r| oracle_set.contains(&(r.doc_id.inner() as u64)))
            .count();

        total_recall_post_rebuild += hits as f64 / k as f64;
    }

    let avg_recall_post_rebuild = total_recall_post_rebuild / num_queries as f64;
    println!(
        "Recall der Restmenge nach Rebuild: {:.4}",
        avg_recall_post_rebuild
    );
    assert!(
        avg_recall_post_rebuild >= 0.95,
        "Recall nach Rebuild zu niedrig: {:.4} (erwartet >= 0.95)",
        avg_recall_post_rebuild
    );

    Ok(())
}
