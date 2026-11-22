//! # Bloom-Filter
//!
//! Doppel-Hashing über XXH64 nach SPEC §9.5 und Anhang A.6.
//! Layout: `k u8 | nbits u64 | bits[nbits / 8]`
//! Keine Integer-Casts (M-02 = 0).

use super::xxh64::xxh64;
use std::fmt;

const SEED2: u64 = 0x9E3779B97F4A7C15;

/// Fehler beim Dekodieren eines Bloom-Filters.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BloomError {
    /// Puffer zu kurz für Header oder Bits.
    UnexpectedEof,
    /// Ungültiges k oder unpassende Bitanzahl.
    Corrupt,
}

impl fmt::Display for BloomError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedEof => write!(f, "Unerwartetes Ende beim Dekodieren des Bloom-Filters"),
            Self::Corrupt => write!(f, "Korrupter Bloom-Filter"),
        }
    }
}

impl std::error::Error for BloomError {}

/// Ein dynamischer Bloom-Filter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BloomFilter {
    k: u8,
    nbits: u64,
    bits: Vec<u8>,
}

impl BloomFilter {
    /// Erstellt einen neuen Bloom-Filter für `key_count` Schlüssel mit `bits_per_key`.
    pub fn new(key_count: usize, bits_per_key: usize) -> Self {
        let bits_per_key = if bits_per_key == 0 { 10 } else { bits_per_key };
        let count_u64 = u64::try_from(key_count).unwrap_or(u64::MAX);
        let bpk_u64 = u64::try_from(bits_per_key).unwrap_or(10);
        let raw_bits = count_u64.saturating_mul(bpk_u64);
        let min_bits = std::cmp::max(64, raw_bits);
        let nbits = min_bits.div_ceil(64).saturating_mul(64);
        let byte_len = usize::try_from(nbits / 8).unwrap_or(8);

        Self {
            k: 7,
            nbits,
            bits: vec![0u8; byte_len],
        }
    }

    /// Erstellt einen Filter mit expliziter Bitanzahl (z. B. für Tests oder Einlesen).
    pub fn with_explicit_bits(k: u8, nbits: u64, bits: Vec<u8>) -> Self {
        Self { k, nbits, bits }
    }

    /// Kodiert den Bloom-Filter in ein Byte-Vektor nach SPEC §9.5.
    ///
    /// Layout: `k u8 | nbits u64 | bits`
    pub fn encode(&self) -> Vec<u8> {
        let nbits_bytes = self.nbits.to_le_bytes();
        let total_len = 1usize
            .checked_add(nbits_bytes.len())
            .and_then(|l| l.checked_add(self.bits.len()))
            .unwrap_or(0);
        let mut out = Vec::with_capacity(total_len);
        out.push(self.k);
        out.extend_from_slice(&nbits_bytes);
        out.extend_from_slice(&self.bits);
        out
    }

    /// Dekodiert einen Bloom-Filter total aus Bytes.
    pub fn decode(bytes: &[u8]) -> Result<Self, BloomError> {
        if bytes.len() < 9 {
            return Err(BloomError::UnexpectedEof);
        }
        let k = *bytes.get(0).ok_or(BloomError::UnexpectedEof)?;
        if k == 0 {
            return Err(BloomError::Corrupt);
        }

        let mut nbits_arr = [0u8; 8];
        let nbits_slice = bytes.get(1..9).ok_or(BloomError::UnexpectedEof)?;
        nbits_arr.copy_from_slice(nbits_slice);
        let nbits = u64::from_le_bytes(nbits_arr);

        let expected_bytes = usize::try_from(nbits.div_ceil(8)).map_err(|_| BloomError::Corrupt)?;
        let bits_slice = bytes.get(9..).ok_or(BloomError::UnexpectedEof)?;

        if bits_slice.len() != expected_bytes {
            return Err(BloomError::Corrupt);
        }

        Ok(Self {
            k,
            nbits,
            bits: bits_slice.to_vec(),
        })
    }

    /// Fügt einen Nutzerschlüssel in den Bloom-Filter ein.
    pub fn insert(&mut self, key: &[u8]) {
        let h1 = xxh64(key, 0);
        let h2 = xxh64(key, SEED2) | 1;

        for i in 0..self.k {
            let offset = u64::from(i).wrapping_mul(h2);
            let pos = h1.wrapping_add(offset) % self.nbits;
            if let Ok(byte_idx) = usize::try_from(pos / 8) {
                if let Ok(bit_idx) = u32::try_from(pos % 8) {
                    let bit_mask = 1u8 << bit_idx;
                    if let Some(b) = self.bits.get_mut(byte_idx) {
                        *b |= bit_mask;
                    }
                }
            }
        }
    }

    /// Prüft, ob ein Schlüssel möglicherweise im Filter enthalten ist.
    pub fn may_contain(&self, key: &[u8]) -> bool {
        if self.bits.is_empty() || self.nbits == 0 {
            return false;
        }

        let h1 = xxh64(key, 0);
        let h2 = xxh64(key, SEED2) | 1;

        for i in 0..self.k {
            let offset = u64::from(i).wrapping_mul(h2);
            let pos = h1.wrapping_add(offset) % self.nbits;
            if let Ok(byte_idx) = usize::try_from(pos / 8) {
                if let Ok(bit_idx) = u32::try_from(pos % 8) {
                    let bit_mask = 1u8 << bit_idx;
                    match self.bits.get(byte_idx) {
                        Some(&b) if (b & bit_mask) != 0 => {}
                        _ => return false,
                    }
                } else {
                    return false;
                }
            } else {
                return false;
            }
        }

        true
    }

    /// Liefert k (Anzahl Hashfunktionen).
    pub fn k(&self) -> u8 {
        self.k
    }

    /// Liefert nbits (Anzahl Bits im Filter).
    pub fn nbits(&self) -> u64 {
        self.nbits
    }

    /// Liefert die Rohbytes des Bit-Arrays.
    pub fn as_bytes(&self) -> &[u8] {
        &self.bits
    }
}

#[cfg(test)]
#[allow(clippy::indexing_slicing, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn test_golden_a6_bloom() {
        let mut filter = BloomFilter::with_explicit_bits(7, 64, vec![0u8; 8]);
        filter.insert(b"k");

        // Hashprüfungen laut Golden A.6
        let h1 = xxh64(b"k", 0);
        assert_eq!(h1, 0xC3D31922C50B1B63);
        let h2 = xxh64(b"k", SEED2) | 1;
        assert_eq!(h2, 0x6CBC77349206F859 | 1);

        // Positionen berechnen und prüfen: 35, 60, 21, 46, 7, 32, 57
        let expected_positions = [35, 60, 21, 46, 7, 32, 57];
        for (i, &expected) in expected_positions.iter().enumerate() {
            let offset = u64::try_from(i).unwrap().wrapping_mul(h2);
            let pos = h1.wrapping_add(offset) % 64;
            assert_eq!(pos, expected);
        }

        assert!(filter.may_contain(b"k"));
        assert!(!filter.may_contain(b"nonexistent_key"));

        // Roundtrip Encode/Decode
        let enc = filter.encode();
        let dec = BloomFilter::decode(&enc).expect("decode valid");
        assert_eq!(filter, dec);
    }
}
