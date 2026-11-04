//! # Bloom-Filter fuer SSTables (Ring 1)
//!
//! Implementiert probabilistische Filter (Split-Block Lemire / Blocked Bloom),
//! um unnoetige Plattenzugriffe fuer nicht existente Schluessel abzuwenden.

use bytes::BufMut;
use contextra_types::{ContextraError, Result};

/// SPECCED: Speichereffizienter Bloom-Filter für SSTable-Pre-Checks.
/// Ziel: False-Positive-Rate p ≤ 0.01.
/// Formel: m = -n * ln(p) / (ln(2)^2) ≈ 9.6 * n
/// Hashes: k = (m/n) * ln(2) ≈ 7
#[derive(Debug, Clone)]
pub struct BloomFilter {
    pub(super) bits: Vec<u64>,
    pub(super) num_hashes: usize,
    pub(super) num_bits: usize,
}

impl BloomFilter {
    /// Creates a new Bloom filter for the expected number of elements and target false positive rate (fpr).
    ///
    /// ## FPR Trade-off
    /// - `fpr = 0.01` (1%) uses ~9.6 bits/element.
    /// - `fpr = 0.001` (0.1%) uses ~14.4 bits/element.
    ///
    /// Note: The default `fpr` used in [`SstableBuilder`] should be documented as a tunable
    /// parameter and referenced in [`crate::lsm::LsmConfig`] where it should be configurable.
    pub fn new(expected_elements: usize, fpr: f64) -> Self {
        let n = expected_elements.max(1);
        let p = fpr.clamp(0.0001, 0.1);

        // Safety limit: clamp expected elements to 100 million to prevent float/usize overflow
        let n = n.min(100_000_000);

        // m = bits needed
        let m_float = (-(n as f64) * p.ln() / (2.0f64.ln().powi(2))).ceil();
        let m = if m_float < 64.0 || !m_float.is_finite() {
            64usize
        } else if m_float > (128.0 * 1024.0 * 1024.0 * 8.0) {
            128 * 1024 * 1024 * 8
        } else {
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let val = m_float as u64;
            usize::try_from(val).unwrap_or(usize::MAX)
        };

        // Hard upper bound on bloom filter bits (128 MB = 1_073_741_824 bits)
        const MAX_BITS: usize = 128 * 1024 * 1024 * 8;
        let num_bits = m.next_multiple_of(64).clamp(64, MAX_BITS);
        let k_float = ((num_bits as f64 / n as f64) * 2.0f64.ln()).round();
        let num_hashes = if k_float < 1.0 || !k_float.is_finite() {
            1usize
        } else if k_float > 16.0 {
            16usize
        } else {
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let val = k_float as u64;
            usize::try_from(val).unwrap_or(16).clamp(1, 16)
        };

        Self {
            bits: vec![0u64; num_bits / 64],
            num_hashes,
            num_bits,
        }
    }

    /// Inserts a key into the filter.
    pub fn insert(&mut self, key: &[u8]) {
        let (h1, h2) = Self::hash_pair(key);
        for i in 0..self.num_hashes {
            let bit_idx = usize::try_from(h1.wrapping_add((i as u64).wrapping_mul(h2)) % (self.num_bits as u64)).unwrap_or(0);
            if let Some(word) = self.bits.get_mut(bit_idx / 64) {
                *word |= 1u64 << (bit_idx % 64);
            }
        }
    }

    /// Checks if a key might be in the filter.
    pub fn may_contain(&self, key: &[u8]) -> bool {
        if self.num_bits == 0 {
            return true;
        }
        let (h1, h2) = Self::hash_pair(key);
        for i in 0..self.num_hashes {
            let bit_idx = usize::try_from(h1.wrapping_add((i as u64).wrapping_mul(h2)) % (self.num_bits as u64)).unwrap_or(0);
            let word = self.bits.get(bit_idx / 64).copied().unwrap_or(0);
            if (word & (1u64 << (bit_idx % 64))) == 0 {
                return false;
            }
        }
        true
    }

