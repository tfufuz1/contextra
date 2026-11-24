//! Structural Run reader. Payload decoding and KeyScope reads are separate APIs.

use super::directory::{ClusterDirectory, ClusterDirectoryEntry, MAX_DIRECTORY_ENTRIES};
use super::extension::{self, ExtensionBlock, MAX_EXTENSION_BLOCKS};
use super::format::{RunFooter, RUN_FOOTER_LEN};
use super::table_cache::TableCache;
use crate::codec::header::{FileHeader, HEADER_LEN};
use contextra_storage_ports::traits::{File, Fs};
use contextra_storage_ports::{ClusterId, Error};
use std::path::Path;
use std::sync::Arc;
use zeroize::Zeroizing;

pub const MAX_PACK_META_BYTES: usize = 1024 * 1024;
pub const MAX_EXTENSION_REGION_BYTES: usize = 16 * 1024 * 1024;

pub struct RunReader {
    file: Arc<dyn File>,
    file_len: u64,
    header: FileHeader,
    footer: RunFooter,
    directory: ClusterDirectory,
    pack_meta: Zeroizing<Vec<u8>>,
    extensions: Vec<ExtensionBlock>,
}

impl RunReader {
    pub fn open(
        fs: &dyn Fs,
        cache: &TableCache,
        path: &Path,
        expected_magic: [u8; 8],
        expected_kind: u8,
        supported_features: u32,
        max_directory_entries: usize,
    ) -> Result<Self, Error> {
        if max_directory_entries > MAX_DIRECTORY_ENTRIES {
            return Err(Error::Limit { limit: "run directory entry bound" });
        }
        let file = cache.pin(fs, path)?;
        let file_len = file.size()?;
        let minimum_len = (HEADER_LEN + RUN_FOOTER_LEN) as u64;
        if file_len < minimum_len { return Err(Error::Corrupt { place: "run file minimum length" }); }
        let header_bytes = read_vec(file.as_ref(), 0, HEADER_LEN)?;
        let header = FileHeader::decode(&header_bytes, expected_magic, expected_kind, supported_features)?;
        let footer_offset = file_len.checked_sub(RUN_FOOTER_LEN as u64).ok_or(Error::Corrupt { place: "run footer offset" })?;
        let footer_bytes = read_vec(file.as_ref(), footer_offset, RUN_FOOTER_LEN)?;
        let footer = RunFooter::decode(&footer_bytes, file_len)?;
        if footer.directory_offset < HEADER_LEN as u64 { return Err(Error::Corrupt { place: "run directory overlaps header" }); }
        let directory_len = usize::try_from(footer.directory_len).map_err(|_| Error::Corrupt { place: "run directory length" })?;
        let max_dir_len = max_directory_entries.checked_mul(60).and_then(|len| len.checked_add(16))
            .ok_or(Error::Limit { limit: "run directory allocation bound" })?;
        if directory_len > max_dir_len { return Err(Error::Corrupt { place: "run directory allocation limit" }); }
        let directory_bytes = read_vec(file.as_ref(), footer.directory_offset, directory_len)?;
        let directory = ClusterDirectory::decode(&directory_bytes, max_directory_entries)?;
        for section in directory.entries() {
            let section_end = section.section_offset.checked_add(section.section_len)
                .ok_or(Error::Corrupt { place: "run section offset overflow" })?;
            if section.section_offset < HEADER_LEN as u64 || section_end > footer.directory_offset {
                return Err(Error::Corrupt { place: "run section outside data region" });
            }
        }
        let pack_len = footer.extension_offset.checked_sub(footer.pack_offset)
            .ok_or(Error::Corrupt { place: "run pack metadata bounds" })?;
        let pack_len = usize::try_from(pack_len).map_err(|_| Error::Corrupt { place: "run pack metadata length" })?;
        if pack_len == 0 || pack_len > MAX_PACK_META_BYTES { return Err(Error::Corrupt { place: "run pack metadata limit" }); }
        let pack_meta = Zeroizing::new(read_vec(file.as_ref(), footer.pack_offset, pack_len)?);
        let ext_len = usize::try_from(footer_offset.checked_sub(footer.extension_offset)
            .ok_or(Error::Corrupt { place: "run extension bounds" })?)
            .map_err(|_| Error::Corrupt { place: "run extension length" })?;
        if ext_len > MAX_EXTENSION_REGION_BYTES { return Err(Error::Corrupt { place: "run extension region limit" }); }
        let extension_bytes = read_vec(file.as_ref(), footer.extension_offset, ext_len)?;
        let extensions = extension::decode(&extension_bytes, MAX_EXTENSION_REGION_BYTES, MAX_EXTENSION_BLOCKS)?;
        Ok(Self { file, file_len, header, footer, directory, pack_meta, extensions })
    }

