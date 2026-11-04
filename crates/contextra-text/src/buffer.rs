//! Textpuffer (`buffer.rs`) und Identitätsentkopplung nach SPEC 9.7, D-49, D-69.

use aes_gcm_siv::aead::{Aead, KeyInit, Payload};
use aes_gcm_siv::{Aes256GcmSiv, Key, Nonce};
use contextra_crypto::dek::Dek;
use contextra_crypto::labels::TBIND_AAD;
use contextra_types::{ContextraError, Result};

pub fn seal_binding_envelope(
    dek: &Dek,
    file_id: [u8; 16],
    doc_ord: u32,
    cluster_tag: [u8; 16],
    doc_tag: [u8; 16],
) -> Result<Vec<u8>> {
    let mut plain = [0u8; 32];
    if let Some(dest) = plain.get_mut(0..16) {
        dest.copy_from_slice(&cluster_tag);
    }
    if let Some(dest) = plain.get_mut(16..32) {
        dest.copy_from_slice(&doc_tag);
    }

    let mut aad = Vec::with_capacity(TBIND_AAD.len() + 16 + 4);
    aad.extend_from_slice(TBIND_AAD);
    aad.extend_from_slice(&file_id);
    aad.extend_from_slice(&doc_ord.to_be_bytes());

    let cipher = Aes256GcmSiv::new(Key::<Aes256GcmSiv>::from_slice(dek.0.as_bytes()));
    let nonce = [0u8; 12];

    let ct_with_tag = cipher
        .encrypt(
            Nonce::from_slice(&nonce),
            Payload {
                msg: &plain,
                aad: &aad,
            },
        )
        .map_err(|_| ContextraError::Text("Failed to encrypt binding envelope".into()))?;

    let mut envelope = Vec::with_capacity(1 + 1 + 12 + ct_with_tag.len());
    envelope.push(2); // version = 2
    envelope.push(1); // alg_id = 1
    envelope.extend_from_slice(&nonce);
    envelope.extend_from_slice(&ct_with_tag);

    Ok(envelope)
}

pub fn open_binding_envelope(
    dek: &Dek,
    file_id: [u8; 16],
    doc_ord: u32,
    envelope: &[u8],
) -> Result<([u8; 16], [u8; 16])> {
    if envelope.len() < 1 + 1 + 12 + 32 + 16 {
        return Err(ContextraError::Index("Envelope too short".into()));
    }
    if envelope.first().copied() != Some(2) || envelope.get(1).copied() != Some(1) {
        return Err(ContextraError::Index("Invalid envelope header".into()));
    }

    let nonce = envelope
        .get(2..14)
        .ok_or_else(|| ContextraError::Index("Missing nonce".into()))?;
    let ct = envelope
        .get(14..)
        .ok_or_else(|| ContextraError::Index("Missing ciphertext".into()))?;

    let mut aad = Vec::with_capacity(TBIND_AAD.len() + 16 + 4);
    aad.extend_from_slice(TBIND_AAD);
    aad.extend_from_slice(&file_id);
    aad.extend_from_slice(&doc_ord.to_be_bytes());

    let cipher = Aes256GcmSiv::new(Key::<Aes256GcmSiv>::from_slice(dek.0.as_bytes()));
    let plain = cipher
        .decrypt(Nonce::from_slice(nonce), Payload { msg: ct, aad: &aad })
        .map_err(|_| {
            ContextraError::Text("KeyDestroyed / Auth failure: binding cannot be decrypted".into())
        })?;

    if plain.len() != 32 {
        return Err(ContextraError::Index("Invalid plain length".into()));
    }

    let mut cluster_tag = [0u8; 16];
    let mut doc_tag = [0u8; 16];
    let ct_slice = plain
        .get(0..16)
        .ok_or_else(|| ContextraError::Index("corrupted cluster tag".into()))?;
    cluster_tag.copy_from_slice(ct_slice);
    let dt_slice = plain
        .get(16..32)
        .ok_or_else(|| ContextraError::Index("corrupted doc tag".into()))?;
    doc_tag.copy_from_slice(dt_slice);

    Ok((cluster_tag, doc_tag))
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TextBufRecord {
    pub cluster_tag: [u8; 16],
    pub doc_tag: [u8; 16],
    pub text: String,
    pub doc_ord: u32,
}

#[derive(Default)]
pub struct TextBuffer {
    records: Vec<TextBufRecord>,
}

impl TextBuffer {
    pub fn new() -> Self {
        Self {
            records: Vec::new(),
        }
    }

    pub fn insert(&mut self, cluster_tag: [u8; 16], doc_tag: [u8; 16], text: String) -> u32 {
        let doc_ord = u32::try_from(self.records.len()).unwrap_or(u32::MAX);
        self.records.push(TextBufRecord {
            cluster_tag,
            doc_tag,
            text,
            doc_ord,
        });
        doc_ord
    }

    pub fn len(&self) -> usize {
        self.records.len()
    }

    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }

    pub fn drain(&mut self) -> Vec<TextBufRecord> {
        std::mem::take(&mut self.records)
    }
}
