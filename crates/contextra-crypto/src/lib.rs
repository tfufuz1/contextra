#![forbid(unsafe_code)]
#![cfg_attr(
    not(test),
    deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)
)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]

//! Kryptoprimitive für Contextra: AEAD, Argon2id, HKDF, HMAC, Ed25519, Tags.
//!
//! Ring: Ring 1 (Krypto-Insel).
//! Erlaubte Abhängigkeiten: `contextra-types`, `contextra-ports`.
//! Referenz: SPEC Abschnitt 5.1, 5.3, 10 (Kryptographie), Anhang F (Formate).
//!
//! ## Schnittstellenvertrag für Tresor (`FileVault` / `erasure`)
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

pub use aead::{open_record, seal_record, unwrap_master, wrap_master, Block, BlockSealer, SlotAad};
pub use chain::{
    chain_next, ledger_mac, verify_chain_complete, verify_wal_chain_completeness, wal_mac,
};
pub use dek::{derive_dek, AncestorSalts, Dek, DocTag, Salt};
pub use error::{CryptoError, Result};
pub use header::{
    Header, HeaderSlot, HEADER_ALG_IDS, HEADER_FORMAT_VERSION, HEADER_KIND, HEADER_MAGIC,
    HEADER_SIZE,
};
pub use hierarchy::{
    CommitKey, EncKey, FileKey, FileKind, HdrKey, IdxKey, KeyHierarchy, LedgerKey, MasterKey,
    SignSeed, UnitKey, VaultMacKey, WalKey,
};
pub use kdf::{
    derive_kek, hkdf_expand, KdfParams, MAX_M_KIB, MAX_P, MAX_T, MIN_M_KIB, MIN_P,
    MIN_PASSPHRASE_BYTES, MIN_T,
};
pub use labels::{
    COLL_TAG, COMMIT, DEK, DEK_SALT, DOC_TAG, ENC, FILE, FKMASK, HDR, IDX, LEDGER, NAME_TAG,
    REC_AAD, REPORT_PREFIX, SIGN, SLOT_AAD, SUBJECT_TAG, TBIND_AAD, TENANT_TAG, TERM_TAG, UNIT,
    VAULT_MAC, WAL,
};
pub use secret::{Secret, SecretKey};
pub use sign::{
    sign_report, verify_report, DerivedReportSigner, ReportSigner, SIGNER_ID_DERIVED,
    SIGNER_ID_OPERATOR,
};
pub use tag::{tag, TagPreimage};
