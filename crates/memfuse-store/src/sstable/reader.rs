use super::block_cache::{BlockCache, SSTABLE_MAGIC_LEGACY, SSTABLE_MAGIC_MFSX};
use super::block_search::{binary_search_entry_in_block, block_binary_search};
use super::bloom::BloomFilter;
use super::builder::SstableMetadata;
use super::io::pread_exact;
use super::stream::SstableStream;
use bytes::Bytes;
use memfuse_core::{MemFuseError, Result};
use memfuse_crypto::crypto::KeyManager;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// A reader for existing SSTables.
pub struct SstableReader {
    pub(super) file: Arc<std::fs::File>,
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

    /// Opens an existing SSTable file for reading.
    pub async fn open(path: impl AsRef<Path>, block_cache: Arc<BlockCache>) -> Result<Self> {
        Self::open_with_key_manager(path, block_cache, None).await
    }

    pub async fn open_with_key_manager(
        path: impl AsRef<Path>,
        block_cache: Arc<BlockCache>,
        key_manager: Option<Arc<KeyManager>>,
    ) -> Result<Self> {
        let path_buf = path.as_ref().to_path_buf();
        let derived_km = if let Some(ref km) = key_manager {
            let file_id = path_buf
                .file_name()
                .map(|n| n.to_string_lossy())
                .unwrap_or_default();
            Some(Arc::new(km.derive_file_key(file_id.as_bytes())?))
        } else {
            None
        };

        let path_for_open = path_buf.clone();
        let (file, file_size) =
            tokio::task::spawn_blocking(move || -> std::io::Result<(std::fs::File, u64)> {
                let file = std::fs::File::open(&path_for_open)?;
                let metadata = file.metadata()?;
                let file_size = metadata.len();
                Ok((file, file_size))
            })
            .await
            .map_err(|e| MemFuseError::Storage(format!("Join error: {}", e)))?
            .map_err(|e| MemFuseError::Storage(format!("File open failed: {}", e)))?;

        if file_size < 12 {
            return Err(MemFuseError::Storage("SSTable file too small".into()));
        }

        let file = Arc::new(file);

        // Read trailer: last 54 bytes (v1) or 52 bytes (v0)
        let trailer_data = {
            let f = Arc::clone(&file);
            tokio::task::spawn_blocking(move || -> std::io::Result<Vec<u8>> {
                // We read up to 54 bytes to check for v1 trailer
                let mut buf = vec![0u8; 54.min(file_size as usize)];
                let offset = file_size.saturating_sub(54);
                pread_exact(&f, &mut buf, offset)?;
                Ok(buf)
            })
            .await
            .map_err(|e| MemFuseError::Storage(format!("Join error: {}", e)))?
            .map_err(|e| MemFuseError::Storage(format!("Trailer read failed: {}", e)))?
        };

        let trailer_len = trailer_data.len();
        if trailer_len < 12 {
            return Err(MemFuseError::Storage("Invalid trailer".into()));
        }

        // Detect version and magic (FIND-STO-003)
        // v1: [..., version:u16][magic:u32] at the end (54 bytes)
        // v0: [..., magic:u32] at the end (52 bytes)
        let mut format_version = 0u16;
        let mut is_mfsx = false;

        if trailer_len >= 54 {
            if let Some(slice) = trailer_data.get(50..54) {
                let magic_v1 = u32::from_le_bytes(
                    slice
                        .try_into()
                        .map_err(|_| MemFuseError::Storage("Invalid trailer".into()))?,
                );
                if magic_v1 == SSTABLE_MAGIC_MFSX {
                    if let Some(ver_slice) = trailer_data.get(48..50) {
                        format_version = u16::from_le_bytes(
                            ver_slice
                                .try_into()
                                .map_err(|_| MemFuseError::Storage("Invalid trailer".into()))?,
                        );
                        is_mfsx = true;
                    }
                }
            }
        }

        if !is_mfsx && trailer_len >= 52 {
            if let Some(slice) = trailer_data.get(48..52) {
                let magic_v0 = u32::from_le_bytes(
                    slice
                        .try_into()
                        .map_err(|_| MemFuseError::Storage("Invalid trailer".into()))?,
                );
                if magic_v0 == SSTABLE_MAGIC_MFSX {
                    is_mfsx = true;
                    format_version = 0;
                }
            }
        }

        let mut has_bloom = false;
        let has_crc = is_mfsx;
        let mut bloom_offset = 0;
        let mut min_tx_id = 0;
        let mut max_tx_id = 0;
        let mut min_seq = 0;
        let mut max_seq = 0;

        let index_offset = if is_mfsx {
            // MFSX trailer (v0 or v1)
            let base = if format_version >= 1 { 0 } else { 2 }; // Offset into our 54-byte buffer
            min_tx_id = u64::from_le_bytes(
                trailer_data[base..base + 8]
                    .try_into()
                    .map_err(|_| MemFuseError::ParseError("Invalid min_tx_id".into()))?,
            );
            max_tx_id = u64::from_le_bytes(
                trailer_data[base + 8..base + 16]
                    .try_into()
                    .map_err(|_| MemFuseError::ParseError("Invalid max_tx_id".into()))?,
            );
            min_seq = u64::from_le_bytes(
                trailer_data[base + 16..base + 24]
                    .try_into()
                    .map_err(|_| MemFuseError::ParseError("Invalid min_seq".into()))?,
            );
            max_seq = u64::from_le_bytes(
                trailer_data[base + 24..base + 32]
                    .try_into()
                    .map_err(|_| MemFuseError::ParseError("Invalid max_seq".into()))?,
            );
            bloom_offset = u64::from_le_bytes(
                trailer_data[base + 32..base + 40]
                    .try_into()
                    .map_err(|_| MemFuseError::ParseError("Invalid bloom_offset".into()))?,
            );
            if bloom_offset > 0 {
                has_bloom = true;
            }
            u64::from_le_bytes(
                trailer_data[base + 40..base + 48]
                    .try_into()
                    .map_err(|_| MemFuseError::ParseError("Invalid index_offset".into()))?,
            )
        } else {
            // Read magic from the very end of 54-byte buffer (which would be the same as end of 52-byte if we read 54)
            let magic_legacy = u32::from_le_bytes(
                trailer_data[50..54]
                    .try_into()
                    .map_err(|_| MemFuseError::checksum_mismatch(path_buf.to_string_lossy(), 0))?,
            );
            if magic_legacy == SSTABLE_MAGIC_LEGACY {
                // Backward-compatible 12-byte trailer: [index_offset: u64][magic: u32]
                u64::from_le_bytes(
                    trailer_data[42..50]
                        .try_into()
                        .map_err(|_| MemFuseError::ParseError("Invalid index offset".into()))?,
                )
            } else {
                return Err(MemFuseError::Storage("Invalid SSTable magic number".into()));
            }
        };

        let bloom_filter = if has_bloom {
            let bloom_end = if is_mfsx {
                file_size.saturating_sub(if format_version >= 1 { 54 } else { 52 })
            } else {
                file_size.saturating_sub(20)
            };

            let bloom_data_raw = {
                let f = Arc::clone(&file);
                tokio::task::spawn_blocking(move || -> std::io::Result<Vec<u8>> {
                    let mut buf =
                        vec![0u8; (bloom_end as usize).saturating_sub(bloom_offset as usize)];
                    pread_exact(&f, &mut buf, bloom_offset)?;
                    Ok(buf)
                })
                .await
                .map_err(|e| MemFuseError::Storage(format!("Join error: {}", e)))?
                .map_err(|e| MemFuseError::Storage(format!("Bloom read failed: {}", e)))?
            };

            let bloom_data = if has_crc {
                if bloom_data_raw.len() < 4 {
                    return Err(MemFuseError::Storage(
                        "Bloom filter data too short for CRC".into(),
                    ));
                }
                let stored_crc = u32::from_le_bytes(
                    bloom_data_raw[0..4]
                        .try_into()
                        .map_err(|_| MemFuseError::Serialization("Invalid CRC".into()))?,
                );
                let payload = &bloom_data_raw[4..];
                if crc32fast::hash(payload) != stored_crc {
                    return Err(MemFuseError::checksum_mismatch(
                        path_buf.to_string_lossy(),
                        bloom_offset,
                    ));
                }
                payload
            } else {
                &bloom_data_raw
            };

            Some(BloomFilter::from_bytes(bloom_data)?)
        } else {
            None
        };

        // Read index
        let index_data_raw = {
            let f = Arc::clone(&file);
            let index_end = if has_bloom {
                bloom_offset as usize
            } else {
                file_size.saturating_sub(if is_mfsx {
                    if format_version >= 1 {
                        54
                    } else {
                        52
                    }
                } else {
                    12
                }) as usize
            };
            tokio::task::spawn_blocking(move || -> std::io::Result<Vec<u8>> {
                let mut buf = vec![0u8; index_end.saturating_sub(index_offset as usize)];
                pread_exact(&f, &mut buf, index_offset)?;
                Ok(buf)
            })
            .await
            .map_err(|e| MemFuseError::Storage(format!("Join error: {}", e)))?
            .map_err(|e| MemFuseError::Storage(format!("Index read failed: {}", e)))?
        };

        let index_bytes_raw = Bytes::from(index_data_raw);
        let index_bytes = if has_crc {
            if index_bytes_raw.len() < 4 {
                return Err(MemFuseError::Storage("Index data too short for CRC".into()));
            }
            let stored_crc = u32::from_le_bytes(
                index_bytes_raw[0..4]
                    .try_into()
                    .map_err(|_| MemFuseError::Serialization("Invalid CRC".into()))?,
            );
            let payload = index_bytes_raw.slice(4..);
            if crc32fast::hash(&payload) != stored_crc {
                return Err(MemFuseError::checksum_mismatch(
                    path_buf.to_string_lossy(),
                    index_offset,
                ));
            }
            payload
        } else {
            index_bytes_raw
        };

        let mut index = Vec::new();
        let mut pos = 0;
        let index_len = index_bytes.len();

        while pos + 10 <= index_len {
            let key_len = u16::from_le_bytes(
                index_bytes[pos..pos + 2]
                    .try_into()
                    .map_err(|_| MemFuseError::ParseError("corrupted index: key_len".into()))?,
            ) as usize;
            pos += 2;

            if pos + key_len + 8 > index_len {
                return Err(MemFuseError::ParseError(
                    "corrupted index: data too short".into(),
                ));
            }

            let key = index_bytes.slice(pos..pos + key_len);
            pos += key_len;

            let offset = u64::from_le_bytes(
                index_bytes[pos..pos + 8]
                    .try_into()
                    .map_err(|_| MemFuseError::ParseError("corrupted index: offset".into()))?,
            );
            pos += 8;
            index.push((key, offset));
        }

        let first_key = if !index.is_empty() {
            let offset = index[0].1;
            let next_offset = if index.len() > 1 {
                index[1].1
            } else {
                index_offset
            };
            let block = Self::read_block_at_file(
                Arc::clone(&file),
                offset,
                next_offset,
                &derived_km,
                has_crc,
                &path_buf,
            )
            .await?;
            if block.len() >= 2 {
                let k_len = u16::from_le_bytes(
                    block[0..2]
                        .try_into()
                        .map_err(|_| MemFuseError::ParseError("corrupted block: k_len".into()))?,
                ) as usize;
                if block.len() >= 2 + k_len {
                    block.slice(2..2 + k_len)
                } else {
                    Bytes::new()
                }
            } else {
                Bytes::new()
            }
        } else {
            Bytes::new()
        };

        Ok(Self {
            file,
            metadata: SstableMetadata {
                first_key,
                last_key: index.last().map(|(k, _)| k.clone()).unwrap_or_default(),
                file_size,
                min_tx_id,
                max_tx_id,
                min_seq,
                max_seq,
            },
            index,
            index_offset,
            file_path: path_buf,
            file_id: {
                static NEXT_FILE_ID: std::sync::atomic::AtomicU64 =
                    std::sync::atomic::AtomicU64::new(1);
                NEXT_FILE_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
            },
            block_cache,
            key_manager: derived_km,
            bloom_filter,
            has_crc,
        })
    }

