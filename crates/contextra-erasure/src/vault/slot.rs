//! Tresor-Slot-Layout, Validierung und Slot-Nullung (SPEC §11.5, Anhang F.3, ER-02, ER-05).
//!
//! Slot-Layout (80 Bytes):
//!   Byte  0      – slot_type (1 B)
//!   Byte  1..16  – tag (16 B)
//!   Byte 17..48  – payload (32 B)
//!   Byte 49      – status (1 B):  0x00 = leer, 0x01 = aktiv, 0x02 = Grabstein
//!   Byte 50..55  – gen (u48, 6 B, little-endian)
//!   Byte 56..79  – mac (24 B, HMAC-SHA256 der ersten 56 Bytes, gekürzt)

use crate::error::ErasureError;

/// Größe eines Tresor-Slots in Bytes (F.3, ER-02).
pub const VAULT_SLOT_SIZE: usize = 80;

/// Slot-Status-Werte.
pub const STATUS_EMPTY: u8 = 0x00;
/// Slot-Status: aktiv.
pub const STATUS_ACTIVE: u8 = 0x01;
/// Slot-Status: Grabstein (für lineare Sondierung).
pub const STATUS_TOMBSTONE: u8 = 0x02;

/// Byte-Offsets innerhalb eines Slots.
pub mod off {
    /// Offset des slot_type-Feldes.
    pub const SLOT_TYPE: usize = 0;
    /// Offset des tag-Feldes.
    pub const TAG: usize = 1;
    /// Offset des payload-Feldes.
    pub const PAYLOAD: usize = 17;
    /// Offset des status-Feldes.
    pub const STATUS: usize = 49;
    /// Offset des gen-Feldes (u48, 6 Bytes).
    pub const GEN: usize = 50;
    /// Offset des mac-Feldes (24 Bytes).
    pub const MAC: usize = 56;
}

/// Typen von Tresor-Slots (F.3, ER-06).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[repr(u8)]
pub enum SlotType {
    /// Mandanten-Stamm-Slot.
    Tenant = 0x54,   // 'T'
    /// Collection-Slot.
    Collection = 0x43, // 'C'
    /// Betroffenen-Slot (Subject).
    Subject = 0x53,  // 'S'
    /// Dokument-Slot.
    Document = 0x44, // 'D'
    /// Datei-Schlüssel-Slot.
    File = 0x46,     // 'F'
}

impl SlotType {
    /// Konvertiert u8 in SlotType.
    pub fn from_u8(b: u8) -> Option<Self> {
        match b {
            0x54 => Some(Self::Tenant),
            0x43 => Some(Self::Collection),
            0x53 => Some(Self::Subject),
            0x44 => Some(Self::Document),
            0x46 => Some(Self::File),
            _ => None,
        }
    }
}

/// Entpackter Slot-Inhalt (in-memory).
#[derive(Clone, Debug)]
pub struct SlotData {
    /// Slot-Typ.
    pub slot_type: SlotType,
    /// 16-Byte Tag.
    pub tag: [u8; 16],
    /// 32-Byte Payload.
    pub payload: [u8; 32],
    /// Generationszähler.
    pub gen: u64,
}

/// Serialisiert einen Slot in 80 Bytes (ohne MAC-Feld, das separat berechnet wird).
pub fn encode_slot_body(sd: &SlotData) -> [u8; VAULT_SLOT_SIZE] {
    let mut buf = [0u8; VAULT_SLOT_SIZE];
    buf[off::SLOT_TYPE] = sd.slot_type as u8;
    buf[off::TAG..off::TAG + 16].copy_from_slice(&sd.tag);
    buf[off::PAYLOAD..off::PAYLOAD + 32].copy_from_slice(&sd.payload);
    buf[off::STATUS] = STATUS_ACTIVE;
    // gen: 6 Bytes little-endian
    let gen_bytes = sd.gen.to_le_bytes();
    buf[off::GEN..off::GEN + 6].copy_from_slice(&gen_bytes[..6]);
    buf
}

/// Deserialisiert einen Slot aus 80 Bytes. Gibt `None` bei leeren Slots zurück,
/// `Err(Corrupt)` bei MAC-Fehler.
pub fn decode_slot(
    raw: &[u8; VAULT_SLOT_SIZE],
    mac_key: &[u8; 32],
) -> Result<Option<SlotData>, ErasureError> {
    let status = raw[off::STATUS];
    match status {
        STATUS_EMPTY => return Ok(None),
        STATUS_TOMBSTONE => return Ok(None),
        STATUS_ACTIVE => {}
        _ => return Err(ErasureError::Corrupt),
    }

    // MAC-Prüfung: HMAC-SHA256 der ersten 56 Bytes, gekürzt auf 24 Bytes
    let expected_mac = compute_slot_mac(mac_key, &raw[..56]);
    if raw[off::MAC..off::MAC + 24] != expected_mac {
        return Err(ErasureError::Corrupt);
    }

    let slot_type_byte = raw[off::SLOT_TYPE];
    let slot_type = SlotType::from_u8(slot_type_byte).ok_or(ErasureError::Corrupt)?;

    let mut tag = [0u8; 16];
    tag.copy_from_slice(&raw[off::TAG..off::TAG + 16]);

    let mut payload = [0u8; 32];
    payload.copy_from_slice(&raw[off::PAYLOAD..off::PAYLOAD + 32]);

    let mut gen_bytes = [0u8; 8];
    gen_bytes[..6].copy_from_slice(&raw[off::GEN..off::GEN + 6]);
    let gen = u64::from_le_bytes(gen_bytes);

    Ok(Some(SlotData {
        slot_type,
        tag,
        payload,
        gen,
    }))
}

/// Berechnet den Slot-MAC: HMAC-SHA256(K_vm, Bytes[0..56])[0..24].
pub fn compute_slot_mac(mac_key: &[u8; 32], body: &[u8]) -> [u8; 24] {
    use sha2::{Digest, Sha256};
    let mut ipad = [0x36u8; 64];
    let mut opad = [0x5cu8; 64];
    for (i, &b) in mac_key.iter().enumerate() {
        if let Some(target) = ipad.get_mut(i) {
            *target ^= b;
        }
        if let Some(target) = opad.get_mut(i) {
            *target ^= b;
        }
    }
    let mut inner = Sha256::new();
    inner.update(&ipad);
    inner.update(body);
    let inner_hash = inner.finalize();

    let mut outer = Sha256::new();
    outer.update(&opad);
    outer.update(&inner_hash);
    let outer_hash = outer.finalize();

    let mut out = [0u8; 24];
    out.copy_from_slice(&outer_hash[..24]);
    out
}

/// Bestimmt die Slot-Position im Slot-Array durch BLAKE3-Hash.
/// `BLAKE3(slot_type ‖ tag)[0..8] mod capacity` → lineares Sondieren.
pub fn slot_probe_start(slot_type: SlotType, tag: &[u8; 16], capacity: usize) -> usize {
    let mut hasher = blake3::Hasher::new();
    hasher.update(&[slot_type as u8]);
    hasher.update(tag);
    let h = hasher.finalize();
    let idx = u64::from_le_bytes(
        h.as_bytes()[..8]
            .try_into()
            .expect("BLAKE3 output >= 8 bytes"),
    );
    (idx as usize) % capacity
}
