//! F.3 Tresor Parser (`<vault_dir>/<nn>.vlt`, D-111, D-112, D-125).
//!
//! SPEC Anhang F.3:
//! - Dateikopf nach F.1 (kind = 10, key_mode = 1) -> 64 Bytes
//!   Ausnahme zu F.1: header_tag (Bytes 48..64) trägt HMAC-SHA256(K_vm, Bytes[0..48))[0..16]
//! - Metablock (Offset 64, 64 Byte):
//!   0..16: vault_id (16 B)
//!   16..17: shard_no (u8)
//!   17..18: cap_log2 (u8)
//!   18..26: live (u64 be, nur Hinweis)
//!   26..34: tombstones (u64 be, nur Hinweis)
//!   34..42: generation (u64 be, Rehash-Generation)
//!   42..50: rsv (u64 be, Cursor für vault_scrub)
//!   50..58: mac (8 B) = HMAC-SHA256(K_vm, Bytes[0..50))[0..8]
//!   58..64: pad (6 B) = 0
//! - Slots à 64 Byte, Offset = 128 + 64 * i:
//!   - 0..1: slot_type (0x01 T, 0x02 C, 0x03 S, 0x04 D, 0x05 F, 0x06 E, 0x00 leer)
//!   - 1..2: status (0x00 leer, 0x01 aktiv, 0x02 Grabstein)
//!   - 2..8: gen (u48 be, created_seq des Records)
//!   - 8..24: tag (16 B: tenant_tag | coll_tag | subject_tag | doc_tag | file_id | epoch_id)
//!   - 24..56: payload (32 B: Salt bzw. maskierter Schlüssel)
//!   - 56..64: mac (8 B) = HMAC-SHA256(K_vm, vault_id || Bytes[0..56))[0..8]
//! - Grabstein / Vernichten (ein Schreibvorgang):
//!   alle Bytes außer status auf 0, status = 0x02, mac = HMAC(K_vm, vault_id || Bytes[0..56))[0..8].
//!   Verifikation Grabstein: Bytes 0 und 2..56 sind 0, Byte 1 ist 0x02, MAC ist gültig.

use hmac::Hmac;
use sha2::digest::Mac;
use sha2::Sha256;

use crate::error::LabError;
use crate::format::file_header::FileHeader;

/// Fester Basis-Offset für den ersten Slot nach SPEC F.3: Offset 128.
pub const VAULT_SLOT_BASE: usize = 128;

/// Größe eines Tresor-Slots in Bytes (SPEC 5.1.0 F.3 / D-111).
pub const VAULT_SLOT_SIZE: usize = 64;

/// Größe des Tresor-Metablocks in Bytes.
pub const VAULT_METABLOCK_SIZE: usize = 64;

/// Slot-Typen laut SPEC F.3.
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
    /// 0x06: Epochenschlüssel (Epoch)
    Epoch = 0x06,
}

impl VaultSlotType {
    /// Parst den Slot-Typen aus einem Byte.
    pub fn from_u8(val: u8) -> Result<Self, LabError> {
        match val {
            0x00 => Ok(Self::Empty),
            0x01 => Ok(Self::Tenant),
            0x02 => Ok(Self::Collection),
            0x03 => Ok(Self::Subject),
            0x04 => Ok(Self::Document),
            0x05 => Ok(Self::FileKey),
            0x06 => Ok(Self::Epoch),
            _other => Err(LabError::Corrupted("Unbekannter VaultSlotType")),
        }
    }
}

/// Slot-Status laut SPEC F.3.
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

/// Strukturierter Tresor-Slot (64 Byte nach SPEC 5.1.0 F.3).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VaultSlot {
    /// Typ des Slots (0..1).
    pub slot_type: VaultSlotType,
    /// Status des Slots (1..2).
    pub status: VaultSlotStatus,
    /// Slot-Generation (48 Bit big-endian, Bytes 2..8).
    pub gen: u64,
    /// Tag (16 Byte, Bytes 8..24).
    pub tag: [u8; 16],
    /// Nutzlast (32 Byte Salt oder maskierter Schlüssel, Bytes 24..56).
    pub payload: [u8; 32],
    /// Authentisierungs-MAC (8 Byte, Bytes 56..64).
    pub mac: [u8; 8],
}

