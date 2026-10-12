use contextra_core::{ContextraError, Result};
use std::path::{Path, PathBuf};
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

use super::pointer::BlobPointer;
use crate::format::FormatHeader;

pub const BLOB_HEADER_SIZE: u64 = 38; // 6 bytes FormatHeader + 32 bytes key_id

/// Append-only writer for `MFBL` blob segment files.
pub struct BlobWriter {
    path: PathBuf,
    file: File,
    segment_id: u64,
    offset: u64,
}

impl BlobWriter {
    pub async fn create(path: impl AsRef<Path>, segment_id: u64, key_id: [u8; 32]) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        let mut file = File::create(&path).await.map_err(|e| {
            ContextraError::Storage(format!(
                "Failed to create blob segment file {}: {}",
                path.display(),
                e
            ))
        })?;

        let header = FormatHeader::new(*b"MFBL", 1);
        let header_bytes = header.to_bytes();
        file.write_all(&header_bytes)
            .await
            .map_err(|e| ContextraError::Storage(format!("Failed to write blob header: {}", e)))?;
        file.write_all(&key_id)
            .await
            .map_err(|e| ContextraError::Storage(format!("Failed to write blob key_id: {}", e)))?;

        file.sync_all()
            .await
            .map_err(|e| ContextraError::Storage(format!("fsync failed on blob header: {}", e)))?;
        crate::util::fsync_parent_dir(&path).await?;

        Ok(Self {
            path,
            file,
            segment_id,
            offset: BLOB_HEADER_SIZE,
        })
    }

    /// Appends a raw byte payload to the segment and returns a `BlobPointer`.
    pub async fn append(&mut self, payload: &[u8]) -> Result<BlobPointer> {
        let len = u32::try_from(payload.len()).map_err(|_| {
            ContextraError::InvalidInput("Blob payload exceeds u32::MAX bytes".into())
        })?;

        let hash = blake3::hash(payload);
        let digest = *hash.as_bytes();

        let mut record = Vec::with_capacity(36 + payload.len());
        record.extend_from_slice(&len.to_le_bytes());
        record.extend_from_slice(&digest);
        record.extend_from_slice(payload);

        let current_offset = self.offset;
        self.file.write_all(&record).await.map_err(|e| {
            ContextraError::Storage(format!(
                "Blob write failed for {}: {}",
                self.path.display(),
                e
            ))
        })?;

        let record_len = u64::try_from(record.len())
            .map_err(|_| ContextraError::Storage("Record length overflow".into()))?;

        self.offset = self
            .offset
            .checked_add(record_len)
            .ok_or_else(|| ContextraError::Storage("Blob file offset overflow".into()))?;

        Ok(BlobPointer {
            segment_id: self.segment_id,
            offset: current_offset,
            len,
            blake3: digest,
        })
    }

    /// Flushes and fsyncs the blob segment file to durable storage.
    pub async fn flush_durable(&mut self) -> Result<()> {
        self.file.flush().await.map_err(|e| {
            ContextraError::Storage(format!(
                "Blob flush failed for {}: {}",
                self.path.display(),
                e
            ))
        })?;
        self.file.sync_all().await.map_err(|e| {
            ContextraError::Storage(format!(
                "Blob fsync failed for {}: {}",
                self.path.display(),
                e
            ))
        })?;
        Ok(())
    }

    pub fn offset(&self) -> u64 {
        self.offset
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}
