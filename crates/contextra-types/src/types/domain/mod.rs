//! Domänentypen für Contextra.
//!
//! # Architektur-Rolle: Ring 0 (Kanonische Domänenmodelle)
//!
//! Enthält die zentralen Domänen-Modelle wie `DocId`, `TxId`, `TenantId`, `Embedding` und `WorkflowState`.
//! Diese Typen sind die "Lingua Franca" zwischen allen Crates im Workspace.
//!
//! # Invarianten
//! - `DocId` und `TxId` sind Newtypes um primitive Typen mit deterministischer Hash-Generierung.
//! - TxId Base Ranges trennen System- (`>= INTERNAL_BASE`) strikt von Collection-Transaktions-IDs.
//! - Keine Erzeugung von Zeit oder Zufall ohne explizite Parameter oder Injektion.

/// Dokumente, Chunks, Embeddings und Vektor-Distanzmetriken.
mod document;
/// Identifikatoren: DocId, EntityId, TxId, TenantId, CollectionId.
mod ids;
/// Wissensgraphen-Elemente (Entity, Edge), MemoryType, WorkflowState und Verfallskonstanten.
mod misc;

pub use document::{DistanceMetric, Embedding, RerankResult, ScoredDocument};
pub use ids::{CollectionId, DocId, EntityId, TenantId, TxId};
pub use misc::{
    AutoExtractionMode, ConfigFingerprint, Edge, Entity, LinkRelation, MemoryLink, MemoryType,
    PprAlgorithm, PprConfig, TlHfdParams, WorkflowState, EXPIRY_METADATA_KEY, MAX_SEARCH_K,
    TOMBSTONE_BIT,
};

#[cfg(test)]
mod tests;
