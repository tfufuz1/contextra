//! Plattenresidente Open-Addressing-Hashtabelle des Tresors (SPEC §11.5, Anhang F.3).
//!
//! Tabellenformat:
//!   Kopf    (64 B, kind = 10)
//!   Metablock (64 B)
//!   N Slots à 80 B (N = capacity, Zweierpotenz)
//!
//! Offset des ersten Slots = 128 B (= VAULT_SLOT_BASE).
//! Lastfaktor ≤ 0,7: (live + tombstones) / capacity ≤ 0,7.

use super::slot::{
    compute_slot_mac, decode_slot, encode_slot_body, slot_probe_start, SlotData, SlotType,
    STATUS_ACTIVE, STATUS_EMPTY, STATUS_TOMBSTONE, VAULT_SLOT_SIZE,
};
use crate::error::ErasureError;

/// Byte-Offset des ersten Slots in der Vault-Datei (F.3, ER-03).
pub const VAULT_SLOT_BASE: usize = 128;

/// In-Memory-Tresor für Tests und als Implementierungsbasis.
pub struct VaultTable {
    /// Slot-Rohdaten (capacity × VAULT_SLOT_SIZE Bytes).
    raw: Vec<u8>,
    /// Kapazität (Zweierpotenz).
    capacity: usize,
    /// Anzahl aktiver (live) Slots.
    live: usize,
    /// Anzahl Grabsteine.
    tombstones: usize,
    /// MAC-Schlüssel (32 Bytes).
    mac_key: [u8; 32],
}

impl VaultTable {
    /// Erstellt eine neue leere Tabelle mit gegebener Kapazität (muss Zweierpotenz sein).
    pub fn new(capacity: usize, mac_key: [u8; 32]) -> Self {
        let cap = capacity.next_power_of_two().max(4);
        Self {
            raw: vec![0u8; cap * VAULT_SLOT_SIZE],
            capacity: cap,
            live: 0,
            tombstones: 0,
            mac_key,
        }
    }

    /// Prüft, ob der Lastfaktor das Maximum überschreitet (unter Einrechnung des nächsten Einfügens).
    pub fn load_exceeded(&self) -> bool {
        // (live + tombstones + 1) * 10 > capacity * 7
        (self.live + self.tombstones + 1).saturating_mul(10) > self.capacity.saturating_mul(7)
    }

    /// Liest rohe Slot-Bytes an Position `idx`.
    fn slot_bytes(&self, idx: usize) -> Option<&[u8; VAULT_SLOT_SIZE]> {
        let off = idx.checked_mul(VAULT_SLOT_SIZE)?;
        let slice = self.raw.get(off..off + VAULT_SLOT_SIZE)?;
        slice.try_into().ok()
    }

    /// Findet den Slot-Index für (slot_type, tag). Gibt `None` zurück wenn nicht gefunden.
    /// Stoppt bei `STATUS_EMPTY` (Sondierung endet dort, D-79).
    pub fn find(&self, slot_type: SlotType, tag: &[u8; 16]) -> Option<usize> {
        let start = slot_probe_start(slot_type, tag, self.capacity);
        for i in 0..self.capacity {
            let idx = (start + i) % self.capacity;
            let bytes = self.slot_bytes(idx)?;
            let status = bytes[super::slot::off::STATUS];
            match status {
                STATUS_EMPTY => return None,
                STATUS_TOMBSTONE => {
                    // Grabstein: Sondierung fortsetzen
                }
                STATUS_ACTIVE => {
                    if bytes[super::slot::off::SLOT_TYPE] == slot_type as u8
                        && &bytes[super::slot::off::TAG..super::slot::off::TAG + 16] == tag
                    {
                        return Some(idx);
                    }
                }
                _ => {}
            }
        }
        None
    }

