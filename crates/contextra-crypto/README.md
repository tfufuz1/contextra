# contextra-crypto

**Zweck:** Alle kryptographischen Primitive: AES-256-GCM-SIV (AEAD), HKDF (Schlüsselableitung), Argon2id, KeyTable, Anker, `RecordSealer`, `SegmentSealer`, `KeyDestroyer`, Wurzel-Backends. **Kryptoprimitive nur hier** (R4). Schlüsselhierarchie: SPEC §5.4–5.8.

**Ebene:** E1

## Öffentliche API (`src/lib.rs`, Stand dieser README)

Ist-Stand. Module oder Namen, die laut PLAN.md entfallen, können hier noch stehen.

```rust
pub mod aead
pub mod anti_tamper
pub mod audit_chain
pub mod contract
pub mod crypto
pub mod dek
pub mod error
pub mod kdf
pub mod kv_cipher
pub mod kv_segment
pub mod secret
pub mod wal_completeness
pub mod wal_crypto
pub use aead::*
pub use contract::*
pub use dek::{derive_dek, AncestorSalts, Dek, UnitKey}
pub use kdf::hkdf_expand
pub use secret::SecretKey
pub use audit_chain::
pub use crypto::KeyManager as CryptoKey
pub use crypto::KeyManager
pub use error::{CryptoError, Result}
pub use kdf::
pub use kdf::{derive_key_argon2id, KdfHeader, KdfParams}
pub use kv_cipher::KvSegmentCipher
pub use kv_cipher::{EncryptedKvLayer, KvCipher, ModelFingerprint}
pub use kv_segment::
pub use secret::
pub use wal_completeness::verify_wal_chain_completeness
pub use wal_crypto::CloudEgressGuard
pub use wal_crypto::
```

## Features (`Cargo.toml`)

`default`, `std`, `tracing`, `test-utils`

## Dokumentation

Implementierungsspezifikation: [`PLAN.md`](PLAN.md). Maßgeblich ist `docs/SPEC.md` v4.1.

