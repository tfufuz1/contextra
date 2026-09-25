use contextra_core::DocId;
use contextra_vector::acorn::{compute_gamma_edge_budget, FilteredIndex, NaiveReferenceIndex};

#[test]
fn test_naive_reference_top_k_with_equality_predicate() {
    let mut index = NaiveReferenceIndex::from_vectors(vec![]);

    // Build small dataset of 20 2D points: (0,0), (1,0), (2,0), ... (19,0)
    for i in 0..20u64 {
        index.insert(DocId::from(i), vec![i as f32, 0.0]);
    }

    let query = [2.1f32, 0.0f32];
    let target_ids = [2u64, 3u64, 4u64, 10u64];

    // Predicate matching only target_ids
    let predicate = move |id: DocId| target_ids.contains(&id.inner());

    // Search k=3 nearest neighbors among matching predicate points
    let k = 3;
    let gamma = 1;
    let results = index
        .search_knn_acorn(&query, k, &predicate, gamma)
        .expect("search_knn_acorn should succeed");

    assert_eq!(results.len(), 3);

    // Closest to 2.1 among {2, 3, 4, 10} are:
    // 1st: DocId(2) -> dist = |2.1 - 2.0| = 0.1
    // 2nd: DocId(3) -> dist = |2.1 - 3.0| = 0.9
    // 3rd: DocId(4) -> dist = |2.1 - 4.0| = 1.9
    assert_eq!(results[0].0, DocId::from(2u64));
    assert!((results[0].1 - 0.1).abs() < 1e-5);

    assert_eq!(results[1].0, DocId::from(3u64));
    assert!((results[1].1 - 0.9).abs() < 1e-5);

    assert_eq!(results[2].0, DocId::from(4u64));
    assert!((results[2].1 - 1.9).abs() < 1e-5);
}

#[test]
fn test_gamma_edge_budget_monotonicity() {
    let base_degree = 16;

    // Test at least 3 selectivity pairs (s_low <= s_high => budget_low >= budget_high)
    let selectivities = [0.01, 0.05, 0.1, 0.25, 0.5, 0.75, 1.0];

    for i in 0..selectivities.len() - 1 {
        let s_low = selectivities[i];
        let s_high = selectivities[i + 1];

        let budget_low = compute_gamma_edge_budget(base_degree, s_low);
        let budget_high = compute_gamma_edge_budget(base_degree, s_high);

        assert!(
            budget_low >= budget_high,
            "Monotonicity failed for s_low={s_low} (budget={budget_low}) vs s_high={s_high} (budget={budget_high})"
        );
    }
}

#[test]
fn test_gamma_edge_budget_edge_cases() {
    let base_degree = 16;

    // Test selectivity 0.0 (maximum restriction, lower bound)
    let budget_zero = compute_gamma_edge_budget(base_degree, 0.0);
    assert!(budget_zero >= base_degree);

    // Test selectivity 1.0 (no restriction, upper bound)
    let budget_one = compute_gamma_edge_budget(base_degree, 1.0);
    assert_eq!(budget_one, base_degree);

    // Test negative, >1.0, and NaN values
    let budget_neg = compute_gamma_edge_budget(base_degree, -0.5);
    assert_eq!(budget_neg, budget_zero);

    let budget_over = compute_gamma_edge_budget(base_degree, 2.5);
    assert_eq!(budget_over, budget_one);

    let budget_nan = compute_gamma_edge_budget(base_degree, f32::NAN);
    assert_eq!(budget_nan, base_degree);

    // Test zero base degree
    assert_eq!(compute_gamma_edge_budget(0, 0.0), 0);
    assert_eq!(compute_gamma_edge_budget(0, 0.5), 0);

    // Test large base degree overflow safety
    let budget_large = compute_gamma_edge_budget(usize::MAX / 2, 0.0);
    assert!(budget_large > 0);
}
