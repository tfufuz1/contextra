//! # Kompatibilitätsbrücke für Fehler und Re-Exports
//!
//! Stellt Konvertierungen zwischen [`crate::CheckpointError`] und [`contextra_types::ContextraError`] bereit.

use crate::CheckpointError;
use contextra_types::ContextraError;

impl From<CheckpointError> for ContextraError {
    fn from(err: CheckpointError) -> Self {
        match err {
            CheckpointError::InvalidInput(msg) => ContextraError::InvalidInput(msg),
            CheckpointError::Serialization(msg) => ContextraError::Serialization(msg),
            CheckpointError::Transaction(msg) => ContextraError::Transaction(msg),
            CheckpointError::CheckpointNotFound => ContextraError::CheckpointNotFound,
            CheckpointError::CrossDeviceLink {
                source_path,
                target_path,
            } => ContextraError::cross_device_link(source_path, target_path),
            CheckpointError::Io(e) => ContextraError::Io(e),
            CheckpointError::Internal(msg) => ContextraError::Internal(msg),
            CheckpointError::IdWidthMismatch { stored, build } => ContextraError::InvalidInput(
                format!("ID width mismatch: stored {stored} bits, build {build} bits"),
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_conversion_roundtrip_display() {
        let cases = vec![
            (
                CheckpointError::InvalidInput("test input".into()),
                "Invalid input: test input",
            ),
            (
                CheckpointError::Serialization("serde failed".into()),
                "Serialization error: serde failed",
            ),
            (
                CheckpointError::Transaction("tx aborted".into()),
                "Transaction error: tx aborted",
            ),
            (
                CheckpointError::CheckpointNotFound,
                "Checkpoint not found",
            ),
            (
                CheckpointError::CrossDeviceLink {
                    source_path: "/dev/sda1".into(),
                    target_path: "/dev/sdb1".into(),
                },
                "Cross-device link failed: cannot hard link '/dev/sda1' to '/dev/sdb1' across distinct filesystems/mounts",
            ),
            (
                CheckpointError::Internal("kernel panic".into()),
                "Internal error: kernel panic",
            ),
        ];

        for (ckpt_err, expected_display) in cases {
            assert_eq!(ckpt_err.to_string(), expected_display);
            let ctx_err: ContextraError = ckpt_err.into();
            assert_eq!(ctx_err.to_string(), expected_display);
        }
    }
}