    pub fn header(&self) -> &FileHeader { &self.header }
    pub fn footer(&self) -> RunFooter { self.footer }
    pub fn file_len(&self) -> u64 { self.file_len }
    pub fn directory(&self) -> &ClusterDirectory { &self.directory }
    pub fn pack_meta(&self) -> &[u8] { &self.pack_meta }
    pub fn extensions(&self) -> &[ExtensionBlock] { &self.extensions }

    pub fn read_section(&self, cluster: ClusterId, max_bytes: usize) -> Result<Option<Zeroizing<Vec<u8>>>, Error> {
        let Some(entry) = self.directory.find(cluster) else { return Ok(None); };
        let len = usize::try_from(entry.section_len).map_err(|_| Error::Corrupt { place: "run section length" })?;
        if len > max_bytes { return Err(Error::Limit { limit: "run section read bound" }); }
        Ok(Some(Zeroizing::new(read_vec(self.file.as_ref(), entry.section_offset, len)?)))
    }

    pub fn section_meta(&self, cluster: ClusterId) -> Option<&ClusterDirectoryEntry> {
        self.directory.find(cluster)
    }

    /// Reads and decodes a cluster entry by key using Filter -> SubIndex -> 1 DataBlock.
    pub fn get_entry(
        &self,
        ctx: &super::section::SectionReadContext<'_>,
        branch: contextra_storage_ports::BranchId,
        user_key: &[u8],
        stamp_cap: contextra_storage_ports::Stamp,
        max_section_bytes: usize,
    ) -> Result<Option<super::block::BlockEntry>, Error> {
        let Some(sec) = self.read_section(ctx.cluster, max_section_bytes)? else {
            return Ok(None);
        };
        super::section::get_in_section(&sec, ctx, branch, user_key, stamp_cap)
    }

    /// Scans visible cluster entries inside `[start_inclusive, end_exclusive)` up to `stamp_cap`.
    pub fn scan_entries(
        &self,
        ctx: &super::section::SectionReadContext<'_>,
        branch: contextra_storage_ports::BranchId,
        start_inclusive: Option<&[u8]>,
        end_exclusive: Option<&[u8]>,
        stamp_cap: contextra_storage_ports::Stamp,
        max_section_bytes: usize,
    ) -> Result<Vec<super::block::BlockEntry>, Error> {
        let Some(sec) = self.read_section(ctx.cluster, max_section_bytes)? else {
            return Ok(Vec::new());
        };
        super::section::scan_section(&sec, ctx, branch, start_inclusive, end_exclusive, stamp_cap)
    }
}

fn read_vec(file: &dyn File, offset: u64, len: usize) -> Result<Vec<u8>, Error> {
    let mut output = vec![0u8; len];
    let mut read = 0usize;
    while read < len {
        let read_offset = offset.checked_add(read as u64).ok_or(Error::Corrupt { place: "run read offset overflow" })?;
        let target = output.get_mut(read..).ok_or(Error::Corrupt { place: "run read buffer" })?;
        let count = file.read_at(read_offset, target, super::table_cache::run_io())?;
        if count == 0 { return Err(Error::Corrupt { place: "run short read" }); }
        read = read.checked_add(count).ok_or(Error::Corrupt { place: "run read length" })?;
    }
    Ok(output)
}
