//! Tresor-Modul (SPEC §11.5, D-111, D-112, D-125).
//!
//! Enthält `FileVault`, die 16-Shard-Fassade über `VaultTable` und `DurableVaultShard`.

pub mod durable;
pub mod keyring;
pub mod open;
pub mod rehash;
pub mod scrub;
pub mod sealer;
pub mod slot;
pub mod table;

use crate::error::ErasureError;
use durable::DurableVaultShard;
use slot::SlotData;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use table::VaultTable;

pub use durable::VAULT_SLOT_BASE;
pub use keyring::{file_id_to_tag, mask_file_key, InMemoryKeyring, VaultKeyring};
pub use open::{
    compute_metablock_mac, encode_metablock, open_vault, open_vault_shard, MetablockParams,
    VaultOpenOutcome, VAULT_METABLOCK_SIZE,
};
pub use rehash::{rehash, rehash_durable_shard, rehash_shard, RehashOutcome};
pub use scrub::{scrub_shard_step, ScrubOutcome, VaultScrubContext};
pub use sealer::VaultSealer;
pub use slot::{
    compute_header_tag, compute_slot_mac, decode_slot, encode_slot, encode_slot_body,
    encode_vault_header, mask_epoch_key, slot_probe_start, tombstone_image, verify_vault_header,
    SlotData as VaultSlotData, SlotType as VaultSlotType, SlotType, CTX_FILE_MAGIC,
    CTX_FORMAT_VERSION, STATUS_ACTIVE, STATUS_EMPTY, STATUS_TOMBSTONE, VAULT_ALG_IDS,
    VAULT_FILE_KIND, VAULT_HEADER_SIZE, VAULT_SLOT_SIZE,
};
pub use table::{Destroyed, VaultTable as VaultShard, VerifiedVia};

/// Anzahl Shards im Tresor nach SPEC D-111 Abs. 1.
pub const VAULT_NUM_SHARDS: usize = 16;

/// Backend-Speicher eines Shards (entweder persistent über VFS oder im Arbeitsspeicher).
pub enum ShardBackend {
    /// In-Memory Shard-Tabelle.
    InMemory(VaultTable),
    /// Dauerhafter Shard über VFS.
    Durable(DurableVaultShard),
}

