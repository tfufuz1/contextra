//! # Block-Rahmen und Integrität
//!
//! Format (SPEC §9.5):
//! `Block := stored_payload | codec u8 | chk (Encrypted: stored_payload = sealed, kein separates chk)`
//! `AAD (Authenticated/Encrypted) = 0x01 (Run) || run_id || block_offset || block_kind || space`
//! `block_kind: 0 Data, 1 Index, 2 Filter, 3 PrefixFilter, 4 RangeDel, 5 Meta, 6 Directory, 0x1000+id Extension`
//! Blockcodec 0 (keine Kompression), Trait reserviert (SD-13).
//! Keine Integer-Casts (M-02 = 0).

use super::error::RunError;
use crate::codec::crc32c::crc32c;
use crate::keys::nonce::run_block_nonce;
use contextra_storage_ports::{Cipher, Mac};
use std::io;

/// Dateikennung für Run in AAD (0x01).
pub const FILE_KIND_RUN: u8 = 0x01;

/// Standard-Block-Codec: keine Kompression (0).
pub const CODEC_NONE: u8 = 0x00;

/// Block-Arten nach SPEC §9.5.
pub mod block_kind {
    /// Nutzdaten-Block mit Schlüssel-Wert-Einträgen.
    pub const DATA: u16 = 0;
    /// Index-Block für Datenblock-Suche.
    pub const INDEX: u16 = 1;
    /// Bloom-Filter über distinkte Nutzerschlüssel.
    pub const FILTER: u16 = 2;
    /// Präfix-Bloom-Filter über erste n Schlüsselbytes.
    pub const PREFIX_FILTER: u16 = 3;
    /// Bereichslöschungen (RangeDel und EraseRange).
    pub const RANGE_DEL: u16 = 4;
    /// Metadaten des Runs (Statistiken, Schlüsselgrenzen).
    pub const META: u16 = 5;
    /// Inhaltsverzeichnis aller Metadaten- und Indexblöcke.
    pub const DIRECTORY: u16 = 6;
    /// Basis-Offset für Erweiterungsblöcke (SPEC §12.4).
    pub const EXTENSION_BASE: u16 = 0x1000;
}

/// Reservierter Trait für optionale spätere Blockkompression (SD-13).
pub trait BlockCodec: Send + Sync {
    /// Eindeutige Kennung des Codecs.
    fn id(&self) -> u8;
    /// Komprimiert einen Eingabepuffer.
    fn compress(&self, src: &[u8], dst: &mut Vec<u8>) -> io::Result<()>;
    /// Dekomprimiert einen Eingabepuffer.
    fn decompress(&self, src: &[u8], dst: &mut Vec<u8>) -> io::Result<()>;
}

/// Integritäts- und Verschlüsselungsmodus für Blöcke.
pub enum BlockSecurity<'a> {
    /// Keine Kryptografie, nur CRC32C.
    Plain,
    /// Authentisiert mit Space-DEK und 16-Byte-MAC.
    Authenticated {
        /// Space-Schlüssel.
        key: &'a [u8; 32],
        /// MAC-Port.
        mac: &'a dyn Mac,
    },
    /// Versiegelt mit Space-DEK über AEAD (Cipher).
    Encrypted {
        /// Space-Schlüssel.
        key: &'a [u8; 32],
        /// Cipher-Port.
        cipher: &'a dyn Cipher,
    },
}

/// Erzeugt die 21-Byte AAD nach SPEC §9.5:
/// `0x01 (Run) || run_id u64 || block_offset u64 || block_kind u16 || space u16`
pub fn make_block_aad(run_id: u64, block_offset: u64, kind: u16, space: u16) -> [u8; 21] {
    let mut aad = [0u8; 21];
    if let Some(b) = aad.get_mut(0) {
        *b = FILE_KIND_RUN;
    }
    let r_bytes = run_id.to_le_bytes();
    let o_bytes = block_offset.to_le_bytes();
    let k_bytes = kind.to_le_bytes();
    let s_bytes = space.to_le_bytes();

    if let Some(dst) = aad.get_mut(1..9) {
        dst.copy_from_slice(&r_bytes);
    }
    if let Some(dst) = aad.get_mut(9..17) {
        dst.copy_from_slice(&o_bytes);
    }
    if let Some(dst) = aad.get_mut(17..19) {
        dst.copy_from_slice(&k_bytes);
    }
    if let Some(dst) = aad.get_mut(19..21) {
        dst.copy_from_slice(&s_bytes);
    }
    aad
}

