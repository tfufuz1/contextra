// FILE-CONTEXT
// ZWECK: Fluent HybridQueryBuilder Fassade für Konsolidierung aller Search-Signaturen.
// INVARIANTEN: execute() delegiert an hybrid_search_with_strategy(); keine Logikduplikation.
// NICHT-OFFENSICHTLICH: Post-RRF Filtering bewahrt Snapshot-Konsistenz und RRF-Skalierung.
// STAND: TS:2026-08-30T21:00:00Z (SESSION: 0dcb9f3b)

mod builder;
mod builder_exec;
mod scope;
mod strategy;
mod weights;

#[cfg(test)]
mod tests;

/// Metric counter name for vector search signal latency in seconds.
pub const METRIC_SEARCH_LATENCY_VECTOR: &str = "search.latency.vector";

/// Metric counter name for full-text search signal latency in seconds.
pub const METRIC_SEARCH_LATENCY_TEXT: &str = "search.latency.text";

/// Metric counter name for graph search signal latency in seconds.
pub const METRIC_SEARCH_LATENCY_GRAPH: &str = "search.latency.graph";

pub use builder::{HybridQueryBuilder, DEFAULT_RERANK_POOL_MAX, DEFAULT_RERANK_POOL_MULTIPLIER};
pub use scope::ScopeConstraint;
pub use strategy::SearchStrategy;
pub use weights::SignalWeights;

use super::Collection;
