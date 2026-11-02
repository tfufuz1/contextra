#![forbid(unsafe_code)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]

//! Kryptoprimitive für Contextra (Ring 1)
//!
//! AEAD, Argon2id, HKDF, HMAC, Ed25519, Tags.
//!
//! ## Schnittstellenvertrag für Tresor (`FileVault`, Sitzung 12 / `erasure`)
//!
//! Gemäß SPEC 10.1, D-79, D-80, D-85 und F.3 stellt `contextra-crypto` für `erasure`
//! folgende zustandslose Operationen bereit:
//! 1. `KeyHierarchy::fkmask(&self, id: [u8; 16]) -> [u8; 32]`:
//!    Maskierungsschlüssel für F-Slots im Tresor via HKDF-Expand("ctx/fkmask/v1" ‖ file_id).
//! 2. `KeyHierarchy::vault_mac(&self) -> VaultMacKey`:
//!    Authentisierungsschlüssel für Tresor-Slots via HKDF-Expand(K_enc, "ctx/vault-mac/v1").
//! 3. `derive_dek(unit: &UnitKey, a: &AncestorSalts, doc: DocTag) -> Dek`:
//!    DEK-Ableitung nach D-79 mit Salt-Label `ctx/dek-salt/v2` und `doc_tag` (L-3).

pub mod aead;
pub mod chain;
pub mod dek;
pub mod error;
pub mod header;
pub mod hierarchy;
pub mod kdf;
pub mod labels;
pub mod secret;
pub mod sign;
pub mod tag;

pub use aead::*;
pub use chain::*;
pub use dek::*;
pub use error::{CryptoError, Result};
pub use header::*;
pub use hierarchy::*;
pub use kdf::*;
pub use secret::*;
pub use sign::*;
pub use tag::*;
