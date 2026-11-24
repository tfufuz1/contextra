// FILE-CONTEXT
// ZWECK: Pending-WAL für inkrementelle Vektor-Inserts in DiskANN (WP-VEC-07).
// INVARIANTEN: Obergrenzenprüfung MAX_PENDING_WAL_DIM vor Vektor-Allokation; HMAC-Schutz.
// STAND: 2026-11-11

use std::io::Read;
use subtle::ConstantTimeEq;

use super::super::format::{
    DISKANN_INTEGRITY_KEY, MAX_PENDING_WAL_DIM, PENDING_WAL_HEADER_SIZE, PENDING_WAL_MAGIC,
    PENDING_WAL_VERSION,
};
use super::super::types::DiskAnnIndex;
use crate::error::{Result, VectorError};
use crate::types::DocId;

impl DiskAnnIndex {
    pub(crate) fn append_to_pending_wal_blocking(
        path: &std::path::Path,
        id: DocId,
        embedding: &[f32],
    ) -> Result<()> {
        use std::io::Write;

        let dim = embedding.len() as u32;
        let id_bytes = id.inner().to_le_bytes();
        let dim_bytes = dim.to_le_bytes();

        let mut entry_bytes =
            Vec::with_capacity(std::mem::size_of::<DocId>() + 4 + (embedding.len() * 4));
        entry_bytes.extend_from_slice(&id_bytes);
        entry_bytes.extend_from_slice(&dim_bytes);
        for &val in embedding {
            entry_bytes.extend_from_slice(&val.to_le_bytes());
        }

        let mut hmac = crate::diskann::integrity::DiskAnnHmac::new(DISKANN_INTEGRITY_KEY)?;
        hmac.update(&entry_bytes);
        let computed_hmac = hmac.finalize();

        crate::diskann::io::append_wal(
            path,
            PENDING_WAL_MAGIC,
            PENDING_WAL_VERSION,
            &entry_bytes,
            &computed_hmac,
        )
    }

    /// Appends a vector insertion record to `pending.wal` with HMAC-SHA256 integrity protection.
    ///
    /// # Binary Format
    /// - File Header (written on new/empty file creation): `[PWAL: 4 bytes] [version=1: 1 byte]`
    /// - Entry Layout:
    ///   - `id`: `u64` LE (8 bytes)
    ///   - `dim`: `u32` LE (4 bytes)
    ///   - `embedding`: `[f32 LE; dim]` (`dim * 4` bytes)
    ///   - `hmac`: `[u8; 32]` (32-byte HMAC-SHA256 computed over `id_bytes || dim_bytes || embedding_bytes`)
    pub(crate) async fn append_to_pending_wal(
        path: &std::path::Path,
        id: DocId,
        embedding: &[f32],
    ) -> Result<()> {
        Self::append_to_pending_wal_blocking(path, id, embedding)
    }

