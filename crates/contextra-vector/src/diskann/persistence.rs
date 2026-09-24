use std::sync::Arc;
// FILE-CONTEXT
// ZWECK: DiskANN-Graphindex für Out-of-Core Approximate Nearest Neighbor Search (WP-4.3).
// INVARIANTEN: Lock-Hierarchie: header -> mmap -> cache / quantizer / doc_ids; atomic rename + parent dir sync bei file persistence.
// NICHT-OFFENSICHTLICH: Mmap für Vektor- & Graphlesezugriffe, ausgelagert an contextra-sys::mmap_readonly.
// STAND: TS:2026-09-10T19:30:00Z (SESSION: a9d67eae)

use super::config::{CachedNode, DiskAnnFallbackPolicy, VectorData};
use super::format::{
    DiskAnnFooter, DiskAnnHeader, DISKANN_FOOTER_MAGIC,
    DISKANN_INTEGRITY_KEY, DISKANN_MAGIC, DISKANN_VERSION,
};
use super::types::{DiskAnnIndex, DiskAnnIndexInner};
use contextra_core::{ContextraError, DocId, Result};
use std::sync::atomic::Ordering;

impl DiskAnnIndex {
    pub fn trigger_background_persist_delta(&self) {
        if !self.inner.flushing_in_progress.swap(true, Ordering::AcqRel) {
            let index_clone = self.clone();
            self.inner.compute_pool.execute(move || {
                struct FlushGuard(Arc<DiskAnnIndexInner>);
                impl Drop for FlushGuard {
                    fn drop(&mut self) {
                        self.0.flushing_in_progress.store(false, Ordering::Release);
                    }
                }
                let _guard = FlushGuard(Arc::clone(&index_clone.inner));
                if let Err(e) = index_clone.persist_delta_sync() {
                    tracing::error!(error = %e, "DiskANN Hintergrund-persist_delta fehlgeschlagen");
                }
            });
        }
    }

    pub async fn persist_delta(&self) -> Result<()> {
        self.persist_delta_sync()
    }

    pub fn persist_delta_sync(&self) -> Result<()> {
        let pending = {
            let mut guard = self.inner.pending_inserts.write();
            self.inner.pending_count.store(0, Ordering::Relaxed);
            std::mem::take(&mut *guard)
        };

        if pending.is_empty() {
            let pending_wal = self.inner.config.index_path.with_extension("pending.wal");
            if pending_wal.exists() {
                let _ = std::fs::remove_file(&pending_wal);
            }
            return Ok(());
        }

        let existing_count = self.len_sync();
        let total = existing_count + pending.len();
        let pending_ratio = pending.len() as f64 / total.max(1) as f64;

        if pending_ratio > 0.10 || existing_count == 0 {
            // Vollrebuild: bestehende + pending
            let (mut all_vecs, mut all_ids) = self.load_all_vectors_from_mmap()?;
            for (id, vec) in &pending {
                all_vecs.push(vec.clone());
                all_ids.push(*id);
            }
            return self.build_sync(&all_vecs, &all_ids);
        }

        // Inkrementeller Pfad
        let tmp_path = self.inner.config.index_path.with_extension("delta.tmp");
        self.write_incremental_to_file_sync(&tmp_path, &pending)?;

        // Atomares Rename + Parent-fsync (INV-DISKANN-1, P3)
        let index_path = self.inner.config.index_path.clone();
        let tmp_file = std::fs::File::open(&tmp_path)?;
        tmp_file
            .sync_all()
            .map_err(|e| ContextraError::Storage(format!("fsync tmp: {e}")))?;
        drop(tmp_file);
        std::fs::rename(&tmp_path, &index_path)
            .map_err(|e| ContextraError::Storage(format!("rename: {e}")))?;
        if let Some(parent) = index_path.parent() {
            if let Ok(dir) = std::fs::File::open(parent) {
                dir.sync_all()
                    .map_err(|e| ContextraError::Storage(format!("parent fsync: {e}")))?;
            }
        }

        let pending_wal = self.inner.config.index_path.with_extension("pending.wal");
        if pending_wal.exists() {
            let _ = std::fs::remove_file(&pending_wal);
        }

        self.load_sync() // Mmap neu laden
    }

