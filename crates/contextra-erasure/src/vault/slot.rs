//! Tresor-Slot-Layout, Validierung und Slot-Nullung (SPEC §11.5, Anhang F.3, D-111, D-125, I-23).
//!
//! Slot-Layout (64 Bytes, big-endian):
//!   Byte  0      – slot_type (1 B): 0x01 T, 0x02 C, 0x03 S, 0x04 D, 0x05 F, 0x06 E; 0x00 leer
//!   Byte  1      – status (1 B):    0x00 leer, 0x01 aktiv, 0x02 Grabstein
//!   Byte  2..7   – gen (u48, 6 B, big-endian)
//!   Byte  8..23  – tag (16 B)
//!   Byte 24..55  – payload (32 B)
//!   Byte 56..63  – mac (8 B, HMAC-SHA256(K_vm, vault_id || Bytes[0..56))[0..8])

use crate::error::ErasureError;
use hmac::{Hmac, Mac};
use sha2::Sha256;

/// Größe eines Tresor-Slots in Bytes (F.3, D-111, I-23).
pub const VAULT_SLOT_SIZE: usize = 64;

/// Slot-Status-Werte (F.3).
pub const STATUS_EMPTY: u8 = 0x00;
/// Slot-Status: aktiv.
pub const STATUS_ACTIVE: u8 = 0x01;
/// Slot-Status: Grabstein (für lineare Sondierung).
pub const STATUS_TOMBSTONE: u8 = 0x02;

/// Byte-Offsets innerhalb eines 64-Byte-Slots (F.3).
pub mod off {
    /// Offset des slot_type-Feldes (Byte 0, 1 B).
    pub const SLOT_TYPE: usize = 0;
    /// Offset des status-Feldes (Byte 1, 1 B).
    pub const STATUS: usize = 1;
    /// Offset des gen-Feldes (Bytes 2..8, u48 big-endian, 6 B).
    pub const GEN: usize = 2;
    /// Offset des tag-Feldes (Bytes 8..24, 16 B).
    pub const TAG: usize = 8;
    /// Offset des payload-Feldes (Bytes 24..56, 32 B).
    pub const PAYLOAD: usize = 24;
    /// Offset des mac-Feldes (Bytes 56..64, 8 B).
    pub const MAC: usize = 56;
}

/// Typen von Tresor-Slots (F.3, D-33, D-79, D-85, D-111, D-125).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[repr(u8)]
pub enum SlotType {
    /// Mandanten-Stamm-Slot.
    Tenant = 0x01,
    /// Collection-Slot.
    Collection = 0x02,
    /// Betroffenen-Slot (Subject).
    Subject = 0x03,
    /// Dokument-Slot.
    Document = 0x04,
    /// Datei-Schlüssel-Slot.
    File = 0x05,
    /// Epochen-Schlüssel-Slot.
    Epoch = 0x06,
}

impl SlotType {
    /// Konvertiert u8 in SlotType.
    pub fn from_u8(b: u8) -> Option<Self> {
        match b {
            0x01 => Some(Self::Tenant),
            0x02 => Some(Self::Collection),
            0x03 => Some(Self::Subject),
            0x04 => Some(Self::Document),
            0x05 => Some(Self::File),
            0x06 => Some(Self::Epoch),
            _ => None,
        }
    }
}

/// Entpackter Slot-Inhalt (in-memory).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SlotData {
    /// Slot-Typ.
    pub slot_type: SlotType,
    /// 16-Byte Tag.
    pub tag: [u8; 16],
    /// 32-Byte Payload (Salt oder maskierter Schlüssel).
    pub payload: [u8; 32],
    /// Generationszähler (u48).
    pub gen: u64,
}