    async fn read_block_at_file(
        file: Arc<std::fs::File>,
        offset: u64,
        next_offset: u64,
        key_manager: &Option<Arc<KeyManager>>,
        has_crc: bool,
        path: &Path,
    ) -> Result<Bytes> {
        let len = next_offset.saturating_sub(offset) as usize;
        let data = tokio::task::spawn_blocking(move || -> std::io::Result<Vec<u8>> {
            let mut buf = vec![0u8; len];
            pread_exact(&file, &mut buf, offset)?;
            Ok(buf)
        })
        .await
        .map_err(|e| MemFuseError::Storage(format!("Join error: {}", e)))?
        .map_err(|e| MemFuseError::Storage(format!("Block read failed: {}", e)))?;

        let block_data = if let Some(km) = key_manager {
            if data.len() < 12 {
                return Err(MemFuseError::Storage("Block too small for nonce".into()));
            }
            let mut nonce = [0u8; 12];
            nonce.copy_from_slice(&data[0..12]);
            let decrypted = km.decrypt_auto_nonce(&data[12..], &nonce)?;
            Bytes::from(decrypted)
        } else {
            Bytes::from(data)
        };

        if has_crc {
            if block_data.len() < 4 {
                return Err(MemFuseError::Storage("Block too small for CRC".into()));
            }
            let stored_crc = u32::from_le_bytes(
                block_data[0..4]
                    .try_into()
                    .map_err(|_| MemFuseError::Serialization("Invalid CRC format".into()))?,
            );
            let payload = &block_data[4..];
            let computed_crc = crc32fast::hash(payload);

            if stored_crc != computed_crc {
                return Err(MemFuseError::checksum_mismatch(
                    path.to_string_lossy(),
                    offset,
                ));
            }
            Ok(block_data.slice(4..))
        } else {
            Ok(block_data)
        }
    }

