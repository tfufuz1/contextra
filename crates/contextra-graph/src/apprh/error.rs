//! Error types for Averaging-based Personalized PageRank for Hypergraphs (APPRH).

use thiserror::Error;

/// Error type for APPRH operations and parameter validations.
#[derive(Error, Debug, Clone, PartialEq)]
pub enum ApprhError {
    /// Invalid parameter value provided to APPRH.
    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),

    /// Empty seed list provided to APPRH.
    #[error("Seed list cannot be empty")]
    EmptySeeds,

    /// Non-finite value encountered during APPRH calculation.
    #[error("Non-finite value encountered during APPRH calculation")]
    NonFinite,
}

impl From<ApprhError> for contextra_types::ContextraError {
    fn from(err: ApprhError) -> Self {
        contextra_types::ContextraError::InvalidInput(err.to_string())
    }
}
