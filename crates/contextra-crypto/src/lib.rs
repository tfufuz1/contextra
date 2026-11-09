//! # Kryptographie-Kern für Contextra (Ring 0 / Klasse A)
//!
//! Dieses Modul bildet die kryptographische Basis des gesamten Contextra-Systems.
//! Als Ring-0-Komponente stellt es grundlegende Sicherheitsprimitive bereit,
//! ohne selbst von höheren Schichten abzuhängen.
//!
//! ## Invarianten
//! - `#![forbid(unsafe_code)]`: Es gibt ausnahmslos keinen Unsafe-Code in dieser Crate.
//! - **Zeroize-on-Drop:** Sämtliche geheimen Schlüsselmaterialien werden bei der Freigabe
//!   deterministisch im Speicher mit Nullen überschrieben (`INV-CRYPTO-ZEROIZE-1`).
//! - **Konstantzeit-Vergleiche:** `INV-CRYPTO-CONSTANT-TIME-1` via `subtle::ConstantTimeEq`.

#![forbid(unsafe_code)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]

pub mod aead;
pub mod anti_tamper;
pub mod crypto;
pub mod dek;
pub mod error;
pub mod kdf;
pub mod kv_cipher;
pub mod secret;
pub mod wal_completeness;
pub mod wal_crypto;

pub use aead::*;
pub use anti_tamper::*;
pub use crypto::KeyManager as CryptoKey;
pub use crypto::KeyManager;
pub use dek::{derive_dek, AncestorSalts, Dek, UnitKey};
pub use error::{CryptoError, Error, Result};
pub use kdf::{derive_key_argon2id, hkdf_expand, KdfHeader, KdfParams};
pub use kv_cipher::{EncryptedKvLayer, KvCipher, ModelFingerprint};
pub use secret::SecretKey;
pub use wal_completeness::verify_wal_chain_completeness;
pub use wal_crypto::{
    EncryptedWal, IntegrityVerifier, KmsProvider, LocalKeyManagerKms, WalEntrySnapshot, WalHmac,
};
