//! Öffnet und faltet eine Catalog-Datei aus dem validen Recordpräfix.

use super::record::{decode_record, encode_record, CatalogIntegrity, MAX_RECORD_BYTES};
use super::state::CatalogState;
use contextra_storage_ports::traits::{File, FileRole, Fs, IoCtx, Mac};
use contextra_storage_ports::{Error, Key32};
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

const IO: IoCtx = IoCtx { role: FileRole::Catalog, purpose: "catalog_open" };
const PREALLOC_BYTES: u64 = 1 << 20;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpenMode { ReadOnly, ReadWrite }

pub struct CatalogFile {
    path: PathBuf,
    file: Box<dyn File>,
    state: CatalogState,
    next_sequence: u64,
    previous_crc: u32,
    previous_mac: [u8; 32],
    append_offset: u64,
    allocated_until: u64,
    writable: bool,
    torn_tail: bool,
}

impl CatalogFile {
    pub fn open_current(
        fs: &dyn Fs,
        root: &Path,
        mode: OpenMode,
        integrity: CatalogIntegrity,
        key: Option<&Key32>,
        mac: Option<&dyn Mac>,
    ) -> Result<Self, Error> {
        match super::rollover::read_current(fs, root) {
            Ok(pointer) => Self::open(fs, &root.join(format!("CATALOG-{}", pointer.generation)), mode, integrity, key, mac),
            Err(Error::Io { kind: ErrorKind::NotFound, .. }) if mode == OpenMode::ReadWrite => {
                let path = root.join("CATALOG-0");
                let catalog = Self::open(fs, &path, mode, integrity, key, mac)?;
                super::rollover::publish_current(fs, root, super::rollover::CurrentPointer { generation: 0 })?;
                Ok(catalog)
            }
            Err(error) => Err(error),
        }
    }

    pub fn open(
        fs: &dyn Fs,
        path: &Path,
        mode: OpenMode,
        integrity: CatalogIntegrity,
        key: Option<&Key32>,
        mac: Option<&dyn Mac>,
    ) -> Result<Self, Error> {
        let (file, created) = match fs.open(path, IO) {
            Ok(file) => (file, false),
            Err(Error::Io { kind: ErrorKind::NotFound, .. }) if mode == OpenMode::ReadWrite => {
                let file = fs.create_new(path, IO)?;
                file.preallocate(PREALLOC_BYTES, IO)?;
                file.sync_all(IO)?;
                if let Some(parent) = path.parent() { fs.sync_dir(parent, IO)?; }
                (file, true)
            }
            Err(error) => return Err(error),
        };
        let mut catalog = Self {
            path: path.to_path_buf(), file, state: CatalogState::default(),
            next_sequence: 1, previous_crc: 0, previous_mac: [0; 32], append_offset: 0,
            allocated_until: if created { PREALLOC_BYTES } else { 0 },
            writable: mode == OpenMode::ReadWrite && created, torn_tail: false,
        };
        if !created { catalog.scan(integrity, key, mac, mode)?; }
        Ok(catalog)
    }

