//! Fehlerkonvertierung StoreError -> ContextraError (SPEC §5.3, §6.2)

use crate::error::StoreError;
use contextra_types::ContextraError;

impl From<StoreError> for ContextraError {
    fn from(err: StoreError) -> Self {
        match err {
            StoreError::Storage(s) => ContextraError::Storage(s),
            StoreError::Io(e) => ContextraError::Io(e),
            StoreError::WalCorruption { offset, reason } => {
                ContextraError::wal_corruption(offset, reason)
            }
            StoreError::ChecksumMismatch { path, block_id } => {
                ContextraError::checksum_mismatch(path, block_id)
            }
            StoreError::WalTruncationDetected {
                expected_hmac,
                actual_hmac,
            } => ContextraError::wal_truncation_detected(expected_hmac, actual_hmac),
            StoreError::CrossDeviceLink {
                source_path,
                target_path,
            } => ContextraError::cross_device_link(source_path, target_path),
            StoreError::Transaction(s) => ContextraError::Transaction(s),
            StoreError::Conflict(s) => ContextraError::Conflict(s),
            StoreError::CommitTimeout { tx_id } => ContextraError::CommitTimeout { tx_id },
            StoreError::Serialization(s) => ContextraError::Serialization(s),
            StoreError::ParseError(s) => ContextraError::ParseError(s),
            StoreError::InvalidInput(s) => ContextraError::InvalidInput(s),
            StoreError::NotFound(s) => ContextraError::NotFound(s),
            StoreError::Internal(s) => ContextraError::Internal(s),
            StoreError::MemoryBudgetExceeded { used_mb, limit_mb } => {
                ContextraError::MemoryBudgetExceeded { used_mb, limit_mb }
            }
            StoreError::LimitExceeded { limit, context } => {
                ContextraError::LimitExceeded { limit, context }
            }
            StoreError::DurabilityConfig(s) => ContextraError::durability_config(s),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::StoreError;
    use contextra_types::ContextraError;

    #[test]
    fn test_error_display_parity() {
        let cases: Vec<(StoreError, ContextraError)> = vec![
            (
                StoreError::Storage("disk failure".into()),
                ContextraError::Storage("disk failure".into()),
            ),
            (
                StoreError::WalCorruption {
                    offset: 128,
                    reason: "bad frame".into(),
                },
                ContextraError::wal_corruption(128, "bad frame"),
            ),
            (
                StoreError::ChecksumMismatch {
                    path: "sst1".into(),
                    block_id: 4,
                },
                ContextraError::checksum_mismatch("sst1", 4),
            ),
            (
                StoreError::WalTruncationDetected {
                    expected_hmac: [1u8; 32],
                    actual_hmac: [2u8; 32],
                },
                ContextraError::wal_truncation_detected([1u8; 32], [2u8; 32]),
            ),
            (
                StoreError::CrossDeviceLink {
                    source_path: "/tmp/a".into(),
                    target_path: "/var/b".into(),
                },
                ContextraError::cross_device_link("/tmp/a", "/var/b"),
            ),
            (
                StoreError::Transaction("aborted".into()),
                ContextraError::Transaction("aborted".into()),
            ),
            (
                StoreError::Conflict("write-write".into()),
                ContextraError::Conflict("write-write".into()),
            ),
            (
                StoreError::CommitTimeout { tx_id: 99 },
                ContextraError::CommitTimeout { tx_id: 99 },
            ),
            (
                StoreError::Serialization("json err".into()),
                ContextraError::Serialization("json err".into()),
            ),
            (
                StoreError::ParseError("invalid hex".into()),
                ContextraError::ParseError("invalid hex".into()),
            ),
            (
                StoreError::InvalidInput("empty key".into()),
                ContextraError::InvalidInput("empty key".into()),
            ),
            (
                StoreError::NotFound("k1".into()),
                ContextraError::NotFound("k1".into()),
            ),
            (
                StoreError::Internal("crash".into()),
                ContextraError::Internal("crash".into()),
            ),
            (
                StoreError::MemoryBudgetExceeded {
                    used_mb: 200,
                    limit_mb: 100,
                },
                ContextraError::MemoryBudgetExceeded {
                    used_mb: 200,
                    limit_mb: 100,
                },
            ),
            (
                StoreError::LimitExceeded {
                    limit: 50,
                    context: "scan".into(),
                },
                ContextraError::LimitExceeded {
                    limit: 50,
                    context: "scan".into(),
                },
            ),
            (
                StoreError::DurabilityConfig("invalid ring".into()),
                ContextraError::durability_config("invalid ring"),
            ),
        ];

        for (store_err, ctx_err) in cases {
            let converted: ContextraError = store_err.into();
            assert_eq!(converted.to_string(), ctx_err.to_string());
        }
    }
}
