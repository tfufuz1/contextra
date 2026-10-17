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

    /// Invalid extension or claim of extended deadline without notice.
    #[error("Invalid extension: {0}")]
    InvalidExtension(String),

    /// Missing or empty required justification for decision or extension.
    #[error("Empty justification: justification string must not be empty or whitespace")]
    EmptyJustification,

    /// Invalid date or date component.
    #[error("Invalid date: {0}")]
    InvalidDate(String),
}
