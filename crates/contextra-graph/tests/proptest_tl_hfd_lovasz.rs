//! Property and differential test suite for TL-HFD Lovász extension calculations (Spec §21.1, §22.2c, Audit F3).
//!
//! Evaluates `compute_lovasz_extension` and `truncate_participants` for invariant enforcement,
//! differential equivalence against a naive brute-force reference, tie-breaking determinism,
//! and truncation bounds.

use contextra_graph::tl_hfd::{compute_lovasz_extension, truncate_participants};
use contextra_types::EntityId;
use proptest::prelude::*;
use std::cmp::Ordering;

/// Independent naive brute-force reference implementation for Lovász extension cost.
///
/// Calculates $f_e(x) = \max_{u \in e} x_u - \min_{u \in e} x_u$ via simple `Vec` iteration with `f32::max`/`min`.
/// Intentionally does NOT copy any tie-breaking logic from `compute_lovasz_extension`.
fn bruteforce_lovasz(participants: &[EntityId], get_x: impl Fn(EntityId) -> f32) -> f32 {
    if participants.len() < 2 {
        return 0.0;
    }
    let mut min_val = f32::INFINITY;
    let mut max_val = f32::NEG_INFINITY;
    for &u in participants {
        let x = get_x(u);
        if x < min_val {
            min_val = x;
        }
        if x > max_val {
            max_val = x;
        }
    }
    if max_val <= min_val {
        0.0
    } else {
        max_val - min_val
    }
}

prop_compose! {
    fn arb_hypergraph_2_to_15()(
        x_vals in proptest::collection::vec(-1000.0f32..1000.0f32, 2..=15),
    ) -> (Vec<EntityId>, Vec<f32>) {
        let participants: Vec<EntityId> = (1..=x_vals.len()).map(|i| EntityId::new(i as u64)).collect();
        (participants, x_vals)
    }
}

prop_compose! {
    fn arb_truncate_input()(
        x_vals in proptest::collection::vec(-1000.0f32..1000.0f32, 2..=15),
    )(
        max_sort_size in 1usize..x_vals.len(),
        participants in Just((1..=x_vals.len()).map(|i| EntityId::new(i as u64)).collect::<Vec<_>>()),
        x_vals in Just(x_vals),
    ) -> (Vec<EntityId>, Vec<f32>, usize) {
        (participants, x_vals, max_sort_size)
    }
}

