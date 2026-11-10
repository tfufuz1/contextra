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