    pub(super) async fn get_block(&self, offset: u64, next_offset: u64) -> Result<Bytes> {
        if let Some(cached) = self.block_cache.get(self.file_id, offset) {
            return Ok(cached);
        }
        let block = Self::read_block_at_file(
            Arc::clone(&self.file),
            offset,
            next_offset,
            &self.key_manager,
            self.has_crc,
            &self.file_path,
        )
        .await?;
        self.block_cache.insert(self.file_id, offset, block.clone());
        Ok(block)
    }

    /// Retrieves a value from the SSTable by key.
    pub async fn get(&self, key: &[u8]) -> Result<Option<(Bytes, u64, u64)>> {
        // 1. Whole-SSTable Bloom Filter Pre-check
        // SPECCED: Only if bloom filter is present (backward compatibility)
        if let Some(bloom) = &self.bloom_filter {
            if !bloom.may_contain(key) {
                return Ok(None);
            }
        }

        if key < self.metadata.first_key || key > self.metadata.last_key {
            return Ok(None);
        }

        let idx = match self.index.binary_search_by(|(k, _)| k.as_ref().cmp(key)) {
            Ok(i) => i,
            Err(i) => i,
        };

        if idx >= self.index.len() {
            return Ok(None);
        }

        let offset = self
            .index
            .get(idx)
            .ok_or_else(|| MemFuseError::Storage("index out of bounds".into()))?
            .1;
        let next_offset = if idx + 1 < self.index.len() {
            self.index
                .get(idx + 1)
                .ok_or_else(|| MemFuseError::Storage("index out of bounds".into()))?
                .1
        } else {
            self.index_offset
        };

        let block_data = self.get_block(offset, next_offset).await?;

        let n = block_data.len();
        if n < 10 {
            return Err(MemFuseError::Storage("block too small".into()));
        }

        let num_offsets = u16::from_le_bytes(
            block_data
                .get(n.saturating_sub(2)..n)
                .ok_or_else(|| {
                    MemFuseError::Storage("malformed block: missing num_offsets".into())
                })?
                .try_into()
                .map_err(|_| MemFuseError::Storage("invalid slice".into()))?,
        ) as usize;

        let offsets_len = num_offsets.saturating_mul(2);
        if n < offsets_len.saturating_add(10) {
            return Err(MemFuseError::Storage(
                "malformed block: num_offsets too large".into(),
            ));
        }
        let offsets_start = n.saturating_sub(2).saturating_sub(offsets_len);
        let bloom_offset = offsets_start.saturating_sub(8);
        let bloom = u64::from_le_bytes(
            block_data
                .get(bloom_offset..bloom_offset.saturating_add(8))
                .ok_or_else(|| {
                    MemFuseError::Storage("malformed block: missing bloom filter".into())
                })?
                .try_into()
                .map_err(|_| MemFuseError::Storage("invalid slice".into()))?,
        );

        // Bloom check
        let hash = blake3::hash(key);
        let hash_bytes = hash.as_bytes();
        let mut may_contain = true;
        // Safety: blake3 outputs 32 bytes, i * 2 + 1 is max 7.
        for i in 0..4 {
            let chunk = u16::from_le_bytes([
                *hash_bytes.get(i * 2).unwrap_or(&0),
                *hash_bytes.get(i * 2 + 1).unwrap_or(&0),
            ]);
            let bit = chunk % 64;
            if (bloom & (1 << bit)) == 0 {
                may_contain = false;
                break;
            }
        }

        if !may_contain {
            return Ok(None);
        }

        if let Some(entry_off) = block_binary_search(&block_data, offsets_start, num_offsets, key)?
        {
            let k_len = u16::from_le_bytes(
                block_data
                    .get(entry_off..entry_off + 2)
                    .ok_or_else(|| MemFuseError::Storage("malformed block: k_len".into()))?
                    .try_into()
                    .map_err(|_| MemFuseError::Storage("invalid slice".into()))?,
            ) as usize;
            let mut ep = entry_off + 2 + k_len;
            let seq_no = u64::from_le_bytes(
                block_data
                    .get(ep..ep + 8)
                    .ok_or_else(|| MemFuseError::Storage("malformed block: seq_no".into()))?
                    .try_into()
                    .map_err(|_| MemFuseError::Storage("invalid slice".into()))?,
            );
            ep += 8;
            let tx_id = u64::from_le_bytes(
                block_data
                    .get(ep..ep + 8)
                    .ok_or_else(|| MemFuseError::Storage("malformed block: tx_id".into()))?
                    .try_into()
                    .map_err(|_| MemFuseError::Storage("invalid slice".into()))?,
            );
            ep += 8;
            let v_len = u32::from_le_bytes(
                block_data
                    .get(ep..ep + 4)
                    .ok_or_else(|| MemFuseError::Storage("malformed block: v_len".into()))?
                    .try_into()
                    .map_err(|_| MemFuseError::Storage("invalid slice".into()))?,
            ) as usize;
            ep += 4;
            if ep + v_len > block_data.len() {
                return Err(MemFuseError::Storage(
                    "malformed block: value length out of bounds".into(),
                ));
            }
            let entry_val = block_data.slice(ep..ep + v_len);
            Ok(Some((entry_val, seq_no, tx_id)))
        } else {
            Ok(None)
        }
    }

