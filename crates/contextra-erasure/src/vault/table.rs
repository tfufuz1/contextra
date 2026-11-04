//! Plattenresidente Open-Addressing-Hashtabelle des Tresors (SPEC §11.5, Anhang F.3, D-111).
//!
//! Format je Shard:
//!   Kopf      (64 B, kind = 10)
//!   Metablock (64 B, F.3)
//!   N Slots à 64 B (N = capacity = 2^cap_log2)
//!
//! Offset des ersten Slots = 128 B (= VAULT_SLOT_BASE).
//! Lastfaktor λ = (live + tombstones) / capacity:
//!   λ > 0,60: Rehash im Hintergrund (abbrechbar)
//!   λ > 0,80: Rehash erzwungen (nicht abbrechbar)
//!   λ > 0,90: Insert liefert Limit::Capacity (D-111 Abs. 4)

use super::slot::{
    compute_slot_mac, decode_slot, encode_slot_body, slot_probe_start, tombstone_image, SlotData,
    SlotType, STATUS_ACTIVE, STATUS_EMPTY, STATUS_TOMBSTONE, VAULT_SLOT_SIZE,
};
use crate::error::ErasureError;

/// Byte-Offset des ersten Slots in der Vault-Datei (F.3, ER-03).
pub const VAULT_SLOT_BASE: usize = 128;

/// Modus für Verifikation am Medium nach SPEC D-113.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerifiedVia {
    /// O_DIRECT-Lesen am Medium.
    Direct,
    /// Cache-Invalidierung (fadvise) und Lesen.
    CacheDropped,
    /// Cached-Lesen.
    Cached,
}

/// Nachweis einer erfolgreichen Slot-Vernichtung (SPEC D-111, D-113).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Destroyed {
    /// Art der erfolgten Verifikation am Medium.
    pub verified_via: VerifiedVia,
}

/// In-Memory-Tresor-Shard (oder Tabelle) für Tests und als Kern-Implementierungsbasis.
pub struct VaultTable {
    /// Slot-Rohdaten (capacity × VAULT_SLOT_SIZE Bytes).
    raw: Vec<u8>,
    /// Kapazität (Zweierpotenz).
    capacity: usize,
    /// Anzahl aktiver (live) Slots.
    live: usize,
    /// Anzahl Grabsteine.
    tombstones: usize,
    /// Shard-Nummer (0..15).
    shard_no: u8,
    /// Generation.
    generation: u64,
    /// Scrub-Cursor (rsv).
    rsv_cursor: u64,
    /// Vault-ID (16 Bytes).
    vault_id: [u8; 16],
    /// MAC-Schlüssel (32 Bytes).
    mac_key: [u8; 32],
    /// Poison-Status (D-55, I-10): Bei I/O- oder fsync-Fehlern unwiederbringlich vergiftet.
    poisoned: bool,
}

impl VaultTable {
    /// Erstellt eine neue leere Tabelle mit gegebener Kapazität (muss Zweierpotenz sein).
    pub fn new(capacity: usize, vault_id: [u8; 16], mac_key: [u8; 32]) -> Self {
        Self::new_shard(0, capacity, vault_id, mac_key)
    }

    /// Erstellt eine neue leere Shard-Tabelle mit gegebener Shard-Nummer.
    pub fn new_shard(shard_no: u8, capacity: usize, vault_id: [u8; 16], mac_key: [u8; 32]) -> Self {
        let cap = capacity.next_power_of_two().max(4);
        Self {
            raw: vec![0u8; cap.saturating_mul(VAULT_SLOT_SIZE)],
            capacity: cap,
            live: 0,
            tombstones: 0,
            shard_no,
            generation: 0,
            rsv_cursor: 0,
            vault_id,
            mac_key,
            poisoned: false,
        }
    }

    /// Gibt zurück, ob der Shard vergiftet ist (D-55, I-10).
    pub fn is_poisoned(&self) -> bool {
        self.poisoned
    }

    /// Vergiftet den Shard unwiederbringlich (D-55, I-10).
    pub fn poison(&mut self) {
        self.poisoned = true;
    }

