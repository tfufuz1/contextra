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
pub use crate::filter;
/// Memory importance and recency decay scoring types.
pub mod importance;
/// Unified 4-signal search query and context types.
pub mod saos;

pub use budget::{BudgetStrategy, Reservation, ResourceBudget, ResourceTracker, TokenBudget};
pub use domain::{
    AutoExtractionMode, CollectionId, ConfigFingerprint, DistanceMetric, DocId, Edge, Embedding,
    Entity, EntityId, LinkRelation, MemoryLink, MemoryType, PprAlgorithm, PprConfig, RerankResult,
    ScoredDocument, TenantId, TlHfdParams, TxId, WorkflowState, EXPIRY_METADATA_KEY, MAX_SEARCH_K,
    TOMBSTONE_BIT,
};
pub use filter::FilterExpr;
pub use importance::{DecayFunction, ImportanceScore, MemoryImportance};
pub use saos::{
    ContextChunk, ContextWindow, FusionStrategy, FusionWeights, GraphTraversalStrategy,
    HybridQuery, HybridQueryBuilder, OnSignalFailure, QueryOptions, QueryOptionsBuilder,
    QueryRewriteOutput, ScoredEntry, Signal, SignalFusionStrategies,
};
