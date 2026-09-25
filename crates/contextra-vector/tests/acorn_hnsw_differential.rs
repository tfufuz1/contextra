// FILE-CONTEXT
// ZWECK: Differential-Testing & Recall-Regressionstests für HnswIndex::search_knn_acorn (Spec §8.5, §22.4).
// INVARIANTEN: Zero Panic; Zero Cross-Contamination; Differential vs. NaiveReferenceIndex >= 95% Recall@k.

use contextra_core::{DistanceMetric, DocId, TxId, VectorIndex};
use contextra_vector::acorn::{compute_gamma_edge_budget, FilteredIndex, NaiveReferenceIndex};
use contextra_vector::hnsw::{HnswConfig, HnswIndex};

use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::collections::HashSet;

/// Helper function to create a random f32 vector of specified dimension using a seeded StdRng.
fn generate_random_vector(dim: usize, rng: &mut StdRng) -> Vec<f32> {
    (0..dim).map(|_| rng.gen_range(-1.0..1.0)).collect()
}

/// 1. test_differential_acorn_hnsw_vs_naive_reference
/// BEWEIST: Für 5 verschiedene, fest geseedete Datensätze (100–1.200 Punkte, Dimension 8–64, Selektivität 1 %–50 %)
/// liefert HnswIndex::search_knn_acorn mindestens 95 % Recall@k im Vergleich zur O(n) NaiveReferenceIndex Ground-Truth.
///
/// Begründung für 95 % Toleranz: HNSW ist ein approximatives Graphen-Suchverfahren. Während die γ-Augmentierung
/// die Navigierbarkeit über Nicht-Prädikat-Knoten als Brücken sichert, ermöglicht die Strahlsuch-Approximation (ef_search)
/// bei extrem großen/komplexen Datensätzen vereinzelte geringfügige Abweichungen von der exakten O(n) Brute-Force-Suche.
/// Daher verlangt Spec §22.4 eine Recall@k-Toleranz von >= 95 %.
#[tokio::test]
async fn test_differential_acorn_hnsw_vs_naive_reference() -> Result<(), Box<dyn std::error::Error>> {
    struct TestConfig {
        seed: u64,
        num_points: usize,
        dimension: usize,
        selectivity: f32,
        k: usize,
    }

    let test_cases = [
        TestConfig {
            seed: 1001,
            num_points: 150,
            dimension: 8,
            selectivity: 0.50,
            k: 10,
        },
        TestConfig {
            seed: 2002,
            num_points: 300,
            dimension: 16,
            selectivity: 0.25,
            k: 10,
        },
        TestConfig {
            seed: 3003,
            num_points: 500,
            dimension: 32,
            selectivity: 0.10,
            k: 10,
        },
        TestConfig {
            seed: 4004,
            num_points: 800,
            dimension: 64,
            selectivity: 0.05,
            k: 15,
        },
        TestConfig {
            seed: 5005,
            num_points: 1200,
            dimension: 16,
            selectivity: 0.01,
            k: 10,
        },
    ];

    for (case_idx, tc) in test_cases.iter().enumerate() {
        let mut rng = StdRng::seed_from_u64(tc.seed);

        let modulus = (1.0 / tc.selectivity).round() as u64;
        let predicate = move |id: DocId| id.inner() % modulus == 0;

        let hnsw_cfg = HnswConfig {
            dimension: tc.dimension,
            m: 16,
            ef_construction: 64,
            ef_search: 64,
            distance_metric: DistanceMetric::Euclidean,
            ..Default::default()
        };

        let hnsw_index = HnswIndex::try_new(hnsw_cfg)?;
        let mut naive_index = NaiveReferenceIndex::new(DistanceMetric::Euclidean);

        let tx = TxId::new(1);

        for i in 1..=tc.num_points as u64 {
            let doc_id = DocId::new(i);
            let vec = generate_random_vector(tc.dimension, &mut rng);

            hnsw_index.insert(tx, doc_id, &vec).await?;
            naive_index.insert(doc_id, vec);
        }
        hnsw_index.commit(tx).await?;

        let query = generate_random_vector(tc.dimension, &mut rng);
        let gamma_budget = compute_gamma_edge_budget(16, tc.selectivity) as u32;

        let naive_results = naive_index
            .search_knn_acorn(&query, tc.k, &predicate, gamma_budget)
            .expect("Naive search should succeed");

        let hnsw_results = hnsw_index
            .search_knn_acorn(&query, tc.k, &predicate, gamma_budget)
            .expect("HNSW ACORN search should succeed");

        // 1. Invariante: Zero-Cross-Contamination
        for (id, _dist) in &hnsw_results {
            assert!(
                predicate(*id),
                "Test case {case_idx}: Returned document {id:?} violates the predicate!"
            );
        }

        if naive_results.is_empty() {
            assert!(
                hnsw_results.is_empty(),
                "Test case {case_idx}: Ground truth returned 0 results, but HNSW returned {} results",
                hnsw_results.len()
            );
            continue;
        }

        let naive_ids: HashSet<DocId> = naive_results.iter().map(|(id, _)| *id).collect();
        let hnsw_ids: HashSet<DocId> = hnsw_results.iter().map(|(id, _)| *id).collect();

        let intersection_count = hnsw_ids.intersection(&naive_ids).count();
        let recall = intersection_count as f64 / naive_ids.len() as f64;

        assert!(
            recall >= 0.95,
            "Test case {case_idx} (seed={}, N={}, dim={}, sel={}): Recall@k failed! Expected >= 0.95, got {:.4} ({}/{} matching items)",
            tc.seed, tc.num_points, tc.dimension, tc.selectivity, recall, intersection_count, naive_ids.len()
        );
    }

    Ok(())
}

