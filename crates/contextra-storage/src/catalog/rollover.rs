//! # Katalog-Rollover
//!
//! SPEC §9.7 und §13.2: Führt den crash-sicheren Wechsel zu einer neuen Katalog-Datei durch.
//!
//! Reihenfolge aus Absturzmatrix:
//! 1. Neue Datei CATALOG-(n+1) mit Snapshot schreiben und sync_all
//! 2. CURRENT per write_atomic umhängen (sync_dir)
//! 3. Alte Datei CATALOG-n löschen und sync_dir

use crate::catalog::current::write_current;
use crate::catalog::log::{
    catalog_file_name, fold_u64_to_u32, CatalogHeader, CATALOG_HEADER_LEN_U64, RECORD_TAG_SNAPSHOT,
};
use crate::catalog::state::CatalogState;
use crate::catalog::types::CatalogError;
use crate::codec::crc32c::crc32c;
use crate::keys::nonce::catalog_record_nonce;
use contextra_storage_ports::{Cipher, Fs, Kdf, Mac, OpenMode, Perm};
use std::path::Path;

/// Führt einen Rollover zu einer neuen Katalog-Datei durch.
pub use rollover as rollover_catalog;

/// Führt einen Rollover zu einer neuen Katalog-Datei durch.
#[allow(clippy::too_many_arguments)]
pub fn rollover(
    fs: &dyn Fs,
    root: &Path,
    old_catalog_no: u64,
    state: &CatalogState,
    integrity_mode: u8,
    root_key: Option<&[u8; 32]>,
    cipher: Option<&dyn Cipher>,
    mac: Option<&dyn Mac>,
    kdf: Option<&dyn Kdf>,
) -> Result<u64, CatalogError> {
    let new_catalog_no = old_catalog_no
        .checked_add(1)
        .ok_or(CatalogError::InvalidFormat("Catalog-Nummer-Überlauf"))?;

    let new_file_name = catalog_file_name(new_catalog_no);
    let new_path = root.join(&new_file_name);

    let new_file = fs
        .open(&new_path, OpenMode::CreateNew(Perm::Private))
        .map_err(CatalogError::from)?;

    let header = CatalogHeader::new(new_catalog_no);
    let header_buf = header.encode();
    new_file.write_all_at(&header_buf, 0).map_err(CatalogError::from)?;

    let offset = CATALOG_HEADER_LEN_U64;
    let snapshot_edit = state.to_snapshot_edit();
    let payload = snapshot_edit.encode();

    let payload_len_u32 =
        u32::try_from(payload.len()).map_err(|_| CatalogError::InvalidFormat("Payload-Überlauf"))?;
    let len = payload_len_u32
        .checked_add(1)
        .ok_or(CatalogError::InvalidFormat("Überlauf"))?;
    let tag = RECORD_TAG_SNAPSHOT;

    let mut record_bytes = Vec::new();
    record_bytes.extend_from_slice(&len.to_le_bytes());
    record_bytes.push(tag);

    let chk_seed = fold_u64_to_u32(new_catalog_no);

    match integrity_mode {
        0 => {
            let mut check_data = Vec::with_capacity(1 + payload.len());
            check_data.push(tag);
            check_data.extend_from_slice(&payload);
            let chk = crc32c(chk_seed, &check_data);
            record_bytes.extend_from_slice(&payload);
            record_bytes.extend_from_slice(&chk.to_le_bytes());
        }
        1 => {
            let m = mac.ok_or(CatalogError::Corrupt("Mac fehlt"))?;
            let rk = root_key.ok_or(CatalogError::Corrupt("Root-Key fehlt"))?;
            let cv_b = chk_seed.to_le_bytes();
            let l_b = len.to_le_bytes();
            let t_b = [tag];
            let c_no_b = new_catalog_no.to_le_bytes();
            let idx_b = 0u64.to_le_bytes();
            let tag_out = m.mac16(rk, &[&cv_b, &l_b, &t_b, &payload, &c_no_b, &idx_b]);
            record_bytes.extend_from_slice(&payload);
            record_bytes.extend_from_slice(&tag_out);
        }
        2 => {
            let c = cipher.ok_or(CatalogError::Corrupt("Cipher fehlt"))?;
            let k = kdf.ok_or(CatalogError::Corrupt("Kdf fehlt"))?;
            let rk = root_key.ok_or(CatalogError::Corrupt("Root-Key fehlt"))?;
            let cat_key = k.derive(rk, &[b"ctxs cat v1", &new_catalog_no.to_le_bytes()]);
            let nonce = catalog_record_nonce(0);
            let mut aad = Vec::new();
            aad.extend_from_slice(&chk_seed.to_le_bytes());
            aad.extend_from_slice(&len.to_le_bytes());
            aad.extend_from_slice(&new_catalog_no.to_le_bytes());
            aad.extend_from_slice(&0u64.to_le_bytes());
            let sealed = c.seal(&cat_key, &nonce, &aad, &payload);
            record_bytes.extend_from_slice(&sealed);
        }
        _ => return Err(CatalogError::Corrupt("Ungültiger Integritätsmodus")),
    }

    new_file.write_all_at(&record_bytes, offset).map_err(CatalogError::from)?;
    new_file.sync_all().map_err(CatalogError::from)?;
    drop(new_file);

    // Absturzmatrix Schritt 2: CURRENT atomar umhängen
    write_current(fs, root, new_catalog_no).map_err(CatalogError::from)?;

    // Absturzmatrix Schritt 3: Alte Datei löschen und sync_dir
    let old_file_name = catalog_file_name(old_catalog_no);
    let old_path = root.join(&old_file_name);
    let _ = fs.remove_file(&old_path);
    let _ = fs.sync_dir(root);

    Ok(new_catalog_no)
}