prop_compose! {
    fn arb_truncate_bounds_input()(
        x_vals in proptest::collection::vec(-1000.0f32..1000.0f32, 1..=15),
        max_sort_size in 1usize..=20,
    ) -> (Vec<EntityId>, Vec<f32>, usize) {
        let participants: Vec<EntityId> = (1..=x_vals.len()).map(|i| EntityId::new(i as u64)).collect();
        (participants, x_vals, max_sort_size)
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// 1. prop_lovasz_extension_is_nonnegative
    /// For arbitrary participant lists with >= 2 nodes and arbitrary finite x-values,
    /// compute_lovasz_extension yields f_e(x) >= 0.0.
    #[test]
    fn prop_lovasz_extension_is_nonnegative(
        (participants, x_vals) in arb_hypergraph_2_to_15(),
    ) {
        let get_x = |u: EntityId| x_vals[(u.inner() - 1) as usize];
        let (f_e, _u_max, _u_min) = compute_lovasz_extension(&participants, get_x);
        prop_assert!(f_e >= 0.0, "Lovász extension cost f_e ({}) must be non-negative", f_e);
    }

    /// 2. prop_lovasz_extension_matches_bruteforce_reference
    /// For hypergraphs with < 15 nodes (2..=15), compute_lovasz_extension(...).0 matches
    /// independent bruteforce_lovasz within a numerical tolerance of 1e-5.
    #[test]
    fn prop_lovasz_extension_matches_bruteforce_reference(
        (participants, x_vals) in arb_hypergraph_2_to_15(),
    ) {
        let get_x = |u: EntityId| x_vals[(u.inner() - 1) as usize];
        let (f_e, _, _) = compute_lovasz_extension(&participants, get_x);
        let bf_fe = bruteforce_lovasz(&participants, get_x);
        let diff = (f_e - bf_fe).abs();
        prop_assert!(
            diff <= 1e-5,
            "f_e ({}) does not match bruteforce ({}) within 1e-5 tolerance (diff = {})",
            f_e,
            bf_fe,
            diff
        );
    }

    /// 3. prop_lovasz_extension_max_min_are_distinct_or_none
    /// Invariant: If f_e(x) > 0.0, then u_max and u_min must both be Some(...) and pairwise distinct.
    /// If f_e(x) == 0.0, both must be None.
    #[test]
    fn prop_lovasz_extension_max_min_are_distinct_or_none(
        (participants, x_vals) in arb_hypergraph_2_to_15(),
    ) {
        let get_x = |u: EntityId| x_vals[(u.inner() - 1) as usize];
        let (f_e, u_max, u_min) = compute_lovasz_extension(&participants, get_x);
        if f_e > 0.0 {
            if let (Some(ma), Some(mi)) = (u_max, u_min) {
                prop_assert_ne!(
                    ma,
                    mi,
                    "u_max ({:?}) and u_min ({:?}) must be distinct when f_e > 0.0",
                    ma,
                    mi
                );
            } else {
                prop_assert!(false, "u_max and u_min must both be Some when f_e > 0.0");
            }
        } else {
            prop_assert!(
                u_max.is_none(),
                "u_max ({:?}) must be None when f_e == 0.0",
                u_max
            );
            prop_assert!(
                u_min.is_none(),
                "u_min ({:?}) must be None when f_e == 0.0",
                u_min
            );
        }
    }

    /// 4. prop_truncate_participants_preserves_min_and_top_k
    /// For random participant lists and random max_sort_size < participants.len(),
    /// proves that:
    /// (a) The result always contains the participant with the global minimum x-value (tie-breaker: smallest EntityId).
    /// (b) All other returned participants belong to the top max_sort_size global largest x-values.
    #[test]
    fn prop_truncate_participants_preserves_min_and_top_k(
        (participants, x_vals, max_sort_size) in arb_truncate_input(),
    ) {
        let get_x = |u: EntityId| x_vals[(u.inner() - 1) as usize];
        let truncated = truncate_participants(&participants, get_x, max_sort_size);

        // Independent brute-force sorting: descending by x_v, tie-breaker: EntityId ascending
        let mut bf_sorted = participants.clone();
        bf_sorted.sort_by(|&a, &b| {
            let xa = get_x(a);
            let xb = get_x(b);
            xb.partial_cmp(&xa)
                .unwrap_or(Ordering::Equal)
                .then_with(|| a.cmp(&b))
        });

        let bf_top_k: Vec<EntityId> = bf_sorted[..max_sort_size].to_vec();
        let bf_min = match bf_sorted.last() {
            Some(&m) => m,
            None => {
                prop_assert!(false, "bf_sorted cannot be empty");
                return Ok(());
            }
        };

        // (a) Truncated must contain global minimum participant (with tie-breaker)
        prop_assert!(
            truncated.contains(&bf_min),
            "Truncated list {:?} must contain global minimum participant {:?}",
            truncated,
            bf_min
        );

        // (b) Top-k elements must all be preserved in truncated
        for &top_node in &bf_top_k {
            prop_assert!(
                truncated.contains(&top_node),
                "Truncated list {:?} must contain top-k participant {:?}",
                truncated,
                top_node
            );
        }

        // (c) Every element in truncated belongs either to top-k or is the global minimum
        for &elem in &truncated {
            prop_assert!(
                bf_top_k.contains(&elem) || elem == bf_min,
                "Participant {:?} in truncated list {:?} is neither top-k nor global minimum {:?}",
                elem,
                truncated,
                bf_min
            );
        }
    }

    /// 5. prop_truncate_participants_result_length_bounded
    /// The result length is bounded by participants.len(), and when participants.len() > max_sort_size,
    /// length is strictly max_sort_size or max_sort_size + 1.
    #[test]
    fn prop_truncate_participants_result_length_bounded(
        (participants, x_vals, max_sort_size) in arb_truncate_bounds_input(),
    ) {
        let get_x = |u: EntityId| x_vals[(u.inner() - 1) as usize];
        let truncated = truncate_participants(&participants, get_x, max_sort_size);

        prop_assert!(
            truncated.len() <= participants.len(),
            "Truncated length {} exceeds original length {}",
            truncated.len(),
            participants.len()
        );

        if participants.len() <= max_sort_size {
            prop_assert_eq!(
                truncated.len(),
                participants.len(),
                "When participants.len() <= max_sort_size, length must be preserved"
            );
        } else {
            prop_assert!(
                truncated.len() == max_sort_size || truncated.len() == max_sort_size + 1,
                "Truncated len {} must be max_sort_size ({}) or max_sort_size + 1 ({}) for input len {}",
                truncated.len(),
                max_sort_size,
                max_sort_size + 1,
                participants.len()
            );
        }
    }
}

/// Audit F3 & Spec §22.2c / §21: Regression test for the explicit boundary case:
/// A hypergraph with 15 nodes (limit of required reference comparison size) and
/// `max_sort_size` capped just below the node count (e.g. 14 and 13).
///
/// Demonstrates that the deterministic cutoff maintains convergence quality within
/// a 1e-5 numerical tolerance compared to the uncapped Lovász extension computation.
///
/// Tolerance explanation:
/// - f_e_full is computed on the full 15-node participant set.
/// - f_e_trunc is computed on the truncated participant set returned by `truncate_participants`.
/// - Because `truncate_participants` retains top-k nodes (including u_max) and explicitly appends
///   the global min node u_min, both u_max and u_min remain present in the truncated set.
/// - Consequently, the edge cut cost f_e(x) = max_u x_u - min_u x_u is preserved identically,
///   giving an f_e delta = |f_e_full - f_e_trunc| = 0.0 <= 1e-5.
#[test]
fn test_audit_f3_regression_15_node_hypergraph_truncation_cutoff() {
    let num_nodes = 15;
    let participants: Vec<EntityId> = (1..=num_nodes).map(|i| EntityId::new(i as u64)).collect();

    // Distinct non-trivial x-values for 15 nodes: x_i = i * 0.7 - 5.0
    let get_x = |u: EntityId| (u.inner() as f32) * 0.7 - 5.0;

    let (full_fe, full_max, full_min) = compute_lovasz_extension(&participants, get_x);
    assert!(full_fe > 0.0);
    assert_eq!(full_max, Some(EntityId::new(15))); // x_15 = 5.5
    assert_eq!(full_min, Some(EntityId::new(1))); // x_1  = -4.3

    // Case A: max_sort_size = 14 (just below 15)
    let truncated_14 = truncate_participants(&participants, get_x, 14);
    let (trunc_fe_14, trunc_max_14, trunc_min_14) = compute_lovasz_extension(&truncated_14, get_x);

    let delta_14 = (full_fe - trunc_fe_14).abs();
    assert!(
        delta_14 <= 1e-5,
        "f_e delta for max_sort_size=14 ({delta_14}) exceeds tolerance 1e-5"
    );
    assert_eq!(trunc_max_14, full_max);
    assert_eq!(trunc_min_14, full_min);

    // Case B: max_sort_size = 13
    let truncated_13 = truncate_participants(&participants, get_x, 13);
    let (trunc_fe_13, trunc_max_13, trunc_min_13) = compute_lovasz_extension(&truncated_13, get_x);

    let delta_13 = (full_fe - trunc_fe_13).abs();
    assert!(
        delta_13 <= 1e-5,
        "f_e delta for max_sort_size=13 ({delta_13}) exceeds tolerance 1e-5"
    );
    assert_eq!(trunc_max_13, full_max);
    assert_eq!(trunc_min_13, full_min);
}
