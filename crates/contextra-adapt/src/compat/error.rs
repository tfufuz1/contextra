//! Fehlerkonvertierung von `AdaptError` nach `contextra_types::ContextraError`.

use crate::error::AdaptError;

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

#[cfg(test)]
mod tests {
    use super::*;
    use contextra_types::ContextraError;

    #[test]
    fn test_adapt_error_to_contextra_error_mapping() {
        let cases: Vec<(AdaptError, String)> = vec![
            (
                AdaptError::DimensionMismatch { expected: 4, actual: 2 },
                "Invalid input: Embedding dimension mismatch: expected 4, actual 2".to_string(),
            ),
            (
                AdaptError::InvalidConfig("bad param".to_string()),
                "Invalid input: Invalid bandit configuration: bad param".to_string(),
            ),
            (
                AdaptError::NonFinite("nan".to_string()),
                "Internal error: Non-finite encountered: nan".to_string(),
            ),
            (
                AdaptError::PrecisionMatrixDriftDetected {
                    updates_since_reset: 100,
                    denominator: 0.001,
                },
                "Internal error: Sherman-Morrison precision matrix drift detected after 100 updates (denominator = 0.001)".to_string(),
            ),
            (
                AdaptError::ZeroPropensityViolation("arm_0".to_string()),
                "Invalid input: Zero propensity violation: arm_0".to_string(),
            ),
            (
                AdaptError::InvalidInput("wrong value".to_string()),
                "Invalid input: wrong value".to_string(),
            ),
        ];

        for (adapt_err, expected_display) in cases {
            let ctx_err: ContextraError = adapt_err.into();
            assert_eq!(ctx_err.to_string(), expected_display);
        }
    }
}
