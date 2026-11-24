//! Bounded cluster-local DataBlock codec with 16-entry prefix restarts.
//!
//! The caller seals the returned block bytes as one SealFrame. Consequently
//! this codec never interprets or persists file offsets or key material.

use crate::codec::checksums::crc32c;
use crate::codec::tags::{untag, Kind, TagError};
use crate::codec::varint::{varint_get, varint_put};
use contextra_storage_ports::{BranchId, Error, Stamp};
use core::fmt;
use zeroize::Zeroizing;

const MAGIC: &[u8; 8] = b"CTXBLK01";
const RESTART_INTERVAL: usize = 16;
pub const MAX_DECODED_BLOCK_BYTES: usize = 8 * 1024 * 1024;
pub const TARGET_BLOCK_BYTES: usize = 16 * 1024;

#[derive(Clone, PartialEq, Eq)]
pub struct BlockEntry {
    pub branch: BranchId,
    pub key: Zeroizing<Vec<u8>>,
    pub tag: u64,
    pub header: Zeroizing<Vec<u8>>,
    pub body: Zeroizing<Vec<u8>>,
}

impl fmt::Debug for BlockEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BlockEntry").field("branch", &self.branch).field("key", &"<redacted>")
            .field("tag", &self.tag).field("header", &"<redacted>").field("body", &"<redacted>").finish()
    }
}

impl BlockEntry {
    pub fn stamp(&self) -> Stamp { self.tag >> 8 }
}

/// Encodes sorted entries, using a full key at every restart entry. `max_bytes`
/// allows the builder's single-entry oversize progress rule while remaining
/// bounded by the configured change limit.
pub fn encode(entries: &[BlockEntry], max_bytes: usize) -> Result<Vec<u8>, Error> {
    if entries.is_empty() { return Err(Error::Limit { limit: "data block must contain entries" }); }
    if max_bytes == 0 || max_bytes > MAX_DECODED_BLOCK_BYTES {
        return Err(Error::Limit { limit: "data block size bound" });
    }
    for entry in entries {
        validate_tag(entry.tag)?;
    }
    for pair in entries.windows(2) {
        let left = pair.first().ok_or(Error::Corrupt { place: "data block pair" })?;
        let right = pair.get(1).ok_or(Error::Corrupt { place: "data block pair" })?;
        if !compare(left, right).is_lt() {
            return Err(Error::Limit { limit: "data block entries must be sorted" });
        }
    }
    let restart_count = entries.len().checked_add(RESTART_INTERVAL - 1)
        .ok_or(Error::Limit { limit: "data block restart count" })? / RESTART_INTERVAL;
    let mut out = Vec::new();
    out.extend_from_slice(MAGIC);
    put_u32(&mut out, u32::try_from(entries.len()).map_err(|_| Error::Limit { limit: "data block entry count" })?);
    put_u32(&mut out, u32::try_from(restart_count).map_err(|_| Error::Limit { limit: "data block restart count" })?);
    let mut restarts = Vec::with_capacity(restart_count);
    let mut previous: &[u8] = &[];
    for (index, entry) in entries.iter().enumerate() {
        let restart = index % RESTART_INTERVAL == 0;
        if restart {
            restarts.push(u32::try_from(out.len()).map_err(|_| Error::Limit { limit: "data block restart offset" })?);
        }
        let shared = if restart { 0 } else { common_prefix(previous, &entry.key) };
        varint_put(&mut out, u64::try_from(shared).map_err(|_| Error::Limit { limit: "data block prefix" })?);
        let suffix = entry.key.get(shared..).ok_or(Error::Corrupt { place: "data block prefix" })?;
        put_bytes(&mut out, suffix)?;
        put_u32(&mut out, entry.branch.0);
        put_u64(&mut out, entry.tag);
        put_bytes(&mut out, &entry.header)?;
        put_bytes(&mut out, &entry.body)?;
        if out.len() > max_bytes { return Err(Error::ChangeTooLarge); }
        previous = &entry.key;
    }
    for offset in restarts { put_u32(&mut out, offset); }
    let crc = crc32c(&out);
    put_u32(&mut out, crc);
    if out.len() > max_bytes { return Err(Error::ChangeTooLarge); }
    Ok(out)
}

