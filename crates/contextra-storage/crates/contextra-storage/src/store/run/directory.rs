//! Bounded ClusterDirectory codec (R8 §5.6).
//!
//! The directory contains structural metadata only. It intentionally carries
//! no keys, branch names, token weights, or payload hashes.

use contextra_storage_ports::{ClusterId, Error, Stamp};

const MAGIC: &[u8; 8] = b"CTXDIR01";
const ENTRY_BYTES: usize = 16 + 8 + 8 + 4 + 8 + 8 + 8;
pub const MAX_DIRECTORY_ENTRIES: usize = 1_000_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClusterDirectoryEntry {
    pub cluster: ClusterId,
    pub section_offset: u64,
    pub section_len: u64,
    pub block_count: u32,
    pub entry_count: u64,
    pub min_stamp: Stamp,
    pub max_stamp: Stamp,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ClusterDirectory {
    entries: Vec<ClusterDirectoryEntry>,
}

impl ClusterDirectory {
    pub fn new(entries: Vec<ClusterDirectoryEntry>) -> Result<Self, Error> {
        validate(&entries)?;
        Ok(Self { entries })
    }

    pub fn entries(&self) -> &[ClusterDirectoryEntry] { &self.entries }

    pub fn find(&self, cluster: ClusterId) -> Option<&ClusterDirectoryEntry> {
        self.entries.binary_search_by_key(&cluster, |entry| entry.cluster)
            .ok().and_then(|index| self.entries.get(index))
    }

    pub fn encode(&self) -> Result<Vec<u8>, Error> {
        validate(&self.entries)?;
        let count = u32::try_from(self.entries.len()).map_err(|_| Error::Limit { limit: "run directory entry count" })?;
        let cap = 8usize.checked_add(4)
            .and_then(|n| n.checked_add(self.entries.len().checked_mul(ENTRY_BYTES)?))
            .and_then(|n| n.checked_add(4))
            .ok_or(Error::Limit { limit: "run directory size" })?;
        let mut out = Vec::with_capacity(cap);
        out.extend_from_slice(MAGIC);
        out.extend_from_slice(&count.to_le_bytes());
        for entry in &self.entries {
            out.extend_from_slice(entry.cluster.as_bytes());
            out.extend_from_slice(&entry.section_offset.to_le_bytes());
            out.extend_from_slice(&entry.section_len.to_le_bytes());
            out.extend_from_slice(&entry.block_count.to_le_bytes());
            out.extend_from_slice(&entry.entry_count.to_le_bytes());
            out.extend_from_slice(&entry.min_stamp.to_le_bytes());
            out.extend_from_slice(&entry.max_stamp.to_le_bytes());
        }
        let crc = crate::codec::checksums::crc32c(&out);
        out.extend_from_slice(&crc.to_le_bytes());
        Ok(out)
    }

    pub fn decode(bytes: &[u8], max_entries: usize) -> Result<Self, Error> {
        if bytes.len() < 16 { return Err(Error::Corrupt { place: "run directory length" }); }
        if bytes.get(..8) != Some(MAGIC.as_slice()) { return Err(Error::Unsupported { what: "run directory magic" }); }
        let count = read_u32(bytes, 8)? as usize;
        if count > max_entries.min(MAX_DIRECTORY_ENTRIES) { return Err(Error::Corrupt { place: "run directory entry limit" }); }
        let expected_len = count.checked_mul(ENTRY_BYTES).and_then(|n| n.checked_add(16))
            .ok_or(Error::Corrupt { place: "run directory size overflow" })?;
        if expected_len != bytes.len() { return Err(Error::Corrupt { place: "run directory encoded length" }); }
        let crc_offset = bytes.len().checked_sub(4).ok_or(Error::Corrupt { place: "run directory CRC" })?;
        if read_u32(bytes, crc_offset)? != crate::codec::checksums::crc32c(bytes.get(..crc_offset).ok_or(Error::Corrupt { place: "run directory CRC input" })?) {
            return Err(Error::Corrupt { place: "run directory CRC" });
        }
        let mut cursor = Cursor { bytes, offset: 12, end: crc_offset };
        let mut entries = Vec::with_capacity(count);
        for _ in 0..count {
            entries.push(ClusterDirectoryEntry {
                cluster: ClusterId::from_wire(cursor.array::<16>()?),
                section_offset: cursor.u64()?,
                section_len: cursor.u64()?,
                block_count: cursor.u32()?,
                entry_count: cursor.u64()?,
                min_stamp: cursor.u64()?,
                max_stamp: cursor.u64()?,
            });
        }
        if cursor.offset != crc_offset { return Err(Error::Corrupt { place: "run directory trailing bytes" }); }
        Self::new(entries)
    }
}

fn validate(entries: &[ClusterDirectoryEntry]) -> Result<(), Error> {
    if entries.len() > MAX_DIRECTORY_ENTRIES { return Err(Error::Limit { limit: "run directory entries" }); }
    for entry in entries {
        if entry.section_len == 0 || entry.block_count == 0 || entry.entry_count == 0 || entry.min_stamp > entry.max_stamp {
            return Err(Error::Corrupt { place: "run directory entry bounds" });
        }
        entry.section_offset.checked_add(entry.section_len).ok_or(Error::Corrupt { place: "run directory section overflow" })?;
    }
    for pair in entries.windows(2) {
        let left = pair.first().ok_or(Error::Corrupt { place: "run directory pair" })?;
        let right = pair.get(1).ok_or(Error::Corrupt { place: "run directory pair" })?;
        if left.cluster >= right.cluster || left.section_offset.checked_add(left.section_len)
            .is_none_or(|end| end > right.section_offset) {
            return Err(Error::Corrupt { place: "run directory order or overlap" });
        }
    }
    Ok(())
}

fn read_u32(bytes: &[u8], offset: usize) -> Result<u32, Error> {
    let end = offset.checked_add(4).ok_or(Error::Corrupt { place: "run directory offset" })?;
    Ok(u32::from_le_bytes(bytes.get(offset..end).ok_or(Error::Corrupt { place: "run directory field" })?
        .try_into().map_err(|_| Error::Corrupt { place: "run directory field" })?))
}

struct Cursor<'a> { bytes: &'a [u8], offset: usize, end: usize }
impl Cursor<'_> {
    fn take(&mut self, len: usize) -> Result<&[u8], Error> {
        let end = self.offset.checked_add(len).ok_or(Error::Corrupt { place: "run directory offset overflow" })?;
        if end > self.end { return Err(Error::Corrupt { place: "run directory truncated" }); }
        let value = self.bytes.get(self.offset..end).ok_or(Error::Corrupt { place: "run directory field" })?;
        self.offset = end;
        Ok(value)
    }
    fn array<const N: usize>(&mut self) -> Result<[u8; N], Error> {
        self.take(N)?.try_into().map_err(|_| Error::Corrupt { place: "run directory field" })
    }
    fn u32(&mut self) -> Result<u32, Error> { Ok(u32::from_le_bytes(self.array()?)) }
    fn u64(&mut self) -> Result<u64, Error> { Ok(u64::from_le_bytes(self.array()?)) }
}
