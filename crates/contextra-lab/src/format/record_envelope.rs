//! F.6 Record-Envelope v2 Parser.
//!
//! SPEC Anhang F.6:
//! - 0..1: version = 2
//! - 1..2: alg_id = 1 (AES-256-GCM-SIV)
//! - 2..18: doc_tag [u8; 16]
//! - 18..30: nonce [u8; 12] (96 Bit OS-CSPRNG)
//! - 30..30+n: ciphertext
//! - 30+n..30+n+16: aead_tag [u8; 16]
//! - AAD: "ctx/rec/v2" || key_bytes (vollständiger SSTable-Schlüssel inkl. !commit_seq)
//! - Klartext: u8 type || Felder; erstes Feld DocRef (subject?, key)

use aes_gcm_siv::aead::{Aead, KeyInit, Payload};
use aes_gcm_siv::{Aes256GcmSiv, Nonce};

use crate::error::LabError;

/// Minimalgröße für Record-Envelope v2: 1 + 1 + 16 + 12 + 0 + 16 = 46 Bytes.
pub const MIN_RECORD_ENVELOPE_SIZE: usize = 46;

/// Version für Record-Envelope v2.
pub const RECORD_ENVELOPE_V2: u8 = 2;

/// Standard Algorithmus-ID (1 = AES-256-GCM-SIV).
pub const ALG_AES_256_GCM_SIV: u8 = 1;

/// Strukturierte Hülle eines Datensatzes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordEnvelopeV2 {
    /// Version (2).
    pub version: u8,
    /// Algorithmus-ID (1).
    pub alg_id: u8,
    /// Dokument-Tag (16 Byte).
    pub doc_tag: [u8; 16],
    /// Nonce (12 Byte).
    pub nonce: [u8; 12],
    /// Chiffrat.
    pub ciphertext: Vec<u8>,
    /// AEAD-Tag (16 Byte).
    pub aead_tag: [u8; 16],
}

impl RecordEnvelopeV2 {
    /// Parst die Rohbytes eines Record-Envelope v2.
    pub fn parse(bytes: &[u8]) -> Result<Self, LabError> {
        if bytes.len() < MIN_RECORD_ENVELOPE_SIZE {
            return Err(LabError::UnexpectedEof {
                expected: MIN_RECORD_ENVELOPE_SIZE,
                actual: bytes.len(),
            });
        }

        let version = *bytes.first().ok_or(LabError::Corrupted("Version fehlt"))?;
        if version != RECORD_ENVELOPE_V2 {
            return Err(LabError::UnsupportedVersion(version as u16));
        }

        let alg_id = *bytes.get(1).ok_or(LabError::Corrupted("AlgId fehlt"))?;
        if alg_id != ALG_AES_256_GCM_SIV {
            return Err(LabError::UnsupportedAlgIds([alg_id, 0, 0, 0]));
        }

        let doc_tag: [u8; 16] = bytes
            .get(2..18)
            .and_then(|s| s.try_into().ok())
            .ok_or(LabError::Corrupted("DocTag fehlt"))?;

        let nonce: [u8; 12] = bytes
            .get(18..30)
            .and_then(|s| s.try_into().ok())
            .ok_or(LabError::Corrupted("Nonce fehlt"))?;

        let ct_len = bytes.len() - 30 - 16;
        let ciphertext = bytes
            .get(30..30 + ct_len)
            .ok_or(LabError::Corrupted("Ciphertext fehlt"))?
            .to_vec();

        let aead_tag: [u8; 16] = bytes
            .get(30 + ct_len..bytes.len())
            .and_then(|s| s.try_into().ok())
            .ok_or(LabError::Corrupted("Tag fehlt"))?;

        Ok(Self {
            version,
            alg_id,
            doc_tag,
            nonce,
            ciphertext,
            aead_tag,
        })
    }

    /// Entschlüsselt den Record-Envelope unter dem Dokumentenschlüssel (DEK).
    /// AAD = "ctx/rec/v2" || key_bytes
    pub fn decrypt(&self, dek: &[u8; 32], key_bytes: &[u8]) -> Result<Vec<u8>, LabError> {
        let mut aad = Vec::with_capacity(10 + key_bytes.len());
        aad.extend_from_slice(b"ctx/rec/v2");
        aad.extend_from_slice(key_bytes);

        let cipher = Aes256GcmSiv::new_from_slice(dek)
            .map_err(|_| LabError::Corrupted("DEK Länge ungültig"))?;

        let mut ct_and_tag = self.ciphertext.clone();
        ct_and_tag.extend_from_slice(&self.aead_tag);

        let payload = Payload {
            msg: &ct_and_tag,
            aad: &aad,
        };

        cipher
            .decrypt(Nonce::from_slice(&self.nonce), payload)
            .map_err(|_| {
                LabError::VerificationFailed("Record-Envelope Entschlüsselung fehlgeschlagen")
            })
    }

    /// Hilfsfunktion zur Erzeugung eines Record-Envelope v2.
    pub fn create_bytes(
        doc_tag: [u8; 16],
        nonce: [u8; 12],
        plaintext: &[u8],
        dek: &[u8; 32],
        key_bytes: &[u8],
    ) -> Result<Vec<u8>, LabError> {
        let mut aad = Vec::with_capacity(10 + key_bytes.len());
        aad.extend_from_slice(b"ctx/rec/v2");
        aad.extend_from_slice(key_bytes);

        let cipher = Aes256GcmSiv::new_from_slice(dek)
            .map_err(|_| LabError::Corrupted("DEK Länge ungültig"))?;

        let payload = Payload {
            msg: plaintext,
            aad: &aad,
        };

        let ct_and_tag = cipher
            .encrypt(Nonce::from_slice(&nonce), payload)
            .map_err(|_| LabError::Corrupted("Verschlüsselung fehlgeschlagen"))?;

        let mut out = Vec::with_capacity(30 + ct_and_tag.len());
        out.push(RECORD_ENVELOPE_V2);
        out.push(ALG_AES_256_GCM_SIV);
        out.extend_from_slice(&doc_tag);
        out.extend_from_slice(&nonce);
        out.extend_from_slice(&ct_and_tag);
        Ok(out)
    }
}
