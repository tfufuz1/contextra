//! SSTable Manifest for append-only tracking of active SSTable sets.
// FILE-CONTEXT
// STAND: 2026-09-11
// ZWECK: Append-Only Manifest-Protokolldatei zur Verfolgung gültiger SSTables für Crash-Safety.
// INVARIANTEN: fsync NACH jedem Manifest-Eintrag; Add erst nach fsync der SSTable-Datei.

use memfuse_core::{MemFuseError, Result};
use std::path::{Path, PathBuf};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

/// Maximum allowed payload size for a single manifest entry (1 MB).
pub const MAX_MANIFEST_ENTRY_SIZE: u32 = 1024 * 1024;

/// Default file size threshold (64 KB) to trigger MANIFEST rollover.
pub const DEFAULT_ROLLOVER_THRESHOLD_BYTES: u64 = 64 * 1024;

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
            return Err(MemFuseError::Serialization(format!(
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
    pub fn from_bytes(data: &[u8]) -> Result<Self> {
        if data.len() < 5 {
            return Err(MemFuseError::Serialization(
                "Manifest entry too short".into(),
            ));
        }

        let stored_crc = u32::from_le_bytes(
            data[0..4]
                .try_into()
                .map_err(|_| MemFuseError::Serialization("Invalid CRC format".into()))?,
        );
        let payload = &data[4..];
        let computed_crc = crc32fast::hash(payload);

        if stored_crc != computed_crc {
            return Err(MemFuseError::Serialization(format!(
                "CRC mismatch: stored={:#010x}, computed={:#010x}",
                stored_crc, computed_crc
            )));
        }

        let op_tag = payload[0];
        let remaining = &payload[1..];

        match op_tag {
            0 => {
                // Add
                if remaining.len() < 12 {
                    return Err(MemFuseError::Serialization("Add payload too short".into()));
                }
                let max_tx =
                    u64::from_le_bytes(remaining[0..8].try_into().map_err(|_| {
                        MemFuseError::Serialization("Invalid max_tx format".into())
                    })?);
                let path_len =
                    u32::from_le_bytes(remaining[8..12].try_into().map_err(|_| {
                        MemFuseError::Serialization("Invalid path_len format".into())
                    })?) as usize;
                if remaining.len() < 12 + path_len {
                    return Err(MemFuseError::Serialization(
                        "Add path data truncated".into(),
                    ));
                }
                let path_str = std::str::from_utf8(&remaining[12..12 + path_len]).map_err(|e| {
                    MemFuseError::Serialization(format!("Invalid path UTF-8: {}", e))
                })?;
                Ok(ManifestEntry::Add {
                    path: PathBuf::from(path_str),
                    max_tx,
                })
            }
            1 => {
                // Remove
                if remaining.len() < 4 {
                    return Err(MemFuseError::Serialization(
                        "Remove payload too short".into(),
                    ));
                }
                let path_len =
                    u32::from_le_bytes(remaining[0..4].try_into().map_err(|_| {
                        MemFuseError::Serialization("Invalid path_len format".into())
                    })?) as usize;
                if remaining.len() < 4 + path_len {
                    return Err(MemFuseError::Serialization(
                        "Remove path data truncated".into(),
                    ));
                }
                let path_str = std::str::from_utf8(&remaining[4..4 + path_len]).map_err(|e| {
                    MemFuseError::Serialization(format!("Invalid path UTF-8: {}", e))
                })?;
                Ok(ManifestEntry::Remove {
                    path: PathBuf::from(path_str),
                })
            }
            2 => {
                // RollbackComplete
                if remaining.len() < 8 {
                    return Err(MemFuseError::Serialization(
                        "RollbackComplete payload too short".into(),
                    ));
                }
                let target_tx =
                    u64::from_le_bytes(remaining[0..8].try_into().map_err(|_| {
                        MemFuseError::Serialization("Invalid target_tx format".into())
                    })?);
                Ok(ManifestEntry::RollbackComplete { target_tx })
            }
            3 => {
                // Replace
                if remaining.len() < 24 {
                    return Err(MemFuseError::Serialization(
                        "Replace payload header too short".into(),
                    ));
                }
                let added_max_tx =
                    u64::from_le_bytes(remaining[0..8].try_into().map_err(|_| {
                        MemFuseError::Serialization("Invalid added_max_tx format".into())
                    })?);
                let rank =
                    u64::from_le_bytes(remaining[8..16].try_into().map_err(|_| {
                        MemFuseError::Serialization("Invalid rank format".into())
                    })?);
                let added_path_len =
                    u32::from_le_bytes(remaining[16..20].try_into().map_err(|_| {
                        MemFuseError::Serialization("Invalid added_path_len format".into())
                    })?) as usize;
                let mut offset = 20;
                if remaining.len() < offset + added_path_len {
                    return Err(MemFuseError::Serialization(
                        "Replace added path data truncated".into(),
                    ));
                }
                let added_str = std::str::from_utf8(&remaining[offset..offset + added_path_len])
                    .map_err(|e| {
                        MemFuseError::Serialization(format!("Invalid added path UTF-8: {}", e))
                    })?;
                let added = PathBuf::from(added_str);
                offset += added_path_len;

                if remaining.len() < offset + 4 {
                    return Err(MemFuseError::Serialization(
                        "Replace removed count truncated".into(),
                    ));
                }
                let removed_count =
                    u32::from_le_bytes(remaining[offset..offset + 4].try_into().map_err(|_| {
                        MemFuseError::Serialization("Invalid removed_count format".into())
                    })?) as usize;
                offset += 4;

                let mut removed = Vec::with_capacity(removed_count);
                for _ in 0..removed_count {
                    if remaining.len() < offset + 4 {
                        return Err(MemFuseError::Serialization(
                            "Replace removed path length truncated".into(),
                        ));
                    }
                    let r_len = u32::from_le_bytes(
                        remaining[offset..offset + 4]
                            .try_into()
                            .map_err(|_| MemFuseError::Serialization("Invalid r_len format".into()))?,
                    ) as usize;
                    offset += 4;
                    if remaining.len() < offset + r_len {
                        return Err(MemFuseError::Serialization(
                            "Replace removed path data truncated".into(),
                        ));
                    }
                    let r_str =
                        std::str::from_utf8(&remaining[offset..offset + r_len]).map_err(|e| {
                            MemFuseError::Serialization(format!("Invalid removed path UTF-8: {}", e))
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
            _ => Err(MemFuseError::Serialization(format!(
                "Unknown Manifest op tag: {}",
                op_tag
            ))),
        }
    }
}

/// Append-only Manifest file handle for recording active SSTable set transitions.
pub struct Manifest {
    path: PathBuf,
    file: tokio::sync::Mutex<tokio::fs::File>,
}

impl std::fmt::Debug for Manifest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Manifest")
            .field("path", &self.path)
            .finish()
    }
}

impl Manifest {
    /// Opens or creates an append-only Manifest file at `path`.
    pub async fn open(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        let (file, is_new) = match tokio::fs::OpenOptions::new()
            .create_new(true)
            .append(true)
            .read(true)
            .open(&path)
            .await
        {
            Ok(file) => (file, true),
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                let file = tokio::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .read(true)
                    .open(&path)
                    .await
                    .map_err(|e| {
                        MemFuseError::Storage(format!("Failed to open MANIFEST: {}", e))
                    })?;
                (file, false)
            }
            Err(e) => {
                return Err(MemFuseError::Storage(format!(
                    "Failed to create MANIFEST: {}",
                    e
                )));
            }
        };

        if is_new {
            file.sync_all().await.map_err(|e| {
                MemFuseError::Storage(format!(
                    "MANIFEST file fsync failed for {}: {}",
                    path.display(),
                    e
                ))
            })?;
            crate::util::fsync_parent_dir(&path).await?;
        }

        Ok(Self {
            path,
            file: tokio::sync::Mutex::new(file),
        })
    }

    /// Appends a new entry to the manifest and performs flush + fsync.
    pub async fn append(&self, entry: &ManifestEntry) -> Result<()> {
        let bytes = entry.to_bytes()?;
        let mut file = self.file.lock().await;
        file.write_all(&bytes).await.map_err(|e| {
            MemFuseError::Storage(format!(
                "MANIFEST write failed for {}: {}",
                self.path.display(),
                e
            ))
        })?;
        file.flush().await.map_err(|e| {
            MemFuseError::Storage(format!(
                "MANIFEST flush failed for {}: {}",
                self.path.display(),
                e
            ))
        })?;
        file.sync_all().await.map_err(|e| {
            MemFuseError::Storage(format!(
                "MANIFEST fsync failed for {}: {}",
                self.path.display(),
                e
            ))
        })?;
        Ok(())
    }

    /// Loads all valid manifest entries from `path`.
    ///
    /// If the file does not exist, returns an empty vector.
    /// Tail-truncation (an incomplete frame at EOF caused by interrupted write / power cut) is recovered
    /// safely by returning valid entries read up to the point of truncation.
    /// Mid-file corruption or CRC mismatches on complete frames return `Err`.
    pub async fn load(path: &Path) -> Result<Vec<ManifestEntry>> {
        if !tokio::fs::try_exists(path).await.unwrap_or(false) {
            return Ok(Vec::new());
        }

        let mut file = match tokio::fs::File::open(path).await {
            Ok(f) => f,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(e) => {
                return Err(MemFuseError::Storage(format!(
                    "Failed to open MANIFEST for reading: {}",
                    e
                )))
            }
        };

        let file_size = file
            .metadata()
            .await
            .map_err(|e| MemFuseError::Storage(e.to_string()))?
            .len();

        let mut reader = tokio::io::BufReader::new(&mut file);
        let mut entries = Vec::new();
        let mut pos = 0u64;

        loop {
            if pos == file_size {
                break;
            }

            let mut len_bytes = [0u8; 4];
            match reader.read_exact(&mut len_bytes).await {
                Ok(_) => {}
                Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                    tracing::warn!(
                        "MANIFEST tail truncation detected (incomplete length header) at offset {}",
                        pos
                    );
                    break;
                }
                Err(e) => {
                    return Err(MemFuseError::Storage(format!(
                        "MANIFEST read error at offset {}: {}",
                        pos, e
                    )));
                }
            }

            let len = u32::from_le_bytes(len_bytes) as usize;
            if len > MAX_MANIFEST_ENTRY_SIZE as usize {
                return Err(MemFuseError::Storage(format!(
                    "MANIFEST entry length ({}) exceeds max size ({}) at offset {}",
                    len, MAX_MANIFEST_ENTRY_SIZE, pos
                )));
            }

            if pos + 4 + len as u64 > file_size {
                // Legitimate tail-truncation after crash during write
                tracing::warn!(
                    "MANIFEST tail truncation detected at offset {} (expected record len {} exceeds file size {}) — breaking load loop",
                    pos,
                    len,
                    file_size
                );
                break;
            }

            let mut entry_raw = vec![0u8; len];
            match reader.read_exact(&mut entry_raw).await {
                Ok(_) => {}
                Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                    tracing::warn!(
                        "MANIFEST tail truncation detected (incomplete payload) at offset {}",
                        pos
                    );
                    break;
                }
                Err(e) => {
                    return Err(MemFuseError::Storage(format!(
                        "MANIFEST read error at offset {}: {}",
                        pos, e
                    )));
                }
            }

            let entry_pos = pos;
            pos += 4 + len as u64;

            match ManifestEntry::from_bytes(&entry_raw) {
                Ok(entry) => entries.push(entry),
                Err(e) => {
                    return Err(MemFuseError::Storage(format!(
                        "MANIFEST entry corruption at offset {}: {}",
                        entry_pos, e
                    )));
                }
            }
        }

        Ok(entries)
    }

    /// Reconstructs the ordered list of currently valid SSTable file names (or path components)
    /// and their rank (shadowing order: smaller rank = older) from a sequence of manifest entries.
    pub fn reconstruct_valid_sstables(entries: &[ManifestEntry]) -> Vec<(PathBuf, u64)> {
        let mut valid_map = std::collections::HashMap::new();
        let mut next_rank = 0u64;

        for entry in entries {
            match entry {
                ManifestEntry::Add { path, .. } => {
                    let key = path
                        .file_name()
                        .map(PathBuf::from)
                        .unwrap_or_else(|| path.clone());
                    valid_map.insert(key, next_rank);
                    next_rank += 1;
                }
                ManifestEntry::Remove { path } => {
                    let key = path
                        .file_name()
                        .map(PathBuf::from)
                        .unwrap_or_else(|| path.clone());
                    valid_map.remove(&key);
                }
                ManifestEntry::RollbackComplete { .. } => {}
                ManifestEntry::Replace {
                    removed,
                    added,
                    rank,
                    ..
                } => {
                    for p in removed {
                        let key = p
                            .file_name()
                            .map(PathBuf::from)
                            .unwrap_or_else(|| p.clone());
                        valid_map.remove(&key);
                    }
                    let key = added
                        .file_name()
                        .map(PathBuf::from)
                        .unwrap_or_else(|| added.clone());
                    valid_map.insert(key, *rank);
                    if *rank >= next_rank {
                        next_rank = rank + 1;
                    }
                }
            }
        }

        let mut sorted: Vec<(PathBuf, u64)> = valid_map.into_iter().collect();
        sorted.sort_by_key(|(_, rank)| *rank);
        sorted
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Evaluates whether the MANIFEST file size exceeds `threshold_bytes` and performs an atomic
    /// rollover if needed.
    ///
    /// Rollover process:
    /// (a) Write current live entries (`live_entries`) as a snapshot into a temporary `MANIFEST.new` file.
    /// (b) `flush` + `sync_all` on the temporary file.
    /// (c) Atomically `rename` temporary file to `MANIFEST`.
    /// (d) `fsync` the parent directory.
    /// (e) Re-open `MANIFEST` handle in append mode and update `self.file`.
    ///
    /// If an error occurs prior to `rename`, the temporary file is cleaned up and the original `MANIFEST`
    /// remains untouched and valid.
    pub async fn maybe_rollover(
        &self,
        live_entries: &[ManifestEntry],
        threshold_bytes: u64,
    ) -> Result<bool> {
        let mut file = self.file.lock().await;
        let metadata = file.metadata().await.map_err(|e| {
            MemFuseError::Storage(format!("Failed to get MANIFEST file metadata: {e}"))
        })?;

        if metadata.len() < threshold_bytes {
            return Ok(false);
        }

        tracing::info!(
            manifest_size = metadata.len(),
            threshold_bytes = threshold_bytes,
            "MANIFEST size threshold exceeded; initiating atomic rollover"
        );

        let parent = self
            .path
            .parent()
            .ok_or_else(|| MemFuseError::Storage("Invalid MANIFEST parent directory".into()))?;

        let pid = std::process::id();
        let rand_val: u64 = rand::random();
        let tmp_path = parent.join(format!("MANIFEST.new.{}.{}", pid, rand_val));

        let rollover_res: Result<()> = async {
            let mut new_file = tokio::fs::OpenOptions::new()
                .create_new(true)
                .write(true)
                .open(&tmp_path)
                .await
                .map_err(|e| {
                    MemFuseError::Storage(format!(
                        "Failed to create temporary MANIFEST file {:?}: {e}",
                        tmp_path
                    ))
                })?;

            for entry in live_entries {
                let bytes = entry.to_bytes()?;
                new_file.write_all(&bytes).await.map_err(|e| {
                    MemFuseError::Storage(format!(
                        "Failed to write entry to temporary MANIFEST {:?}: {e}",
                        tmp_path
                    ))
                })?;
            }

            new_file.flush().await.map_err(|e| {
                MemFuseError::Storage(format!(
                    "Failed to flush temporary MANIFEST {:?}: {e}",
                    tmp_path
                ))
            })?;
            new_file.sync_all().await.map_err(|e| {
                MemFuseError::Storage(format!(
                    "Failed to fsync temporary MANIFEST {:?}: {e}",
                    tmp_path
                ))
            })?;

            drop(new_file);

            tokio::fs::rename(&tmp_path, &self.path).await.map_err(|e| {
                MemFuseError::Storage(format!(
                    "Failed to rename temporary MANIFEST from {:?} to {:?}: {e}",
                    tmp_path, self.path
                ))
            })?;

            crate::util::fsync_parent_dir(&self.path).await?;

            let reopened_file = tokio::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .read(true)
                .open(&self.path)
                .await
                .map_err(|e| {
                    MemFuseError::Storage(format!(
                        "Failed to reopen MANIFEST after rollover {:?}: {e}",
                        self.path
                    ))
                })?;

            *file = reopened_file;
            Ok(())
        }
        .await;

        if let Err(ref e) = rollover_res {
            tracing::error!(
                path = ?self.path,
                tmp_path = ?tmp_path,
                error = %e,
                "MANIFEST rollover failed"
            );
            if let Err(clean_err) = tokio::fs::remove_file(&tmp_path).await {
                if clean_err.kind() != std::io::ErrorKind::NotFound {
                    tracing::warn!(
                        tmp_path = ?tmp_path,
                        error = %clean_err,
                        "Failed to remove temporary MANIFEST file after failed rollover"
                    );
                }
            }
            return Err(rollover_res.unwrap_err());
        }

        tracing::info!(
            path = ?self.path,
            live_entries_count = live_entries.len(),
            "MANIFEST rollover completed successfully"
        );

        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_manifest_entry_roundtrip() {
        let entries = vec![
            ManifestEntry::Add {
                path: PathBuf::from("sst-00000000000000000001-000000.sst"),
                max_tx: 42,
            },
            ManifestEntry::Remove {
                path: PathBuf::from("sst-00000000000000000001-000000.sst"),
            },
            ManifestEntry::RollbackComplete { target_tx: 100 },
            ManifestEntry::Replace {
                removed: vec![
                    PathBuf::from("sst-00000000000000000001-000000.sst"),
                    PathBuf::from("sst-00000000000000000002-000000.sst"),
                ],
                added: PathBuf::from("sst-compact-00000000000000000003-0000.sst"),
                added_max_tx: 50,
                rank: 1,
            },
        ];

        for entry in entries {
            let bytes = entry.to_bytes().expect("serialization should succeed");
            let payload_from_bytes = &bytes[4..]; // Skip total_payload_size prefix
            let decoded = ManifestEntry::from_bytes(payload_from_bytes)
                .expect("deserialization should succeed");
            assert_eq!(entry, decoded);
        }
    }

    #[tokio::test]
    async fn test_manifest_crc_corruption_detection() {
        let dir = tempdir().expect("tempdir");
        let manifest_path = dir.path().join("MANIFEST");

        let manifest = Manifest::open(&manifest_path).await.expect("open manifest");

        let entry1 = ManifestEntry::Add {
            path: PathBuf::from("sst-1.sst"),
            max_tx: 10,
        };
        let entry2 = ManifestEntry::Add {
            path: PathBuf::from("sst-2.sst"),
            max_tx: 20,
        };

        manifest.append(&entry1).await.expect("append 1");
        manifest.append(&entry2).await.expect("append 2");
        drop(manifest);

        // Corrupt entry 2 (flip bytes near the end of the file)
        let mut file_bytes = tokio::fs::read(&manifest_path).await.expect("read file");
        let len = file_bytes.len();
        file_bytes[len - 2] ^= 0xFF;
        tokio::fs::write(&manifest_path, file_bytes)
            .await
            .expect("write corrupted file");

        let loaded_res = Manifest::load(&manifest_path).await;
        assert!(
            loaded_res.is_err(),
            "Internal corruption in MANIFEST must return Err instead of silent degradation"
        );
    }

    #[tokio::test]
    async fn test_manifest_truncated_tail_recovery() {
        let dir = tempdir().expect("tempdir");
        let manifest_path = dir.path().join("MANIFEST");

        let manifest = Manifest::open(&manifest_path).await.expect("open manifest");

        let entry1 = ManifestEntry::Add {
            path: PathBuf::from("sst-1.sst"),
            max_tx: 10,
        };
        let entry2 = ManifestEntry::Add {
            path: PathBuf::from("sst-2.sst"),
            max_tx: 20,
        };

        manifest.append(&entry1).await.expect("append 1");
        manifest.append(&entry2).await.expect("append 2");
        drop(manifest);

        // Truncate the file in the middle of entry 2
        let mut file_bytes = tokio::fs::read(&manifest_path).await.expect("read file");
        file_bytes.truncate(file_bytes.len() - 8);
        tokio::fs::write(&manifest_path, file_bytes)
            .await
            .expect("write truncated file");

        let loaded = Manifest::load(&manifest_path).await.expect("load manifest");
        assert_eq!(
            loaded.len(),
            1,
            "Truncated tail entry should be safely ignored"
        );
        assert_eq!(loaded[0], entry1);
    }

    #[test]
    fn test_reconstruct_valid_sstables() {
        let entries = vec![
            ManifestEntry::Add {
                path: PathBuf::from("/data/sst-1.sst"),
                max_tx: 10,
            },
            ManifestEntry::Add {
                path: PathBuf::from("sst-2.sst"),
                max_tx: 20,
            },
            ManifestEntry::Remove {
                path: PathBuf::from("/data/sst-1.sst"),
            },
            ManifestEntry::Add {
                path: PathBuf::from("sst-3.sst"),
                max_tx: 30,
            },
            ManifestEntry::Replace {
                removed: vec![PathBuf::from("sst-2.sst"), PathBuf::from("sst-3.sst")],
                added: PathBuf::from("sst-compact-1.sst"),
                added_max_tx: 30,
                rank: 1,
            },
        ];

        let valid = Manifest::reconstruct_valid_sstables(&entries);
        assert_eq!(valid.len(), 1);
        assert_eq!(valid[0], (PathBuf::from("sst-compact-1.sst"), 1));
    }

    #[tokio::test]
    async fn test_manifest_rollover_reduces_size_and_preserves_live_sstables() {
        let dir = tempdir().expect("tempdir");
        let manifest_path = dir.path().join("MANIFEST");

        let manifest = Manifest::open(&manifest_path).await.expect("open manifest");

        // Add 50 SSTables, then Remove 45 of them to generate a long history with few live SSTables
        for i in 1..=50 {
            let entry = ManifestEntry::Add {
                path: PathBuf::from(format!("sst-{:04}.sst", i)),
                max_tx: i as u64,
            };
            manifest.append(&entry).await.expect("append add");
        }
        for i in 1..=45 {
            let entry = ManifestEntry::Remove {
                path: PathBuf::from(format!("sst-{:04}.sst", i)),
            };
            manifest.append(&entry).await.expect("append remove");
        }

        let pre_entries = Manifest::load(&manifest_path)
            .await
            .expect("load pre-rollover entries");
        let pre_live_set = Manifest::reconstruct_valid_sstables(&pre_entries);
        assert_eq!(pre_live_set.len(), 5);

        let pre_size = tokio::fs::metadata(&manifest_path)
            .await
            .expect("pre metadata")
            .len();

        let live_entries_snapshot: Vec<ManifestEntry> = (46..=50)
            .map(|i| ManifestEntry::Add {
                path: PathBuf::from(format!("sst-{:04}.sst", i)),
                max_tx: i as u64,
            })
            .collect();

        // Perform rollover with threshold lower than pre_size
        let rolled_over = manifest
            .maybe_rollover(&live_entries_snapshot, 100)
            .await
            .expect("maybe_rollover should succeed");
        assert!(rolled_over, "Rollover should have been triggered");

        let post_size = tokio::fs::metadata(&manifest_path)
            .await
            .expect("post metadata")
            .len();
        assert!(
            post_size < pre_size,
            "Post-rollover size ({}) must be strictly smaller than pre-rollover size ({})",
            post_size,
            pre_size
        );

        let post_entries = Manifest::load(&manifest_path)
            .await
            .expect("load post-rollover entries");
        let post_live_set = Manifest::reconstruct_valid_sstables(&post_entries);
        assert_eq!(
            pre_live_set, post_live_set,
            "Live SSTable set after rollover must be identical to pre-rollover live set"
        );

        // Verify reopened manifest handle can still append new entries
        let new_entry = ManifestEntry::Add {
            path: PathBuf::from("sst-0051.sst"),
            max_tx: 51,
        };
        manifest.append(&new_entry).await.expect("append post-rollover");

        let final_entries = Manifest::load(&manifest_path)
            .await
            .expect("load final entries");
        let final_live_set = Manifest::reconstruct_valid_sstables(&final_entries);
        assert_eq!(final_live_set.len(), 6);
        assert!(final_live_set.contains(Path::new("sst-0051.sst")));
    }

    #[tokio::test]
    async fn test_manifest_rollover_crash_injection_preserves_old_manifest() {
        use crate::lsm::{LsmConfig, LsmStorage};

        let dir = tempdir().expect("tempdir");
        let manifest_path = dir.path().join("MANIFEST");

        let manifest = Manifest::open(&manifest_path).await.expect("open manifest");

        let entry1 = ManifestEntry::Add {
            path: PathBuf::from("sst-1.sst"),
            max_tx: 10,
        };
        let entry2 = ManifestEntry::Add {
            path: PathBuf::from("sst-2.sst"),
            max_tx: 20,
        };
        manifest.append(&entry1).await.expect("append entry 1");
        manifest.append(&entry2).await.expect("append entry 2");
        drop(manifest);

        // Simulate crash before rename: leftover temp file MANIFEST.new.123.456 exists
        let leftover_tmp = dir.path().join("MANIFEST.new.123.456");
        tokio::fs::write(&leftover_tmp, b"partial manifest content from crash")
            .await
            .expect("write leftover temp file");

        // Verify loading original MANIFEST is unaffected by leftover temp file
        let loaded = Manifest::load(&manifest_path)
            .await
            .expect("load manifest during crash recovery");
        assert_eq!(loaded.len(), 2);
        assert_eq!(loaded[0], entry1);
        assert_eq!(loaded[1], entry2);

        // Verify startup recovery removes leftover MANIFEST.new.* file
        let config = LsmConfig {
            path: dir.path().to_path_buf(),
            ..Default::default()
        };
        let _storage = LsmStorage::new(config)
            .await
            .expect("LsmStorage startup with leftover temp manifest file");

        assert!(
            !leftover_tmp.exists(),
            "Leftover MANIFEST.new.* temp file must be cleaned up during startup recovery"
        );
    }

    #[tokio::test]
    async fn test_manifest_rollover_restart_recovery() {
        let dir = tempdir().expect("tempdir");
        let manifest_path = dir.path().join("MANIFEST");

        let live_sstables = vec![
            ManifestEntry::Add {
                path: PathBuf::from("sst-10.sst"),
                max_tx: 10,
            },
            ManifestEntry::Add {
                path: PathBuf::from("sst-20.sst"),
                max_tx: 20,
            },
        ];

        {
            let manifest = Manifest::open(&manifest_path).await.expect("open manifest");
            for entry in &live_sstables {
                manifest.append(entry).await.expect("append entry");
            }

            let rolled = manifest
                .maybe_rollover(&live_sstables, 1)
                .await
                .expect("rollover");
            assert!(rolled);
        }

        // Process restart simulation: reload manifest from disk
        let reloaded_entries = Manifest::load(&manifest_path)
            .await
            .expect("load manifest after restart");
        let valid_sstables = Manifest::reconstruct_valid_sstables(&reloaded_entries);

        assert_eq!(valid_sstables.len(), 2);
        assert!(valid_sstables.contains(Path::new("sst-10.sst")));
        assert!(valid_sstables.contains(Path::new("sst-20.sst")));
    }
}
