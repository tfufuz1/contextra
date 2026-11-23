//! Cluster-sealed per-block summaries used for one-block reads and skipping.

use contextra_storage_ports::{Error, Stamp};
use core::fmt;
use zeroize::Zeroizing;

const MAGIC: &[u8; 8] = b"CTXSUB01";
const FIXED_ENTRY_BYTES: usize = 8 + 4 + 4 + 8 + 8 + 16 + 4 * 8;

#[derive(Clone, PartialEq, Eq)]
pub struct SubIndexEntry {
    pub last_key: Zeroizing<Vec<u8>>,
    pub offset: u64,
    pub len: u32,
    pub entries: u32,
    pub weight_sum: [u64; 4],
    pub min_stamp: Stamp,
    pub max_stamp: Stamp,
    pub block_uid: [u8; 16],
}

impl fmt::Debug for SubIndexEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SubIndexEntry").field("last_key", &"<redacted>")
            .field("offset", &self.offset).field("len", &self.len).field("entries", &self.entries)
            .field("weight_sum", &"<redacted>").field("min_stamp", &self.min_stamp)
            .field("max_stamp", &self.max_stamp).field("block_uid", &"<redacted>").finish()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SubIndex { entries: Vec<SubIndexEntry> }

impl SubIndex {
    pub fn new(entries: Vec<SubIndexEntry>) -> Result<Self, Error> {
        validate_entries(&entries)?;
        Ok(Self { entries })
    }

    pub fn entries(&self) -> &[SubIndexEntry] { &self.entries }

    pub fn candidate(&self, key: &[u8], stamp_cap: Stamp) -> Option<&SubIndexEntry> {
        let index = self.entries.partition_point(|entry| entry.last_key.as_slice() < key);
        self.entries.get(index).filter(|entry| entry.min_stamp <= stamp_cap)
    }

    pub fn encode(&self, max_bytes: usize) -> Result<Vec<u8>, Error> {
        validate_entries(&self.entries)?;
        let mut out = Vec::new();
        out.extend_from_slice(MAGIC);
        put_u32(&mut out, u32::try_from(self.entries.len()).map_err(|_| Error::Limit { limit: "subindex block count" })?);
        for entry in &self.entries {
            let key_len = u32::try_from(entry.last_key.len()).map_err(|_| Error::Limit { limit: "subindex key length" })?;
            put_u32(&mut out, key_len);
            out.extend_from_slice(&entry.last_key);
            put_u64(&mut out, entry.offset);
            put_u32(&mut out, entry.len);
            put_u32(&mut out, entry.entries);
            put_u64(&mut out, entry.min_stamp);
            put_u64(&mut out, entry.max_stamp);
            out.extend_from_slice(&entry.block_uid);
            for weight in entry.weight_sum { put_u64(&mut out, weight); }
            if out.len().checked_add(4).ok_or(Error::Limit { limit: "subindex size" })? > max_bytes {
                return Err(Error::Limit { limit: "subindex exceeds configured size" });
            }
        }
        let crc = crate::codec::checksums::crc32c(&out);
        put_u32(&mut out, crc);
        if out.len() > max_bytes { return Err(Error::Limit { limit: "subindex exceeds configured size" }); }
        Ok(out)
    }

