//! Konstanten und Geometrie eines vorallokierten Redo-Segments.

use contextra_storage_ports::Error;

pub const DEFAULT_SEGMENT_BYTES: u64 = 64 * 1024 * 1024;
pub const DEFAULT_MARK_BYTES: u64 = 8 * 1024 * 1024;
pub const DEFAULT_ALIGNMENT: u64 = 4096;
pub const GROUP_MAGIC: &[u8; 4] = b"R8G1";

pub fn align_up(value: u64, alignment: u64) -> Result<u64, Error> {
    if alignment == 0 || !alignment.is_power_of_two() {
        return Err(Error::Limit { limit: "redo alignment must be a power of two" });
    }
    value.checked_add(alignment - 1)
        .map(|rounded| rounded & !(alignment - 1))
        .ok_or(Error::Limit { limit: "redo segment offset" })
}

pub fn group_padding(offset: u64, alignment: u64) -> Result<usize, Error> {
    let aligned = align_up(offset, alignment)?;
    usize::try_from(aligned - offset).map_err(|_| Error::Limit { limit: "redo alignment padding" })
}
