//! Dauerhafter Tresor-Shard über VFS nach SPEC §11.5, D-55, D-111, D-113, I-10, I-23.

use crate::error::ErasureError;
use crate::vault::open::{encode_metablock, MetablockParams, VAULT_METABLOCK_SIZE};
use crate::vault::slot::{
    compute_slot_mac, decode_slot, encode_slot_body, encode_vault_header, slot_probe_start,
    tombstone_image, verify_vault_header, SlotData, SlotType, STATUS_ACTIVE, STATUS_EMPTY,
    STATUS_TOMBSTONE, VAULT_HEADER_SIZE, VAULT_SLOT_SIZE,
};
pub use contextra_ports::vfs::VerifiedVia;
use contextra_ports::vfs::{Vfs, VfsFile, VfsOpenOptions};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// Basis-Offset der Slots in einer Shard-Datei:
/// 64 Bytes F.1-Kopf + 64 Bytes Metablock = Offset 128 (SPEC F.3).
pub const VAULT_SLOT_BASE: usize = 128;

/// Dauerhafter Tresor-Shard über VFS.
pub struct DurableVaultShard {
    file: Box<dyn VfsFile>,
    path: PathBuf,
    shard_no: u8,
    cap_log2: u8,
    capacity: usize,
    live: u64,
    tombstones: u64,
    generation: u64,
    rsv_cursor: u64,
    vault_id: [u8; 16],
    mac_key: [u8; 32],
    poisoned: AtomicBool,
}

impl DurableVaultShard {
    /// Erstellt eine neue dauerhafte Shard-Datei über VFS.
    pub fn create_file(
        vfs: &Arc<dyn Vfs>,
        path: PathBuf,
        shard_no: u8,
        capacity: usize,
        vault_id: [u8; 16],
        mac_key: [u8; 32],
        created_seq: u64,
    ) -> Result<Self, ErasureError> {
        let cap_log2 = u8::try_from(capacity.trailing_zeros()).unwrap_or(10);
        let real_capacity = 1usize
            .checked_shl(cap_log2 as u32)
            .ok_or(ErasureError::Capacity)?;

        let open_opts = VfsOpenOptions {
            read: true,
            write: true,
            create: true,
            create_new: true,
            truncate: false,
            append: false,
        };

        let file = vfs.open(&path, &open_opts).map_err(|e| match e {
            contextra_ports::ContextraError::Io(io_err) => ErasureError::Io(io_err),
            other => ErasureError::Io(std::io::Error::other(other.to_string())),
        })?;

        // 1. Dateikopf schreiben (64 Bytes)
        let header_buf = encode_vault_header(&vault_id, created_seq, &mac_key);
        file.write_at(&header_buf, 0).map_err(ErasureError::Io)?;

        // 2. Metablock schreiben (64 Bytes bei Offset 64)
        let meta_buf = encode_metablock(MetablockParams {
            vault_id: &vault_id,
            shard_no,
            cap_log2,
            live: 0,
            tombstones: 0,
            generation: 0,
            scrub_cursor: 0,
            mac_key: &mac_key,
        });
        file.write_at(&meta_buf, 64).map_err(ErasureError::Io)?;

        // 3. Leere Slots vorallozieren (real_capacity * 64 Bytes ab Offset 128)
        let total_bytes = (VAULT_SLOT_BASE as u64)
            .checked_add(
                (real_capacity as u64)
                    .checked_mul(VAULT_SLOT_SIZE as u64)
                    .ok_or(ErasureError::Capacity)?,
            )
            .ok_or(ErasureError::Capacity)?;

        file.set_len(total_bytes).map_err(ErasureError::Io)?;

        // 4. Dauerhaft festschreiben via sync_data (D-111, D-55)
        file.sync_data().map_err(ErasureError::Io)?;

        Ok(Self {
            file,
            path,
            shard_no,
            cap_log2,
            capacity: real_capacity,
            live: 0,
            tombstones: 0,
            generation: 0,
            rsv_cursor: 0,
            vault_id,
            mac_key,
            poisoned: AtomicBool::new(false),
        })
    }

