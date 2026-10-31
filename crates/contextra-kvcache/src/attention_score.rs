#![forbid(unsafe_code)]

//! # Attention-basierte Verdrängung (Eviction)
//!
//! ANCHOR[contextra-kvcache:ATTN-001]
//!
//! Dieses Modul implementiert aufmerksamkeitsgesteuerte Verdrängungsstrategien (SnapKV / H2O = Heavy-Hitter Oracle).
//! Es kombiniert klassische LRU-Zugriffszeitstempel (Least Recently Used) mit Attention-Wichtigkeits-Scores
//! aus großen Sprachmodellen.
//!
//! ## Warum Attention-gestützte Verdrängung?
//! Ein reines LRU-Verfahren verwirft stets die am längsten nicht zugegriffenen KV-Blöcke. Bei LLMs
//! gibt es jedoch Token (z. B. System-Instruktionen, syntaktische Anker oder zentrale Entitäten),
//! auf die fast jeder nachfolgende Token eine sehr hohe Aufmerksamkeit (Attention Weight) richtet.
//! Durch Berücksichtigung dieser Attention-Scores werden wichtige Segmente im RAM gehalten, selbst
//! wenn der letzte Zugriff bereits etwas zurückliegt.
//!
//! ## Fallback-Verhalten
//! Fehlen Attention-Informationen (z. B. bei Verwendung von [`NullAttentionScoreSource`]), fällt
//! der Algorithmus streng deterministisch auf das Standard-LRU-Verfahren zurück.

use contextra_ports::RequestId;
use contextra_types::TenantId;
use std::time::Instant;

// ANCHOR[contextra-kvcache:ATTN-002]
/// Dyn-kompatible Trait-Schnittstelle zur Bereitstellung von Attention-Wichtigkeits-Scores.
///
/// Wird von Upstream-Inferenz-Backends (z. B. Candle/Ollama) oder Test-Mocks implementiert,
/// um Aufmerksamkeitsscores bereitzustellen, ohne direkte Abhängigkeiten zu Tensor-Frameworks einzuführen.
pub trait AttentionScoreSource: Send + Sync {
    /// Liefert den Wichtigkeits-Score für ein gegebenes `segment_id`.
    ///
    /// - Höhere Werte bedeuten höhere Relevanz (bleibt länger im Cache).
    /// - Niedrigere Werte bedeuten geringe Relevanz (wird bevorzugt verdrängt).
    /// - Liefert `None`, falls für dieses Segment kein Score bekannt ist.
    fn importance_score(&self, segment_id: u64) -> Option<f32>;

    /// Liefert den Wichtigkeits-Score für einen Mandanten und ein `segment_id`.
    ///
    /// Standardmäßig wird an [`Self::importance_score`] delegiert.
    fn importance_score_for_tenant(&self, _tenant_id: TenantId, segment_id: u64) -> Option<f32> {
        self.importance_score(segment_id)
    }

    /// Registriert eine Zuordnung von `(tenant_id, segment_id)` zu `request_id`.
    fn register_segment(&self, _tenant_id: TenantId, _segment_id: u64, _request_id: RequestId) {}

    /// Entfernt eine Zuordnung, wenn das Segment verdrängt oder freigegeben wurde.
    fn unregister_segment(&self, _tenant_id: TenantId, _segment_id: u64) {}
}

// ANCHOR[contextra-kvcache:ATTN-003]
/// Standard-Implementierung, die keine Attention-Scores liefert (reines LRU-Verhalten).
///
/// # Fallback-Verhalten
/// Wenn kein Attention-Signal verfügbar ist, erfolgt die Verdrängung streng deterministisch
/// nach dem LRU-Prinzip (älteste Zugriffszeit zuerst).
/// Im Monitoring wird eine Warnmeldung geloggt, um das Fehlen des Signals transparent zu machen.
#[derive(Debug, Default, Clone, Copy)]
pub struct NullAttentionScoreSource;

impl AttentionScoreSource for NullAttentionScoreSource {
    fn importance_score(&self, _segment_id: u64) -> Option<f32> {
        None
    }

    fn importance_score_for_tenant(&self, tenant_id: TenantId, segment_id: u64) -> Option<f32> {
        tracing::warn!(
            tenant_id = ?tenant_id,
            segment_id = segment_id,
            "Evicting/ranking KV-cache segment without attention score (NullAttentionScoreSource fallback to LRU)"
        );
        None
    }
}

// ANCHOR[contextra-kvcache:ATTN-004]
/// Bewertet Verdrängungskandidaten mit Standard-Gewichtung (50% LRU-Alter, 50% Attention-Score).
///
/// Gibt eine sortierte Liste von Segment-IDs zurück (erstes Element = wird zuerst verdrängt).
pub fn rank_for_eviction(
    candidates: &[(u64, Instant)],
    scores: &dyn AttentionScoreSource,
) -> Vec<u64> {
    rank_for_eviction_weighted(candidates, scores, 0.5)
}

// ANCHOR[contextra-kvcache:ATTN-005]
/// Bewertet Verdrängungskandidaten unter Kombination von normalisiertem LRU-Alter und Attention-Score.
///
/// # Parameter
/// - `candidates`: Liste aus Tupeln `(Segment-ID, Zugriffszeitstempel)`.
/// - `scores`: Quelle für Aufmerksamkeitsscores.
/// - `attention_weight`: Gewichtung zwischen `0.0` (rein LRU) und `1.0` (rein Attention).
pub fn rank_for_eviction_weighted(
    candidates: &[(u64, Instant)],
    scores: &dyn AttentionScoreSource,
    attention_weight: f32,
) -> Vec<u64> {
    rank_for_eviction_weighted_for_tenant(TenantId::SYSTEM, candidates, scores, attention_weight)
}

