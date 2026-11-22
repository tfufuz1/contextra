//! # Redo-Operationen
//!
//! SPEC §9.4: Operation (Inhalt von `payload`, `op_count` Stück hintereinander):
//! `kind u8 | space u16 | opflags u8 | [domain u32 wenn opflags.DOMAIN] | klen varint | key | vlen varint | value`

use contextra_storage_ports::Cipher;
use crate::codec::varint::{decode_u64, encode_u64};
use crate::keys::nonce::redo_op_body_nonce;
use crate::redo::error::RedoError;

/// Bit 0 in `opflags`: Domäne ist angegeben (`domain u32` folgt).
pub const OPFLAG_DOMAIN: u8 = 0x01;
/// Bit 1 in `opflags`: Körper ist mit dem Domänenschlüssel versiegelt.
pub const OPFLAG_SEALED: u8 = 0x02;
/// Bit 2 in `opflags`: Exklusives Ende ist unbegrenzt (kein Endschlüssel).
pub const OPFLAG_UNBOUNDED_END: u8 = 0x04;

/// Art einer Operation im Redo-Log.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RedoOpKind {
    /// Eintrag setzen (kind = 1).
    Put = 1,
    /// Eintrag löschen (kind = 2).
    Delete = 2,
    /// Bereich löschen (kind = 3).
    RangeDelete = 3,
    /// Physische Bereichslöschung (kind = 4).
    EraseRange = 4,
}

impl RedoOpKind {
    /// Wandelt u8 in `RedoOpKind` um.
    pub fn from_u8(v: u8) -> Result<Self, RedoError> {
        match v {
            1 => Ok(Self::Put),
            2 => Ok(Self::Delete),
            3 => Ok(Self::RangeDelete),
            4 => Ok(Self::EraseRange),
            other => Err(RedoError::Corrupt(format!("Unbekannte RedoOp-Art: {other}"))),
        }
    }

    /// Liefert den numerischen u8-Wert.
    pub fn to_u8(self) -> u8 {
        match self {
            Self::Put => 1,
            Self::Delete => 2,
            Self::RangeDelete => 3,
            Self::EraseRange => 4,
        }
    }
}

/// Einzelne Operation im Redo-Log nach SPEC §9.4.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RedoOp {
    /// Operationsart.
    pub kind: RedoOpKind,
    /// Space-ID.
    pub space: u16,
    /// Operations-Flags (`OPFLAG_DOMAIN`, `OPFLAG_SEALED`, `OPFLAG_UNBOUNDED_END`).
    pub opflags: u8,
    /// Optionale Domänen-ID (wenn `OPFLAG_DOMAIN` gesetzt).
    pub domain: Option<u32>,
    /// Nutzerschlüssel bzw. Bereichsstart.
    pub key: Vec<u8>,
    /// Wert bzw. Bereichsende.
    pub value: Vec<u8>,
}

impl RedoOp {
    /// Erstellt eine normale unversiegelte Put-Operation.
    pub fn new_put(space: u16, domain: Option<u32>, key: Vec<u8>, value: Vec<u8>) -> Self {
        let mut opflags = 0u8;
        if domain.is_some() {
            opflags |= OPFLAG_DOMAIN;
        }
        Self {
            kind: RedoOpKind::Put,
            space,
            opflags,
            domain,
            key,
            value,
        }
    }

    /// Erstellt eine domänenversiegelte Put-Operation nach SPEC §9.4.
    pub fn new_sealed_put(
        space: u16,
        domain: u32,
        key: &[u8],
        header: &[u8],
        body: &[u8],
        segment_id: u64,
        frame_index: u32,
        op_index: u32,
        domain_key: &[u8; 32],
        cipher: &dyn Cipher,
    ) -> Result<Self, RedoError> {
        let mut aad = Vec::new();
        aad.extend_from_slice(&space.to_le_bytes());
        aad.extend_from_slice(&domain.to_le_bytes());
        aad.extend_from_slice(key);

        let seg_u32 = u32::from_le_bytes(
            match segment_id.to_le_bytes().get(..4).and_then(|s| s.try_into().ok()) {
                Some(b) => b,
                None => [0; 4],
            },
        );
        let nonce = redo_op_body_nonce(seg_u32, frame_index, op_index);
        let sealed_body = cipher.seal(domain_key, &nonce, &aad, body);

        let mut value = Vec::with_capacity(header.len().saturating_add(sealed_body.len()));
        value.extend_from_slice(header);
        value.extend_from_slice(&sealed_body);

        Ok(Self {
            kind: RedoOpKind::Put,
            space,
            opflags: OPFLAG_DOMAIN | OPFLAG_SEALED,
            domain: Some(domain),
            key: key.to_vec(),
            value,
        })
    }

