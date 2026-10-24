// FILE-CONTEXT
// ZWECK: Regelbasierte Entitäts- und Relations-Extraktion aus Freitext mit Graph-Ingestion-Anbindung.
//
// PRODUKTIV-AUFRUFER FUNDSTELLE:
// Ein künftiger Produktiv-Aufrufer müsste im Ingestion-Commit-Pfad angesetzt werden:
// Datei: `crates/contextra-engine/src/collection/crud/auto_extraction.rs`
// Zeile: 140–160 (innerhalb der Funktion `auto_extract_and_relate`)
//
// Dort wird aktuell `extract_triples` aufgerufen. Ein regelbasierter Fallback oder primärer Extractor
// kann an genau dieser Stelle durch Aufruf von `RuleBasedEntityExtractor::extract` und anschließender
// Übergabe der extrahierten Relationen an `ingest_extracted_relations` eingehängt werden.

#![forbid(unsafe_code)]

use crate::csr::{CsrGraph, EdgeType};
use crate::error::GraphMutationError;
use crate::hyperedge::{HyperEdge, HyperEdgeId, RoleBinding, RoleId};
use contextra_types::{DocId, Entity, EntityId, Result};

/// Repräsentiert eine aus Freitext extrahierte Binärbeziehung zwischen zwei Entitäten.
#[derive(Debug, Clone, PartialEq)]
pub struct ExtractedRelation {
    /// Name / Identifikator der Quell-Entität (Subjekt).
    pub source: String,
    /// Name / Identifikator der Ziel-Entität (Objekt).
    pub target: String,
    /// Typ / Bezeichner der Beziehung (Prädikat), z.B. "arbeitet_bei", "wohnt_in".
    pub relation_type: String,
    /// Geschätzter Konfidenzwert der Extraktion im Bereich [0.0, 1.0].
    pub confidence: f32,
}

impl ExtractedRelation {
    /// Erstellt eine neue `ExtractedRelation`.
    pub fn new(
        source: impl Into<String>,
        target: impl Into<String>,
        relation_type: impl Into<String>,
        confidence: f32,
    ) -> Self {
        Self {
            source: source.into(),
            target: target.into(),
            relation_type: relation_type.into(),
            confidence: confidence.clamp(0.0, 1.0),
        }
    }
}

/// Trait für Entitäts- und Relations-Extraktionsstufen.
pub trait EntityExtractor: Send + Sync {
    /// Extrahiert Relationen aus dem gegebenen Freitext.
    fn extract(&self, text: &str) -> Vec<ExtractedRelation>;
}

/// Vordefinierte Satzmuster für die regelbasierte Extraktion.
#[derive(Debug, Clone)]
struct RelationPattern {
    /// Schlüsselphrasen/Prädikate (in Kleinschreibung), z.B. "arbeitet bei", "is connected to".
    triggers: Vec<&'static str>,
    /// Bezeichner des Beziehungstyps.
    relation_type: &'static str,
    /// Basis-Konfidenz für Treffer dieses Musters.
    base_confidence: f32,
}

/// Regelbasierte Referenzimplementierung zur Extraktion von Entitäten und Relationen.
///
/// Nutzt Phrasen-Matching in Kombination mit Erkennung von Großbuchstaben-Substantiven / Eigennamen
/// zur Identifikation von Subjekt- und Objekt-Entitäten.
#[derive(Debug, Clone)]
pub struct RuleBasedEntityExtractor {
    /// Minimale Konfidenzschwelle. Ergebnisse unterhalb dieser Schwelle werden nicht zurückgegeben.
    min_confidence: f32,
    /// Registrierte Extraktionsmuster.
    patterns: Vec<RelationPattern>,
}

impl RuleBasedEntityExtractor {
    /// Erstellt einen neuen `RuleBasedEntityExtractor` mit der angegebenen Mindest-Konfidenz-Schwelle.
    pub fn new(min_confidence: f32) -> Self {
        let patterns = vec![
            RelationPattern {
                triggers: vec!["arbeitet bei", "arbeitet fuer", "works at", "works for"],
                relation_type: "WORKS_AT",
                base_confidence: 0.85,
            },
            RelationPattern {
                triggers: vec!["wohnt in", "lebt in", "lives in", "resides in"],
                relation_type: "LIVES_IN",
                base_confidence: 0.85,
            },
            RelationPattern {
                triggers: vec!["kooperiert mit", "partnered with", "collaborates with"],
                relation_type: "PARTNERED_WITH",
                base_confidence: 0.80,
            },
            RelationPattern {
                triggers: vec!["gruendete", "founded", "hat gegruendet"],
                relation_type: "FOUNDED",
                base_confidence: 0.90,
            },
            RelationPattern {
                triggers: vec![
                    "ist verbunden mit",
                    "is connected to",
                    "gehoert zu",
                    "belongs to",
                ],
                relation_type: "CONNECTED_TO",
                base_confidence: 0.75,
            },
            RelationPattern {
                triggers: vec!["leitet", "manages", "leads", "ist chef von"],
                relation_type: "LEADS",
                base_confidence: 0.85,
            },
        ];

        Self {
            min_confidence: min_confidence.clamp(0.0, 1.0),
            patterns,
        }
    }

    /// Hilfsfunktion zur Bereinigung von Wortmarken / Entitätsbezeichnern.
    fn clean_entity_name(raw: &str) -> String {
        raw.trim()
            .trim_matches(|c: char| !c.is_alphanumeric() && c != '_' && c != '-')
            .to_string()
    }

