//! Provenance explanation module converting machine-readable search provenance
//! into structured explanation objects and German human-readable text.

use std::fmt;

use serde::{Deserialize, Serialize};

use crate::fusion::ProvenanceRecord;

/// Search signal kind used during multi-signal fusion.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SignalKind {
    /// Vector (semantic k-NN) search signal.
    Vector,
    /// Text (BM25 keyword) search signal.
    Bm25,
    /// Graph (traversal / PageRank) search signal.
    Graph,
    /// Cross-encoder reranking signal.
    Rerank,
}

impl SignalKind {
    /// Returns the German display name for the signal.
    pub fn name_de(&self) -> &'static str {
        match self {
            SignalKind::Vector => "Vektor-Ähnlichkeit",
            SignalKind::Bm25 => "BM25-Textsuche",
            SignalKind::Graph => "Graph-Traversierung",
            SignalKind::Rerank => "Rerank-Bewertung",
        }
    }
}

impl fmt::Display for SignalKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name_de())
    }
}

/// Breakdown entry representing the contribution of a single search signal.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExplanationEntry {
    /// Identified search signal type.
    pub signal: SignalKind,
    /// Raw unnormalized score or distance.
    pub raw_score: f32,
    /// 1-based rank in the signal's result set, if available.
    pub rank: Option<u32>,
    /// Optional signal weight.
    pub weight: Option<f32>,
    /// Normalized share of total score contribution (range 0.0 to 1.0).
    pub contribution_share: f32,
}

/// Structured explanation detailing why a document was retrieved with its given rank.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RetrievalExplanation {
    /// Individual signal contribution entries, sorted descending by `contribution_share`.
    pub entries: Vec<ExplanationEntry>,
    /// Dominant search signal contributing most to the retrieved item's score.
    pub dominant_signal: Option<SignalKind>,
    /// Resonance coherence bonus value added during score fusion.
    pub coherence_bonus: f32,
    /// Source collection name, if specified in provenance.
    pub source_collection: Option<String>,
}

impl RetrievalExplanation {
    /// Generates a human-readable German explanation string for audit logging or CLI output.
    ///
    /// Formats all signal contributions using exact ranks and normalized percentage shares,
    /// along with any resonance coherence bonus or collection source.
    pub fn to_human_readable_de(&self) -> String {
        if self.entries.is_empty() {
            return "Keine Treffersignale für dieses Dokument vorhanden.".to_string();
        }

        let mut signal_texts = Vec::with_capacity(self.entries.len());
        for (idx, entry) in self.entries.iter().enumerate() {
            let pct = (entry.contribution_share * 100.0).round() as u32;
            let rank_str = entry
                .rank
                .map(|r| format!("Rang {r}"))
                .unwrap_or_else(|| "kein Rang".to_string());

            if idx == 0 {
                signal_texts.push(format!(
                    "Dokument primär über {} gefunden ({rank_str}, Beitrag {pct} %)",
                    entry.signal.name_de()
                ));
            } else {
                signal_texts.push(format!(
                    "zusätzlich bestätigt durch {} ({rank_str}, Beitrag {pct} %)",
                    entry.signal.name_de()
                ));
            }
        }

        let mut result = signal_texts.join(", ");
        result.push('.');

        if self.coherence_bonus != 0.0 {
            if self.coherence_bonus > 0.0 {
                result.push_str(&format!(" Kohärenz-Bonus: +{:.2}.", self.coherence_bonus));
            } else {
                result.push_str(&format!(" Kohärenz-Bonus: {:.2}.", self.coherence_bonus));
            }
        }

        if let Some(ref col) = self.source_collection {
            result.push_str(&format!(" [Kollektion: {col}]"));
        }

        result
    }
}

impl fmt::Display for RetrievalExplanation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_human_readable_de())
    }
}