impl VaultSlot {
    /// Parst einen 64-Byte-Slot.
    pub fn parse(bytes: &[u8]) -> Result<Self, LabError> {
        if bytes.len() < VAULT_SLOT_SIZE {
            return Err(LabError::UnexpectedEof {
                expected: VAULT_SLOT_SIZE,
                actual: bytes.len(),
            });
        }

        let st_byte = *bytes.first().ok_or(LabError::Corrupted("SlotType fehlt"))?;
        let slot_type = VaultSlotType::from_u8(st_byte)?;

        let status_byte = *bytes.get(1).ok_or(LabError::Corrupted("Status fehlt"))?;
        let status = VaultSlotStatus::from_u8(status_byte)?;

        let gen_bytes = bytes.get(2..8).ok_or(LabError::Corrupted("Gen fehlt"))?;
        let mut full_gen = [0u8; 8];
        if let Some(dest) = full_gen.get_mut(2..8) {
            dest.copy_from_slice(gen_bytes);
        }
        let gen = u64::from_be_bytes(full_gen);

        let tag: [u8; 16] = bytes
            .get(8..24)
            .and_then(|s| s.try_into().ok())
            .ok_or(LabError::Corrupted("Tag fehlt"))?;

        let payload: [u8; 32] = bytes
            .get(24..56)
            .and_then(|s| s.try_into().ok())
            .ok_or(LabError::Corrupted("Payload fehlt"))?;

        let mac: [u8; 8] = bytes
            .get(56..64)
            .and_then(|s| s.try_into().ok())
            .ok_or(LabError::Corrupted("Mac fehlt"))?;

        // Verifikation Grabstein: Bytes 0 und 2..56 müssen 0 sein
        if status == VaultSlotStatus::Tombstone {
            if st_byte != 0 {
                return Err(LabError::VerificationFailed(
                    "Ungültiger Grabstein: slot_type ist nicht 0",
                ));
            }
            let p_gen_payload = bytes
                .get(2..56)
                .ok_or(LabError::Corrupted("Slot Body fehlt"))?;
            if p_gen_payload.iter().any(|&b| b != 0) {
                return Err(LabError::VerificationFailed(
                    "Ungültiger Grabstein: Nicht-Status-Bytes sind nicht 0",
                ));
            }
        }

        Ok(Self {
            slot_type,
            status,
            gen,
            tag,
            payload,
            mac,
        })
    }

    /// Verifiziert den 8-Byte-MAC eines Slots.
    /// K_vm = HKDF-Expand(K_enc, "ctx/vault-mac/v1", 32)
    /// mac = HMAC-SHA256(K_vm, vault_id || Bytes[0..56))[0..8]
    pub fn verify_mac(
        &self,
        raw_slot_64: &[u8],
        vault_id: &[u8; 16],
        k_vm: &[u8; 32],
    ) -> Result<(), LabError> {
        if self.status == VaultSlotStatus::Empty {
            return Ok(());
        }

        if raw_slot_64.len() < VAULT_SLOT_SIZE {
            return Err(LabError::UnexpectedEof {
                expected: VAULT_SLOT_SIZE,
                actual: raw_slot_64.len(),
            });
        }

        type HmacSha256 = Hmac<Sha256>;
        let mut mac_hasher = <HmacSha256 as hmac::digest::KeyInit>::new_from_slice(k_vm)
            .map_err(|_| LabError::Corrupted("HMAC Initialisierung fehlgeschlagen"))?;

        mac_hasher.update(vault_id);
        let pre = raw_slot_64
            .get(0..56)
            .ok_or(LabError::Corrupted("Slot Pre-Daten ungültig"))?;
        mac_hasher.update(pre);

        let full_tag = mac_hasher.finalize().into_bytes();
        let expected_8 = full_tag
            .get(0..8)
            .ok_or(LabError::Corrupted("Tag Slice fehlt"))?;

        if self.mac != expected_8 {
            return Err(LabError::VerificationFailed("Vault-Slot MAC ungültig"));
        }

        Ok(())
    }
}

/// Tresor Metablock (64 Byte nach SPEC 5.1.0 F.3).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VaultMetaBlock {
    /// Eindeutige Vault-ID (16 Byte).
    pub vault_id: [u8; 16],
    /// Shard-Nummer (0..15).
    pub shard_no: u8,
    /// Basis-2 Logarithmus der Slot-Kapazität.
    pub cap_log2: u8,
    /// Anzahl aktiver Slots (Hinweis).
    pub live: u64,
    /// Anzahl Grabsteine (Hinweis).
    pub tombstones: u64,
    /// Generationsnummer für Rehash.
    pub generation: u64,
    /// Cursor für vault_scrub (rsv).
    pub rsv: u64,
    /// Authentisierungs-MAC (8 Byte).
    pub mac: [u8; 8],
}

