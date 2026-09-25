use contextra_core::{ContextraError, Result};
use std::path::PathBuf;

use super::MAX_MANIFEST_ENTRY_SIZE;

/// An entry in the SSTable manifest log.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ManifestEntry {
    /// A new SSTable file was created and fully fsynced.
    Add { path: PathBuf, max_tx: u64 },
    /// An SSTable file was removed or superseded.
    Remove { path: PathBuf },
    /// A transaction rollback operation completed.
    RollbackComplete { target_tx: u64 },
    /// An atomic replacement of multiple input SSTables by a single compacted output SSTable.
    Replace {
        removed: Vec<PathBuf>,
        added: PathBuf,
        added_max_tx: u64,
        rank: u64,
    },
}

impl ManifestEntry {
    /// Serializes the entry into binary format with a 4-byte total size prefix and a 4-byte CRC32.
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        let mut payload = Vec::new();
        match self {
            ManifestEntry::Add { path, max_tx } => {
                payload.push(0u8); // op_tag = 0
                payload.extend_from_slice(&max_tx.to_le_bytes());
                let path_str = path.to_string_lossy();
                let path_bytes = path_str.as_bytes();
                payload.extend_from_slice(&(path_bytes.len() as u32).to_le_bytes());
                payload.extend_from_slice(path_bytes);
            }
            ManifestEntry::Remove { path } => {
                payload.push(1u8); // op_tag = 1
                let path_str = path.to_string_lossy();
                let path_bytes = path_str.as_bytes();
                payload.extend_from_slice(&(path_bytes.len() as u32).to_le_bytes());
                payload.extend_from_slice(path_bytes);
            }
            ManifestEntry::RollbackComplete { target_tx } => {
                payload.push(2u8); // op_tag = 2
                payload.extend_from_slice(&target_tx.to_le_bytes());
            }
            ManifestEntry::Replace {
                removed,
                added,
                added_max_tx,
                rank,
            } => {
                payload.push(3u8); // op_tag = 3
                payload.extend_from_slice(&added_max_tx.to_le_bytes());
                payload.extend_from_slice(&rank.to_le_bytes());
                let added_str = added.to_string_lossy();
                let added_bytes = added_str.as_bytes();
                payload.extend_from_slice(&(added_bytes.len() as u32).to_le_bytes());
                payload.extend_from_slice(added_bytes);
                payload.extend_from_slice(&(removed.len() as u32).to_le_bytes());
                for p in removed {
                    let p_str = p.to_string_lossy();
                    let p_bytes = p_str.as_bytes();
                    payload.extend_from_slice(&(p_bytes.len() as u32).to_le_bytes());
                    payload.extend_from_slice(p_bytes);
                }
            }
        }

        let total_payload_size = (4 + payload.len()) as u32;
        if total_payload_size > MAX_MANIFEST_ENTRY_SIZE {
            return Err(ContextraError::Serialization(format!(
                "Manifest entry exceeds max size: {} bytes",
                total_payload_size
            )));
        }

        let crc = crc32fast::hash(&payload);

        let mut buf = Vec::with_capacity(4 + total_payload_size as usize);
        buf.extend_from_slice(&total_payload_size.to_le_bytes());
        buf.extend_from_slice(&crc.to_le_bytes());
        buf.extend_from_slice(&payload);