/// Generates a structured `RetrievalExplanation` from a `ProvenanceRecord`.
///
/// Computes per-signal `contribution_share` normalized such that all present signals sum to `1.0`
/// (or `0.0` if no signals are present). Prefers using `signal_contributions` (`rrf_contribution`)
/// if present and consistent; otherwise falls back to normalizing raw positive scores.
pub fn explain(record: &ProvenanceRecord) -> RetrievalExplanation {
    let mut raw_entries = Vec::new();

    let signal_configs: &[(SignalKind, Option<f32>, &[&str])] = &[
        (
            SignalKind::Vector,
            record.vector_distance,
            &["vector", "vec"],
        ),
        (
            SignalKind::Bm25,
            record.bm25_score,
            &["text", "bm25", "keyword"],
        ),
        (SignalKind::Graph, record.graph_score, &["graph"]),
        (SignalKind::Rerank, record.rerank_score, &["rerank"]),
    ];

    for &(kind, raw_score_opt, keys) in signal_configs {
        let contrib_opt = keys
            .iter()
            .find_map(|k| record.signal_contributions.get(*k));
        let rank_opt = contrib_opt.map(|c| c.rank).or_else(|| {
            keys.iter()
                .find_map(|k| record.signal_ranks.get(*k).copied())
        });

        let is_present = raw_score_opt.is_some() || contrib_opt.is_some() || rank_opt.is_some();

        if is_present {
            let raw_score = raw_score_opt
                .or_else(|| contrib_opt.map(|c| c.raw_score))
                .unwrap_or(0.0);
            let rrf_contrib = contrib_opt.map(|c| c.rrf_contribution);

            raw_entries.push((kind, raw_score, rank_opt, rrf_contrib));
        }
    }

    if raw_entries.is_empty() {
        return RetrievalExplanation {
            entries: Vec::new(),
            dominant_signal: None,
            coherence_bonus: record.coherence_bonus,
            source_collection: record.source_collection.clone(),
        };
    }

    let all_have_rrf = raw_entries
        .iter()
        .all(|(_, _, _, rrf)| matches!(rrf, Some(v) if *v >= 0.0 && v.is_finite()));
    let sum_rrf: f32 = raw_entries
        .iter()
        .map(|(_, _, _, rrf)| rrf.unwrap_or(0.0))
        .sum();

    let shares: Vec<f32> = if raw_entries.len() == 1 {
        vec![1.0]
    } else if all_have_rrf && sum_rrf > 0.0 && sum_rrf.is_finite() {
        raw_entries
            .iter()
            .map(|(_, _, _, rrf)| rrf.unwrap_or(0.0) / sum_rrf)
            .collect()
    } else {
        let sum_raw: f32 = raw_entries.iter().map(|(_, raw, _, _)| raw.max(0.0)).sum();
        if sum_raw > 0.0 && sum_raw.is_finite() {
            raw_entries
                .iter()
                .map(|(_, raw, _, _)| raw.max(0.0) / sum_raw)
                .collect()
        } else {
            let equal_share = 1.0 / (raw_entries.len() as f32);
            vec![equal_share; raw_entries.len()]
        }
    };

    let mut entries: Vec<ExplanationEntry> = raw_entries
        .into_iter()
        .zip(shares)
        .map(
            |((signal, raw_score, rank, _), contribution_share)| ExplanationEntry {
                signal,
                raw_score,
                rank,
                weight: None,
                contribution_share,
            },
        )
        .collect();

    entries.sort_by(|a, b| b.contribution_share.total_cmp(&a.contribution_share));

    let dominant_signal = entries.first().map(|e| e.signal);

    RetrievalExplanation {
        entries,
        dominant_signal,
        coherence_bonus: record.coherence_bonus,
        source_collection: record.source_collection.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fusion::SignalContribution;
    use ahash::AHashMap;

    #[test]
    fn test_empty_provenance_record() {
        let record = ProvenanceRecord::default();
        let explanation = explain(&record);
        assert!(explanation.entries.is_empty());
        assert_eq!(explanation.dominant_signal, None);
        assert_eq!(explanation.coherence_bonus, 0.0);
        assert_eq!(explanation.source_collection, None);
    }

    #[test]
    fn test_single_signal_record() {
        let mut record = ProvenanceRecord::default();
        record.bm25_score = Some(12.5);
        let explanation = explain(&record);
        assert_eq!(explanation.entries.len(), 1);
        assert_eq!(explanation.entries[0].signal, SignalKind::Bm25);
        assert_eq!(explanation.entries[0].contribution_share, 1.0);
        assert_eq!(explanation.dominant_signal, Some(SignalKind::Bm25));
    }

    #[test]
    fn test_four_signals_exact_contributions() {
        let mut record = ProvenanceRecord::default();
        record.vector_distance = Some(0.1);
        record.bm25_score = Some(15.0);
        record.graph_score = Some(0.8);
        record.rerank_score = Some(0.95);

        let mut contribs = AHashMap::new();
        contribs.insert(
            "vector".to_string(),
            SignalContribution {
                raw_score: 0.1,
                rank: 1,
                rrf_contribution: 0.40,
            },
        );
        contribs.insert(
            "text".to_string(),
            SignalContribution {
                raw_score: 15.0,
                rank: 2,
                rrf_contribution: 0.30,
            },
        );
        contribs.insert(
            "graph".to_string(),
            SignalContribution {
                raw_score: 0.8,
                rank: 3,
                rrf_contribution: 0.20,
            },
        );
        contribs.insert(
            "rerank".to_string(),
            SignalContribution {
                raw_score: 0.95,
                rank: 4,
                rrf_contribution: 0.10,
            },
        );
        record.signal_contributions = contribs;

        let explanation = explain(&record);
        assert_eq!(explanation.entries.len(), 4);

        // Sum of rrf_contributions = 0.40 + 0.30 + 0.20 + 0.10 = 1.00
        // Vector share = 0.40 / 1.00 = 0.40
        // Bm25 share = 0.30 / 1.00 = 0.30
        // Graph share = 0.20 / 1.00 = 0.20
        // Rerank share = 0.10 / 1.00 = 0.10
        assert_eq!(explanation.entries[0].signal, SignalKind::Vector);
        assert_eq!(explanation.entries[0].contribution_share, 0.40);

        assert_eq!(explanation.entries[1].signal, SignalKind::Bm25);
        assert_eq!(explanation.entries[1].contribution_share, 0.30);

        assert_eq!(explanation.entries[2].signal, SignalKind::Graph);
        assert_eq!(explanation.entries[2].contribution_share, 0.20);

        assert_eq!(explanation.entries[3].signal, SignalKind::Rerank);
        assert_eq!(explanation.entries[3].contribution_share, 0.10);

        assert_eq!(explanation.dominant_signal, Some(SignalKind::Vector));
    }

    #[test]
    fn test_sorting_descending_by_contribution_share() {
        let mut record = ProvenanceRecord::default();
        record.vector_distance = Some(0.5);
        record.bm25_score = Some(2.0);
        record.graph_score = Some(10.0);

        let mut contribs = AHashMap::new();
        contribs.insert(
            "vector".to_string(),
            SignalContribution {
                raw_score: 0.5,
                rank: 5,
                rrf_contribution: 0.05,
            },
        );
        contribs.insert(
            "text".to_string(),
            SignalContribution {
                raw_score: 2.0,
                rank: 2,
                rrf_contribution: 0.25,
            },
        );
        contribs.insert(
            "graph".to_string(),
            SignalContribution {
                raw_score: 10.0,
                rank: 1,
                rrf_contribution: 0.70,
            },
        );
        record.signal_contributions = contribs;

        let explanation = explain(&record);
        assert_eq!(explanation.entries.len(), 3);
        assert_eq!(explanation.entries[0].signal, SignalKind::Graph);
        assert_eq!(explanation.entries[1].signal, SignalKind::Bm25);
        assert_eq!(explanation.entries[2].signal, SignalKind::Vector);

        assert!(
            explanation.entries[0].contribution_share >= explanation.entries[1].contribution_share
        );
        assert!(
            explanation.entries[1].contribution_share >= explanation.entries[2].contribution_share
        );
    }

    #[test]
    fn test_to_human_readable_de() {
        let mut record = ProvenanceRecord::default();
        record.bm25_score = Some(10.0);
        record.vector_distance = Some(0.2);
        record.coherence_bonus = 0.03;
        record.source_collection = Some("knowledge_base".to_string());

        let mut contribs = AHashMap::new();
        contribs.insert(
            "text".to_string(),
            SignalContribution {
                raw_score: 10.0,
                rank: 2,
                rrf_contribution: 0.61,
            },
        );
        contribs.insert(
            "vector".to_string(),
            SignalContribution {
                raw_score: 0.2,
                rank: 5,
                rrf_contribution: 0.24,
            },
        );
        record.signal_contributions = contribs;

        let explanation = explain(&record);
        let text = explanation.to_human_readable_de();
        assert!(text.contains("Dokument primär über BM25-Textsuche gefunden"));
        assert!(text.contains("Rang 2"));
        assert!(text.contains("zusätzlich bestätigt durch Vektor-Ähnlichkeit"));
        assert!(text.contains("Rang 5"));
        assert!(text.contains("Kohärenz-Bonus: +0.03"));
        assert!(text.contains("[Kollektion: knowledge_base]"));

        let display_text = format!("{explanation}");
        assert_eq!(text, display_text);
    }
}
