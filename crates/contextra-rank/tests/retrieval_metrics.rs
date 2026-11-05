//! Retrieval-Metriken (nDCG@k, Recall@k, MRR) als Prüfwerkzeug für Such- und RRF-Rankings.
//!
//! SPEC: AGENTS.md (Fachprofil: Metriken nDCG, Recall@k, MRR als Prüfwerkzeug, T-RANK-1).
//! Alle Metriken arbeiten rein auf Rängen ohne Score-Skalenannahmen zwischen BM25 und Vektor.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]

use contextra_rank::{DocHandle, Fused};
use std::collections::BTreeSet;

/// Berechnet den Recall@k bezüglich einer geordneten Menge relevanter Dokumente (`relevant`).
///
/// $\text{Recall@k} = \frac{|\text{Retrieved@k} \cap \text{Relevant}|}{|\text{Relevant}|}$
/// Falls `relevant` leer ist, ist Recall definiert als 1.0 (trivial erfüllt).
/// Falls $k = 0$, ist Recall 0.0.
fn recall_at_k(ranked: &[Fused], relevant: &BTreeSet<DocHandle>, k: usize) -> f64 {
    if relevant.is_empty() {
        return 1.0;
    }
    if k == 0 || ranked.is_empty() {
        return 0.0;
    }
    let top_k = ranked.iter().take(k);
    let mut hits = 0usize;
    for doc in top_k {
        if relevant.contains(&doc.handle) {
            hits += 1;
        }
    }
    hits as f64 / relevant.len() as f64
}

/// Berechnet den Mean Reciprocal Rank (MRR) bezüglich relevanter Dokumente.
///
/// $\text{MRR} = \frac{1}{\text{Rang des ersten relevanten Dokuments}}$ (1-basiert),
/// oder 0.0, falls kein relevantes Dokument gefunden wird oder $k = 0$.
fn mrr_at_k(ranked: &[Fused], relevant: &BTreeSet<DocHandle>, k: usize) -> f64 {
    if relevant.is_empty() || k == 0 || ranked.is_empty() {
        return 0.0;
    }
    for (idx, doc) in ranked.iter().take(k).enumerate() {
        if relevant.contains(&doc.handle) {
            return 1.0 / (idx as f64 + 1.0);
        }
    }
    0.0
}

/// Berechnet den Discounted Cumulative Gain bei Tiefe k:
/// $\text{DCG@k} = \sum_{i=1}^{\min(k, |ranked|)} \frac{2^{\text{relevance}(doc_i)} - 1}{\log_2(i + 1)}$
///
/// Hier verwendet für binäre Relevanz (relevance $\in \{0, 1\}$), womit $2^1 - 1 = 1$.
fn dcg_at_k(ranked: &[Fused], relevant: &BTreeSet<DocHandle>, k: usize) -> f64 {
    if k == 0 || ranked.is_empty() || relevant.is_empty() {
        return 0.0;
    }
    let mut dcg = 0.0;
    for (idx, doc) in ranked.iter().take(k).enumerate() {
        if relevant.contains(&doc.handle) {
            let rank = (idx + 1) as f64;
            // log2(rank + 1)
            dcg += 1.0 / (rank + 1.0).log2();
        }
    }
    dcg
}

/// Berechnet das normalisierte Discounted Cumulative Gain (nDCG@k) bei Tiefe k:
/// $\text{nDCG@k} = \frac{\text{DCG@k}}{\text{IDCG@k}}$
///
/// Bei leerer Relevanzmenge: 1.0 (oder 0.0, hier 1.0 für leere Menge / triviale Übereinstimmung).
fn ndcg_at_k(ranked: &[Fused], relevant: &BTreeSet<DocHandle>, k: usize) -> f64 {
    if relevant.is_empty() {
        return 1.0;
    }
    if k == 0 || ranked.is_empty() {
        return 0.0;
    }
    let dcg = dcg_at_k(ranked, relevant, k);
    if dcg <= 0.0 {
        return 0.0;
    }

    // IDCG@k: Perfekte Rangliste hätte alle relevanten Dokumente an den obersten Rängen
    let ideal_hits = relevant.len().min(k);
    let mut idcg = 0.0;
    for i in 1..=ideal_hits {
        idcg += 1.0 / (i as f64 + 1.0).log2();
    }

    if idcg <= 0.0 {
        0.0
    } else {
        dcg / idcg
    }
}

/// Helper zum Erzeugen eines deterministischen DocHandle
fn make_doc(id: u8) -> DocHandle {
    DocHandle {
        cluster_tag: [0; 16],
        doc_tag: [id; 16],
    }
}

fn make_fused(id: u8) -> Fused {
    Fused {
        handle: make_doc(id),
        chunk: 0,
        score: contextra_rank::RrfScore(0),
    }
}

