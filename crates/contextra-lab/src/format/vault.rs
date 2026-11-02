//! F.3 Tresor Parser (`vault/vault.dat`).
//!
//! SPEC Anhang F.3:
//! - Dateikopf nach F.1 (kind = 10) -> 64 Bytes
//! - Metablock (64 B):
//!   capacity(u64) || live(u64) || tombstones(u64) || generation(u64) || zero(32)
//! - Slots à 80 Byte, Offset = 128 + 64 + i * 80:
//!   - 0..1: slot_type (0x01 T, 0x02 C, 0x03 S, 0x04 D, 0x05 F, 0x00 leer)
//!   - 1..17: tag (16 B)
//!   - 17..49: payload (32 B)
//!   - 49..50: status (0x00 leer, 0x01 aktiv, 0x02 Grabstein)
//!   - 50..56: gen (u48 big-endian, Slot-Generation)
//!   - 56..80: mac (24 B) = HMAC-SHA256(K_vm, Bytes[0..56))[0..24]
//! - Grabstein / Vernichten: alle 80 Byte auf 0, Byte 49 = 0x02. MAC = 0.
//! - Verifikation Grabstein: Bytes 0..49 und 50..80 sind 0.

use hmac::Hmac;
use sha2::digest::Mac;
use sha2::Sha256;

use crate::error::LabError;
use crate::format::file_header::FileHeader;

/// Fester Offset für den ersten Slot nach SPEC F.3 / L-4:
/// 128 + 64 = 192.
pub const VAULT_SLOT_BASE: usize = 192;

/// Größe eines Tresor-Slots.
pub const VAULT_SLOT_SIZE: usize = 80;

/// Slot-Typen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum VaultSlotType {
    /// 0x00: Unbelegt
    Empty = 0x00,
    /// 0x01: Mandant (Tenant)
    Tenant = 0x01,
    /// 0x02: Collection
    Collection = 0x02,
    /// 0x03: Betroffener (Subject)
    Subject = 0x03,
    /// 0x04: Dokument (Document)
    Document = 0x04,
    /// 0x05: Dateischlüssel (FileKey)
    FileKey = 0x05,
}

impl VaultSlotType {
    /// Parst den Slot-Typen.
    pub fn from_u8(val: u8) -> Result<Self, LabError> {
        match val {
            0x00 => Ok(Self::Empty),
            0x01 => Ok(Self::Tenant),
            0x02 => Ok(Self::Collection),
            0x03 => Ok(Self::Subject),
            0x04 => Ok(Self::Document),
            0x05 => Ok(Self::FileKey),
            _other => Err(LabError::Corrupted("Unbekannter VaultSlotType")),
        }
    }
}

/// Slot-Status.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum VaultSlotStatus {
    /// 0x00: Leer
    Empty = 0x00,
    /// 0x01: Aktiv
    Active = 0x01,
    /// 0x02: Grabstein (Tombstone)
    Tombstone = 0x02,
}

impl VaultSlotStatus {
    /// Parst das Statusbyte.
    pub fn from_u8(val: u8) -> Result<Self, LabError> {
        match val {
            0x00 => Ok(Self::Empty),
            0x01 => Ok(Self::Active),
            0x02 => Ok(Self::Tombstone),
            _ => Err(LabError::Corrupted("Unbekannter VaultSlotStatus")),
        }
    }
}

/// Strukturierter Tresor-Slot (80 Byte).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VaultSlot {
    /// Typ des Slots.
    pub slot_type: VaultSlotType,
    /// Tag (16 Byte).
    pub tag: [u8; 16],
    /// Nutzlast (32 Byte Salt oder maskierter Dateischlüssel).
    pub payload: [u8; 32],
    /// Status (leer, aktiv, Grabstein).
    pub status: VaultSlotStatus,
    /// Slot-Generation (48 Bit).
    pub gen: u64,
    /// Authentisierungs-MAC (24 Byte).
    pub mac: [u8; 24],
}

