//! Position-independent blob record framing from R8 §5.7.

use contextra_storage_ports::Error;
use core::fmt;
use zeroize::Zeroizing;

pub const RECORD_UID_LEN: usize = 16;
pub const RECORD_OVERHEAD: usize = RECORD_UID_LEN + 4 + 4;

#[derive(Clone, PartialEq, Eq)]
pub struct BlobRecord {
    pub record_uid: [u8; RECORD_UID_LEN],
    pub seal_frame: Zeroizing<Vec<u8>>,
}

impl fmt::Debug for BlobRecord {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BlobRecord").field("record_uid", &"<redacted>")
            .field("seal_frame", &"<redacted>").finish()
    }
}

impl BlobRecord {
    pub fn encode(&self, max_frame_bytes: usize) -> Result<Vec<u8>, Error> {
        if self.seal_frame.is_empty() || self.seal_frame.len() > max_frame_bytes {
            return Err(Error::Limit { limit: "blob seal frame length" });
        }
        let frame_len = u32::try_from(self.seal_frame.len()).map_err(|_| Error::Limit { limit: "blob seal frame length" })?;
        let capacity = RECORD_OVERHEAD.checked_add(self.seal_frame.len()).ok_or(Error::Limit { limit: "blob record size" })?;
        let mut out = Vec::with_capacity(capacity);
        out.extend_from_slice(&self.record_uid);
        out.extend_from_slice(&frame_len.to_le_bytes());
        out.extend_from_slice(&self.seal_frame);
        let crc = crate::codec::checksums::crc32c(&out);
        out.extend_from_slice(&crc.to_le_bytes());
        Ok(out)
    }

    pub fn decode(bytes: &[u8], max_frame_bytes: usize) -> Result<Self, Error> {
        if bytes.len() < RECORD_OVERHEAD { return Err(Error::Corrupt { place: "blob record truncated" }); }
        let frame_len = usize::try_from(read_u32(bytes, RECORD_UID_LEN)?).map_err(|_| Error::Corrupt { place: "blob frame length" })?;
        if frame_len == 0 || frame_len > max_frame_bytes { return Err(Error::Corrupt { place: "blob frame length limit" }); }
        let expected_len = RECORD_OVERHEAD.checked_add(frame_len).ok_or(Error::Corrupt { place: "blob record length overflow" })?;
        if bytes.len() != expected_len { return Err(Error::Corrupt { place: "blob record encoded length" }); }
        let crc_offset = bytes.len().checked_sub(4).ok_or(Error::Corrupt { place: "blob record CRC" })?;
        if read_u32(bytes, crc_offset)? != crate::codec::checksums::crc32c(bytes.get(..crc_offset).ok_or(Error::Corrupt { place: "blob record CRC input" })?) {
            return Err(Error::Corrupt { place: "blob record CRC" });
        }
        let record_uid = bytes.get(..RECORD_UID_LEN).ok_or(Error::Corrupt { place: "blob record uid" })?
            .try_into().map_err(|_| Error::Corrupt { place: "blob record uid" })?;
        let seal_end = RECORD_UID_LEN.checked_add(4).and_then(|v| v.checked_add(frame_len))
            .ok_or(Error::Corrupt { place: "blob seal frame end" })?;
        let seal_frame = Zeroizing::new(bytes.get(RECORD_UID_LEN + 4..seal_end)
            .ok_or(Error::Corrupt { place: "blob seal frame" })?.to_vec());
        Ok(Self { record_uid, seal_frame })
    }
}

fn read_u32(bytes: &[u8], offset: usize) -> Result<u32, Error> {
    let end = offset.checked_add(4).ok_or(Error::Corrupt { place: "blob record offset" })?;
    Ok(u32::from_le_bytes(bytes.get(offset..end).ok_or(Error::Corrupt { place: "blob record field" })?
        .try_into().map_err(|_| Error::Corrupt { place: "blob record field" })?))
}
