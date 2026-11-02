//! Feste Labels aus SPEC Anhang C (TY-14).
//!
//! Konstanten aller kanonischen Labels für kryptografische Ableitung, Domain-Separation und Tags.

/// Label für TenantTag-Erzeugung.
pub const LABEL_TENANT_TAG: &[u8] = b"ctx/tenant/v1";

/// Label für CollTag-Erzeugung.
pub const LABEL_COLL_TAG: &[u8] = b"ctx/coll/v1";

/// Label für SubjectTag-Erzeugung.
pub const LABEL_SUBJECT_TAG: &[u8] = b"ctx/subj/v1";

/// Label für DocTag-Erzeugung.
pub const LABEL_DOC_TAG: &[u8] = b"ctx/doc/v2";

/// Label für TermTag-Erzeugung.
pub const LABEL_TERM_TAG: &[u8] = b"ctx/term/v1";

/// Label für NameTag-Erzeugung.
pub const LABEL_NAME_TAG: &[u8] = b"ctx/name/v1";

/// Label für Epochen-Commitment.
pub const LABEL_COMMIT: &[u8] = b"ctx/commit/v1";

/// Label für Verschlüsselungsschlüssel (K_enc).
pub const LABEL_ENC: &[u8] = b"ctx/enc/v1";

/// Label für Index-Tag-Schlüssel (K_idx).
pub const LABEL_IDX: &[u8] = b"ctx/idx/v1";

/// Label für Einheiten-Schlüssel (K_unit).
pub const LABEL_UNIT: &[u8] = b"ctx/unit/v1";

/// Label für Signatur-Schlüsselableitung (K_sign).
pub const LABEL_SIGN: &[u8] = b"ctx/sign/v1";

/// Label für WAL-HMAC-Kette (K_wal).
pub const LABEL_WAL: &[u8] = b"ctx/wal/v1";

/// Label für Ledger-Authentisierung (K_ledger).
pub const LABEL_LEDGER: &[u8] = b"ctx/ledger/v1";

/// Label für Steuerdatei-Schlüssel (K_file).
pub const LABEL_FILE: &[u8] = b"ctx/file/v1";

/// Label für Maskierung von Dateischlüsseln (F-Slots).
pub const LABEL_FKMASK: &[u8] = b"ctx/fkmask/v1";

/// Label für Dokument-DEK-Ableitung.
pub const LABEL_DEK: &[u8] = b"ctx/dek/v2";

/// Label für Dokument-DEK-Salt-Kombination.
pub const LABEL_DEK_SALT: &[u8] = b"ctx/dek-salt/v2";

/// Label für Record-AAD-Bindung.
pub const LABEL_RECORD_AAD: &[u8] = b"ctx/rec/v2";

/// Label für Keyslot-AAD-Bindung.
pub const LABEL_KEYSLOT_AAD: &[u8] = b"ctx/slot/v1";

/// Label für Vault-Slot-MAC.
pub const LABEL_VAULT_MAC: &[u8] = b"ctx/vault-mac/v1";

/// Label für Header-MAC.
pub const LABEL_HEADER_MAC: &[u8] = b"ctx/hdr/v1";

/// Label für Textbindung-AAD.
pub const LABEL_TEXT_BINDING_AAD: &[u8] = b"ctx/tbind/v1";
