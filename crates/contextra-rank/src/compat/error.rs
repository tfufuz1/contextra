//! Fehlerkonvertierung für `contextra-rank` nach `contextra_types::ContextraError`.

use crate::error::RankError;

impl From<RankError> for contextra_types::ContextraError {
    fn from(err: RankError) -> Self {
        match err {
            RankError::InvalidInput(msg) => contextra_types::ContextraError::invalid_input(msg),
            RankError::Calibration(msg) => contextra_types::ContextraError::invalid_input(msg),
            RankError::FusionFailed(msg) => contextra_types::ContextraError::invalid_input(msg),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rank_error_to_contextra_error_mapping() {
        let cases = vec![
            (
                RankError::InvalidInput("invalid doc id".to_string()),
                "Invalid input: invalid doc id",
            ),
            (
                RankError::Calibration("pava error".to_string()),
                "Invalid input: pava error",
            ),
            (
                RankError::FusionFailed("no active sets".to_string()),
                "Invalid input: no active sets",
            ),
        ];

        for (rank_err, expected_display) in cases {
            let ctx_err: contextra_types::ContextraError = rank_err.into();
            assert_eq!(ctx_err.to_string(), expected_display);
            assert!(
                matches!(ctx_err, contextra_types::ContextraError::InvalidInput(_)),
                "Erwartete InvalidInput, erhalten: {ctx_err:?}"
            );
        }
    }
}
