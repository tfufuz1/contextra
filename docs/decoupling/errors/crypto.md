# Fehlerabbildung `contextra-crypto`

| Lokale Variante (`CryptoError`) | Ziel-Variante (`contextra_types::ContextraError`) | Display-Verhalten |
|---|---|---|
| `WalCorruption { offset, reason }` | `ContextraError::WalCorruption { offset, reason }` | Bitgleich / identische Felder |
| `InvalidInput(msg)` | `ContextraError::InvalidInput(msg)` | Bitgleich |
| `KvFormatVersionMismatch { expected, found }` | `ContextraError::InvalidInput("KV cache format version mismatch: expected {expected}, found {found}")` | Bitgleich |
| `KeyDerivation(msg)` | `ContextraError::Crypto(Display)` | Bitgleich via `other.to_string()` |
| `Encryption(msg)` | `ContextraError::Crypto(Display)` | Bitgleich via `other.to_string()` |
| `Decryption(msg)` | `ContextraError::Crypto(Display)` | Bitgleich via `other.to_string()` |
| `Aead(msg)` | `ContextraError::Crypto(Display)` | Bitgleich via `other.to_string()` |
| `InvalidLength(msg)` | `ContextraError::Crypto(Display)` | Bitgleich via `other.to_string()` |
| `IntegrityViolation` | `ContextraError::Crypto(Display)` | Bitgleich via `other.to_string()` |
| `Crypto(msg)` | `ContextraError::Crypto(Display)` | Bitgleich via `other.to_string()` |
| `WalTruncationDetected` | `ContextraError::Crypto(Display)` | Bitgleich via `other.to_string()` |
| `KeyRevoked(msg)` | `ContextraError::Crypto(Display)` | Bitgleich via `other.to_string()` |
| `Storage(err)` | `ContextraError::Storage(Display)` | Bitgleich via `err` |
