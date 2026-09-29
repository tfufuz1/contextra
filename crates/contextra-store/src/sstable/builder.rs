use super::block_cache::{BLOCK_SIZE, SSTABLE_MAGIC_MFSX};
use super::bloom::BloomFilter;
use crate::wal::KeyManager;
use bytes::{BufMut, Bytes, BytesMut};
use contextra_core::{ContextraError, Result};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

/// A builder for SSTable data blocks (Format v3: 256-bit Bloom, 32-bit offsets).
pub struct BlockBuilder {
    data: BytesMut,
    offsets: Vec<u32>,
    pub(super) block_size: usize,
    bloom: [u64; 4],
}

impl BlockBuilder {
    pub fn new(block_size: usize) -> Self {
        Self {
            data: BytesMut::new(),
            offsets: Vec::new(),
            block_size: block_size.clamp(512, 64 * 1024),
            bloom: [0u64; 4],
        }
    }

    fn update_bloom(&mut self, key: &[u8]) {
        let hash = blake3::hash(key);
        let bytes = hash.as_bytes();
        // Use 8 x 16-bit chunks modulo 256 from the 256-bit hash for Bloom filter bits (256-bit filter)
        // Safety: blake3 outputs 32 bytes, i * 2 + 1 is max 15.
        for i in 0..8 {
            let chunk = u16::from_le_bytes([
                *bytes.get(i * 2).unwrap_or(&0),
                *bytes.get(i * 2 + 1).unwrap_or(&0),
            ]);
            let bit_idx = chunk as usize % 256;
            self.bloom[bit_idx / 64] |= 1u64 << (bit_idx % 64);
        }
    }

    pub fn add(&mut self, key: &[u8], value: &[u8], seq_no: u64, tx_id: u64) -> bool {
        // Format v3 size calculation:
        // entry payload: key_len(2) + key + seq_no(8) + tx_id(8) + val_len(4) + value
        // block trailer: bloom(32) + offsets (offsets.len() * 4) + offset count (4 bytes)
        let entry_header_and_meta_len = 22usize; // 2 + 8 + 8 + 4
        if !self.data.is_empty()
            && self
                .current_size()
                .saturating_add(key.len())
                .saturating_add(value.len())
                .saturating_add(entry_header_and_meta_len)
                > self.block_size
        {
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
        // data + bloom(32) + offsets(u32 * len) + offset count (4 bytes)
        self.data
            .len()
            .saturating_add(32)
            .saturating_add(self.offsets.len().saturating_mul(4))
            .saturating_add(4)
    }

    pub fn is_empty(&self) -> bool {
        self.offsets.is_empty()
    }

    /// Finalizes the block and returns the bytes.
    pub fn build(mut self) -> Bytes {
        for word in &self.bloom {
            self.data.put_u64_le(*word);
        }
        for &offset in &self.offsets {
            self.data.put_u32_le(offset);
        }
        let num_offsets_u32 = u32::try_from(self.offsets.len()).unwrap_or(u32::MAX);
        self.data.put_u32_le(num_offsets_u32);
        self.data.freeze()
    }
}

/// Metadata for an SSTable.
#[derive(Debug, Clone)]
pub struct SstableMetadata {
    pub first_key: Bytes,
    pub last_key: Bytes,
    pub file_size: u64,
    pub min_tx_id: u64,
    pub max_tx_id: u64,
    pub min_seq: u64,
    pub max_seq: u64,
}

/// A builder for creating new SSTables.
///
/// Note: Uses a whole-SSTable Bloom filter with a default FPR. The Bloom filter FPR should be
/// treated as a tunable parameter and configured via [`crate::lsm::LsmConfig`].
pub struct SstableBuilder {
    path: PathBuf,
    file: File,
    block_builder: BlockBuilder,
    index: Vec<(Bytes, u64)>, // (last_key, offset)
    first_key: Option<Bytes>,
    last_key: Option<Bytes>,
    offset: u64,
    key_manager: Option<Arc<KeyManager>>,
    /// Whole-SSTable bloom filter for cross-block pre-checks.
    bloom_filter: BloomFilter,
    key_count: usize,
    min_tx_id: u64,
    max_tx_id: u64,
    min_seq: u64,
    max_seq: u64,
}

impl SstableBuilder {
    /// Creates a new SstableBuilder that writes to the given file path.
    pub async fn create(path: impl AsRef<Path>) -> Result<Self> {
        Self::create_with_key_manager(path, None).await
    }

