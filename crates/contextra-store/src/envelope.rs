//! Record-Envelope v2 nach SPEC Anhang F.6 und §9.4 (D-32).
//!
//! Format:
//! ```text
//! 0   1   version   = 2
//! 1   1   alg_id    = 1 (AES-256-GCM-SIV)
//! 2   16  doc_tag
//! 18  12  nonce     96 Bit OS-CSPRNG
//! 30  n   ciphertext
//! 30+n 16 aead_tag
//! AAD = "ctx/rec/v2" ‖ key_bytes (vollständiger 78-Byte KeyBuf)
//! ```

use contextra_types::{key::KeyBuf, tags::DocTag, ContextraError, Result};

pub const ENVELOPE_V2_VERSION: u8 = 2;
pub const ENVELOPE_V2_ALG_ID: u8 = 1;
pub const NONCE_LEN: usize = 12;
pub const AEAD_TAG_LEN: usize = 16;
pub const HEADER_LEN: usize = 1 + 1 + 16 + NONCE_LEN; // 30 Bytes

/// Repräsentiert einen entschlüsselbaren oder versiegelten Record-Envelope v2.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordEnvelopeV2 {
    pub doc_tag: DocTag,
    pub nonce: [u8; NONCE_LEN],
    pub ciphertext: Vec<u8>,
    pub aead_tag: [u8; AEAD_TAG_LEN],
}

impl RecordEnvelopeV2 {
    /// Kodiert den Envelope in Bytes gemäß F.6.
    pub fn encode(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(HEADER_LEN + self.ciphertext.len() + AEAD_TAG_LEN);
        out.push(ENVELOPE_V2_VERSION);
        out.push(ENVELOPE_V2_ALG_ID);
        out.extend_from_slice(self.doc_tag.as_bytes());
        out.extend_from_slice(&self.nonce);
        out.extend_from_slice(&self.ciphertext);
        out.extend_from_slice(&self.aead_tag);
        out
    }

    /// Dekodiert und validiert einen Envelope aus Rohbytes.
    /// Gibt `ContextraError::Corrupt` zurück, falls Version != 2 oder zu kurz.
    pub fn decode(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < HEADER_LEN + AEAD_TAG_LEN {
            return Err(ContextraError::Storage(
                "RecordEnvelopeV2: Payload zu kurz".into(),
            ));
        }

        let version = *bytes
            .first()
            .ok_or_else(|| ContextraError::Storage("empty envelope".into()))?;
        if version != ENVELOPE_V2_VERSION {
            return Err(ContextraError::Storage(format!(
                "RecordEnvelopeV2: Ungültige Version {version}, erwartet 2 (Legacy v1 nicht unterstützt)"
            )));
        }

        let alg_id = *bytes
            .get(1)
            .ok_or_else(|| ContextraError::Storage("missing alg_id".into()))?;
        if alg_id != ENVELOPE_V2_ALG_ID {
            return Err(ContextraError::Storage(format!(
                "RecordEnvelopeV2: Nicht unterstützte alg_id {alg_id}"
            )));
        }

        let mut doc_tag_bytes = [0u8; 16];
        let doc_tag_slice = bytes
            .get(2..18)
            .ok_or_else(|| ContextraError::Storage("invalid doc_tag slice".into()))?;
        doc_tag_bytes.copy_from_slice(doc_tag_slice);
        let doc_tag = DocTag::new(doc_tag_bytes);

        let mut nonce = [0u8; NONCE_LEN];
        let nonce_slice = bytes
            .get(18..30)
            .ok_or_else(|| ContextraError::Storage("invalid nonce slice".into()))?;
        nonce.copy_from_slice(nonce_slice);

        let ct_len = bytes.len().saturating_sub(HEADER_LEN + AEAD_TAG_LEN);
        let ct_slice = bytes
            .get(30..30 + ct_len)
            .ok_or_else(|| ContextraError::Storage("invalid ct slice".into()))?;
        let ciphertext = ct_slice.to_vec();

        let mut aead_tag = [0u8; AEAD_TAG_LEN];
        let tag_slice = bytes
            .get(30 + ct_len..)
            .ok_or_else(|| ContextraError::Storage("invalid tag slice".into()))?;
        aead_tag.copy_from_slice(tag_slice);

        Ok(Self {
            doc_tag,
            nonce,
            ciphertext,
            aead_tag,
        })
    }

    /// Konstruiert AAD für AEAD: `"ctx/rec/v2" ‖ key_bytes`.
    pub fn construct_aad(key: &KeyBuf) -> Vec<u8> {
        let prefix = b"ctx/rec/v2";
        let mut aad = Vec::with_capacity(prefix.len() + key.as_bytes().len());
        aad.extend_from_slice(prefix);
        aad.extend_from_slice(key.as_bytes());
        aad
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_envelope_v2_roundtrip() {
        let env = RecordEnvelopeV2 {
            doc_tag: DocTag::new([42u8; 16]),
            nonce: [7u8; 12],
            ciphertext: b"encrypted secret document chunk".to_vec(),
            aead_tag: [9u8; 16],
        };

        let encoded = env.encode();
        let decoded = RecordEnvelopeV2::decode(&encoded).expect("decode");
        assert_eq!(env, decoded);
    }

    #[test]
    fn test_envelope_v1_rejected_fail_closed() {
        let mut bad_v1 = vec![1u8]; // Version 1
        bad_v1.resize(HEADER_LEN + AEAD_TAG_LEN, 0u8);
        assert!(RecordEnvelopeV2::decode(&bad_v1).is_err());
    }
}
