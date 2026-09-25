#![forbid(unsafe_code)]

//! Attention-aware Eviction Module (SnapKV/H2O Foundation).
//!
//! Provides traits and ranking logic for KV-cache segment eviction, combining
//! traditional LRU access timestamps with LLM attention importance scores.

use std::time::Instant;

/// Dyn-compatible source for segment attention importance scores.
///
/// Implemented by upstream inference backends or test mocks to supply
/// attention weights without introducing dependencies on specific ML tensor frameworks.
pub trait AttentionScoreSource: Send + Sync {
    /// Returns the importance score for a given `segment_id`.
    ///
    /// Higher values indicate higher retention importance (less evictable).
    /// Lower values indicate lower retention importance (more evictable).
    /// Returns `None` if no attention score is available for the segment.
    fn importance_score(&self, segment_id: u64) -> Option<f32>;
}

/// Default implementation that provides no attention scores (pure LRU behavior).
#[derive(Debug, Default, Clone, Copy)]
pub struct NullAttentionScoreSource;

impl AttentionScoreSource for NullAttentionScoreSource {
    fn importance_score(&self, _segment_id: u64) -> Option<f32> {
        None
    }
}

/// Ranks eviction candidates using a default balanced weighting (50% LRU age, 50% attention score).
///
/// Returns a vector of segment IDs ordered by eviction priority (first element = evicted first).
/// If `scores` supplies `None` for all candidates (such as [`NullAttentionScoreSource`]),
/// the result strictly preserves LRU order (oldest access instant evicted first).
pub fn rank_for_eviction(
    candidates: &[(u64, Instant)],
    scores: &dyn AttentionScoreSource,
) -> Vec<u64> {
    rank_for_eviction_weighted(candidates, scores, 0.5)
}

/// Ranks eviction candidates combining normalized LRU access age and attention scores.
///
/// `attention_weight` must be in `[0.0, 1.0]`, controlling the weight given to attention importance
/// versus LRU access age.
///
/// Returns a vector of segment IDs ordered by eviction priority (first element = evicted first).
pub fn rank_for_eviction_weighted(
    candidates: &[(u64, Instant)],
    scores: &dyn AttentionScoreSource,
    attention_weight: f32,
) -> Vec<u64> {
    if candidates.is_empty() {
        return Vec::new();
    }
    if candidates.len() == 1 {
        return vec![candidates[0].0];
    }

    let weight = attention_weight.clamp(0.0, 1.0);

    // Find min and max access instants for LRU normalization
    let min_instant = candidates.iter().map(|(_, t)| *t).min().unwrap_or(candidates[0].1);
    let max_instant = candidates.iter().map(|(_, t)| *t).max().unwrap_or(candidates[0].1);
    let time_span_secs = max_instant.duration_since(min_instant).as_secs_f32();

    // Collect candidate importance scores and find min/max
    let raw_scores: Vec<(u64, Instant, Option<f32>)> = candidates
        .iter()
        .map(|&(id, instant)| (id, instant, scores.importance_score(id)))
        .collect();

    let valid_scores: Vec<f32> = raw_scores
        .iter()
        .filter_map(|(_, _, s)| *s)
        .filter(|s| s.is_finite())
        .collect();

    let (min_score, max_score) = if !valid_scores.is_empty() {
        let min_s = valid_scores.iter().copied().fold(f32::INFINITY, f32::min);
        let max_s = valid_scores.iter().copied().fold(f32::NEG_INFINITY, f32::max);
        (min_s, max_s)
    } else {
        (0.0, 0.0)
    };

    let score_span = max_score - min_score;

    let mut ranked: Vec<(u64, Instant, f32)> = raw_scores
        .into_iter()
        .map(|(id, instant, opt_score)| {
            // Normalized LRU age: 0.0 = oldest access (most evictable), 1.0 = newest access
            let norm_lru = if time_span_secs > 0.0 {
                instant.duration_since(min_instant).as_secs_f32() / time_span_secs
            } else {
                0.0
            };

            // Normalized importance score: 0.0 = lowest importance (most evictable), 1.0 = highest importance
            let norm_importance = match opt_score {
                Some(s) if s.is_finite() && score_span > 0.0 => (s - min_score) / score_span,
                _ => norm_lru, // Fallback to LRU score if score is missing or uniform
            };

            // Composite eviction score: lower = evicted first
            let composite_score = (1.0 - weight) * norm_lru + weight * norm_importance;

            (id, instant, composite_score)
        })
        .collect();

    // Sort by composite score ascending (lowest score = evicted first).
    // Tie-break by access instant ascending (oldest access first), then segment ID ascending.
    ranked.sort_by(|a, b| {
        a.2.total_cmp(&b.2)
            .then_with(|| a.1.cmp(&b.1))
            .then_with(|| a.0.cmp(&b.0))
    });

    ranked.into_iter().map(|(id, _, _)| id).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::time::Duration;

    struct MapAttentionSource {
        scores: HashMap<u64, f32>,
    }

    impl AttentionScoreSource for MapAttentionSource {
        fn importance_score(&self, segment_id: u64) -> Option<f32> {
            self.scores.get(&segment_id).copied()
        }
    }

    #[test]
    fn test_null_attention_score_source_preserves_pure_lru() {
        let base = Instant::now();
        let candidates = vec![
            (100, base + Duration::from_secs(10)), // Newest
            (200, base + Duration::from_secs(2)),  // Mid
            (300, base + Duration::from_secs(1)),  // Oldest
        ];

        let null_source = NullAttentionScoreSource;
        let ranked = rank_for_eviction(&candidates, &null_source);

        // Expected eviction order: oldest access first -> 300, 200, 100
        assert_eq!(ranked, vec![300, 200, 100]);
    }

    #[test]
    fn test_attention_score_overrides_lru_order_when_weighted() {
        let base = Instant::now();
        let candidates = vec![
            (1, base + Duration::from_secs(1)), // Older access, but high importance (0.9)
            (2, base + Duration::from_secs(5)), // Newer access, but low importance (0.1)
        ];

        let mut scores_map = HashMap::new();
        scores_map.insert(1, 0.9);
        scores_map.insert(2, 0.1);
        let source = MapAttentionSource { scores: scores_map };

        // With heavy weight on attention score (0.8)
        let ranked = rank_for_eviction_weighted(&candidates, &source, 0.8);

        // Segment 2 has low importance (0.1), so it should be evicted before Segment 1
        assert_eq!(ranked, vec![2, 1]);
    }

    #[test]
    fn test_rank_empty_and_single_candidate() {
        let null_source = NullAttentionScoreSource;
        assert_eq!(rank_for_eviction(&[], &null_source), Vec::<u64>::new());

        let single = vec![(42, Instant::now())];
        assert_eq!(rank_for_eviction(&single, &null_source), vec![42]);
    }
}