    pub async fn create_with_key_manager(
        path: impl AsRef<Path>,
        key_manager: Option<Arc<KeyManager>>,
    ) -> Result<Self> {
        let path_ref = path.as_ref();
        let derived_km = if let Some(km) = key_manager {
            let file_id = path_ref
                .file_name()
                .map(|n| n.to_string_lossy())
                .unwrap_or_default();
            Some(Arc::new(km.derive_file_key(file_id.as_bytes())?))
        } else {
            None
        };
        let file = File::create(path_ref)
            .await
            .map_err(|e| ContextraError::Storage(format!("Failed to create SSTable: {}", e)))?;

        Ok(Self {
            path: path_ref.to_path_buf(),
            file,
            block_builder: BlockBuilder::new(BLOCK_SIZE),
            index: Vec::new(),
            first_key: None,
            last_key: None,
            offset: 0,
            key_manager: derived_km,
            bloom_filter: BloomFilter::new(100_000, 0.01),
            key_count: 0,
            min_tx_id: u64::MAX,
            max_tx_id: 0,
            min_seq: u64::MAX,
            max_seq: 0,
        })
    }

    /// Adds a key-value pair to the SSTable being built.
    pub async fn add(&mut self, key: &[u8], value: &[u8], seq_no: u64, tx_id: u64) -> Result<()> {
        if key.is_empty() {
            return Err(ContextraError::InvalidInput(
                "SSTable key cannot be empty".to_string(),
            ));
        }
        if key.len() > 65535 || value.len() > crate::lsm::MAX_VALUE_SIZE {
            return Err(ContextraError::InvalidInput(format!(
                "Key ({} bytes) exceeds 65535 limit or value ({} bytes) exceeds MAX_VALUE_SIZE limit",
                key.len(),
                value.len()
            )));
        }

        if self.first_key.is_none() {
            self.first_key = Some(Bytes::copy_from_slice(key));
        }

        if !self.block_builder.add(key, value, seq_no, tx_id) {
            self.flush_block().await?;
            if !self.block_builder.add(key, value, seq_no, tx_id) {
                return Err(ContextraError::Storage(format!(
                    "Key-value entry too large for SSTable block (key: {} bytes, val: {} bytes)",
                    key.len(),
                    value.len()
                )));
            }
        }

        self.bloom_filter.insert(key);
        self.key_count += 1;
        self.min_tx_id = self.min_tx_id.min(tx_id);
        self.max_tx_id = self.max_tx_id.max(tx_id);
        let raw_seq = seq_no & !contextra_core::TOMBSTONE_BIT;
        self.min_seq = self.min_seq.min(raw_seq);
        self.max_seq = self.max_seq.max(raw_seq);
        self.last_key = Some(Bytes::copy_from_slice(key));
        Ok(())
    }

    async fn flush_block(&mut self) -> Result<()> {
        if self.block_builder.is_empty() {
            return Ok(());
        }

        let last_key = self
            .last_key
            .clone()
            .ok_or_else(|| ContextraError::Storage("Missing last_key".into()))?;
        let block_data =
            std::mem::replace(&mut self.block_builder, BlockBuilder::new(BLOCK_SIZE)).build();

        // Compute CRC before encryption
        let crc = crc32fast::hash(&block_data);
        let mut block_with_crc = Vec::with_capacity(4 + block_data.len());
        block_with_crc.extend_from_slice(&crc.to_le_bytes());
        block_with_crc.extend_from_slice(&block_data);

        let mut block = Bytes::from(block_with_crc);

        if let Some(km) = &self.key_manager {
            let (encrypted, nonce) = km.encrypt_auto_nonce(&block)?;
            let mut new_block = BytesMut::with_capacity(12 + encrypted.len());
            new_block.put_slice(&nonce);
            new_block.put_slice(&encrypted);
            block = new_block.freeze();
        }

        let block_len = u64::try_from(block.len())
            .map_err(|_| ContextraError::Storage("Block size exceeds u64::MAX".into()))?;

        self.file
            .write_all(&block)
            .await
            .map_err(|e| ContextraError::Storage(format!("SSTable block write failed: {}", e)))?;

        self.index.push((last_key, self.offset));
        self.offset = self
            .offset
            .checked_add(block_len)
            .ok_or_else(|| ContextraError::Storage("SSTable offset overflow".into()))?;
        Ok(())
    }

