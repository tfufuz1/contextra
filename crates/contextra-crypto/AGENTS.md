# AGENTS.md — `contextra-crypto`

> Werkzeug/Crate-Kontext für Agenten. Der obere Teil ist generiert; manuelle Notizen unterhalb ergänzen.

<!-- BEGIN GENERATED: crate_inventory.py -->

## Inventar (generiert)

- **Pfad:** `crates/contextra-crypto` · **Ring:** 0 · **Version:** 0.1.0
- **Beschreibung:** Encryption at Rest and KV-Cache Security utilities for Contextra
- **src:** 22 Dateien, 4479 LOC · **unsafe:** 0 Vorkommen, Policy `forbid`
- **Interne Abhängigkeiten (normal, 4):** `contextra-env`, `contextra-ids`, `contextra-ports`, `contextra-types`
- **Interne Dev/Build-Abhängigkeiten (4):** `contextra-crypto`, `contextra-env`, `contextra-ports`, `contextra-types`
- **Externe Abhängigkeiten (normal, 18):** `aes-gcm-siv`, `ahash`, `argon2`, `bincode`, `blake3`, `hkdf`, `hmac`, `lru`, `parking_lot`, `rand`, `regex`, `serde`, `serde_json`, `sha2`, `subtle`, `thiserror`, `tracing`, `zeroize`
- **Reverse-Abhängigkeiten (4):** `contextra-infer-candle`, `contextra-kvcache`, `contextra-mcp`, `contextra-store`
- **Kopplung an Hub-Crates:** ausgehend `contextra-ports`, `contextra-types`; eingehend –
- **Features (7):** `cloud-egress-guard`, `default`, `kv-encryption`, `ports-compat`, `test-utils`, `types-compat`, `workspace`

### Öffentliche API (grob, aus src/)
- struct (34): `AncestorSalts`, `AuditChain`, `AuditChainEntry`, `AuditChainHeadSignature`, `AuditHeadAnchor`, `Blake3Chain`, `CloudEgressGuard`, `Dek`, `DerivedKey`, `DeterministicEntropy`, `EncryptedCommitmentSalt`, `EncryptedKvLayer`, `EncryptedSegmentPayload`, `EncryptedWal`, `EvictionWorker` … (+19)
- enum (3): `CryptoError`, `DataClass`, `KvDeleteMode`
- trait (4): `CryptoEntropy`, `KmsContract`, `KmsProvider`, `KvCipher`
- fn (103): `append`, `as_bytes`, `cipher_for`, `cipher_for_bytes`, `cipher_for_scoped`, `cipher_for_scoped_bytes`, `clear_all`, `compute_hash`, `compute_record_commitment`, `constant_time_compare`, `current_hash`, `decrypt`, `decrypt_auto_nonce`, `decrypt_chunk`, `decrypt_data` … (+88)
- type (4): `DocTag`, `Result`, `Salt`, `SpillHandler`
- const (18): `CURRENT_KV_FORMAT_VERSION`, `CURRENT_KV_KEY_DERIVATION_VERSION`, `DEFAULT_M_COST_KIB`, `DEFAULT_P_COST`, `DEFAULT_SEGMENT_CAPACITY_PER_TENANT`, `DEFAULT_SHARD_COUNT`, `DEFAULT_T_COST`, `DEK_LABEL`, `DEK_SALT_LABEL`, `KDF_HEADER_MAGIC`, `KDF_HEADER_VERSION_1`, `KDF_ID_ARGON2ID`, `MIN_M_COST_KIB`, `MIN_P_COST`, `MIN_SALT_LEN` … (+3)
- mod (20): `aead`, `anti_tamper`, `audit_chain`, `compat`, `contract`, `crypto`, `dek`, `delete_mode`, `error`, `eviction_worker`, `kdf`, `kv_cipher`, `kv_segment`, `manager`, `model_fingerprint` … (+5)
- pub use: `aead::*`, `audit_chain::{ compute_record_commitment, AuditChain, AuditChainEntry, AuditChainHeadSignature, DataClass, EncryptedCommitmentSalt, }`, `contract::*`, `crypto::KeyManager`, `crypto::KeyManager as CryptoKey`, `dek::{derive_dek, AncestorSalts, Dek, UnitKey}`, `delete_mode::KvDeleteMode`, `error::{CryptoError, Result}`, `eviction_worker::{emergency_wipe, EvictionWorker}`, `kdf::hkdf_expand` … (+11)

Regeneriert mit `python3 tools/crate_inventory.py agents`. Manuelle Notizen stehen außerhalb dieses Blocks.

<!-- END GENERATED: crate_inventory.py -->
