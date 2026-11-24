//! SSTable Reader: Open- und Initialisierungspfade (VFS, Mmap, Index- & Bloom-Laden).
// ANCHOR[contextra-store:SST-RDR-100]

use super::SstableReader;
use crate::error::{StoreError as ContextraError, StoreResult as Result};
use crate::io_seam::{RealFs, Vfs, VfsFile, VfsOpenOptions};
use crate::sstable::block_cache::{BlockCache, SSTABLE_MAGIC_LEGACY, SSTABLE_MAGIC_MFSX};
use crate::sstable::bloom::BloomFilter;
use crate::sstable::builder::SstableMetadata;
use crate::sstable::io::pread_exact;
use crate::wal::KeyManager;
use bytes::Bytes;
use std::path::{Path, PathBuf};
use std::sync::Arc;

impl SstableReader {
    pub async fn open(path: impl AsRef<Path>, block_cache: Arc<BlockCache>) -> Result<Self> {
        Self::open_with_vfs(path, block_cache, None, Arc::new(RealFs)).await
    }

    pub async fn open_with_key_manager(
        path: impl AsRef<Path>,
        block_cache: Arc<BlockCache>,
        key_manager: Option<Arc<KeyManager>>,
    ) -> Result<Self> {
        Self::open_with_vfs(path, block_cache, key_manager, Arc::new(RealFs)).await
    }

