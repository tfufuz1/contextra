//! F.1 / SPEC: Scope-Datei (`kind = 9`).
//!
//! Scope-Dateien erfassen den berechneten Umfang einer Löschoperation
//! (`LayerScope`) unter AEAD des Dateischlüssels.

use crate::crypto_ref::{aes_256_gcm_siv_decrypt, aes_256_gcm_siv_encrypt};
use crate::error::LabError;
use crate::format::file_header::{
    FileHeader, FileKind, FILE_HEADER_SIZE, FILE_MAGIC, STANDARD_ALG_IDS,
};

/// Repräsentiert eine Scope-Datei nach SPEC Anhang F.1 (`kind = 9`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScopeFile {
    /// Dateikopf nach F.1.
    pub header: FileHeader,
    /// Entschlüsselte Nutzlast des Scopes.
    pub payload: Vec<u8>,
}

impl ScopeFile {
    /// Parst eine Scope-Datei und entschlüsselt sie mit dem Dateischlüssel.
    pub fn parse(bytes: &[u8], file_key: &[u8; 32]) -> Result<Self, LabError> {
        let header = FileHeader::parse(bytes)?;
        if header.kind != FileKind::ScopeFile {
            return Err(LabError::UnsupportedKind(header.kind as u8));
        }

        let body = bytes
            .get(FILE_HEADER_SIZE..)
            .ok_or(LabError::UnexpectedEof {
                expected: FILE_HEADER_SIZE,
                actual: bytes.len(),
            })?;

        if body.len() < 12 {
            return Err(LabError::UnexpectedEof {
                expected: 12,
                actual: body.len(),
            });
        }

        let nonce: [u8; 12] = body
            .get(..12)
            .and_then(|s| s.try_into().ok())
            .ok_or(LabError::Corrupted("Scope Nonce fehlt"))?;
        let ct = body
            .get(12..)
            .ok_or(LabError::Corrupted("Scope Ciphertext fehlt"))?;

        let decrypted = aes_256_gcm_siv_decrypt(file_key, &nonce, &header.file_id, ct)?;

        Ok(Self {
            header,
            payload: decrypted,
        })
    }

    /// Serialisiert eine Scope-Datei mit F.1 Dateikopf und AEAD.
    pub fn serialize(
        file_id: [u8; 16],
        payload: &[u8],
        file_key: &[u8; 32],
    ) -> Result<Vec<u8>, LabError> {
        let nonce = [0x12u8; 12];
        let encrypted = aes_256_gcm_siv_encrypt(file_key, &nonce, &file_id, payload)?;

        let mut header_bytes = Vec::with_capacity(FILE_HEADER_SIZE);
        // F.1 header
        header_bytes.extend_from_slice(FILE_MAGIC);
        header_bytes.extend_from_slice(&1u16.to_be_bytes()); // format_version = 1
        header_bytes.push(FileKind::ScopeFile as u8); // kind = 9
        header_bytes.extend_from_slice(&STANDARD_ALG_IDS); // alg_ids
        header_bytes.extend_from_slice(&file_id); // file_id
        header_bytes.extend_from_slice(&[0u8; 8]); // created_seq = 0
        header_bytes.push(FileKind::ScopeFile.expected_key_mode()); // key_mode = 0
        header_bytes.extend_from_slice(&[0u8; 24]); // rest of 64 bytes header

        let mut out = header_bytes;
        out.extend_from_slice(&nonce);
        out.extend_from_slice(&encrypted);
        Ok(out)
    }
}
