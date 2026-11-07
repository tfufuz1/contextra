//! # Information Retrieval Evaluierungsmetriken (`metrics`)
//!
//! **Ring:** Ring 0 (Ranking & Kalibrierung)
//!
//! Reines, deterministisches Berechnungsmodul zur Offline- und Online-Evaluation
//! von Retrieval- und Ranking-Ergebnissen (Reciprocal Rank / MRR, Precision@k, Recall@k, nDCG@k).
//!
//! Alle Metriken arbeiten frei von I/O, Netzwerk, Zufall oder Nebenläufigkeit und garantieren
//! vollständige Abwesenheit von Panics bei ungültigen oder leeren Eingaben.

use ahash::{AHashMap, AHashSet};

/// Berechnet den Reciprocal Rank (RR) für eine geordnete Liste von Treffer-IDs.
///
/// Findet die 1-basierte Rangposition des ersten relevanten Dokuments und liefert `1.0 / rank`.
/// Falls kein relevantes Dokument enthalten ist oder die Ergebnisliste leer ist, wird `0.0` zurückgegeben.
///
/// # Argumente
/// * `ranked_ids` - Nach Relevanz absteigend sortierte Dokument-IDs.
/// * `relevant_ids` - Menge der als relevant geltenden Dokument-IDs.
#[must_use]
pub fn reciprocal_rank<'a, I>(ranked_ids: &[I], relevant_ids: &AHashSet<&str>) -> f32
where
    I: AsRef<str> + 'a,
{
    if ranked_ids.is_empty() || relevant_ids.is_empty() {
        return 0.0;
    }

    for (idx, id) in ranked_ids.iter().enumerate() {
        if relevant_ids.contains(id.as_ref()) {
            #[allow(clippy::cast_precision_loss)]
            let rank = (idx + 1) as f32;
            return 1.0 / rank;
        }
    }

    0.0
}

/// Berechnet den Mean Reciprocal Rank (MRR) über mehrere Abfragen hinweg.
///
/// Liefert das arithmetische Mittel der Reciprocal Ranks aller Abfragen.
/// Bei einer leeren Menge an Abfragen wird `0.0` zurückgegeben.
#[must_use]
pub fn mean_reciprocal_rank<'a, I>(queries: &[(&[I], &AHashSet<&str>)]) -> f32
where
    I: AsRef<str> + 'a,
{
    if queries.is_empty() {
        return 0.0;
    }

    let mut sum = 0.0f32;
    for (ranked_ids, relevant_ids) in queries {
        sum += reciprocal_rank(ranked_ids, relevant_ids);
    }

    #[allow(clippy::cast_precision_loss)]
    let count = queries.len() as f32;
    sum / count
}

/// Berechnet Precision@k (Genauigkeit der Top-k-Ergebnisse).
///
/// Anteil der relevanten Dokumente unter den ersten `k` Treffern:
/// `|relevant ∩ top_k| / k`.
///
/// Falls `k == 0` oder `ranked_ids.is_empty()`, wird `0.0` zurückgegeben.
#[must_use]
pub fn precision_at_k<'a, I>(ranked_ids: &[I], relevant_ids: &AHashSet<&str>, k: usize) -> f32
where
    I: AsRef<str> + 'a,
{
    if k == 0 || ranked_ids.is_empty() || relevant_ids.is_empty() {
        return 0.0;
    }

    let cutoff = k.min(ranked_ids.len());
    let mut relevant_count = 0usize;

    if let Some(slice) = ranked_ids.get(..cutoff) {
        for id in slice {
            if relevant_ids.contains(id.as_ref()) {
                relevant_count += 1;
            }
        }
    }

    #[allow(clippy::cast_precision_loss)]
    let k_f32 = k as f32;
    #[allow(clippy::cast_precision_loss)]
    let rel_f32 = relevant_count as f32;

    (rel_f32 / k_f32).clamp(0.0, 1.0)
}

