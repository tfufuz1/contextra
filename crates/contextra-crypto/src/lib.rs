// FILE-CONTEXT
// ZWECK: Contextra cryptographic kernel - Layer 1 crate for AEAD block encryption, HKDF key derivation, and WAL integrity.
// INVARIANTEN: Cryptographic primitives strictly isolated to this crate. All key operations zeroized on drop. Lock-free & I/O-free in src/.
// NICHT-OFFENSICHTLICH: KeyManager uses AES-256-GCM-SIV with OsRng 8-byte random suffix + 4-byte prefix to prevent nonce-reuse key leakage.
// HOTSPOTS: [1-25]
// STAND: TS:2026-08-31T21:13:05Z (SESSION: 8427f167)

#![forbid(unsafe_code)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]

//! # Kryptographie-Kern für Contextra (Ring 0)
//!
//! Dieses Modul bildet die kryptographische Basis des gesamten Contextra-Systems.
//! Als Ring-0-Komponente (Kern) stellt es grundlegende Sicherheitsprimitive bereit,
//! ohne selbst von höheren Schichten abzuhängen.
//!
//! ## Ring-Zugehörigkeit und Einbindung
//! - **Ring:** Ring 0 (Kern / Foundation)
//! - **Abhängigkeiten (ausgehend):** `contextra-ports`, `contextra-types`
//! - **Wird eingebunden von:** `contextra`, `contextra-db`, `contextra-engine`,
//!   `contextra-infer-candle`, `contextra-kvcache`, `contextra-mcp`, `contextra-privacy`,
//!   `contextra-store`, `contextra-vector`.
//!
//! ## Hauptaufgaben
//! 1. **AEAD-Blockverschlüsselung:** Authentifizierte Verschlüsselung via AES-256-GCM-SIV
//!    mit deterministischer Nonce-Generierung zur Verhinderung von Nonce-Wiederverwendung.
//! 2. **Schlüsselableitung (KDF):** Schlüsselhierarchie via HKDF (HMAC-SHA256) und
//!    Argon2id für sichere Passphrasen-Verarbeitung.
//! 3. **WAL-Integrität:** Kryptographische Integritätsketten mittels HMAC-SHA256 für
//!    das Write-Ahead-Log (WAL).
//! 4. **DSGVO-Löschbeweise:** Erstellung und Verifikation kryptographisch signierter
//!    Löschzertifikate (Ed25519, BLAKE3) nach Art. 17 DSGVO.
//! 5. **KV-Segment-Verschlüsselung & Shredding:** Feingranulare Mandanten- und Segment-
//!    Isolierung mit kryptographischem Shredding (Crypto-Erase).
//!
//! ## Invarianten
//! - `#![forbid(unsafe_code)]`: Es gibt ausnahmslos keinen Unsafe-Code in dieser Crate.
//! - **Zeroize-on-Drop:** Sämtliche geheimen Schlüsselmaterialien werden bei der Freigabe
//!   deterministisch im Speicher mit Nullen überschrieben.
//! - **I/O- und Lock-Freiheit in `src/crypto`:** Reine Kryptographie-Routinen führen keine
//!   Dateisystem- oder Netzwerk-I/O durch.


pub mod anti_tamper;
pub mod audit_chain;
pub mod crypto;
pub mod deletion_proof;
pub mod ed25519_proof;
pub mod error;
pub mod kdf;
pub mod kv_cipher;
pub mod kv_segment;
pub mod kv_shredding;
pub mod revocation_log;
pub mod wal_completeness;
pub mod wal_crypto;

pub use audit_chain::{
    compute_record_commitment, AuditChain, AuditChainEntry, AuditChainHeadSignature, DataClass,
    EncryptedCommitmentSalt,
};
pub use crypto::KeyManager as CryptoKey;
pub use crypto::KeyManager;
pub use deletion_proof::{
    CustodyLevel, DeletionLayer, DeletionProof, DeletionProofKeyPair, DeletionScope, ExcludedScope,
    LayerCleanupProof, SigningKeyHandle,
};
pub use ed25519_proof::{DeletionProofError, SignatureVersion};
pub use error::{CryptoError, Result};
pub use kdf::{derive_key_argon2id, KdfHeader, KdfParams};
pub use kv_cipher::{EncryptedKvLayer, KvCipher, KvSegmentCipher, ModelFingerprint};
pub use kv_segment::{
    eviction_worker::{emergency_wipe, EvictionWorker},
    segment::KvSegment,
    store::TenantIsolatedKvStore,
    CURRENT_KV_KEY_DERIVATION_VERSION,
};
pub use kv_shredding::{derive_subkey, KeyRegistry, SubKey, DEFAULT_SHRED_KEY_GROUP_SIZE};
pub use revocation_log::{RevocationEntry, RevocationLog, RevocationTarget};
pub use wal_completeness::verify_wal_chain_completeness;
#[cfg(feature = "cloud-egress-guard")]
pub use wal_crypto::CloudEgressGuard;
pub use wal_crypto::{
    EncryptedWal, IntegrityVerifier, KmsProvider, LocalKeyManagerKms, WalEntrySnapshot, WalHmac,
};