    /// Öffnet eine bestehende dauerhafte Shard-Datei über VFS.
    pub fn open_file(
        vfs: &Arc<dyn Vfs>,
        path: PathBuf,
        expected_shard_no: u8,
        expected_vault_id: &[u8; 16],
        mac_key: &[u8; 32],
    ) -> Result<Self, ErasureError> {
        let open_opts = VfsOpenOptions {
            read: true,
            write: true,
            create: false,
            create_new: false,
            truncate: false,
            append: false,
        };

        let file = vfs.open(&path, &open_opts).map_err(|e| match e {
            contextra_ports::ContextraError::Io(io_err) => ErasureError::Io(io_err),
            other => ErasureError::Io(std::io::Error::other(other.to_string())),
        })?;

        // 1. Kopf lesen und prüfen (Offset 0..64)
        let mut header_buf = [0u8; VAULT_HEADER_SIZE];
        let n_hdr = file.read_at(&mut header_buf, 0).map_err(ErasureError::Io)?;
        if n_hdr < VAULT_HEADER_SIZE {
            return Err(ErasureError::Corrupt);
        }
        let (_hdr_file_id, _created_seq) = verify_vault_header(&header_buf, mac_key)?;

        // 2. Metablock lesen und prüfen (Offset 64..128)
        let mut meta_buf = [0u8; VAULT_METABLOCK_SIZE];
        let n_meta = file.read_at(&mut meta_buf, 64).map_err(ErasureError::Io)?;
        if n_meta < VAULT_METABLOCK_SIZE {
            return Err(ErasureError::Corrupt);
        }

        let pre = meta_buf.get(0..50).ok_or(ErasureError::Corrupt)?;
        let expected_mac = crate::vault::open::compute_metablock_mac(mac_key, pre);
        let actual_mac = meta_buf.get(50..58).ok_or(ErasureError::Corrupt)?;
        if actual_mac != expected_mac {
            return Err(ErasureError::Corrupt);
        }

        let pad = meta_buf.get(58..64).ok_or(ErasureError::Corrupt)?;
        if pad.iter().any(|&b| b != 0) {
            return Err(ErasureError::Corrupt);
        }

        let mut vault_id = [0u8; 16];
        if let (Some(dest), Some(src)) = (vault_id.get_mut(..), meta_buf.get(0..16)) {
            dest.copy_from_slice(src);
        }
        if &vault_id != expected_vault_id {
            return Err(ErasureError::Corrupt);
        }

        let shard_no = meta_buf.get(16).copied().unwrap_or(0);
        if shard_no != expected_shard_no {
            return Err(ErasureError::Corrupt);
        }

        let cap_log2 = meta_buf.get(17).copied().unwrap_or(0);
        if cap_log2 > 40 {
            return Err(ErasureError::Corrupt);
        }
        let capacity = 1usize
            .checked_shl(cap_log2 as u32)
            .ok_or(ErasureError::Corrupt)?;

        let mut live_bytes = [0u8; 8];
        if let (Some(dest), Some(src)) = (live_bytes.get_mut(..), meta_buf.get(18..26)) {
            dest.copy_from_slice(src);
        }
        let live = u64::from_be_bytes(live_bytes);

        let mut tomb_bytes = [0u8; 8];
        if let (Some(dest), Some(src)) = (tomb_bytes.get_mut(..), meta_buf.get(26..34)) {
            dest.copy_from_slice(src);
        }
        let tombstones = u64::from_be_bytes(tomb_bytes);

        let mut gen_bytes = [0u8; 8];
        if let (Some(dest), Some(src)) = (gen_bytes.get_mut(..), meta_buf.get(34..42)) {
            dest.copy_from_slice(src);
        }
        let generation = u64::from_be_bytes(gen_bytes);

        let mut rsv_bytes = [0u8; 8];
        if let (Some(dest), Some(src)) = (rsv_bytes.get_mut(..), meta_buf.get(42..50)) {
            dest.copy_from_slice(src);
        }
        let rsv_cursor = u64::from_be_bytes(rsv_bytes);

        // 3. Stichprobe von Slots prüfen (D-111 Abs. 8: bis zu 256 Slots prüfen)
        let samples = capacity.min(256);
        let step = (capacity / samples).max(1);
        let mut slot_buf = [0u8; VAULT_SLOT_SIZE];

        for s in 0..samples {
            let slot_idx = (s.saturating_mul(step)) % capacity;
            let off = (VAULT_SLOT_BASE as u64)
                .checked_add(
                    (slot_idx as u64)
                        .checked_mul(VAULT_SLOT_SIZE as u64)
                        .ok_or(ErasureError::Corrupt)?,
                )
                .ok_or(ErasureError::Corrupt)?;
            let n = file.read_at(&mut slot_buf, off).map_err(ErasureError::Io)?;
            if n == VAULT_SLOT_SIZE {
                decode_slot(&slot_buf, &vault_id, mac_key)?;
            }
        }

        Ok(Self {
            file,
            path,
            shard_no,
            cap_log2,
            capacity,
            live,
            tombstones,
            generation,
            rsv_cursor,
            vault_id,
            mac_key: *mac_key,
            poisoned: AtomicBool::new(false),
        })
    }

