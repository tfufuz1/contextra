//! Physical Run assembly from already encoded cluster sections.
//!
//! This layer owns file ordering, offsets, preallocation, directory/footer
//! framing, and final sync. Entry ordering, sealing, and pack-policy checks
//! belong to the higher-level entry builder.

use super::directory::{ClusterDirectory, ClusterDirectoryEntry};
use super::extension::{self, ExtensionBlock};
use super::format::RunFooter;
use crate::codec::header::{FileHeader, HEADER_LEN};
use contextra_storage_ports::traits::{File, FileRole, Fs, IoCtx};
use contextra_storage_ports::{ClusterId, Error, Stamp};
use std::path::{Path, PathBuf};

const IO: IoCtx = IoCtx { role: FileRole::Run, purpose: "run_build" };
pub const RUN_PREALLOC_BYTES: u64 = 4 * 1024 * 1024;
pub const MAX_PACK_META_BYTES: usize = 1024 * 1024;

pub struct BuiltRun { pub path: PathBuf, pub file_bytes: u64, pub directory: ClusterDirectory }

pub struct RunFileBuilder {
    path: PathBuf,
    file: Option<Box<dyn File>>,
    offset: u64,
    allocated_until: u64,
    sections: Vec<ClusterDirectoryEntry>,
    last_cluster: Option<ClusterId>,
    poisoned: bool,
}

impl RunFileBuilder {
    pub fn create(fs: &dyn Fs, path: &Path, header: &FileHeader) -> Result<Self, Error> {
        let file = fs.create_new(path, IO)?;
        file.preallocate(RUN_PREALLOC_BYTES, IO)?;
        file.append_prealloc(&header.encode()?, IO)?;
        Ok(Self { path: path.to_path_buf(), file: Some(file), offset: HEADER_LEN as u64,
            allocated_until: RUN_PREALLOC_BYTES, sections: Vec::new(), last_cluster: None, poisoned: false })
    }

    /// Appends one complete, already encoded ClusterSection. The section
    /// bytes stay cluster-local and are copied without interpreting payloads.
    pub fn append_section(
        &mut self,
        cluster: ClusterId,
        section_bytes: &[u8],
        block_count: u32,
        entry_count: u64,
        min_stamp: Stamp,
        max_stamp: Stamp,
    ) -> Result<(), Error> {
        if self.poisoned || self.file.is_none() { return Err(Error::Poisoned { cause: "run builder is not writable" }); }
        if self.last_cluster.is_some_and(|previous| previous >= cluster) {
            return Err(Error::Limit { limit: "run sections must have unique ascending cluster ids" });
        }
        if section_bytes.is_empty() || block_count == 0 || entry_count == 0 || min_stamp > max_stamp {
            return Err(Error::Limit { limit: "invalid run section metadata" });
        }
        let len = u64::try_from(section_bytes.len()).map_err(|_| Error::Limit { limit: "run section size" })?;
        let next = self.offset.checked_add(len).ok_or(Error::Limit { limit: "run file size" })?;
        self.ensure_capacity(next)?;
        let file = self.file.as_ref().ok_or(Error::Poisoned { cause: "run builder is closed" })?;
        if let Err(error) = file.append_prealloc(section_bytes, IO) {
            self.poisoned = true;
            return Err(error);
        }
        self.sections.push(ClusterDirectoryEntry { cluster, section_offset: self.offset,
            section_len: len, block_count, entry_count, min_stamp, max_stamp });
        self.offset = next;
        self.last_cluster = Some(cluster);
        Ok(())
    }

    /// Finishes the immutable file. `pack_meta` is opaque here because its
    /// owner-SP has not frozen its field layout yet.
    pub fn finish(mut self, pack_meta: &[u8], extensions: &[ExtensionBlock]) -> Result<BuiltRun, Error> {
        if self.poisoned || self.file.is_none() { return Err(Error::Poisoned { cause: "run builder is not writable" }); }
        if pack_meta.is_empty() || pack_meta.len() > MAX_PACK_META_BYTES {
            return Err(Error::Limit { limit: "pack metadata size" });
        }
        let directory = ClusterDirectory::new(std::mem::take(&mut self.sections))?;
        let directory_bytes = directory.encode()?;
        let extension_bytes = extension::encode(extensions, MAX_EXTENSION_REGION_BYTES)?;
        let directory_offset = self.offset;
        self.append(&directory_bytes)?;
        let pack_offset = self.offset;
        self.append(pack_meta)?;
        let extension_offset = self.offset;
        self.append(&extension_bytes)?;
        let file_bytes = self.offset.checked_add(super::format::RUN_FOOTER_LEN as u64)
            .ok_or(Error::Limit { limit: "run footer size" })?;
        let footer = RunFooter { directory_offset, directory_len: directory_bytes.len() as u64, pack_offset, extension_offset }
            .encode(file_bytes)?;
        self.append(&footer)?;
        let file = self.file.take().ok_or(Error::Poisoned { cause: "run builder is closed" })?;
        file.sync_all(IO).map_err(|_| Error::Poisoned { cause: "run file sync failed" })?;
        Ok(BuiltRun { path: self.path.clone(), file_bytes, directory })
    }

    fn append(&mut self, bytes: &[u8]) -> Result<(), Error> {
        let len = u64::try_from(bytes.len()).map_err(|_| Error::Limit { limit: "run append length" })?;
        let next = self.offset.checked_add(len).ok_or(Error::Limit { limit: "run file size" })?;
        self.ensure_capacity(next)?;
        let file = self.file.as_ref().ok_or(Error::Poisoned { cause: "run builder is closed" })?;
        if let Err(error) = file.append_prealloc(bytes, IO) { self.poisoned = true; return Err(error); }
        self.offset = next;
        Ok(())
    }

    fn ensure_capacity(&mut self, required: u64) -> Result<(), Error> {
        if required <= self.allocated_until { return Ok(()); }
        let rounded = required.checked_add(RUN_PREALLOC_BYTES - 1)
            .map(|value| (value / RUN_PREALLOC_BYTES) * RUN_PREALLOC_BYTES)
            .ok_or(Error::Limit { limit: "run preallocation size" })?;
        let file = self.file.as_ref().ok_or(Error::Poisoned { cause: "run builder is closed" })?;
        if let Err(error) = file.preallocate(rounded, IO) { return Err(error); }
        self.allocated_until = rounded;
        Ok(())
    }
}

const MAX_EXTENSION_REGION_BYTES: usize = 16 * 1024 * 1024;