impl VaultSlot {
    /// Parst einen 80-Byte-Slot.
    pub fn parse(bytes: &[u8]) -> Result<Self, LabError> {
        if bytes.len() < VAULT_SLOT_SIZE {
            return Err(LabError::UnexpectedEof {
                expected: VAULT_SLOT_SIZE,
                actual: bytes.len(),
            });
        }

        let st_byte = *bytes.first().ok_or(LabError::Corrupted("SlotType fehlt"))?;
        let slot_type = VaultSlotType::from_u8(st_byte)?;

        let tag: [u8; 16] = bytes
            .get(1..17)
            .and_then(|s| s.try_into().ok())
            .ok_or(LabError::Corrupted("Tag fehlt"))?;

        let payload: [u8; 32] = bytes
            .get(17..49)
            .and_then(|s| s.try_into().ok())
            .ok_or(LabError::Corrupted("Payload fehlt"))?;

        let status_byte = *bytes.get(49).ok_or(LabError::Corrupted("Status fehlt"))?;
        let status = VaultSlotStatus::from_u8(status_byte)?;

        let gen_bytes = bytes.get(50..56).ok_or(LabError::Corrupted("Gen fehlt"))?;
        let mut full_gen = [0u8; 8];
        if let Some(dest) = full_gen.get_mut(2..8) {
            dest.copy_from_slice(gen_bytes);
        }
        let gen = u64::from_be_bytes(full_gen);

        let mac: [u8; 24] = bytes
            .get(56..80)
            .and_then(|s| s.try_into().ok())
            .ok_or(LabError::Corrupted("Mac fehlt"))?;

        if status == VaultSlotStatus::Tombstone {
            let p1 = bytes.get(0..49).ok_or(LabError::Corrupted("P1 fehlt"))?;
            let p2 = bytes.get(50..80).ok_or(LabError::Corrupted("P2 fehlt"))?;
            if p1.iter().any(|&b| b != 0) || p2.iter().any(|&b| b != 0) {
                return Err(LabError::VerificationFailed(
                    "Ungültiger Grabstein: Nicht-Status-Bytes sind nicht 0",
                ));
            }
        }

        Ok(Self {
            slot_type,
            tag,
            payload,
            status,
            gen,
            mac,
        })
    }

    /// Verifiziert den 24-Byte-MAC eines aktiven Slots.
    /// K_vm = HKDF-Expand(K_enc, "ctx/vault-mac/v1", 32)
    /// mac = HMAC-SHA256(K_vm, Bytes[0..56))[0..24]
    pub fn verify_mac(&self, raw_slot_80: &[u8], k_vm: &[u8; 32]) -> Result<(), LabError> {
        if self.status == VaultSlotStatus::Empty {
            return Ok(());
        }
        if self.status == VaultSlotStatus::Tombstone {
            if self.mac.iter().any(|&b| b != 0) {
                return Err(LabError::VerificationFailed("Grabstein mit Nicht-Null-MAC"));
            }
            return Ok(());
        }

        if raw_slot_80.len() < VAULT_SLOT_SIZE {
            return Err(LabError::UnexpectedEof {
                expected: VAULT_SLOT_SIZE,
                actual: raw_slot_80.len(),
            });
        }

        type HmacSha256 = Hmac<Sha256>;
        let mut mac = <HmacSha256 as hmac::digest::KeyInit>::new_from_slice(k_vm)
            .map_err(|_| LabError::Corrupted("HMAC Initialisierung fehlgeschlagen"))?;

        let pre = raw_slot_80
            .get(0..56)
            .ok_or(LabError::Corrupted("Slot Pre-Daten ungültig"))?;
        mac.update(pre);

        let full_tag = mac.finalize().into_bytes();
        let expected_24 = full_tag
            .get(0..24)
            .ok_or(LabError::Corrupted("Tag Slice fehlt"))?;

        if self.mac != expected_24 {
            return Err(LabError::VerificationFailed("Vault-Slot MAC ungültig"));
        }

        Ok(())
    }
}