    /// Metrics result for point lookup instrumentation.
    pub async fn lookup_metrics(&self, key: &[u8]) -> (bool, bool, bool, bool) {
        // Returns (bloom_passed, range_passed, block_read, key_found)
        if let Some(bloom) = &self.bloom_filter {
            if !bloom.may_contain(key) {
                return (false, false, false, false);
            }
        }

        if key < self.metadata.first_key || key > self.metadata.last_key {
            return (true, false, false, false);
        }

        let idx = match self.index.binary_search_by(|(k, _)| k.as_ref().cmp(key)) {
            Ok(i) => i,
            Err(i) => i,
        };

        if idx >= self.index.len() {
            return (true, true, false, false);
        }

        let offset = match self.index.get(idx) {
            Some((_, off)) => *off,
            None => return (true, true, false, false),
        };
        let next_offset = if idx + 1 < self.index.len() {
            match self.index.get(idx + 1) {
                Some((_, off)) => *off,
                None => self.index_offset,
            }
        } else {
            self.index_offset
        };

        let block_data = match self.get_block(offset, next_offset).await {
            Ok(b) => b,
            Err(_) => return (true, true, false, false),
        };

        let n = block_data.len();
        if n < 10 {
            return (true, true, true, false);
        }

        let num_offsets = match block_data.get(n.saturating_sub(2)..n) {
            Some(slice) => match slice.try_into() {
                Ok(arr) => u16::from_le_bytes(arr) as usize,
                Err(_) => return (true, true, true, false),
            },
            None => return (true, true, true, false),
        };

        let offsets_len = num_offsets.saturating_mul(2);
        if n < offsets_len.saturating_add(10) {
            return (true, true, true, false);
        }
        let offsets_start = n.saturating_sub(2).saturating_sub(offsets_len);
        let bloom_offset = offsets_start.saturating_sub(8);
        let bloom = match block_data.get(bloom_offset..bloom_offset.saturating_add(8)) {
            Some(slice) => match slice.try_into() {
                Ok(arr) => u64::from_le_bytes(arr),
                Err(_) => return (true, true, true, false),
            },
            None => return (true, true, true, false),
        };

        // Block bloom check
        let hash = blake3::hash(key);
        let hash_bytes = hash.as_bytes();
        let mut may_contain = true;
        for i in 0..4 {
            let chunk = u16::from_le_bytes([
                *hash_bytes.get(i * 2).unwrap_or(&0),
                *hash_bytes.get(i * 2 + 1).unwrap_or(&0),
            ]);
            let bit = chunk % 64;
            if (bloom & (1 << bit)) == 0 {
                may_contain = false;
                break;
            }
        }

        if !may_contain {
            return (true, true, true, false);
        }

        match binary_search_entry_in_block(&block_data, offsets_start, num_offsets, key) {
            Ok(Some(_)) => (true, true, true, true),
            _ => (true, true, true, false),
        }
    }

