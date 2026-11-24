//! Fehlerkonvertierung StoreError -> ContextraError (SPEC §5.3, §6.2)

use crate::error::StoreError;
use contextra_types::ContextraError;

#[cfg(test)]
pub mod tests;

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
            StoreError::Sandbox(s) => ContextraError::Sandbox(s),
        }
    }
}
