# FEHLERABBILDUNG — contextra-store (ST6.2)

Tabelle gemäß SPEC-v2 §5.3:
Lokale Variante `StoreError` -> `ContextraError`-Variante -> Display-Text bitgleich.

| Lokale Variante (StoreError) | ContextraError-Variante | Display-Format |
|---|---|---|
| `Storage(String)` | `Storage(s)` | `Storage error: {0}` |
| `Io(std::io::Error)` | `Io(e)` | `I/O error: {0}` |
| `WalCorruption { offset: u64, reason: String }` | `WalCorruption { offset, reason }` | `WAL corruption detected at offset {offset}: {reason}` |
| `ChecksumMismatch { path: String, block_id: u64 }` | `ChecksumMismatch { path, block_id }` | `Checksum mismatch: file={path}, block={block_id}` |
| `WalTruncationDetected { expected_hmac: [u8; 32], actual_hmac: [u8; 32] }` | `WalTruncationDetected { expected_hmac, actual_hmac }` | `WAL truncation attack detected: expected tail HMAC {expected_hmac:?}, actual tail HMAC {actual_hmac:?}` |
| `CrossDeviceLink { source_path: String, target_path: String }` | `CrossDeviceLink { source_path, target_path }` | `Cross-device link failed: cannot hard link '{source_path}' to '{target_path}' across distinct filesystems/mounts` |
| `Transaction(String)` | `Transaction(s)` | `Transaction error: {0}` |
| `Conflict(String)` | `Conflict(s)` | `Conflict: {0}` |
| `CommitTimeout { tx_id: u64 }` | `CommitTimeout { tx_id }` | `Group commit timed out for transaction {tx_id}` |
| `Serialization(String)` | `Serialization(s)` | `Serialization error: {0}` |
| `ParseError(String)` | `ParseError(s)` | `Parse error: {0}` |
| `InvalidInput(String)` | `InvalidInput(s)` | `Invalid input: {0}` |
| `NotFound(String)` | `NotFound(s)` | `Not found: {0}` |
| `Internal(String)` | `Internal(s)` | `Internal error: {0}` |
| `MemoryBudgetExceeded { used_mb: u64, limit_mb: u64 }` | `MemoryBudgetExceeded { used_mb, limit_mb }` | `Memory budget exceeded: {used_mb}MB / {limit_mb}MB` |
| `LimitExceeded { limit: usize, context: String }` | `LimitExceeded { limit, context }` | `Limit exceeded: {context} (limit: {limit})` |
| `DurabilityConfig(String)` | `DurabilityConfig(s)` | `Durability configuration error: {0}` |
