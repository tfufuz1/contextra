//! # Block-Builder (Ring 1)
//!
//! Zweck: Baut einzelne SSTable-Datenblöcke auf mit Restriktion auf `block_size`
//! und kodiert Schlüssel-Wert-Einträge samt Offsets und Trailer.
// ANCHOR[contextra-store:SST-BLD-100]

use crate::sstable::bloom::BloomFilter;
use bytes::{BufMut, Bytes, BytesMut};

/// A builder for SSTable data blocks (Format v4 default, supports v3).
pub struct BlockBuilder {
    data: BytesMut,
    offsets: Vec<u32>,
    pub(crate) block_size: usize,
    key_hashes: Vec<(u64, u64)>,
    v3_bloom: [u64; 4],
    format_version: u16,
}

impl BlockBuilder {
    pub fn new(block_size: usize) -> Self {
        Self::new_with_version(block_size, 4)
    }

    pub fn new_with_version(block_size: usize, format_version: u16) -> Self {
        Self {
            data: BytesMut::new(),
            offsets: Vec::new(),
            block_size: block_size.clamp(512, 64 * 1024),
            key_hashes: Vec::new(),
            v3_bloom: [0u64; 4],
            format_version,
        }
    }

    fn update_bloom(&mut self, key: &[u8]) {
        if self.format_version >= 4 {
            self.key_hashes.push(BloomFilter::hash_pair(key));
        } else {
            let hash = blake3::hash(key);
            let bytes = hash.as_bytes();
            for i in 0..8 {
                let chunk = u16::from_le_bytes([
                    *bytes.get(i * 2).unwrap_or(&0),
                    *bytes.get(i * 2 + 1).unwrap_or(&0),
                ]);
                let bit_idx = chunk as usize % 256;
                self.v3_bloom[bit_idx / 64] |= 1u64 << (bit_idx % 64);
            }
        }
    }

    pub fn add(&mut self, key: &[u8], value: &[u8], seq_no: u64, tx_id: u64) -> bool {
        let entry_header_and_meta_len = 22usize; // 2 + 8 + 8 + 4
        let projected_n = self.offsets.len().saturating_add(1);

        let projected_total_len = if self.format_version >= 4 {
            let projected_bits = projected_n.saturating_mul(10).next_multiple_of(64);
            let projected_bloom_bytes = projected_bits / 8;
            self.data
                .len()
                .saturating_add(key.len())
                .saturating_add(value.len())
                .saturating_add(entry_header_and_meta_len)
                .saturating_add(projected_bloom_bytes)
                .saturating_add(projected_n.saturating_mul(4))
                .saturating_add(8)
        } else {
            self.data
                .len()
                .saturating_add(key.len())
                .saturating_add(value.len())
                .saturating_add(entry_header_and_meta_len)
                .saturating_add(32) // v3 fixed bloom
                .saturating_add(projected_n.saturating_mul(4))
                .saturating_add(4)
        };

        if !self.data.is_empty() && projected_total_len > self.block_size {
            return false;
        }

        let key_len_u16 = match u16::try_from(key.len()) {
            Ok(l) => l,
            Err(_) => return false,
        };
        let val_len_u32 = match u32::try_from(value.len()) {
            Ok(l) => l,
            Err(_) => return false,
        };
        let offset_u32 = match u32::try_from(self.data.len()) {
            Ok(o) => o,
            Err(_) => return false,
        };

        self.update_bloom(key);
        self.offsets.push(offset_u32);
        self.data.put_u16_le(key_len_u16);
        self.data.put_slice(key);
        self.data.put_u64_le(seq_no);
        self.data.put_u64_le(tx_id);
        self.data.put_u32_le(val_len_u32);
        self.data.put_slice(value);
        true
    }

    pub fn current_size(&self) -> usize {
        let n = self.offsets.len();
        if self.format_version >= 4 {
            let num_bits = if n == 0 {
                64
            } else {
                n.saturating_mul(10).next_multiple_of(64)
            };
            let bloom_bytes = num_bits / 8;

            self.data
                .len()
                .saturating_add(bloom_bytes)
                .saturating_add(n.saturating_mul(4))
                .saturating_add(8)
        } else {
            self.data
                .len()
                .saturating_add(32)
                .saturating_add(n.saturating_mul(4))
                .saturating_add(4)
        }
    }

    pub fn is_empty(&self) -> bool {
        self.offsets.is_empty()
    }

    /// Finalizes the block and returns the bytes.
    pub fn build(mut self) -> Bytes {
        let n = self.offsets.len();
        if self.format_version >= 4 {
            let num_bits = if n == 0 {
                64
            } else {
                n.saturating_mul(10).next_multiple_of(64)
            };
            let bloom_len = num_bits / 8;
            let mut bloom_vec = vec![0u8; bloom_len];

            if num_bits > 0 {
                for &(h1, h2) in &self.key_hashes {
                    for i in 0..7u64 {
                        let bit_idx = (h1.wrapping_add(i.wrapping_mul(h2)) as usize) % num_bits;
                        bloom_vec[bit_idx / 8] |= 1u8 << (bit_idx % 8);
                    }
                }
            }

            self.data.put_slice(&bloom_vec);

            for &offset in &self.offsets {
                self.data.put_u32_le(offset);
            }

            let bloom_len_u32 = u32::try_from(bloom_len).unwrap_or(u32::MAX);
            self.data.put_u32_le(bloom_len_u32);

            let num_offsets_u32 = u32::try_from(n).unwrap_or(u32::MAX);
            self.data.put_u32_le(num_offsets_u32);
        } else {
            for word in &self.v3_bloom {
                self.data.put_u64_le(*word);
            }
            for &offset in &self.offsets {
                self.data.put_u32_le(offset);
            }
            let num_offsets_u32 = u32::try_from(n).unwrap_or(u32::MAX);
            self.data.put_u32_le(num_offsets_u32);
        }

        self.data.freeze()
    }
}
