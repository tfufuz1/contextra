// FILE-CONTEXT
// ZWECK: Regelbasierte Entitäts- und Relationsextraktion (Layer 3 - Engine).
// INVARIANTEN: Zero panic / unwrap / expect in non-test paths; Rein, deterministisch, ohne Uhr, ohne Rng, ohne Netz.
// STAND: 2026-10-01

#![forbid(unsafe_code)]

use super::types::{EntityExtractionConfig, ExtractedTriple};
use contextra_types::{ContextraError, Result};

/// Maximale zulässige Eingabelänge in Zeichen für die regelbasierte Extraktion.
pub const MAX_EXTRACTION_TEXT_LENGTH: usize = 500_000;

/// Maximale Anzahl extrahierter Tripel pro Durchlauf.
pub const MAX_EXTRACTED_TRIPLES: usize = 5_000;

/// Vordefiniertes Satzmuster für die regelbasierte Extraktion.
#[derive(Debug, Clone)]
struct SPOPattern {
    triggers: &'static [&'static str],
    predicate: &'static str,
    base_confidence: f32,
}

/// Statisch konfigurierte SPO-Muster (Deutsch und Englisch).
static SPO_PATTERNS: &[SPOPattern] = &[
    SPOPattern {
        triggers: &["arbeitet bei", "arbeitet fuer", "works at", "works for"],
        predicate: "WORKS_AT",
        base_confidence: 0.85,
    },
    SPOPattern {
        triggers: &["wohnt in", "lebt in", "lives in", "resides in"],
        predicate: "LIVES_IN",
        base_confidence: 0.85,
    },
    SPOPattern {
        triggers: &["kooperiert mit", "collaborates with", "partnered with"],
        predicate: "PARTNERED_WITH",
        base_confidence: 0.80,
    },
    SPOPattern {
        triggers: &["gruendete", "hat gegruendet", "founded"],
        predicate: "FOUNDED",
        base_confidence: 0.90,
    },
    SPOPattern {
        triggers: &[
            "ist verbunden mit",
            "is connected to",
            "belongs to",
            "gehoert zu",
        ],
        predicate: "CONNECTED_TO",
        base_confidence: 0.75,
    },
    SPOPattern {
        triggers: &["leitet", "manages", "leads", "ist chef von"],
        predicate: "LEADS",
        base_confidence: 0.85,
    },
];

/// Hilfskonstrukt zur Komposita-Erkennung und -Schutz im Deutschen ohne externe Bibliotheken.
/// Schützt typische deutsche Komposita-Endungen vor fehlerhafter Trennung.
fn is_protected_compound(word: &str) -> bool {
    let lower = word.to_lowercase();
    lower.ends_with("gesellschaft")
        || lower.ends_with("unternehmen")
        || lower.ends_with("organisation")
        || lower.ends_with("verein")
        || lower.ends_with("verband")
        || lower.ends_with("stiftung")
        || lower.ends_with("institut")
        || lower.ends_with("gruppe")
        || lower.ends_with("ag")
        || lower.ends_with("gmbh")
}

/// Bereinigt Entitätsnamen von umschließenden Satzzeichen.
fn clean_entity_token(raw: &str) -> &str {
    raw.trim_matches(|c: char| !c.is_alphanumeric() && c != '_' && c != '-')
}

/// Identifiziert zusammenhängende Eigennamen / Großschreibungs-Kandidaten.
fn extract_entity_candidate(tokens: &[&str]) -> Option<String> {
    let mut chosen = Vec::new();

    for token in tokens {
        let cleaned = clean_entity_token(token);
        if cleaned.is_empty() {
            continue;
        }

        let is_comp = is_protected_compound(cleaned);
        let first_char = cleaned.chars().next().unwrap_or('a');

        if first_char.is_uppercase() || is_comp {
            chosen.push(cleaned.to_string());
        } else if !chosen.is_empty() {
            break;
        }
    }

    if chosen.is_empty() {
        tokens
            .iter()
            .map(|t| clean_entity_token(t))
            .find(|s| !s.is_empty())
            .map(|s| s.to_string())
    } else {
        Some(chosen.join(" "))
    }
}

