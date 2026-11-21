//! Domänentypen für Contextra.
//!
//! # Architektur-Rolle: Typen-Fundament (Kanonische Domänenmodelle)
//!
//! Enthält die zentralen Domänen-Modelle wie `DocId`, `TxId`, `TenantId`, `Embedding` und `WorkflowState`.
//! Diese Typen sind die "Lingua Franca" zwischen allen Crates im Workspace.
//!
//! # Invarianten
//! - `DocId` und `TxId` sind Newtypes um primitive Typen mit deterministischer Hash-Generierung.
//! - TxId Base Ranges trennen System- (`>= INTERNAL_BASE`) strikt von Collection-Transaktions-IDs.
//! - Keine Erzeugung von Zeit oder Zufall ohne explizite Parameter oder Injektion.

// SIEHE AUCH: rules/tag_taxonomy.md, DECISIONS.md (ADR-016, ADR-025, ADR-041)

//! # Architektur
//! Enthält die zentralen Domänen-Modelle wie `DocId`, `TxId` und `WorkflowState`.
//! Diese Typen sind die "Lingua Franca" zwischen allen Crates.
//!
//! # Invarianten
//! - `DocId` und `TxId` sind Wrapper um primitive Typen mit deterministischer Hash-Generierung.

/// Dokumente, Chunks, Embeddings und Vektor-Distanzmetriken.
mod document;
/// Identifikatoren: DocId, EntityId, TxId, TenantId, CollectionId.
mod ids;
/// Wissensgraphen-Elemente (Entity, Edge), MemoryType, WorkflowState und Verfallskonstanten.
mod misc;

pub use document::*;
pub use ids::*;
pub use misc::*;

#[cfg(test)]
mod tests;
