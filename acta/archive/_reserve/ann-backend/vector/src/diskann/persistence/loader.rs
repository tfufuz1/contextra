// FILE-CONTEXT
// ZWECK: Laden des DiskANN-Indexes aus Speicherabbildung (WP-VEC-07).
// INVARIANTEN: Zero-Panic bei korruptem Header/Footer; Bounds-Prüfungen vor Slice-Zugriff.
// STAND: 2026-11-11

use std::sync::Arc;
use subtle::ConstantTimeEq;

use super::super::config::DiskAnnFallbackPolicy;
use super::super::format::{
    DiskAnnFooter, DiskAnnHeader, DISKANN_FOOTER_MAGIC, DISKANN_INTEGRITY_KEY, DISKANN_MAGIC,
    DISKANN_VERSION,
};
use super::super::integrity::DiskAnnHmac;
use super::super::types::DiskAnnIndex;
use crate::error::{Result, VectorError};
use crate::types::{DistanceMetric, DocId};

impl DiskAnnIndex {
    pub(crate) fn load_all_vectors_from_mmap(&self) -> Result<(Vec<Vec<f32>>, Vec<DocId>)> {
        let disk_count = {
            let guard = self.inner.header.read();
            guard.as_ref().map(|h| h.node_count as usize).unwrap_or(0)
        };
        let mut all_vecs = Vec::with_capacity(disk_count);
        let mut all_ids = Vec::with_capacity(disk_count);

        let tombstones = self.inner.tombstones.read();

        for i in 0..disk_count as u32 {
            let node = self.load_node(i)?;
            if tombstones.contains(node.doc_id.inner()) {
                continue;
            }
            let vec_f32 = match node.vector {
                VectorData::F32(v) => v,
                VectorData::U8(v) => {
                    let q_guard = self.inner.quantizer.read();
                    let q = q_guard
                        .as_ref()
                        .ok_or_else(|| VectorError::Index("Quantizer missing".into()))?;
                    q.dequantize(&v)?
                }
            };
            all_vecs.push(vec_f32);
            all_ids.push(node.doc_id);
        }
        Ok((all_vecs, all_ids))
    }

    pub async fn load(&self) -> Result<()> {
        self.load_sync()
    }