/// Berechnet Recall@k (Trefferquote bezogen auf alle bekannten relevanten Dokumente).
///
/// Anteil der gefundenen relevanten Dokumente unter den ersten `k` Treffern:
/// `|relevant ∩ top_k| / |relevant|`.
///
/// Falls `k == 0`, `ranked_ids.is_empty()` oder `relevant_ids.is_empty()`, wird `0.0` zurückgegeben.
#[must_use]
pub fn recall_at_k<'a, I>(ranked_ids: &[I], relevant_ids: &AHashSet<&str>, k: usize) -> f32
where
    I: AsRef<str> + 'a,
{
    if k == 0 || ranked_ids.is_empty() || relevant_ids.is_empty() {
        return 0.0;
    }

    let cutoff = k.min(ranked_ids.len());
    let mut relevant_count = 0usize;

    if let Some(slice) = ranked_ids.get(..cutoff) {
        for id in slice {
            if relevant_ids.contains(id.as_ref()) {
                relevant_count += 1;
            }
        }
    }

    #[allow(clippy::cast_precision_loss)]
    let total_relevant = relevant_ids.len() as f32;
    #[allow(clippy::cast_precision_loss)]
    let rel_f32 = relevant_count as f32;

    (rel_f32 / total_relevant).clamp(0.0, 1.0)
}

/// Berechnet den Discounted Cumulative Gain (DCG@k).
///
/// Formel: $\sum_{i=1}^{\min(k, n)} \frac{2^{rel_i} - 1}{\log_2(i + 1)}$
fn dcg_at_k<'a, I>(ranked_ids: &[I], relevance_scores: &AHashMap<&str, f32>, k: usize) -> f64
where
    I: AsRef<str> + 'a,
{
    if k == 0 || ranked_ids.is_empty() || relevance_scores.is_empty() {
        return 0.0;
    }

    let cutoff = k.min(ranked_ids.len());
    let mut dcg = 0.0f64;

    if let Some(slice) = ranked_ids.get(..cutoff) {
        for (idx, id) in slice.iter().enumerate() {
            let rel = relevance_scores
                .get(id.as_ref())
                .copied()
                .unwrap_or(0.0)
                .max(0.0) as f64;
            if rel > 0.0 {
                let rank = (idx + 1) as f64;
                // Standard IR discount: log2(rank + 1)
                let discount = (rank + 1.0).log2();
                if discount > 0.0 {
                    let gain = 2.0f64.powf(rel) - 1.0;
                    dcg += gain / discount;
                }
            }
        }
    }

    dcg
}

