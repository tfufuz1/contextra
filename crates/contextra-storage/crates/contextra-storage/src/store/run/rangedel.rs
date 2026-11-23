//! Cluster-section range-delete markers (R8 §4.2 and §5.6).

use crate::codec::tags::{untag, Kind, TagError};
use contextra_storage_ports::{BranchId, Error, Stamp};
use core::fmt;
use zeroize::Zeroizing;

const MAGIC: &[u8; 8] = b"CTXRNG01";
pub const MAX_RANGE_MARKERS: usize = 1_000_000;

#[derive(Clone, PartialEq, Eq)]
pub struct RangeMarker {
    pub branch: BranchId,
    pub start: Zeroizing<Vec<u8>>,
    /// Exclusive upper bound.
    pub end: Zeroizing<Vec<u8>>,
    pub tag: u64,
}

impl fmt::Debug for RangeMarker {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RangeMarker").field("branch", &self.branch)
            .field("start", &"<redacted>").field("end", &"<redacted>")
            .field("tag", &self.tag).finish()
    }
}

impl RangeMarker {
    pub fn stamp(&self) -> Result<Stamp, Error> {
        let (stamp, kind) = untag(self.tag).map_err(|error| match error {
            TagError::UnknownKind(_) | TagError::ReservedMerge => Error::Unsupported { what: "range marker tag" },
            TagError::StampZero | TagError::StampOutOfRange => Error::Corrupt { place: "range marker tag" },
        })?;
        if kind != Kind::RangeDelete { return Err(Error::Corrupt { place: "range marker kind" }); }
        Ok(stamp)
    }
}

pub fn encode(markers: &[RangeMarker], max_bytes: usize) -> Result<Vec<u8>, Error> {
    validate(markers)?;
    if markers.len() > MAX_RANGE_MARKERS { return Err(Error::Limit { limit: "range marker count" }); }
    let mut out = Vec::new();
    out.extend_from_slice(MAGIC);
    put_u32(&mut out, u32::try_from(markers.len()).map_err(|_| Error::Limit { limit: "range marker count" })?);
    for marker in markers {
        put_u32(&mut out, marker.branch.0);
        put_key(&mut out, &marker.start)?;
        put_key(&mut out, &marker.end)?;
        out.extend_from_slice(&marker.tag.to_le_bytes());
        if out.len().checked_add(4).is_none_or(|len| len > max_bytes) {
            return Err(Error::Limit { limit: "range marker block size" });
        }
    }
    let crc = crate::codec::checksums::crc32c(&out);
    put_u32(&mut out, crc);
    if out.len() > max_bytes { return Err(Error::Limit { limit: "range marker block size" }); }
    Ok(out)
}

pub fn decode(bytes: &[u8], max_bytes: usize, max_markers: usize) -> Result<Vec<RangeMarker>, Error> {
    if bytes.len() > max_bytes || bytes.len() < 16 { return Err(Error::Corrupt { place: "range marker block length" }); }
    if bytes.get(..8) != Some(MAGIC.as_slice()) { return Err(Error::Unsupported { what: "range marker block magic" }); }
    let payload_end = bytes.len().checked_sub(4).ok_or(Error::Corrupt { place: "range marker CRC" })?;
    if read_u32(bytes, payload_end)? != crate::codec::checksums::crc32c(bytes.get(..payload_end).ok_or(Error::Corrupt { place: "range marker CRC input" })?) {
        return Err(Error::Corrupt { place: "range marker CRC" });
    }
    let count = read_u32(bytes, 8)? as usize;
    if count > max_markers.min(MAX_RANGE_MARKERS) { return Err(Error::Corrupt { place: "range marker count limit" }); }
    let mut cursor = Cursor { bytes, offset: 12, end: payload_end, max_key: max_bytes };
    let mut markers = Vec::with_capacity(count);
    for _ in 0..count {
        markers.push(RangeMarker {
            branch: BranchId(cursor.u32()?),
            start: Zeroizing::new(cursor.key()?),
            end: Zeroizing::new(cursor.key()?),
            tag: cursor.u64()?,
        });
    }
    if cursor.offset != payload_end { return Err(Error::Corrupt { place: "range marker trailing bytes" }); }
    validate(&markers)?;
    Ok(markers)
}

fn validate(markers: &[RangeMarker]) -> Result<(), Error> {
    if markers.len() > MAX_RANGE_MARKERS { return Err(Error::Limit { limit: "range marker count" }); }
    for marker in markers {
        if marker.start.as_slice() >= marker.end.as_slice() {
            return Err(Error::Corrupt { place: "range marker bounds" });
        }
        let _ = marker.stamp()?;
    }
    for pair in markers.windows(2) {
        let left = pair.first().ok_or(Error::Corrupt { place: "range marker pair" })?;
        let right = pair.get(1).ok_or(Error::Corrupt { place: "range marker pair" })?;
        if (left.branch, left.start.as_slice(), left.end.as_slice(), left.tag)
            >= (right.branch, right.start.as_slice(), right.end.as_slice(), right.tag) {
            return Err(Error::Corrupt { place: "range marker order" });
        }
    }
    Ok(())
}

fn put_key(out: &mut Vec<u8>, key: &[u8]) -> Result<(), Error> {
    out.extend_from_slice(&u32::try_from(key.len()).map_err(|_| Error::Limit { limit: "range marker key length" })?.to_le_bytes());
    out.extend_from_slice(key);
    Ok(())
}
fn put_u32(out: &mut Vec<u8>, value: u32) { out.extend_from_slice(&value.to_le_bytes()); }
fn read_u32(bytes: &[u8], at: usize) -> Result<u32, Error> {
    let end = at.checked_add(4).ok_or(Error::Corrupt { place: "range marker offset" })?;
    Ok(u32::from_le_bytes(bytes.get(at..end).ok_or(Error::Corrupt { place: "range marker field" })?
        .try_into().map_err(|_| Error::Corrupt { place: "range marker field" })?))
}

struct Cursor<'a> { bytes: &'a [u8], offset: usize, end: usize, max_key: usize }
impl Cursor<'_> {
    fn take(&mut self, len: usize) -> Result<&[u8], Error> {
        let end = self.offset.checked_add(len).ok_or(Error::Corrupt { place: "range marker offset overflow" })?;
        if end > self.end { return Err(Error::Corrupt { place: "range marker truncated" }); }
        let value = self.bytes.get(self.offset..end).ok_or(Error::Corrupt { place: "range marker field" })?;
        self.offset = end;
        Ok(value)
    }
    fn array<const N: usize>(&mut self) -> Result<[u8; N], Error> { self.take(N)?.try_into().map_err(|_| Error::Corrupt { place: "range marker field" }) }
    fn u32(&mut self) -> Result<u32, Error> { Ok(u32::from_le_bytes(self.array()?)) }
    fn u64(&mut self) -> Result<u64, Error> { Ok(u64::from_le_bytes(self.array()?)) }
    fn key(&mut self) -> Result<Vec<u8>, Error> {
        let len = usize::try_from(self.u32()?).map_err(|_| Error::Corrupt { place: "range marker key length" })?;
        if len > self.max_key { return Err(Error::Corrupt { place: "range marker key limit" }); }
        Ok(self.take(len)?.to_vec())
    }
}
