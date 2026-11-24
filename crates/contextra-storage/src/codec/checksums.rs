//! CRC32C (Castagnoli), reflected form, used for accidental corruption checks.

const CRC32C_POLY: u32 = 0x82f6_3b78;

pub fn crc32c(bytes: &[u8]) -> u32 {
    let mut crc = !0u32;
    for byte in bytes {
        crc ^= u32::from(*byte);
        for _ in 0..8 {
            let mask = 0u32.wrapping_sub(crc & 1);
            crc = (crc >> 1) ^ (CRC32C_POLY & mask);
        }
    }
    !crc
}

pub fn append_crc32c(out: &mut Vec<u8>, bytes: &[u8]) {
    out.extend_from_slice(&crc32c(bytes).to_le_bytes());
}

pub fn verify_crc32c(bytes: &[u8], expected_le: [u8; 4]) -> bool {
    crc32c(bytes) == u32::from_le_bytes(expected_le)
}