    pub async fn open_with_vfs(
        path: impl AsRef<Path>,
        block_cache: Arc<BlockCache>,
        key_manager: Option<Arc<KeyManager>>,
        vfs: Arc<dyn Vfs>,
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
        let vfs_clone = Arc::clone(&vfs);
        let (file, file_size) =
            tokio::task::spawn_blocking(move || -> Result<(Arc<dyn VfsFile>, u64)> {
                let file = vfs_clone.open(&path_for_open, &VfsOpenOptions::read_only())?;
                let file_size = file.len().map_err(ContextraError::Io)?;
                let file: Arc<dyn VfsFile> = Arc::from(file);
                Ok((file, file_size))
            })
            .await
            .map_err(|e| ContextraError::Storage(format!("Join error: {}", e)))??;

        if file_size < 12 {
            return Err(ContextraError::Storage("SSTable file too small".into()));
        }

        // Read trailer: last 54 bytes (v1..v4) or 52 bytes (v0)
        let trailer_data = {
            let f = Arc::clone(&file);
            let read_len = usize::try_from(54u64.min(file_size))
                .map_err(|_| ContextraError::Storage("File size conversion failed".into()))?;
            tokio::task::spawn_blocking(move || -> std::io::Result<Vec<u8>> {
                let mut buf = vec![0u8; read_len];
                let offset = file_size.saturating_sub(54);
                pread_exact(f.as_ref(), &mut buf, offset)?;
                Ok(buf)
            })
            .await
            .map_err(|e| ContextraError::Storage(format!("Join error: {}", e)))?
            .map_err(|e| ContextraError::Storage(format!("Trailer read failed: {}", e)))?
        };

        let trailer_len = trailer_data.len();
        if trailer_len < 12 {
            return Err(ContextraError::Storage("Invalid trailer".into()));
        }

        let mut format_version = 0u16;
        let mut is_mfsx = false;

        if trailer_len >= 54 {
            if let Some(slice) = trailer_data.get(50..54) {
                let magic_v1 = u32::from_le_bytes(
                    slice
                        .try_into()
                        .map_err(|_| ContextraError::Storage("Invalid trailer".into()))?,
                );
                if magic_v1 == SSTABLE_MAGIC_MFSX {
                    if let Some(ver_slice) = trailer_data.get(48..50) {
                        format_version = u16::from_le_bytes(
                            ver_slice
                                .try_into()
                                .map_err(|_| ContextraError::Storage("Invalid trailer".into()))?,
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
                        .map_err(|_| ContextraError::Storage("Invalid trailer".into()))?,
                );
                if magic_v0 == SSTABLE_MAGIC_MFSX {
                    is_mfsx = true;
                    format_version = 0;
                }
            }
        }

        // Unknown format version error check (supports versions up to v4)
        if is_mfsx && format_version > 4 {
            return Err(ContextraError::Storage(format!(
                "Unsupported SSTable format version: {}",
                format_version
            )));
        }

        let mut has_bloom = false;
        let has_crc = is_mfsx;
        let mut bloom_offset = 0;
        let mut min_tx_id = 0;
        let mut max_tx_id = 0;
        let mut min_seq = 0;
        let mut max_seq = 0;

        let index_offset = if is_mfsx {
            let base = if format_version >= 1 { 0 } else { 2 };
            min_tx_id = u64::from_le_bytes(
                trailer_data
                    .get(base..base + 8)
                    .ok_or_else(|| ContextraError::ParseError("Invalid min_tx_id".into()))?
                    .try_into()
                    .map_err(|_| ContextraError::ParseError("Invalid min_tx_id".into()))?,
            );
            max_tx_id = u64::from_le_bytes(
                trailer_data
                    .get(base + 8..base + 16)
                    .ok_or_else(|| ContextraError::ParseError("Invalid max_tx_id".into()))?
                    .try_into()
                    .map_err(|_| ContextraError::ParseError("Invalid max_tx_id".into()))?,
            );
            min_seq = u64::from_le_bytes(
                trailer_data
                    .get(base + 16..base + 24)
                    .ok_or_else(|| ContextraError::ParseError("Invalid min_seq".into()))?
                    .try_into()
                    .map_err(|_| ContextraError::ParseError("Invalid min_seq".into()))?,
            );
            max_seq = u64::from_le_bytes(
                trailer_data
                    .get(base + 24..base + 32)
                    .ok_or_else(|| ContextraError::ParseError("Invalid max_seq".into()))?
                    .try_into()
                    .map_err(|_| ContextraError::ParseError("Invalid max_seq".into()))?,
            );
            bloom_offset = u64::from_le_bytes(
                trailer_data
                    .get(base + 32..base + 40)
                    .ok_or_else(|| ContextraError::ParseError("Invalid bloom_offset".into()))?
                    .try_into()
                    .map_err(|_| ContextraError::ParseError("Invalid bloom_offset".into()))?,
            );
            if bloom_offset > 0 {
                has_bloom = true;
            }
            u64::from_le_bytes(
                trailer_data
                    .get(base + 40..base + 48)
                    .ok_or_else(|| ContextraError::ParseError("Invalid index_offset".into()))?
                    .try_into()
                    .map_err(|_| ContextraError::ParseError("Invalid index_offset".into()))?,
            )
        } else if trailer_len >= 54 {
            let magic_legacy =
                u32::from_le_bytes(trailer_data[50..54].try_into().map_err(|_| {
                    ContextraError::checksum_mismatch(path_buf.to_string_lossy(), 0)
                })?);
            if magic_legacy == SSTABLE_MAGIC_LEGACY {
                u64::from_le_bytes(
                    trailer_data[42..50]
                        .try_into()
                        .map_err(|_| ContextraError::ParseError("Invalid index offset".into()))?,
                )
            } else {
                return Err(ContextraError::Storage(
                    "Invalid SSTable magic number".into(),
                ));
            }
        } else {
            return Err(ContextraError::Storage(
                "Invalid SSTable trailer length".into(),
            ));
        };

        let trailer_offset = file_size.saturating_sub(if is_mfsx {
            if format_version >= 1 {
                54
            } else {
                52
            }
        } else {
            12
        });

        if index_offset > trailer_offset
            || (has_bloom && (bloom_offset < index_offset || bloom_offset > trailer_offset))
        {
            return Err(ContextraError::Storage(
                "corrupted SSTable: invalid section offsets".into(),
            ));
        }

        let bloom_filter = if has_bloom {
            let bloom_end = if is_mfsx {
                file_size.saturating_sub(if format_version >= 1 { 54 } else { 52 })
            } else {
                file_size.saturating_sub(20)
            };

            let bloom_read_len =
                usize::try_from(bloom_end.saturating_sub(bloom_offset)).map_err(|_| {
                    ContextraError::Storage("Bloom filter read length conversion failed".into())
                })?;

            let bloom_data_raw = {
                let f = Arc::clone(&file);
                tokio::task::spawn_blocking(move || -> std::io::Result<Vec<u8>> {
                    let mut buf = vec![0u8; bloom_read_len];
                    pread_exact(f.as_ref(), &mut buf, bloom_offset)?;
                    Ok(buf)
                })
                .await
                .map_err(|e| ContextraError::Storage(format!("Join error: {}", e)))?
                .map_err(|e| ContextraError::Storage(format!("Bloom read failed: {}", e)))?
            };

            let bloom_data = if has_crc {
                if bloom_data_raw.len() < 4 {
                    return Err(ContextraError::Storage(
                        "Bloom filter data too short for CRC".into(),
                    ));
                }
                let stored_crc = u32::from_le_bytes(
                    bloom_data_raw[0..4]
                        .try_into()
                        .map_err(|_| ContextraError::Serialization("Invalid CRC".into()))?,
                );
                let payload = &bloom_data_raw[4..];
                if crc32fast::hash(payload) != stored_crc {
                    return Err(ContextraError::checksum_mismatch(
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
                bloom_offset
            } else {
                file_size.saturating_sub(if is_mfsx {
                    if format_version >= 1 {
                        54
                    } else {
                        52
                    }
                } else {
                    12
                })
            };

            let index_read_len =
                usize::try_from(index_end.saturating_sub(index_offset)).map_err(|_| {
                    ContextraError::Storage("Index read length conversion failed".into())
                })?;

            tokio::task::spawn_blocking(move || -> std::io::Result<Vec<u8>> {
                let mut buf = vec![0u8; index_read_len];
                pread_exact(f.as_ref(), &mut buf, index_offset)?;
                Ok(buf)
            })
            .await
            .map_err(|e| ContextraError::Storage(format!("Join error: {}", e)))?
            .map_err(|e| ContextraError::Storage(format!("Index read failed: {}", e)))?
        };

        let index_bytes_raw = Bytes::from(index_data_raw);
        let index_bytes = if has_crc {
            if index_bytes_raw.len() < 4 {
                return Err(ContextraError::Storage(
                    "Index data too short for CRC".into(),
                ));
            }
            let stored_crc = u32::from_le_bytes(
                index_bytes_raw[0..4]
                    .try_into()
                    .map_err(|_| ContextraError::Serialization("Invalid CRC".into()))?,
            );
            let payload = index_bytes_raw.slice(4..);
            if crc32fast::hash(&payload) != stored_crc {
                return Err(ContextraError::checksum_mismatch(
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
            let key_len = usize::from(u16::from_le_bytes(
                index_bytes[pos..pos + 2]
                    .try_into()
                    .map_err(|_| ContextraError::ParseError("corrupted index: key_len".into()))?,
            ));
            pos += 2;

            if pos + key_len + 8 > index_len {
                return Err(ContextraError::ParseError(
                    "corrupted index: data too short".into(),
                ));
            }

            let key = index_bytes.slice(pos..pos + key_len);
            pos += key_len;

            let offset = u64::from_le_bytes(
                index_bytes[pos..pos + 8]
                    .try_into()
                    .map_err(|_| ContextraError::ParseError("corrupted index: offset".into()))?,
            );
            pos += 8;

            if offset > index_offset {
                return Err(ContextraError::Storage(
                    "corrupted SSTable index: block offset exceeds index offset".into(),
                ));
            }

            if let Some((prev_key, prev_offset)) = index.last() {
                if key < *prev_key {
                    return Err(ContextraError::Storage(
                        "corrupted SSTable index: keys out of order".into(),
                    ));
                }
                if offset < *prev_offset {
                    return Err(ContextraError::Storage(
                        "corrupted SSTable index: block offsets out of order".into(),
                    ));
                }
            }

            index.push((key, offset));
        }

        if pos != index_len {
            return Err(ContextraError::ParseError(
                "corrupted index: trailing invalid bytes".into(),
            ));
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
            if block.len() < 2 {
                return Err(ContextraError::ParseError(
                    "corrupted block: header too short".into(),
                ));
            }
            let k_len =
                usize::from(u16::from_le_bytes(block[0..2].try_into().map_err(
                    |_| ContextraError::ParseError("corrupted block: k_len".into()),
                )?));
            let end_k = 2usize.checked_add(k_len).ok_or_else(|| {
                ContextraError::ParseError("corrupted block: k_len overflow".into())
            })?;
            if block.len() < end_k {
                return Err(ContextraError::ParseError(
                    "corrupted block: key truncated".into(),
                ));
            }
            block.slice(2..end_k)
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
            format_version,
        })
    }
}
