//! Tests für die Fehlerkonvertierung StoreError -> ContextraError (WP-H.4)

#[cfg(test)]
mod tests {
    use crate::error::StoreError;
    use contextra_types::ContextraError;

    #[test]
    fn test_store_error_conversion_table() {
        let cases: Vec<(StoreError, String)> = vec![
            (
                StoreError::Storage("corrupt disk".into()),
                "Storage error: corrupt disk".into(),
            ),
            (
                StoreError::WalCorruption {
                    offset: 1024,
                    reason: "bad record".into(),
                },
                "WAL corruption detected at offset 1024: bad record".into(),
            ),
            (
                StoreError::ChecksumMismatch {
                    path: "/data/001.sst".into(),
                    block_id: 42,
                },
                "Checksum mismatch: file=/data/001.sst, block=42".into(),
            ),
            (
                StoreError::WalTruncationDetected {
                    expected_hmac: [1u8; 32],
                    actual_hmac: [2u8; 32],
                },
                format!(
                    "WAL truncation attack detected: expected tail HMAC {:?}, actual tail HMAC {:?}",
                    [1u8; 32],
                    [2u8; 32]
                ),
            ),
            (
                StoreError::CrossDeviceLink {
                    source_path: "/tmp/a".into(),
                    target_path: "/data/b".into(),
                },
                "Cross-device link failed: cannot hard link '/tmp/a' to '/data/b' across distinct filesystems/mounts".into(),
            ),
            (
                StoreError::Transaction("abort".into()),
                "Transaction error: abort".into(),
            ),
            (
                StoreError::Conflict("write conflict".into()),
                "Conflict: write conflict".into(),
            ),
            (
                StoreError::CommitTimeout { tx_id: 99 },
                "Group commit timed out for transaction 99".into(),
            ),
            (
                StoreError::Serialization("bincode err".into()),
                "Serialization error: bincode err".into(),
            ),
            (
                StoreError::ParseError("invalid header".into()),
                "Parse error: invalid header".into(),
            ),
            (
                StoreError::InvalidInput("bad key".into()),
                "Invalid input: bad key".into(),
            ),
            (
                StoreError::NotFound("doc 1".into()),
                "Not found: doc 1".into(),
            ),
            (
                StoreError::Internal("panic bug".into()),
                "Internal error: panic bug".into(),
            ),
            (
                StoreError::MemoryBudgetExceeded {
                    used_mb: 500,
                    limit_mb: 256,
                },
                "Memory budget exceeded: 500MB / 256MB".into(),
            ),
            (
                StoreError::LimitExceeded {
                    limit: 10,
                    context: "scanned rows".into(),
                },
                "Limit exceeded: scanned rows (limit: 10)".into(),
            ),
            (
                StoreError::DurabilityConfig("sync mode invalid".into()),
                "Durability configuration error: sync mode invalid".into(),
            ),
            (
                StoreError::Sandbox("permission denied".into()),
                "Sandbox error: permission denied".into(),
            ),
        ];

        for (store_err, expected_display) in cases {
            assert_eq!(store_err.to_string(), expected_display);
            let ctx_err: ContextraError = store_err.into();
            assert_eq!(ctx_err.to_string(), expected_display);
        }
    }
}
