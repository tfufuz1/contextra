//! # Modul: strategy.rs
//!
//! **Zweck:** Suchstrategien (z.B. RRF, ScoreNormalized).
//!
//! **Architektur:** Gehört zur Crate `contextra-engine` (Ring 2).
//! Eingehaltene Invarianten: Fail-Closed bei Fehlern, strikte Mandanten- und Indexisolation.

use contextra_types::{FusionStrategy, SignalFusionStrategies};

/// Strategy for hybrid search signal fusion.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SearchStrategy {
    /// Reciprocal Rank Fusion (default fusion strategy per ADR-003).
    #[default]
    Rrf,
    /// Score-normalized fusion (CombSUM / Z-Score).
    ScoreNormalized,
}

impl SearchStrategy {
    /// Converts `SearchStrategy` into core `FusionStrategy`.
    pub fn to_fusion_strategy(&self) -> FusionStrategy {
        match self {
            SearchStrategy::Rrf => FusionStrategy::Rrf,
            SearchStrategy::ScoreNormalized => FusionStrategy::ScoreNormalized,
        }
    }

    /// Converts `SearchStrategy` into core `SignalFusionStrategies`.
    pub fn to_signal_fusion_strategies(&self) -> SignalFusionStrategies {
        self.to_fusion_strategy().into()
    }
}

impl From<FusionStrategy> for SearchStrategy {
    fn from(strategy: FusionStrategy) -> Self {
        match strategy {
            FusionStrategy::Rrf => SearchStrategy::Rrf,
            FusionStrategy::ScoreNormalized => SearchStrategy::ScoreNormalized,
        }
    }
}

impl From<contextra_types::GraphTraversalStrategy> for SearchStrategy {
    fn from(_: contextra_types::GraphTraversalStrategy) -> Self {
        SearchStrategy::default()
    }
}