    pub fn decode(bytes: &[u8], max_bytes: usize, max_entries: usize) -> Result<Self, Error> {
        if bytes.len() > max_bytes || bytes.len() < 16 {
            return Err(Error::Corrupt { place: "subindex length" });
        }
        if bytes.get(..8) != Some(MAGIC.as_slice()) { return Err(Error::Unsupported { what: "subindex magic" }); }
        let payload_end = bytes.len().checked_sub(4).ok_or(Error::Corrupt { place: "subindex CRC" })?;
        let expected = read_u32(bytes, payload_end)?;
        if crate::codec::checksums::crc32c(bytes.get(..payload_end).ok_or(Error::Corrupt { place: "subindex CRC input" })?) != expected {
            return Err(Error::Corrupt { place: "subindex CRC" });
        }
        let count = usize::try_from(read_u32(bytes, 8)?).map_err(|_| Error::Corrupt { place: "subindex count" })?;
        let min_size = count.checked_mul(FIXED_ENTRY_BYTES + 4)
            .and_then(|value| value.checked_add(16))
            .ok_or(Error::Corrupt { place: "subindex count size" })?;
        if count > max_entries || min_size > bytes.len() { return Err(Error::Corrupt { place: "subindex count limit" }); }
        let mut cursor = Cursor { bytes, offset: 12, end: payload_end };
        let mut entries = Vec::with_capacity(count);
        for _ in 0..count {
            let key_len = usize::try_from(cursor.u32()?).map_err(|_| Error::Corrupt { place: "subindex key length" })?;
            if key_len > max_bytes { return Err(Error::Corrupt { place: "subindex key limit" }); }
            let last_key = Zeroizing::new(cursor.take(key_len)?.to_vec());
            let offset = cursor.u64()?;
            let len = cursor.u32()?;
            let entry_count = cursor.u32()?;
            let min_stamp = cursor.u64()?;
            let max_stamp = cursor.u64()?;
            let block_uid = cursor.array::<16>()?;
            let mut weight_sum = [0u64; 4];
            for weight in &mut weight_sum { *weight = cursor.u64()?; }
            entries.push(SubIndexEntry { last_key, offset, len, entries: entry_count,
                weight_sum, min_stamp, max_stamp, block_uid });
        }
        if cursor.offset != payload_end { return Err(Error::Corrupt { place: "subindex trailing bytes" }); }
        Self::new(entries)
    }
}

fn validate_entries(entries: &[SubIndexEntry]) -> Result<(), Error> {
    for entry in entries {
        if entry.len == 0 || entry.entries == 0 || entry.min_stamp > entry.max_stamp {
            return Err(Error::Corrupt { place: "subindex entry bounds" });
        }
        let _end = entry.offset.checked_add(u64::from(entry.len))
            .ok_or(Error::Corrupt { place: "subindex offset overflow" })?;
    }
    for pair in entries.windows(2) {
        let left = pair.first().ok_or(Error::Corrupt { place: "subindex pair" })?;
        let right = pair.get(1).ok_or(Error::Corrupt { place: "subindex pair" })?;
        let left_end = left.offset.checked_add(u64::from(left.len))
            .ok_or(Error::Corrupt { place: "subindex offset overflow" })?;
        if left.last_key.as_slice() >= right.last_key.as_slice() || left_end > right.offset {
            return Err(Error::Corrupt { place: "subindex order" });
        }
    }
    Ok(())
}

fn put_u32(out: &mut Vec<u8>, value: u32) { out.extend_from_slice(&value.to_le_bytes()); }
fn put_u64(out: &mut Vec<u8>, value: u64) { out.extend_from_slice(&value.to_le_bytes()); }
fn read_u32(bytes: &[u8], offset: usize) -> Result<u32, Error> {
    let end = offset.checked_add(4).ok_or(Error::Corrupt { place: "subindex offset" })?;
    Ok(u32::from_le_bytes(bytes.get(offset..end).ok_or(Error::Corrupt { place: "subindex field" })?
        .try_into().map_err(|_| Error::Corrupt { place: "subindex field" })?))
}
struct Cursor<'a> { bytes: &'a [u8], offset: usize, end: usize }
impl Cursor<'_> {
    fn take(&mut self, len: usize) -> Result<&[u8], Error> {
        let end = self.offset.checked_add(len).ok_or(Error::Corrupt { place: "subindex cursor overflow" })?;
        if end > self.end { return Err(Error::Corrupt { place: "subindex truncated" }); }
        let value = self.bytes.get(self.offset..end).ok_or(Error::Corrupt { place: "subindex field" })?;
        self.offset = end;
        Ok(value)
    }
    fn array<const N: usize>(&mut self) -> Result<[u8; N], Error> {
        self.take(N)?.try_into().map_err(|_| Error::Corrupt { place: "subindex field" })
    }
    fn u32(&mut self) -> Result<u32, Error> { Ok(u32::from_le_bytes(self.array()?)) }
    fn u64(&mut self) -> Result<u64, Error> { Ok(u64::from_le_bytes(self.array()?)) }
}