/// Decodes exactly one block. Every wire length is checked against both the
/// remaining input and the caller's configured allocation limit.
pub fn decode(bytes: &[u8], max_bytes: usize) -> Result<Vec<BlockEntry>, Error> {
    if bytes.len() > max_bytes || bytes.len() > MAX_DECODED_BLOCK_BYTES || bytes.len() < 24 {
        return Err(Error::Corrupt { place: "data block length" });
    }
    if bytes.get(..8) != Some(MAGIC.as_slice()) { return Err(Error::Unsupported { what: "data block magic" }); }
    let payload_end = bytes.len().checked_sub(4).ok_or(Error::Corrupt { place: "data block CRC" })?;
    let expected_crc = read_u32(bytes, payload_end)?;
    if crc32c(bytes.get(..payload_end).ok_or(Error::Corrupt { place: "data block CRC input" })?) != expected_crc {
        return Err(Error::Corrupt { place: "data block CRC" });
    }
    let count = read_u32(bytes, 8)? as usize;
    let restart_count = read_u32(bytes, 12)? as usize;
    if count == 0 || count > MAX_DECODED_BLOCK_BYTES / 8 {
        return Err(Error::Corrupt { place: "data block entry count" });
    }
    let expected_restarts = count.checked_add(RESTART_INTERVAL - 1)
        .ok_or(Error::Corrupt { place: "data block restart count" })? / RESTART_INTERVAL;
    if restart_count != expected_restarts { return Err(Error::Corrupt { place: "data block restart count" }); }
    let restart_bytes = restart_count.checked_mul(4).ok_or(Error::Corrupt { place: "data block restart bytes" })?;
    let records_end = payload_end.checked_sub(restart_bytes).ok_or(Error::Corrupt { place: "data block restart table" })?;
    let entry_bytes = count.checked_mul(std::mem::size_of::<BlockEntry>())
        .ok_or(Error::Corrupt { place: "data block allocation size" })?;
    if entry_bytes > max_bytes { return Err(Error::Corrupt { place: "data block entry allocation limit" }); }
    let mut cursor = Cursor { bytes, offset: 16, end: records_end };
    let mut entries = Vec::with_capacity(count);
    let mut previous = Zeroizing::new(Vec::new());
    let mut actual_restarts = Vec::with_capacity(restart_count);
    let mut decoded_allocation = entry_bytes;
    for index in 0..count {
        if index % RESTART_INTERVAL == 0 {
            actual_restarts.push(u32::try_from(cursor.offset).map_err(|_| Error::Corrupt { place: "data block restart offset" })?);
        }
        let shared = usize::try_from(cursor.varint()?).map_err(|_| Error::Corrupt { place: "data block prefix length" })?;
        if (index % RESTART_INTERVAL == 0 && shared != 0) || shared > previous.len() {
            return Err(Error::Corrupt { place: "data block prefix length" });
        }
        let suffix = Zeroizing::new(cursor.bytes(max_bytes)?.to_vec());
        let key_len = shared.checked_add(suffix.len()).ok_or(Error::Corrupt { place: "data block key length" })?;
        if key_len > max_bytes { return Err(Error::Corrupt { place: "data block key length" }); }
        let branch = BranchId(cursor.u32()?);
        let tag = cursor.u64()?;
        validate_tag(tag)?;
        let header = Zeroizing::new(cursor.bytes(max_bytes)?.to_vec());
        let body = Zeroizing::new(cursor.bytes(max_bytes)?.to_vec());
        decoded_allocation = decoded_allocation.checked_add(key_len)
            .and_then(|value| value.checked_add(header.len()))
            .and_then(|value| value.checked_add(body.len()))
            .ok_or(Error::Corrupt { place: "data block decoded allocation" })?;
        if decoded_allocation > max_bytes { return Err(Error::Corrupt { place: "data block decoded allocation limit" }); }
        let mut key = Zeroizing::new(Vec::with_capacity(key_len));
        key.extend_from_slice(previous.get(..shared).ok_or(Error::Corrupt { place: "data block shared prefix" })?);
        key.extend_from_slice(&suffix);
        let entry = BlockEntry { branch, key, tag, header, body };
        if entries.last().is_some_and(|prior| !compare(prior, &entry).is_lt()) {
            return Err(Error::Corrupt { place: "data block sort order" });
        }
        previous = entry.key.clone();
        entries.push(entry);
    }
    if cursor.offset != records_end { return Err(Error::Corrupt { place: "data block record trailing bytes" }); }
    for (index, expected) in actual_restarts.into_iter().enumerate() {
        let offset = records_end.checked_add(index.checked_mul(4).ok_or(Error::Corrupt { place: "data block restart index" })?)
            .ok_or(Error::Corrupt { place: "data block restart index" })?;
        if read_u32(bytes, offset)? != expected { return Err(Error::Corrupt { place: "data block restart offset" }); }
    }
    Ok(entries)
}

