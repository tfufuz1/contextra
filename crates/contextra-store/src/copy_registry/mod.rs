// FILE-CONTEXT
// STAND: 2026-10-11 (T-2026-9103)
// ZWECK: CopyRegistry Modul-Einstieg und Implementierung der Kopienregister-Persistenz.
// INVARIANTEN: Zero-Panic, Fail-Closed, Frame-Chaining.

use contextra_core::{ContextraError, Result};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use tokio::fs::OpenOptions;
use tokio::io::AsyncWriteExt;

pub const REGISTRY_FILENAME: &str = "copies.registry";

/// Kind of copy recorded in the registry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CopyKind {
    Hardlink = 1,
    Reflink = 2,
    FullCopy = 3,
}

/// An entry in the copy registry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CopyRegEntry {
    pub copy_id: [u8; 16],
    pub kind: CopyKind,
    pub file_ids: Vec<[u8; 16]>,
    pub created_tx: u64,
}

impl CopyRegEntry {
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        let mut buf = Vec::new();
        buf.extend_from_slice(&self.copy_id);
        let kind_u8 = match self.kind {
            CopyKind::Hardlink => 1u8,
            CopyKind::Reflink => 2u8,
            CopyKind::FullCopy => 3u8,
        };
        buf.push(kind_u8);
        #[allow(clippy::cast_possible_truncation)]
        buf.extend_from_slice(&(self.file_ids.len() as u32).to_be_bytes());
        for fid in &self.file_ids {
            buf.extend_from_slice(fid);
        }
        buf.extend_from_slice(&self.created_tx.to_be_bytes());
        Ok(buf)
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < 16 + 1 + 4 + 8 {
            return Err(ContextraError::Serialization("CopyRegEntry too short".into()));
        }

        let mut copy_id = [0u8; 16];
        copy_id.copy_from_slice(&bytes[0..16]);

        let kind = match bytes[16] {
            1 => CopyKind::Hardlink,
            2 => CopyKind::Reflink,
            3 => CopyKind::FullCopy,
            k => return Err(ContextraError::Serialization(format!("Unknown CopyKind {}", k))),
        };

        let file_count = u32::from_be_bytes(bytes[17..21].try_into().unwrap_or([0; 4])) as usize;
        let expected_len = 16 + 1 + 4 + (file_count * 16) + 8;
        if bytes.len() < expected_len {
            return Err(ContextraError::Serialization("CopyRegEntry truncated file_ids".into()));
        }

        let mut offset = 21;
        let mut file_ids = Vec::with_capacity(file_count);
        for _ in 0..file_count {
            let mut fid = [0u8; 16];
            fid.copy_from_slice(&bytes[offset..offset + 16]);
            file_ids.push(fid);
            offset += 16;
        }

        let created_tx = u64::from_be_bytes(bytes[offset..offset + 8].try_into().unwrap_or([0; 8]));

        Ok(Self {
            copy_id,
            kind,
            file_ids,
            created_tx,
        })
    }
}

/// Persistent registry tracking physical/virtual snapshot file copies.
pub struct CopyRegistry {
    dir: PathBuf,
    entries: Vec<CopyRegEntry>,
    seen_copy_ids: HashSet<[u8; 16]>,
}

impl std::fmt::Debug for CopyRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CopyRegistry")
            .field("dir", &self.dir)
            .field("entries_count", &self.entries.len())
            .finish()
    }
}

impl CopyRegistry {
    pub async fn open(dir: impl AsRef<Path>) -> Result<Self> {
        let dir = dir.as_ref().to_path_buf();
        let path = dir.join(REGISTRY_FILENAME);

        if !tokio::fs::try_exists(&path).await.unwrap_or(false) {
            return Ok(Self {
                dir,
                entries: Vec::new(),
                seen_copy_ids: HashSet::new(),
            });
        }

        let bytes = tokio::fs::read(&path).await.map_err(|e| ContextraError::Storage(e.to_string()))?;
        let mut offset = 0;
        let mut entries = Vec::new();
        let mut seen_copy_ids = HashSet::new();

        while offset < bytes.len() {
            if bytes.len() < offset + 4 {
                break; // Tail truncation
            }
            let frame_len = u32::from_be_bytes(bytes[offset..offset + 4].try_into().unwrap_or([0; 4])) as usize;
            if frame_len > 65536 || bytes.len() < offset + 4 + frame_len + 4 {
                break; // Tail truncation
            }

            let payload = &bytes[offset + 4..offset + 4 + frame_len];
            let crc_stored = u32::from_be_bytes(bytes[offset + 4 + frame_len..offset + 4 + frame_len + 4].try_into().unwrap_or([0; 4]));
            let crc_computed = crc32fast::hash(payload);

            if crc_stored != crc_computed {
                return Err(ContextraError::Storage("CopyRegistry CRC mismatch".into()));
            }

            let entry = CopyRegEntry::from_bytes(payload)?;
            if !seen_copy_ids.contains(&entry.copy_id) {
                seen_copy_ids.insert(entry.copy_id);
                entries.push(entry);
            }

            offset += 4 + frame_len + 4;
        }

        Ok(Self {
            dir,
            entries,
            seen_copy_ids,
        })
    }

    pub async fn register(&mut self, entry: CopyRegEntry) -> Result<()> {
        if self.seen_copy_ids.contains(&entry.copy_id) {
            return Ok(()); // Idempotent
        }

        let path = self.dir.join(REGISTRY_FILENAME);
        let payload = entry.to_bytes()?;
        #[allow(clippy::cast_possible_truncation)]
        let frame_len = payload.len() as u32;
        let crc = crc32fast::hash(&payload);

        let mut frame = Vec::with_capacity(4 + payload.len() + 4);
        frame.extend_from_slice(&frame_len.to_be_bytes());
        frame.extend_from_slice(&payload);
        frame.extend_from_slice(&crc.to_be_bytes());

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .await
            .map_err(|e| ContextraError::Storage(e.to_string()))?;

        file.write_all(&frame).await.map_err(|e| ContextraError::Storage(e.to_string()))?;
        file.sync_all().await.map_err(|e| ContextraError::Storage(e.to_string()))?;
        crate::util::fsync_parent_dir(&path).await?;

        self.seen_copy_ids.insert(entry.copy_id);
        self.entries.push(entry);

        Ok(())
    }

    pub fn list(&self) -> &[CopyRegEntry] {
        &self.entries
    }

    pub fn holds_files(&self, file_ids: &[[u8; 16]]) -> Vec<[u8; 16]> {
        let mut matched = Vec::new();
        let search_set: HashSet<&[u8; 16]> = file_ids.iter().collect();

        for entry in &self.entries {
            for fid in &entry.file_ids {
                if search_set.contains(fid) {
                    matched.push(entry.copy_id);
                    break;
                }
            }
        }

        matched
    }

    pub async fn remove(&mut self, copy_id: &[u8; 16]) -> Result<()> {
        self.entries.retain(|e| &e.copy_id != copy_id);
        self.seen_copy_ids.remove(copy_id);

        let path = self.dir.join(REGISTRY_FILENAME);
        let mut buffer = Vec::new();

        for entry in &self.entries {
            let payload = entry.to_bytes()?;
            #[allow(clippy::cast_possible_truncation)]
            let frame_len = payload.len() as u32;
            let crc = crc32fast::hash(&payload);

            buffer.extend_from_slice(&frame_len.to_be_bytes());
            buffer.extend_from_slice(&payload);
            buffer.extend_from_slice(&crc.to_be_bytes());
        }

        contextra_durable_fs::atomic_replace(&path, &buffer)?;
        Ok(())
    }
}