    /// Prüft ob der Shard vergiftet ist (D-55, I-10).
    pub fn is_poisoned(&self) -> bool {
        self.poisoned.load(Ordering::Acquire)
    }

    /// Vergiftet den Shard irreversibel (D-55, I-10).
    pub fn poison(&self) {
        self.poisoned.store(true, Ordering::Release);
    }

    /// Hilfsfunktion zur Fehlerprüfung auf Vergiftung.
    fn check_poison(&self) -> Result<(), ErasureError> {
        if self.is_poisoned() {
            Err(ErasureError::Io(std::io::Error::other(
                "vault shard is poisoned due to previous sync or I/O failure (D-55)",
            )))
        } else {
            Ok(())
        }
    }

    /// Shard-Pfad.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Setzt den Pfad.
    pub fn set_path(&mut self, path: PathBuf) {
        self.path = path;
    }

    /// Shard-Nummer (0..15).
    pub fn shard_no(&self) -> u8 {
        self.shard_no
    }

    /// Shard-Kapazität.
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Geschätzte Live-Anzahl laut Metablock.
    pub fn live_count(&self) -> usize {
        usize::try_from(self.live).unwrap_or(0)
    }

    /// Geschätzte Tombstone-Anzahl laut Metablock.
    pub fn tombstone_count(&self) -> usize {
        usize::try_from(self.tombstones).unwrap_or(0)
    }

    /// Shard-Generation.
    pub fn generation(&self) -> u64 {
        self.generation
    }

    /// Setzt die Shard-Generation.
    pub fn set_generation(&mut self, gen: u64) {
        self.generation = gen;
    }

    /// Scrub-Cursor (`rsv`).
    pub fn rsv_cursor(&self) -> u64 {
        self.rsv_cursor
    }

    /// Setzt den Scrub-Cursor (`rsv`).
    pub fn set_rsv_cursor(&mut self, cursor: u64) {
        self.rsv_cursor = cursor;
    }

    /// Vault-ID Referenz.
    pub fn vault_id(&self) -> &[u8; 16] {
        &self.vault_id
    }

    /// MAC-Schlüssel Referenz.
    pub fn mac_key_ref(&self) -> &[u8; 32] {
        &self.mac_key
    }

    /// Aktueller Lastfaktor λ = (live + tombstones) / capacity.
    pub fn load_factor(&self) -> f64 {
        if self.capacity == 0 {
            return 1.0;
        }
        (self.live.saturating_add(self.tombstones) as f64) / (self.capacity as f64)
    }

    /// Prüft ob Rehash im Hintergrund ausgelöst werden soll (λ > 0,60).
    pub fn should_background_rehash(&self) -> bool {
        self.load_factor() > 0.60
    }

    /// Prüft ob Rehash erzwungen werden muss (λ > 0,80).
    pub fn must_force_rehash(&self) -> bool {
        self.load_factor() > 0.80
    }

