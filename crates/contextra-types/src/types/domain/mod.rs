//! Domain types for Contextra.

// FILE-CONTEXT
// STAND: 2026-09-09T14:43:31Z (SESSION: a69d21e4)
// ZWECK: Kanonische Domain-Typen (DocId, EntityId, TxId, TenantId, Embedding, DistanceMetric, Edge, Entity).
// INVARIANTEN: TxId Base Ranges trennen System- (>= INTERNAL_BASE) von Collection-TxIds. TxId NIEMALS aus SystemTime erzeugen.
// HOTSPOTS: 80-600
// NICHT-OFFENSICHTLICH: DocId::from_key nutzt BLAKE3 8-Byte Präfix für deterministisches Slicing.
// SIEHE AUCH: rules/tag_taxonomy.md, DECISIONS.md (ADR-016, ADR-025, ADR-041)

//! # Architektur
//! Enthält die zentralen Domänen-Modelle wie `DocId`, `TxId` und `WorkflowState`.
//! Diese Typen sind die "Lingua Franca" zwischen allen Crates.
//!
//! # Invarianten
//! - `DocId` und `TxId` sind Wrapper um primitive Typen mit deterministischer Hash-Generierung.

mod document;
mod ids;
mod misc;

pub use document::*;
pub use ids::*;
pub use misc::*;

#[cfg(test)]
mod tests;
