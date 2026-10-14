use super::openable::FileKind;
use contextra_core::ContextraError;

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum FormatError {
    #[error("Corrupt file format: {reason}")]
    Corrupt { reason: String },

    #[error("Unsupported newer format version for {kind:?}: file version {file_version} > max supported {max_supported}")]
    UnsupportedNewerVersion {
        kind: FileKind,
        file_version: u16,
        max_supported: u16,
    },

    #[error("Downgrade attempt detected for {kind:?}: file version {file_version} < recorded highest {recorded_version}")]
    DowngradeAttempt {
        kind: FileKind,
        file_version: u16,
        recorded_version: u16,
    },

    #[error("IdWidth mismatch: expected {expected}, found {found}")]
    IdWidthMismatch { expected: u8, found: u8 },
}

impl From<FormatError> for ContextraError {
    fn from(err: FormatError) -> Self {
        ContextraError::Storage(err.to_string())
    }
}
