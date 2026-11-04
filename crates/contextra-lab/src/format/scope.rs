//! F.1 Scope-Datei Parser (`erasure/scope/<erasure_id>.scp`, SPEC §11.1, Anhang F.1, ER-17, D-60, D-85).
//!
//! SPEC Anhang F.1:
//! - Dateikopf nach F.1 (kind = 9, ScopeFile, key_mode = 0, d. h. zufälliger Dateischlüssel) -> 64 Bytes
//! - Körper: Block-AEAD unter dem zufälligen Dateischlüssel (F-Slot im Tresor)
//! - Block := u32 len ‖ ciphertext(len) ‖ aead_tag(16)
//!   Nonce 0x00000000 ‖ u64(Blockindex), AAD: kind ‖ file_id ‖ u64(Blockindex)
//! - Klartext: JSON- oder Byte-Repräsentation des aufgelösten `LayerScopeSet`

use aes_gcm_siv::aead::{Aead, KeyInit, Payload};
use aes_gcm_siv::{Aes256GcmSiv, Nonce};
use serde::{Deserialize, Serialize};

use crate::error::LabError;
use crate::format::file_header::{FileHeader, FileKind, FILE_HEADER_SIZE};

/// Auflösung einer Schicht in der Scope-Datei.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScopeEntry {
    /// Schichtkennung (16 Byte).
    pub layer_id: [u8; 16],
    /// Anzahl betroffener Einheiten.
    pub unit_count: u64,
    /// Opake Einheitsliste (Schicht-spezifisch).
    pub units: Vec<u64>,
}

/// Geparste Scope-Datei (`kind = 9`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScopeFile {
    /// F.1 Dateikopf.
    pub header: FileHeader,
    /// Entschlüsselte Scope-Einträge aller Schichten.
    pub scopes: Vec<ScopeEntry>,
}

impl ScopeFile {
    /// Parst eine Scope-Datei und entschlüsselt den Körper mit dem Dateischlüssel.
    pub fn parse(bytes: &[u8], file_key: &[u8; 32]) -> Result<Self, LabError> {
        let header = FileHeader::parse(bytes)?;
        if header.kind != FileKind::ScopeFile {
            return Err(LabError::UnsupportedKind(header.kind as u8));
        }

        // Verifiziere den Dateikopf mit dem Dateischlüssel
        let raw_head = bytes
            .get(0..FILE_HEADER_SIZE)
            .ok_or(LabError::UnexpectedEof {
                expected: FILE_HEADER_SIZE,
                actual: bytes.len(),
            })?;
        header.verify_tag(raw_head, file_key)?;

        let mut offset = FILE_HEADER_SIZE;
        let mut block_idx: u64 = 0;
        let mut decrypted_payload = Vec::new();

        let cipher = Aes256GcmSiv::new_from_slice(file_key)
            .map_err(|_| LabError::Corrupted("Schlüssellänge ungültig"))?;

        while offset < bytes.len() {
            if offset.saturating_add(4) > bytes.len() {
                return Err(LabError::UnexpectedEof {
                    expected: offset.saturating_add(4),
                    actual: bytes.len(),
                });
            }

            let len_bytes: [u8; 4] = bytes
                .get(offset..offset + 4)
                .and_then(|s| s.try_into().ok())
                .ok_or(LabError::Corrupted("Blocklänge fehlt"))?;
            let len = u32::from_be_bytes(len_bytes) as usize;

            let total_block_len = 4usize.saturating_add(len).saturating_add(16);
            let end = offset.saturating_add(total_block_len);
            if end > bytes.len() {
                return Err(LabError::UnexpectedEof {
                    expected: end,
                    actual: bytes.len(),
                });
            }

            let ct = bytes
                .get(offset + 4..offset + 4 + len)
                .ok_or(LabError::Corrupted("Ciphertext fehlt"))?;
            let tag = bytes
                .get(offset + 4 + len..end)
                .ok_or(LabError::Corrupted("Tag fehlt"))?;

            let mut ct_and_tag = Vec::with_capacity(len + 16);
            ct_and_tag.extend_from_slice(ct);
            ct_and_tag.extend_from_slice(tag);

            // Nonce: 0x00000000 || u64(Blockindex)
            let mut nonce_bytes = [0u8; 12];
            if let Some(dest) = nonce_bytes.get_mut(4..12) {
                dest.copy_from_slice(&block_idx.to_be_bytes());
            }

            // AAD: kind || file_id || u64(Blockindex)
            let mut aad = Vec::with_capacity(1 + 16 + 8);
            aad.push(FileKind::ScopeFile as u8);
            aad.extend_from_slice(&header.file_id);
            aad.extend_from_slice(&block_idx.to_be_bytes());

            let payload = Payload {
                msg: &ct_and_tag,
                aad: &aad,
            };

            let pt = cipher
                .decrypt(Nonce::from_slice(&nonce_bytes), payload)
                .map_err(|_| LabError::VerificationFailed("Scope Block AEAD fehlerhaft"))?;

            decrypted_payload.extend_from_slice(&pt);

            offset = end;
            block_idx = block_idx.saturating_add(1);
        }

        let scopes: Vec<ScopeEntry> = serde_json::from_slice(&decrypted_payload)
            .map_err(|_| LabError::Corrupted("Scope JSON Deserialisierung fehlgeschlagen"))?;

        Ok(Self { header, scopes })
    }

    /// Hilfsmethode: Erzeugt eine serialisierte Scope-Datei aus Scopes.
    pub fn create_bytes(
        file_id: [u8; 16],
        created_seq: u64,
        scopes: &[ScopeEntry],
        file_key: &[u8; 32],
    ) -> Result<Vec<u8>, LabError> {
        let header_bytes =
            FileHeader::create_bytes(FileKind::ScopeFile, file_id, created_seq, file_key);

        let payload_json = serde_json::to_vec(scopes)
            .map_err(|_| LabError::Corrupted("Scope Serialisierung fehlgeschlagen"))?;

        let cipher = Aes256GcmSiv::new_from_slice(file_key)
            .map_err(|_| LabError::Corrupted("Schlüssellänge ungültig"))?;

        let mut out = Vec::new();
        out.extend_from_slice(&header_bytes);

        // Schreibe einen Block mit Blockindex 0
        let block_idx: u64 = 0;
        let mut nonce_bytes = [0u8; 12];
        if let Some(dest) = nonce_bytes.get_mut(4..12) {
            dest.copy_from_slice(&block_idx.to_be_bytes());
        }

        let mut aad = Vec::with_capacity(1 + 16 + 8);
        aad.push(FileKind::ScopeFile as u8);
        aad.extend_from_slice(&file_id);
        aad.extend_from_slice(&block_idx.to_be_bytes());

        let payload = Payload {
            msg: &payload_json,
            aad: &aad,
        };

        let ct_with_tag = cipher
            .encrypt(Nonce::from_slice(&nonce_bytes), payload)
            .map_err(|_| LabError::VerificationFailed("Verschlüsselung fehlgeschlagen"))?;

        let ct_len = ct_with_tag.len().saturating_sub(16);
        let len_u32 = u32::try_from(ct_len).map_err(|_| LabError::Corrupted("Block zu groß"))?;

        out.extend_from_slice(&len_u32.to_be_bytes());
        out.extend_from_slice(&ct_with_tag);

        Ok(out)
    }
}
