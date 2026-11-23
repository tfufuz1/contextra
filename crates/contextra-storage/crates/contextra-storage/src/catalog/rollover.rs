//! CURRENT pointer codec and atomic publication helper.

use contextra_storage_ports::traits::{FileRole, Fs, IoCtx};
use contextra_storage_ports::Error;
use std::path::Path;

const MAGIC: &[u8; 8] = b"CTXCUR01";
const CURRENT_LEN: usize = 8 + 8 + 4;
const IO: IoCtx = IoCtx { role: FileRole::Catalog, purpose: "catalog_current" };

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CurrentPointer { pub generation: u64 }

impl CurrentPointer {
    pub fn encode(self) -> Result<[u8; CURRENT_LEN], Error> {
        let mut out = [0u8; CURRENT_LEN];
        let mut offset = 0usize;
        put(&mut out, &mut offset, MAGIC);
        put(&mut out, &mut offset, &self.generation.to_le_bytes());
        let crc = {
            let crc_input = out.get(..offset).ok_or(Error::Corrupt { place: "CURRENT encode offset" })?;
            crate::codec::checksums::crc32c(crc_input)
        };
        put(&mut out, &mut offset, &crc.to_le_bytes());
        Ok(out)
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, Error> {
        if bytes.len() != CURRENT_LEN { return Err(Error::Corrupt { place: "CURRENT length" }); }
        if bytes.get(..8) != Some(MAGIC.as_slice()) { return Err(Error::Unsupported { what: "CURRENT magic" }); }
        let generation = u64::from_le_bytes(bytes.get(8..16).ok_or(Error::Corrupt { place: "CURRENT generation" })?
            .try_into().map_err(|_| Error::Corrupt { place: "CURRENT generation" })?);
        let crc = u32::from_le_bytes(bytes.get(16..20).ok_or(Error::Corrupt { place: "CURRENT CRC" })?
            .try_into().map_err(|_| Error::Corrupt { place: "CURRENT CRC" })?);
        if crate::codec::checksums::crc32c(bytes.get(..16).ok_or(Error::Corrupt { place: "CURRENT CRC input" })?) != crc {
            return Err(Error::Corrupt { place: "CURRENT CRC" });
        }
        Ok(Self { generation })
    }
}

pub fn publish_current(fs: &dyn Fs, root: &Path, pointer: CurrentPointer) -> Result<(), Error> {
    let temp = root.join("CURRENT.building");
    let current = root.join("CURRENT");
    match fs.unlink(&temp, IO) {
        Ok(()) => {}
        Err(Error::Io { kind: std::io::ErrorKind::NotFound, .. }) => {}
        Err(error) => return Err(error),
    }
    let file = fs.create_new(&temp, IO)?;
    let bytes = pointer.encode()?;
    file.preallocate(bytes.len() as u64, IO)?;
    file.append_prealloc(&bytes, IO)?;
    file.sync_all(IO)?;
    fs.rename(&temp, &current, IO)?;
    fs.sync_dir(root, IO)?;
    Ok(())
}

fn put(out: &mut [u8], offset: &mut usize, bytes: &[u8]) {
    for (target, byte) in out.iter_mut().skip(*offset).zip(bytes.iter().copied()) { *target = byte; }
    *offset = offset.saturating_add(bytes.len());
}

pub fn read_current(fs: &dyn Fs, root: &Path) -> Result<CurrentPointer, Error> {
    let file = fs.open(&root.join("CURRENT"), IO)?;
    if file.size()? != CURRENT_LEN as u64 { return Err(Error::Corrupt { place: "CURRENT file length" }); }
    let mut bytes = [0u8; CURRENT_LEN];
    let mut read = 0usize;
    while read < bytes.len() {
        let offset = read as u64;
        let target = bytes.get_mut(read..).ok_or(Error::Corrupt { place: "CURRENT buffer" })?;
        let count = file.read_at(offset, target, IO)?;
        if count == 0 { return Err(Error::Corrupt { place: "CURRENT short read" }); }
        read = read.checked_add(count).ok_or(Error::Corrupt { place: "CURRENT read length" })?;
    }
    CurrentPointer::decode(&bytes)
}
