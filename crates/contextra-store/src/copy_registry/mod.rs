// FILE-CONTEXT
// STAND: 2026-10-11 (T-2026-9103)
// ZWECK: CopyRegistry Modul-Einstieg und Implementierung der Kopienregister-Persistenz.
// INVARIANTEN: Zero-Panic, Fail-Closed, Frame-Chaining.

use contextra_types::{ContextraError, Result};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use tokio::fs::OpenOptions;
use tokio::io::AsyncWriteExt;

pub const REGISTRY_FILENAME: &str = "copies.registry";

/// Computes a deterministic 16-byte file identifier for a file name using BLAKE3 derive_key.
pub fn file_id_for_name(name: &str) -> [u8; 16] {
    let hash = blake3::Hasher::new_derive_key("contextra/copy-file-id/v1")
        .update(name.as_bytes())
        .finalize();
    let mut file_id = [0u8; 16];
    file_id.copy_from_slice(&hash.as_bytes()[..16]);
    file_id
}

pub use contextra_core::copy_registry::{CopyKind, CopyRegEntry};

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

        let bytes = tokio::fs::read(&path)
            .await
            .map_err(|e| ContextraError::Storage(e.to_string()))?;
        let mut offset = 0;
        let mut entries = Vec::new();
        let mut seen_copy_ids = HashSet::new();

        while offset < bytes.len() {
            if bytes.len() < offset + 4 {
                break; // Tail truncation
            }
            let frame_len =
                u32::from_be_bytes(bytes[offset..offset + 4].try_into().unwrap_or([0; 4])) as usize;
            if frame_len > 65536 || bytes.len() < offset + 4 + frame_len + 4 {
                break; // Tail truncation
            }

            let payload = &bytes[offset + 4..offset + 4 + frame_len];
            let crc_stored = u32::from_be_bytes(
                bytes[offset + 4 + frame_len..offset + 4 + frame_len + 4]
                    .try_into()
                    .unwrap_or([0; 4]),
            );
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

        file.write_all(&frame)
            .await
            .map_err(|e| ContextraError::Storage(e.to_string()))?;
        file.sync_all()
            .await
            .map_err(|e| ContextraError::Storage(e.to_string()))?;
        crate::util::fsync_parent_dir(&path).await?;

        self.seen_copy_ids.insert(entry.copy_id);
        self.entries.push(entry);

        Ok(())
    }

    pub fn list(&self) -> &[CopyRegEntry] {
        &self.entries
    }

    /// Returns a list of all currently registered copies (read-only observability for engine exclusion sets).
    pub fn registered_copies(&self) -> Vec<CopyRegEntry> {
        self.entries.clone()
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

    /// Returns copy_ids of all registered copies holding any of the specified scope file_ids.
    pub fn holds_scope_files(&self, live_file_ids: &[[u8; 16]]) -> Vec<[u8; 16]> {
        self.holds_files(live_file_ids)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_id_for_name_bit_exactness() {
        let name1 = "000001.sst";
        let name2 = "manifest.json";

        let id1 = file_id_for_name(name1);
        let id2 = file_id_for_name(name2);

        let expected1_hash = blake3::Hasher::new_derive_key("contextra/copy-file-id/v1")
            .update(name1.as_bytes())
            .finalize();
        let mut expected1 = [0u8; 16];
        expected1.copy_from_slice(&expected1_hash.as_bytes()[..16]);

        assert_eq!(id1, expected1);
        assert_ne!(id1, [0u8; 16]);
        assert_ne!(id2, [0u8; 16]);
        assert_ne!(id1, id2);
    }
}
