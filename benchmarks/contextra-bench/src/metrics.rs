//! Evaluations-Metriken für Information Retrieval und Benchmark-Harness
//!
//! Implementiert:
//! - Recall@k
//! - nDCG@k
//! - MRR@k (Mean Reciprocal Rank)
//! - Latenz-Statistiken (p50, p95, p99)
//! - System-Ressourcen (RAM-Peak / RSS, Indexaufbauzeit)
//!
//! Alle Metrik-Funktionen sind autark, frei von Fremdabhängigkeiten und
//! mit unabhängig handberechneten Testvektoren abgesichert.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::time::Duration;

/// Relevanzurteil für ein Dokument
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RelevanceGrade(pub f64);

/// Ein einzelnes abgerufenes Dokument mit Rang
#[derive(Debug, Clone, PartialEq)]
pub struct RankedResult {
    pub doc_id: String,
    pub score: f64,
}

/// Evaluierungsbericht für eine einzelne Abfrage
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct QueryEvaluation {
    pub query_id: String,
    pub recall_at_10: f64,
    pub ndcg_at_10: f64,
    pub mrr_at_10: f64,
}

/// Aggregierte Gesamtmetriken
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct AggregateMetrics {
    pub total_queries: usize,
    pub recall_at_10: f64,
    pub ndcg_at_10: f64,
    pub mrr_at_10: f64,
    pub latency_p50_us: u64,
    pub latency_p95_us: u64,
    pub latency_p99_us: u64,
    pub peak_rss_bytes: u64,
    pub index_build_duration_ms: u64,
}

/// Berechnet Recall@k
///
/// Recall@k = |relevante Dokumente in Top-k| / |alle relevanten Dokumente|
/// Wenn es keine relevanten Dokumente gibt, wird 0.0 zurückgegeben.
pub fn calculate_recall_at_k(
    retrieved: &[RankedResult],
    ground_truth: &HashMap<String, f64>,
    k: usize,
    relevance_threshold: f64,
) -> f64 {
    let relevant_total = ground_truth
        .values()
        .filter(|&&val| val >= relevance_threshold)
        .count();

    if relevant_total == 0 {
        return 0.0;
    }

    let top_k = retrieved.iter().take(k);
    let mut relevant_retrieved = 0usize;
    let mut seen = HashSet::new();

    for item in top_k {
        if seen.insert(&item.doc_id) {
            if let Some(&rel) = ground_truth.get(&item.doc_id) {
                if rel >= relevance_threshold {
                    relevant_retrieved += 1;
                }
            }
        }
    }

    relevant_retrieved as f64 / relevant_total as f64
}

/// Berechnet Mean Reciprocal Rank (MRR@k) für eine Abfrage
///
/// MRR@k = 1 / erster relevanter Rang (1-basiert) innerhalb von Top-k.
/// Falls kein relevantes Dokument in Top-k ist: 0.0.
pub fn calculate_mrr_at_k(
    retrieved: &[RankedResult],
    ground_truth: &HashMap<String, f64>,
    k: usize,
    relevance_threshold: f64,
) -> f64 {
    for (idx, item) in retrieved.iter().take(k).enumerate() {
        if let Some(&rel) = ground_truth.get(&item.doc_id) {
            if rel >= relevance_threshold {
                return 1.0 / (idx + 1) as f64;
            }
        }
    }
    0.0
}

/// Berechnet Normalized Discounted Cumulative Gain (nDCG@k)
///
/// DCG@k = \sum_{i=1}^k \frac{2^{rel_i} - 1}{\log_2(i + 1)}
/// nDCG@k = DCG@k / IDCG@k
pub fn calculate_ndcg_at_k(
    retrieved: &[RankedResult],
    ground_truth: &HashMap<String, f64>,
    k: usize,
) -> f64 {
    let top_k = retrieved.iter().take(k);
    let mut dcg = 0.0;

    for (idx, item) in top_k.enumerate() {
        let rank = idx + 1;
        let rel = ground_truth.get(&item.doc_id).copied().unwrap_or(0.0);
        if rel > 0.0 {
            let gain = (2.0f64.powf(rel) - 1.0) / (rank as f64 + 1.0).log2();
            dcg += gain;
        }
    }

    if dcg <= 0.0 {
        return 0.0;
    }

    let mut ideal_gains: Vec<f64> = ground_truth.values().copied().collect();
    ideal_gains.sort_by(|a, b| b.total_cmp(a));

    let mut idcg = 0.0;
    for (idx, &rel) in ideal_gains.iter().take(k).enumerate() {
        let rank = idx + 1;
        if rel > 0.0 {
            let gain = (2.0f64.powf(rel) - 1.0) / (rank as f64 + 1.0).log2();
            idcg += gain;
        }
    }

    if idcg <= 0.0 {
        return 0.0;
    }

    dcg / idcg
}

/// Berechnet Perzentile p50, p95, p99 einer Latenzreihe in Mikrosekunden
pub fn calculate_latency_percentiles(mut latencies: Vec<Duration>) -> (u64, u64, u64) {
    if latencies.is_empty() {
        return (0, 0, 0);
    }
    latencies.sort();
    let n = latencies.len();

    let p50_idx = ((n as f64 * 0.50).ceil() as usize)
        .saturating_sub(1)
        .min(n - 1);
    let p95_idx = ((n as f64 * 0.95).ceil() as usize)
        .saturating_sub(1)
        .min(n - 1);
    let p99_idx = ((n as f64 * 0.99).ceil() as usize)
        .saturating_sub(1)
        .min(n - 1);

    (
        latencies[p50_idx].as_micros() as u64,
        latencies[p95_idx].as_micros() as u64,
        latencies[p99_idx].as_micros() as u64,
    )
}