    /// Prüft ob Lastfaktor mit nächstem Insert > 0,90 wäre (D-111 Abs. 4).
    pub fn load_exceeded(&self) -> bool {
        if self.capacity == 0 {
            return true;
        }
        let next = self.live.saturating_add(self.tombstones).saturating_add(1);
        (next as f64) / (self.capacity as f64) > 0.90
    }

    /// Liest rohe 64-Byte Slot-Bytes an Index `idx`.
    pub fn read_slot_raw_pub(&self, idx: usize) -> Result<[u8; VAULT_SLOT_SIZE], ErasureError> {
        self.read_slot_raw(idx)
    }

    /// Interne Hilfsfunktion für Slot-Lesen.
    fn read_slot_raw(&self, idx: usize) -> Result<[u8; VAULT_SLOT_SIZE], ErasureError> {
        let off = (VAULT_SLOT_BASE as u64)
            .checked_add(
                (idx as u64)
                    .checked_mul(VAULT_SLOT_SIZE as u64)
                    .ok_or(ErasureError::Corrupt)?,
            )
            .ok_or(ErasureError::Corrupt)?;
        let mut buf = [0u8; VAULT_SLOT_SIZE];
        let n = self.file.read_at(&mut buf, off).map_err(|e| {
            self.poison();
            ErasureError::Io(e)
        })?;
        if n < VAULT_SLOT_SIZE {
            return Err(ErasureError::Corrupt);
        }
        Ok(buf)
    }

    /// Findet den Slot-Index für (slot_type, tag) durch lineare Sondierung.
    pub fn find(&self, slot_type: SlotType, tag: &[u8; 16]) -> Result<Option<usize>, ErasureError> {
        self.check_poison()?;
        let (_shard, start) = slot_probe_start(slot_type, tag, self.capacity);
        for i in 0..self.capacity {
            let idx = (start.saturating_add(i)) % self.capacity;
            let bytes = self.read_slot_raw(idx)?;
            let status = bytes
                .get(super::slot::off::STATUS)
                .copied()
                .unwrap_or(STATUS_EMPTY);
            match status {
                STATUS_EMPTY => return Ok(None),
                STATUS_TOMBSTONE => {
                    // Sondierung fortsetzen
                }
                STATUS_ACTIVE => {
                    let st = bytes.get(super::slot::off::SLOT_TYPE).copied().unwrap_or(0);
                    let tag_slice = bytes.get(super::slot::off::TAG..super::slot::off::PAYLOAD);
                    if st == slot_type as u8 && tag_slice == Some(tag.as_slice()) {
                        return Ok(Some(idx));
                    }
                }
                _ => return Err(ErasureError::Corrupt),
            }
        }
        Ok(None)
    }

    /// Liest und decodiert einen Slot.
    pub fn get(
        &self,
        slot_type: SlotType,
        tag: &[u8; 16],
    ) -> Result<Option<SlotData>, ErasureError> {
        self.check_poison()?;
        match self.find(slot_type, tag)? {
            None => Ok(None),
            Some(idx) => {
                let bytes = self.read_slot_raw(idx)?;
                decode_slot(&bytes, &self.vault_id, &self.mac_key)
            }
        }
    }

