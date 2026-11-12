// FILE-CONTEXT
// ZWECK: Exaktes Nachranking und Score-Berechnung für SQ8 und unquantisierte Kandidaten.
// INVARIANTEN: Deterministische Sortierung (Score absteigend, DocId aufsteigend).
// HOTSPOTS: compute_distance_score, sort_and_truncate_results
// STAND: 2026-11-12

//! Exaktes Nachranking und Metrik-Konvertierung für Suchkandidaten.

use crate::error::{Result, VectorError};
use crate::types::{DistanceMetric, DocId, ScoredDocument};

/// Berechnet den normierten Ähnlichkeitsscore basierend auf Distanzmetrik und Distanzwert.
#[inline]
pub(crate) fn compute_distance_score(metric: DistanceMetric, final_dist: f32) -> Result<f32> {
    match metric {
        DistanceMetric::Cosine => Ok(1.0 - final_dist),
        DistanceMetric::Euclidean => Ok(1.0 / (1.0 + final_dist)),
        DistanceMetric::DotProduct => Ok(-final_dist),
        #[allow(unreachable_patterns)]
        other => Err(VectorError::Index(format!(
            "Unsupported DistanceMetric variant in search: {other:?}"
        ))),
    }
}

/// Sortiert gefundene Dokumente deterministisch (Score absteigend, DocId aufsteigend) und begrenzt auf k.
#[inline]
pub(crate) fn sort_and_truncate_results(results: &mut Vec<ScoredDocument>, k: usize) {
    if results.len() > k {
        results.select_nth_unstable_by(k - 1, |a, b| {
            b.score
                .total_cmp(&a.score)
                .then_with(|| a.doc_id.cmp(&b.doc_id))
        });
        results.truncate(k);
    }
    results.sort_by(|a, b| {
        b.score
            .total_cmp(&a.score)
            .then_with(|| a.doc_id.cmp(&b.doc_id))
    });
}
