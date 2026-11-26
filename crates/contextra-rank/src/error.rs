//! Lokaler Fehlertyp für `contextra-rank` (B4-Konformität).

use thiserror::Error;

/// Fehlerzustände von `contextra-rank`.
#[derive(Debug, Error, PartialEq, Clone)]
#[non_exhaustive]
pub enum RankError {
    /// Ungültige Eingabedaten oder Konfiguration.
    #[error("invalid input: {0}")]
    InvalidInput(String),

    /// Fehler bei der Kalibrierung oder Wahrscheinlichkeitsberechnung.
    #[error("calibration error: {0}")]
    Calibration(String),

    /// Fehler während der Signal- oder Rangfusion.
    #[error("fusion failed: {0}")]
    FusionFailed(String),
}

/// Standard-Result-Typ für `contextra-rank`.
pub type Result<T> = core::result::Result<T, RankError>;

#[cfg(feature = "types-compat")]
impl From<RankError> for contextra_types::ContextraError {
    fn from(err: RankError) -> Self {
        match err {
            RankError::InvalidInput(msg) => contextra_types::ContextraError::invalid_input(msg),
            RankError::Calibration(msg) => contextra_types::ContextraError::invalid_input(msg),
            RankError::FusionFailed(msg) => contextra_types::ContextraError::invalid_input(msg),
        }
    }
}
