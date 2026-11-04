//! Kern-Domänen-, Budget-, Filter-, Relevanz- und Suchabfragetypen.
//!
//! # Architektur-Rolle: Ring 0
//!
//! Dieses Modul bündelt alle strukturellen Datentypen des Gesamtsystems:
//! - `domain`: Fundamentale Identifikatoren (`DocId`, `TxId`, `TenantId`) und Wissensgraphen-Elemente.
//! - `budget`: Token- und Ressourcen-Budgetierung für Inferenz und Abfrageverarbeitung.
//! - `filter`: Strukturierte Metadaten-Filterausdrücke mit logischen Operatoren.
//! - `importance`: Relevanzbewertung und zeitlicher Verfall (Decay) von Erinnerungen.
//! - `saos`: Einheitliche 4-Signal-Suchabfragestrukturen (Semantik, BM25-Text, Graph, Recency).

/// Memory and token resource budget management types.
pub mod budget;
/// Fundamental domain models (`DocId`, `TxId`, `Entity`, `DistanceMetric`, etc.).
pub mod domain;
/// Structured metadata expression filter types.
/// Memory importance and recency decay scoring types.
pub mod importance;
pub use crate::filter::FilterExpr;
/// Unified 4-signal search query and context types.
pub use budget::{ResourceBudget, ResourceTracker};
pub use domain::{
    AutoExtractionMode, CollectionId, ConfigFingerprint, DistanceMetric, DocId, Edge, Embedding,
    Entity, EntityId, LinkRelation, MemoryLink, MemoryType, PprAlgorithm, PprConfig, RerankResult,
    ScoredDocument, TenantId, TlHfdParams, TxId, WorkflowState, EXPIRY_METADATA_KEY, MAX_SEARCH_K,
    TOMBSTONE_BIT,
};
pub use importance::{DecayFunction, ImportanceScore, MemoryImportance};