/// Liest den aktuellen VmRSS (Resident Set Size) in Bytes aus /proc/self/status (Linux)
pub fn read_process_rss_bytes() -> u64 {
    if let Ok(status) = fs::read_to_string("/proc/self/status") {
        for line in status.lines() {
            if line.starts_with("VmRSS:") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    if let Ok(kb) = parts[1].parse::<u64>() {
                        return kb * 1024;
                    }
                }
            }
        }
    }
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recall_at_k_hand_calculated() {
        let mut gt = HashMap::new();
        gt.insert("doc1".to_string(), 1.0);
        gt.insert("doc2".to_string(), 1.0);
        gt.insert("doc3".to_string(), 1.0);
        gt.insert("doc4".to_string(), 1.0);

        let retrieved = vec![
            RankedResult {
                doc_id: "doc1".to_string(),
                score: 0.9,
            },
            RankedResult {
                doc_id: "doc5".to_string(),
                score: 0.8,
            },
            RankedResult {
                doc_id: "doc2".to_string(),
                score: 0.7,
            },
            RankedResult {
                doc_id: "doc6".to_string(),
                score: 0.6,
            },
            RankedResult {
                doc_id: "doc7".to_string(),
                score: 0.5,
            },
        ];

        assert!((calculate_recall_at_k(&retrieved, &gt, 1, 1.0) - 0.25).abs() < 1e-9);
        assert!((calculate_recall_at_k(&retrieved, &gt, 2, 1.0) - 0.25).abs() < 1e-9);
        assert!((calculate_recall_at_k(&retrieved, &gt, 3, 1.0) - 0.50).abs() < 1e-9);
        assert!((calculate_recall_at_k(&retrieved, &gt, 10, 1.0) - 0.50).abs() < 1e-9);

        let empty_gt = HashMap::new();
        assert_eq!(calculate_recall_at_k(&retrieved, &empty_gt, 5, 1.0), 0.0);
    }

    #[test]
    fn test_mrr_at_k_hand_calculated() {
        let mut gt = HashMap::new();
        gt.insert("doc3".to_string(), 1.0);
        gt.insert("doc1".to_string(), 1.0);

        let r1 = vec![
            RankedResult {
                doc_id: "doc1".to_string(),
                score: 0.9,
            },
            RankedResult {
                doc_id: "doc2".to_string(),
                score: 0.8,
            },
        ];
        assert!((calculate_mrr_at_k(&r1, &gt, 5, 1.0) - 1.0).abs() < 1e-9);

        let r2 = vec![
            RankedResult {
                doc_id: "doc9".to_string(),
                score: 0.9,
            },
            RankedResult {
                doc_id: "doc8".to_string(),
                score: 0.8,
            },
            RankedResult {
                doc_id: "doc3".to_string(),
                score: 0.7,
            },
        ];
        assert!((calculate_mrr_at_k(&r2, &gt, 5, 1.0) - 1.0 / 3.0).abs() < 1e-9);
        assert_eq!(calculate_mrr_at_k(&r2, &gt, 2, 1.0), 0.0);

        let r3 = vec![RankedResult {
            doc_id: "doc99".to_string(),
            score: 0.5,
        }];
        assert_eq!(calculate_mrr_at_k(&r3, &gt, 5, 1.0), 0.0);
    }

    #[test]
    fn test_ndcg_at_k_hand_calculated() {
        let mut gt = HashMap::new();
        gt.insert("docA".to_string(), 3.0);
        gt.insert("docB".to_string(), 2.0);
        gt.insert("docC".to_string(), 1.0);

        let idcg_hand = 7.0 + (3.0 / 3.0f64.log2()) + 0.5;

        let perfect = vec![
            RankedResult {
                doc_id: "docA".to_string(),
                score: 0.9,
            },
            RankedResult {
                doc_id: "docB".to_string(),
                score: 0.8,
            },
            RankedResult {
                doc_id: "docC".to_string(),
                score: 0.7,
            },
        ];
        let ndcg_perfect = calculate_ndcg_at_k(&perfect, &gt, 3);
        assert!((ndcg_perfect - 1.0).abs() < 1e-9);

        let inverted = vec![
            RankedResult {
                doc_id: "docC".to_string(),
                score: 0.9,
            },
            RankedResult {
                doc_id: "docB".to_string(),
                score: 0.8,
            },
            RankedResult {
                doc_id: "docA".to_string(),
                score: 0.7,
            },
        ];
        let dcg_hand = 1.0 + (3.0 / 3.0f64.log2()) + 3.5;
        let expected_ndcg = dcg_hand / idcg_hand;
        let actual_ndcg = calculate_ndcg_at_k(&inverted, &gt, 3);
        assert!((actual_ndcg - expected_ndcg).abs() < 1e-9);

        let irrelevant = vec![
            RankedResult {
                doc_id: "docX".to_string(),
                score: 0.9,
            },
            RankedResult {
                doc_id: "docY".to_string(),
                score: 0.8,
            },
        ];
        assert_eq!(calculate_ndcg_at_k(&irrelevant, &gt, 3), 0.0);
    }

    #[test]
    fn test_latency_percentiles_hand_calculated() {
        let latencies: Vec<Duration> = (1..=100).map(|i| Duration::from_micros(i)).collect();
        let (p50, p95, p99) = calculate_latency_percentiles(latencies);

        assert_eq!(p50, 50);
        assert_eq!(p95, 95);
        assert_eq!(p99, 99);
    }
}