    /// Reads and verifies uncommitted vector entries from `pending.wal`.
    ///
    /// # Integrity & Defensive Parsing
    /// 1. Validates the 5-byte file header (`PWAL` magic + version 1). Rejects unversioned/legacy formats.
    /// 2. Validates `dim` against `MAX_PENDING_WAL_DIM` (65,536) prior to buffer allocation to prevent allocation-DoS attacks.
    /// 3. Reads entry payload and entry HMAC, computing the expected HMAC-SHA256 over `(id || dim || embedding)`.
    /// 4. Compares HMACs using constant-time comparison (`subtle::ConstantTimeEq`). Corrupted entries are logged via `tracing::error!` with byte offset and skipped, continuing to read subsequent valid entries.
    pub(crate) fn read_pending_wal(path: &std::path::Path) -> Result<Vec<(DocId, Vec<f32>)>> {
        use std::io::Read;
        use subtle::ConstantTimeEq;

        if !path.exists() {
            return Ok(Vec::new());
        }
        let mut file = match crate::diskann::io::open_read(path) {
            Ok(f) => f,
            Err(_) => return Ok(Vec::new()),
        };

        let file_len = match file.metadata() {
            Ok(m) => m.len() as usize,
            Err(_) => return Ok(Vec::new()),
        };

        if file_len == 0 {
            return Ok(Vec::new());
        }

        if file_len < PENDING_WAL_HEADER_SIZE {
            return Err(VectorError::Storage(
                "Unbekanntes oder veraltetes pending.wal-Format, bitte vor Upgrade leeren".into(),
            ));
        }

        let mut magic_buf = [0u8; 4];
        let mut version_buf = [0u8; 1];
        if file.read_exact(&mut magic_buf).is_err() || file.read_exact(&mut version_buf).is_err() {
            return Err(VectorError::Storage(
                "Unbekanntes oder veraltetes pending.wal-Format, bitte vor Upgrade leeren".into(),
            ));
        }

        if magic_buf != *PENDING_WAL_MAGIC || version_buf[0] != PENDING_WAL_VERSION {
            return Err(VectorError::Storage(
                "Unbekanntes oder veraltetes pending.wal-Format, bitte vor Upgrade leeren".into(),
            ));
        }

        let mut recovered = Vec::new();
        let mut offset = PENDING_WAL_HEADER_SIZE;
        let doc_id_size = std::mem::size_of::<DocId>();
        let mut buf_id = vec![0u8; doc_id_size];
        let mut buf_dim = [0u8; 4];

        while file.read_exact(&mut buf_id).is_ok() {
            let entry_offset = offset;
            offset += doc_id_size;

            if file.read_exact(&mut buf_dim).is_err() {
                tracing::warn!("Truncated dim in pending.wal");
                break;
            }
            offset += 4;

            #[cfg(not(feature = "docid-128"))]
            let doc_id = DocId::from(u64::from_le_bytes(
                buf_id
                    .as_slice()
                    .try_into()
                    .map_err(|_| VectorError::Storage("Corrupt DocId in WAL".into()))?,
            ));
            #[cfg(feature = "docid-128")]
            let doc_id = DocId::from(u128::from_le_bytes(
                buf_id
                    .as_slice()
                    .try_into()
                    .map_err(|_| VectorError::Storage("Corrupt DocId in WAL".into()))?,
            ));
            let dim = u32::from_le_bytes(buf_dim) as usize;

            if dim == 0 || dim > MAX_PENDING_WAL_DIM {
                tracing::warn!(
                    dim,
                    offset = entry_offset,
                    "pending.wal: Eintrag mit unplausibler Dimension verworfen"
                );
                break;
            }

            let vec_byte_len = dim * 4;
            let mut vec_bytes = vec![0u8; vec_byte_len];
            if file.read_exact(&mut vec_bytes).is_err() {
                tracing::warn!("Truncated vector payload in pending.wal");
                break;
            }
            offset += vec_byte_len;

            let mut buf_hmac = [0u8; 32];
            if file.read_exact(&mut buf_hmac).is_err() {
                tracing::warn!("Truncated HMAC in pending.wal");
                break;
            }
            offset += 32;

            let mut hmac = crate::diskann::integrity::DiskAnnHmac::new(DISKANN_INTEGRITY_KEY)?;
            hmac.update(&buf_id);
            hmac.update(&buf_dim);
            hmac.update(&vec_bytes);
            let computed_hmac = hmac.finalize();

            if computed_hmac.ct_eq(&buf_hmac).into() {
                let mut vec = Vec::with_capacity(dim);
                let (chunks, _) = vec_bytes.as_chunks::<4>();
                for &b in chunks {
                    vec.push(f32::from_le_bytes(b));
                }
                recovered.push((doc_id, vec));
            } else {
                tracing::error!(
                    offset = entry_offset,
                    doc_id = %doc_id.inner(),
                    "pending.wal: HMAC-Mismatch bei Eintrag, Eintrag verworfen"
                );
            }
        }
        Ok(recovered)
    }

    /// Liest uncommittete Pending-Vektoren aus `pending.wal` und fügt sie in den Index ein.
    pub fn recover_pending_delta(
        &self,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<usize>> + Send + '_>> {
        Box::pin(async move { self.recover_pending_delta_sync() })
    }

    pub fn recover_pending_delta_sync(&self) -> Result<usize> {
        let pending_wal = self.inner.config.index_path.with_extension("pending.wal");
        if !pending_wal.exists() {
            return Ok(0);
        }
        let recovered = Self::read_pending_wal(&pending_wal)?;
        let count = recovered.len();
        if count > 0 {
            {
                let mut guard = self.inner.pending_inserts.write();
                self.inner
                    .pending_count
                    .store(count as u64, Ordering::Relaxed);
                *guard = recovered;
            }
            self.persist_delta_sync()?;
        } else {
            let _ = crate::diskann::io::remove_if_exists(&pending_wal);
        }
        Ok(count)
    }
}