    #[allow(clippy::unnecessary_cast)]
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
            if tombstones.contains(node.doc_id.inner() as u64) {
                continue;
            }
            let vec_f32 = match node.vector {
                VectorData::F32(v) => v,
                VectorData::U8(v) => {
                    let q_guard = self.inner.quantizer.read();
                    let q = q_guard
                        .as_ref()
                        .ok_or_else(|| ContextraError::Index("Quantizer missing".into()))?;
                    q.dequantize(&v)?
                }
            };
            all_vecs.push(vec_f32);
            all_ids.push(node.doc_id);
        }
        Ok((all_vecs, all_ids))
    }

    pub(crate) fn write_to_path_sync(
        &self,
        path: &std::path::Path,
        graph: &[Vec<u32>],
        vectors: &[Vec<f32>],
        ids: &[DocId],
    ) -> Result<()> {
        use std::fs::OpenOptions;
        use std::io::Write;
        use std::sync::atomic::Ordering;

        let n = vectors.len();
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(path)
            .map_err(ContextraError::Io)?;

        let quantizer_opt = {
            let q_guard = self.inner.quantizer.read();
            q_guard.clone()
        };
        let (q_min, q_max, quantized) = if let Some(ref q) = quantizer_opt {
            (
                q.mins().first().copied().unwrap_or(0.0),
                q.maxes().first().copied().unwrap_or(0.0),
                1,
            )
        } else {
            (0.0, 0.0, 0)
        };

        let header = DiskAnnHeader {
            magic: *DISKANN_MAGIC,
            version: DISKANN_VERSION,
            node_count: n as u64,
            dimension: self.inner.config.dimension as u32,
            max_degree: self.inner.config.max_degree as u32,
            sector_size: self.inner.config.sector_size as u32,
            entry_point: 0,
            metric: self.inner.config.distance_metric as u8,
            quantized,
            q_min,
            q_max,
        };

        let mut hmac = contextra_crypto::wal_crypto::WalHmac::new(DISKANN_INTEGRITY_KEY)?;

        let header_bytes = header.to_bytes();
        hmac.update(&header_bytes);
        file.write_all(&header_bytes).map_err(ContextraError::Io)?;
        let padding = vec![
            0u8;
            self.inner.config.sector_size
                - (DiskAnnHeader::SIZE % self.inner.config.sector_size)
        ];
        hmac.update(&padding);
        file.write_all(&padding).map_err(ContextraError::Io)?;

        for i in 0..n {
            let mut node_buf = Vec::new();

            if let Some(ref q) = quantizer_opt {
                let qv = q.quantize(&vectors[i])?;
                node_buf.extend_from_slice(&qv);
            } else {
                for &val in &vectors[i] {
                    node_buf.extend_from_slice(&val.to_le_bytes());
                }
            }

            let neighbors = &graph[i];
            node_buf.extend_from_slice(&(neighbors.len() as u32).to_le_bytes());
            for &neighbor in neighbors {
                node_buf.extend_from_slice(&neighbor.to_le_bytes());
            }
            let padding_count = self.inner.config.max_degree - neighbors.len();
            node_buf.extend_from_slice(&vec![0u8; padding_count * 4]);
            node_buf.extend_from_slice(&ids[i].inner().to_le_bytes());

            let used = node_buf.len();
            let node_size = self.inner.node_size_bytes.load(Ordering::SeqCst) as usize;
            if used < node_size {
                node_buf.extend_from_slice(&vec![0u8; node_size - used]);
            }

            hmac.update(&node_buf);
            file.write_all(&node_buf).map_err(ContextraError::Io)?;
        }

        let computed_hmac = hmac.finalize();
        let footer = DiskAnnFooter {
            magic: *DISKANN_FOOTER_MAGIC,
            hmac: computed_hmac,
        };
        file.write_all(&footer.to_bytes())
            .map_err(ContextraError::Io)?;

        file.sync_all().map_err(ContextraError::Io)?;
        Ok(())
    }

    pub(crate) fn init_hnsw_fallback(&self) -> Result<Arc<crate::hnsw::HnswIndex>> {
        let hnsw_config = crate::hnsw::HnswConfig {
            dimension: self.inner.config.dimension,
            distance_metric: self.inner.config.distance_metric,
            quantize: self.inner.config.quantize,
            ..crate::hnsw::HnswConfig::default()
        };
        let hnsw = Arc::new(crate::hnsw::HnswIndex::try_new(hnsw_config)?);
        *self.inner.hnsw_fallback.write() = Some(hnsw.clone());
        Ok(hnsw)
    }

    /// Loads the index from the configured path.
    pub async fn load(&self) -> Result<()> {
        self.load_sync()
    }

    pub fn load_sync(&self) -> Result<()> {
        let index_exists = self.inner.config.index_path.exists();
        let pending_wal = self.inner.config.index_path.with_extension("pending.wal");
        let wal_exists = pending_wal.exists();

        if !index_exists && !wal_exists {
            return Err(ContextraError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!(
                    "DiskANN index file does not exist: {}",
                    self.inner.config.index_path.display()
                ),
            )));
        }

        if index_exists {
            let inner = Arc::clone(&self.inner);
            let load_closure = move || {
                use std::sync::atomic::Ordering;

                // Clean up any orphaned temporary files from interrupted persist_delta or build calls
                let delta_tmp = inner.config.index_path.with_extension("delta.tmp");
                if delta_tmp.exists() {
                    if let Err(e) = std::fs::remove_file(&delta_tmp) {
                        tracing::warn!(
                            "Failed to remove orphaned delta.tmp file {}: {e}",
                            delta_tmp.display()
                        );
                    }
                }
                let idx_tmp = inner.config.index_path.with_extension("idx.tmp");
                if idx_tmp.exists() {
                    if let Err(e) = std::fs::remove_file(&idx_tmp) {
                        tracing::warn!(
                            "Failed to remove orphaned idx.tmp file {}: {e}",
                            idx_tmp.display()
                        );
                    }
                }

                let file =
                    std::fs::File::open(&inner.config.index_path).map_err(ContextraError::Io)?;
                let mmap = contextra_sys::mmap_readonly(&file).map_err(ContextraError::Io)?;

                if mmap.len() < DiskAnnHeader::SIZE + DiskAnnFooter::SIZE {
                    return Err(ContextraError::Storage(
                        "DiskANN file too small for header and footer".into(),
                    ));
                }

                let header_slice = mmap.get(0..DiskAnnHeader::SIZE).ok_or_else(|| {
                    ContextraError::Storage("DiskANN file too small for header".into())
                })?;
                let header = DiskAnnHeader::try_from_bytes(header_slice)?;

                // Verify HMAC integrity footer
                let footer_slice =
                    mmap.get(mmap.len() - DiskAnnFooter::SIZE..)
                        .ok_or_else(|| {
                            ContextraError::Storage("DiskANN file too small for footer".into())
                        })?;
                let footer = DiskAnnFooter::try_from_bytes(footer_slice)?;

                let payload = mmap
                    .get(..mmap.len() - DiskAnnFooter::SIZE)
                    .ok_or_else(|| {
                        ContextraError::Storage("DiskANN payload slice out of bounds".into())
                    })?;

                let mut hmac = contextra_crypto::wal_crypto::WalHmac::new(DISKANN_INTEGRITY_KEY)?;
                hmac.update(payload);
                let computed_hmac = hmac.finalize();

                if footer.hmac != computed_hmac {
                    return Err(ContextraError::Storage(
                        "DiskANN index file HMAC integrity validation failed: checksum mismatch"
                            .into(),
                    ));
                }

                if inner.config.sector_size != header.sector_size as usize {
                    return Err(ContextraError::Index(format!(
                        "DiskANN-Index inkompatibel: Config-sector_size={} stimmt nicht mit \
                         Header-sector_size={} überein. Index muss neu aufgebaut werden.",
                        inner.config.sector_size, header.sector_size
                    )));
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
                let doc_id_size = 8;
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
                    return Err(ContextraError::Storage(format!(
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
                    return Err(ContextraError::Index(
                        "Read size must be a multiple of sector_size".into(),
                    ));
                }

                for i in 0..header.node_count as u32 {
                    let index_offset =
                        (i as usize).checked_mul(node_size_bytes).ok_or_else(|| {
                            ContextraError::Index("Node offset multiplication overflow".into())
                        })?;
                    let offset = start_offset.checked_add(index_offset).ok_or_else(|| {
                        ContextraError::Index("Node offset addition overflow".into())
                    })?;
                    if offset % sector_size != 0 {
                        return Err(ContextraError::Index(
                            "Read offset must be sector-aligned".into(),
                        ));
                    }
                    let inner_mmap = inner.mmap.read();
                    let mmap_ref = inner_mmap
                        .as_ref()
                        .ok_or(ContextraError::Index("Mmap failed".into()))?;
                    let doc_id_offset = offset
                        .checked_add(vector_size)
                        .and_then(|o| o.checked_add(neighbors_size))
                        .ok_or_else(|| ContextraError::Index("DocId offset overflow".into()))?;
                    let doc_id_end = doc_id_offset
                        .checked_add(8)
                        .ok_or_else(|| ContextraError::Index("DocId end offset overflow".into()))?;
                    let doc_id_bytes =
                        mmap_ref.get(doc_id_offset..doc_id_end).ok_or_else(|| {
                            ContextraError::Storage("DiskANN file truncated before doc_id".into())
                        })?;
                    let doc_id = u64::from_le_bytes(
                        doc_id_bytes
                            .try_into()
                            .map_err(|_| ContextraError::Index("Corrupt doc_id".into()))?,
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

    pub(crate) fn load_node(&self, index: u32) -> Result<CachedNode> {
        use std::sync::atomic::Ordering;
        if let Some(node) = self.inner.cache.read().get(&index) {
            return Ok(node.clone());
        }

        let mmap_guard = self.inner.mmap.read();
        let mmap = mmap_guard
            .as_ref()
            .ok_or_else(|| ContextraError::Index("Index not loaded".into()))?;
        let header_guard = self.inner.header.read();
        let header = header_guard
            .as_ref()
            .ok_or_else(|| ContextraError::Index("Header missing".into()))?;

        let sector_size = header.sector_size as usize;
        let node_size = self.inner.node_size_bytes.load(Ordering::SeqCst) as usize;
        let read_size = node_size;
        if !read_size.is_multiple_of(sector_size) {
            return Err(ContextraError::Index(
                "Read size must be a multiple of sector_size".into(),
            ));
        }

        let start_offset = DiskAnnHeader::SIZE.div_ceil(sector_size) * sector_size;
        let index_offset = (index as usize)
            .checked_mul(node_size)
            .ok_or_else(|| ContextraError::Index("Node offset multiplication overflow".into()))?;
        let node_offset = start_offset
            .checked_add(index_offset)
            .ok_or_else(|| ContextraError::Index("Node offset addition overflow".into()))?;
        if node_offset % sector_size != 0 {
            return Err(ContextraError::Index(
                "Node read offset must be sector-aligned".into(),
            ));
        }

        let end_offset = node_offset
            .checked_add(node_size)
            .ok_or_else(|| ContextraError::Index("Node end offset addition overflow".into()))?;

        if end_offset > mmap.len() {
            return Err(ContextraError::Index("Node offset out of bounds".into()));
        }

        let node_data = mmap
            .get(node_offset..end_offset)
            .ok_or_else(|| ContextraError::Index("Node data out of bounds".into()))?;
        let mut cursor: usize = 0;

        let vector = if header.quantized != 0 {
            let dim = header.dimension as usize;
            let next_cursor = cursor
                .checked_add(dim)
                .ok_or_else(|| ContextraError::Index("Cursor overflow in quantized vector".into()))?;
            let slice = node_data
                .get(cursor..next_cursor)
                .ok_or_else(|| ContextraError::Index("Truncated quantized vector data".into()))?;
            cursor = next_cursor;
            VectorData::U8(slice.to_vec())
        } else {
            let dim = header.dimension as usize;
            let mut v = Vec::with_capacity(dim);
            for _ in 0..dim {
                let next_cursor = cursor
                    .checked_add(4)
                    .ok_or_else(|| ContextraError::Index("Cursor overflow in f32 vector".into()))?;
                let slice = node_data
                    .get(cursor..next_cursor)
                    .ok_or_else(|| ContextraError::Index("Truncated f32 vector data".into()))?;
                v.push(f32::from_le_bytes(slice.try_into().map_err(|_| {
                    ContextraError::Index("Invalid vector data".into())
                })?));
                cursor = next_cursor;
            }
            VectorData::F32(v)
        };

        let next_cursor = cursor
            .checked_add(4)
            .ok_or_else(|| ContextraError::Index("Cursor overflow in neighbor count".into()))?;
        let count_bytes = node_data
            .get(cursor..next_cursor)
            .ok_or_else(|| ContextraError::Index("Truncated neighbor count".into()))?;
        let neighbor_count = u32::from_le_bytes(
            count_bytes
                .try_into()
                .map_err(|_| ContextraError::Index("Invalid neighbor count".into()))?,
        ) as usize;
        cursor = next_cursor;

        if neighbor_count > header.max_degree as usize {
            return Err(ContextraError::Index(format!(
                "Corrupt DiskANN node: neighbor_count {} > max_degree {}",
                neighbor_count, header.max_degree
            )));
        }

        let mut neighbors = Vec::with_capacity(neighbor_count);
        for _ in 0..neighbor_count {
            let nxt = cursor
                .checked_add(4)
                .ok_or_else(|| ContextraError::Index("Cursor overflow in neighbor ID".into()))?;
            let slice = node_data
                .get(cursor..nxt)
                .ok_or_else(|| ContextraError::Index("Truncated neighbor ID".into()))?;
            neighbors.push(u32::from_le_bytes(
                slice
                    .try_into()
                    .map_err(|_| ContextraError::Index("Invalid neighbor ID".into()))?,
            ));
            cursor = nxt;
        }

        let skip_bytes = (header.max_degree as usize)
            .checked_sub(neighbor_count)
            .and_then(|diff| diff.checked_mul(4))
            .ok_or_else(|| ContextraError::Index("Padding overflow".into()))?;
        cursor = cursor
            .checked_add(skip_bytes)
            .ok_or_else(|| ContextraError::Index("Cursor overflow in padding".into()))?;

        let doc_id_end = cursor
            .checked_add(8)
            .ok_or_else(|| ContextraError::Index("Cursor overflow in DocId".into()))?;
        let doc_id_bytes = node_data
            .get(cursor..doc_id_end)
            .ok_or_else(|| ContextraError::Index("Truncated DocId".into()))?;
        let doc_id = DocId::from(u64::from_le_bytes(
            doc_id_bytes
                .try_into()
                .map_err(|_| ContextraError::Index("Invalid DocId".into()))?,
        ));

        let node = CachedNode {
            vector,
            neighbors,
            doc_id,
        };

        let mut cache = self.inner.cache.write();
        if cache.len() * node_size >= self.inner.config.memory_budget {
            // FIND-IND-004: Replace full cache wipe with 25% partial eviction
            let to_remove = cache.len() / 4;
            let keys: Vec<_> = cache.keys().take(to_remove).cloned().collect();
            for k in keys {
                cache.remove(&k);
            }
        }
        cache.insert(index, node.clone());

        Ok(node)
    }

}
