// ANCHOR[id=compat-error-root]
//! # Workspace-Kompatibilitätsabbildung für Fehler
//!
//! LINK crates/contextra-mvcc/src/mvcc_error.rs
//! LINK crates/contextra-mvcc/src/compat/mod.rs
//!
//! Dieses Modul implementiert die tabellengesteuerte Konvertierung des lokalen `MvccError`
//! in den globalen `contextra_types::ContextraError`.

use crate::mvcc_error::MvccError;
use contextra_types::ContextraError;

// SECTION Error Conversion
// ANCHOR[id=from-mvcc-error-for-contextra-error]
impl From<MvccError> for ContextraError {
    fn from(err: MvccError) -> Self {
        // NOTE: Bit- und stringgleiche Abbildung gemäß SPEC-v2 §5.3 / SPEC-v3 §5.3
        match err {
            MvccError::Conflict(msg) => ContextraError::Conflict(msg),
            MvccError::LimitExceeded { limit, context } => {
                ContextraError::limit_exceeded(limit, context)
            }
            MvccError::Transaction(msg) => ContextraError::Transaction(msg),
            MvccError::Storage(msg) => ContextraError::Storage(msg),
            MvccError::InvalidInput(msg) => ContextraError::InvalidInput(msg),
            MvccError::Internal(msg) => ContextraError::Internal(msg),
        }
    }
}
// !SECTION Error Conversion

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_mapping_table_driven() {
        let cases = vec![
            (
                MvccError::Conflict("conflict msg".into()),
                ContextraError::Conflict("conflict msg".into()),
            ),
            (
                MvccError::limit_exceeded(5, "snapshots"),
                ContextraError::limit_exceeded(5, "snapshots"),
            ),
            (
                MvccError::Transaction("aborted".into()),
                ContextraError::Transaction("aborted".into()),
            ),
            (
                MvccError::Storage("shard out of bounds".into()),
                ContextraError::Storage("shard out of bounds".into()),
            ),
            (
                MvccError::InvalidInput("sentinel".into()),
                ContextraError::InvalidInput("sentinel".into()),
            ),
            (
                MvccError::Internal("corrupt state".into()),
                ContextraError::Internal("corrupt state".into()),
            ),
        ];

        for (mvcc_err, expected_ctx_err) in cases {
            let actual_ctx_err: ContextraError = mvcc_err.into();
            assert_eq!(actual_ctx_err.to_string(), expected_ctx_err.to_string());
        }
    }
}