/// 2. test_high_selectivity_beam_starvation_regression
/// BEWEIST: Bei hoher Selektivität (1 %, d. h. nur 1 % der Punkte erfüllt das Prädikat) liefert
/// ACORN search_knn_acorn weiterhin k Ergebnisse (oder alle verbleibenden), während das klassische Post-Filtering
/// (Nicht-ACORN-Suche + Filter) bei gleicher Beam-Breite wegen Beam-Verhungerung beweisbar weniger Ergebnisse liefert.
#[tokio::test]
async fn test_high_selectivity_beam_starvation_regression() -> Result<(), Box<dyn std::error::Error>> {
    let seed = 9999;
    let mut rng = StdRng::seed_from_u64(seed);

    let num_points = 2000;
    let dim = 16;
    let k = 10;
    let selectivity = 0.01; // 1% selectivity: exactly 1 in 100 pass filter

    let modulus = 100u64;
    let predicate = move |id: DocId| id.inner() % modulus == 0;

    let hnsw_cfg = HnswConfig {
        dimension: dim,
        m: 16,
        ef_construction: 64,
        ef_search: 16, // Fixed tight beam width
        distance_metric: DistanceMetric::Euclidean,
        ..Default::default()
    };

    let hnsw_index = HnswIndex::try_new(hnsw_cfg)?;
    let tx = TxId::new(1);

    for i in 1..=num_points as u64 {
        let doc_id = DocId::new(i);
        let vec = generate_random_vector(dim, &mut rng);
        hnsw_index.insert(tx, doc_id, &vec).await?;
    }
    hnsw_index.commit(tx).await?;

    let query = generate_random_vector(dim, &mut rng);
    let gamma_budget = compute_gamma_edge_budget(16, selectivity) as u32;

    // ACORN search with bridge traversal and augmented beam
    let acorn_results = hnsw_index
        .search_knn_acorn(&query, k, &predicate, gamma_budget)
        .expect("ACORN search should succeed");

    // Standard unaugmented search (simulating classic post-filtering without bridge traversal)
    let standard_search_results = hnsw_index.search(&query, 16).await?;
    let post_filter_results: Vec<_> = standard_search_results
        .into_iter()
        .filter(|doc| predicate(doc.doc_id))
        .collect();

    assert_eq!(
        acorn_results.len(),
        k,
        "ACORN search should find full k={k} matching items at 1% selectivity, found {}",
        acorn_results.len()
    );

    assert!(
        acorn_results.len() > post_filter_results.len(),
        "ACORN search ({}) must return strictly more results than post-filtering baseline ({}) under beam starvation at 1% selectivity",
        acorn_results.len(),
        post_filter_results.len()
    );

    for (id, _dist) in &acorn_results {
        assert!(
            predicate(*id),
            "Returned document {id:?} in ACORN results violates predicate!"
        );
    }

    Ok(())
}

/// 3. test_zero_cross_contamination_invariant
/// BEWEIST: [Invariante] Zu KEINEM Zeitpunkt erscheint ein Knoten, der das Prädikat verletzt, in den
/// zurückgegebenen ACORN-Suchergebnissen (Zero Cross-Contamination).
#[tokio::test]
async fn test_zero_cross_contamination_invariant() -> Result<(), Box<dyn std::error::Error>> {
    let seed = 7777;
    let mut rng = StdRng::seed_from_u64(seed);

    let dim = 16;
    let hnsw_cfg = HnswConfig {
        dimension: dim,
        ..Default::default()
    };
    let hnsw_index = HnswIndex::try_new(hnsw_cfg)?;
    let tx = TxId::new(1);

    for i in 1..=300u64 {
        let vec = generate_random_vector(dim, &mut rng);
        hnsw_index.insert(tx, DocId::new(i), &vec).await?;
    }
    hnsw_index.commit(tx).await?;

    // Arbitrary restrictor predicate (e.g. only prime IDs or specific ranges)
    let predicate = |id: DocId| id.inner() > 100 && id.inner() % 3 == 0;

    for _query_run in 0..20 {
        let query = generate_random_vector(dim, &mut rng);
        let results = hnsw_index
            .search_knn_acorn(&query, 10, &predicate, 4)
            .expect("Search should succeed");

        for (id, _dist) in &results {
            assert!(
                predicate(*id),
                "Zero Cross-Contamination violation! Document {id:?} was returned but fails predicate!"
            );
        }
    }

    Ok(())
}

/// 4. test_candidate_storage_vec_structure_assertion
/// BEWEIST: [Architektur-Invariante] Die Visited- und Kandidatenverwaltung in acorn_filtered.rs nutzt
/// nachweislich Vec-basierte Speicherstrukturen (Vec<bool>, Vec<Candidate>) und KEINE Hash-basierten Container
/// (AHashSet/HashSet) im Traversal-Hotpath.
#[test]
fn test_candidate_storage_vec_structure_assertion() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let file_path = format!("{manifest_dir}/src/hnsw/acorn_filtered.rs");
    let source_code = std::fs::read_to_string(&file_path)
        .unwrap_or_else(|e| panic!("Should read {file_path}: {e}"));

    // Strip comment lines to check code statements
    let code_without_comments: String = source_code
        .lines()
        .filter(|line| !line.trim().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n");

    assert!(
        !code_without_comments.contains("AHashSet"),
        "acorn_filtered.rs hot-path must NOT contain AHashSet to prevent hashing latency in graph traversal"
    );

    assert!(
        !code_without_comments.contains("HashSet"),
        "acorn_filtered.rs hot-path must NOT contain HashSet to prevent allocation/hashing latency in graph traversal"
    );

    assert!(
        source_code.contains("Vec<bool>") || source_code.contains("vec![false;"),
        "acorn_filtered.rs must use Vec<bool> for hot-path visited tracking"
    );
}
