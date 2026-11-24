//! Lokale Fehlertypen für `contextra-simd` (B4-Konformität).

use thiserror::Error;

#[derive(Debug, Error, PartialEq, Clone)]
#[non_exhaustive]
pub enum SimdError {
    #[error("invalid input: {0}")]
    InvalidInput(&'static str),

    #[error("dimension mismatch: left={left}, right={right}")]
    DimensionMismatch { left: usize, right: usize },

    #[error("unsupported metric")]
    UnsupportedMetric,
}

pub type Result<T> = core::result::Result<T, SimdError>;

/// Typ-Alias `Error` für B4-Konformität (genau ein primärer Fehlertyp `Error`).
pub type Error = SimdError;
