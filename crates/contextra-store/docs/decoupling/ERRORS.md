# Fehlerinventar contextra-store (S-37-A)
| StoreError-Variante | ContextraError-Variante | Exakter Display-String |
|---|---|---|
| Storage(String) | ContextraError::Storage(String) | Storage error: {0} |
| Io(std::io::Error) | ContextraError::Io(std::io::Error) | I/O error: {0} |
| WalCorruption { offset: u64, reason: String } | ContextraError::WalCorruption | WAL corruption detected at offset {offset}: {reason} |
| ChecksumMismatch { path: String, block_id: u64 } | ContextraError::ChecksumMismatch | Checksum mismatch: file={path}, block={block_id} |
| WalTruncationDetected | ContextraError::WalTruncationDetected | WAL truncation attack detected |
| CrossDeviceLink | ContextraError::CrossDeviceLink | Cross-device link failed |
| Transaction(String) | ContextraError::Transaction(String) | Transaction error: {0} |
| Conflict(String) | ContextraError::Conflict(String) | Conflict: {0} |
| CommitTimeout { tx_id: u64 } | ContextraError::CommitTimeout | Group commit timed out for transaction {tx_id} |
| Serialization(String) | ContextraError::Serialization(String) | Serialization error: {0} |
| ParseError(String) | ContextraError::ParseError(String) | Parse error: {0} |
| InvalidInput(String) | ContextraError::InvalidInput(String) | Invalid input: {0} |
| NotFound(String) | ContextraError::NotFound(String) | Not found: {0} |
| Internal(String) | ContextraError::Internal(String) | Internal error: {0} |
| MemoryBudgetExceeded | ContextraError::MemoryBudgetExceeded | Memory budget exceeded: {used_mb}MB / {limit_mb}MB |
| LimitExceeded | ContextraError::LimitExceeded | Limit exceeded: {context} (limit: {limit}) |
| DurabilityConfig(String) | ContextraError::DurabilityConfig | Durability configuration error: {0} |
| Sandbox(String) | ContextraError::Sandbox(String) | Sandbox error: {0} |
