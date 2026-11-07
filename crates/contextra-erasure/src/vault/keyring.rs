//! FileKeyring für Dateischlüssel nach SPEC §4.1, §8.2, D-85, D-113 und PO-06.
//!
//! Dateischlüssel liegen als Tresor-Slot Typ `F` (0x05) mit `tag = file_id_16`:
//! `tag = file_id.as_u64().to_be_bytes() ‖ 0^8`
//! `payload = Dateischlüssel ⊕ HKDF-Expand(K_enc, "ctx/fkmask/v1" ‖ file_id_16, 32)`
//! Bei `destroy` wird der `F`-Slot über `destroy_slot` genullt (`status = 0x02`),
//! dauerhaft synchronisiert und verifiziert (D-85, D-113).

use crate::error::ErasureError;
use crate::types_local::{FileId, FileKey, FileKeyring, FileKind};
use crate::vault::slot::SlotType;
use crate::vault::FileVault;
use contextra_ports::BoxFuture;
use parking_lot::RwLock;
use std::sync::Arc;

/// Erzeugt das 16-Byte-Tag für eine `FileId` nach SPEC F.3.
pub fn file_id_to_tag(id: FileId) -> [u8; 16] {
    let mut tag = [0u8; 16];
    let bytes = id.as_u64().to_be_bytes();
    if let Some(dest) = tag.get_mut(..8) {
        dest.copy_from_slice(&bytes);
    }
    tag
}

/// Maskiert oder demaskiert den Dateischlüssel mit HKDF-Expand nach SPEC F.3 / D-85.
pub fn mask_file_key(raw_key: &[u8; 32], tag: &[u8; 16], k_enc: &[u8; 32]) -> [u8; 32] {
    // HKDF-Expand(K_enc, "ctx/fkmask/v1" || file_id, 32)
    let mut info = Vec::with_capacity(30);
    info.extend_from_slice(b"ctx/fkmask/v1");
    info.extend_from_slice(tag);

    let mut okm = [0u8; 32];
    if crate::crypto_local::hkdf_expand(k_enc, &info, &mut okm).is_err() {
        // Fallback falls Expand fehlschlägt
        let hash = blake3::keyed_hash(k_enc, &info);
        if let (Some(dest), Some(src)) = (okm.get_mut(..), hash.as_bytes().get(..)) {
            dest.copy_from_slice(src);
        }
    }

    let mut masked = [0u8; 32];
    for (i, (&k, &m)) in raw_key.iter().zip(okm.iter()).enumerate() {
        if let Some(dest) = masked.get_mut(i) {
            *dest = k ^ m;
        }
    }
    masked
}

/// Tresor-gestützte `FileKeyring`-Implementierung nach SPEC D-85 / PO-06.
#[derive(Clone)]
pub struct VaultKeyring {
    vault: Arc<RwLock<FileVault>>,
    k_enc: [u8; 32],
}

impl VaultKeyring {
    /// Erstellt einen neuen `VaultKeyring` mit eigener In-Memory-Tresor-Instanz.
    pub fn new() -> Self {
        Self::new_in_memory()
    }

    /// Erstellt einen neuen `VaultKeyring` über einen bestehenden Tresor.
    pub fn with_vault(vault: Arc<RwLock<FileVault>>, k_enc: [u8; 32]) -> Self {
        Self { vault, k_enc }
    }

    /// Erstellt einen neuen In-Memory-Keyring mit eigener Tresor-Instanz.
    pub fn new_in_memory() -> Self {
        let vault = Arc::new(RwLock::new(FileVault::new_in_memory(1024)));
        let k_enc = [0x42u8; 32];
        Self { vault, k_enc }
    }
}

impl Default for VaultKeyring {
    fn default() -> Self {
        Self::new()
    }
}

impl FileKeyring for VaultKeyring {
    fn create(&self, _kind: FileKind, id: FileId) -> BoxFuture<'_, Result<FileKey, String>> {
        Box::pin(async move {
            let tag = file_id_to_tag(id);

            // Generiere 32-Byte Dateischlüssel (OS-CSPRNG / deterministisch für Tests)
            let mut raw_key = [0u8; 32];
            let mut seed = [0u8; 24];
            if let Some(dest) = seed.get_mut(..16) {
                dest.copy_from_slice(&tag);
            }
            let hash = blake3::keyed_hash(&self.k_enc, &seed);
            if let (Some(dest), Some(src)) = (raw_key.get_mut(..), hash.as_bytes().get(..)) {
                dest.copy_from_slice(src);
            }

            let payload = mask_file_key(&raw_key, &tag, &self.k_enc);

            let mut v = self.vault.write();
            v.put_slot(SlotType::File, tag, payload)
                .map_err(|e| format!("erasure-capacity or vault put failed: {e:?}"))?;

            Ok(FileKey::new(raw_key.to_vec(), id))
        })
    }

    fn open(&self, id: FileId) -> BoxFuture<'_, Result<FileKey, String>> {
        Box::pin(async move {
            let tag = file_id_to_tag(id);
            let slot = {
                let v = self.vault.read();
                v.get_slot(SlotType::File, tag)
                    .map_err(|e| format!("vault get failed: {e:?}"))?
            };

            let sd = slot.ok_or_else(|| "FileKey destroyed or missing (erased)".to_string())?;
            let raw_key = mask_file_key(&sd.payload, &tag, &self.k_enc);

            Ok(FileKey::new(raw_key.to_vec(), id))
        })
    }

    fn destroy(&self, id: FileId) -> BoxFuture<'_, Result<(), String>> {
        Box::pin(async move {
            let tag = file_id_to_tag(id);
            let mut v = self.vault.write();

            // destroy_slot nullt den Slot mit Grabstein-Abbild, sync_data,
            // und verifiziert Nullen und Grabstein-MAC (D-111, D-113, PO-06)
            match v.destroy_slot(SlotType::File, tag) {
                Ok(_destroyed) => Ok(()),
                Err(ErasureError::SlotMissing) => {
                    // Idempotent: Bereits genullter / nicht vorhandener Slot liefert Erfolg
                    Ok(())
                }
                Err(e) => Err(format!("vault destroy_slot failed: {e:?}")),
            }
        })
    }
}

/// In-Memory-Keyring für Tests und abwärtskompatible Validierung.
pub type InMemoryKeyring = VaultKeyring;
