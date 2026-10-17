use contextra_types::{ContextraError, Result};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use super::pointer::BlobPointer;
use super::writer::BLOB_HEADER_SIZE;
use crate::format::{check_openable, FileKind, FormatHeader};
use crate::sstable::io::pread_exact;

/// Reader for `MFBL` blob segment files.
pub struct BlobReader {
    file: Arc<std::fs::File>,
    path: PathBuf,
}

impl BlobReader {
    pub async fn open(path: impl AsRef<Path>) -> Result<Self> {
        let path_buf = path.as_ref().to_path_buf();
        let path_for_open = path_buf.clone();

        let (file, file_size) =
            tokio::task::spawn_blocking(move || -> std::io::Result<(std::fs::File, u64)> {
                let file = std::fs::File::open(&path_for_open)?;
                let meta = file.metadata()?;
                let len = meta.len();
                Ok((file, len))
            })
            .await
            .map_err(|e| ContextraError::Storage(format!("Join error opening blob: {}", e)))?
            .map_err(|e| {
                ContextraError::Storage(format!(
                    "Failed to open blob file {}: {}",
                    path_buf.display(),
                    e
                ))
            })?;

        if file_size < BLOB_HEADER_SIZE {
            return Err(ContextraError::Storage(format!(
                "Blob file {} too small ({} bytes)",
                path_buf.display(),
                file_size
            )));
        }

        let file = Arc::new(file);

        // Read FormatHeader (6 bytes)
        let header_data = {
            let f = Arc::clone(&file);
            tokio::task::spawn_blocking(move || -> std::io::Result<[u8; 6]> {
                let mut buf = [0u8; 6];
                pread_exact(&f, &mut buf, 0)?;
                Ok(buf)
            })
            .await
            .map_err(|e| ContextraError::Storage(format!("Join error: {}", e)))?
            .map_err(|e| ContextraError::Storage(format!("Failed to read blob header: {}", e)))?
        };

        let header = FormatHeader::parse(&header_data)?;
        check_openable(FileKind::Blob, &header, 1)?;

        if let Some(parent) = path_buf.parent() {
            let manifest_path = parent.join("MANIFEST");
            if manifest_path.exists() {
                if let Ok(entries) = crate::manifest::Manifest::load(&manifest_path).await {
                    let tracker = crate::manifest::ManifestFormatTracker::from_entries(&entries);
                    tracker.verify_file_version(FileKind::Blob, header.version)?;
                }
            }
        }

        Ok(Self {
            file,
            path: path_buf,
        })
    }

    /// Reads payload at `pointer.offset` using `pread` with BLAKE3 verification.
    pub async fn read(&self, pointer: &BlobPointer) -> Result<Vec<u8>> {
        let read_len = usize::try_from(pointer.len)
            .map_err(|_| ContextraError::Storage("Blob pointer len exceeds platform usize".into()))?
            .checked_add(36)
            .ok_or_else(|| ContextraError::Storage("Blob record len overflow".into()))?;

        let file = Arc::clone(&self.file);
        let offset = pointer.offset;
        let path_str = self.path.to_string_lossy().to_string();

        let record_bytes = tokio::task::spawn_blocking(move || -> std::io::Result<Vec<u8>> {
            let mut buf = vec![0u8; read_len];
            pread_exact(&file, &mut buf, offset)?;
            Ok(buf)
        })
        .await
        .map_err(|e| ContextraError::Storage(format!("Join error: {}", e)))?
        .map_err(|e| {
            ContextraError::Storage(format!("Blob read failed at offset {}: {}", offset, e))
        })?;

        if record_bytes.len() < 36 {
            return Err(ContextraError::Storage("Blob record too short".into()));
        }

        let len_bytes: [u8; 4] = record_bytes[0..4]
            .try_into()
            .map_err(|_| ContextraError::Storage("Invalid len slice".into()))?;
        let record_payload_len = u32::from_le_bytes(len_bytes);

        if record_payload_len != pointer.len {
            return Err(ContextraError::Storage(format!(
                "Blob payload length mismatch: expected {}, got {}",
                pointer.len, record_payload_len
            )));
        }

        let stored_digest: [u8; 32] = record_bytes[4..36]
            .try_into()
            .map_err(|_| ContextraError::Storage("Invalid blake3 slice".into()))?;

        let payload = &record_bytes[36..];
        let computed_digest = *blake3::hash(payload).as_bytes();

        if stored_digest != pointer.blake3 || computed_digest != pointer.blake3 {
            return Err(ContextraError::checksum_mismatch(path_str, offset));
        }

        Ok(payload.to_vec())
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}