impl VaultMetaBlock {
    /// Parst die 64 Byte des Metablocks.
    pub fn parse(bytes: &[u8]) -> Result<Self, LabError> {
        if bytes.len() < VAULT_METABLOCK_SIZE {
            return Err(LabError::UnexpectedEof {
                expected: VAULT_METABLOCK_SIZE,
                actual: bytes.len(),
            });
        }

        let vault_id: [u8; 16] = bytes
            .get(0..16)
            .and_then(|s| s.try_into().ok())
            .ok_or(LabError::Corrupted("vault_id fehlt"))?;

        let shard_no = *bytes.get(16).ok_or(LabError::Corrupted("shard_no fehlt"))?;
        let cap_log2 = *bytes.get(17).ok_or(LabError::Corrupted("cap_log2 fehlt"))?;

        if cap_log2 > 40 {
            return Err(LabError::Corrupted("cap_log2 unzulässig hoch"));
        }

        let live = u64::from_be_bytes(
            bytes
                .get(18..26)
                .and_then(|s| s.try_into().ok())
                .ok_or(LabError::Corrupted("Live fehlt"))?,
        );

        let tombstones = u64::from_be_bytes(
            bytes
                .get(26..34)
                .and_then(|s| s.try_into().ok())
                .ok_or(LabError::Corrupted("Tombstones fehlt"))?,
        );

        let generation = u64::from_be_bytes(
            bytes
                .get(34..42)
                .and_then(|s| s.try_into().ok())
                .ok_or(LabError::Corrupted("Generation fehlt"))?,
        );

        let rsv = u64::from_be_bytes(
            bytes
                .get(42..50)
                .and_then(|s| s.try_into().ok())
                .ok_or(LabError::Corrupted("rsv fehlt"))?,
        );

        let mac: [u8; 8] = bytes
            .get(50..58)
            .and_then(|s| s.try_into().ok())
            .ok_or(LabError::Corrupted("mac fehlt"))?;

        let pad = bytes
            .get(58..64)
            .ok_or(LabError::Corrupted("Padding fehlt"))?;
        if pad.iter().any(|&b| b != 0) {
            return Err(LabError::Corrupted("Metablock Pad ist nicht 0"));
        }

        Ok(Self {
            vault_id,
            shard_no,
            cap_log2,
            live,
            tombstones,
            generation,
            rsv,
            mac,
        })
    }

    /// Berechnet die effektive Slotkapazität aus cap_log2 (2^cap_log2).
    pub fn capacity(&self) -> u64 {
        1u64.checked_shl(self.cap_log2 as u32).unwrap_or(0)
    }

    /// Verifiziert den 8-Byte-MAC des Metablocks.
    /// mac = HMAC-SHA256(K_vm, Bytes[0..50))[0..8]
    pub fn verify_mac(&self, raw_meta_64: &[u8], k_vm: &[u8; 32]) -> Result<(), LabError> {
        if raw_meta_64.len() < VAULT_METABLOCK_SIZE {
            return Err(LabError::UnexpectedEof {
                expected: VAULT_METABLOCK_SIZE,
                actual: raw_meta_64.len(),
            });
        }

        type HmacSha256 = Hmac<Sha256>;
        let mut mac_hasher = <HmacSha256 as hmac::digest::KeyInit>::new_from_slice(k_vm)
            .map_err(|_| LabError::Corrupted("HMAC Init fehlgeschlagen"))?;

        let pre = raw_meta_64
            .get(0..50)
            .ok_or(LabError::Corrupted("Meta Pre-Daten fehlen"))?;
        mac_hasher.update(pre);

        let full_tag = mac_hasher.finalize().into_bytes();
        let expected_8 = full_tag
            .get(0..8)
            .ok_or(LabError::Corrupted("Tag Slice fehlt"))?;

        if self.mac != expected_8 {
            return Err(LabError::VerificationFailed("Metablock MAC ungültig"));
        }

        Ok(())
    }
}

/// Strukturierte Gesamtrepräsentation einer Tresor-Shard-Datei (`<nn>.vlt`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VaultFile {
    /// F.1 Dateikopf (Offset 0..64).
    pub header: FileHeader,
    /// Metablock (Offset 64..128).
    pub meta: VaultMetaBlock,
    /// Slots (Offset 128 + 64 * i).
    pub slots: Vec<VaultSlot>,
}

impl VaultFile {
    /// Parst eine Tresor-Shard-Datei.
    pub fn parse(bytes: &[u8]) -> Result<Self, LabError> {
        let header = FileHeader::parse(bytes)?;
        if header.kind != crate::format::file_header::FileKind::Vault {
            return Err(LabError::UnsupportedKind(header.kind as u8));
        }

        let meta_bytes = bytes
            .get(64..128)
            .ok_or(LabError::Corrupted("Metablock fehlt"))?;
        let meta = VaultMetaBlock::parse(meta_bytes)?;

        let cap = meta.capacity();
        let cap_usize =
            usize::try_from(cap).map_err(|_| LabError::Corrupted("Capacity Überlauf"))?;
        let mut slots = Vec::with_capacity(cap_usize);
        for i in 0..cap {
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
