//! SSTable Reader: Block-Leseoperationen und Zero-Copy-Block-Dekodierung.
// ANCHOR[contextra-store:SST-RDR-110]

use super::SstableReader;
use crate::error::{StoreError as ContextraError, StoreResult as Result};
use crate::io_seam::VfsFile;
use crate::sstable::io::pread_exact;
use crate::wal::KeyManager;
use bytes::Bytes;
use std::path::Path;
use std::sync::Arc;

impl SstableReader {
    pub(crate) async fn read_block_at_file(
        file: Arc<dyn VfsFile>,
        offset: u64,
        next_offset: u64,
        key_manager: &Option<Arc<KeyManager>>,
        has_crc: bool,
        path: &Path,
    ) -> Result<Bytes> {
        if next_offset <= offset {
            return Err(ContextraError::Storage(
                "corrupted SSTable block offsets: next_offset <= offset".into(),
            ));
        }

        let len = usize::try_from(next_offset - offset).map_err(|_| {
            ContextraError::Storage("Block offset difference exceeds platform usize".into())
        })?;

        if len < 4 || len > 16 * 1024 * 1024 {
            return Err(ContextraError::Storage(
                "SSTable block size invalid or exceeds maximum allowed size".into(),
            ));
        }

        let data = tokio::task::spawn_blocking(move || -> std::io::Result<Vec<u8>> {
            let mut buf = vec![0u8; len];
            pread_exact(file.as_ref(), &mut buf, offset)?;
            Ok(buf)
        })
        .await
        .map_err(|e| ContextraError::Storage(format!("Join error: {}", e)))?
        .map_err(|e| ContextraError::Storage(format!("Block read failed: {}", e)))?;

        let block_data = if let Some(km) = key_manager {
            if data.len() < 12 {
                return Err(ContextraError::Storage("Block too small for nonce".into()));
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
                return Err(ContextraError::Storage("Block too small for CRC".into()));
            }
            let stored_crc = u32::from_le_bytes(
                block_data[0..4]
                    .try_into()
                    .map_err(|_| ContextraError::Serialization("Invalid CRC format".into()))?,
            );
            let payload = &block_data[4..];
            let computed_crc = crc32fast::hash(payload);

            if stored_crc != computed_crc {
                return Err(ContextraError::checksum_mismatch(
                    path.to_string_lossy(),
                    offset,
                ));
            }
            Ok(block_data.slice(4..))
        } else {
            Ok(block_data)
        }
    }

    pub(crate) async fn get_block(&self, offset: u64, next_offset: u64) -> Result<Bytes> {
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
}
