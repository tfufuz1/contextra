//! # Redo-Frame und Siegelung
//!
//! SPEC §9.4 und Golden A.5:
//! `Frame: len u32 | stamp u64 | op_count u32 | flags u8 | payload | chk`
//! `len = 13 + |payload| (zählt stamp … payload, nicht len selbst, nicht chk)`
//! `len == 0` beendet das Segment (vorallokierter Nullbereich)

use contextra_storage_ports::{Cipher, Mac};
use crate::codec::crc32c::crc32c;
use crate::codec::limits::MAX_CHANGE_BYTES;
use crate::keys::nonce::redo_frame_nonce;
use crate::redo::error::RedoError;
use crate::redo::op::RedoOp;

/// Durability-Stufe eines Frames nach SPEC §9.4.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Durability {
    /// Dauerhaft synchronisiert per `sync_data`.
    Always = 0,
    /// Intervall-Synchronisation.
    Interval = 1,
    /// Keine synchrone Absicherung.
    Never = 2,
}

impl Durability {
    /// Wandelt u8 in Durability um.
    pub fn from_u8(v: u8) -> Option<Self> {
        match v {
            0 => Some(Self::Always),
            1 => Some(Self::Interval),
            2 => Some(Self::Never),
            _ => None,
        }
    }

    /// Liefert den numerischen u8-Wert.
    pub fn to_u8(self) -> u8 {
        match self {
            Self::Always => 0,
            Self::Interval => 1,
            Self::Never => 2,
        }
    }
}

/// Integritätsmodus für Redo-Frames.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RedoIntegrityMode {
    /// Prüfsummenkette per CRC32C.
    Plain = 0,
    /// Authentifiziert per MAC16.
    Authenticated = 1,
    /// Verschlüsselt per AEAD (ChaCha20Poly1305).
    Encrypted = 2,
}

/// Bitmaske für Durability in Frame-Flags.
pub const FLAG_DURABILITY_MASK: u8 = 0x03;
/// Bit 7 in Frame-Flags: Seal-Frame.
pub const FLAG_SEAL: u8 = 0x80;
/// Reservierte `op_count`-Kennung für Seal-Frames (0xFFFF_FFFF).
pub const SEAL_OP_COUNT: u32 = 0xFFFF_FFFF;

/// Redo-Frame im Speicher nach SPEC §9.4.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RedoFrame {
    /// Gesamtlänge von stamp … payload (13 + |payload|).
    pub len: u32,
    /// Commit-Stamp dieses Frames.
    pub stamp: u64,
    /// Anzahl enthaltener Operationen (bzw. 0xFFFF_FFFF bei Seal).
    pub op_count: u32,
    /// Frame-Flags (Durability und SEAL-Bit).
    pub flags: u8,
    /// Nutzdaten (kodierte Operationen bzw. Seal-Inhalt).
    pub payload: Vec<u8>,
    /// Prüfwert (4 Byte CRC32C für Plain, 16 Byte MAC für Authenticated, leer für Encrypted).
    pub chk: Vec<u8>,
}

impl RedoFrame {
    /// Prüft, ob dieser Frame ein Seal-Frame ist (INV-LOG-6).
    pub fn is_seal(&self) -> bool {
        self.op_count == SEAL_OP_COUNT && (self.flags & FLAG_SEAL) != 0
    }

    /// Liefert die Durability-Stufe.
    pub fn durability(&self) -> Durability {
        Durability::from_u8(self.flags & FLAG_DURABILITY_MASK).unwrap_or(Durability::Always)
    }

