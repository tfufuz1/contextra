// FILE-CONTEXT
// ZWECK: Datentypen für OpenIE Entitätsextraktion.
// INVARIANTEN: No unwrap/expect in non-test paths.
// STAND: 2026-09-07

#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

/// Ein aus dem Text extrahiertes Wissens-Tripel (Subject, Predicate, Object).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExtractedTriple {
    /// Subjekt der Relation / Entität
    pub subject: String,
    /// Prädikat / Relation
    pub predicate: String,
    /// Objekt der Relation / Entität
    pub object: String,
    /// Konfidenzwert des LLM (0.0 bis 1.0)
    pub confidence: f32,
    /// Optionaler Zeichenbereich (Char-Start, Char-Ende) im Ursprungstext
    pub source_span: Option<(usize, usize)>,
}

/// Konfiguration für die automatische Entitätsextraktion.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityExtractionConfig {
    /// Ob Entitätsextraktion aktiviert ist
    pub enabled: bool,
    /// Maximale Anzahl von LLM-Aufrufen pro Zyklus / Extraktionsdurchlauf
    pub max_llm_calls_per_cycle: usize,
    /// Mindestkonfidenz (0.0 bis 1.0) für gefilterte Tripel
    pub min_confidence: f32,
}

impl Default for EntityExtractionConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_llm_calls_per_cycle: 10,
            min_confidence: 0.5,
        }
    }
}
