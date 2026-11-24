//! Background block scrubber and checksum verification (SPEC §16, SP-MAINT).
#![forbid(unsafe_code)]

use crate::codec::checksums::crc32c;
use contextra_storage_ports::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScrubReport {
    pub blocks_checked: u64,
    pub bytes_checked: u64,
    pub corrupted_blocks: u64,
}

pub struct Scrubber;

impl Scrubber {
    /// Verifies block integrity using raw stored CRC32C without decrypting payload.
    pub fn scrub_block(raw_block_bytes: &[u8]) -> Result<bool, Error> {
        if raw_block_bytes.len() < 4 {
            return Err(Error::Corrupt {
                place: "scrub block too short for checksum",
            });
        }
        let data_len = raw_block_bytes.len() - 4;
        let payload = &raw_block_bytes[..data_len];
        let expected_crc = u32::from_be_bytes(
            raw_block_bytes[data_len..]
                .try_into()
                .map_err(|_| Error::Corrupt {
                    place: "scrub block crc parse",
                })?,
        );
        let computed_crc = crc32c(payload);
        Ok(computed_crc == expected_crc)
    }
}
