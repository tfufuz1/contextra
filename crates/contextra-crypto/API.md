# Öffentliche API von contextra-crypto

## Module
- `aead`: Authentifizierte Verschlüsselung (AES-256-GCM-SIV, RFC 8452).
- `anti_tamper`: Konstantzeit-Vergleiche (`constant_time_compare`, `verify_tag_constant_time`), `VolatileEncryptionKey`, `Blake3Chain`.
- `audit_chain`: Blake3-basierte Hash-Kette für Audit-Einträge (`AuditChain`, `AuditChainEntry`, `DataClass`).
- `crypto`: Zentraler `KeyManager` (`CryptoKey`) für HKDF-Schlüsselableitung und AEAD-Verschlüsselung mit deterministischer Nonce-Generierung.
- `dek`: Data Encryption Key (DEK) Wrapping (`wrap_dek`, `unwrap_dek`, `Dek`, `UnitKey`, `derive_dek`).
- `error`: Fehlertypen (`CryptoError`, `Result<T>`).
- `kdf`: Schlüsselableitung via Argon2id (`derive_key_argon2id`, `KdfHeader`, `KdfParams`) und HKDF-Expand (`hkdf_expand`).
- `kv_cipher`: KV-Segment-Verschlüsselung (`KvSegmentCipher`, `EncryptedKvLayer`, `KvCipher`, `ModelFingerprint`).
- `kv_segment`: KV-Segment-Datenstrukturen und LRU-Store (`KvSegment`, `TenantIsolatedKvStore`, `KvDeleteMode`, `KvSegmentManager`, `EvictionWorker`).
- `secret`: Sichere Container mit Zeroize-on-Drop (`SecretKey`, `Secret`).
- `wal_completeness`: Prüfung der Vollständigkeit von WAL-HMAC-Ketten (`verify_wal_chain_completeness`).
- `wal_crypto`: WAL-HMAC-Integritätsüberprüfung und Chunk-Verschlüsselung (`EncryptedWal`, `IntegrityVerifier`, `WalHmac`, `WalEntrySnapshot`, `KmsProvider`).
