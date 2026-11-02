//! Definition eines Vault-Slots nach SPEC F.3.
//!
//! Slot à 80 Byte:
//! `slot_type(1) ‖ tag(16) ‖ payload(32) ‖ status(1) ‖ gen(6) ‖ mac(24)`

use serde::{Deserialize, Serialize};
use zeroize::Zeroize;

/// Größe eines einzelnen Vault-Slots in Bytes (SPEC F.3, ER-02).
pub const VAULT_SLOT_SIZE: usize = 80;

/// Typ des Schlüssel-Slots in der Hierarchie (T, C, S, D, F).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum SlotType {
    /// Tenant-Schlüssel (T)
    Tenant = 0x01,
    /// Collection-Schlüssel (C)
    Collection = 0x02,
    /// Subject-Schlüssel (S)
    Subject = 0x03,
    /// Document-Schlüssel (D)
    Document = 0x04,
    /// File-Schlüssel (F)
    File = 0x05,
}

impl SlotType {
    /// Parst das Typ-Byte.
    pub const fn from_u8(b: u8) -> Option<Self> {
        match b {
            0x01 => Some(Self::Tenant),
            0x02 => Some(Self::Collection),
            0x03 => Some(Self::Subject),
            0x04 => Some(Self::Document),
            0x05 => Some(Self::File),
            _ => None,
        }
    }

    /// Konvertiert in das Typ-Byte.
    pub const fn as_u8(self) -> u8 {
        self as u8
    }
}

/// Status eines Slots (SPEC F.3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum SlotStatus {
    /// Unbenutzt / Leer
    Empty = 0x00,
    /// Aktiv belegt
    Active = 0x01,
    /// Grabstein (Tombstone nach Vernichtung, hält Sondierkette)
    Tombstone = 0x02,
}

impl SlotStatus {
    /// Parst das Status-Byte.
    pub const fn from_u8(b: u8) -> Option<Self> {
        match b {
            0x00 => Some(Self::Empty),
            0x01 => Some(Self::Active),
            0x02 => Some(Self::Tombstone),
            _ => None,
        }
    }
}

/// Ein einzelner Slot im Tresor (80 Bytes).
#[derive(Clone, PartialEq, Eq)]
pub struct VaultSlot {
    /// Slot-Typ (1 Byte)
    pub slot_type: SlotType,
    /// Tag (16 Bytes, z. B. BLAKE3(Tag-Quelle)[0..16])
    pub tag: [u8; 16],
    /// Schlüsselmaterial / Payload (32 Bytes)
    pub payload: [u8; 32],
    /// Status (1 Byte): Empty, Active, Tombstone
    pub status: SlotStatus,
    /// Generation (u48, 6 Bytes)
    pub generation: [u8; 6],
    /// Slot-MAC (24 Bytes)
    pub mac: [u8; 24],
}

impl VaultSlot {
    /// Erstellt einen leeren Slot.
    pub const fn empty() -> Self {
        Self {
            slot_type: SlotType::Document,
            tag: [0u8; 16],
            payload: [0u8; 32],
            status: SlotStatus::Empty,
            generation: [0u8; 6],
            mac: [0u8; 24],
        }
    }

    /// Serialisiert den Slot in einen 80-Byte-Puffer.
    pub fn to_bytes(&self) -> [u8; VAULT_SLOT_SIZE] {
        let mut buf = [0u8; VAULT_SLOT_SIZE];
        if self.status == SlotStatus::Tombstone {
            buf[49] = SlotStatus::Tombstone as u8;
            return buf;
        }
        buf[0] = self.slot_type.as_u8();
        buf[1..17].copy_from_slice(&self.tag);
        buf[17..49].copy_from_slice(&self.payload);
        buf[49] = self.status as u8;
        buf[50..56].copy_from_slice(&self.generation);
        buf[56..80].copy_from_slice(&self.mac);
        buf
    }

    /// Deserialisiert den Slot aus einem 80-Byte-Puffer.
    pub fn from_bytes(buf: &[u8; VAULT_SLOT_SIZE]) -> Option<Self> {
        let slot_type = SlotType::from_u8(buf[0])?;
        let status = SlotStatus::from_u8(buf[49])?;
        let mut tag = [0u8; 16];
        tag.copy_from_slice(&buf[1..17]);
        let mut payload = [0u8; 32];
        payload.copy_from_slice(&buf[17..49]);
        let mut generation = [0u8; 6];
        generation.copy_from_slice(&buf[50..56]);
        let mut mac = [0u8; 24];
        mac.copy_from_slice(&buf[56..80]);

        Some(Self {
            slot_type,
            tag,
            payload,
            status,
            generation,
            mac,
        })
    }

    /// Prüft, ob der Slot komplett genullt ist mit Grabstein auf Byte 49 (ER-05).
    pub fn is_destroyed_verifiably(&self) -> bool {
        let bytes = self.to_bytes();
        if bytes[..49].iter().any(|&b| b != 0) {
            return false;
        }
        if bytes[49] != 0x02 {
            return false;
        }
        if bytes[50..80].iter().any(|&b| b != 0) {
            return false;
        }
        true
    }
}

impl Drop for VaultSlot {
    fn drop(&mut self) {
        self.payload.zeroize();
    }
}
