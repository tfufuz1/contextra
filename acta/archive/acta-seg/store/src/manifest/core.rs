//! # Manifest-Kernlogik (Ring 1)
//!
//! Serialisiert und deserialisiert Manifest-Eintraege fuer Dateiaustausch und Compaction.
//! Verwaltet Ladevorgänge, Tail-Truncation Recovery und die Re-Exporte der Teilmodule.

use crate::error::{StoreError, StoreResult};
use std::path::Path;
use tokio::io::{AsyncReadExt, AsyncSeekExt};

use super::entry::ManifestEntry;
use super::recovery::is_valid_tail_truncation_candidate;
use super::MAX_MANIFEST_ENTRY_SIZE;

pub const MANIFEST_HEADER_MAGIC: &[u8; 4] = b"MFMN";
pub const CURRENT_MANIFEST_VERSION: u8 = 2;
#[allow(dead_code)]
pub const MANIFEST_VERSION_V1: u8 = 1;
#[allow(dead_code)]
pub const MANIFEST_VERSION_V2: u8 = 2;

// Submodule für fokussierte Verantwortlichkeiten
// MIGRATION(S-70): Zerlegt in handle, apply, cleanup
pub use super::handle::Manifest;

impl Manifest {
    /// Loads all valid manifest entries from `path`.
    pub async fn load(path: &Path) -> StoreResult<Vec<ManifestEntry>> {
        if !tokio::fs::try_exists(path).await.unwrap_or(false) {
            return Ok(Vec::new());
        }

        let mut file = match tokio::fs::File::open(path).await {
            Ok(f) => f,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(e) => {
                return Err(StoreError::Storage(format!(
                    "Failed to open MANIFEST for reading: {}",
                    e
                )))
            }
        };

        let file_size = file
            .metadata()
            .await
            .map_err(|e| StoreError::Storage(e.to_string()))?
            .len();

        let mut reader = tokio::io::BufReader::new(&mut file);
        let mut entries = Vec::new();
        let mut pos = 0u64;

        if file_size >= 5 {
            let mut header = [0u8; 5];
            if reader.read_exact(&mut header).await.is_ok() {
                if &header[0..4] == MANIFEST_HEADER_MAGIC {
                    let version = header[4];
                    if version > CURRENT_MANIFEST_VERSION {
                        return Err(StoreError::Storage(format!(
                            "Unsupported MANIFEST format version: {}",
                            version
                        )));
                    }
                    pos = 5;
                } else {
                    // Legacy version 0 manifest without header magic
                    reader
                        .seek(tokio::io::SeekFrom::Start(0))
                        .await
                        .map_err(|e| {
                            StoreError::Storage(format!("Failed to seek MANIFEST: {}", e))
                        })?;
                }
            }
        }

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
                    return Err(StoreError::Storage(format!(
                        "MANIFEST read error at offset {}: {}",
                        pos, e
                    )));
                }
            }

            let len = u32::from_le_bytes(len_bytes) as usize;
            if len < 5 || len > MAX_MANIFEST_ENTRY_SIZE as usize {
                return Err(StoreError::Storage(format!(
                    "MANIFEST entry length ({}) invalid or exceeds max size ({}) at offset {}",
                    len, MAX_MANIFEST_ENTRY_SIZE, pos
                )));
            }

            let entry_pos = pos;
            pos += 4 + len as u64;

            if pos <= file_size {
                let mut entry_raw = vec![0u8; len];
                if let Err(e) = reader.read_exact(&mut entry_raw).await {
                    return Err(StoreError::Storage(format!(
                        "MANIFEST read error at offset {}: {}",
                        entry_pos, e
                    )));
                }

                match ManifestEntry::from_bytes(&entry_raw) {
                    Ok(entry) => entries.push(entry),
                    Err(e) => {
                        return Err(StoreError::Storage(format!(
                            "MANIFEST entry corruption at offset {}: {}",
                            entry_pos, e
                        )));
                    }
                }
            } else {
                let avail = (file_size - (entry_pos + 4)) as usize;
                let mut partial_raw = vec![0u8; avail];
                if let Err(e) = reader.read_exact(&mut partial_raw).await {
                    return Err(StoreError::Storage(format!(
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
                    return Err(StoreError::Storage(format!(
                        "MANIFEST mid-file corruption at offset {}: invalid frame length {} or corrupt frame structure (possible SSTable resurrection risk)",
                        entry_pos, len
                    )));
                }
            }
        }

        Ok(entries)
    }
}