/// Serialisiert einen aktiven Slot in 64 Bytes (ohne MAC-Feld).
/// Kodiert einen vollständigen Tresor-Slot inkl. MAC (F.3, D-111).
pub fn encode_slot(
    vault_id: &[u8; 16],
    slot_type: SlotType,
    status: u8,
    gen: u64,
    tag: &[u8; 16],
    payload: &[u8; 32],
    mac_key: &[u8; 32],
) -> [u8; VAULT_SLOT_SIZE] {
    let mut buf = [0u8; VAULT_SLOT_SIZE];
    buf[off::SLOT_TYPE] = slot_type as u8;
    buf[off::STATUS] = status;
    let gen_bytes = gen.to_be_bytes();
    if let (Some(dest), Some(src)) = (buf.get_mut(off::GEN..off::GEN + 6), gen_bytes.get(2..8)) {
        dest.copy_from_slice(src);
    }
    if let Some(dest) = buf.get_mut(off::TAG..off::TAG + 16) {
        dest.copy_from_slice(tag);
    }
    if let Some(dest) = buf.get_mut(off::PAYLOAD..off::PAYLOAD + 32) {
        dest.copy_from_slice(payload);
    }
    let mac = compute_slot_mac(mac_key, vault_id, &buf[0..56]);
    if let Some(dest) = buf.get_mut(off::MAC..off::MAC + 8) {
        dest.copy_from_slice(&mac);
    }
    buf
}

/// Serialisiert die Nutzlast eines Slots (ohne MAC).
pub fn encode_slot_body(sd: &SlotData) -> [u8; VAULT_SLOT_SIZE] {
    let mut buf = [0u8; VAULT_SLOT_SIZE];
    buf[off::SLOT_TYPE] = sd.slot_type as u8;
    buf[off::STATUS] = STATUS_ACTIVE;
    // gen: 6 Bytes big-endian (u48)
    let gen_be = sd.gen.to_be_bytes();
    if let (Some(dest), Some(src)) = (buf.get_mut(off::GEN..off::TAG), gen_be.get(2..8)) {
        dest.copy_from_slice(src);
    }
    if let Some(dest) = buf.get_mut(off::TAG..off::PAYLOAD) {
        dest.copy_from_slice(&sd.tag);
    }
    if let Some(dest) = buf.get_mut(off::PAYLOAD..off::MAC) {
        dest.copy_from_slice(&sd.payload);
    }
    buf
}

/// Berechnet den 8-Byte Slot-MAC nach SPEC F.3:
/// `HMAC-SHA256(K_vm, vault_id ‖ Bytes[0..56))[0..8]`.
pub fn compute_slot_mac(mac_key: &[u8; 32], vault_id: &[u8; 16], slot_pre_mac: &[u8]) -> [u8; 8] {
    let mut out = [0u8; 8];
    if let Ok(mut hmac) = <Hmac<Sha256> as Mac>::new_from_slice(mac_key) {
        <Hmac<Sha256> as Mac>::update(&mut hmac, vault_id);
        <Hmac<Sha256> as Mac>::update(&mut hmac, slot_pre_mac);
        let full = <Hmac<Sha256> as Mac>::finalize(hmac).into_bytes();
        if let (Some(src), Some(dest)) = (full.get(..8), out.get_mut(..8)) {
            dest.copy_from_slice(src);
        }
    }
    out
}

/// Erstellt das kanonische Grabstein-Abbild (tombstone_image) nach SPEC D-111 Abs. 3 / F.3:
/// Alle Bytes 0 außer `status = 0x02`, MAC über das Abbild mit `vault_id`.
pub fn tombstone_image(vault_id: &[u8; 16], mac_key: &[u8; 32]) -> [u8; VAULT_SLOT_SIZE] {
    let mut buf = [0u8; VAULT_SLOT_SIZE];
    buf[off::STATUS] = STATUS_TOMBSTONE;
    if let Some(pre) = buf.get(..off::MAC) {
        let mac = compute_slot_mac(mac_key, vault_id, pre);
        if let Some(dest) = buf.get_mut(off::MAC..off::MAC + 8) {
            dest.copy_from_slice(&mac);
        }
    }
    buf
}