        Ok(buf)
    }

    /// Deserializes a manifest entry from a slice containing `[crc32: u32 LE] [payload...]`.
    #[deny(clippy::indexing_slicing)]
    pub fn from_bytes(data: &[u8]) -> Result<Self> {
        if data.len() < 5 {
            return Err(ContextraError::Serialization(
                "Manifest entry too short".into(),
            ));
        }

        let stored_crc_bytes = data.get(0..4).ok_or_else(|| {
            ContextraError::Serialization("Failed to read stored CRC bytes".into())
        })?;
        let stored_crc = u32::from_le_bytes(
            stored_crc_bytes
                .try_into()
                .map_err(|_| ContextraError::Serialization("Invalid CRC format".into()))?,
        );
        let payload = data
            .get(4..)
            .ok_or_else(|| ContextraError::Serialization("Failed to read payload bytes".into()))?;
        let computed_crc = crc32fast::hash(payload);

        if stored_crc != computed_crc {
            return Err(ContextraError::Serialization(format!(
                "CRC mismatch: stored={:#010x}, computed={:#010x}",
                stored_crc, computed_crc
            )));
        }

        let op_tag = *payload
            .first()
            .ok_or_else(|| ContextraError::Serialization("Failed to read op_tag".into()))?;
        let remaining = payload.get(1..).ok_or_else(|| {
            ContextraError::Serialization("Failed to read remaining payload".into())
        })?;

        match op_tag {
            0 => {
                // Add
                if remaining.len() < 12 {
                    return Err(ContextraError::Serialization(
                        "Add payload too short".into(),
                    ));
                }
                let max_tx_bytes = remaining.get(0..8).ok_or_else(|| {
                    ContextraError::Serialization("Add max_tx bytes missing".into())
                })?;
                let max_tx =
                    u64::from_le_bytes(max_tx_bytes.try_into().map_err(|_| {
                        ContextraError::Serialization("Invalid max_tx format".into())
                    })?);
                let path_len_bytes = remaining.get(8..12).ok_or_else(|| {
                    ContextraError::Serialization("Add path_len bytes missing".into())
                })?;
                let path_len =
                    u32::from_le_bytes(path_len_bytes.try_into().map_err(|_| {
                        ContextraError::Serialization("Invalid path_len format".into())
                    })?) as usize;
                if remaining.len() < 12 + path_len {
                    return Err(ContextraError::Serialization(
                        "Add path data truncated".into(),
                    ));
                }
                let path_bytes = remaining.get(12..12 + path_len).ok_or_else(|| {
                    ContextraError::Serialization("Add path bytes missing".into())
                })?;
                let path_str = std::str::from_utf8(path_bytes).map_err(|e| {
                    ContextraError::Serialization(format!("Invalid path UTF-8: {}", e))
                })?;
                Ok(ManifestEntry::Add {
                    path: PathBuf::from(path_str),
                    max_tx,
                })
            }
            1 => {
                // Remove
                if remaining.len() < 4 {
                    return Err(ContextraError::Serialization(
                        "Remove payload too short".into(),
                    ));
                }
                let path_len_bytes = remaining.get(0..4).ok_or_else(|| {
                    ContextraError::Serialization("Remove path_len bytes missing".into())
                })?;
                let path_len =
                    u32::from_le_bytes(path_len_bytes.try_into().map_err(|_| {
                        ContextraError::Serialization("Invalid path_len format".into())
                    })?) as usize;
                if remaining.len() < 4 + path_len {
                    return Err(ContextraError::Serialization(
                        "Remove path data truncated".into(),
                    ));
                }
                let path_bytes = remaining.get(4..4 + path_len).ok_or_else(|| {
                    ContextraError::Serialization("Remove path bytes missing".into())
                })?;
                let path_str = std::str::from_utf8(path_bytes).map_err(|e| {
                    ContextraError::Serialization(format!("Invalid path UTF-8: {}", e))
                })?;
                Ok(ManifestEntry::Remove {
                    path: PathBuf::from(path_str),
                })
            }
            2 => {
                // RollbackComplete
                if remaining.len() < 8 {
                    return Err(ContextraError::Serialization(
                        "RollbackComplete payload too short".into(),
                    ));
                }
                let target_tx_bytes = remaining.get(0..8).ok_or_else(|| {
                    ContextraError::Serialization("target_tx bytes missing".into())
                })?;
                let target_tx = u64::from_le_bytes(target_tx_bytes.try_into().map_err(|_| {
                    ContextraError::Serialization("Invalid target_tx format".into())
                })?);
                Ok(ManifestEntry::RollbackComplete { target_tx })
            }
            3 => {
                // Replace
                if remaining.len() < 24 {
                    return Err(ContextraError::Serialization(
                        "Replace payload header too short".into(),
                    ));
                }
                let added_max_tx_bytes = remaining.get(0..8).ok_or_else(|| {
                    ContextraError::Serialization("added_max_tx bytes missing".into())
                })?;
                let added_max_tx =
                    u64::from_le_bytes(added_max_tx_bytes.try_into().map_err(|_| {
                        ContextraError::Serialization("Invalid added_max_tx format".into())
                    })?);
                let rank_bytes = remaining
                    .get(8..16)
                    .ok_or_else(|| ContextraError::Serialization("rank bytes missing".into()))?;
                let rank =
                    u64::from_le_bytes(rank_bytes.try_into().map_err(|_| {
                        ContextraError::Serialization("Invalid rank format".into())
                    })?);
                let added_path_len_bytes = remaining.get(16..20).ok_or_else(|| {
                    ContextraError::Serialization("added_path_len bytes missing".into())
                })?;
                let added_path_len =
                    u32::from_le_bytes(added_path_len_bytes.try_into().map_err(|_| {
                        ContextraError::Serialization("Invalid added_path_len format".into())
                    })?) as usize;
                let mut offset = 20;
                if remaining.len() < offset + added_path_len {
                    return Err(ContextraError::Serialization(
                        "Replace added path data truncated".into(),
                    ));
                }
                let added_path_bytes =
                    remaining
                        .get(offset..offset + added_path_len)
                        .ok_or_else(|| {
                            ContextraError::Serialization("added_path bytes missing".into())
                        })?;
                let added_str = std::str::from_utf8(added_path_bytes).map_err(|e| {
                    ContextraError::Serialization(format!("Invalid added path UTF-8: {}", e))
                })?;
                let added = PathBuf::from(added_str);
                offset += added_path_len;

                if remaining.len() < offset + 4 {
                    return Err(ContextraError::Serialization(
                        "Replace removed count truncated".into(),
                    ));
                }
                let removed_count_bytes = remaining.get(offset..offset + 4).ok_or_else(|| {
                    ContextraError::Serialization("removed_count bytes missing".into())
                })?;
                let removed_count =
                    u32::from_le_bytes(removed_count_bytes.try_into().map_err(|_| {
                        ContextraError::Serialization("Invalid removed_count format".into())
                    })?) as usize;
                offset += 4;

                let mut removed = Vec::with_capacity(removed_count);
                for _ in 0..removed_count {
                    if remaining.len() < offset + 4 {
                        return Err(ContextraError::Serialization(
                            "Replace removed path length truncated".into(),
                        ));
                    }
                    let r_len_bytes = remaining.get(offset..offset + 4).ok_or_else(|| {
                        ContextraError::Serialization("r_len bytes missing".into())
                    })?;
                    let r_len = u32::from_le_bytes(r_len_bytes.try_into().map_err(|_| {
                        ContextraError::Serialization("Invalid r_len format".into())
                    })?) as usize;
                    offset += 4;
                    if remaining.len() < offset + r_len {
                        return Err(ContextraError::Serialization(
                            "Replace removed path data truncated".into(),
                        ));
                    }
                    let r_bytes = remaining.get(offset..offset + r_len).ok_or_else(|| {
                        ContextraError::Serialization("r_path bytes missing".into())
                    })?;
                    let r_str = std::str::from_utf8(r_bytes).map_err(|e| {
                        ContextraError::Serialization(format!("Invalid removed path UTF-8: {}", e))
                    })?;
                    removed.push(PathBuf::from(r_str));
                    offset += r_len;
                }

                Ok(ManifestEntry::Replace {
                    removed,
                    added,
                    added_max_tx,
                    rank,
                })
            }
            _ => Err(ContextraError::Serialization(format!(
                "Unknown Manifest op tag: {}",
                op_tag
            ))),
        }
    }
}
