//! # SSTable-Reader (Ring 1)
//!
//! Ermoeglicht wahlfreien und sequentiellen Lesezugriff auf unveraenderliche SSTables
//! mit Block-Caching und Bloom-Filter-Abfragen.

use super::block_cache::{BlockCache, SSTABLE_MAGIC_LEGACY, SSTABLE_MAGIC_MFSX};
use super::block_search::{
    binary_search_entry_in_block, binary_search_first_index_in_block, get_entry_at_index,
};
use super::bloom::BloomFilter;
use super::builder::SstableMetadata;
use super::io::pread_exact;
use super::stream::SstableStream;
use crate::error::{StoreError as ContextraError, StoreResult as Result};
use crate::io_seam::RealFs;
use crate::io_seam::{Vfs, VfsFile, VfsOpenOptions};
use crate::wal::KeyManager;
use bytes::Bytes;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Helper function to parse block trailer across format versions v0..v4.
/// Returns `(num_offsets, offsets_start, bloom_bytes)`.
pub fn parse_block_trailer(
    block_data: &[u8],
    format_version: u16,
) -> Result<(usize, usize, &[u8])> {
    let n = block_data.len();
    if format_version >= 4 {
        if n < 8 {
            return Err(ContextraError::Storage("block too small".into()));
        }
        let num_offsets = usize::try_from(u32::from_le_bytes(
            block_data
                .get(n.saturating_sub(4)..n)
                .ok_or_else(|| {
                    ContextraError::Storage("malformed block: missing num_offsets".into())
                })?
                .try_into()
                .map_err(|_| ContextraError::Storage("invalid slice".into()))?,
        ))
        .map_err(|_| ContextraError::Storage("num_offsets exceeds platform usize".into()))?;

        let bloom_len = usize::try_from(u32::from_le_bytes(
            block_data
                .get(n.saturating_sub(8)..n.saturating_sub(4))
                .ok_or_else(|| {
                    ContextraError::Storage("malformed block: missing bloom_len".into())
                })?
                .try_into()
                .map_err(|_| ContextraError::Storage("invalid slice".into()))?,
        ))
        .map_err(|_| ContextraError::Storage("bloom_len exceeds platform usize".into()))?;

        let offsets_len = num_offsets
            .checked_mul(4)
            .ok_or_else(|| ContextraError::Storage("overflow calculating offsets length".into()))?;

        let total_trailer_len = 8usize
            .checked_add(offsets_len)
            .and_then(|l| l.checked_add(bloom_len))
            .ok_or_else(|| ContextraError::Storage("overflow calculating trailer length".into()))?;

        if n < total_trailer_len {
            return Err(ContextraError::Storage(
                "malformed block: trailer length out of bounds".into(),
            ));
        }

        let offsets_start = n.saturating_sub(8).saturating_sub(offsets_len);
        let bloom_start = offsets_start.saturating_sub(bloom_len);
        let bloom_bytes = block_data.get(bloom_start..offsets_start).ok_or_else(|| {
            ContextraError::Storage("malformed block: missing bloom bytes".into())
        })?;

        Ok((num_offsets, offsets_start, bloom_bytes))
    } else if format_version == 3 {
        if n < 36 {
            return Err(ContextraError::Storage("block too small".into()));
        }
        let num_offsets = usize::try_from(u32::from_le_bytes(
            block_data
                .get(n.saturating_sub(4)..n)
                .ok_or_else(|| {
                    ContextraError::Storage("malformed block: missing num_offsets".into())
                })?
                .try_into()
                .map_err(|_| ContextraError::Storage("invalid slice".into()))?,
        ))
        .map_err(|_| ContextraError::Storage("num_offsets exceeds platform usize".into()))?;

        let offsets_len = num_offsets
            .checked_mul(4)
            .ok_or_else(|| ContextraError::Storage("overflow calculating offsets length".into()))?;

        if n < offsets_len.saturating_add(36) {
            return Err(ContextraError::Storage(
                "malformed block: num_offsets too large".into(),
            ));
        }
        let offsets_start = n.saturating_sub(4).saturating_sub(offsets_len);
        let bloom_start = offsets_start.saturating_sub(32);
        let bloom_bytes = block_data.get(bloom_start..offsets_start).ok_or_else(|| {
            ContextraError::Storage("malformed block: missing bloom bytes".into())
        })?;

        Ok((num_offsets, offsets_start, bloom_bytes))
    } else {
        if n < 10 {
            return Err(ContextraError::Storage("block too small".into()));
        }
        let num_offsets = usize::from(u16::from_le_bytes(
            block_data
                .get(n.saturating_sub(2)..n)
                .ok_or_else(|| {
                    ContextraError::Storage("malformed block: missing num_offsets".into())
                })?
                .try_into()
                .map_err(|_| ContextraError::Storage("invalid slice".into()))?,
        ));

        let offsets_len = num_offsets
            .checked_mul(2)
            .ok_or_else(|| ContextraError::Storage("overflow calculating offsets length".into()))?;

        if n < offsets_len.saturating_add(10) {
            return Err(ContextraError::Storage(
                "malformed block: num_offsets too large".into(),
            ));
        }
        let offsets_start = n.saturating_sub(2).saturating_sub(offsets_len);
        let bloom_start = offsets_start.saturating_sub(8);
        let bloom_bytes = block_data.get(bloom_start..offsets_start).ok_or_else(|| {
            ContextraError::Storage("malformed block: missing bloom bytes".into())
        })?;

        Ok((num_offsets, offsets_start, bloom_bytes))
    }
}

