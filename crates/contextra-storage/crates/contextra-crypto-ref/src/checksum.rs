//! CRC32C (Castagnoli) reference implementation.
use contextra_storage_ports::traits::Checksum;

#[derive(Debug, Default, Clone, Copy)]
pub struct Crc32c;

impl Checksum for Crc32c {
    fn crc32c(&self, data: &[u8]) -> u32 {
        let mut crc = !0u32;
        for byte in data {
            crc ^= u32::from(*byte);
            for _ in 0..8 {
                let mask = 0u32.wrapping_sub(crc & 1);
                crc = (crc >> 1) ^ (0x82f6_3b78 & mask);
            }
        }
        !crc
    }
}