/// Kodiert einen Block inklusive Codec-Byte und Prüfsumme / Tag.
pub fn encode_block(
    payload: &[u8],
    kind: u16,
    run_id: u64,
    block_offset: u64,
    space: u16,
    security: &BlockSecurity<'_>,
) -> Vec<u8> {
    match security {
        BlockSecurity::Plain => {
            let cap = payload.len().saturating_add(5);
            let mut out = Vec::with_capacity(cap);
            out.extend_from_slice(payload);
            out.push(CODEC_NONE);
            let chk = crc32c(0, &out);
            out.extend_from_slice(&chk.to_le_bytes());
            out
        }
        BlockSecurity::Authenticated { key, mac } => {
            let cap = payload.len().saturating_add(17);
            let mut out = Vec::with_capacity(cap);
            out.extend_from_slice(payload);
            out.push(CODEC_NONE);
            let aad = make_block_aad(run_id, block_offset, kind, space);
            let tag = mac.mac16(key, &[&out, &aad]);
            out.extend_from_slice(&tag);
            out
        }
        BlockSecurity::Encrypted { key, cipher } => {
            let nonce = run_block_nonce(block_offset);
            let aad = make_block_aad(run_id, block_offset, kind, space);
            let mut plain = Vec::with_capacity(payload.len().saturating_add(1));
            plain.extend_from_slice(payload);
            plain.push(CODEC_NONE);
            cipher.seal(key, &nonce, &aad, &plain)
        }
    }
}

