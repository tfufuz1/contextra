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

#[cfg(feature = "types-compat")]
impl From<SimdError> for contextra_types::ContextraError {
    fn from(err: SimdError) -> Self {
        match err {
            SimdError::InvalidInput(msg) => contextra_types::ContextraError::invalid_input(msg),
            SimdError::DimensionMismatch { left, right } => {
                contextra_types::ContextraError::invalid_input(format!(
                    "dimension mismatch: left={}, right={}",
                    left, right
                ))
            }
            SimdError::UnsupportedMetric => {
                contextra_types::ContextraError::invalid_input("unsupported metric")
            }
        }
    }
}
