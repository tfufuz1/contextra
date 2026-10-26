//! Types for DiBud (Dynamic Budgeted RRF) fusion.

use contextra_types::{ContextraError, DocId};
use serde::{Deserialize, Serialize};

/// Identifies the channels managed by DiBud fusion budget.
///
/// Order of variants is defined such that `Vector < Text < Graph`, matching
/// the tie-breaking priority requirement `Graph > Text > Vector`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum BudgetedChannel {
    /// Vector (semantic) search channel (index 0).
    Vector = 0,
    /// Text (BM25) search channel (index 1).
    Text = 1,
    /// Graph search channel (index 2).
    Graph = 2,
}

impl BudgetedChannel {
    /// Converts channel variant to array index 0..3.
    #[inline]
    pub const fn index(self) -> usize {
        self as usize
    }

    /// Converts array index 0..3 to channel variant.
    #[inline]
    pub const fn from_index(index: usize) -> Option<Self> {
        match index {
            0 => Some(Self::Vector),
            1 => Some(Self::Text),
            2 => Some(Self::Graph),
            _ => None,
        }
    }
}

/// Dynamic budget configuration for DiBud fusion.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FusionBudget {
    /// Maximum total channel access budget (reads).
    pub max_total_accesses: usize,
    /// Minimum required certified prefix length before stopping.
    pub min_certified_results: usize,
    /// Multiplicative weight $w_{\text{er}}$ for edge-reinforcement bonus term.
    pub edge_reinforcement_weight: f32,
    /// Channel weights $w_i$ in index order `[Vector, Text, Graph]`. Default: `[1.0, 1.0, 1.0]`.
    pub channel_weights: [f32; 3],
    /// Upper bound $\hat{er}$ for unobserved edge-reinforcement scores.
    pub edge_reinforcement_upper_bound: f32,
}

impl Default for FusionBudget {
    fn default() -> Self {
        Self {
            max_total_accesses: 100,
            min_certified_results: 10,
            edge_reinforcement_weight: 0.0,
            channel_weights: [1.0, 1.0, 1.0],
            edge_reinforcement_upper_bound: 0.0,
        }
    }
}

impl FusionBudget {
    /// Validates budget parameter constraints.
    ///
    /// Ensures all weights are finite and non-negative, and minimum accesses/certified lengths
    /// are at least 1.
    pub fn validate(&self) -> Result<(), ContextraError> {
        if self.max_total_accesses == 0 {
            return Err(ContextraError::InvalidInput(
                "FusionBudget max_total_accesses must be at least 1".to_string(),
            ));
        }
        if self.min_certified_results == 0 {
            return Err(ContextraError::InvalidInput(
                "FusionBudget min_certified_results must be at least 1".to_string(),
            ));
        }
        for (idx, &w) in self.channel_weights.iter().enumerate() {
            if !w.is_finite() || w < 0.0 {
                return Err(ContextraError::InvalidInput(format!(
                    "FusionBudget channel_weights[{idx}] must be a finite non-negative float, got {w}"
                )));
            }
        }
        if !self.edge_reinforcement_weight.is_finite() || self.edge_reinforcement_weight < 0.0 {
            return Err(ContextraError::InvalidInput(format!(
                "FusionBudget edge_reinforcement_weight must be a finite non-negative float, got {}",
                self.edge_reinforcement_weight
            )));
        }
        if !self.edge_reinforcement_upper_bound.is_finite()
            || self.edge_reinforcement_upper_bound < 0.0
        {
            return Err(ContextraError::InvalidInput(format!(
                "FusionBudget edge_reinforcement_upper_bound must be a finite non-negative float, got {}",
                self.edge_reinforcement_upper_bound
            )));
        }
        Ok(())
    }
}

/// Final outcome of a DiBud fusion execution.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DiBudOutcome {
    /// Ranked candidate document IDs sorted descending by lower bound $F^-(x)$ (tie-breaking by ascending `DocId`).
    pub ranked: Vec<DocId>,
    /// Number of prefix elements certified to be in their exact final position.
    pub certified_len: usize,
    /// Total number of channel read accesses performed.
    pub accesses: usize,
    /// Indicates whether fusion terminated due to hitting `max_total_accesses`.
    pub budget_exhausted: bool,
}