    /// Finalizes the SSTable and returns metadata.
    pub async fn finish(mut self) -> Result<SstableMetadata> {
        self.flush_block().await?;

        let index_offset = self.offset;
        let mut index_builder = BytesMut::new();

        for (key, offset) in &self.index {
            let key_len_u16 = u16::try_from(key.len()).map_err(|_| {
                ContextraError::InvalidInput("Index key length exceeds u16::MAX".into())
            })?;
            index_builder.put_u16_le(key_len_u16);
            index_builder.put_slice(key);
            index_builder.put_u64_le(*offset);
        }

        let index_bytes = index_builder.freeze();

        // Add CRC to index
        let index_crc = crc32fast::hash(&index_bytes);
        let mut index_with_crc = Vec::with_capacity(4 + index_bytes.len());
        index_with_crc.extend_from_slice(&index_crc.to_le_bytes());
        index_with_crc.extend_from_slice(&index_bytes);
        let index_to_write = index_with_crc;

        self.file
            .write_all(&index_to_write)
            .await
            .map_err(|e| ContextraError::Storage(format!("SSTable index write failed: {}", e)))?;

        let index_write_len = u64::try_from(index_to_write.len())
            .map_err(|_| ContextraError::Storage("Index bytes length overflow".into()))?;

        // SPECCED: Write the whole-SSTable Bloom filter
        let bloom_offset = index_offset
            .checked_add(index_write_len)
            .ok_or_else(|| ContextraError::Storage("Bloom offset overflow".into()))?;
        let bloom_data = self.bloom_filter.to_bytes();

        // Add CRC to bloom
        let bloom_crc = crc32fast::hash(&bloom_data);
        let mut bloom_with_crc = Vec::with_capacity(4 + bloom_data.len());
        bloom_with_crc.extend_from_slice(&bloom_crc.to_le_bytes());
        bloom_with_crc.extend_from_slice(&bloom_data);
        let bloom_to_write = bloom_with_crc;

        self.file
            .write_all(&bloom_to_write)
            .await
            .map_err(|e| ContextraError::Storage(format!("SSTable bloom write failed: {}", e)))?;

        // Write trailer: [min_tx][max_tx][min_seq][max_seq][bloom_offset][index_offset][version=3][magic]
        // This is 54 bytes.
        self.file
            .write_u64_le(self.min_tx_id)
            .await
            .map_err(|e| ContextraError::Storage(e.to_string()))?;
        self.file
            .write_u64_le(self.max_tx_id)
            .await
            .map_err(|e| ContextraError::Storage(e.to_string()))?;
        self.file
            .write_u64_le(self.min_seq)
            .await
            .map_err(|e| ContextraError::Storage(e.to_string()))?;
        self.file
            .write_u64_le(self.max_seq)
            .await
            .map_err(|e| ContextraError::Storage(e.to_string()))?;
        self.file
            .write_u64_le(bloom_offset)
            .await
            .map_err(|e| ContextraError::Storage(e.to_string()))?;
        self.file
            .write_u64_le(index_offset)
            .await
            .map_err(|e| ContextraError::Storage(e.to_string()))?;

        // Format version 3 (256-bit block bloom & 32-bit offsets)
        self.file
            .write_u16_le(3)
            .await
            .map_err(|e| ContextraError::Storage(e.to_string()))?;

        self.file
            .write_u32_le(SSTABLE_MAGIC_MFSX)
            .await
            .map_err(|e| ContextraError::Storage(e.to_string()))?;

        self.file
            .sync_all()
            .await
            .map_err(|e| ContextraError::Storage(e.to_string()))?;

        crate::util::fsync_parent_dir(&self.path).await?;

        let file_size = self
            .file
            .metadata()
            .await
            .map_err(|e| ContextraError::Storage(e.to_string()))?
            .len();

        Ok(SstableMetadata {
            first_key: self.first_key.clone().unwrap_or_default(),
            last_key: self.last_key.clone().unwrap_or_default(),
            file_size,
            min_tx_id: self.min_tx_id,
            max_tx_id: self.max_tx_id,
            min_seq: self.min_seq,
            max_seq: self.max_seq,
        })
    }
}