    /// Extrahieren von Großbuchstaben-Entitätskandidaten vor und nach dem Trigger-Satzteil.
    fn find_entity_candidate(tokens: &[&str]) -> Option<String> {
        let mut selected = Vec::new();

        for token in tokens {
            let clean = Self::clean_entity_name(token);
            if clean.is_empty() {
                continue;
            }
            // Eigenname hat Großbuchstaben zu Beginn
            let first_char = clean.chars().next().unwrap_or('a');
            if first_char.is_uppercase() {
                selected.push(clean);
            } else if !selected.is_empty() {
                // Ende der zusammenhängenden Eigennamensequenz
                break;
            }
        }

        if selected.is_empty() {
            // Fallback: Nimm das letzte/erste nicht-leere Wort falls kein capitalized Wort gefunden wurde
            tokens
                .iter()
                .map(|t| Self::clean_entity_name(t))
                .find(|s| !s.is_empty())
        } else {
            Some(selected.join(" "))
        }
    }
}

impl Default for RuleBasedEntityExtractor {
    fn default() -> Self {
        Self::new(0.50)
    }
}

impl EntityExtractor for RuleBasedEntityExtractor {
    fn extract(&self, text: &str) -> Vec<ExtractedRelation> {
        let mut results = Vec::new();

        // Zerlege Text in Sätze
        let sentences = text.split(['.', '!', '?', ';', '\n']);

        for sentence in sentences {
            let sentence_trimmed = sentence.trim();
            if sentence_trimmed.is_empty() {
                continue;
            }

            let sentence_lower = sentence_trimmed.to_lowercase();

            for pattern in &self.patterns {
                for trigger in &pattern.triggers {
                    if let Some(pos) = sentence_lower.find(trigger) {
                        let left_part = &sentence_trimmed[..pos];
                        let right_part = &sentence_trimmed[pos + trigger.len()..];

                        let left_tokens: Vec<&str> = left_part.split_whitespace().collect();
                        let right_tokens: Vec<&str> = right_part.split_whitespace().collect();

                        // Für Subjekt betrachten wir bevorzugt die Wörter nahe am Trigger (Rückwärts)
                        let left_rev: Vec<&str> = left_tokens.iter().rev().copied().collect();
                        let source_opt = Self::find_entity_candidate(&left_rev)
                            .map(|s| s.split_whitespace().rev().collect::<Vec<&str>>().join(" "));
                        let target_opt = Self::find_entity_candidate(&right_tokens);

                        if let (Some(source), Some(target)) = (source_opt, target_opt) {
                            if !source.is_empty() && !target.is_empty() && source != target {
                                let conf = pattern.base_confidence;
                                if conf >= self.min_confidence {
                                    results.push(ExtractedRelation::new(
                                        source,
                                        target,
                                        pattern.relation_type,
                                        conf,
                                    ));
                                }
                            }
                        }
                    }
                }
            }
        }

        results
    }
}

/// Wandelt extrahierte Relationen oberhalb der Konfidenzschwelle `min_confidence`
/// in Hyperkanten auf dem `CsrGraph` um (`graph.relate_n_ary`).
///
/// Fügt auch fehlende Entitäten über `graph.insert_entity_direct` in den Graph ein.
///
/// # Parameter
/// - `graph`: Referenz auf den Ziel-CSR-Graph.
/// - `relations`: Slice der extrahierten Relationen.
/// - `min_confidence_threshold`: Schwellenwert. Nur Relationen mit `confidence >= threshold` werden verarbeitet.
/// - `doc_id`: Optionale Quell-Dokument-ID für die Herkunftsnachverfolgung (Provenance).
///
/// # Rückgabe
/// Gibt ein `Vec<HyperEdge>` aller erfolgreich im Graph erstellten Hyperkanten zurück.
pub fn ingest_extracted_relations(
    graph: &CsrGraph,
    relations: &[ExtractedRelation],
    min_confidence_threshold: f32,
    doc_id: Option<DocId>,
) -> Result<Vec<HyperEdge>> {
    let mut created_edges = Vec::new();

    for rel in relations {
        if rel.confidence < min_confidence_threshold {
            continue;
        }

        let source_id = EntityId::from_key(&rel.source)?;
        let target_id = EntityId::from_key(&rel.target)?;

        // Stelle sicher, dass Entitäten im Graph existieren
        let source_entity = Entity::new(source_id, &rel.source, "ExtractedEntity");
        let target_entity = Entity::new(target_id, &rel.target, "ExtractedEntity");

        graph.insert_entity_direct(source_entity)?;
        graph.insert_entity_direct(target_entity)?;

        // Erstelle Hyperkanten-IDs
        let max_id = graph.max_hyperedge_id();
        let next_he_id = HyperEdgeId::new(max_id + 1 + created_edges.len() as u64);

        let participants = vec![
            RoleBinding::new(RoleId::new(1), source_id),
            RoleBinding::new(RoleId::new(2), target_id),
        ];

        match graph.relate_n_ary(
            next_he_id,
            EdgeType::Default,
            participants,
            rel.confidence,
            doc_id,
        ) {
            Ok(he) => {
                created_edges.push(he);
            }
            Err(GraphMutationError::DuplicateHyperEdgeId(_)) => {
                // Falls ID kollidiert, versuche mit versetztem Offset
                let alt_id = HyperEdgeId::new(next_he_id.inner() + 1000);
                let participants_alt = vec![
                    RoleBinding::new(RoleId::new(1), source_id),
                    RoleBinding::new(RoleId::new(2), target_id),
                ];
                if let Ok(he) = graph.relate_n_ary(
                    alt_id,
                    EdgeType::Default,
                    participants_alt,
                    rel.confidence,
                    doc_id,
                ) {
                    created_edges.push(he);
                }
            }
            Err(e) => {
                return Err(contextra_types::ContextraError::Internal(format!(
                    "Failed to convert ExtractedRelation to hyperedge: {e}"
                )));
            }
        }
    }

    Ok(created_edges)
}