    pub fn metadata(&self) -> &SstableMetadata {
        &self.metadata
    }

    #[allow(clippy::unused_async)]
    pub async fn stream(self: &Arc<Self>) -> Result<SstableStream> {
        Ok(SstableStream {
            reader: Arc::clone(self),
            block_idx: 0,
            entry_idx: 0,
            current_block: None,
            num_offsets: 0,
            offsets_start: 0,
        })
    }

    /// Iterates over all entries in sorted key order (allocates memory for all entries).
    pub async fn iter(&self) -> Result<Vec<(Bytes, Bytes, u64)>> {
        let mut results = Vec::new();
        if self.index.is_empty() {
            return Ok(results);
        }

        for idx in 0..self.index.len() {
            let offset = self
                .index
                .get(idx)
                .ok_or_else(|| MemFuseError::Storage("index out of bounds".into()))?
                .1;
            let next_offset = if idx + 1 < self.index.len() {
                self.index
                    .get(idx + 1)
                    .ok_or_else(|| MemFuseError::Storage("index out of bounds".into()))?
                    .1
            } else {
                self.index_offset
            };

            let block_data = self.get_block(offset, next_offset).await?;

            let n = block_data.len();
            if n < 10 {
                continue;
            }

            let num_offsets = u16::from_le_bytes(
                block_data
                    .get(n.saturating_sub(2)..n)
                    .ok_or_else(|| {
                        MemFuseError::Storage("malformed block: missing num_offsets".into())
                    })?
                    .try_into()
                    .map_err(|_| MemFuseError::Storage("invalid slice".into()))?,
            ) as usize;

            let offsets_len = num_offsets * 2;
            if n < 10 + offsets_len {
                return Err(MemFuseError::Storage(
                    "malformed block: num_offsets too large".into(),
                ));
            }
            let offsets_start = n - 2 - offsets_len;

            for i in 0..num_offsets {
                let off_pos = offsets_start + i * 2;
                let entry_off = u16::from_le_bytes(
                    block_data
                        .get(off_pos..off_pos + 2)
                        .ok_or_else(|| MemFuseError::Storage("malformed block: off_pos".into()))?
                        .try_into()
                        .map_err(|_| MemFuseError::Storage("invalid slice".into()))?,
                ) as usize;

                let mut ep = entry_off;
                let k_len = u16::from_le_bytes(
                    block_data
                        .get(ep..ep + 2)
                        .ok_or_else(|| MemFuseError::Storage("malformed block: k_len".into()))?
                        .try_into()
                        .map_err(|_| MemFuseError::Storage("invalid slice".into()))?,
                ) as usize;
                ep += 2;
                let _entry_key = block_data
                    .get(ep..ep + k_len)
                    .ok_or_else(|| MemFuseError::Storage("malformed block: entry_key".into()))?;
                ep += k_len;

                let seq_no = u64::from_le_bytes(
                    block_data
                        .get(ep..ep + 8)
                        .ok_or_else(|| MemFuseError::Storage("malformed block: seq_no".into()))?
                        .try_into()
                        .map_err(|_| MemFuseError::Storage("invalid slice".into()))?,
                );
                ep += 8;
                let _tx_id = u64::from_le_bytes(
                    block_data
                        .get(ep..ep + 8)
                        .ok_or_else(|| MemFuseError::Storage("malformed block: tx_id".into()))?
                        .try_into()
                        .map_err(|_| MemFuseError::Storage("invalid slice".into()))?,
                );
                ep += 8;
                let v_len = u32::from_le_bytes(
                    block_data
                        .get(ep..ep + 4)
                        .ok_or_else(|| MemFuseError::Storage("malformed block: v_len".into()))?
                        .try_into()
                        .map_err(|_| MemFuseError::Storage("invalid slice".into()))?,
                ) as usize;
                ep += 4;
                if ep + v_len > block_data.len() {
                    return Err(MemFuseError::Storage(
                        "malformed block: value length out of bounds".into(),
                    ));
                }
                let key_bytes = block_data.slice(entry_off + 2..entry_off + 2 + k_len);
                let val_bytes = block_data.slice(ep..ep + v_len);

                results.push((key_bytes, val_bytes, seq_no));
            }
        }

        Ok(results)
    }
}