/// Deserialisiert einen Slot aus 64 Bytes nach SPEC F.3.
/// Gibt `Ok(None)` bei leeren oder Grabstein-Slots zurück,
/// `Err(Corrupt)` bei ungültigem Status oder ungültigem MAC.
pub fn decode_slot(
    raw: &[u8; VAULT_SLOT_SIZE],
    vault_id: &[u8; 16],
    mac_key: &[u8; 32],
) -> Result<Option<SlotData>, ErasureError> {
    let status = match raw.get(off::STATUS) {
        Some(&s) => s,
        None => return Err(ErasureError::Corrupt),
    };

    match status {
        STATUS_EMPTY => {
            // Ein komplett leerer Slot
            if raw.iter().all(|&b| b == 0) {
                return Ok(None);
            }
            return Err(ErasureError::Corrupt);
        }
        STATUS_TOMBSTONE => {
            // Grabstein-Prüfung nach D-111 Abs. 3:
            // Bytes 0 und 2..56 müssen 0 sein, Byte 1 ist 0x02, MAC muss gültig sein
            let pre = match raw.get(..off::MAC) {
                Some(p) => p,
                None => return Err(ErasureError::Corrupt),
            };
            let expected_mac = compute_slot_mac(mac_key, vault_id, pre);
            let mac_slice = match raw.get(off::MAC..off::MAC + 8) {
                Some(m) => m,
                None => return Err(ErasureError::Corrupt),
            };
            if mac_slice != expected_mac {
                return Err(ErasureError::Corrupt);
            }
            if raw.get(off::SLOT_TYPE).copied() != Some(0) {
                return Err(ErasureError::Corrupt);
            }
            let body = match raw.get(off::GEN..off::MAC) {
                Some(b) => b,
                None => return Err(ErasureError::Corrupt),
            };
            if body.iter().any(|&b| b != 0) {
                return Err(ErasureError::Corrupt);
            }
            return Ok(None);
        }
        STATUS_ACTIVE => {}
        _ => return Err(ErasureError::Corrupt),
    }

    // MAC-Prüfung für aktiven Slot: HMAC-SHA256(K_vm, vault_id || Bytes[0..56))[0..8]
    let pre = match raw.get(..off::MAC) {
        Some(p) => p,
        None => return Err(ErasureError::Corrupt),
    };
    let expected_mac = compute_slot_mac(mac_key, vault_id, pre);
    let mac_slice = match raw.get(off::MAC..off::MAC + 8) {
        Some(m) => m,
        None => return Err(ErasureError::Corrupt),
    };
    if mac_slice != expected_mac {
        return Err(ErasureError::Corrupt);
    }

    let slot_type_byte = match raw.get(off::SLOT_TYPE) {
        Some(&b) => b,
        None => return Err(ErasureError::Corrupt),
    };
    let slot_type = SlotType::from_u8(slot_type_byte).ok_or(ErasureError::Corrupt)?;

    let mut tag = [0u8; 16];
    if let (Some(dest), Some(src)) = (tag.get_mut(..), raw.get(off::TAG..off::PAYLOAD)) {
        dest.copy_from_slice(src);
    }

    let mut payload = [0u8; 32];
    if let (Some(dest), Some(src)) = (payload.get_mut(..), raw.get(off::PAYLOAD..off::MAC)) {
        dest.copy_from_slice(src);
    }

    let mut gen_bytes = [0u8; 8];
    if let (Some(dest), Some(src)) = (gen_bytes.get_mut(2..8), raw.get(off::GEN..off::TAG)) {
        dest.copy_from_slice(src);
    }
    let gen = u64::from_be_bytes(gen_bytes);

    Ok(Some(SlotData {
        slot_type,
        tag,
        payload,
        gen,
    }))
}

/// Bestimmt Shard und Startindex für lineare Sondierung nach SPEC D-111 Abs. 1:
/// `h = BLAKE3(slot_type ‖ tag)`
/// `shard = h[0] >> 4` (0..15)
/// `i = u64_be(h[1..9]) mod 2^cap_log2`
pub fn slot_probe_start(slot_type: SlotType, tag: &[u8; 16], capacity: usize) -> (u8, usize) {
    if capacity == 0 {
        return (0, 0);
    }
    let mut hasher = blake3::Hasher::new();
    hasher.update(&[slot_type as u8]);
    hasher.update(tag);
    let h = hasher.finalize();
    let bytes = h.as_bytes();

    let shard = match bytes.first() {
        Some(&b) => b >> 4,
        None => 0,
    };

    let mut idx_bytes = [0u8; 8];
    if let (Some(dest), Some(src)) = (idx_bytes.get_mut(..), bytes.get(1..9)) {
        dest.copy_from_slice(src);
    }
    let hash_val = u64::from_be_bytes(idx_bytes);
    let idx = usize::try_from(hash_val % (capacity as u64)).unwrap_or(0);

    (shard, idx)
}
