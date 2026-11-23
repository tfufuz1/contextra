//! Run footer codec and checked section layout (R8 §5.6).

use contextra_storage_ports::Error;

pub const RUN_FOOTER_MAGIC: [u8; 8] = *b"CTXRUN01";
pub const RUN_FOOTER_LEN: usize = 44;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RunFooter {
    pub directory_offset: u64,
    pub directory_len: u64,
    pub pack_offset: u64,
    pub extension_offset: u64,
}

impl RunFooter {
    pub fn encode(self, file_len: u64) -> Result<[u8; RUN_FOOTER_LEN], Error> {
        self.validate(file_len)?;
        let mut out = [0u8; RUN_FOOTER_LEN];
        let mut cursor = 0usize;
        for value in [self.directory_offset, self.directory_len, self.pack_offset, self.extension_offset] {
            put(&mut out, &mut cursor, &value.to_le_bytes())?;
        }
        let crc = crate::codec::checksums::crc32c(out.get(..cursor).ok_or(Error::Corrupt { place: "run footer prefix" })?);
        put(&mut out, &mut cursor, &crc.to_le_bytes())?;
        put(&mut out, &mut cursor, &RUN_FOOTER_MAGIC)?;
        Ok(out)
    }

    pub fn decode(bytes: &[u8], file_len: u64) -> Result<Self, Error> {
        if bytes.len() != RUN_FOOTER_LEN { return Err(Error::Corrupt { place: "run footer length" }); }
        if bytes.get(36..44) != Some(RUN_FOOTER_MAGIC.as_slice()) { return Err(Error::Unsupported { what: "run footer magic" }); }
        let expected_crc = read_u32(bytes, 32)?;
        if crate::codec::checksums::crc32c(bytes.get(..32).ok_or(Error::Corrupt { place: "run footer CRC input" })?) != expected_crc {
            return Err(Error::Corrupt { place: "run footer CRC" });
        }
        let footer = Self {
            directory_offset: read_u64(bytes, 0)?, directory_len: read_u64(bytes, 8)?,
            pack_offset: read_u64(bytes, 16)?, extension_offset: read_u64(bytes, 24)?,
        };
        footer.validate(file_len)?;
        Ok(footer)
    }

    fn validate(self, file_len: u64) -> Result<(), Error> {
        let footer_offset = file_len.checked_sub(RUN_FOOTER_LEN as u64).ok_or(Error::Corrupt { place: "run footer file length" })?;
        let directory_end = self.directory_offset.checked_add(self.directory_len)
            .ok_or(Error::Corrupt { place: "run directory end overflow" })?;
        if self.directory_len == 0 || self.directory_offset >= self.pack_offset || directory_end > self.pack_offset
            || self.pack_offset >= self.extension_offset || self.extension_offset > footer_offset {
            return Err(Error::Corrupt { place: "run footer offsets" });
        }
        Ok(())
    }
}

fn put<const N: usize>(out: &mut [u8; RUN_FOOTER_LEN], offset: &mut usize, bytes: &[u8; N]) -> Result<(), Error> {
    let end = offset.checked_add(N).ok_or(Error::Corrupt { place: "run footer offset overflow" })?;
    out.get_mut(*offset..end).ok_or(Error::Corrupt { place: "run footer output bounds" })?.copy_from_slice(bytes);
    *offset = end;
    Ok(())
}

fn read_u32(bytes: &[u8], offset: usize) -> Result<u32, Error> {
    let end = offset.checked_add(4).ok_or(Error::Corrupt { place: "run footer field offset" })?;
    Ok(u32::from_le_bytes(bytes.get(offset..end).ok_or(Error::Corrupt { place: "run footer field" })?
        .try_into().map_err(|_| Error::Corrupt { place: "run footer field" })?))
}

fn read_u64(bytes: &[u8], offset: usize) -> Result<u64, Error> {
    let end = offset.checked_add(8).ok_or(Error::Corrupt { place: "run footer field offset" })?;
    Ok(u64::from_le_bytes(bytes.get(offset..end).ok_or(Error::Corrupt { place: "run footer field" })?
        .try_into().map_err(|_| Error::Corrupt { place: "run footer field" })?))
}
