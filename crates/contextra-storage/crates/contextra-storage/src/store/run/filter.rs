//! Salted Bloom filter v1. The containing Run section seals this byte string.

use contextra_storage_ports::Error;
use core::fmt;
use zeroize::Zeroizing;

const MAGIC: &[u8; 8] = b"CTXBLM01";
pub const FILTER_KIND_BLOOM_V1: u8 = 1;
pub const MAX_FILTER_BYTES: usize = 8 * 1024 * 1024;
const MIN_FALSE_POSITIVE: f64 = 1e-9;

#[derive(Clone, PartialEq, Eq)]
pub struct BloomFilter {
    salt: [u8; 16],
    bit_count: u32,
    hash_count: u8,
    bits: Zeroizing<Vec<u8>>,
}

impl fmt::Debug for BloomFilter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BloomFilter").field("salt", &"<redacted>")
            .field("bit_count", &self.bit_count).field("hash_count", &self.hash_count)
            .field("bits", &"<redacted>").finish()
    }
}

impl BloomFilter {
    pub fn new(expected_items: u32, false_positive_rate: f64, salt: [u8; 16]) -> Result<Self, Error> {
        if expected_items == 0 || !false_positive_rate.is_finite()
            || !(MIN_FALSE_POSITIVE..=0.01).contains(&false_positive_rate) {
            return Err(Error::Limit { limit: "Bloom filter item count or false-positive rate" });
        }
        let n = f64::from(expected_items);
        let raw_bits = (-n * false_positive_rate.ln() / (std::f64::consts::LN_2 * std::f64::consts::LN_2)).ceil();
        if !raw_bits.is_finite() || raw_bits > (MAX_FILTER_BYTES * 8) as f64 {
            return Err(Error::Limit { limit: "Bloom filter exceeds maximum size" });
        }
        let bit_count = (raw_bits as u32).max(64);
        let hashes = ((f64::from(bit_count) / n) * std::f64::consts::LN_2).round().clamp(1.0, 16.0) as u8;
        let byte_count = usize::try_from(bit_count.checked_add(7).ok_or(Error::Limit { limit: "Bloom filter bit count" })? / 8)
            .map_err(|_| Error::Limit { limit: "Bloom filter byte count" })?;
        Ok(Self { salt, bit_count, hash_count: hashes, bits: Zeroizing::new(vec![0; byte_count]) })
    }

    pub fn insert(&mut self, key: &[u8]) {
        let (first, step) = hash_pair(&self.salt, key);
        for i in 0..u64::from(self.hash_count) {
            let bit = first.wrapping_add(i.wrapping_mul(step)) % u64::from(self.bit_count);
            let byte = usize::try_from(bit / 8).unwrap_or(usize::MAX);
            if let Some(target) = self.bits.get_mut(byte) { *target |= 1u8 << (bit % 8); }
        }
    }

    pub fn may_contain(&self, key: &[u8]) -> bool {
        let (first, step) = hash_pair(&self.salt, key);
        (0..u64::from(self.hash_count)).all(|i| {
            let bit = first.wrapping_add(i.wrapping_mul(step)) % u64::from(self.bit_count);
            usize::try_from(bit / 8).ok().and_then(|byte| self.bits.get(byte))
                .is_some_and(|value| value & (1u8 << (bit % 8)) != 0)
        })
    }

    pub fn encode(&self) -> Result<Vec<u8>, Error> {
        self.validate()?;
        let mut out = Vec::with_capacity(8 + 1 + 16 + 4 + 1 + 4 + self.bits.len() + 4);
        out.extend_from_slice(MAGIC);
        out.push(FILTER_KIND_BLOOM_V1);
        out.extend_from_slice(&self.salt);
        out.extend_from_slice(&self.bit_count.to_le_bytes());
        out.push(self.hash_count);
        out.extend_from_slice(&u32::try_from(self.bits.len()).map_err(|_| Error::Limit { limit: "Bloom filter byte count" })?.to_le_bytes());
        out.extend_from_slice(&self.bits);
        let crc = crate::codec::checksums::crc32c(&out);
        out.extend_from_slice(&crc.to_le_bytes());
        Ok(out)
    }