/// 16-Shard Tresor nach SPEC D-111.
pub struct FileVault {
    shards: Vec<ShardBackend>,
    vault_id: [u8; 16],
    mac_key: [u8; 32],
    vfs: Option<Arc<dyn crate::types_local::Vfs>>,
    vault_dir: Option<PathBuf>,
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
            shards.push(ShardBackend::InMemory(VaultTable::new_shard(
                u8::try_from(i).unwrap_or(0),
                shard_cap,
                vault_id,
                mac_key,
            )));
        }
        Self {
            shards,
            vault_id,
            mac_key,
            vfs: None,
            vault_dir: None,
        }
    }

    /// Berechnet den Shard-Dateinamen `<nn>.vlt` für Shard-Index `0..15`.
    pub fn shard_file_name(shard_no: u8) -> String {
        format!("{shard_no:02x}.vlt")
    }

    /// Berechnet die Anfangskapazität je Shard nach SPEC D-111 Abs. 7:
    /// `nextpow2(planned_docs * 1.3 / 16 / 0.5).max(1024)`
    pub fn initial_shard_capacity(planned_docs: usize) -> usize {
        let val = ((planned_docs as f64) * 1.3) / 16.0 / 0.5;
        let p2 = (val as usize).next_power_of_two();
        p2.max(1024)
    }

    /// Erstellt einen neuen dauerhaften Tresor mit 16 Shards in `vault_dir` über VFS (SPEC D-111, D-112).
    pub fn create_dir(
        vfs: &Arc<dyn crate::types_local::Vfs>,
        vault_dir: &Path,
        vault_id: [u8; 16],
        mac_key: [u8; 32],
        planned_docs: usize,
        created_seq: u64,
    ) -> Result<Self, ErasureError> {
        vfs.create_dir_all(vault_dir).map_err(|e| match e {
            crate::error::ErasureError::Io(io_err) => ErasureError::Io(io_err),
            other => ErasureError::Io(std::io::Error::other(other.to_string())),
        })?;

        let shard_cap = Self::initial_shard_capacity(planned_docs);
        let mut shards = Vec::with_capacity(VAULT_NUM_SHARDS);

        for i in 0..VAULT_NUM_SHARDS {
            let shard_no = u8::try_from(i).unwrap_or(0);
            let fname = Self::shard_file_name(shard_no);
            let shard_path = vault_dir.join(fname);

            let shard = DurableVaultShard::create_file(
                vfs,
                shard_path,
                shard_no,
                shard_cap,
                vault_id,
                mac_key,
                created_seq,
            )?;
            shards.push(ShardBackend::Durable(shard));
        }

        vfs.sync_dir(vault_dir).map_err(|e| match e {
            crate::error::ErasureError::Io(io_err) => ErasureError::Io(io_err),
            other => ErasureError::Io(std::io::Error::other(other.to_string())),
        })?;

        Ok(Self {
            shards,
            vault_id,
            mac_key,
            vfs: Some(Arc::clone(vfs)),
            vault_dir: Some(vault_dir.to_path_buf()),
        })
    }

    /// Öffnet einen bestehenden dauerhaften Tresor über VFS nach SPEC D-111 Abs. 8 / D-112:
    pub fn open_dir(
        vfs: &Arc<dyn crate::types_local::Vfs>,
        vault_dir: &Path,
        expected_vault_id: &[u8; 16],
        mac_key: &[u8; 32],
    ) -> Result<Self, ErasureError> {
        let mut shards = Vec::with_capacity(VAULT_NUM_SHARDS);

        for i in 0..VAULT_NUM_SHARDS {
            let shard_no = u8::try_from(i).unwrap_or(0);
            let fname = Self::shard_file_name(shard_no);
            let shard_path = vault_dir.join(fname);

            let shard = DurableVaultShard::open_file(
                vfs,
                shard_path,
                shard_no,
                expected_vault_id,
                mac_key,
            )?;
            shards.push(ShardBackend::Durable(shard));
        }

        Ok(Self {
            shards,
            vault_id: *expected_vault_id,
            mac_key: *mac_key,
            vfs: Some(Arc::clone(vfs)),
            vault_dir: Some(vault_dir.to_path_buf()),
        })
    }

    /// Ermittelt den zuständigen Shard nach SPEC D-111 Abs. 1.
    pub fn shard_index(&self, slot_type: SlotType, tag: &[u8; 16]) -> usize {
        let (shard, _idx) = slot_probe_start(slot_type, tag, 1);
        (shard as usize) % VAULT_NUM_SHARDS
    }

    /// Speichert einen Slot (ER-04, D-111, I-9).
    pub fn put_slot(
        &mut self,
        slot_type: SlotType,
        tag: [u8; 16],
        payload: [u8; 32],
    ) -> Result<(), ErasureError> {
        self.put_slot_with_gen(slot_type, tag, payload, 0)
    }

    /// Speichert einen Slot mit explizitem Generationszähler (u48).
    pub fn put_slot_with_gen(
        &mut self,
        slot_type: SlotType,
        tag: [u8; 16],
        payload: [u8; 32],
        gen: u64,
    ) -> Result<(), ErasureError> {
        let s_idx = self.shard_index(slot_type, &tag);
        let shard = self.shards.get_mut(s_idx).ok_or(ErasureError::Corrupt)?;
        let sd = SlotData {
            slot_type,
            tag,
            payload,
            gen,
        };
        match shard {
            ShardBackend::InMemory(t) => t.put(sd),
            ShardBackend::Durable(d) => d.put(sd),
        }
    }

    /// Liest einen Slot (ER-02, D-111).
    pub fn get_slot(
        &self,
        slot_type: SlotType,
        tag: [u8; 16],
    ) -> Result<Option<SlotData>, ErasureError> {
        let s_idx = self.shard_index(slot_type, &tag);
        let shard = self.shards.get(s_idx).ok_or(ErasureError::Corrupt)?;
        match shard {
            ShardBackend::InMemory(t) => t.get(slot_type, &tag),
            ShardBackend::Durable(d) => d.get(slot_type, &tag),
        }
    }

    /// Vernichtet einen Slot (D-111 Abs. 3, D-113, I-23):
    pub fn destroy_slot(
        &mut self,
        slot_type: SlotType,
        tag: [u8; 16],
    ) -> Result<Destroyed, ErasureError> {
        let s_idx = self.shard_index(slot_type, &tag);
        let shard = self.shards.get_mut(s_idx).ok_or(ErasureError::Corrupt)?;
        match shard {
            ShardBackend::InMemory(t) => t.destroy(slot_type, &tag),
            ShardBackend::Durable(d) => d.destroy(slot_type, &tag),
        }
    }

    /// Führt Gruppen-Sync für alle Shards aus (D-111 Abs. 9, I-9).
    pub fn sync_all_shards(&self) -> Result<(), ErasureError> {
        for shard in &self.shards {
            if let ShardBackend::Durable(d) = shard {
                d.sync_data()?;
            }
        }
        Ok(())
    }

    /// Prüft ob die Lastgrenze in mindestens einem Shard überschritten ist (λ > 0,90).
    pub fn load_exceeded(&self) -> bool {
        self.shards.iter().any(|s| match s {
            ShardBackend::InMemory(t) => t.load_exceeded(),
            ShardBackend::Durable(d) => d.load_exceeded(),
        })
    }

    /// Gesamtzahl aktiver Slots über alle 16 Shards.
    pub fn live_count(&self) -> usize {
        self.shards
            .iter()
            .map(|s| match s {
                ShardBackend::InMemory(t) => t.live_count(),
                ShardBackend::Durable(d) => d.live_count(),
            })
            .sum()
    }

    /// Gesamtzahl Grabsteine über alle 16 Shards.
    pub fn tombstone_count(&self) -> usize {
        self.shards
            .iter()
            .map(|s| match s {
                ShardBackend::InMemory(t) => t.tombstone_count(),
                ShardBackend::Durable(d) => d.tombstone_count(),
            })
            .sum()
    }

    /// Gesamtkapazität über alle 16 Shards.
    pub fn capacity(&self) -> usize {
        self.shards
            .iter()
            .map(|s| match s {
                ShardBackend::InMemory(t) => t.capacity(),
                ShardBackend::Durable(d) => d.capacity(),
            })
            .sum()
    }

    /// Prüft ob mindestens ein Shard vergiftet ist (D-55, I-10).
    pub fn is_poisoned(&self) -> bool {
        self.shards.iter().any(|s| match s {
            ShardBackend::InMemory(t) => t.is_poisoned(),
            ShardBackend::Durable(d) => d.is_poisoned(),
        })
    }

    /// Zugriff auf einzelnen Shard (In-Memory).
    pub fn shard(&self, index: usize) -> Option<&VaultTable> {
        match self.shards.get(index)? {
            ShardBackend::InMemory(t) => Some(t),
            ShardBackend::Durable(_) => None,
        }
    }

    /// Veränderlicher Zugriff auf einzelnen Shard (In-Memory).
    pub fn shard_mut(&mut self, index: usize) -> Option<&mut VaultTable> {
        match self.shards.get_mut(index)? {
            ShardBackend::InMemory(t) => Some(t),
            ShardBackend::Durable(_) => None,
        }
    }

    /// Zugriff auf dauerhaften Shard.
    pub fn durable_shard(&self, index: usize) -> Option<&DurableVaultShard> {
        match self.shards.get(index)? {
            ShardBackend::Durable(d) => Some(d),
            ShardBackend::InMemory(_) => None,
        }
    }

    /// Veränderlicher Zugriff auf dauerhaften Shard.
    pub fn durable_shard_mut(&mut self, index: usize) -> Option<&mut DurableVaultShard> {
        match self.shards.get_mut(index)? {
            ShardBackend::Durable(d) => Some(d),
            ShardBackend::InMemory(_) => None,
        }
    }

    /// Ersetzt einen dauerhaften Shard (z.B. nach Rehash).
    pub fn replace_durable_shard(&mut self, index: usize, shard: DurableVaultShard) {
        if let Some(slot) = self.shards.get_mut(index) {
            *slot = ShardBackend::Durable(shard);
        }
    }

    /// Vault-ID.
    pub fn vault_id(&self) -> &[u8; 16] {
        &self.vault_id
    }

    /// MAC-Schlüssel.
    pub fn mac_key(&self) -> &[u8; 32] {
        &self.mac_key
    }

    /// Vault-Verzeichnis (falls dauerhaft).
    pub fn vault_dir(&self) -> Option<&Path> {
        self.vault_dir.as_deref()
    }

    /// Vfs-Instanz (falls dauerhaft).
    pub fn vfs(&self) -> Option<&Arc<dyn crate::types_local::Vfs>> {
        self.vfs.as_ref()
    }
}
