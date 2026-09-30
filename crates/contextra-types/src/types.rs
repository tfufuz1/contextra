//! Core domain, budget, filter, importance, and search query data types.

/// Memory and token resource budget management types.
pub mod budget;
/// Fundamental domain models (`DocId`, `TxId`, `Entity`, `DistanceMetric`, etc.).
pub mod domain;
/// Structured metadata expression filter types.
pub mod filter;
/// Memory importance and recency decay scoring types.
pub mod importance;
/// Unified 4-signal search query and context types.
pub mod saos;

pub use budget::*;
pub use domain::*;
pub use filter::*;
pub use importance::*;
pub use saos::*;

/// Result item produced by cross-encoder reranking.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct RerankResult {
    /// Original index of the document candidate before reranking.
    pub original_index: usize,
    /// Score computed by cross-encoder reranking.
    pub score: f32,
}