    /// Entsiegelt den Körper einer domänenversiegelten Put-Operation.
    pub fn unseal_body(
        &self,
        header_len: usize,
        segment_id: u64,
        frame_index: u32,
        op_index: u32,
        domain_key: &[u8; 32],
        cipher: &dyn Cipher,
    ) -> Result<Vec<u8>, RedoError> {
        if (self.opflags & OPFLAG_SEALED) == 0 {
            let slice = self.value.get(header_len..).unwrap_or(&[]);
            return Ok(slice.to_vec());
        }
        let domain = match self.domain {
            Some(d) => d,
            None => return Err(RedoError::Corrupt("Sealed Op ohne Domäne".into())),
        };

        let mut aad = Vec::new();
        aad.extend_from_slice(&self.space.to_le_bytes());
        aad.extend_from_slice(&domain.to_le_bytes());
        aad.extend_from_slice(&self.key);

        let seg_u32 = u32::from_le_bytes(
            match segment_id.to_le_bytes().get(..4).and_then(|s| s.try_into().ok()) {
                Some(b) => b,
                None => [0; 4],
            },
        );
        let nonce = redo_op_body_nonce(seg_u32, frame_index, op_index);

        let sealed_body = match self.value.get(header_len..) {
            Some(s) => s,
            None => return Err(RedoError::Corrupt("Wert kürzer als Headerlänge".into())),
        };

        cipher
            .open(domain_key, &nonce, &aad, sealed_body)
            .map_err(|e| RedoError::CryptoError(e.to_string()))
    }

    /// Kodiert diese Operation in den Zielpuffer.
    pub fn encode(&self, dst: &mut Vec<u8>) {
        dst.push(self.kind.to_u8());
        dst.extend_from_slice(&self.space.to_le_bytes());
        dst.push(self.opflags);

        if (self.opflags & OPFLAG_DOMAIN) != 0 {
            let d = self.domain.unwrap_or(0);
            dst.extend_from_slice(&d.to_le_bytes());
        }

        let mut var_buf = [0u8; 10];
        let klen = encode_u64(match u64::try_from(self.key.len()) {
            Ok(v) => v,
            Err(_) => 0,
        }, &mut var_buf);
        if let Some(slice) = var_buf.get(..klen) {
            dst.extend_from_slice(slice);
        }
        dst.extend_from_slice(&self.key);

        let vlen = encode_u64(match u64::try_from(self.value.len()) {
            Ok(v) => v,
            Err(_) => 0,
        }, &mut var_buf);
        if let Some(slice) = var_buf.get(..vlen) {
            dst.extend_from_slice(slice);
        }
        dst.extend_from_slice(&self.value);
    }

    /// Dekodiert eine Operation aus einem Byte-Strom.
    pub fn decode(cursor: &mut &[u8]) -> Result<Self, RedoError> {
        let kind_byte = match cursor.first() {
            Some(&b) => b,
            None => return Err(RedoError::Corrupt("Unerwartetes Ende der Operation (kind)".into())),
        };
        *cursor = match cursor.get(1..) {
            Some(c) => c,
            None => &[],
        };
        let kind = RedoOpKind::from_u8(kind_byte)?;

        let space_bytes: [u8; 2] = match cursor.get(0..2).and_then(|s| s.try_into().ok()) {
            Some(b) => b,
            None => return Err(RedoError::Corrupt("Unerwartetes Ende der Operation (space)".into())),
        };
        *cursor = match cursor.get(2..) {
            Some(c) => c,
            None => &[],
        };
        let space = u16::from_le_bytes(space_bytes);

        let opflags = match cursor.first() {
            Some(&b) => b,
            None => return Err(RedoError::Corrupt("Unerwartetes Ende der Operation (flags)".into())),
        };
        *cursor = match cursor.get(1..) {
            Some(c) => c,
            None => &[],
        };

        let domain = if (opflags & OPFLAG_DOMAIN) != 0 {
            let dom_bytes: [u8; 4] = match cursor.get(0..4).and_then(|s| s.try_into().ok()) {
                Some(b) => b,
                None => return Err(RedoError::Corrupt("Unerwartetes Ende der Operation (domain)".into())),
            };
            *cursor = match cursor.get(4..) {
                Some(c) => c,
                None => &[],
            };
            Some(u32::from_le_bytes(dom_bytes))
        } else {
            None
        };

        let (klen_u64, klen_consumed) = match decode_u64(cursor) {
            Ok(pair) => pair,
            Err(_) => return Err(RedoError::Corrupt("Ungültige Schlüssellänge".into())),
        };
        *cursor = match cursor.get(klen_consumed..) {
            Some(c) => c,
            None => &[],
        };
        let klen = usize::try_from(klen_u64).map_err(|_| RedoError::Corrupt("Schlüssellänge übersteigt Adressraum".into()))?;

        if cursor.len() < klen {
            return Err(RedoError::Corrupt("Unerwartetes Ende beim Schlüssel".into()));
        }
        let key = match cursor.get(..klen) {
            Some(k) => k.to_vec(),
            None => Vec::new(),
        };
        *cursor = match cursor.get(klen..) {
            Some(c) => c,
            None => &[],
        };

        let (vlen_u64, vlen_consumed) = match decode_u64(cursor) {
            Ok(pair) => pair,
            Err(_) => return Err(RedoError::Corrupt("Ungültige Wertlänge".into())),
        };
        *cursor = match cursor.get(vlen_consumed..) {
            Some(c) => c,
            None => &[],
        };
        let vlen = usize::try_from(vlen_u64).map_err(|_| RedoError::Corrupt("Wertlänge übersteigt Adressraum".into()))?;

        if cursor.len() < vlen {
            return Err(RedoError::Corrupt("Unerwartetes Ende beim Wert".into()));
        }
        let value = match cursor.get(..vlen) {
            Some(v) => v.to_vec(),
            None => Vec::new(),
        };
        *cursor = match cursor.get(vlen..) {
            Some(c) => c,
            None => &[],
        };

        Ok(Self {
            kind,
            space,
            opflags,
            domain,
            key,
            value,
        })
    }
}
