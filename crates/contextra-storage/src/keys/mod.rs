//! # Schlüssel- und Nonce-Verwaltung
//!
//! Zentrales Protokoll für Nonces und Schlüsselhierarchie (SPEC §15.2, §9.3).
//! Dieses Modul bildet die normative Grundlage für KEK, DEK, Wurzelschlüssel,
//! Clusterschlüssel, Ableitungen und Nonces. Versiegelungen nutzen 24-Byte-Zufallsnonces
//! (SPEC §10.1); die 12-Byte-Konstruktoren in `nonce` sind Bestand (siehe dort).

pub mod cluster_keys;
pub mod epoch;
pub mod hierarchy;
pub mod key_cache;
pub mod nonce;
pub mod protocol;
pub mod rotation;
pub mod secret;

pub use cluster_keys::{ClusterId, ClusterKeyError, ClusterKeys, KeyState, PortKeys};
pub use epoch::{EpochTracker, KeyEpoch};
pub use hierarchy::{
    derive_blob_key, derive_catalog_key, derive_cluster_seal_key, derive_format_mac_key,
    derive_keys_key, derive_redo_key, derive_run_key, derive_segment_key, derive_token_key,
    CONTEXT_BLOB, CONTEXT_CATALOG, CONTEXT_FORMAT, CONTEXT_KEYS, CONTEXT_REDO, CONTEXT_RUN,
    CONTEXT_SEAL, CONTEXT_SEGMENT, CONTEXT_TOKEN,
};
pub use key_cache::KeyCache;
pub use nonce::{
    blob_body_nonce, blob_record_nonce, catalog_record_nonce, keys_nonce, keys_nonce_from_entropy,
    redo_frame_nonce, redo_op_body_nonce, run_block_nonce, seal_nonce_from_entropy, SEAL_NONCE_LEN,
};
pub use protocol::{KeysError, KeysFile, KeysPlain, SpaceDekRecord, KEYS_MAGIC, KEYS_VERSION_1};
pub use rotation::{rotation_due, SEAL_ROTATION_LIMIT};
pub use secret::{Secret32, SecretCell};