    pub fn load_sync(&self) -> Result<()> {
        let index_exists = self.inner.config.index_path.exists();
        let pending_wal = self.inner.config.index_path.with_extension("pending.wal");
        let wal_exists = pending_wal.exists();

        if !index_exists && !wal_exists {
            return Err(VectorError::Io(
                format!(
                    "DiskANN index file does not exist: {}",
                    self.inner.config.index_path.display()
                ),
            ));
        }

        if index_exists {
            let inner = Arc::clone(&self.inner);
            let load_closure = move || {
                use std::sync::atomic::Ordering;

                // Clean up any orphaned temporary files from interrupted persist_delta or build calls
                let delta_tmp = inner.config.index_path.with_extension("delta.tmp");
                if delta_tmp.exists() {
                    if let Err(e) = crate::diskann::io::remove_if_exists(&delta_tmp) {
                        tracing::warn!(
                            "Failed to remove orphaned delta.tmp file {}: {e}",
                            delta_tmp.display()
                        );
                    }
                }
                let idx_tmp = inner.config.index_path.with_extension("idx.tmp");
                if idx_tmp.exists() {
                    if let Err(e) = crate::diskann::io::remove_if_exists(&idx_tmp) {
                        tracing::warn!(
                            "Failed to remove orphaned idx.tmp file {}: {e}",
                            idx_tmp.display()
                        );
                    }
                }

                let file = crate::diskann::io::open_read(&inner.config.index_path)?;
                let mmap = acta_sys::mmap_readonly(&file).map_err(VectorError::from)?;

                if mmap.len() < DiskAnnHeader::SIZE + DiskAnnFooter::SIZE {
                    return Err(VectorError::Storage(
                        "DiskANN file too small for header and footer".into(),
                    ));
                }

                let header_slice = mmap.get(0..DiskAnnHeader::SIZE).ok_or_else(|| {
                    VectorError::Storage("DiskANN file too small for header".into())
                })?;
                let header = DiskAnnHeader::try_from_bytes(header_slice)?;

                // Verify HMAC integrity footer
                let footer_slice =
                    mmap.get(mmap.len() - DiskAnnFooter::SIZE..)
                        .ok_or_else(|| {
                            VectorError::Storage("DiskANN file too small for footer".into())
                        })?;
                let footer = DiskAnnFooter::try_from_bytes(footer_slice)?;

                let payload = mmap
                    .get(..mmap.len() - DiskAnnFooter::SIZE)
                    .ok_or_else(|| {
                        VectorError::Storage("DiskANN payload slice out of bounds".into())
                    })?;

                let mut hmac = crate::diskann::integrity::DiskAnnHmac::new(DISKANN_INTEGRITY_KEY)?;
                hmac.update(payload);
                let computed_hmac = hmac.finalize();

                if footer.hmac != computed_hmac {
                    return Err(VectorError::Storage(
                        "DiskANN index file HMAC integrity validation failed: checksum mismatch"
                            .into(),
                    ));
                }

                if inner.config.sector_size != header.sector_size as usize {
                    return Err(VectorError::Index(format!(
                        "DiskANN-Index inkompatibel: Config-sector_size={} stimmt nicht mit \
                         Header-sector_size={} überein. Index muss neu aufgebaut werden.",
                        inner.config.sector_size, header.sector_size
                    ));
                }

                if header.quantized != 0 {
                    let dim = header.dimension as usize;
                    let range = (header.q_max - header.q_min).max(1e-6);
                    *inner.quantizer.write() = Some(crate::quantize::ScalarQuantizer {
                        mins: vec![header.q_min; dim],
                        maxes: vec![header.q_max; dim],
                        scales: vec![255.0 / range; dim],
                        inv_scales: vec![range / 255.0; dim],
                        dimension: dim,
                        total_queries: std::sync::atomic::AtomicU64::new(0),
                        out_of_range_queries: std::sync::atomic::AtomicU64::new(0),
                    });
                }

                let vector_size = if header.quantized != 0 {
                    header.dimension as usize
                } else {
                    header.dimension as usize * 4
                };
                let neighbors_size = 4 + (header.max_degree as usize * 4);
                let doc_id_size = std::mem::size_of::<DocId>();
                let raw_node_size = vector_size + neighbors_size + doc_id_size;
                let node_size_bytes = raw_node_size.div_ceil(header.sector_size as usize)
                    * header.sector_size as usize;
                inner
                    .node_size_bytes
                    .store(node_size_bytes as u64, Ordering::SeqCst);

                let file_len = mmap.len();
                let sector_size = header.sector_size as usize;
                let start_offset = DiskAnnHeader::SIZE.div_ceil(sector_size) * sector_size;
                let expected_min_size = start_offset
                    .saturating_add((header.node_count as usize).saturating_mul(node_size_bytes));
                if file_len < expected_min_size {
                    return Err(VectorError::Storage(format!(
                        "DiskANN file truncated or corrupt node_count: file len {}, expected at least {}",
                        file_len, expected_min_size
                    )));
                }
                *inner.header.write() = Some(header);
                *inner.mmap.write() = Some(mmap);
                inner.cache.write().clear();

                let mut ids = Vec::with_capacity(header.node_count as usize);
                let read_size = node_size_bytes;
                if !read_size.is_multiple_of(sector_size) {
                    return Err(VectorError::Index(
                        "Read size must be a multiple of sector_size".into(),
                    ));
                }

                for i in 0..header.node_count as u32 {
                    let index_offset =
                        (i as usize).checked_mul(node_size_bytes).ok_or_else(|| {
                            VectorError::Index("Node offset multiplication overflow".into())
                        })?;
                    let offset = start_offset.checked_add(index_offset).ok_or_else(|| {
                        VectorError::Index("Node offset addition overflow".into())
                    })?;
                    if offset % sector_size != 0 {
                        return Err(VectorError::Index(
                            "Read offset must be sector-aligned".into(),
                        ));
                    }
                    let inner_mmap = inner.mmap.read();
                    let mmap_ref = inner_mmap
                        .as_ref()
                        .ok_or(VectorError::Index("Mmap failed".into()))?;
                    let doc_id_offset = offset
                        .checked_add(vector_size)
                        .and_then(|o| o.checked_add(neighbors_size))
                        .ok_or_else(|| VectorError::Index("DocId offset overflow".into()))?;
                    let doc_id_end = doc_id_offset
                        .checked_add(doc_id_size)
                        .ok_or_else(|| VectorError::Index("DocId end offset overflow".into()))?;
                    let doc_id_bytes =
                        mmap_ref.get(doc_id_offset..doc_id_end).ok_or_else(|| {
                            VectorError::Storage("DiskANN file truncated before doc_id".into())
                        })?;
                    #[cfg(not(feature = "docid-128"))]
                    let doc_id = u64::from_le_bytes(
                        doc_id_bytes
                            .try_into()
                            .map_err(|_| VectorError::Index("Corrupt doc_id".into()))?,
                    );
                    #[cfg(feature = "docid-128")]
                    let doc_id = u128::from_le_bytes(
                        doc_id_bytes
                            .try_into()
                            .map_err(|_| VectorError::Index("Corrupt doc_id".into()))?,
                    );
                    ids.push(DocId::from(doc_id));
                }
                *inner.doc_ids.write() = ids;
                Ok(())
            };

            let load_res = load_closure();

            if let Err(err) = load_res {
                if self.inner.config.fallback_policy == DiskAnnFallbackPolicy::UseHnswOnFailure {
                    tracing::error!(
                        index_path = %self.inner.config.index_path.display(),
                        error = %err,
                        "DiskANN index loading or integrity validation failed — using HNSW fallback"
                    );
                    self.init_hnsw_fallback()?;
                    return Ok(());
                } else {
                    return Err(err);
                }
            }
        }

        if wal_exists {
            self.recover_pending_delta_sync()?;
        }

        // Recover tombstones from tombstone.wal
        let tombstone_wal = self.inner.config.index_path.with_extension("tombstone.wal");
        if tombstone_wal.exists() {
            let recovered_tombstones = Self::read_tombstone_wal(&tombstone_wal)?;
            *self.inner.tombstones.write() = recovered_tombstones;
        }

        Ok(())
    }
}
