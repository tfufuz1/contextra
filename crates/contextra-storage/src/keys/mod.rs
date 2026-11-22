//! # Schlüssel- und Nonce-Verwaltung
//!
//! Zentrales Protokoll für Nonces und Schlüsselhierarchie (SPEC §15.2, §9.3).
//! Dieses Modul bildet die normative Grundlage für KEK, DEK, Wurzelschlüssel,
//! Ableitungen und 12-Byte-Nonces.

pub mod epoch;
pub mod hierarchy;
pub mod nonce;
pub mod protocol;
pub mod secret;

pub use epoch::{EpochTracker, KeyEpoch};
pub use hierarchy::{
    derive_blob_key, derive_catalog_key, derive_format_mac_key, derive_keys_key, derive_redo_key,
    derive_run_key, derive_segment_key, derive_token_key, CONTEXT_BLOB, CONTEXT_CATALOG,
    CONTEXT_FORMAT, CONTEXT_KEYS, CONTEXT_REDO, CONTEXT_RUN, CONTEXT_SEGMENT, CONTEXT_TOKEN,
};
pub use nonce::{
    blob_body_nonce, blob_record_nonce, catalog_record_nonce, keys_nonce, keys_nonce_from_entropy,
    redo_frame_nonce, redo_op_body_nonce, run_block_nonce,
};
pub use protocol::{KeysError, KeysFile, KeysPlain, SpaceDekRecord, KEYS_MAGIC, KEYS_VERSION_1};
pub use secret::{Secret32, SecretCell};