/// Tresor Metablock (64 Byte).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VaultMetaBlock {
    /// Slot-Kapazität (Zweierpotenz).
    pub capacity: u64,
    /// Anzahl aktiver Slots.
    pub live: u64,
    /// Anzahl Grabsteine.
    pub tombstones: u64,
    /// Generationsnummer.
    pub generation: u64,
}

impl VaultMetaBlock {
    /// Parst die 64 Byte des Metablocks.
    pub fn parse(bytes: &[u8]) -> Result<Self, LabError> {
        if bytes.len() < 64 {
            return Err(LabError::UnexpectedEof {
                expected: 64,
                actual: bytes.len(),
            });
        }

        let capacity = u64::from_be_bytes(
            bytes
                .get(0..8)
                .and_then(|s| s.try_into().ok())
                .ok_or(LabError::Corrupted("Capacity fehlt"))?,
        );
        if !capacity.is_power_of_two() || capacity == 0 {
            return Err(LabError::Corrupted("Capacity ist keine Zweierpotenz"));
        }

        let live = u64::from_be_bytes(
            bytes
                .get(8..16)
                .and_then(|s| s.try_into().ok())
                .ok_or(LabError::Corrupted("Live fehlt"))?,
        );

        let tombstones = u64::from_be_bytes(
            bytes
                .get(16..24)
                .and_then(|s| s.try_into().ok())
                .ok_or(LabError::Corrupted("Tombstones fehlt"))?,
        );

        let generation = u64::from_be_bytes(
            bytes
                .get(24..32)
                .and_then(|s| s.try_into().ok())
                .ok_or(LabError::Corrupted("Generation fehlt"))?,
        );

        let zero = bytes
            .get(32..64)
            .ok_or(LabError::Corrupted("Zero Padding fehlt"))?;
        if zero.iter().any(|&b| b != 0) {
            return Err(LabError::Corrupted("Metablock Zero Padding ist nicht 0"));
        }

        Ok(Self {
            capacity,
            live,
            tombstones,
            generation,
        })
    }
}

/// Strukturierte Gesamtrepräsentation einer Tresordatei (`vault.dat`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VaultFile {
    /// F.1 Dateikopf.
    pub header: FileHeader,
    /// Metablock.
    pub meta: VaultMetaBlock,
    /// Slots.
    pub slots: Vec<VaultSlot>,
}

impl VaultFile {
    /// Parst eine Tresordatei.
    pub fn parse(bytes: &[u8]) -> Result<Self, LabError> {
        let header = FileHeader::parse(bytes)?;
        if header.kind != crate::format::file_header::FileKind::Vault {
            return Err(LabError::UnsupportedKind(header.kind as u8));
        }

        let meta_bytes = bytes
            .get(64..128)
            .ok_or(LabError::Corrupted("Metablock fehlt"))?;
        let meta = VaultMetaBlock::parse(meta_bytes)?;

        let cap_usize =
            usize::try_from(meta.capacity).map_err(|_| LabError::Corrupted("Capacity Überlauf"))?;
        let mut slots = Vec::with_capacity(cap_usize);
        for i in 0..meta.capacity {
            let offset = VAULT_SLOT_BASE
                + usize::try_from(i).map_err(|_| LabError::Corrupted("Index Überlauf"))?
                    * VAULT_SLOT_SIZE;
            let end = offset + VAULT_SLOT_SIZE;
            let slot_bytes = bytes.get(offset..end).ok_or(LabError::UnexpectedEof {
                expected: end,
                actual: bytes.len(),
            })?;
            slots.push(VaultSlot::parse(slot_bytes)?);
        }

        Ok(Self {
            header,
            meta,
            slots,
        })
    }
}
