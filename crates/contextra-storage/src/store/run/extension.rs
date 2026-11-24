//! Opaque, versioned ExtensionBlock records (R8 §5.6, SP-EXT boundary).

use contextra_storage_ports::Error;
use core::fmt;
use zeroize::Zeroizing;

const MAGIC: &[u8; 8] = b"CTXEXT01";
pub const MAX_EXTENSION_BLOCKS: usize = 4096;

#[derive(Clone, PartialEq, Eq)]
pub struct ExtensionBlock {
    pub tag: u16,
    pub version: u16,
    pub payload: Zeroizing<Vec<u8>>,
}

impl fmt::Debug for ExtensionBlock {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ExtensionBlock").field("tag", &self.tag)
            .field("version", &self.version).field("payload", &"<redacted>").finish()
    }
}

impl ExtensionBlock {
    pub fn new(tag: u16, version: u16, payload: Vec<u8>) -> Result<Self, Error> {
        if tag == 0 || version == 0 { return Err(Error::Limit { limit: "extension tag and version must be nonzero" }); }
        Ok(Self { tag, version, payload: Zeroizing::new(payload) })
    }
}

pub fn encode(blocks: &[ExtensionBlock], max_bytes: usize) -> Result<Vec<u8>, Error> {
    if blocks.len() > MAX_EXTENSION_BLOCKS { return Err(Error::Limit { limit: "extension block count" }); }
    let mut out = Vec::new();
    out.extend_from_slice(MAGIC);
    out.extend_from_slice(&u32::try_from(blocks.len()).map_err(|_| Error::Limit { limit: "extension block count" })?.to_le_bytes());
    for block in blocks {
        if block.tag == 0 || block.version == 0 { return Err(Error::Limit { limit: "extension tag and version must be nonzero" }); }
        let payload_len = u32::try_from(block.payload.len()).map_err(|_| Error::Limit { limit: "extension payload length" })?;
        out.extend_from_slice(&block.tag.to_le_bytes());
        out.extend_from_slice(&block.version.to_le_bytes());
        out.extend_from_slice(&payload_len.to_le_bytes());
        out.extend_from_slice(&block.payload);
        if out.len().checked_add(4).is_none_or(|len| len > max_bytes) {
            return Err(Error::Limit { limit: "extension region size" });
        }
    }
    let crc = crate::codec::checksums::crc32c(&out);
    out.extend_from_slice(&crc.to_le_bytes());
    if out.len() > max_bytes { return Err(Error::Limit { limit: "extension region size" }); }
    Ok(out)
}

pub fn decode(bytes: &[u8], max_bytes: usize, max_blocks: usize) -> Result<Vec<ExtensionBlock>, Error> {
    if bytes.len() > max_bytes || bytes.len() < 16 { return Err(Error::Corrupt { place: "extension region length" }); }
    if bytes.get(..8) != Some(MAGIC.as_slice()) { return Err(Error::Unsupported { what: "extension region magic" }); }
    let end = bytes.len().checked_sub(4).ok_or(Error::Corrupt { place: "extension CRC" })?;
    if read_u32(bytes, end)? != crate::codec::checksums::crc32c(bytes.get(..end).ok_or(Error::Corrupt { place: "extension CRC input" })?) {
        return Err(Error::Corrupt { place: "extension CRC" });
    }
    let count = read_u32(bytes, 8)? as usize;
    if count > max_blocks.min(MAX_EXTENSION_BLOCKS) { return Err(Error::Corrupt { place: "extension block count limit" }); }
    if count.checked_mul(8).and_then(|n| n.checked_add(16)).is_none_or(|n| n > bytes.len()) {
        return Err(Error::Corrupt { place: "extension block count bounds" });
    }
    let mut cursor = Cursor { bytes, offset: 12, end };
    let mut blocks = Vec::with_capacity(count);
    for _ in 0..count {
        let tag = cursor.u16()?;
        let version = cursor.u16()?;
        let len = usize::try_from(cursor.u32()?).map_err(|_| Error::Corrupt { place: "extension payload length" })?;
        if tag == 0 || version == 0 { return Err(Error::Corrupt { place: "extension tag/version" }); }
        blocks.push(ExtensionBlock { tag, version, payload: Zeroizing::new(cursor.take(len)?.to_vec()) });
    }
    if cursor.offset != end { return Err(Error::Corrupt { place: "extension trailing bytes" }); }
    Ok(blocks)
}

fn read_u32(bytes: &[u8], offset: usize) -> Result<u32, Error> {
    let end = offset.checked_add(4).ok_or(Error::Corrupt { place: "extension offset" })?;
    Ok(u32::from_le_bytes(bytes.get(offset..end).ok_or(Error::Corrupt { place: "extension field" })?
        .try_into().map_err(|_| Error::Corrupt { place: "extension field" })?))
}

struct Cursor<'a> { bytes: &'a [u8], offset: usize, end: usize }
impl Cursor<'_> {
    fn take(&mut self, len: usize) -> Result<&[u8], Error> {
        let end = self.offset.checked_add(len).ok_or(Error::Corrupt { place: "extension offset overflow" })?;
        if end > self.end { return Err(Error::Corrupt { place: "extension truncated" }); }
        let value = self.bytes.get(self.offset..end).ok_or(Error::Corrupt { place: "extension field" })?;
        self.offset = end;
        Ok(value)
    }
    fn u16(&mut self) -> Result<u16, Error> { Ok(u16::from_le_bytes(self.take(2)?.try_into().map_err(|_| Error::Corrupt { place: "extension field" })?)) }
    fn u32(&mut self) -> Result<u32, Error> { Ok(u32::from_le_bytes(self.take(4)?.try_into().map_err(|_| Error::Corrupt { place: "extension field" })?)) }
}
