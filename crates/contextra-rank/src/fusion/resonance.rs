use super::rrf::weighted_reciprocal_rank_fusion;
use super::types::{FusedScore, SearchResult};
use serde::{Deserialize, Serialize};

/// Configuration for Resonance Coherence Bonus.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ResonanceConfig {
    /// Exponent β for coherence bonus. Default: 0.5.
    pub beta: f32,
    /// Boost strength γ. Default: 0.3.
    pub gamma: f32,
}

impl Default for ResonanceConfig {
    fn default() -> Self {
        Self {
            beta: 0.5,
            gamma: 0.3,
        }
    }
}

/// Convenience API to fuse multi-signal search results.
pub fn fuse_signals(
    result_sets: Vec<(String, Vec<SearchResult>, f32)>,
    max_results: usize,
) -> Vec<FusedScore> {
    weighted_reciprocal_rank_fusion(result_sets, max_results)
}

/// Applies Resonance Coherence Bonus to fused search results.
pub fn apply_resonance_bonus(
    results: Vec<SearchResult>,
    valid_signal_count: usize,
    config: &ResonanceConfig,
) -> Vec<SearchResult> {
    if !config.beta.is_finite() || !config.gamma.is_finite() {
        tracing::warn!(
            beta = config.beta,
            gamma = config.gamma,
            "ResonanceConfig contains non-finite value: beta={}, gamma={}; skipping resonance bonus",
            config.beta,
            config.gamma
        );
        return results;
    }
    if valid_signal_count == 0 {
        return results;
    }
    let beta = config.beta.clamp(0.1, 2.0);
    let gamma = config.gamma.clamp(0.0, 1.0);
    let mut results: Vec<_> = results
        .into_iter()
        .map(|mut r| {
            if !r.score.is_finite() {
                tracing::error!(
                    doc_id = %r.id,
                    raw_score = r.score,
                    "apply_resonance_bonus: non-finite score entering resonance bonus stage"
                );
            }
            let signal_count = r.matched_signals.len();
            let coherence = (signal_count as f32 / valid_signal_count as f32).powf(beta);
            let bonus = gamma * coherence;
            r.score *= 1.0 + bonus;
            if let Some(ref mut prov) = r.provenance {
                prov.coherence_bonus = bonus;
            }
            r
        })
        .collect();

    results.sort_by(|a, b| match (a.score.is_finite(), b.score.is_finite()) {
        (true, false) => std::cmp::Ordering::Less,
        (false, true) => std::cmp::Ordering::Greater,
        _ => b.score.total_cmp(&a.score).then_with(|| a.id.cmp(&b.id)),
    });

    results
}