/// SPEC: T-RANK-1
/// Prüft Recall@k gegen exakte mathematische Referenzwerte.
/// SPEC: T-RANK-1
#[test]
fn t_eval_1_recall_at_k_reference_values() {
    let _d1 = make_doc(1);
    let d2 = make_doc(2);
    let _d3 = make_doc(3);
    let d4 = make_doc(4);

    let ranked = [make_fused(1), make_fused(2), make_fused(3), make_fused(4)];

    let mut relevant = BTreeSet::new();
    relevant.insert(d2);
    relevant.insert(d4);

    // k = 0: 0.0
    assert_eq!(recall_at_k(&ranked, &relevant, 0), 0.0);

    // k = 1: [d1] -> 0/2 = 0.0
    assert_eq!(recall_at_k(&ranked, &relevant, 1), 0.0);

    // k = 2: [d1, d2] -> 1/2 = 0.5
    assert_eq!(recall_at_k(&ranked, &relevant, 2), 0.5);

    // k = 3: [d1, d2, d3] -> 1/2 = 0.5
    assert_eq!(recall_at_k(&ranked, &relevant, 3), 0.5);

    // k = 4: [d1, d2, d3, d4] -> 2/2 = 1.0
    assert_eq!(recall_at_k(&ranked, &relevant, 4), 1.0);

    // Leere Relevanzmenge -> 1.0
    let empty_relevant = BTreeSet::new();
    assert_eq!(recall_at_k(&ranked, &empty_relevant, 4), 1.0);

    // Leere Trefferliste -> 0.0
    assert_eq!(recall_at_k(&[], &relevant, 4), 0.0);
}

/// SPEC: T-RANK-1
/// Prüft MRR gegen exakte mathematische Referenzwerte.
/// SPEC: T-RANK-1
#[test]
fn t_eval_1_mrr_reference_values() {
    let d1 = make_doc(1);
    let d2 = make_doc(2);
    let d3 = make_doc(3);

    let ranked = [make_fused(1), make_fused(2), make_fused(3)];

    // Erstes relevantes Dokument an Rang 1 -> MRR = 1.0
    let mut rel_r1 = BTreeSet::new();
    rel_r1.insert(d1);
    rel_r1.insert(d3);
    assert_eq!(mrr_at_k(&ranked, &rel_r1, 3), 1.0);

    // Erstes relevantes Dokument an Rang 2 -> MRR = 1/2 = 0.5
    let mut rel_r2 = BTreeSet::new();
    rel_r2.insert(d2);
    assert_eq!(mrr_at_k(&ranked, &rel_r2, 3), 0.5);

    // Erstes relevantes Dokument an Rang 3 -> MRR = 1/3
    let mut rel_r3 = BTreeSet::new();
    rel_r3.insert(d3);
    let mrr_3 = mrr_at_k(&ranked, &rel_r3, 3);
    assert!((mrr_3 - (1.0 / 3.0)).abs() < 1e-9);

    // Kein Treffer in top-2 für d3 -> MRR@2 = 0.0
    assert_eq!(mrr_at_k(&ranked, &rel_r3, 2), 0.0);

    // Grenzfälle
    assert_eq!(mrr_at_k(&[], &rel_r1, 3), 0.0);
    let empty_rel = BTreeSet::new();
    assert_eq!(mrr_at_k(&ranked, &empty_rel, 3), 0.0);
    assert_eq!(mrr_at_k(&ranked, &rel_r1, 0), 0.0);
}

/// SPEC: T-RANK-1
/// Prüft nDCG@k gegen deterministische Referenzwerte (perfektes vs. invertiertes Ranking).
/// SPEC: T-RANK-1
#[test]
fn t_eval_1_ndcg_reference_values() {
    let d1 = make_doc(1);
    let d2 = make_doc(2);
    let _d3 = make_doc(3);

    let mut relevant = BTreeSet::new();
    relevant.insert(d1);
    relevant.insert(d2);

    // Perfektes Ranking: [d1, d2, d3]
    let perfect = [make_fused(1), make_fused(2), make_fused(3)];
    let ndcg_perf = ndcg_at_k(&perfect, &relevant, 3);
    assert!((ndcg_perf - 1.0).abs() < 1e-9);

    // Invertiertes Ranking: [d3, d2, d1]
    // Relevante liegen auf Rang 2 und Rang 3.
    // DCG = 1/log2(3) + 1/log2(4) = 1/1.5849625 + 1/2.0 = 0.63092975 + 0.5 = 1.13092975
    // IDCG = 1/log2(2) + 1/log2(3) = 1/1.0 + 1/1.5849625 = 1.0 + 0.63092975 = 1.63092975
    // nDCG = 1.13092975 / 1.63092975 ≈ 0.693426
    let inverted = [make_fused(3), make_fused(2), make_fused(1)];
    let ndcg_inv = ndcg_at_k(&inverted, &relevant, 3);
    let idcg_expected = 1.0 / 2.0_f64.log2() + 1.0 / 3.0_f64.log2();
    let dcg_expected = 1.0 / 3.0_f64.log2() + 1.0 / 4.0_f64.log2();
    let expected_ratio = dcg_expected / idcg_expected;
    assert!((ndcg_inv - expected_ratio).abs() < 1e-6);

    // Relevante liegen außerhalb k: [d3, d1, d2] bei k = 1 -> DCG = 0 -> nDCG = 0.0
    let none_in_k = [make_fused(3), make_fused(1), make_fused(2)];
    assert_eq!(ndcg_at_k(&none_in_k, &relevant, 1), 0.0);

    // Grenzfälle
    assert_eq!(ndcg_at_k(&[], &relevant, 3), 0.0);
    let empty_rel = BTreeSet::new();
    assert_eq!(ndcg_at_k(&perfect, &empty_rel, 3), 1.0);
    assert_eq!(ndcg_at_k(&perfect, &relevant, 0), 0.0);
}