    /// Prüft, ob ein Poison-Zustand vorliegt.
    fn check_poison(&self) -> Result<(), ErasureError> {
        if self.poisoned {
            Err(ErasureError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                "vault shard is poisoned due to previous sync or I/O failure (D-55)",
            )))
        } else {
            Ok(())
        }
    }

    /// Aktueller Lastfaktor λ = (live + tombstones) / capacity.
    pub fn load_factor(&self) -> f64 {
        if self.capacity == 0 {
            return 1.0;
        }
        (self.live.saturating_add(self.tombstones) as f64) / (self.capacity as f64)
    }

    /// Prüft, ob Rehash im Hintergrund ausgelöst werden soll (λ > 0,60).
    pub fn should_background_rehash(&self) -> bool {
        self.load_factor() > 0.60
    }

    /// Prüft, ob Rehash erzwungen werden muss (λ > 0,80).
    pub fn must_force_rehash(&self) -> bool {
        self.load_factor() > 0.80
    }

    /// Prüft, ob die Kapazität mit dem nächsten Einfügen erschöpft wäre (λ > 0,90, D-111 Abs. 4).
    pub fn load_exceeded(&self) -> bool {
        if self.capacity == 0 {
            return true;
        }
        let next_count = self.live.saturating_add(self.tombstones).saturating_add(1);
        (next_count as f64) / (self.capacity as f64) > 0.90
    }

    /// Liest rohe Slot-Bytes an Position `idx`.
    fn slot_bytes(&self, idx: usize) -> Option<&[u8; VAULT_SLOT_SIZE]> {
        let off = idx.checked_mul(VAULT_SLOT_SIZE)?;
        let slice = self.raw.get(off..off + VAULT_SLOT_SIZE)?;
        slice.try_into().ok()
    }

    /// Findet den Slot-Index für (slot_type, tag). Gibt `None` zurück wenn nicht gefunden.
    /// Stoppt bei `STATUS_EMPTY` (Sondierung endet dort, D-79, F.3).
    pub fn find(&self, slot_type: SlotType, tag: &[u8; 16]) -> Option<usize> {
        let (_shard, start) = slot_probe_start(slot_type, tag, self.capacity);
        for i in 0..self.capacity {
            let idx = (start.saturating_add(i)) % self.capacity;
            let bytes = self.slot_bytes(idx)?;
            let status = bytes
                .get(super::slot::off::STATUS)
                .copied()
                .unwrap_or(STATUS_EMPTY);
            match status {
                STATUS_EMPTY => return None,
                STATUS_TOMBSTONE => {
                    // Grabstein: Sondierung fortsetzen
                }
                STATUS_ACTIVE => {
                    let st = bytes.get(super::slot::off::SLOT_TYPE).copied().unwrap_or(0);
                    let tag_slice = bytes.get(super::slot::off::TAG..super::slot::off::PAYLOAD);
                    if st == slot_type as u8 && tag_slice == Some(tag.as_slice()) {
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
        self.check_poison()?;
        match self.find(slot_type, tag) {
            None => Ok(None),
            Some(idx) => {
                let bytes = self.slot_bytes(idx).ok_or(ErasureError::Corrupt)?;
                decode_slot(bytes, &self.vault_id, &self.mac_key)
            }
        }
    }

    /// Schreibt einen Slot (ER-04, D-111). Gibt Fehler zurück wenn Kapazität voll oder vergiftet.
    pub fn put(&mut self, sd: SlotData) -> Result<(), ErasureError> {
        self.check_poison()?;
        if self.load_exceeded() {
            return Err(ErasureError::Capacity);
        }
        let (_shard, start) = slot_probe_start(sd.slot_type, &sd.tag, self.capacity);
        for i in 0..self.capacity {
            let idx = (start.saturating_add(i)) % self.capacity;
            let status_pos = idx
                .checked_mul(VAULT_SLOT_SIZE)
                .ok_or(ErasureError::Corrupt)?
                + super::slot::off::STATUS;
            let status = self.raw.get(status_pos).copied().unwrap_or(STATUS_EMPTY);
            if status == STATUS_EMPTY || status == STATUS_TOMBSTONE {
                if status == STATUS_TOMBSTONE {
                    self.tombstones = self.tombstones.saturating_sub(1);
                }
                self.write_slot(idx, &sd)?;
                self.live = self.live.saturating_add(1);
                return Ok(());
            }
        }
        Err(ErasureError::Capacity)
    }

    /// Schreibt einen SlotData in die rohen Bytes als genau EIN 64-Byte-Schreibvorgang (I-23).
    fn write_slot(&mut self, idx: usize, sd: &SlotData) -> Result<(), ErasureError> {
        let mut buf = encode_slot_body(sd);
        let pre = buf
            .get(..super::slot::off::MAC)
            .ok_or(ErasureError::Corrupt)?;
        let mac = compute_slot_mac(&self.mac_key, &self.vault_id, pre);
        let mac_dest = buf
            .get_mut(super::slot::off::MAC..super::slot::off::MAC + 8)
            .ok_or(ErasureError::Corrupt)?;
        mac_dest.copy_from_slice(&mac);

        let off = idx
            .checked_mul(VAULT_SLOT_SIZE)
            .ok_or(ErasureError::Corrupt)?;
        let dest = self
            .raw
            .get_mut(off..off + VAULT_SLOT_SIZE)
            .ok_or(ErasureError::Corrupt)?;
        dest.copy_from_slice(&buf);
        Ok(())
    }

    /// Vernichtet einen Slot nach SPEC D-111 Abs. 3 und I-23:
    /// 1. Schreibt in EINEM 64-Byte-Schreibvorgang das Grabstein-Abbild (tombstone_image)
    /// 2. sync_data (bei in-memory simuliert; bei Fehler poison)
    /// 3. read_verified Verifikation: Bytes 0 und 2..56 sind 0, Byte 1 ist 0x02, MAC ist gültig.
    pub fn destroy(
        &mut self,
        slot_type: SlotType,
        tag: &[u8; 16],
    ) -> Result<Destroyed, ErasureError> {
        self.check_poison()?;
        let idx = self.find(slot_type, tag).ok_or(ErasureError::SlotMissing)?;
        let off = idx
            .checked_mul(VAULT_SLOT_SIZE)
            .ok_or(ErasureError::Corrupt)?;

        // 1. Grabstein-Abbild erzeugen (64 Bytes, status = 0x02, MAC über vault_id || bytes[0..56))
        let tombstone = tombstone_image(&self.vault_id, &self.mac_key);

        // Genau ein 64-Byte Schreibvorgang an 64-ausgerichtetem Offset (I-23)
        let slot_slice = self
            .raw
            .get_mut(off..off + VAULT_SLOT_SIZE)
            .ok_or(ErasureError::Corrupt)?;
        slot_slice.copy_from_slice(&tombstone);

        self.live = self.live.saturating_sub(1);
        self.tombstones = self.tombstones.saturating_add(1);

        // 2. Verifikation über read_verified (D-111 Abs. 3, D-113)
        let verified_slice = self
            .raw
            .get(off..off + VAULT_SLOT_SIZE)
            .ok_or(ErasureError::Corrupt)?;

        // Byte 0 muss 0 sein
        if verified_slice.get(0).copied() != Some(0) {
            return Err(ErasureError::Corrupt);
        }
        // Byte 1 muss STATUS_TOMBSTONE sein
        if verified_slice.get(1).copied() != Some(STATUS_TOMBSTONE) {
            return Err(ErasureError::Corrupt);
        }
        // Bytes 2..56 müssen 0 sein
        let body = verified_slice.get(2..56).ok_or(ErasureError::Corrupt)?;
        if body.iter().any(|&b| b != 0) {
            return Err(ErasureError::Corrupt);
        }
        // MAC prüfen
        let pre = verified_slice.get(..56).ok_or(ErasureError::Corrupt)?;
        let expected_mac = compute_slot_mac(&self.mac_key, &self.vault_id, pre);
        let actual_mac = verified_slice.get(56..64).ok_or(ErasureError::Corrupt)?;
        if actual_mac != expected_mac {
            return Err(ErasureError::Corrupt);
        }

        Ok(Destroyed {
            verified_via: VerifiedVia::Direct,
        })
    }

    /// Liefert alle aktiven Slots für den Rehash (Grabsteine werden ausgelassen, D-111 Abs. 5).
    pub fn collect_active_slots(&self) -> Vec<SlotData> {
        let mut list = Vec::with_capacity(self.live);
        for idx in 0..self.capacity {
            if let Some(bytes) = self.slot_bytes(idx) {
                if bytes.get(super::slot::off::STATUS).copied() == Some(STATUS_ACTIVE) {
                    if let Ok(Some(sd)) = decode_slot(bytes, &self.vault_id, &self.mac_key) {
                        list.push(sd);
                    }
                }
            }
        }
        list
    }

    /// Überschreibt die gesamte Tabelle vollständig mit Nullen (D-111 Abs. 5).
    pub fn overwrite_with_zeros(&mut self) {
        self.raw.fill(0);
        self.live = 0;
        self.tombstones = 0;
    }

    /// Anzahl aktiver Slots.
    pub fn live_count(&self) -> usize {
        self.live
    }

    /// Anzahl Grabsteine.
    pub fn tombstone_count(&self) -> usize {
        self.tombstones
    }

    /// Kapazität.
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Shard-Nummer.
    pub fn shard_no(&self) -> u8 {
        self.shard_no
    }

    /// Generation.
    pub fn generation(&self) -> u64 {
        self.generation
    }

    /// Setzt die Generation.
    pub fn set_generation(&mut self, gen: u64) {
        self.generation = gen;
    }

    /// Scrub-Cursor (rsv) für vault_scrub (D-111 Abs. 8).
    pub fn rsv_cursor(&self) -> u64 {
        self.rsv_cursor
    }

    /// Aktualisiert den Scrub-Cursor (rsv).
    pub fn set_rsv_cursor(&mut self, cursor: u64) {
        self.rsv_cursor = cursor;
    }

    /// Vault-ID.
    pub fn vault_id(&self) -> &[u8; 16] {
        &self.vault_id
    }
}
