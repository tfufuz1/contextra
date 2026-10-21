// FILE-CONTEXT
// ZWECK: Contextra cryptographic kernel - Layer 1 crate for AEAD block encryption, HKDF key derivation, and WAL integrity.
// INVARIANTEN: Cryptographic primitives strictly isolated to this crate. All key operations zeroized on drop. Lock-free & I/O-free in src/.
// NICHT-OFFENSICHTLICH: KeyManager uses AES-256-GCM-SIV with OsRng 8-byte random suffix + 4-byte prefix to prevent nonce-reuse key leakage.
// HOTSPOTS: [1-25]
// STAND: TS:2026-08-31T21:13:05Z (SESSION: 8427f167)

#![forbid(unsafe_code)]

//! Cryptography module for Contextra.
//!
//! # Architektur
//! Bietet Verschlüsselung und Integritätsschutz für WAL und SSTables.
//! Nutzt AES-256-GCM-SIV für Datenverschlüsselung und HMAC-SHA256 für Integrität.
//!
//! # Invarianten
//! - Pro Datei wird ein eindeutiger Schlüssel abgeleitet (Nonce-Reuse Mitigation).
//! - Passwörter werden via HKDF in Keys expandiert.
//! - Absolut lock-frei und frei von synchronen/asynchronen I/O-Operationen.

pub mod anti_tamper;
pub mod audit_chain;
pub mod canonical;
pub mod crypto;
pub mod deletion_proof;
pub mod deletion_proof_typestate;
pub mod deletion_witness;
pub mod ed25519_proof;
pub mod error;
pub mod kdf;
pub mod keystore;
pub mod kms;
pub mod kv_cipher;
pub mod kv_shredding;
pub mod revocation_log;
pub mod scope_keyring;
pub mod secure_buf;
pub mod wal_completeness;
pub mod wal_crypto;
pub mod wiring;

pub use audit_chain::{
    compute_record_commitment, AuditChain, AuditChainEntry, AuditChainHeadSignature, DataClass,
    EncryptedCommitmentSalt,
};
pub use canonical::{CanonicalReader, CanonicalTlvEntry, CanonicalWriter};
pub use crypto::KeyManager as CryptoKey;
pub use crypto::KeyManager;
pub use deletion_proof::{
    keys::{KeyClass, TrustEntry, TrustList},
    probe::ProbeError,
    probe::ZeuStoreProbe,
    signer::{ProofSignerProvider, SignerError},
    timestamp_anchor::{
        AnchorQualification, AnchorReceipt, LocalMonotonicAnchor, QualificationLevel,
        RealTsaProofToken, Rfc3161TimeAnchor, TimeAnchor, TsaSignatureVerifier,
    },
    verify::{verify_standalone, VerifiedProof, VerifyError},
    DeletionLayer, DeletionProof, DeletionProofKeyPair, DeletionScope, DurabilityProof,
    ExcludedScope, LayerCleanupProof,
};
#[cfg(any(test, feature = "testing", feature = "test-support", feature = "test-utils"))]
#[allow(deprecated)]
pub use deletion_proof::timestamp_anchor::{AcceptAllTsaVerifier, NoOpTsaSignatureVerifier};
pub use deletion_witness::{
    compute_witness_footprint_hash, LayerWitness, ScanOutcome, WitnessError, WitnessEvidence,
};
pub use ed25519_proof::{DeletionProofError, SignatureVersion};
pub use error::{CryptoError, Result};
pub use kdf::{derive_key_argon2id, KdfHeader, KdfParams};
pub use keystore::MfksKeystore;
pub use kms::{DestroyedKey, KmsProvider, LocalKmsProvider};
pub use kv_cipher::{EncryptedKvLayer, KvCipher, KvSegmentCipher, ModelFingerprint};
pub use kv_shredding::{derive_subkey, KeyRegistry, SubKey, DEFAULT_SHRED_KEY_GROUP_SIZE};
pub use revocation_log::{RevocationEntry, RevocationLog, RevocationTarget};
pub use scope_keyring::{
    DekHandle, DestructionReceipt, KeyId, KeyScope, KeyState, ScopeKeyring,
};
pub use secure_buf::CchSecureBuf;
pub use wal_completeness::verify_wal_chain_completeness;
pub use wal_crypto::IntegrityVerifier;
pub use wiring::{KmsWiring, KmsWiringDeps};
