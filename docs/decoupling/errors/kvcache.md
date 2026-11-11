# Fehlerabbildung: contextra-kvcache (SPEC §5.3 / WP-KV-1)

Normative Abbildungstabelle zwischen den kern-lokalen `KvCacheError`-Varianten und `contextra_types::ContextraError` gemäß Z4 und G-E. Die Konvertierung ist in `src/compat/error.rs` implementiert und durch `tests/error_display_test.rs` verifiziert.

| Lokale Variante (`KvCacheError`) | `ContextraError`-Variante | Display-Formatierung |
|---|---|---|
| `InvalidTenant(msg)` | `ContextraError::InvalidTenant(msg)` | `Ungültiger Mandant: {msg}` |
| `InvalidInput(msg)` | `ContextraError::InvalidInput(msg)` | `Invalid input: {msg}` |
| `InvalidSegment(msg)` | `ContextraError::Internal(format!("Ungültiges Segment: {msg}"))` | `Ungültiges Segment: {msg}` |
| `SegmentNotFound(id)` | `ContextraError::NotFound(format!("Segment {id} not found"))` | `Segment nicht gefunden: {id}` |
| `CryptoError(msg)` | `ContextraError::Crypto(msg)` | `Crypto error: {msg}` |
| `SegmentKeyDestroyed` | `ContextraError::Internal("Segmentschlüssel ist zerstört".to_string())` | `Segmentschlüssel ist zerstört` |
| `BudgetExceeded(msg)` | `ContextraError::MemoryLimitExceeded(msg)` | `Budget oder Kapazität überschritten: {msg}` |
| `PinBudgetExceeded(msg)` | `ContextraError::PinBudgetExceeded(msg)` | `Pin budget exceeded: {msg}` |
| `LimitExceeded { limit, context }` | `ContextraError::LimitExceeded(format!("{context} (limit: {limit})"))` | `Limit exceeded: {context} (limit: {limit})` |
| `PolicyViolation(msg)` | `ContextraError::PolicyViolation(msg)` | `Policy violation: {msg}` |
| `Serialization(msg)` | `ContextraError::Serialization(msg)` | `Serialization error: {msg}` |
| `Internal(msg)` | `ContextraError::Internal(msg)` | `Internal error: {msg}` |
| `ConfigError(msg)` | `ContextraError::Configuration(msg)` | `Konfigurationsfehler: {msg}` |
| `QuantizationError(msg)` | `ContextraError::KvQuantization(msg)` | `KV quantization error: {msg}` |
| `IoError(err)` | `ContextraError::Io(err.to_string())` | `IO error: {err}` |