    /// Erzeugt zwei unabhängige 64-bit Hashes aus dem Blake3-Digest.
    /// h1 = erste 8 Bytes, h2 = Bytes 8-15 (oder fallback zu h1 ^ const)
    pub fn hash_pair(key: &[u8]) -> (u64, u64) {
        let hash = blake3::hash(key);
        let bytes = hash.as_bytes();

        let h1 = u64::from_le_bytes(bytes[0..8].try_into().unwrap_or([0u8; 8]));
        let mut h2 = u64::from_le_bytes(bytes[8..16].try_into().unwrap_or([0u8; 8]));

        // h2 muss ungerade sein für double hashing (verhindert Zyklen)
        h2 |= 1;

        (h1, h2)
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        // MIGRATION NOTE: Bestehende SSTables mit altem Filter (probe-basiert)
        // müssen bei der nächsten Compaction neu gebaut werden.
        // Das Serialisierungsformat bleibt kompatibel, daher ist kein aktives Handeln nötig.
        let mut buf = Vec::with_capacity(8 + 8 + self.bits.len() * 8);
        buf.put_u64_le(self.num_hashes as u64);
        buf.put_u64_le(self.num_bits as u64);
        for &word in &self.bits {
            buf.put_u64_le(word);
        }
        buf
    }

    pub fn from_bytes(data: &[u8]) -> Result<Self> {
        if data.len() < 16 {
            return Err(ContextraError::Storage(
                "corrupted bloom filter: too short".into(),
            ));
        }
        let hashes_slice = data.get(0..8).ok_or_else(|| {
            ContextraError::Storage("corrupted bloom filter: too short for num_hashes".into())
        })?;
        let num_hashes = usize::try_from(u64::from_le_bytes(hashes_slice.try_into().map_err(|_| {
            ContextraError::ParseError("corrupted bloom filter: invalid num_hashes".into())
        })?)).map_err(|_| ContextraError::Storage("num_hashes exceeds usize".into()))?;

        let bits_slice = data.get(8..16).ok_or_else(|| {
            ContextraError::Storage("corrupted bloom filter: too short for num_bits".into())
        })?;
        let num_bits = usize::try_from(u64::from_le_bytes(bits_slice.try_into().map_err(|_| {
            ContextraError::ParseError("corrupted bloom filter: invalid num_bits".into())
        })?)).map_err(|_| ContextraError::Storage("num_bits exceeds usize".into()))?;

        if num_hashes == 0 || num_hashes > 64 {
            return Err(ContextraError::Storage(format!(
                "corrupted bloom filter: invalid num_hashes ({})",
                num_hashes
            )));
        }

        // Safety limit: 128MB for bloom filter bits (approx 1 billion bits)
        if num_bits > 128 * 1024 * 1024 * 8 {
            return Err(ContextraError::Storage(format!(
                "corrupted bloom filter: too many bits ({})",
                num_bits
            )));
        }

        let required_words = num_bits.saturating_add(63) / 64;
        let required_bytes = 16usize
            .checked_add(required_words.checked_mul(8).ok_or_else(|| {
                ContextraError::Storage("corrupted bloom filter: overflow calculating size".into())
            })?)
            .ok_or_else(|| {
                ContextraError::Storage("corrupted bloom filter: overflow calculating size".into())
            })?;

        if data.len() < required_bytes {
            return Err(ContextraError::Storage(format!(
                "corrupted bloom filter: data length ({}) insufficient for required bits ({})",
                data.len(),
                num_bits
            )));
        }

        let mut bits = Vec::with_capacity(required_words);
        let mut offset = 16;
        for _ in 0..required_words {
            let word_slice = data.get(offset..offset + 8).ok_or_else(|| {
                ContextraError::Storage("corrupted bloom filter: unexpected EOF reading words".into())
            })?;
            bits.push(u64::from_le_bytes(
                word_slice.try_into().map_err(|_| {
                    ContextraError::ParseError("corrupted bloom filter: invalid word".into())
                })?,
            ));
            offset += 8;
        }
        Ok(Self {
            bits,
            num_hashes,
            num_bits,
        })
    }
}