/// Evaluates per-block Bloom filter across format versions v0..v4.
pub(super) fn may_contain_key_in_block_bloom(
    bloom_bytes: &[u8],
    key: &[u8],
    format_version: u16,
) -> bool {
    if format_version >= 4 {
        let m = bloom_bytes.len().saturating_mul(8);
        if m == 0 {
            return true;
        }
        let (h1, h2) = BloomFilter::hash_pair(key);
        for i in 0..7u64 {
            let bit_idx = (h1.wrapping_add(i.wrapping_mul(h2)) as usize) % m;
            let byte_idx = bit_idx / 8;
            let bit_mask = 1u8 << (bit_idx % 8);
            if let Some(&byte) = bloom_bytes.get(byte_idx) {
                if (byte & bit_mask) == 0 {
                    return false;
                }
            } else {
                return false;
            }
        }
        true
    } else if format_version == 3 {
        if bloom_bytes.len() < 32 {
            return false;
        }
        let mut bloom_words = [0u64; 4];
        for (w, word) in bloom_words.iter_mut().enumerate().take(4) {
            if let Some(chunk) = bloom_bytes.get(w * 8..(w + 1) * 8) {
                if let Ok(arr) = chunk.try_into() {
                    *word = u64::from_le_bytes(arr);
                } else {
                    return false;
                }
            } else {
                return false;
            }
        }

        let hash = blake3::hash(key);
        let hash_bytes = hash.as_bytes();
        for i in 0..8 {
            let chunk = u16::from_le_bytes([
                *hash_bytes.get(i * 2).unwrap_or(&0),
                *hash_bytes.get(i * 2 + 1).unwrap_or(&0),
            ]);
            let bit_idx = chunk as usize % 256;
            if (bloom_words[bit_idx / 64] & (1u64 << (bit_idx % 64))) == 0 {
                return false;
            }
        }
        true
    } else {
        if bloom_bytes.len() < 8 {
            return false;
        }
        let bloom = if let Ok(arr) = bloom_bytes[0..8].try_into() {
            u64::from_le_bytes(arr)
        } else {
            return false;
        };

        let hash = blake3::hash(key);
        let hash_bytes = hash.as_bytes();
        for i in 0..4 {
            let chunk = u16::from_le_bytes([
                *hash_bytes.get(i * 2).unwrap_or(&0),
                *hash_bytes.get(i * 2 + 1).unwrap_or(&0),
            ]);
            let bit = chunk % 64;
            if (bloom & (1 << bit)) == 0 {
                return false;
            }
        }
        true
    }
}

/// A reader for existing SSTables.
// ANCHOR[contextra-store:SST-RDR-001]
/// Liest SSTable-Dateien mit Block-Cache und Bloom-Filter-Unterstuetzung.
pub struct SstableReader {
    pub(super) file: Arc<dyn VfsFile>,
    pub(super) index: Vec<(Bytes, u64)>,
    pub(super) metadata: SstableMetadata,
    /// Byte offset where the index data begins (= end of last block).
    pub(super) index_offset: u64,
    /// File path of this SSTable (for compaction cleanup).
    pub(super) file_path: PathBuf,
    /// Unique ID for this SSTable (for cache keys).
    pub(super) file_id: u64,
    /// Shared block cache.
    pub(super) block_cache: Arc<BlockCache>,
    pub(super) key_manager: Option<Arc<KeyManager>>,
    /// Optional whole-SSTable bloom filter for cross-block pre-checks.
    pub(super) bloom_filter: Option<BloomFilter>,
    /// Whether blocks have CRC32 checksums.
    pub(super) has_crc: bool,
    /// Format version of the SSTable file (0, 1, 2, 3, or 4).
    pub format_version: u16,
}

impl SstableReader {
    pub fn first_key(&self) -> &Bytes {
        &self.metadata.first_key
    }

    pub fn last_key(&self) -> &Bytes {
        &self.metadata.last_key
    }

    pub fn min_tx_id(&self) -> u64 {
        self.metadata.min_tx_id
    }

    pub fn max_tx_id(&self) -> u64 {
        self.metadata.max_tx_id
    }

    pub fn file_path(&self) -> &Path {
        &self.file_path
    }
}

mod block;
mod iter;
mod lookup;
mod open;
