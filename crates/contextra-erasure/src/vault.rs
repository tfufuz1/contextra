//! Vault: FileVault, RecordSealer, FileKeyring.

pub mod keyring;
pub mod sealer;
pub mod slot;

pub use keyring::*;
pub use sealer::*;
pub use slot::*;

use crate::error::ErasureError;
use std::collections::BTreeMap;

/// Basis-Offset der Slots im Vault-Dateiformat (SPEC F.3, L-4: Kopf 64 B + Metablock 64 B = 128 B).
pub const VAULT_SLOT_BASE: usize = 128;

/// Maximaler Lastfaktor vor Rehash / Kapazitätslimit (0.7).
pub const MAX_LOAD_FACTOR: f64 = 0.7;

/// `FileVault` verwaltet die Schlüssel-Slots auf Datenträgern oder im Speicher (F.3, ER-02..ER-05).
pub struct FileVault {
    capacity: usize,
    slots: Vec<VaultSlot>,
    live_count: usize,
    tombstone_count: usize,
    /// In-Memory-Index für schnelles Sondieren
    index: BTreeMap<(SlotType, [u8; 16]), usize>,
}

impl FileVault {
    /// Erstellt einen neuen In-Memory-FileVault mit einer vorgegebenen Zweierpotenz-Kapazität.
    pub fn new_in_memory(capacity: usize) -> Self {
        let cap = capacity.next_power_of_two().max(16);
        let mut slots = Vec::with_capacity(cap);
        for _ in 0..cap {
            slots.push(VaultSlot::empty());
        }
        Self {
            capacity: cap,
            slots,
            live_count: 0,
            tombstone_count: 0,
            index: BTreeMap::new(),
        }
    }

    /// Berechnet die Startposition durch lineares Hashing (SPEC F.3).
    #[allow(clippy::cast_possible_truncation)]
    fn hash_position(&self, slot_type: SlotType, tag: &[u8; 16]) -> usize {
        let mut hasher = blake3::Hasher::new();
        hasher.update(&[slot_type.as_u8()]);
        hasher.update(tag);
        let hash = hasher.finalize();
        let bytes = hash.as_bytes();
        let val = u64::from_le_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
        ]);
        (val as usize) % self.capacity
    }

    /// Sucht einen Slot über lineares Sondieren.
    pub fn get_slot(
        &self,
        slot_type: SlotType,
        tag: [u8; 16],
    ) -> Result<Option<VaultSlot>, ErasureError> {
        let start_pos = self.hash_position(slot_type, &tag);
        for i in 0..self.capacity {
            let pos = (start_pos + i) % self.capacity;
            let slot = match self.slots.get(pos) {
                Some(s) => s,
                None => return Err(ErasureError::Vault("Ungültiger Index".into())),
            };
            match slot.status {
                SlotStatus::Empty => {
                    return Ok(None);
                }
                SlotStatus::Active => {
                    if slot.slot_type == slot_type && slot.tag == tag {
                        return Ok(Some(slot.clone()));
                    }
                }
                SlotStatus::Tombstone => {
                    continue;
                }
            }
        }
        Ok(None)
    }

    /// Fügt einen neuen Slot ein oder aktualisiert ihn.
    pub fn put_slot(
        &mut self,
        slot_type: SlotType,
        tag: [u8; 16],
        payload: [u8; 32],
    ) -> Result<(), ErasureError> {
        let total_occupied = self.live_count + self.tombstone_count + 1;
        if (total_occupied as f64) / (self.capacity as f64) > MAX_LOAD_FACTOR {
            return Err(ErasureError::Vault(
                "Maximaler Lastfaktor von 0.7 überschritten (Rehash erforderlich)".into(),
            ));
        }

        let start_pos = self.hash_position(slot_type, &tag);
        let mut insert_pos = None;

        for i in 0..self.capacity {
            let pos = (start_pos + i) % self.capacity;
            let slot = match self.slots.get(pos) {
                Some(s) => s,
                None => return Err(ErasureError::Vault("Ungültiger Index".into())),
            };
            match slot.status {
                SlotStatus::Empty => {
                    if insert_pos.is_none() {
                        insert_pos = Some(pos);
                    }
                    break;
                }
                SlotStatus::Tombstone => {
                    if insert_pos.is_none() {
                        insert_pos = Some(pos);
                    }
                }
                SlotStatus::Active => {
                    if slot.slot_type == slot_type && slot.tag == tag {
                        insert_pos = Some(pos);
                        break;
                    }
                }
            }
        }

        let pos = match insert_pos {
            Some(p) => p,
            None => {
                return Err(ErasureError::Vault(
                    "Kein freier Slot beim Sondieren gefunden".into(),
                ))
            }
        };

        let is_existing = self
            .slots
            .get(pos)
            .is_some_and(|s| s.status == SlotStatus::Active);
        let is_tombstone = self
            .slots
            .get(pos)
            .is_some_and(|s| s.status == SlotStatus::Tombstone);

        let new_slot = VaultSlot {
            slot_type,
            tag,
            payload,
            status: SlotStatus::Active,
            generation: [0u8; 6],
            mac: [0u8; 24],
        };

        if let Some(target) = self.slots.get_mut(pos) {
            *target = new_slot;
        }

        if !is_existing {
            self.live_count += 1;
            if is_tombstone {
                self.tombstone_count = self.tombstone_count.saturating_sub(1);
            }
        }
        self.index.insert((slot_type, tag), pos);
        Ok(())
    }

    /// Vernichtet einen Slot nach ER-05 (alles null, Grabstein 0x02 auf Byte 49, Nullen verifizieren).
    pub fn destroy_slot(&mut self, slot_type: SlotType, tag: [u8; 16]) -> Result<(), ErasureError> {
        let start_pos = self.hash_position(slot_type, &tag);
        let mut target_pos = None;

        for i in 0..self.capacity {
            let pos = (start_pos + i) % self.capacity;
            let slot = match self.slots.get(pos) {
                Some(s) => s,
                None => return Err(ErasureError::Vault("Ungültiger Index".into())),
            };
            match slot.status {
                SlotStatus::Empty => break,
                SlotStatus::Tombstone => continue,
                SlotStatus::Active => {
                    if slot.slot_type == slot_type && slot.tag == tag {
                        target_pos = Some(pos);
                        break;
                    }
                }
            }
        }

        let pos = match target_pos {
            Some(p) => p,
            None => return Ok(()),
        };

        let destroyed = VaultSlot {
            slot_type: SlotType::Document,
            tag: [0u8; 16],
            payload: [0u8; 32],
            status: SlotStatus::Tombstone,
            generation: [0u8; 6],
            mac: [0u8; 24],
        };

        if let Some(target) = self.slots.get_mut(pos) {
            *target = destroyed;
        }

        let slot_read = match self.slots.get(pos) {
            Some(s) => s,
            None => return Err(ErasureError::Vault("Slot nach Vernichtung unlesbar".into())),
        };
        if !slot_read.is_destroyed_verifiably() {
            return Err(ErasureError::Vault(
                "Slot-Vernichtung verfehlt Zero-Verification".into(),
            ));
        }

        self.live_count = self.live_count.saturating_sub(1);
        self.tombstone_count += 1;
        self.index.remove(&(slot_type, tag));
        Ok(())
    }
}
