//! Tresor-Modul (SPEC §11.5).
//!
//! Enthält `FileVault`, die öffentliche Fassade über `VaultTable`.

pub mod table;
pub mod slot;
pub mod sealer;
pub mod keyring;
pub mod rehash;
pub mod open;

use crate::error::ErasureError;
use slot::{SlotData, SlotType};
use table::{VaultTable, VAULT_SLOT_BASE};

/// Öffentlich re-exportierte Konstanten für Tests und externe Crates.
pub use slot::{VAULT_SLOT_SIZE, SlotType as VaultSlotType};
pub use table::VAULT_SLOT_BASE;

/// In-Memory-Tresor (Fassade über `VaultTable`).
///
/// `FileVault::new_in_memory` wird für Tests und für die vollständige Implementierung
/// verwendet; die produktive Variante arbeitet über `Vfs` und ein reales VfsFile.
pub struct FileVault {
    table: VaultTable,
}

impl FileVault {
    /// Erstellt einen leeren In-Memory-Tresor mit gegebener Kapazität (Zweierpotenz).
    pub fn new_in_memory(capacity: usize) -> Self {
        let mac_key = [0u8; 32]; // Test-Schlüssel; produktiv: aus KeyHierarchy::vault_mac()
        Self {
            table: VaultTable::new(capacity, mac_key),
        }
    }

    /// Erstellt einen In-Memory-Tresor mit einem spezifischen MAC-Schlüssel.
    pub fn new_in_memory_with_key(capacity: usize, mac_key: [u8; 32]) -> Self {
        Self {
            table: VaultTable::new(capacity, mac_key),
        }
    }

    /// Speichert einen Slot (ER-04).
    pub fn put_slot(
        &mut self,
        slot_type: SlotType,
        tag: [u8; 16],
        payload: [u8; 32],
    ) -> Result<(), ErasureError> {
        let sd = SlotData {
            slot_type,
            tag,
            payload,
            gen: 0,
        };
        self.table.put(sd)
    }

    /// Liest einen Slot (ER-02).
    pub fn get_slot(
        &self,
        slot_type: SlotType,
        tag: [u8; 16],
    ) -> Result<Option<SlotData>, ErasureError> {
        self.table.get(slot_type, &tag)
    }

    /// Vernichtet einen Slot (ER-05): Nullung + Grabstein + Verifikation.
    pub fn destroy_slot(
        &mut self,
        slot_type: SlotType,
        tag: [u8; 16],
    ) -> Result<(), ErasureError> {
        self.table.destroy(slot_type, &tag)
    }

    /// Prüft ob der Lastfaktor überschritten ist.
    pub fn load_exceeded(&self) -> bool {
        self.table.load_exceeded()
    }

    /// Anzahl aktiver Slots.
    pub fn live_count(&self) -> usize {
        self.table.live_count()
    }

    /// Kapazität.
    pub fn capacity(&self) -> usize {
        self.table.capacity()
    }
}
