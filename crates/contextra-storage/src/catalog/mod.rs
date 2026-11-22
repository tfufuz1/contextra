//! # Katalog-Modul
//!
//! SPEC §9.7: Der Katalog als einzige Wahrheit über lebendige Dateien,
//! atomare Edits (P-2), crash-sicherer Rollover und Typzustand für Dauerhaftigkeit (M-5).

pub mod codec;
pub mod current;
pub mod durable;
pub mod edit;
pub mod field_decode;
pub mod field_encode;
pub mod format;
pub mod log;
pub mod reserve;
pub mod rollover;
pub mod state;
pub mod types;

pub use current::{read_current, write_current, CURRENT_FILE_NAME};
pub use durable::{sync_existing, write_atomic_synced, Synced};
pub use edit::{CatalogEdit, FIELD_OPTIONAL_FLAG};
pub use format::{
    add_required_feature, FormatError, FormatHeader, IntegrityMode, FEATURE_AUTHENTICATED,
    FEATURE_COLD_TIER, FEATURE_DOMAIN_SEALED, FEATURE_ENCRYPTED, FORMAT_AUTH_ENC_LEN,
    FORMAT_MAGIC, FORMAT_PLAIN_LEN, FORMAT_VERSION_1, KNOWN_REQUIRED_FEATURES,
};
pub use log::{catalog_file_name, CatalogHeader, CatalogLog, CATALOG_HEADER_LEN, CATALOG_MAGIC};
pub use reserve::{IdError, IdReserve, ID_RESERVE_BLOCK};
pub use rollover::rollover;
pub use state::{apply_edit, CatalogState};
pub use types::{
    CatalogError, CatalogFileKind, CatalogKeyRange, CatalogSpaceOptions, ErasePendingMeta,
    ErasedRangeMeta, FileMeta, HoldMeta,
};
