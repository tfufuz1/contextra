//! # Erklärbarkeitsmodul (`explain`)
//!
//! **Ring:** Ring 0 (Ranking & Kalibrierung)
//!
//! Dieses Modul transformiert maschinenlesbare Herkunftsnachweise (`ProvenanceRecord`)
//! in strukturierte Erklärungen (`RetrievalExplanation`) sowie in verständliche,
//! standardisierte deutsche Beschreibungstexte.

use std::fmt;

use serde::{Deserialize, Serialize};

use crate::fusion::ProvenanceRecord;

pub use crate::fusion::SignalKind;

/// Breakdown entry representing the contribution of a single search signal.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
// ANCHOR[contextra-rank:EXP-001]
/// Einzelner Beitrags-Eintrag für eine spezifische Signalquelle innerhalb der Erklärung.
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
// ANCHOR[contextra-rank:EXP-002]
/// Strukturierte Gesamterklärung zur Nachvollziehbarkeit eines Retrieval-Ergebnisses.
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

// ADR-098 (Prozentwert-Anzeige: f32 -> u32 Konvertierung)
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn round_share_to_pct(share: f32) -> u32 {
    let scaled = (share * 100.0).round();
    if !scaled.is_finite() || scaled <= 0.0 {
        return 0;
    }
    if scaled >= 100.0 {
        return 100;
    }
    scaled as u32
}