    pub fn decode(bytes: &[u8], max_bytes: usize) -> Result<Self, Error> {
        if bytes.len() > max_bytes.min(MAX_FILTER_BYTES + 38) || bytes.len() < 38 {
            return Err(Error::Corrupt { place: "Bloom filter length" });
        }
        if bytes.get(..8) != Some(MAGIC.as_slice()) { return Err(Error::Unsupported { what: "filter block magic" }); }
        if bytes.get(8).copied() != Some(FILTER_KIND_BLOOM_V1) { return Err(Error::Unsupported { what: "filter kind" }); }
        let salt: [u8; 16] = bytes.get(9..25).ok_or(Error::Corrupt { place: "Bloom salt" })?
            .try_into().map_err(|_| Error::Corrupt { place: "Bloom salt" })?;
        let bit_count = read_u32(bytes, 25)?;
        let hash_count = *bytes.get(29).ok_or(Error::Corrupt { place: "Bloom hash count" })?;
        let byte_count = usize::try_from(read_u32(bytes, 30)?).map_err(|_| Error::Corrupt { place: "Bloom byte count" })?;
        if byte_count > MAX_FILTER_BYTES { return Err(Error::Corrupt { place: "Bloom byte count limit" }); }
        let expected_len = 34usize.checked_add(byte_count).and_then(|n| n.checked_add(4))
            .ok_or(Error::Corrupt { place: "Bloom encoded size overflow" })?;
        if expected_len != bytes.len() { return Err(Error::Corrupt { place: "Bloom encoded length" }); }
        let crc_offset = bytes.len().checked_sub(4).ok_or(Error::Corrupt { place: "Bloom CRC" })?;
        if read_u32(bytes, crc_offset)? != crate::codec::checksums::crc32c(bytes.get(..crc_offset).ok_or(Error::Corrupt { place: "Bloom CRC input" })?) {
            return Err(Error::Corrupt { place: "Bloom CRC" });
        }
        let bits = Zeroizing::new(bytes.get(34..crc_offset).ok_or(Error::Corrupt { place: "Bloom bits" })?.to_vec());
        let filter = Self { salt, bit_count, hash_count, bits };
        filter.validate()?;
        Ok(filter)
    }

    fn validate(&self) -> Result<(), Error> {
        if self.bit_count < 64 || self.hash_count == 0 || self.hash_count > 16
            || self.bits.len() != usize::try_from(self.bit_count.checked_add(7).ok_or(Error::Corrupt { place: "Bloom bit count" })? / 8)
                .map_err(|_| Error::Corrupt { place: "Bloom byte count" })? {
            return Err(Error::Corrupt { place: "Bloom filter parameters" });
        }
        if self.bits.len() > MAX_FILTER_BYTES { return Err(Error::Corrupt { place: "Bloom filter size limit" }); }
        Ok(())
    }
}

fn hash_pair(salt: &[u8; 16], key: &[u8]) -> (u64, u64) {
    let first = siphash24(salt, key);
    let mut domain = Zeroizing::new(Vec::with_capacity(key.len().saturating_add(1)));
    domain.push(1);
        domain.extend_from_slice(key);
    let second = siphash24(salt, &domain) | 1;
    (first, second)
}

// SipHash-2-4 is used only to distribute Bloom bits; it is not an integrity
// primitive. The salt is persisted inside the cluster-sealed FilterBlock.
fn siphash24(key: &[u8; 16], data: &[u8]) -> u64 {
    let k0 = u64::from_le_bytes(key.get(..8).and_then(|bytes| bytes.try_into().ok()).unwrap_or([0; 8]));
    let k1 = u64::from_le_bytes(key.get(8..).and_then(|bytes| bytes.try_into().ok()).unwrap_or([0; 8]));
    let mut v0 = 0x736f6d6570736575 ^ k0;
    let mut v1 = 0x646f72616e646f6d ^ k1;
    let mut v2 = 0x6c7967656e657261 ^ k0;
    let mut v3 = 0x7465646279746573 ^ k1;
    let mut chunks = data.chunks_exact(8);
    for chunk in &mut chunks {
        let message = u64::from_le_bytes(chunk.try_into().unwrap_or([0; 8]));
        v3 ^= message;
        sip_round(&mut v0, &mut v1, &mut v2, &mut v3);
        sip_round(&mut v0, &mut v1, &mut v2, &mut v3);
        v0 ^= message;
    }
    let tail = chunks.remainder();
    let mut last = (data.len() as u64) << 56;
    for (index, byte) in tail.iter().enumerate() { last |= u64::from(*byte) << (index * 8); }
    v3 ^= last;
    sip_round(&mut v0, &mut v1, &mut v2, &mut v3);
    sip_round(&mut v0, &mut v1, &mut v2, &mut v3);
    v0 ^= last;
    v2 ^= 0xff;
    for _ in 0..4 { sip_round(&mut v0, &mut v1, &mut v2, &mut v3); }
    v0 ^ v1 ^ v2 ^ v3
}

fn sip_round(v0: &mut u64, v1: &mut u64, v2: &mut u64, v3: &mut u64) {
    *v0 = v0.wrapping_add(*v1); *v1 = v1.rotate_left(13); *v1 ^= *v0; *v0 = v0.rotate_left(32);
    *v2 = v2.wrapping_add(*v3); *v3 = v3.rotate_left(16); *v3 ^= *v2;
    *v0 = v0.wrapping_add(*v3); *v3 = v3.rotate_left(21); *v3 ^= *v0;
    *v2 = v2.wrapping_add(*v1); *v1 = v1.rotate_left(17); *v1 ^= *v2; *v2 = v2.rotate_left(32);
}

fn read_u32(bytes: &[u8], offset: usize) -> Result<u32, Error> {
    let end = offset.checked_add(4).ok_or(Error::Corrupt { place: "Bloom field offset" })?;
    Ok(u32::from_le_bytes(bytes.get(offset..end).ok_or(Error::Corrupt { place: "Bloom field" })?
        .try_into().map_err(|_| Error::Corrupt { place: "Bloom field" })?))
}
