//! Lokale Fehlerdefinition für `contextra-adapt` (SPEC v4.0 §2 B4).
//!
//! Zentraler, modularer Fehlertyp ohne Abhängigkeit zu `contextra-types`.

use thiserror::Error;

/// Zentraler Fehlertyp für alle adaptiven Regler, Bandits und Drift-Watcher.
#[derive(Debug, Error, PartialEq, Clone)]
#[non_exhaustive]
pub enum AdaptError {
    /// Dimension des Eingabevektors stimmt nicht mit dem erwarteten Zustand überein.
    #[error("Dimension mismatch: expected {expected}, actual {actual}")]
    DimensionMismatch {
        /// Erwartete Dimension.
        expected: usize,
        /// Tatsächliche Dimension.
        actual: usize,
    },

    /// Ungültige Konfiguration eines Reglers oder Bandits.
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    /// Nicht-endlicher numerischer Wert (NaN oder Unendlich) aufgetreten.
    #[error("Non-finite numerical value encountered: {0}")]
    NonFinite(String),

    /// Drift in der Sherman-Morrison Präzisionsmatrix erkannt.
    #[error("Sherman-Morrison precision matrix drift detected after {updates_since_reset} updates (denominator = {denominator})")]
    PrecisionMatrixDriftDetected {
        /// Anzahl der Aktualisierungen seit dem letzten Reset.
        updates_since_reset: u64,
        /// Berechneter Nenner des Sherman-Morrison Updates.
        denominator: f32,
    },

    /// Positivitätsverletzung bei Off-Policy IPS-Evaluation.
    #[error("Zero propensity violation for strategy: {0}")]
    ZeroPropensityViolation(String),

    /// Ungültige Eingabewerte.
    #[error("Invalid input: {0}")]
    InvalidInput(String),
}

#[cfg(feature = "bandit-routing")]
impl From<crate::bandit::BanditError> for AdaptError {
    fn from(err: crate::bandit::BanditError) -> Self {
        match err {
            crate::bandit::BanditError::DimensionMismatch { expected, actual } => {
                AdaptError::DimensionMismatch { expected, actual }
            }
            crate::bandit::BanditError::InvalidConfig(msg) => AdaptError::InvalidConfig(msg),
            crate::bandit::BanditError::NonFiniteVariance(val) => {
                AdaptError::NonFinite(format!("variance: {val}"))
            }
            crate::bandit::BanditError::PrecisionMatrixDriftDetected {
                updates_since_reset,
                denominator,
            } => AdaptError::PrecisionMatrixDriftDetected {
                updates_since_reset,
                denominator,
            },
        }
    }
}

#[cfg(feature = "flow-corrected-thompson")]
impl From<crate::flow_thompson::FcTsError> for AdaptError {
    fn from(err: crate::flow_thompson::FcTsError) -> Self {
        match err {
            crate::flow_thompson::FcTsError::DimensionMismatch { expected, actual } => {
                AdaptError::DimensionMismatch { expected, actual }
            }
            crate::flow_thompson::FcTsError::NonFinite => {
                AdaptError::NonFinite("FcTs value non-finite".to_string())
            }
            crate::flow_thompson::FcTsError::InvalidConfig(msg) => AdaptError::InvalidConfig(msg),
        }
    }
}

#[cfg(feature = "flow-corrected-thompson")]
impl From<crate::flow_thompson::OffPolicyError> for AdaptError {
    fn from(err: crate::flow_thompson::OffPolicyError) -> Self {
        match err {
            crate::flow_thompson::OffPolicyError::ZeroPropensityViolation(strategy) => {
                AdaptError::ZeroPropensityViolation(format!("{strategy:?}"))
            }
        }
    }
}

#[cfg(feature = "rie-greedy-personalization")]
impl From<crate::rie_greedy::RieGreedyError> for AdaptError {
    fn from(err: crate::rie_greedy::RieGreedyError) -> Self {
        match err {
            crate::rie_greedy::RieGreedyError::DimensionMismatch { expected, actual } => {
                AdaptError::DimensionMismatch { expected, actual }
            }
            crate::rie_greedy::RieGreedyError::NonFinite => {
                AdaptError::NonFinite("RIE-Greedy value non-finite".to_string())
            }
            crate::rie_greedy::RieGreedyError::InvalidConfig(msg) => AdaptError::InvalidConfig(msg),
        }
    }
}

#[cfg(feature = "types-compat")]
impl From<AdaptError> for contextra_types::ContextraError {
    fn from(err: AdaptError) -> Self {
        match err {
            AdaptError::DimensionMismatch { expected, actual } => {
                contextra_types::ContextraError::InvalidInput(format!(
                    "Embedding dimension mismatch: expected {expected}, actual {actual}"
                ))
            }
            AdaptError::InvalidConfig(msg) => contextra_types::ContextraError::InvalidInput(format!(
                "Invalid bandit configuration: {msg}"
            )),
            AdaptError::NonFinite(msg) => {
                contextra_types::ContextraError::Internal(format!("Non-finite encountered: {msg}"))
            }
            AdaptError::PrecisionMatrixDriftDetected {
                updates_since_reset,
                denominator,
            } => contextra_types::ContextraError::Internal(format!(
                "Sherman-Morrison precision matrix drift detected after {updates_since_reset} updates (denominator = {denominator})"
            )),
            AdaptError::ZeroPropensityViolation(msg) => {
                contextra_types::ContextraError::InvalidInput(format!(
                    "Zero propensity violation: {msg}"
                ))
            }
            AdaptError::InvalidInput(msg) => contextra_types::ContextraError::InvalidInput(msg),
        }
    }
}

/// Ergebnistyp für Operationen in `contextra-adapt`.
pub type Result<T> = std::result::Result<T, AdaptError>;