    fn scan(&mut self, integrity: CatalogIntegrity, key: Option<&Key32>, mac: Option<&dyn Mac>, mode: OpenMode) -> Result<(), Error> {
        let file_size = self.file.size()?;
        let mut offset = 0u64;
        while offset < file_size {
            let remaining = file_size - offset;
            if remaining < 4 {
                self.torn_tail = true;
                break;
            }
            let mut len_bytes = [0u8; 4];
            read_exact_at(self.file.as_ref(), offset, &mut len_bytes)?;
            let record_len = u32::from_le_bytes(len_bytes) as usize;
            if record_len == 0 {
                if !zero_tail(self.file.as_ref(), offset, file_size)? {
                    return Err(Error::Corrupt { place: "catalog nonzero bytes after padding" });
                }
                break;
            }
            if record_len > MAX_RECORD_BYTES { return Err(Error::Corrupt { place: "catalog record exceeds limit" }); }
            let total = u64::try_from(record_len.checked_add(4).ok_or(Error::Corrupt { place: "catalog record length" })?)
                .map_err(|_| Error::Corrupt { place: "catalog record length" })?;
            if total > remaining {
                self.torn_tail = true;
                break;
            }
            let mut bytes = vec![0u8; record_len + 4];
            read_exact_at(self.file.as_ref(), offset, &mut bytes)?;
            let (record, used) = decode_record(
                &bytes, self.next_sequence, self.previous_crc, integrity, key, &self.previous_mac, mac,
            )?;
            if used != bytes.len() { return Err(Error::Corrupt { place: "catalog record trailing bytes" }); }
            self.state.apply_record(&record)?;
            self.next_sequence = self.next_sequence.checked_add(1).ok_or(Error::Limit { limit: "catalog sequence" })?;
            self.previous_crc = record.crc;
            self.previous_mac = record.chain_mac.unwrap_or([0; 32]);
            offset = offset.checked_add(total).ok_or(Error::Corrupt { place: "catalog offset" })?;
        }
        self.append_offset = offset;
        self.allocated_until = offset.checked_add(PREALLOC_BYTES - 1)
            .map(|v| (v / PREALLOC_BYTES) * PREALLOC_BYTES)
            .ok_or(Error::Limit { limit: "catalog allocation bound" })?;
        self.writable = mode == OpenMode::ReadWrite && !self.torn_tail;
        Ok(())
    }

    /// Convenience path for one caller; the group writer below is the shared
    /// durability primitive.
    pub fn append_durable(
        &mut self,
        ops: &[super::ops::CatalogOp],
        integrity: CatalogIntegrity,
        key: Option<&Key32>,
        mac: Option<&dyn Mac>,
    ) -> Result<u64, Error> {
        self.append_group_durable(&[ops.to_vec()], integrity, key, mac)?
            .first().copied().ok_or(Error::Corrupt { place: "catalog group result" })
    }

    /// Installs several waiting logical edits in one record, append, and sync.
    /// Operation patterns that must remain coupled are built before submission.
    pub fn append_group_durable(
        &mut self,
        edits: &[Vec<super::ops::CatalogOp>],
        integrity: CatalogIntegrity,
        key: Option<&Key32>,
        mac: Option<&dyn Mac>,
    ) -> Result<Vec<u64>, Error> {
        if !self.writable { return Err(Error::Poisoned { cause: "catalog is read-only or poisoned" }); }
        if edits.is_empty() { return Err(Error::Limit { limit: "catalog edit group must not be empty" }); }
        let next_sequence = self.next_sequence.checked_add(1).ok_or(Error::Limit { limit: "catalog sequence" })?;
        let total_ops = edits.iter().try_fold(0usize, |count, edit| count.checked_add(edit.len()))
            .ok_or(Error::Limit { limit: "catalog group op count" })?;
        if total_ops == 0 || total_ops > 4096 { return Err(Error::Limit { limit: "catalog ops per edit group" }); }
        let ops: Vec<_> = edits.iter().flat_map(|edit| edit.iter().cloned()).collect();
        let mut next_state = self.state.clone();
        let record = super::record::CatalogRecord {
            sequence: self.next_sequence,
            previous_crc: self.previous_crc,
            ops: ops.clone(),
            crc: 0,
            chain_mac: None,
        };
        next_state.apply_record(&record)?;
        let encoded = encode_record(self.next_sequence, self.previous_crc, &ops, integrity, key, &self.previous_mac, mac)?;
        // Validate the exact encoded bytes before the first write. Once the
        // sync below succeeds, there must be no fallible format check left
        // that could leave durable disk state ahead of this handle's state.
        let (decoded, used) = decode_record(
            &encoded, self.next_sequence, self.previous_crc, integrity, key, &self.previous_mac, mac,
        )?;
        if used != encoded.len() {
            return Err(Error::Corrupt { place: "catalog encoded record trailing bytes" });
        }
        let next_offset = self.append_offset.checked_add(encoded.len() as u64)
            .ok_or(Error::Limit { limit: "catalog append offset" })?;
        if next_offset > self.allocated_until {
            let additional = next_offset.checked_add(PREALLOC_BYTES - 1)
                .map(|v| (v / PREALLOC_BYTES) * PREALLOC_BYTES)
                .ok_or(Error::Limit { limit: "catalog preallocation" })?;
            if let Err(error) = self.file.preallocate(additional, IO) {
                if !matches!(error, Error::DiskFull) {
                    self.writable = false;
                    return Err(Error::Poisoned { cause: "catalog preallocation failed" });
                }
                return Err(error);
            }
            if self.file.sync_all(IO).is_err() {
                self.writable = false;
                return Err(Error::Poisoned { cause: "catalog preallocation sync failed" });
            }
            self.allocated_until = additional;
        }
        if let Err(error) = self.file.append_prealloc(&encoded, IO) {
            self.writable = false;
            return Err(error);
        }
        if self.file.sync_data(IO).is_err() {
            self.writable = false;
            return Err(Error::Poisoned { cause: "catalog edit sync failed" });
        }
        self.state = next_state;
        self.previous_crc = decoded.crc;
        self.previous_mac = decoded.chain_mac.unwrap_or([0; 32]);
        self.next_sequence = next_sequence;
        self.append_offset = next_offset;
        Ok(edits.iter().map(|_| self.state.epoch()).collect())
    }

