//! Crate-eigener Fehlertyp von `contextra-simd` (SPEC v4.0 §2 B4).
//!
//! Bewusst ohne Abhängigkeit auf andere `contextra-*`-Crates. Die Konvertierung in
//! den Storage-Fehler liegt im Adapter (z. B. `contextra-vector`).

/// Fehler der SIMD-Distanzkerne.
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    /// Ungültige Eingabe (NaN/Unendlich, unbekannter Test-Override, …).
    #[error("invalid input: {0}")]
    InvalidInput(String),

    /// Vektoren unterschiedlicher Dimension.
    #[error("embedding dimension mismatch: expected {expected}, got {got}")]
    EmbeddingDimensionMismatch {
        /// Erwartete Dimension.
        expected: usize,
        /// Tatsächliche Dimension.
        got: usize,
    },
}

impl Error {
    /// Erzeugt einen [`Error::InvalidInput`].
    pub fn invalid_input(msg: impl Into<String>) -> Self {
        Self::InvalidInput(msg.into())
    }
}

/// Ergebnistyp des Crates.
pub type Result<T> = core::result::Result<T, Error>;
