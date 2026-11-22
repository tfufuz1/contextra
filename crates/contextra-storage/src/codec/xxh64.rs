//! # XXH64
//!
//! Portables 64-Bit-XXHash nach Spezifikation (SD-04, SPEC §9.1).
//! Verifiziert gegen Referenzvektoren aus SPEC Anhang A.1:
//! - XXH64("", 0) = 0xEF46DB3751D8E999
//! - XXH64("a", 0) = 0xD24EC4F1A98C6E5B
//! - XXH64("abc", 0) = 0x44BC2CF5AD770999
//! Keine Integer-Casts (M-02 = 0).


const PRIME64_1: u64 = 0x9E3779B185EBCA87;
const PRIME64_2: u64 = 0xC2B2AE3D27D4EB4F;
const PRIME64_3: u64 = 0x165667B19E3779F9;
const PRIME64_4: u64 = 0x85EBCA77C2B2AE63;
const PRIME64_5: u64 = 0x27D4EB2F165667C5;

#[inline]
fn round(acc: u64, input: u64) -> u64 {
    acc.wrapping_add(input.wrapping_mul(PRIME64_2))
        .rotate_left(31)
        .wrapping_mul(PRIME64_1)
}

#[inline]
fn merge_round(acc: u64, val: u64) -> u64 {
    (acc ^ round(0, val))
        .wrapping_mul(PRIME64_1)
        .wrapping_add(PRIME64_4)
}

/// Berechnet den 64-Bit-XXHash für einen Byte-Slice mit Seed.
pub fn xxh64(data: &[u8], seed: u64) -> u64 {
    let len = data.len();
    let mut h64: u64;

    if len >= 32 {
        let mut v1 = seed.wrapping_add(PRIME64_1).wrapping_add(PRIME64_2);
        let mut v2 = seed.wrapping_add(PRIME64_2);
        let mut v3 = seed;
        let mut v4 = seed.wrapping_sub(PRIME64_1);

        let mut offset = 0usize;
        while offset.saturating_add(32) <= len {
            if let (Some(s1), Some(s2), Some(s3), Some(s4)) = (
                data.get(offset..offset + 8),
                data.get(offset + 8..offset + 16),
                data.get(offset + 16..offset + 24),
                data.get(offset + 24..offset + 32),
            ) {
                v1 = round(v1, read_u64_le(s1));
                v2 = round(v2, read_u64_le(s2));
                v3 = round(v3, read_u64_le(s3));
                v4 = round(v4, read_u64_le(s4));
            }
            offset = offset.saturating_add(32);
        }

        h64 = v1.rotate_left(1)
            .wrapping_add(v2.rotate_left(7))
            .wrapping_add(v3.rotate_left(12))
            .wrapping_add(v4.rotate_left(18));

        h64 = merge_round(h64, v1);
        h64 = merge_round(h64, v2);
        h64 = merge_round(h64, v3);
        h64 = merge_round(h64, v4);
    } else {
        h64 = seed.wrapping_add(PRIME64_5);
    }

    let len_u64 = u64::try_from(len).unwrap_or(0);
    h64 = h64.wrapping_add(len_u64);

    let mut rem = len % 32;
    let mut offset = len.saturating_sub(rem);

    while rem >= 8 {
        if let Some(slice) = data.get(offset..offset + 8) {
            let k1 = round(0, read_u64_le(slice));
            h64 = (h64 ^ k1)
                .rotate_left(27)
                .wrapping_mul(PRIME64_1)
                .wrapping_add(PRIME64_4);
        }
        offset = offset.saturating_add(8);
        rem = rem.saturating_sub(8);
    }

    if rem >= 4 {
        if let Some(slice) = data.get(offset..offset + 4) {
            let k1 = u64::from(read_u32_le(slice)).wrapping_mul(PRIME64_1);
            h64 = (h64 ^ k1)
                .rotate_left(23)
                .wrapping_mul(PRIME64_2)
                .wrapping_add(PRIME64_3);
        }
        offset = offset.saturating_add(4);
        rem = rem.saturating_sub(4);
    }

    while rem > 0 {
        if let Some(&byte) = data.get(offset) {
            let k1 = u64::from(byte).wrapping_mul(PRIME64_5);
            h64 = (h64 ^ k1).rotate_left(11).wrapping_mul(PRIME64_1);
        }
        offset = offset.saturating_add(1);
        rem = rem.saturating_sub(1);
    }

    // Avalanche
    h64 ^= h64 >> 33;
    h64 = h64.wrapping_mul(PRIME64_2);
    h64 ^= h64 >> 29;
    h64 = h64.wrapping_mul(PRIME64_3);
    h64 ^= h64 >> 32;

    h64
}

#[inline]
fn read_u64_le(bytes: &[u8]) -> u64 {
    let mut arr = [0u8; 8];
    arr.copy_from_slice(bytes);
    u64::from_le_bytes(arr)
}

#[inline]
fn read_u32_le(bytes: &[u8]) -> u32 {
    let mut arr = [0u8; 4];
    arr.copy_from_slice(bytes);
    u32::from_le_bytes(arr)
}

#[cfg(test)]
#[allow(clippy::indexing_slicing)]
mod tests {
    use super::*;

    #[test]
    fn test_golden_a1_xxh64() {
        assert_eq!(xxh64(b"", 0), 0xEF46DB3751D8E999);
        assert_eq!(xxh64(b"a", 0), 0xD24EC4F1A98C6E5B);
        assert_eq!(xxh64(b"abc", 0), 0x44BC2CF5AD770999);
    }
}
