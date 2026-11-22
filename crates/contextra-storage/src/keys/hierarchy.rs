//! # Schlüsselhierarchie nach SPEC §15.2
//!
//! KDF-Kontexte und deterministische Schlüsselableitungen:
//! - KEK -> KEYS-Schlüssel
//! - root_key -> Redo-, Catalog-, FORMAT-MAC-, Token-Schlüssel
//! - Space-DEK -> Run-, Blob-, Segment-Schlüssel (unter Berücksichtigung von `key_epoch`)

use super::secret::Secret32;
use contextra_storage_ports::Kdf;

/// KDF-Kontext für den KEYS-Dateischlüssel.
pub const CONTEXT_KEYS: &[u8] = b"ctxs keys v1";
/// KDF-Kontext für Redo-Segment-Schlüssel.
pub const CONTEXT_REDO: &[u8] = b"ctxs redo v1";
/// KDF-Kontext für Catalog-Schlüssel.
pub const CONTEXT_CATALOG: &[u8] = b"ctxs cat v1";
/// KDF-Kontext für FORMAT-MAC-Schlüssel.
pub const CONTEXT_FORMAT: &[u8] = b"ctxs format v1";
/// KDF-Kontext für ResumeToken-MAC-Schlüssel.
pub const CONTEXT_TOKEN: &[u8] = b"ctxs token v1";
/// KDF-Kontext für SSTable-Run-Schlüssel.
pub const CONTEXT_RUN: &[u8] = b"ctxs run v1";
/// KDF-Kontext für Blob-Dateischlüssel.
pub const CONTEXT_BLOB: &[u8] = b"ctxs blob v1";
/// KDF-Kontext für anwendungsspezifische Segment-Schlüssel.
pub const CONTEXT_SEGMENT: &[u8] = b"ctxs seg v1";

/// Leitet den KEYS-Schlüssel aus dem KEK ab.
///
/// `KEYS-Schlüssel = Kdf(KEK, "ctxs keys v1")`
pub fn derive_keys_key(kdf: &dyn Kdf, kek: &Secret32) -> Secret32 {
    let key_bytes = kdf.derive(kek.expose(), &[CONTEXT_KEYS]);
    Secret32::new(key_bytes)
}

/// Leitet den Redo-Schlüssel für ein Segment ab.
///
/// `Redo-Schlüssel = Kdf(root_key, "ctxs redo v1" || segment_id u32)`
pub fn derive_redo_key(kdf: &dyn Kdf, root_key: &Secret32, segment_id: u32) -> Secret32 {
    let seg_bytes = segment_id.to_le_bytes();
    let key_bytes = kdf.derive(root_key.expose(), &[CONTEXT_REDO, seg_bytes.as_slice()]);
    Secret32::new(key_bytes)
}

/// Leitet den Catalog-Schlüssel für eine Catalog-Generation ab.
///
/// `Catalog-Schlüssel = Kdf(root_key, "ctxs cat v1" || catalog_no u64)`
pub fn derive_catalog_key(kdf: &dyn Kdf, root_key: &Secret32, catalog_no: u64) -> Secret32 {
    let cat_bytes = catalog_no.to_le_bytes();
    let key_bytes = kdf.derive(root_key.expose(), &[CONTEXT_CATALOG, cat_bytes.as_slice()]);
    Secret32::new(key_bytes)
}

/// Leitet den FORMAT-MAC-Schlüssel ab.
///
/// `FORMAT-MAC-Schl. = Kdf(root_key, "ctxs format v1")`
pub fn derive_format_mac_key(kdf: &dyn Kdf, root_key: &Secret32) -> Secret32 {
    let key_bytes = kdf.derive(root_key.expose(), &[CONTEXT_FORMAT]);
    Secret32::new(key_bytes)
}

/// Leitet den Token-MAC-Schlüssel für ResumeTokens ab.
///
/// `Token-Schlüssel = Kdf(root_key, "ctxs token v1")`
pub fn derive_token_key(kdf: &dyn Kdf, root_key: &Secret32) -> Secret32 {
    let key_bytes = kdf.derive(root_key.expose(), &[CONTEXT_TOKEN]);
    Secret32::new(key_bytes)
}

/// Leitet den Run-Schlüssel für einen SSTable-Run ab.
///
/// `Run-Schlüssel = Kdf(DEK, "ctxs run v1" || key_epoch u32 || run_id u64)`
pub fn derive_run_key(
    kdf: &dyn Kdf,
    space_dek: &Secret32,
    key_epoch: u32,
    run_id: u64,
) -> Secret32 {
    let epoch_bytes = key_epoch.to_le_bytes();
    let run_bytes = run_id.to_le_bytes();
    let key_bytes = kdf.derive(
        space_dek.expose(),
        &[CONTEXT_RUN, epoch_bytes.as_slice(), run_bytes.as_slice()],
    );
    Secret32::new(key_bytes)
}

/// Leitet den Blob-Schlüssel für eine Blob-Datei ab.
///
/// `Blob-Schlüssel = Kdf(DEK, "ctxs blob v1" || key_epoch u32 || blob_no u64)`
pub fn derive_blob_key(
    kdf: &dyn Kdf,
    space_dek: &Secret32,
    key_epoch: u32,
    blob_no: u64,
) -> Secret32 {
    let epoch_bytes = key_epoch.to_le_bytes();
    let blob_bytes = blob_no.to_le_bytes();
    let key_bytes = kdf.derive(
        space_dek.expose(),
        &[CONTEXT_BLOB, epoch_bytes.as_slice(), blob_bytes.as_slice()],
    );
    Secret32::new(key_bytes)
}

/// Leitet den Segment-Schlüssel für ein benutzerdefiniertes Segment ab.
///
/// `Segment-Schlüssel = Kdf(DEK, "ctxs seg v1" || key_epoch u32 || segment_kind u16 || segment_id u64)`
pub fn derive_segment_key(
    kdf: &dyn Kdf,
    space_dek: &Secret32,
    key_epoch: u32,
    segment_kind: u16,
    segment_id: u64,
) -> Secret32 {
    let epoch_bytes = key_epoch.to_le_bytes();
    let kind_bytes = segment_kind.to_le_bytes();
    let seg_bytes = segment_id.to_le_bytes();
    let key_bytes = kdf.derive(
        space_dek.expose(),
        &[
            CONTEXT_SEGMENT,
            epoch_bytes.as_slice(),
            kind_bytes.as_slice(),
            seg_bytes.as_slice(),
        ],
    );
    Secret32::new(key_bytes)
}
