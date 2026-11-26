//! Vergleichsmatrix für Hybrid-Retrieval-Strategien
//!
//! Evaluierungsmodi gemäß Spezifikation WP-113:
//! 1. Vektor allein
//! 2. BM25 allein
//! 3. RRF (Vektor + Text)
//! 4. RRF (Vektor + Text + Graph)
//! 5. Gewichtetes RRF
//! 6. Normalisierte Linearkombination (Score Normalized)
//!
//! Die Fusion nutzt die standardisierte öffentliche API von `contextra-rank`
//! (`contextra_rank::rrf_fuse`, `contextra_rank::weighted_reciprocal_rank_fusion`,
//! `contextra_rank::score_normalized_fusion_with_options`).
//! Es findet KEIN lokaler Nachbau der Fusionslogik statt.

use contextra_rank::{
    rrf_fuse, score_normalized_fusion_with_options, weighted_reciprocal_rank_fusion,
    MetadataMergePriority, ScoredItem, SearchResult,
};
use serde::{Deserialize, Serialize};

use crate::metrics::{AggregateMetrics, RankedResult};

/// Getestete Retrieval-Strategie in der Vergleichsmatrix
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvaluatedStrategy {
    VectorOnly,
    Bm25Only,
    RrfVectorText,
    RrfVectorTextGraph,
    WeightedRrf,
    ScoreNormalizedLinear,
}

impl EvaluatedStrategy {
    pub fn name(&self) -> &'static str {
        match self {
            Self::VectorOnly => "Vektor allein",
            Self::Bm25Only => "BM25 allein",
            Self::RrfVectorText => "RRF (Vektor + Text)",
            Self::RrfVectorTextGraph => "RRF (Vektor + Text + Graph)",
            Self::WeightedRrf => "Gewichtetes RRF",
            Self::ScoreNormalizedLinear => "Normalisierte Linearkombination",
        }
    }
}

/// Ergebnis für eine einzelne Strategie in der Matrix
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StrategyMatrixRow {
    pub strategy: EvaluatedStrategy,
    pub name: String,
    pub metrics: Option<AggregateMetrics>,
    pub status: String,
}

/// Gesamte Vergleichsmatrix über alle Evaluierungsmodi
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ComparisonMatrixReport {
    pub date: String,
    pub dataset_name: String,
    pub total_queries: usize,
    pub total_documents: usize,
    pub rows: Vec<StrategyMatrixRow>,
}

/// Konvertiert interne `RankedResult` in `contextra_rank::ScoredItem`
pub fn to_rank_items(results: &[RankedResult]) -> Vec<ScoredItem> {
    results
        .iter()
        .map(|r| ScoredItem::new(r.doc_id.clone(), r.score as f32))
        .collect()
}

/// Konvertiert `contextra_rank::ScoredItem` zurück in `RankedResult`
pub fn from_rank_items(items: &[ScoredItem]) -> Vec<RankedResult> {
    items
        .iter()
        .map(|i| RankedResult {
            doc_id: i.id.clone(),
            score: i.score as f64,
        })
        .collect()
}

/// Konvertiert interne `RankedResult` in `contextra_rank::SearchResult`
pub fn to_search_results(results: &[RankedResult]) -> Vec<SearchResult> {
    results
        .iter()
        .map(|r| SearchResult {
            id: r.doc_id.clone(),
            score: r.score as f32,
            metadata: None,
            matched_signals: Vec::new(),
            provenance: None,
        })
        .collect()
}

/// Führt RRF-Fusion über die öffentliche `contextra_rank::rrf_fuse` API aus
pub fn fuse_via_rank_rrf(
    lists: &[Vec<ScoredItem>],
    k: usize,
) -> Result<Vec<ScoredItem>, contextra_rank::RankError> {
    rrf_fuse(lists, k)
}

/// Führt gewichtete RRF-Fusion über die öffentliche `contextra_rank::weighted_reciprocal_rank_fusion` API aus
pub fn fuse_via_rank_weighted_rrf(
    signal_sets: Vec<(String, Vec<SearchResult>, f32)>,
    max_results: usize,
) -> Vec<SearchResult> {
    weighted_reciprocal_rank_fusion(signal_sets, max_results)
}

/// Führt normalisierte Linearkombination über `contextra_rank::score_normalized_fusion_with_options` aus
pub fn fuse_via_rank_score_normalized(
    signal_sets: Vec<(String, Vec<SearchResult>, f32)>,
    max_results: usize,
) -> Vec<SearchResult> {
    score_normalized_fusion_with_options(
        signal_sets,
        max_results,
        MetadataMergePriority::default(),
        false,
        None,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rank_api_rrf_integration() {
        let l1 = vec![ScoredItem::new("doc1", 0.9), ScoredItem::new("doc2", 0.8)];
        let l2 = vec![ScoredItem::new("doc2", 0.9), ScoredItem::new("doc1", 0.7)];

        let fused = fuse_via_rank_rrf(&[l1, l2], 60).expect("RRF fusion should succeed");
        assert_eq!(fused.len(), 2);
        assert!((fused[0].score - fused[1].score).abs() < 1e-6);
    }

    #[test]
    fn test_rank_api_weighted_rrf_integration() {
        let s1 = (
            "vector".to_string(),
            vec![SearchResult {
                id: "doc1".to_string(),
                score: 0.9,
                metadata: None,
                matched_signals: vec!["vector".to_string()],
                provenance: None,
            }],
            2.0f32,
        );
        let s2 = (
            "text".to_string(),
            vec![SearchResult {
                id: "doc2".to_string(),
                score: 0.9,
                metadata: None,
                matched_signals: vec!["text".to_string()],
                provenance: None,
            }],
            1.0f32,
        );

        let fused = fuse_via_rank_weighted_rrf(vec![s1, s2], 10);
        assert_eq!(fused.len(), 2);
        assert_eq!(fused[0].id, "doc1");
        assert_eq!(fused[1].id, "doc2");
        assert!(fused[0].score > fused[1].score);
    }

    #[test]
    fn test_rank_api_score_normalized_integration() {
        let s1 = (
            "vector".to_string(),
            vec![
                SearchResult {
                    id: "doc1".to_string(),
                    score: 0.9,
                    metadata: None,
                    matched_signals: vec!["vector".to_string()],
                    provenance: None,
                },
                SearchResult {
                    id: "doc2".to_string(),
                    score: 0.1,
                    metadata: None,
                    matched_signals: vec!["vector".to_string()],
                    provenance: None,
                },
            ],
            1.0f32,
        );

        let fused = fuse_via_rank_score_normalized(vec![s1], 10);
        assert_eq!(fused.len(), 2);
        assert_eq!(fused[0].id, "doc1");
    }
}