/// Berechnet den Normalized Discounted Cumulative Gain (nDCG@k).
///
/// Setzt den tatsächlichen DCG@k ins Verhältnis zum idealen DCG@k (IDCG@k) bei perfekter Sortierung.
///
/// Liefert einen normalisierten Wert zwischen `0.0` und `1.0`.
/// Falls kein relevantes Dokument existiert (IDCG == 0), wird `0.0` zurückgegeben.
///
/// # Argumente
/// * `ranked_ids` - Erhaltene Reihenfolge der Dokument-IDs.
/// * `relevance_scores` - Relevanz-Grade der Dokumente (z. B. 0.0 bis 4.0 oder Wahrscheinlichkeiten).
/// * `k` - Maximale Auswertungstiefe.
#[must_use]
pub fn ndcg_at_k<'a, I>(ranked_ids: &[I], relevance_scores: &AHashMap<&str, f32>, k: usize) -> f32
where
    I: AsRef<str> + 'a,
{
    if k == 0 || ranked_ids.is_empty() || relevance_scores.is_empty() {
        return 0.0;
    }

    let actual_dcg = dcg_at_k(ranked_ids, relevance_scores, k);
    if actual_dcg <= 0.0 {
        return 0.0;
    }

    // Sortiere Relevanzwerte absteigend für IDCG
    let mut ideal_scores: Vec<f32> = relevance_scores
        .values()
        .copied()
        .filter(|&s| s > 0.0)
        .collect();
    ideal_scores.sort_by(|a, b| b.total_cmp(a));

    let ideal_cutoff = k.min(ideal_scores.len());
    let mut idcg = 0.0f64;
    if let Some(slice) = ideal_scores.get(..ideal_cutoff) {
        for (idx, &rel_val) in slice.iter().enumerate() {
            let rel = rel_val as f64;
            let rank = (idx + 1) as f64;
            let discount = (rank + 1.0).log2();
            if discount > 0.0 {
                let gain = 2.0f64.powf(rel) - 1.0;
                idcg += gain / discount;
            }
        }
    }

    if idcg <= 0.0 {
        return 0.0;
    }

    #[allow(clippy::cast_possible_truncation)]
    let result = (actual_dcg / idcg) as f32;
    result.clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reciprocal_rank_basic() {
        let mut relevant = AHashSet::new();
        relevant.insert("doc_b");

        // doc_b on position 1 (0-indexed) => rank 2 => RR = 0.5
        let ranked = ["doc_a", "doc_b", "doc_c"];
        let rr = reciprocal_rank(&ranked, &relevant);
        assert!((rr - 0.5).abs() < 1e-6);

        // doc_b on position 0 => rank 1 => RR = 1.0
        let ranked_top = ["doc_b", "doc_a"];
        let rr_top = reciprocal_rank(&ranked_top, &relevant);
        assert!((rr_top - 1.0).abs() < 1e-6);

        // Not found
        let ranked_none = ["doc_x", "doc_y"];
        assert_eq!(reciprocal_rank(&ranked_none, &relevant), 0.0);
    }

    #[test]
    fn test_mean_reciprocal_rank() {
        let mut rel1 = AHashSet::new();
        rel1.insert("doc_1");
        let q1 = (["doc_1", "doc_2"].as_slice(), &rel1); // RR = 1.0

        let mut rel2 = AHashSet::new();
        rel2.insert("doc_2");
        let q2 = (["doc_1", "doc_2"].as_slice(), &rel2); // RR = 0.5

        let mrr = mean_reciprocal_rank(&[q1, q2]);
        assert!((mrr - 0.75).abs() < 1e-6);
    }

    #[test]
    fn test_precision_and_recall_at_k() {
        let mut relevant = AHashSet::new();
        relevant.insert("doc_1");
        relevant.insert("doc_3");
        relevant.insert("doc_5"); // 3 relevant docs in total

        let ranked = ["doc_1", "doc_2", "doc_3", "doc_4"];

        // At k = 2: doc_1 is relevant -> 1/2 = 0.5 precision, 1/3 recall
        assert!((precision_at_k(&ranked, &relevant, 2) - 0.5).abs() < 1e-6);
        assert!((recall_at_k(&ranked, &relevant, 2) - (1.0 / 3.0)).abs() < 1e-6);

        // At k = 4: doc_1 and doc_3 are relevant -> 2/4 = 0.5 precision, 2/3 recall
        assert!((precision_at_k(&ranked, &relevant, 4) - 0.5).abs() < 1e-6);
        assert!((recall_at_k(&ranked, &relevant, 4) - (2.0 / 3.0)).abs() < 1e-6);

        // At k = 0
        assert_eq!(precision_at_k(&ranked, &relevant, 0), 0.0);
        assert_eq!(recall_at_k(&ranked, &relevant, 0), 0.0);
    }

    #[test]
    fn test_ndcg_at_k_perfect_ranking() {
        let mut rel = AHashMap::new();
        rel.insert("doc_1", 3.0);
        rel.insert("doc_2", 2.0);
        rel.insert("doc_3", 1.0);

        // Perfect ranking
        let perfect = ["doc_1", "doc_2", "doc_3"];
        let ndcg = ndcg_at_k(&perfect, &rel, 3);
        assert!((ndcg - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_ndcg_at_k_inverted_ranking() {
        let mut rel = AHashMap::new();
        rel.insert("doc_1", 3.0);
        rel.insert("doc_2", 1.0);

        // Suboptimal ranking
        let ranked = ["doc_2", "doc_1"];
        let ndcg = ndcg_at_k(&ranked, &rel, 2);
        assert!(ndcg > 0.0 && ndcg < 1.0);
    }

    #[test]
    fn test_metrics_empty_inputs_safety() {
        let empty_rel: AHashSet<&str> = AHashSet::new();
        let empty_map: AHashMap<&str, f32> = AHashMap::new();
        let ranked: [&str; 0] = [];

        assert_eq!(reciprocal_rank(&ranked, &empty_rel), 0.0);
        assert_eq!(mean_reciprocal_rank::<&str>(&[]), 0.0);
        assert_eq!(precision_at_k(&ranked, &empty_rel, 5), 0.0);
        assert_eq!(recall_at_k(&ranked, &empty_rel, 5), 0.0);
        assert_eq!(ndcg_at_k(&ranked, &empty_map, 5), 0.0);
    }
}
