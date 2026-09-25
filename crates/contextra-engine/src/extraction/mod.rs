// FILE-CONTEXT
// ZWECK: Moduldefinition für OpenIE Entitätsextraktion (Layer 3 - Engine).
// INVARIANTEN: Zero panic / unwrap / expect in non-test paths.
// STAND: 2026-09-07

//! Modul für LLM-gestützte OpenIE (Open Information Extraction) Entitäts- und Relationsextraktion.
//!
//! # GRENZEN DIESER ERSTVERSION (Scope-Entscheidung)
//! Diese Implementierung stellt eine Erstversion für automatische Extraktion dar:
//! - **Kein Caching**: LLM-Anfragen werden pro Aufruf ohne In-Memory/Disk-Cache ausgeführt.
//! - **Keine Deduplizierung**: Extrahierten Entitäten/Tripel werden nicht gegen bereits in der Graph-DB
//!   existierende Knoten oder Kanten dedupliziert.
//! - **Keine Koreferenzauflösung**: Pronomen oder Verweise ("er", "sie", "das Unternehmen") werden nicht
//!   über Satzgrenzen hinweg auf kanonische Entitätsnamen aufgelöst.

#![forbid(unsafe_code)]

pub mod open_ie;
#[cfg(test)]
pub mod tests;
pub mod types;

pub use open_ie::extract_triples;
pub use types::{EntityExtractionConfig, ExtractedTriple};