    /// Liest (frame_count, final_chain) aus einem Seal-Frame.
    pub fn parse_seal_payload(&self) -> Result<(u64, u32), RedoError> {
        if !self.is_seal() {
            return Err(RedoError::Corrupt("Frame ist kein Seal-Frame".into()));
        }
        if self.payload.len() < 12 {
            return Err(RedoError::Corrupt("Seal-Payload zu kurz".into()));
        }
        let fc_bytes: [u8; 8] = match self.payload.get(0..8).and_then(|s| s.try_into().ok()) {
            Some(b) => b,
            None => return Err(RedoError::Corrupt("Fehlende frame_count".into())),
        };
        let final_chain_bytes: [u8; 4] = match self.payload.get(8..12).and_then(|s| s.try_into().ok()) {
            Some(b) => b,
            None => return Err(RedoError::Corrupt("Fehlende final_chain".into())),
        };
        Ok((
            u64::from_le_bytes(fc_bytes),
            u32::from_le_bytes(final_chain_bytes),
        ))
    }

    /// Erstellt und kodiert einen regulären Frame im Plain-Modus (Golden A.5).
    pub fn encode_plain(
        stamp: u64,
        durability: Durability,
        ops: &[RedoOp],
        prev_chk: u32,
    ) -> Result<(Self, u32), RedoError> {
        let mut payload = Vec::new();
        for op in ops {
            op.encode(&mut payload);
        }
        let op_count = match u32::try_from(ops.len()) {
            Ok(v) => v,
            Err(_) => return Err(RedoError::Unsupported("Zu viele Operationen im Frame".into())),
        };

        let len_u64 = 13u64.checked_add(match u64::try_from(payload.len()) {
            Ok(v) => v,
            Err(_) => return Err(RedoError::Unsupported("Payload zu groß".into())),
        }).ok_or_else(|| RedoError::Unsupported("Frame-Länge Overflow".into()))?;

        if len_u64 > match u64::try_from(MAX_CHANGE_BYTES) {
            Ok(v) => v,
            Err(_) => u64::MAX,
        } {
            return Err(RedoError::Unsupported("Frame überschreitet 32 MiB".into()));
        }
        let len = u32::try_from(len_u64).map_err(|_| RedoError::Unsupported("Länge übersteigt u32".into()))?;

        let flags = durability.to_u8();

        let mut data = Vec::with_capacity(usize::try_from(len).unwrap_or(0).saturating_add(4));
        data.extend_from_slice(&len.to_le_bytes());
        data.extend_from_slice(&stamp.to_le_bytes());
        data.extend_from_slice(&op_count.to_le_bytes());
        data.push(flags);
        data.extend_from_slice(&payload);

        let chk_val = crc32c(prev_chk, &data);
        let chk = chk_val.to_le_bytes().to_vec();

        Ok((
            Self {
                len,
                stamp,
                op_count,
                flags,
                payload,
                chk,
            },
            chk_val,
        ))
    }

    /// Erstellt einen Seal-Frame im Plain-Modus nach SPEC §9.4 und Golden A.5.
    pub fn new_seal_plain(
        stamp: u64,
        frame_count: u64,
        final_chain: u32,
        prev_chk: u32,
    ) -> (Self, u32) {
        let mut payload = Vec::with_capacity(12);
        payload.extend_from_slice(&frame_count.to_le_bytes());
        payload.extend_from_slice(&final_chain.to_le_bytes());

        let len: u32 = 13 + 12; // 25
        let op_count = SEAL_OP_COUNT;
        let flags = FLAG_SEAL;

        let mut data = Vec::with_capacity(25 + 4);
        data.extend_from_slice(&len.to_le_bytes());
        data.extend_from_slice(&stamp.to_le_bytes());
        data.extend_from_slice(&op_count.to_le_bytes());
        data.push(flags);
        data.extend_from_slice(&payload);

        let seal_chk_val = crc32c(prev_chk, &data);
        let chk = seal_chk_val.to_le_bytes().to_vec();

        (
            Self {
                len,
                stamp,
                op_count,
                flags,
                payload,
                chk,
            },
            seal_chk_val,
        )
    }

