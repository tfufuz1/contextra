//! # Beispiel: Autarke Multi-Signal-Rangfusion & IR-Evaluierung
//!
//! Dieses Beispiel demonstriert, wie `contextra-rank` isoliert als reine Bibliothek
//! ohne Datenbank, ohne Netzwerk und ohne Laufzeit für Rangfusion und
//! Qualitätsmessung eingesetzt wird.

use ahash::{AHashMap, AHashSet};
use contextra_rank::fusion::{reciprocal_rank_fusion, SearchResult};
use contextra_rank::metrics::{ndcg_at_k, precision_at_k, recall_at_k, reciprocal_rank};

fn make_result(id: &str, score: f32) -> SearchResult {
    SearchResult {
        id: id.to_string(),
        score,
        metadata: None,
        matched_signals: vec![],
        provenance: None,
    }
}

fn main() {
    println!("=== Contextra-Rank: Multi-Signal Rangfusion & IR-Metriken ===");

    // 1. Zwei Trefferlisten simulieren (z. B. BM25-Volltextsuche und Dense-Vektorsuche)
    let bm25_results = vec![
        make_result("doc_101", 12.5),
        make_result("doc_102", 9.8),
        make_result("doc_103", 5.2),
    ];

    let vector_results = vec![
        make_result("doc_102", 0.94),
        make_result("doc_104", 0.88),
        make_result("doc_101", 0.72),
    ];

    let result_sets = vec![bm25_results, vector_results];

    // 2. Reciprocal Rank Fusion (RRF) mit Standard-Smoothing k=60
    let top_k = 5;
    let fused = reciprocal_rank_fusion(result_sets, top_k);

    println!("\nTop-{top_k} fusionierte Suchergebnisse (RRF):");
    for (rank, res) in fused.iter().enumerate() {
        println!("  #{}: ID={}, RRF-Score={:.6}", rank + 1, res.id, res.score);
    }

    // 3. Evaluation gegen Ground Truth (Relevanz-Urteile)
    let mut relevant_set = AHashSet::new();
    relevant_set.insert("doc_102");
    relevant_set.insert("doc_101");

    let mut relevance_grades = AHashMap::new();
    relevance_grades.insert("doc_102", 3.0); // Hochrelevant
    relevance_grades.insert("doc_101", 1.0); // Relevant

    let ranked_ids: Vec<&str> = fused.iter().map(|r| r.id.as_str()).collect();

    let rr = reciprocal_rank(&ranked_ids, &relevant_set);
    let p_at_2 = precision_at_k(&ranked_ids, &relevant_set, 2);
    let r_at_2 = recall_at_k(&ranked_ids, &relevant_set, 2);
    let ndcg_at_3 = ndcg_at_k(&ranked_ids, &relevance_grades, 3);

    println!("\nInformation-Retrieval Evaluierung:");
    println!("  Reciprocal Rank (RR):   {rr:.4}");
    println!("  Precision@2:            {p_at_2:.4}");
    println!("  Recall@2:               {r_at_2:.4}");
    println!("  nDCG@3:                 {ndcg_at_3:.4}");

    assert!(rr > 0.0, "RR muss positiv sein");
    assert_eq!(p_at_2, 1.0, "Top 2 Dokumente müssen beide relevant sein");
    println!("\nErfolgreich abgeschlossen!");
}
