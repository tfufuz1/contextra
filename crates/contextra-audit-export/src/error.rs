//! Error types for the audit export crate.

use thiserror::Error;

/// Error type returned by operations in `contextra-audit-export`.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum AuditExportError {
    /// Failure during JSON serialization or formatting.
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Missing mandatory field required by GDPR Article 30.
    #[error("Missing mandatory field: {field}")]
    MissingField {
        /// Name of the missing or empty field.
        field: String,
    },

    /// Failure during register data collection.
    #[error("Source collection error: {0}")]
    SourceCollection(String),

    /// Requested tenant identifier is invalid or missing: {0}.
    #[error("Requested tenant identifier is invalid or missing: {0}")]
    InvalidTenant(String),

    /// Failure during CSV formatting or output generation.
    #[error("CSV export formatting error: {0}")]
    CsvExport(String),
}
