// FILE-CONTEXT
// ZWECK: Algorithmische Kantenbudget-Berechnung (γ-Augmentierung) für ACORN (Patel et al. 2024).
// INVARIANTEN: Ring-0 kompatibel (kein I/O, kein tokio); Zero Panic; monotone Eigenschaft bezüglich Selectivity.

/// Maximum gamma multiplier factor for edge degree augmentation.
const MAX_GAMMA_MULTIPLIER: f32 = 16.0;

/// Minimum selectivity threshold to prevent division by zero.
const MIN_SELECTIVITY_THRESHOLD: f32 = 0.01;

/// Computes the augmented edge budget factor $\gamma$ for graph expansion in ACORN search.
///
/// Based on Patel et al. 2024 (ACORN: Performant and Predicate-Agnostic Vector Search):
/// When predicate selectivity is high (meaning a low `predicate_selectivity` fraction of documents match,
/// e.g. 0.01 = 1%), graph connectivity degrades significantly if traversing only matching edges.
/// To preserve navigability, the graph traversal edge budget $\gamma \cdot M$ is augmented inversely
/// to the predicate selectivity.
///
/// # Monotonicity
/// Lower `predicate_selectivity` values (more restrictive filters) return higher or equal edge budgets:
/// `s1 < s2 => compute_gamma_edge_budget(deg, s1) >= compute_gamma_edge_budget(deg, s2)`
///
/// # Parameters
/// - `base_degree`: Base graph node degree (e.g., $M$ or $M_{max}$ in HNSW).
/// - `predicate_selectivity`: Fraction of documents passing the predicate, bounded in `[0.0, 1.0]`.
///
/// # Returns
/// The augmented edge budget count as `usize`. Zero or NaN inputs are handled safely without panics or overflows.
pub fn compute_gamma_edge_budget(base_degree: usize, predicate_selectivity: f32) -> usize {
    if base_degree == 0 {
        return 0;
    }

    // Handle NaN and bound selectivity to [0.0, 1.0]
    let sel = if predicate_selectivity.is_nan() {
        1.0
    } else {
        predicate_selectivity.clamp(0.0, 1.0)
    };

    // Inverse scaling: lower selectivity => higher multiplier
    let effective_sel = sel.max(MIN_SELECTIVITY_THRESHOLD);
    let raw_multiplier = 1.0 / effective_sel;
    let multiplier = raw_multiplier.min(MAX_GAMMA_MULTIPLIER);

    let budget = (base_degree as f64 * multiplier as f64).round();

    if budget >= usize::MAX as f64 {
        usize::MAX
    } else {
        budget as usize
    }
}