/// Regelbasierter Entitäts- und Relationsextraktor.
#[derive(Debug, Clone, Default)]
pub struct RuleBasedExtractor;

impl RuleBasedExtractor {
    /// Erstellt eine neue Instanz von `RuleBasedExtractor`.
    pub fn new() -> Self {
        Self
    }

    /// Extrahieren von Wissens-Tripeln rein regelbasiert und deterministisch.
    pub fn extract(
        &self,
        text: &str,
        config: &EntityExtractionConfig,
    ) -> Result<Vec<ExtractedTriple>> {
        if !config.enabled || text.trim().is_empty() {
            return Ok(Vec::new());
        }

        let char_len = text.chars().count();
        if char_len > MAX_EXTRACTION_TEXT_LENGTH {
            return Err(ContextraError::limit_exceeded(
                MAX_EXTRACTION_TEXT_LENGTH,
                format!(
                    "Text length of {char_len} chars exceeds maximum allowed length of {MAX_EXTRACTION_TEXT_LENGTH}"
                ),
            ));
        }

        let mut raw_triples = Vec::new();

        // Zerlegen nach Sätzen unter Wahrung der Zeichenindizes im Originaltext
        let mut sentence_spans = Vec::new();
        let mut start_idx = 0;

        for (idx, ch) in text.char_indices() {
            if ch == '.' || ch == '!' || ch == '?' || ch == '\n' || ch == ';' {
                let slice = &text[start_idx..idx];
                if !slice.trim().is_empty() {
                    sentence_spans.push((start_idx, idx, slice));
                }
                start_idx = idx + ch.len_utf8();
            }
        }
        if start_idx < text.len() {
            let slice = &text[start_idx..];
            if !slice.trim().is_empty() {
                sentence_spans.push((start_idx, text.len(), slice));
            }
        }

        for (sent_start, sent_end, sent_text) in sentence_spans {
            let sent_lower = sent_text.to_lowercase();

            for pattern in SPO_PATTERNS {
                for &trigger in pattern.triggers {
                    if let Some(rel_pos) = sent_lower.find(trigger) {
                        let left_part = &sent_text[..rel_pos];
                        let right_part = &sent_text[rel_pos + trigger.len()..];

                        let left_tokens: Vec<&str> = left_part.split_whitespace().collect();
                        let right_tokens: Vec<&str> = right_part.split_whitespace().collect();

                        let left_rev: Vec<&str> = left_tokens.iter().rev().copied().collect();
                        let subj_opt = extract_entity_candidate(&left_rev)
                            .map(|s| s.split_whitespace().rev().collect::<Vec<&str>>().join(" "));
                        let obj_opt = extract_entity_candidate(&right_tokens);

                        if let (Some(subject), Some(object)) = (subj_opt, obj_opt) {
                            if !subject.is_empty() && !object.is_empty() && subject != object {
                                let conf = pattern.base_confidence;
                                if conf >= config.min_confidence {
                                    raw_triples.push(ExtractedTriple {
                                        subject,
                                        predicate: pattern.predicate.to_string(),
                                        object,
                                        confidence: conf,
                                        source_span: Some((sent_start, sent_end)),
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }

        if raw_triples.len() > MAX_EXTRACTED_TRIPLES {
            return Err(ContextraError::limit_exceeded(
                MAX_EXTRACTED_TRIPLES,
                format!(
                    "Extracted triple count {} exceeds maximum limit of {}",
                    raw_triples.len(),
                    MAX_EXTRACTED_TRIPLES
                ),
            ));
        }

        // Stabile Sortierung der Ausgabe (Determinismus)
        // Sortierschlüssel: (source_span.start, source_span.end, subject, predicate, object)
        raw_triples.sort_by(|a, b| {
            let span_a = a.source_span.unwrap_or((0, 0));
            let span_b = b.source_span.unwrap_or((0, 0));

            span_a
                .cmp(&span_b)
                .then_with(|| a.subject.cmp(&b.subject))
                .then_with(|| a.predicate.cmp(&b.predicate))
                .then_with(|| a.object.cmp(&b.object))
        });

        // Deduplizierung aufeinanderfolgender identischer Tripel nach Sortierung
        raw_triples.dedup();

        Ok(raw_triples)
    }
}
