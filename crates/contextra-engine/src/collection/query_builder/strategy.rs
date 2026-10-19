use contextra_types::{FusionStrategy, GraphTraversalStrategy, SignalFusionStrategies};

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
        }
    }

    /// Converts `SearchStrategy` into core `FusionStrategy`.
    pub fn to_fusion_strategy(&self) -> FusionStrategy {
        match self {
            SearchStrategy::Rrf => FusionStrategy::Rrf,
            SearchStrategy::ScoreNormalized => FusionStrategy::ScoreNormalized,
            SearchStrategy::Hops { .. } | SearchStrategy::PersonalizedPageRank(_) => {
                FusionStrategy::Rrf
            }
        }
    }

    /// Converts `SearchStrategy` into core `SignalFusionStrategies`.
    pub fn to_signal_fusion_strategies(&self) -> SignalFusionStrategies {
        self.to_fusion_strategy().into()
    }
}

impl From<GraphTraversalStrategy> for SearchStrategy {
    fn from(strategy: GraphTraversalStrategy) -> Self {
        match strategy {
            GraphTraversalStrategy::Hops { max_hops } => SearchStrategy::Hops { max_hops },
            GraphTraversalStrategy::PersonalizedPageRank(cfg) => {
                SearchStrategy::PersonalizedPageRank(cfg)
            }
            GraphTraversalStrategy::PathRag { max_hops, .. } => {
                SearchStrategy::Hops { max_hops }
            }
        }
    }
}