    /// Persists typed production edits. PurgeBatch, UnlinkAck and
    /// PurgeComplete are intentionally distinct operations and retain their
    /// per-file lifecycle in the folded state.
    pub fn append_edits_durable(
        &mut self,
        edits: &[super::edit::CatalogEdit],
        integrity: CatalogIntegrity,
        key: Option<&Key32>,
        mac: Option<&dyn Mac>,
    ) -> Result<Vec<u64>, Error> {
        let ops = edits.iter().cloned().map(|edit| vec![edit.into_op()]).collect::<Vec<_>>();
        self.append_group_durable(&ops, integrity, key, mac)
    }

    /// Durably reserves a user SpaceId interval before returning it to an
    /// allocator. Callers must serialize allocation through this mutable
    /// CatalogFile handle; after reopen, unused IDs in the interval are lost.
    pub fn reserve_space_ids_durable(
        &mut self,
        amount: u64,
        integrity: CatalogIntegrity,
        key: Option<&Key32>,
        mac: Option<&dyn Mac>,
    ) -> Result<super::reserve::Reservation, Error> {
        let (op, reservation) = super::reserve::next_space_id_reservation(
            self.state.space_id_limit(), amount,
        )?;
        self.append_durable(&[op], integrity, key, mac)?;
        Ok(reservation)
    }

    pub fn path(&self) -> &Path { &self.path }
    pub fn state(&self) -> &CatalogState { &self.state }
    pub fn next_sequence(&self) -> u64 { self.next_sequence }
    pub fn append_offset(&self) -> u64 { self.append_offset }
    pub fn is_writable(&self) -> bool { self.writable }
    pub fn has_torn_tail(&self) -> bool { self.torn_tail }
}

fn read_exact_at(file: &dyn File, offset: u64, output: &mut [u8]) -> Result<(), Error> {
    let mut read = 0usize;
    while read < output.len() {
        let read_offset = offset.checked_add(read as u64).ok_or(Error::Limit { limit: "catalog read offset" })?;
        let target = output.get_mut(read..).ok_or(Error::Corrupt { place: "catalog read buffer" })?;
        let count = file.read_at(read_offset, target, IO)?;
        if count == 0 { return Err(Error::Corrupt { place: "catalog short read" }); }
        read = read.checked_add(count).ok_or(Error::Limit { limit: "catalog read length" })?;
    }
    Ok(())
}

fn zero_tail(file: &dyn File, mut offset: u64, end: u64) -> Result<bool, Error> {
    let mut buffer = [0u8; 4096];
    while offset < end {
        let count = usize::try_from((end - offset).min(buffer.len() as u64))
            .map_err(|_| Error::Limit { limit: "catalog tail length" })?;
        let chunk = buffer.get_mut(..count).ok_or(Error::Limit { limit: "catalog tail buffer" })?;
        read_exact_at(file, offset, chunk)?;
        if chunk.iter().any(|byte| *byte != 0) { return Ok(false); }
        offset = offset.checked_add(count as u64).ok_or(Error::Limit { limit: "catalog tail offset" })?;
    }
    Ok(true)
}
