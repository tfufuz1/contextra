//! Lokale Retrieval-Strategie-Definition für `contextra-adapt` (SPEC v4.0 §2 B3).
//!
//! Standardisiertes Enum zur arm- und pfadbasierten Optimierung in Bandit-Algorithmen.

use serde::{Deserialize, Serialize};

/// Retrieval-Strategie für Multi-Armed- und Contextual-Bandits.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum RetrievalStrategy {
    /// Vektorbasierte Ähnlichkeitssuche (Embedding / HNSW / DiskANN).
    Vector,
    /// Volltext- und Keyword-Suche (BM25 / Sparse).
    Text,
    /// Graphbasierte Suche (Hypergraph / Knowledge Graph).
    Graph,
    /// Hybride Kombination aus Vektor- und Textsuche.
    Hybrid,
    /// Globale / Community-Retrieval Strategie.
    Global,
}

#[cfg(feature = "types-compat")]
impl From<RetrievalStrategy> for contextra_types::RetrievalStrategy {
    fn from(strat: RetrievalStrategy) -> Self {
        match strat {
            RetrievalStrategy::Vector => contextra_types::RetrievalStrategy::Vector,
            RetrievalStrategy::Text => contextra_types::RetrievalStrategy::Text,
            RetrievalStrategy::Graph => contextra_types::RetrievalStrategy::Graph,
            RetrievalStrategy::Hybrid => contextra_types::RetrievalStrategy::Hybrid,
            RetrievalStrategy::Global => contextra_types::RetrievalStrategy::Global {
                max_community_nodes: None,
                min_community_size: None,
            },
        }
    }
}

#[cfg(feature = "types-compat")]
impl From<contextra_types::RetrievalStrategy> for RetrievalStrategy {
    fn from(strat: contextra_types::RetrievalStrategy) -> Self {
        match strat {
            contextra_types::RetrievalStrategy::Vector => RetrievalStrategy::Vector,
            contextra_types::RetrievalStrategy::Text => RetrievalStrategy::Text,
            contextra_types::RetrievalStrategy::Graph => RetrievalStrategy::Graph,
            contextra_types::RetrievalStrategy::Hybrid => RetrievalStrategy::Hybrid,
            contextra_types::RetrievalStrategy::Global { .. } => RetrievalStrategy::Global,
            _ => RetrievalStrategy::Vector,
        }
    }
}
