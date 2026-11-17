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