// ANCHOR[contextra-kvcache:ATTN-006]
/// Bewertet Verdrängungskandidaten mandantenspezifisch.
///
/// # Ablauf
/// 1. Schnellausstieg bei 0 oder 1 Kandidaten.
/// 2. Zeitstempel normalisieren: Zeitdifferenz zur ältesten Anfrage ermitteln.
/// 3. Attention-Scores der Kandidaten abfragen und ungültige/nicht-endliche Werte (NaN, Inf) filtern.
/// 4. Min/Max-Skalierung (Min-Max-Normalization) über gefundene Scores berechnen.
/// 5. Zusammengesetzten Eviction-Score berechnen:
///    `composite_score = (1.0 - weight) * norm_lru + weight * norm_importance`.
/// 6. Stabil aufsteigend nach zusammengesetztem Score sortieren (niedrigste Wichtigkeit zuerst).
pub fn rank_for_eviction_weighted_for_tenant(
    tenant_id: TenantId,
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
    let min_instant = candidates
        .iter()
        .map(|(_, t)| *t)
        .min()
        .unwrap_or(candidates[0].1);
    let max_instant = candidates
        .iter()
        .map(|(_, t)| *t)
        .max()
        .unwrap_or(candidates[0].1);
    let time_span_secs = max_instant.duration_since(min_instant).as_secs_f32();

    // Collect candidate importance scores and find min/max
    let raw_scores: Vec<(u64, Instant, Option<f32>)> = candidates
        .iter()
        .map(|&(id, instant)| {
            (
                id,
                instant,
                scores.importance_score_for_tenant(tenant_id, id),
            )
        })
        .collect();

    // NAN-CHECK-OK: Eviction candidate ranking filters non-finite scores before compute
    let valid_scores: Vec<f32> = raw_scores
        .iter()
        .filter_map(|(_, _, s)| *s)
        .filter(|s| s.is_finite())
        .collect();

    let (min_score, max_score) = if !valid_scores.is_empty() {
        let min_s = valid_scores.iter().copied().fold(f32::INFINITY, f32::min);
        let max_s = valid_scores
            .iter()
            .copied()
            .fold(f32::NEG_INFINITY, f32::max);
        (min_s, max_s)
    } else {
        tracing::warn!(
            tenant_id = ?tenant_id,
            candidate_count = candidates.len(),
            "Eviction ranking performed without attention scores: falling back to deterministic LRU order"
        );
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

    // Prüft, dass ohne Attention-Scores die reine LRU-Reihenfolge strikt erhalten bleibt.
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

    // Prüft, dass hohe Attention-Werte die LRU-Reihenfolge gezielt überschreiben können.
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

    // Prüft Randfälle wie leere Kandidatenlisten und einelementige Mengen.
    #[test]
    fn test_rank_empty_and_single_candidate() {
        let null_source = NullAttentionScoreSource;
        assert_eq!(rank_for_eviction(&[], &null_source), Vec::<u64>::new());

        let single = vec![(42, Instant::now())];
        assert_eq!(rank_for_eviction(&single, &null_source), vec![42]);
    }

    // Prüft die numerische Stabilität bei gleichen Zeitstempeln, gleichen Scores oder NaN/Inf.
    #[test]
    fn test_v5_numerical_stability_edge_cases() {
        let base = Instant::now();
        // Edge Case 1: Identical timestamps (zero time_span_secs)
        let candidates_same_time = vec![(10, base), (20, base), (30, base)];

        let mut scores_map = HashMap::new();
        scores_map.insert(10, 5.0);
        scores_map.insert(20, 10.0);
        scores_map.insert(30, 1.0);
        let source = MapAttentionSource { scores: scores_map };

        let ranked = rank_for_eviction_weighted(&candidates_same_time, &source, 0.5);
        // Lowest score (30: 1.0) evicted first, then 10 (5.0), then 20 (10.0)
        assert_eq!(ranked, vec![30, 10, 20]);

        // Edge Case 2: Identical importance scores (zero score_span)
        let candidates_diff_time = vec![
            (100, base + Duration::from_secs(1)),
            (200, base + Duration::from_secs(5)),
        ];
        let mut uniform_map = HashMap::new();
        uniform_map.insert(100, 42.0);
        uniform_map.insert(200, 42.0);
        let uniform_source = MapAttentionSource {
            scores: uniform_map,
        };

        let ranked_uniform =
            rank_for_eviction_weighted(&candidates_diff_time, &uniform_source, 0.5);
        // With uniform scores, fallback to LRU age: oldest (100) evicted first
        assert_eq!(ranked_uniform, vec![100, 200]);

        // Edge Case 3: Non-finite scores (NaN / Infinity)
        let mut non_finite_map = HashMap::new();
        non_finite_map.insert(100, f32::NAN);
        non_finite_map.insert(200, f32::INFINITY);
        let non_finite_source = MapAttentionSource {
            scores: non_finite_map,
        };

        let ranked_non_finite =
            rank_for_eviction_weighted(&candidates_diff_time, &non_finite_source, 0.5);
        assert_eq!(ranked_non_finite, vec![100, 200]);
    }
}
