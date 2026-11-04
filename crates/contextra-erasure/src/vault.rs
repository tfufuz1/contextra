//! Tresor-Modul (SPEC §11.5, D-111, D-112, D-125).
//!
//! Enthält `FileVault`, die 16-Shard-Fassade über `VaultTable`.

pub mod keyring;
pub mod open;
pub mod rehash;
pub mod sealer;
pub mod slot;
pub mod table;

use crate::error::ErasureError;
use slot::SlotData;
use table::VaultTable;

pub use keyring::{file_id_to_tag, mask_file_key, InMemoryKeyring, VaultKeyring};
pub use open::{
    compute_metablock_mac, encode_metablock, open_vault, open_vault_shard, MetablockParams,
    VaultOpenOutcome, VAULT_METABLOCK_SIZE,
};
pub use rehash::{rehash, rehash_shard, RehashOutcome};
pub use sealer::VaultSealer;
pub use slot::{
    compute_slot_mac, decode_slot, encode_slot_body, slot_probe_start, tombstone_image,
    SlotData as VaultSlotData, SlotType as VaultSlotType, SlotType, STATUS_ACTIVE, STATUS_EMPTY,
    STATUS_TOMBSTONE, VAULT_SLOT_SIZE,
};
pub use table::{Destroyed, VaultTable as VaultShard, VerifiedVia, VAULT_SLOT_BASE};

/// Anzahl Shards im Tresor nach SPEC D-111 Abs. 1.
pub const VAULT_NUM_SHARDS: usize = 16;

/// 16-Shard Tresor nach SPEC D-111.
///
/// `FileVault` verteilt Slots deterministisch auf 16 Shards nach:
/// `h = BLAKE3(slot_type ‖ tag)`
/// `shard = h[0] >> 4` (0..15)
/// `i = u64_be(h[1..9]) mod 2^cap_log2`
pub struct FileVault {
    shards: Vec<VaultTable>,
    vault_id: [u8; 16],
    mac_key: [u8; 32],
}

impl FileVault {
    /// Erstellt einen leeren In-Memory-Tresor mit 16 Shards und gegebener Gesamtkapazität.
    pub fn new_in_memory(total_capacity: usize) -> Self {
        let vault_id = [0u8; 16];
        let mac_key = [0u8; 32];
        Self::new_in_memory_with_id_and_key(total_capacity, vault_id, mac_key)
    }

    /// Erstellt einen In-Memory-Tresor mit einem spezifischen MAC-Schlüssel.
    pub fn new_in_memory_with_key(total_capacity: usize, mac_key: [u8; 32]) -> Self {
        let vault_id = [0u8; 16];
        Self::new_in_memory_with_id_and_key(total_capacity, vault_id, mac_key)
    }

    /// Erstellt einen In-Memory-Tresor mit vault_id und MAC-Schlüssel.
    pub fn new_in_memory_with_id_and_key(
        total_capacity: usize,
        vault_id: [u8; 16],
        mac_key: [u8; 32],
    ) -> Self {
        let shard_cap = (total_capacity / VAULT_NUM_SHARDS)
            .next_power_of_two()
            .max(4);
        let mut shards = Vec::with_capacity(VAULT_NUM_SHARDS);
        for i in 0..VAULT_NUM_SHARDS {
            shards.push(VaultTable::new_shard(
                u8::try_from(i).unwrap_or(0),
                shard_cap,
                vault_id,
                mac_key,
            ));
        }
        Self {
            shards,
            vault_id,
            mac_key,
        }
    }

    /// Ermittelt den zuständigen Shard nach SPEC D-111 Abs. 1.
    fn shard_index(&self, slot_type: SlotType, tag: &[u8; 16]) -> usize {
        let (shard, _idx) = slot_probe_start(slot_type, tag, 1);
        (shard as usize) % VAULT_NUM_SHARDS
    }

    /// Speichert einen Slot (ER-04, D-111).
    pub fn put_slot(
        &mut self,
        slot_type: SlotType,
        tag: [u8; 16],
        payload: [u8; 32],
    ) -> Result<(), ErasureError> {
        let s_idx = self.shard_index(slot_type, &tag);
        let shard = self.shards.get_mut(s_idx).ok_or(ErasureError::Corrupt)?;
        let sd = SlotData {
            slot_type,
            tag,
            payload,
            gen: 0,
        };
        shard.put(sd)
    }

    /// Liest einen Slot (ER-02, D-111).
    pub fn get_slot(
        &self,
        slot_type: SlotType,
        tag: [u8; 16],
    ) -> Result<Option<SlotData>, ErasureError> {
        let s_idx = self.shard_index(slot_type, &tag);
        let shard = self.shards.get(s_idx).ok_or(ErasureError::Corrupt)?;
        shard.get(slot_type, &tag)
    }

    /// Vernichtet einen Slot (D-111 Abs. 3, I-23):
    /// Schreibt Grabstein-Abbild in einem 64-Byte-Vorgang, sync, und verifiziert.
    pub fn destroy_slot(
        &mut self,
        slot_type: SlotType,
        tag: [u8; 16],
    ) -> Result<Destroyed, ErasureError> {
        let s_idx = self.shard_index(slot_type, &tag);
        let shard = self.shards.get_mut(s_idx).ok_or(ErasureError::Corrupt)?;
        shard.destroy(slot_type, &tag)
    }

    /// Prüft ob die Lastgrenze in mindestens einem Shard überschritten ist (λ > 0,90).
    pub fn load_exceeded(&self) -> bool {
        self.shards.iter().any(|s| s.load_exceeded())
    }

    /// Gesamtzahl aktiver Slots über alle 16 Shards.
    pub fn live_count(&self) -> usize {
        self.shards.iter().map(|s| s.live_count()).sum()
    }

    /// Gesamtzahl Grabsteine über alle 16 Shards.
    pub fn tombstone_count(&self) -> usize {
        self.shards.iter().map(|s| s.tombstone_count()).sum()
    }

    /// Gesamtkapazität über alle 16 Shards.
    pub fn capacity(&self) -> usize {
        self.shards.iter().map(|s| s.capacity()).sum()
    }

    /// Zugriff auf einzelnen Shard.
    pub fn shard(&self, index: usize) -> Option<&VaultTable> {
        self.shards.get(index)
    }

    /// Veränderlicher Zugriff auf einzelnen Shard.
    pub fn shard_mut(&mut self, index: usize) -> Option<&mut VaultTable> {
        self.shards.get_mut(index)
    }

    /// Vault-ID.
    pub fn vault_id(&self) -> &[u8; 16] {
        &self.vault_id
    }

    /// MAC-Schlüssel.
    pub fn mac_key(&self) -> &[u8; 32] {
        &self.mac_key
    }
}
