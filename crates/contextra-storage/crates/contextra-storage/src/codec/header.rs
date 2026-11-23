//! Fixed-width file header from R8 §5.3 (little-endian).

use contextra_storage_ports::Error;

pub const HEADER_LEN: usize = 43;
pub const HEADER_VERSION: u16 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FileHeader {
    pub magic: [u8; 8],
    pub file_kind: u8,
    pub format_ver: u16,
    pub feature_bits: u32,
    pub file_id: u64,
    pub instance_id: [u8; 16],
}

impl FileHeader {
    pub fn encode(&self) -> Result<[u8; HEADER_LEN], Error> {
        let mut out = [0u8; HEADER_LEN];
        let mut offset = 0usize;
        write(&mut out, &mut offset, &self.magic);
        write(&mut out, &mut offset, &[self.file_kind]);
        write(&mut out, &mut offset, &self.format_ver.to_le_bytes());
        write(&mut out, &mut offset, &self.feature_bits.to_le_bytes());
        write(&mut out, &mut offset, &self.file_id.to_le_bytes());
        write(&mut out, &mut offset, &self.instance_id);
        let prefix = out.get(..offset).ok_or(Error::Corrupt { place: "file header encode offset" })?;
        let crc = super::checksums::crc32c(prefix);
        write(&mut out, &mut offset, &crc.to_le_bytes());
        Ok(out)
    }

    pub fn decode(input: &[u8], expected_magic: [u8; 8], expected_kind: u8, supported_features: u32) -> Result<Self, Error> {
        if input.len() < HEADER_LEN { return Err(Error::Corrupt { place: "file header truncated" }); }
        let bytes = input.get(..HEADER_LEN).ok_or(Error::Corrupt { place: "file header" })?;
        if bytes.get(..8) != Some(expected_magic.as_slice()) { return Err(Error::Unsupported { what: "file magic" }); }
        let version = read_u16(bytes, 9)?;
        if version != HEADER_VERSION { return Err(Error::Unsupported { what: "file header version" }); }
        if bytes.get(8).copied() != Some(expected_kind) { return Err(Error::Unsupported { what: "file kind" }); }
        let features = read_u32(bytes, 11)?;
        if features & !supported_features != 0 { return Err(Error::Unsupported { what: "required file feature" }); }
        let expected_crc = read_u32(bytes, 39)?;
        if super::checksums::crc32c(bytes.get(..39).ok_or(Error::Corrupt { place: "file header CRC input" })?) != expected_crc {
            return Err(Error::Corrupt { place: "file header CRC" });
        }
        Ok(Self {
            magic: bytes.get(..8).ok_or(Error::Corrupt { place: "file magic" })?.try_into().map_err(|_| Error::Corrupt { place: "file magic" })?,
            file_kind: *bytes.get(8).ok_or(Error::Corrupt { place: "file kind" })?,
            format_ver: version,
            feature_bits: features,
            file_id: read_u64(bytes, 15)?,
            instance_id: bytes.get(23..39).ok_or(Error::Corrupt { place: "instance id" })?.try_into().map_err(|_| Error::Corrupt { place: "instance id" })?,
        })
    }
}

fn write(dst: &mut [u8], offset: &mut usize, src: &[u8]) {
    for (target, value) in dst.iter_mut().skip(*offset).zip(src.iter().copied()) { *target = value; }
    *offset = offset.saturating_add(src.len());
}

fn read_u16(b: &[u8], at: usize) -> Result<u16, Error> {
    Ok(u16::from_le_bytes(b.get(at..at + 2).ok_or(Error::Corrupt { place: "file header field" })?
        .try_into().map_err(|_| Error::Corrupt { place: "file header field" })?))
}
fn read_u32(b: &[u8], at: usize) -> Result<u32, Error> {
    Ok(u32::from_le_bytes(b.get(at..at + 4).ok_or(Error::Corrupt { place: "file header field" })?
        .try_into().map_err(|_| Error::Corrupt { place: "file header field" })?))
}
fn read_u64(b: &[u8], at: usize) -> Result<u64, Error> {
    Ok(u64::from_le_bytes(b.get(at..at + 8).ok_or(Error::Corrupt { place: "file header field" })?
        .try_into().map_err(|_| Error::Corrupt { place: "file header field" })?))
}