impl RetrievalExplanation {
    /// Generates a human-readable German explanation string for audit logging or CLI output.
    ///
    /// Formats all signal contributions using exact ranks and normalized percentage shares,
    /// along with any resonance coherence bonus or collection source.
    // ANCHOR[contextra-rank:EXP-003]
    /// Erzeugt eine verständliche deutsche Textzusammenfassung für Audit-Logs oder UIs.
    pub fn to_human_readable_de(&self) -> String {
        if self.entries.is_empty() {
            return "Keine Treffersignale für dieses Dokument vorhanden.".to_string();
        }

        let mut signal_texts = Vec::with_capacity(self.entries.len());
        for (idx, entry) in self.entries.iter().enumerate() {
            let pct = round_share_to_pct(entry.contribution_share);
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
type RawSignalEntry = (SignalKind, f32, Option<u32>, Option<f32>);

fn extract_raw_signal_entries(record: &ProvenanceRecord) -> Vec<RawSignalEntry> {
    let mut raw_entries = Vec::new();

    let signal_configs: &[(SignalKind, Option<f32>, &[&str])] = &[
        (
            SignalKind::Vector,
            record.vector_distance,
            &["vector", "vec"],
        ),
        (
            SignalKind::Text,
            record.bm25_score,
            &["text", "bm25", "keyword"],
        ),
        (SignalKind::Graph, record.graph_score, &["graph"]),
        (
            SignalKind::EdgeReinforcement,
            record.rerank_score,
            &["rerank", "edge-reinforcement"],
        ),
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

    raw_entries
}

fn compute_contribution_shares(raw_entries: &[RawSignalEntry]) -> Vec<f32> {
    if raw_entries.len() == 1 {
        return vec![1.0];
    }

    let all_have_rrf = raw_entries
        .iter()
        .all(|(_, _, _, rrf)| matches!(rrf, Some(v) if *v >= 0.0 && v.is_finite()));
    let sum_rrf: f32 = raw_entries
        .iter()
        .map(|(_, _, _, rrf)| rrf.unwrap_or(0.0))
        .sum();

    if all_have_rrf && sum_rrf > 0.0 && sum_rrf.is_finite() {
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
    }
}

/// Generates a structured `RetrievalExplanation` from a `ProvenanceRecord`.
///
/// Computes per-signal `contribution_share` normalized such that all present signals sum to `1.0`
/// (or `0.0` if no signals are present).
// ANCHOR[contextra-rank:EXP-004]
/// Erzeugt aus einem maschinenlesbaren `ProvenanceRecord` eine strukturierte `RetrievalExplanation`.
pub fn explain(record: &ProvenanceRecord) -> RetrievalExplanation {
    let raw_entries = extract_raw_signal_entries(record);

    if raw_entries.is_empty() {
        return RetrievalExplanation {
            entries: Vec::new(),
            dominant_signal: None,
            coherence_bonus: record.coherence_bonus,
            source_collection: record.source_collection.clone(),
        };
    }

    let shares = compute_contribution_shares(&raw_entries);

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
        let record = ProvenanceRecord {
            bm25_score: Some(12.5),
            ..Default::default()
        };
        let explanation = explain(&record);
        assert_eq!(explanation.entries.len(), 1);
        if let Some(entry) = explanation.entries.first() {
            assert_eq!(entry.signal, SignalKind::Text);
            assert_eq!(entry.contribution_share, 1.0);
        }
        assert_eq!(explanation.dominant_signal, Some(SignalKind::Text));
    }

    #[test]
    fn test_four_signals_exact_contributions() {
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

        let record = ProvenanceRecord {
            vector_distance: Some(0.1),
            bm25_score: Some(15.0),
            graph_score: Some(0.8),
            rerank_score: Some(0.95),
            signal_contributions: contribs,
            ..Default::default()
        };

        let explanation = explain(&record);
        assert_eq!(explanation.entries.len(), 4);

        if let Some(e) = explanation.entries.first() {
            assert_eq!(e.signal, SignalKind::Vector);
            assert_eq!(e.contribution_share, 0.40);
        }
        if let Some(e) = explanation.entries.get(1) {
            assert_eq!(e.signal, SignalKind::Text);
            assert_eq!(e.contribution_share, 0.30);
        }
        if let Some(e) = explanation.entries.get(2) {
            assert_eq!(e.signal, SignalKind::Graph);
            assert_eq!(e.contribution_share, 0.20);
        }
        if let Some(e) = explanation.entries.get(3) {
            assert_eq!(e.signal, SignalKind::EdgeReinforcement);
            assert_eq!(e.contribution_share, 0.10);
        }

        assert_eq!(explanation.dominant_signal, Some(SignalKind::Vector));
    }

    #[test]
    fn test_sorting_descending_by_contribution_share() {
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

        let record = ProvenanceRecord {
            vector_distance: Some(0.5),
            bm25_score: Some(2.0),
            graph_score: Some(10.0),
            signal_contributions: contribs,
            ..Default::default()
        };

        let explanation = explain(&record);
        assert_eq!(explanation.entries.len(), 3);
        if let (Some(e0), Some(e1), Some(e2)) = (
            explanation.entries.first(),
            explanation.entries.get(1),
            explanation.entries.get(2),
        ) {
            assert_eq!(e0.signal, SignalKind::Graph);
            assert_eq!(e1.signal, SignalKind::Text);
            assert_eq!(e2.signal, SignalKind::Vector);
            assert!(e0.contribution_share >= e1.contribution_share);
            assert!(e1.contribution_share >= e2.contribution_share);
        }
    }

    #[test]
    fn test_signal_kind_golden_serde_and_display() {
        let kinds = [
            (SignalKind::Vector, "vector", "Vektor-Ähnlichkeit"),
            (SignalKind::Text, "text", "BM25-Textsuche"),
            (SignalKind::Graph, "graph", "Graph-Traversierung"),
            (
                SignalKind::EdgeReinforcement,
                "edge-reinforcement",
                "Rerank-Bewertung",
            ),
        ];

        for (kind, expected_json_str, expected_de) in kinds {
            assert_eq!(kind.as_str(), expected_json_str);
            assert_eq!(kind.name_de(), expected_de);
            assert_eq!(format!("{kind}"), expected_de);

            if let Ok(json) = serde_json::to_string(&kind) {
                assert_eq!(json, format!("\"{expected_json_str}\""));
                if let Ok(de_kind) = serde_json::from_str::<SignalKind>(&json) {
                    assert_eq!(de_kind, kind);
                }
            }
        }
    }

    #[test]
    fn test_to_human_readable_de() {
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

        let record = ProvenanceRecord {
            bm25_score: Some(10.0),
            vector_distance: Some(0.2),
            coherence_bonus: 0.03,
            source_collection: Some("knowledge_base".to_string()),
            signal_contributions: contribs,
            ..Default::default()
        };

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
