//! Error types for the audit export crate.

use thiserror::Error;

/// Error type returned by operations in `contextra-audit-export`.
#[derive(Debug, Error)]
pub enum AuditExportError {
    /// Failure during JSON serialization or formatting.
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Failure during register data collection.
    #[error("Source collection error: {0}")]
    SourceCollection(String),

    /// Requested tenant identifier is invalid or missing.
    #[error("Invalid tenant ID: {0}")]
    InvalidTenant(String),
}