    /// Erstellt und kodiert einen Frame im Authenticated-Modus.
    pub fn encode_authenticated(
        stamp: u64,
        durability: Durability,
        ops: &[RedoOp],
        segment_id: u64,
        frame_index: u32,
        mac_key: &[u8; 32],
        mac: &dyn Mac,
    ) -> Result<Self, RedoError> {
        let mut payload = Vec::new();
        for op in ops {
            op.encode(&mut payload);
        }
        let op_count = u32::try_from(ops.len()).map_err(|_| RedoError::Unsupported("Zu viele Ops".into()))?;
        let len_u64 = 13u64.checked_add(u64::try_from(payload.len()).map_err(|_| RedoError::Unsupported("Payload zu groß".into()))?)
            .ok_or_else(|| RedoError::Unsupported("Längenoverflow".into()))?;
        let len = u32::try_from(len_u64).map_err(|_| RedoError::Unsupported("Länge übersteigt u32".into()))?;
        let flags = durability.to_u8();

        let len_bytes = len.to_le_bytes();
        let stamp_bytes = stamp.to_le_bytes();
        let op_count_bytes = op_count.to_le_bytes();
        let flags_bytes = [flags];
        let seg_bytes = segment_id.to_le_bytes();
        let idx_bytes = frame_index.to_le_bytes();

        let parts: [&[u8]; 7] = [
            &len_bytes,
            &stamp_bytes,
            &op_count_bytes,
            &flags_bytes,
            &payload,
            &seg_bytes,
            &idx_bytes,
        ];
        let tag = mac.mac16(mac_key, &parts);

        Ok(Self {
            len,
            stamp,
            op_count,
            flags,
            payload,
            chk: tag.to_vec(),
        })
    }

    /// Erstellt und kodiert einen Frame im Encrypted-Modus.
    pub fn encode_encrypted(
        stamp: u64,
        durability: Durability,
        ops: &[RedoOp],
        segment_id: u64,
        frame_index: u32,
        redo_key: &[u8; 32],
        cipher: &dyn Cipher,
    ) -> Result<Self, RedoError> {
        let mut plain_payload = Vec::new();
        for op in ops {
            op.encode(&mut plain_payload);
        }
        let op_count = u32::try_from(ops.len()).map_err(|_| RedoError::Unsupported("Zu viele Ops".into()))?;
        let flags = durability.to_u8();

        // Vorläufige Schätzung für AAD: Ciphertext hat 16 Bytes Tag
        let enc_len_u64 = 13u64.checked_add(u64::try_from(plain_payload.len()).map_err(|_| RedoError::Unsupported("Payload groß".into()))?)
            .and_then(|v| v.checked_add(16))
            .ok_or_else(|| RedoError::Unsupported("Overflow".into()))?;
        let len = u32::try_from(enc_len_u64).map_err(|_| RedoError::Unsupported("u32 Overflow".into()))?;

        let mut aad = Vec::with_capacity(4 + 8 + 4 + 1 + 8 + 4);
        aad.extend_from_slice(&len.to_le_bytes());
        aad.extend_from_slice(&stamp.to_le_bytes());
        aad.extend_from_slice(&op_count.to_le_bytes());
        aad.push(flags);
        aad.extend_from_slice(&segment_id.to_le_bytes());
        aad.extend_from_slice(&frame_index.to_le_bytes());

        let nonce = redo_frame_nonce(frame_index);
        let sealed_payload = cipher.seal(redo_key, &nonce, &aad, &plain_payload);

        Ok(Self {
            len,
            stamp,
            op_count,
            flags,
            payload: sealed_payload,
            chk: Vec::new(),
        })
    }

    /// Serialisiert den Frame vollständig in Bytes.
    pub fn to_bytes(&self) -> Vec<u8> {
        let total = 4 + usize::try_from(self.len).unwrap_or(0) + self.chk.len();
        let mut out = Vec::with_capacity(total);
        out.extend_from_slice(&self.len.to_le_bytes());
        out.extend_from_slice(&self.stamp.to_le_bytes());
        out.extend_from_slice(&self.op_count.to_le_bytes());
        out.push(self.flags);
        out.extend_from_slice(&self.payload);
        out.extend_from_slice(&self.chk);
        out
    }
}
