# AGENTS.md — contextra-crypto — Kryptographische Primitive

**Status:** Normativ  
**Crate-Spezifikation:** [`docs/crates/contextra-crypto.md`](file:///home/freddy/Projekte/contextra/docs/crates/contextra-crypto.md)  
**Gesamtspezifikation:** [`docs/SPEC.md`](file:///home/freddy/Projekte/contextra/docs/SPEC.md) — Bindung: docs/SPEC.md §4 (Schlüsselhierarchie), §5 (Löschkette), §7 (ErasureLayer), §13 (Invarianten)  
**Workspace-Richtlinie:** [`AGENTS.md`](file:///home/freddy/Projekte/contextra/AGENTS.md) (Wurzel)  
**Ebene:** E1 (Autonomer Domänenbaustein)  
**Kern-Closure:** ids, env (2 Mitglieder)  
**Unsafe-Status:** Verboten (`#![forbid(unsafe_code)]`)  

---

## 1. Zweck & Systemrolle

Kryptographische Kernalgorithmen: AES-256-GCM-SIV, HKDF-SHA-256, Argon2id, Memory Sanitization.

---

## 2. Kern-Richtlinien für Agenten

- **Kryptographische Standards:**
  - AEAD: AES-256-GCM-SIV (RFC 8452, 256-Bit Key, 96-Bit Zufallsnonce, 128-Bit Tag).
  - Schlüsselableitung: HKDF-SHA-256 (RFC 5869) mit strikter Domänentrennung.
  - Passphrase KDF: Argon2id (m=256MB, t=4, p=1, 32-Byte Salt).
  - Zufall: OS CSPRNG via fehlbarem `try_fill_bytes`. Kein deterministischer Fallback!
- **Sicherheits-Eigenschaften für Schlüssel:**
  - Kein `Clone`, kein `Debug`, kein `Serialize`.
  - `ZeroizeOnDrop` zwingend für alle Schlüsselstrukturen.
- **AAD-Bindung:** Jede AEAD-Verschlüsselung bindet zwingend `tenant_id || collection_id || doc_id || version`.
- **Rolle in der Löschkette:** Schritt 3 (Schlüsselslot mit Zufall überschreiben + nullen + fsync) und Schritt 4g (Slot-Bereinigung via `ErasureLayer`).

---

<!-- BEGIN GENERATED: crate_inventory.py -->

## Inventar (generiert)

- **Pfad:** `crates/contextra-crypto` · **Ring:** 0 · **Version:** 0.1.0
- **Beschreibung:** Encryption at Rest and KV-Cache Security utilities for Contextra
- **src:** 22 Dateien, 4479 LOC · **unsafe:** 0 Vorkommen, Policy `forbid`
- **Interne Abhängigkeiten (normal, 4):** `contextra-env`, `contextra-ids`, `contextra-ports`, `contextra-types`
- **Interne Dev/Build-Abhängigkeiten (4):** `contextra-crypto`, `contextra-env`, `contextra-ports`, `contextra-types`
- **Externe Abhängigkeiten (normal, 18):** `aes-gcm-siv`, `ahash`, `argon2`, `bincode`, `blake3`, `hkdf`, `hmac`, `lru`, `parking_lot`, `rand`, `regex`, `serde`, `serde_json`, `sha2`, `subtle`, `thiserror`, `tracing`, `zeroize`
- **Reverse-Abhängigkeiten (2):** `contextra-mcp`, `contextra-store`
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