    /// Schreibt einen Slot dauerhaft via VFS (genau ein 64-Byte-Schreibvorgang, I-23).
    pub fn put(&mut self, sd: SlotData) -> Result<(), ErasureError> {
        self.check_poison()?;
        if self.load_exceeded() {
            return Err(ErasureError::Capacity);
        }

        let (_shard, start) = slot_probe_start(sd.slot_type, &sd.tag, self.capacity);
        for i in 0..self.capacity {
            let idx = (start.saturating_add(i)) % self.capacity;
            let bytes = self.read_slot_raw(idx)?;
            let status = bytes
                .get(super::slot::off::STATUS)
                .copied()
                .unwrap_or(STATUS_EMPTY);

            if status == STATUS_EMPTY || status == STATUS_TOMBSTONE {
                if status == STATUS_TOMBSTONE {
                    self.tombstones = self.tombstones.saturating_sub(1);
                }

                // Slot-Daten vorbereiten
                let mut buf = encode_slot_body(&sd);
                let pre = buf
                    .get(..super::slot::off::MAC)
                    .ok_or(ErasureError::Corrupt)?;
                let mac = compute_slot_mac(&self.mac_key, &self.vault_id, pre);
                let mac_dest = buf
                    .get_mut(super::slot::off::MAC..super::slot::off::MAC + 8)
                    .ok_or(ErasureError::Corrupt)?;
                mac_dest.copy_from_slice(&mac);

                let off = (VAULT_SLOT_BASE as u64)
                    .checked_add(
                        (idx as u64)
                            .checked_mul(VAULT_SLOT_SIZE as u64)
                            .ok_or(ErasureError::Corrupt)?,
                    )
                    .ok_or(ErasureError::Corrupt)?;

                // Genau EIN 64-Byte Schreibvorgang an 64-ausgerichtetem Offset (I-23)
                if let Err(e) = self.file.write_at(&buf, off) {
                    self.poison();
                    return Err(ErasureError::Io(e));
                }

                self.live = self.live.saturating_add(1);

                // Metablock-Zähler aktualisieren und sync
                self.flush_meta()?;
                self.sync_data()?;

                return Ok(());
            }
        }
        Err(ErasureError::Capacity)
    }

    /// Vernichtet einen Slot nach SPEC D-111 Abs. 3, D-113 und I-23:
    /// 1. Schreibt in EINEM 64-Byte-Schreibvorgang das Grabstein-Abbild (tombstone_image)
    /// 2. sync_data
    /// 3. read_verified Verifikation: Bytes 0 und 2..56 sind 0, Byte 1 ist 0x02, MAC ist gültig.
    /// Bei Fehler: Shard irreversibel poisoned (D-55, I-10).
    pub fn destroy(
        &mut self,
        slot_type: SlotType,
        tag: &[u8; 16],
    ) -> Result<super::table::Destroyed, ErasureError> {
        self.check_poison()?;
        let idx = match self.find(slot_type, tag)? {
            Some(i) => i,
            None => return Err(ErasureError::SlotMissing),
        };

        let off = (VAULT_SLOT_BASE as u64)
            .checked_add(
                (idx as u64)
                    .checked_mul(VAULT_SLOT_SIZE as u64)
                    .ok_or(ErasureError::Corrupt)?,
            )
            .ok_or(ErasureError::Corrupt)?;

        // 1. Grabstein-Abbild erzeugen (64 Bytes, status = 0x02)
        let tombstone = tombstone_image(&self.vault_id, &self.mac_key);

        // Genau EIN 64-Byte Schreibvorgang
        if let Err(e) = self.file.write_at(&tombstone, off) {
            self.poison();
            return Err(ErasureError::Io(e));
        }

        self.live = self.live.saturating_sub(1);
        self.tombstones = self.tombstones.saturating_add(1);

        // Metablock aktualisieren und sync
        if let Err(e) = self.flush_meta() {
            self.poison();
            return Err(e);
        }

        if let Err(e) = self.file.sync_data() {
            self.poison();
            return Err(ErasureError::Io(e));
        }

        // 2. Verifikation über read_verified (D-111 Abs. 3, D-113)
        let (verified_bytes, verified_via) = match self.file.read_verified(off, VAULT_SLOT_SIZE) {
            Ok(res) => res,
            Err(e) => {
                self.poison();
                return Err(ErasureError::Io(e));
            }
        };

        if verified_bytes.len() < VAULT_SLOT_SIZE {
            self.poison();
            return Err(ErasureError::Corrupt);
        }

        // Byte 0 muss 0 sein
        if verified_bytes.first().copied() != Some(0) {
            self.poison();
            return Err(ErasureError::Corrupt);
        }
        // Byte 1 muss STATUS_TOMBSTONE (0x02) sein
        if verified_bytes.get(1).copied() != Some(STATUS_TOMBSTONE) {
            self.poison();
            return Err(ErasureError::Corrupt);
        }
        // Bytes 2..56 müssen 0 sein
        let body = verified_bytes.get(2..56).ok_or_else(|| {
            self.poison();
            ErasureError::Corrupt
        })?;
        if body.iter().any(|&b| b != 0) {
            self.poison();
            return Err(ErasureError::Corrupt);
        }
        // MAC prüfen
        let pre = verified_bytes.get(..56).ok_or_else(|| {
            self.poison();
            ErasureError::Corrupt
        })?;
        let expected_mac = compute_slot_mac(&self.mac_key, &self.vault_id, pre);
        let actual_mac = verified_bytes.get(56..64).ok_or_else(|| {
            self.poison();
            ErasureError::Corrupt
        })?;
        if actual_mac != expected_mac {
            self.poison();
            return Err(ErasureError::Corrupt);
        }

        Ok(super::table::Destroyed {
            verified_via: match verified_via {
                contextra_ports::vfs::VerifiedVia::Direct => {
                    crate::vault::table::VerifiedVia::Direct
                }
                contextra_ports::vfs::VerifiedVia::CacheDropped => {
                    crate::vault::table::VerifiedVia::CacheDropped
                }
                contextra_ports::vfs::VerifiedVia::Cached => {
                    crate::vault::table::VerifiedVia::Cached
                }
            },
        })
    }