    /// Liest einen Slot (dekodiert mit MAC-Prüfung).
    pub fn get(
        &self,
        slot_type: SlotType,
        tag: &[u8; 16],
    ) -> Result<Option<SlotData>, ErasureError> {
        match self.find(slot_type, tag) {
            None => Ok(None),
            Some(idx) => {
                let bytes = self.slot_bytes(idx).ok_or(ErasureError::Corrupt)?;
                decode_slot(bytes, &self.mac_key)
            }
        }
    }

    /// Schreibt einen Slot (ER-04). Gibt Fehler zurück wenn Kapazität voll.
    pub fn put(&mut self, sd: SlotData) -> Result<(), ErasureError> {
        if self.load_exceeded() {
            return Err(ErasureError::Capacity);
        }
        let start = slot_probe_start(sd.slot_type, &sd.tag, self.capacity);
        for i in 0..self.capacity {
            let idx = (start + i) % self.capacity;
            let status_pos = idx * VAULT_SLOT_SIZE + super::slot::off::STATUS;
            let status = self.raw.get(status_pos).copied().unwrap_or(STATUS_EMPTY);
            if status == STATUS_EMPTY || status == STATUS_TOMBSTONE {
                // Freien oder Grabstein-Slot überschreiben
                if status == STATUS_TOMBSTONE {
                    self.tombstones = self.tombstones.saturating_sub(1);
                }
                self.write_slot(idx, &sd)?;
                self.live += 1;
                return Ok(());
            }
        }
        Err(ErasureError::Capacity)
    }

    /// Schreibt einen SlotData in die rohen Bytes inkl. MAC.
    fn write_slot(&mut self, idx: usize, sd: &SlotData) -> Result<(), ErasureError> {
        let mut buf = encode_slot_body(sd);
        // MAC berechnen und einsetzen
        let mac = compute_slot_mac(&self.mac_key, &buf[..56]);
        buf[super::slot::off::MAC..super::slot::off::MAC + 24].copy_from_slice(&mac);
        let off = idx * VAULT_SLOT_SIZE;
        let dest = self
            .raw
            .get_mut(off..off + VAULT_SLOT_SIZE)
            .ok_or(ErasureError::Corrupt)?;
        dest.copy_from_slice(&buf);
        Ok(())
    }

    /// Vernichtet einen Slot (ER-05): alle 80 Bytes auf 0, dann Byte 49 = 0x02 (Grabstein).
    /// Gibt Fehler zurück wenn Slot nicht gefunden.
    pub fn destroy(&mut self, slot_type: SlotType, tag: &[u8; 16]) -> Result<(), ErasureError> {
        let idx = self.find(slot_type, tag).ok_or(ErasureError::SlotMissing)?;
        let off = idx * VAULT_SLOT_SIZE;

        // Alle 80 Bytes auf 0 setzen
        let slot_slice = self
            .raw
            .get_mut(off..off + VAULT_SLOT_SIZE)
            .ok_or(ErasureError::Corrupt)?;
        for b in slot_slice.iter_mut() {
            *b = 0;
        }

        // Grabstein setzen (Byte 49 = 0x02)
        let status_ref = self
            .raw
            .get_mut(off + super::slot::off::STATUS)
            .ok_or(ErasureError::Corrupt)?;
        *status_ref = STATUS_TOMBSTONE;

        self.live = self.live.saturating_sub(1);
        self.tombstones += 1;

        // Verifizierung: Bytes 0..49 und 50..80 müssen 0 sein
        let prefix = self
            .raw
            .get(off..off + super::slot::off::STATUS)
            .ok_or(ErasureError::Corrupt)?;
        if prefix.iter().any(|&b| b != 0) {
            return Err(ErasureError::Corrupt);
        }

        let suffix = self
            .raw
            .get(off + super::slot::off::GEN..off + VAULT_SLOT_SIZE)
            .ok_or(ErasureError::Corrupt)?;
        if suffix.iter().any(|&b| b != 0) {
            return Err(ErasureError::Corrupt);
        }

        Ok(())
    }

    /// Anzahl aktiver Slots.
    pub fn live_count(&self) -> usize {
        self.live
    }

    /// Kapazität.
    pub fn capacity(&self) -> usize {
        self.capacity
    }
}
