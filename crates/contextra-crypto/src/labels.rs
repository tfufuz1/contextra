//! Central table of cryptographic labels and constants according to SPEC Appendix C.

/// Tag labels
pub const TENANT_TAG: &[u8] = b"ctx/tenant/v1";
pub const COLL_TAG: &[u8] = b"ctx/coll/v1";
pub const SUBJECT_TAG: &[u8] = b"ctx/subj/v1";
pub const DOC_TAG: &[u8] = b"ctx/doc/v2";
pub const TERM_TAG: &[u8] = b"ctx/term/v1";
pub const NAME_TAG: &[u8] = b"ctx/name/v1";

/// Commitment label
pub const COMMIT: &[u8] = b"ctx/commit/v1";

/// Key derivation labels
pub const ENC: &[u8] = b"ctx/enc/v1";
pub const IDX: &[u8] = b"ctx/idx/v1";
pub const UNIT: &[u8] = b"ctx/unit/v1";
pub const SIGN: &[u8] = b"ctx/sign/v1";
pub const WAL: &[u8] = b"ctx/wal/v1";
pub const LEDGER: &[u8] = b"ctx/ledger/v1";
pub const FILE: &[u8] = b"ctx/file/v1";
pub const FKMASK: &[u8] = b"ctx/fkmask/v1";
pub const DEK: &[u8] = b"ctx/dek/v2";
pub const DEK_SALT: &[u8] = b"ctx/dek-salt/v2";

/// AAD labels
pub const REC_AAD: &[u8] = b"ctx/rec/v2";
pub const SLOT_AAD: &[u8] = b"ctx/slot/v1";
pub const VAULT_MAC: &[u8] = b"ctx/vault-mac/v1";
pub const HDR: &[u8] = b"ctx/hdr/v1";
pub const TBIND_AAD: &[u8] = b"ctx/tbind/v1";
pub const REPORT_PREFIX: &[u8] = b"ctx/report/v1";