    /// Aktualisiert den Metablock auf Platte.
    pub fn flush_meta(&self) -> Result<(), ErasureError> {
        self.check_poison()?;
        let meta_buf = encode_metablock(MetablockParams {
            vault_id: &self.vault_id,
            shard_no: self.shard_no,
            cap_log2: self.cap_log2,
            live: self.live,
            tombstones: self.tombstones,
            generation: self.generation,
            scrub_cursor: self.rsv_cursor,
            mac_key: &self.mac_key,
        });
        if let Err(e) = self.file.write_at(&meta_buf, 64) {
            self.poison();
            return Err(ErasureError::Io(e));
        }
        Ok(())
    }

    /// Synchronisiert die Datenblöcke der Datei (`sync_data`).
    /// Bei Fehler: Shard wird vergiftet (D-55, I-10).
    pub fn sync_data(&self) -> Result<(), ErasureError> {
        self.check_poison()?;
        if let Err(e) = self.file.sync_data() {
            self.poison();
            return Err(ErasureError::Io(e));
        }
        Ok(())
    }

    /// Sammelt alle aktiven Slots für den Rehash (Grabsteine werden nicht übernommen, D-111 Abs. 5).
    pub fn collect_active_slots(&self) -> Result<Vec<SlotData>, ErasureError> {
        self.check_poison()?;
        let mut list = Vec::with_capacity(usize::try_from(self.live).unwrap_or(0));
        for idx in 0..self.capacity {
            let bytes = self.read_slot_raw(idx)?;
            if bytes.get(super::slot::off::STATUS).copied() == Some(STATUS_ACTIVE) {
                if let Some(sd) = decode_slot(&bytes, &self.vault_id, &self.mac_key)? {
                    list.push(sd);
                }
            }
        }
        Ok(list)
    }

    /// Überschreibt die Shard-Datei über das gehaltene Handle vollständig mit Nullen (D-111 Abs. 5).
    pub fn overwrite_with_zeros(&mut self) -> Result<(), ErasureError> {
        self.check_poison()?;
        let len = self.file.len().map_err(|e| {
            self.poison();
            ErasureError::Io(e)
        })?;
        let zero_buf = [0u8; 4096];
        let mut written = 0u64;
        while written < len {
            let to_write = usize::try_from((len - written).min(4096)).unwrap_or(4096);
            if let Some(slice) = zero_buf.get(..to_write) {
                self.file.write_at(slice, written).map_err(|e| {
                    self.poison();
                    ErasureError::Io(e)
                })?;
                written = written.saturating_add(to_write as u64);
            }
        }
        self.sync_data()?;
        self.live = 0;
        self.tombstones = 0;
        Ok(())
    }
}
