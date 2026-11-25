//! Error types for AVV document generation.

use thiserror::Error;

/// Errors that can occur during AVV template rendering.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum AvvGeneratorError {
    /// A required field in the context was empty or invalid.
    #[error("Ungültiger Kontext: {0}")]
    InvalidContext(String),

    /// Error during template rendering or formatting.
    #[error("Render-Fehler: {0}")]
    RenderError(String),
}
