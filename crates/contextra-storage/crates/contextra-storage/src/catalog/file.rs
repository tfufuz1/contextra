//! Öffnet und faltet eine Catalog-Datei aus dem validen Recordpräfix.

use super::record::{decode_record, CatalogIntegrity, CatalogRecord, MAX_RECORD_BYTES};
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
    records: Vec<CatalogRecord>,
    next_sequence: u64,
    previous_crc: u32,
    previous_mac: [u8; 32],
    append_offset: u64,
    writable: bool,
    torn_tail: bool,
}

impl CatalogFile {
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
            path: path.to_path_buf(), file, state: CatalogState::default(), records: Vec::new(),
            next_sequence: 1, previous_crc: 0, previous_mac: [0; 32], append_offset: 0,
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
            self.records.push(record);
        }
        self.append_offset = offset;
        self.writable = mode == OpenMode::ReadWrite && !self.torn_tail;
        Ok(())
    }

    pub fn path(&self) -> &Path { &self.path }
    pub fn state(&self) -> &CatalogState { &self.state }
    pub fn records(&self) -> &[CatalogRecord] { &self.records }
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
