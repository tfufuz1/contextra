//! Persistenz von Scope-Dateien scope/<erasure_id>.scp (SPEC §11.1, D-75, D-85, Anhang F.1 kind=9, F.6).
//!
//! Scope-Dateien verwenden Dateityp kind=9, zufälligen Dateischlüssel (key_mode=0 via FileKeyring).
//! Der Dateiname enthält nur Tags bzw. die ErasureId in Hex (D-75).

use super::codec::{decode_scope, encode_scope};
use super::set::LayerScopeSet;
use crate::crypto_local::Dek;
use crate::crypto_local::SecretKey;
use crate::crypto_local::{open_record, seal_record};
use crate::error::ErasureError;
use crate::types_local::{FileId, FileKeyring, FileKind as PortFileKind};
use contextra_ports::vfs::Vfs;
use std::path::PathBuf;

const FILE_HEADER_MAGIC: &[u8; 8] = b"CTXFILE\0";
const FILE_KIND_SCOPE: u8 = 9;

/// Ermittelt den relativen Pfad der Scope-Datei nach SPEC D-75:
/// `scope/<erasure_id_hex>.scp`
pub fn scope_file_path(erasure_id: &[u8; 16]) -> PathBuf {
    let hex_id = erasure_id
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect::<String>();
    PathBuf::from(format!("scope/{}.scp", hex_id))
}

/// Speichert einen `LayerScopeSet` als persistente, verschlüsselte Scope-Datei (kind=9).
pub async fn store_scope_file(
    vfs: &dyn Vfs,
    keyring: &dyn FileKeyring,
    erasure_id: &[u8; 16],
    file_id: FileId,
    scope: &LayerScopeSet,
) -> Result<(), ErasureError> {
    let plain_bytes = encode_scope(scope)?;

    // Dateischlüssel über FileKeyring anfordern (key_mode = 0, F-Slot im Tresor, D-85)
    let port_key = keyring
        .create(PortFileKind::Auxiliary, file_id)
        .await
        .map_err(|e| ErasureError::Crypto(format!("Keyring create failed: {}", e)))?;

    let mut key_arr = [0u8; 32];
    key_arr.copy_from_slice(port_key.as_bytes());
    let dek = Dek(SecretKey::new(key_arr));

    let mut file_id_arr = [0u8; 16];
    if let Some(dest) = file_id_arr.get_mut(0..8) {
        dest.copy_from_slice(&file_id.as_u64().to_be_bytes());
    }

    // Verschlüssele Scope-Nutzlast über AEAD-Record Envelope (F.6)
    let key_binding = b"ctx/scope-file/v1";
    let envelope = seal_record(&dek, *erasure_id, key_binding, &plain_bytes);
    if envelope.is_empty() {
        return Err(ErasureError::Crypto("Seal scope record failed".to_string()));
    }

    // F.1 Dateikopf (64 Byte)
    let mut header = [0u8; 64];
    if let Some(dest) = header.get_mut(0..8) {
        dest.copy_from_slice(FILE_HEADER_MAGIC);
    }
    if let Some(dest) = header.get_mut(8..10) {
        dest.copy_from_slice(&1u16.to_be_bytes());
    }
    if let Some(dest) = header.get_mut(10) {
        *dest = FILE_KIND_SCOPE;
    }
    if let Some(dest) = header.get_mut(11..15) {
        dest.copy_from_slice(&[1, 0, 1, 1]);
    }
    if let Some(dest) = header.get_mut(15..31) {
        dest.copy_from_slice(&file_id_arr);
    }
    if let Some(dest) = header.get_mut(31..39) {
        dest.copy_from_slice(&0u64.to_be_bytes());
    }
    if let Some(dest) = header.get_mut(39) {
        *dest = 0; // key_mode 0
    }

    // Header-Tag (16 Byte): MAC über Header-Bytes [0..48)
    let header48 = header.get(0..48).ok_or(ErasureError::Corrupt)?;
    let header_mac = blake3::keyed_hash(&key_arr, header48);
    if let (Some(dest), Some(src)) = (header.get_mut(48..64), header_mac.as_bytes().get(0..16)) {
        dest.copy_from_slice(src);
    }

    let len = u32::try_from(envelope.len())
        .map_err(|_| ErasureError::LimitExceeded("Envelope exceeds u32".to_string()))?;

    let mut file_bytes = Vec::with_capacity(64 + 4 + envelope.len());
    file_bytes.extend_from_slice(&header);
    file_bytes.extend_from_slice(&len.to_be_bytes());
    file_bytes.extend_from_slice(&envelope);

    let path = scope_file_path(erasure_id);
    if let Some(parent) = path.parent() {
        vfs.create_dir_all(parent)
            .map_err(|e| ErasureError::Io(std::io::Error::other(e.to_string())))?;
    }
    vfs.write_atomic(&path, &file_bytes)
        .map_err(|e| ErasureError::Io(std::io::Error::other(e.to_string())))?;

    Ok(())
}

/// Lädt einen `LayerScopeSet` aus einer persistenten Scope-Datei (kind=9).
pub async fn load_scope_file(
    vfs: &dyn Vfs,
    keyring: &dyn FileKeyring,
    erasure_id: &[u8; 16],
    file_id: FileId,
) -> Result<LayerScopeSet, ErasureError> {
    let path = scope_file_path(erasure_id);
    let bytes = vfs
        .read(&path)
        .map_err(|e| ErasureError::Io(std::io::Error::other(e.to_string())))?;

    if bytes.len() < 64 + 4 + 16 {
        return Err(ErasureError::Corrupt);
    }

    let header = bytes.get(0..64).ok_or(ErasureError::Corrupt)?;
    let body = bytes.get(64..).ok_or(ErasureError::Corrupt)?;

    if header.get(0..8) != Some(FILE_HEADER_MAGIC) {
        return Err(ErasureError::Corrupt);
    }
    if header.get(10).copied() != Some(FILE_KIND_SCOPE) {
        return Err(ErasureError::Corrupt);
    }

    let port_key = keyring
        .open(file_id)
        .await
        .map_err(|e| ErasureError::Crypto(format!("Keyring open failed: {}", e)))?;

    let mut key_arr = [0u8; 32];
    key_arr.copy_from_slice(port_key.as_bytes());

    // Header-Tag prüfen
    let header48 = header.get(0..48).ok_or(ErasureError::Corrupt)?;
    let expected_mac = blake3::keyed_hash(&key_arr, header48);
    let tag = header.get(48..64).ok_or(ErasureError::Corrupt)?;
    let expected_tag = expected_mac
        .as_bytes()
        .get(0..16)
        .ok_or(ErasureError::Corrupt)?;
    if tag != expected_tag {
        return Err(ErasureError::Corrupt);
    }

    let dek = Dek(SecretKey::new(key_arr));

    // Body parsen: len(u32) || envelope
    let len_bytes = body.get(0..4).ok_or(ErasureError::Corrupt)?;
    let mut len_arr = [0u8; 4];
    len_arr.copy_from_slice(len_bytes);
    let len = usize::try_from(u32::from_be_bytes(len_arr)).map_err(|_| ErasureError::Corrupt)?;

    let envelope = body.get(4..).ok_or(ErasureError::Corrupt)?;
    if envelope.len() < len {
        return Err(ErasureError::Corrupt);
    }

    let key_binding = b"ctx/scope-file/v1";
    let env_slice = envelope.get(..len).ok_or(ErasureError::Corrupt)?;
    let plain = open_record(&dek, key_binding, env_slice).map_err(|_| ErasureError::Corrupt)?;

    decode_scope(&plain)
}
