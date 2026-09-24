use contextra_core::{ContextraError, Result};
use std::path::{Path, PathBuf};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use super::entry::ManifestEntry;
use super::recovery::is_valid_tail_truncation_candidate;
use super::MAX_MANIFEST_ENTRY_SIZE;

/// Append-only Manifest file handle for recording active SSTable set transitions.
pub struct Manifest {
    pub(super) path: PathBuf,
    pub(super) file: tokio::sync::Mutex<tokio::fs::File>,
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
                        ContextraError::Storage(format!("Failed to open MANIFEST: {}", e))
                    })?;
                (file, false)
            }
            Err(e) => {
                return Err(ContextraError::Storage(format!(
                    "Failed to create MANIFEST: {}",
                    e
                )));
            }
        };

        if is_new {
            file.sync_all().await.map_err(|e| {
                ContextraError::Storage(format!(
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
        self.append_batch(std::slice::from_ref(entry)).await
    }

    /// Appends multiple entries to the manifest in a single atomic batch with flush + fsync.
    pub async fn append_batch(&self, entries: &[ManifestEntry]) -> Result<()> {
        if entries.is_empty() {
            return Ok(());
        }
        let mut batch_bytes = Vec::new();
        for entry in entries {
            batch_bytes.extend(entry.to_bytes()?);
        }

        let mut file = self.file.lock().await;
        file.write_all(&batch_bytes).await.map_err(|e| {
            ContextraError::Storage(format!(
                "MANIFEST write failed for {}: {}",
                self.path.display(),
                e
            ))
        })?;
        file.flush().await.map_err(|e| {
            ContextraError::Storage(format!(
                "MANIFEST flush failed for {}: {}",
                self.path.display(),
                e
            ))
        })?;
        file.sync_all().await.map_err(|e| {
            ContextraError::Storage(format!(
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
                return Err(ContextraError::Storage(format!(
                    "Failed to open MANIFEST for reading: {}",
                    e
                )))
            }
        };

        let file_size = file
            .metadata()
            .await
            .map_err(|e| ContextraError::Storage(e.to_string()))?
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
                    return Err(ContextraError::Storage(format!(
                        "MANIFEST read error at offset {}: {}",
                        pos, e
                    )));
                }
            }

            let len = u32::from_le_bytes(len_bytes) as usize;
            if len < 5 || len > MAX_MANIFEST_ENTRY_SIZE as usize {
                return Err(ContextraError::Storage(format!(
                    "MANIFEST entry length ({}) invalid or exceeds max size ({}) at offset {}",
                    len, MAX_MANIFEST_ENTRY_SIZE, pos
                )));
            }

            let entry_pos = pos;
            pos += 4 + len as u64;

            if pos <= file_size {
                let mut entry_raw = vec![0u8; len];
                if let Err(e) = reader.read_exact(&mut entry_raw).await {
                    return Err(ContextraError::Storage(format!(
                        "MANIFEST read error at offset {}: {}",
                        entry_pos, e
                    )));
                }

                match ManifestEntry::from_bytes(&entry_raw) {
                    Ok(entry) => entries.push(entry),
                    Err(e) => {
                        return Err(ContextraError::Storage(format!(
                            "MANIFEST entry corruption at offset {}: {}",
                            entry_pos, e
                        )));
                    }
                }
            } else {
                let avail = (file_size - (entry_pos + 4)) as usize;
                let mut partial_raw = vec![0u8; avail];
                if let Err(e) = reader.read_exact(&mut partial_raw).await {
                    return Err(ContextraError::Storage(format!(
                        "MANIFEST read error for partial payload at offset {}: {}",
                        entry_pos, e
                    )));
                }

                if is_valid_tail_truncation_candidate(len, &partial_raw) {
                    tracing::warn!(
                        "MANIFEST tail truncation detected (incomplete payload) at offset {}",
                        entry_pos
                    );
                    break;
                } else {
                    return Err(ContextraError::Storage(format!(
                        "MANIFEST mid-file corruption at offset {}: invalid frame length {} or corrupt frame structure (possible SSTable resurrection risk)",
                        entry_pos, len
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

    /// Reconstructs the set of SSTable file names (or path components) that are proven dead
    /// (explicitly removed via `Remove` or `Replace.removed`) and are not in the valid set.
    pub fn reconstruct_dead_sstables(
        entries: &[ManifestEntry],
    ) -> std::collections::HashSet<PathBuf> {
        let valid = Self::reconstruct_valid_sstables(entries);
        let valid_set: std::collections::HashSet<PathBuf> =
            valid.into_iter().map(|(p, _)| p).collect();
        let mut dead_set = std::collections::HashSet::new();

        for entry in entries {
            match entry {
                ManifestEntry::Remove { path } => {
                    let key = path
                        .file_name()
                        .map(PathBuf::from)
                        .unwrap_or_else(|| path.clone());
                    if !valid_set.contains(&key) {
                        dead_set.insert(key);
                    }
                }
                ManifestEntry::Replace { removed, .. } => {
                    for p in removed {
                        let key = p
                            .file_name()
                            .map(PathBuf::from)
                            .unwrap_or_else(|| p.clone());
                        if !valid_set.contains(&key) {
                            dead_set.insert(key);
                        }
                    }
                }
                ManifestEntry::Add { .. } | ManifestEntry::RollbackComplete { .. } => {}
            }
        }

        dead_set
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}
