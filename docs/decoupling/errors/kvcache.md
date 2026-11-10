# Fehlerabbildung: contextra-kvcache (KV3.3)

| Lokale Variante (`KvCacheError`) | `ContextraError`-Variante | Display-Formatierung |
|---|---|---|
| `InvalidInput(msg)` | `ContextraError::InvalidInput(msg)` | `Invalid input: {msg}` |
| `CryptoError(msg)` | `ContextraError::Crypto(msg)` | `Crypto error: {msg}` |
| `PinBudgetExceeded(msg)` | `ContextraError::PinBudgetExceeded(msg)` | `Pin budget exceeded: {msg}` |
| `LimitExceeded(msg)` | `ContextraError::LimitExceeded(msg)` | `Limit exceeded: {msg}` |
| `PolicyViolation(msg)` | `ContextraError::PolicyViolation(msg)` | `Policy violation: {msg}` |
| `Serialization(msg)` | `ContextraError::Serialization(msg)` | `Serialization error: {msg}` |
| `QuantizationError(msg)` | `ContextraError::KvQuantization(msg)` | `KV quantization error: {msg}` |
| `Internal(msg)` | `ContextraError::Internal(msg)` | `Internal error: {msg}` |
| `InvalidTenant(msg)` | `ContextraError::InvalidTenant(msg)` | `Ungültiger Mandant: {msg}` |
| `InvalidSegment(msg)` | `ContextraError::Internal(msg)` | `Ungültiges Segment: {msg}` |
| `SegmentNotFound(id)` | `ContextraError::NotFound(id.to_string())` | `Segment nicht gefunden: {id}` |
| `SegmentKeyDestroyed` | `ContextraError::Internal(...)` | `Segmentschlüssel ist zerstört` |
| `BudgetExceeded(msg)` | `ContextraError::MemoryLimitExceeded(msg)` | `Budget oder Kapazität überschritten: {msg}` |
| `ConfigError(msg)` | `ContextraError::Configuration(msg)` | `Konfigurationsfehler: {msg}` |
| `IoError(err)` | `ContextraError::Io(err.to_string())` | `IO error: {err}` |
