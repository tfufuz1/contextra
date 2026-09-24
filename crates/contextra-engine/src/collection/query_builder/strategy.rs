use contextra_types::{FusionStrategy, GraphTraversalStrategy};

/// Strategy for hybrid search and graph signal traversal.
#[derive(Debug, Clone, PartialEq)]
pub enum SearchStrategy {
    /// Reciprocal Rank Fusion (default fusion strategy per ADR-003).
    Rrf,
    /// Score-normalized fusion (CombSUM / Z-Score).
    ScoreNormalized,
    /// Multi-hop BFS graph traversal strategy.
    Hops {
        /// Maximum traversal hop depth.
        max_hops: usize,
    },
    /// Personalized PageRank power iteration graph traversal strategy.
    PersonalizedPageRank(contextra_types::PprConfig),
    /// PathRAG bidirectional Dijkstra graph traversal strategy.
    PathRag {
        /// Maximum traversal hop depth.
        max_hops: usize,
        /// Sufficiency threshold for filtering low-confidence paths.
        sufficiency_threshold: f64,
    },
}

impl SearchStrategy {
    /// Converts `SearchStrategy` into core `GraphTraversalStrategy`.
    pub fn to_graph_strategy(&self) -> GraphTraversalStrategy {
        match self {
            SearchStrategy::Rrf | SearchStrategy::ScoreNormalized => {
                GraphTraversalStrategy::Hops { max_hops: 3 }
            }
            SearchStrategy::Hops { max_hops } => GraphTraversalStrategy::Hops {
                max_hops: *max_hops,
            },
            SearchStrategy::PersonalizedPageRank(cfg) => {
                GraphTraversalStrategy::PersonalizedPageRank(cfg.clone())
            }
            SearchStrategy::PathRag {
                max_hops,
                sufficiency_threshold,
            } => GraphTraversalStrategy::PathRag {
                max_hops: *max_hops,
                sufficiency_threshold: *sufficiency_threshold,
            },
        }
    }

    /// Converts `SearchStrategy` into core `FusionStrategy`.
    pub fn to_fusion_strategy(&self) -> FusionStrategy {
        match self {
            SearchStrategy::Rrf => FusionStrategy::Rrf,
            SearchStrategy::ScoreNormalized => FusionStrategy::ScoreNormalized,
            _ => FusionStrategy::Rrf,
        }
    }
}

impl From<GraphTraversalStrategy> for SearchStrategy {
    fn from(strategy: GraphTraversalStrategy) -> Self {
        match strategy {
            GraphTraversalStrategy::Hops { max_hops } => SearchStrategy::Hops { max_hops },
            GraphTraversalStrategy::PersonalizedPageRank(cfg) => {
                SearchStrategy::PersonalizedPageRank(cfg)
            }
            GraphTraversalStrategy::PathRag {
                max_hops,
                sufficiency_threshold,
            } => SearchStrategy::PathRag {
                max_hops,
                sufficiency_threshold,
            },
        }
    }
}