fn validate_tag(value: u64) -> Result<(), Error> {
    match untag(value) {
        Ok((_, Kind::Put | Kind::Delete | Kind::PutRef)) => Ok(()),
        Ok((_, Kind::RangeDelete | Kind::EraseRange)) => {
            Err(Error::Unsupported { what: "data block tag kind" })
        }
        Err(TagError::UnknownKind(_) | TagError::ReservedMerge) => {
            Err(Error::Unsupported { what: "data block tag" })
        }
        Err(TagError::StampZero | TagError::StampOutOfRange) => {
            Err(Error::Corrupt { place: "data block tag" })
        }
    }
}

pub fn compare(left: &BlockEntry, right: &BlockEntry) -> std::cmp::Ordering {
    left.branch.cmp(&right.branch)
        .then_with(|| left.key.cmp(&right.key))
        .then_with(|| right.tag.cmp(&left.tag))
}

fn common_prefix(left: &[u8], right: &[u8]) -> usize {
    left.iter().zip(right).take_while(|(a, b)| a == b).count()
}

fn put_bytes(out: &mut Vec<u8>, value: &[u8]) -> Result<(), Error> {
    varint_put(out, u64::try_from(value.len()).map_err(|_| Error::Limit { limit: "data block field length" })?);
    out.extend_from_slice(value);
    Ok(())
}

fn put_u32(out: &mut Vec<u8>, value: u32) { out.extend_from_slice(&value.to_le_bytes()); }
fn put_u64(out: &mut Vec<u8>, value: u64) { out.extend_from_slice(&value.to_le_bytes()); }

fn read_u32(bytes: &[u8], offset: usize) -> Result<u32, Error> {
    let end = offset.checked_add(4).ok_or(Error::Corrupt { place: "data block offset" })?;
    Ok(u32::from_le_bytes(bytes.get(offset..end).ok_or(Error::Corrupt { place: "data block field" })?
        .try_into().map_err(|_| Error::Corrupt { place: "data block field" })?))
}

struct Cursor<'a> { bytes: &'a [u8], offset: usize, end: usize }
impl Cursor<'_> {
    fn take(&mut self, len: usize) -> Result<&[u8], Error> {
        let end = self.offset.checked_add(len).ok_or(Error::Corrupt { place: "data block cursor overflow" })?;
        if end > self.end { return Err(Error::Corrupt { place: "data block truncated" }); }
        let value = self.bytes.get(self.offset..end).ok_or(Error::Corrupt { place: "data block field" })?;
        self.offset = end;
        Ok(value)
    }
    fn varint(&mut self) -> Result<u64, Error> {
        let (value, len) = varint_get(self.bytes.get(self.offset..self.end).ok_or(Error::Corrupt { place: "data block varint" })?)
            .map_err(|_| Error::Corrupt { place: "data block varint" })?;
        self.take(len)?;
        Ok(value)
    }
    fn bytes(&mut self, limit: usize) -> Result<&[u8], Error> {
        let len = usize::try_from(self.varint()?).map_err(|_| Error::Corrupt { place: "data block field length" })?;
        if len > limit { return Err(Error::Corrupt { place: "data block field exceeds limit" }); }
        self.take(len)
    }
    fn u32(&mut self) -> Result<u32, Error> { Ok(u32::from_le_bytes(self.take(4)?.try_into().map_err(|_| Error::Corrupt { place: "data block u32" })?)) }
    fn u64(&mut self) -> Result<u64, Error> { Ok(u64::from_le_bytes(self.take(8)?.try_into().map_err(|_| Error::Corrupt { place: "data block u64" })?)) }
}