/// Dekodiert und verifiziert einen Block. Liefert die dekomprimierte Nutzlast.
pub fn decode_block(
    raw: &[u8],
    kind: u16,
    run_id: u64,
    block_offset: u64,
    space: u16,
    security: &BlockSecurity<'_>,
) -> Result<Vec<u8>, RunError> {
    match security {
        BlockSecurity::Plain => {
            // raw: stored_payload | codec u8 | crc32c u32 (min. 5 Bytes)
            if raw.len() < 5 {
                return Err(RunError::Corrupt("Plain-Block zu kurz (< 5 B)".into()));
            }
            let payload_codec_len = raw.len().saturating_sub(4);
            let payload_codec = raw
                .get(0..payload_codec_len)
                .ok_or_else(|| RunError::Corrupt("Puffergrenzen ungültig".into()))?;
            let mut chk_bytes = [0u8; 4];
            if let Some(s) = raw.get(payload_codec_len..) {
                if s.len() == 4 {
                    chk_bytes.copy_from_slice(s);
                }
            }
            let stored_crc = u32::from_le_bytes(chk_bytes);
            let expected_crc = crc32c(0, payload_codec);
            if stored_crc != expected_crc {
                return Err(RunError::Corrupt(format!(
                    "Block-CRC32C ungültig: erwartet {expected_crc:#010x}, gefunden {stored_crc:#010x}"
                )));
            }

            let codec = *payload_codec
                .get(payload_codec_len.saturating_sub(1))
                .ok_or_else(|| RunError::Corrupt("Codec-Byte fehlt".into()))?;
            if codec != CODEC_NONE {
                return Err(RunError::Unsupported(format!(
                    "Block-Codec {codec} nicht unterstützt"
                )));
            }

            let payload = payload_codec
                .get(0..payload_codec_len.saturating_sub(1))
                .ok_or_else(|| RunError::Corrupt("Payload fehlt".into()))?;
            Ok(payload.to_vec())
        }
        BlockSecurity::Authenticated { key, mac } => {
            if raw.len() < 17 {
                return Err(RunError::Corrupt("Authenticated-Block zu kurz (< 17 B)".into()));
            }
            let payload_codec_len = raw.len().saturating_sub(16);
            let payload_codec = raw
                .get(0..payload_codec_len)
                .ok_or_else(|| RunError::Corrupt("Puffergrenzen ungültig".into()))?;
            let mut tag = [0u8; 16];
            if let Some(s) = raw.get(payload_codec_len..) {
                if s.len() == 16 {
                    tag.copy_from_slice(s);
                }
            }
            let aad = make_block_aad(run_id, block_offset, kind, space);
            if !mac.verify16(key, &[payload_codec, &aad], &tag) {
                return Err(RunError::Corrupt("Block-MAC-Prüfung fehlgeschlagen".into()));
            }

            let codec = *payload_codec
                .get(payload_codec_len.saturating_sub(1))
                .ok_or_else(|| RunError::Corrupt("Codec-Byte fehlt".into()))?;
            if codec != CODEC_NONE {
                return Err(RunError::Unsupported(format!(
                    "Block-Codec {codec} nicht unterstützt"
                )));
            }

            let payload = payload_codec
                .get(0..payload_codec_len.saturating_sub(1))
                .ok_or_else(|| RunError::Corrupt("Payload fehlt".into()))?;
            Ok(payload.to_vec())
        }
        BlockSecurity::Encrypted { key, cipher } => {
            let nonce = run_block_nonce(block_offset);
            let aad = make_block_aad(run_id, block_offset, kind, space);
            let opened = cipher
                .open(key, &nonce, &aad, raw)
                .map_err(|e| RunError::Corrupt(format!("Cipher open fehlgeschlagen: {e}")))?;

            if opened.is_empty() {
                return Err(RunError::Corrupt("Versiegelter Block ist leer".into()));
            }
            let codec_idx = opened.len().saturating_sub(1);
            let codec = *opened
                .get(codec_idx)
                .ok_or_else(|| RunError::Corrupt("Codec-Byte fehlt".into()))?;
            if codec != CODEC_NONE {
                return Err(RunError::Unsupported(format!(
                    "Block-Codec {codec} nicht unterstützt"
                )));
            }

            let payload = opened
                .get(0..codec_idx)
                .ok_or_else(|| RunError::Corrupt("Payload fehlt".into()))?;
            Ok(payload.to_vec())
        }
    }
}

#[cfg(test)]
#[allow(clippy::indexing_slicing)]
mod tests {
    use super::*;

    #[test]
    fn test_plain_block_roundtrip() {
        let payload = b"hello world data block";
        let encoded = encode_block(
            payload,
            block_kind::DATA,
            1,
            0,
            1,
            &BlockSecurity::Plain,
        );
        assert_eq!(encoded.len(), payload.len() + 1 + 4);

        let decoded = decode_block(
            &encoded,
            block_kind::DATA,
            1,
            0,
            1,
            &BlockSecurity::Plain,
        )
        .unwrap_or_default();
        assert_eq!(decoded, payload);
    }

    #[test]
    fn test_plain_block_corruption() {
        let payload = b"test payload";
        let mut encoded = encode_block(
            payload,
            block_kind::DATA,
            1,
            0,
            1,
            &BlockSecurity::Plain,
        );
        if let Some(b) = encoded.get_mut(2) {
            *b ^= 0x55;
        }
        assert!(decode_block(
            &encoded,
            block_kind::DATA,
            1,
            0,
            1,
            &BlockSecurity::Plain
        )
        .is_err());
    }

    #[test]
    fn test_aad_structure() {
        let aad = make_block_aad(0x1234, 4096, block_kind::INDEX, 42);
        assert_eq!(aad.len(), 21);
        assert_eq!(aad[0], 0x01);
        assert_eq!(&aad[1..9], &0x1234u64.to_le_bytes());
        assert_eq!(&aad[9..17], &4096u64.to_le_bytes());
        assert_eq!(&aad[17..19], &1u16.to_le_bytes());
        assert_eq!(&aad[19..21], &42u16.to_le_bytes());
    }
}
