// FILE-CONTEXT
// ZWECK: DiskANN-Graphindex für Out-of-Core Approximate Nearest Neighbor Search (WP-4.3).
// INVARIANTEN: Lock-Hierarchie: header -> mmap -> cache / quantizer / doc_ids; atomic rename + parent dir sync bei file persistence.
// NICHT-OFFENSICHTLICH: Mmap für Vektor- & Graphlesezugriffe, ausgelagert an contextra-sys::mmap_readonly.
// STAND: TS:2026-09-10T19:30:00Z (SESSION: a9d67eae)

use super::format::{
    DISKANN_INTEGRITY_KEY, MAX_PENDING_WAL_DIM, PENDING_WAL_HEADER_SIZE, PENDING_WAL_MAGIC,
    PENDING_WAL_VERSION, TOMBSTONE_WAL_HEADER_SIZE, TOMBSTONE_WAL_MAGIC, TOMBSTONE_WAL_VERSION,
};
use super::types::{DiskAnnIndex, SearchCandidate};
use crate::distance::compute_distance_trusted;
use contextra_core::{ContextraError, DocId, Result};
use roaring::RoaringTreemap;
use std::sync::atomic::Ordering;

impl DiskAnnIndex {
    pub(crate) fn prune_in_memory(
        &self,
        candidates: &mut [SearchCandidate],
        vectors: &[Vec<f32>],
        max_degree: usize,
        alpha: f32,
    ) -> Result<Vec<u32>> {
        if candidates.is_empty() {
            return Ok(Vec::new());
        }
        candidates.sort_by(|a, b| a.distance.total_cmp(&b.distance));

        let mut pruned = Vec::with_capacity(max_degree);
        for cand in candidates.iter() {
            if pruned.len() >= max_degree {
                break;
            }

            let mut keep = true;
            for &p_idx in &pruned {
                let dist_p_cand = compute_distance_trusted(
                    &vectors[cand.index as usize],
                    &vectors[p_idx as usize],
                    self.inner.config.distance_metric,
                )?;
                if !dist_p_cand.is_finite() || !cand.distance.is_finite() {
                    tracing::warn!(
                        "DiskANN robust-prune: non-finite distance encountered, treating candidate as non-prunable (fail-open, candidate retained)"
                    );
                    continue;
                }
                if alpha * dist_p_cand < cand.distance {
                    keep = false;
                    break;
                }
            }

            if keep {
                pruned.push(cand.index);
            }
        }
        Ok(pruned)
    }

    /// Appends a tombstone record to `tombstone.wal` with HMAC-SHA256 integrity protection.
    ///
    /// # Binary Format
    /// - File Header (written on new/empty file creation): `[TWAL: 4 bytes] [version=1: 1 byte]`
    /// - Entry Layout:
    ///   - `id`: `u64` LE (8 bytes)
    ///   - `hmac`: `[u8; 32]` (32-byte HMAC-SHA256 computed over `id_bytes`)
    pub(crate) async fn append_to_tombstone_wal(path: &std::path::Path, id: DocId) -> Result<()> {
        use std::io::Write;

        let is_new = match std::fs::metadata(path) {
            Ok(meta) => meta.len() == 0,
            Err(_) => true,
        };

        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .map_err(ContextraError::Io)?;

        if is_new {
            file.write_all(TOMBSTONE_WAL_MAGIC)
                .map_err(ContextraError::Io)?;
            file.write_all(&[TOMBSTONE_WAL_VERSION])
                .map_err(ContextraError::Io)?;
        }

        let id_bytes = id.inner().to_le_bytes();
        let mut hmac = contextra_crypto::wal_crypto::WalHmac::new(DISKANN_INTEGRITY_KEY)?;
        hmac.update(&id_bytes);
        let computed_hmac = hmac.finalize();

        file.write_all(&id_bytes).map_err(ContextraError::Io)?;
        file.write_all(&computed_hmac).map_err(ContextraError::Io)?;
        file.sync_all().map_err(ContextraError::Io)?;
        Ok(())
    }

    /// Reads and verifies uncommitted tombstone entries from `tombstone.wal`.
    pub(crate) fn read_tombstone_wal(path: &std::path::Path) -> Result<RoaringTreemap> {
        use std::io::Read;
        use subtle::ConstantTimeEq;

        let mut bitset = RoaringTreemap::new();

        if !path.exists() {
            return Ok(bitset);
        }
        let mut file = match std::fs::File::open(path) {
            Ok(f) => f,
            Err(_) => return Ok(bitset),
        };

        let file_len = match file.metadata() {
            Ok(m) => m.len() as usize,
            Err(_) => return Ok(bitset),
        };

        if file_len == 0 {
            return Ok(bitset);
        }

        if file_len < TOMBSTONE_WAL_HEADER_SIZE {
            return Err(ContextraError::Storage(
                "Unbekanntes oder veraltetes tombstone.wal-Format, bitte vor Upgrade leeren".into(),
            ));
        }

        let mut magic_buf = [0u8; 4];
        let mut version_buf = [0u8; 1];
        if file.read_exact(&mut magic_buf).is_err() || file.read_exact(&mut version_buf).is_err() {
            return Err(ContextraError::Storage(
                "Unbekanntes oder veraltetes tombstone.wal-Format, bitte vor Upgrade leeren".into(),
            ));
        }

        if magic_buf != *TOMBSTONE_WAL_MAGIC || version_buf[0] != TOMBSTONE_WAL_VERSION {
            return Err(ContextraError::Storage(
                "Unbekanntes oder veraltetes tombstone.wal-Format, bitte vor Upgrade leeren".into(),
            ));
        }

        let mut offset = TOMBSTONE_WAL_HEADER_SIZE;
        let doc_id_size = std::mem::size_of::<DocId>();
        let mut buf_id = vec![0u8; doc_id_size];

        while file.read_exact(&mut buf_id).is_ok() {
            let entry_offset = offset;
            offset += doc_id_size;

            #[cfg(not(feature = "docid-128"))]
            let doc_id = u64::from_le_bytes(
                buf_id
                    .as_slice()
                    .try_into()
                    .map_err(|_| ContextraError::Storage("Corrupt DocId in WAL".into()))?,
            );
            #[cfg(feature = "docid-128")]
            let doc_id = u128::from_le_bytes(
                buf_id
                    .as_slice()
                    .try_into()
                    .map_err(|_| ContextraError::Storage("Corrupt DocId in WAL".into()))?,
            );

            let mut buf_hmac = [0u8; 32];
            if file.read_exact(&mut buf_hmac).is_err() {
                tracing::warn!("Truncated HMAC in tombstone.wal");
                break;
            }
            offset += 32;

            let mut hmac = contextra_crypto::wal_crypto::WalHmac::new(DISKANN_INTEGRITY_KEY)?;
            hmac.update(&buf_id);
            let computed_hmac = hmac.finalize();

            if computed_hmac.ct_eq(&buf_hmac).into() {
                bitset.insert(doc_id);
            } else {
                tracing::error!(
                    offset = entry_offset,
                    doc_id = doc_id,
                    "tombstone.wal: HMAC-Mismatch bei Eintrag, Eintrag verworfen"
                );
            }
        }
        Ok(bitset)
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
        use std::io::Write;

        let is_new = match std::fs::metadata(path) {
            Ok(meta) => meta.len() == 0,
            Err(_) => true,
        };

        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .map_err(ContextraError::Io)?;

        if is_new {
            file.write_all(PENDING_WAL_MAGIC)
                .map_err(ContextraError::Io)?;
            file.write_all(&[PENDING_WAL_VERSION])
                .map_err(ContextraError::Io)?;
        }

        let dim = embedding.len() as u32;
        let id_bytes = id.inner().to_le_bytes();
        let dim_bytes = dim.to_le_bytes();

        let mut entry_bytes = Vec::with_capacity(8 + 4 + (embedding.len() * 4));
        entry_bytes.extend_from_slice(&id_bytes);
        entry_bytes.extend_from_slice(&dim_bytes);
        for &val in embedding {
            entry_bytes.extend_from_slice(&val.to_le_bytes());
        }

        let mut hmac = contextra_crypto::wal_crypto::WalHmac::new(DISKANN_INTEGRITY_KEY)?;
        hmac.update(&entry_bytes);
        let computed_hmac = hmac.finalize();

        file.write_all(&entry_bytes).map_err(ContextraError::Io)?;
        file.write_all(&computed_hmac).map_err(ContextraError::Io)?;
        file.sync_all().map_err(ContextraError::Io)?;
        Ok(())
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
        let mut file = match std::fs::File::open(path) {
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
            return Err(ContextraError::Storage(
                "Unbekanntes oder veraltetes pending.wal-Format, bitte vor Upgrade leeren".into(),
            ));
        }

        let mut magic_buf = [0u8; 4];
        let mut version_buf = [0u8; 1];
        if file.read_exact(&mut magic_buf).is_err() || file.read_exact(&mut version_buf).is_err() {
            return Err(ContextraError::Storage(
                "Unbekanntes oder veraltetes pending.wal-Format, bitte vor Upgrade leeren".into(),
            ));
        }

        if magic_buf != *PENDING_WAL_MAGIC || version_buf[0] != PENDING_WAL_VERSION {
            return Err(ContextraError::Storage(
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
            let doc_id =
                DocId::from(u64::from_le_bytes(buf_id.as_slice().try_into().map_err(
                    |_| ContextraError::Storage("Corrupt DocId in WAL".into()),
                )?));
            #[cfg(feature = "docid-128")]
            let doc_id =
                DocId::from(u128::from_le_bytes(buf_id.as_slice().try_into().map_err(
                    |_| ContextraError::Storage("Corrupt DocId in WAL".into()),
                )?));
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

            let mut hmac = contextra_crypto::wal_crypto::WalHmac::new(DISKANN_INTEGRITY_KEY)?;
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
            let _ = std::fs::remove_file(&pending_wal);
        }
        Ok(count)
    }

    /// Mergt pending inserts in den On-Disk-Graphen.
    ///
    /// ALGORITHMUS (arXiv:2602.21514 §4 "Streaming DiskANN"):
    /// Wenn pending_ratio > 10%: Vollrebuild (bestehend + pending).
    /// Sonst: Inkrementeller Greedy-Search-basierter Insert pro Vektor.
    ///
    /// ATOMARES WRITE: Tmp → fsync → Rename → Parent-fsync (P3-konform).
    /// INVARIANTE INV-DISKANN-1: Atomares Rename-Muster immer eingehalten.
    pub(crate) fn prune_streaming(
        &self,
        cand_idx: u32,
        candidates: &mut [SearchCandidate],
        existing_count: usize,
        new_vecs: &[(DocId, Vec<f32>)],
        max_degree: usize,
        alpha: f32,
    ) -> Result<Vec<u32>> {
        if candidates.is_empty() {
            return Ok(Vec::new());
        }
        candidates.sort_by(|a, b| a.distance.total_cmp(&b.distance));

        let mut pruned = Vec::with_capacity(max_degree);
        let mut pruned_vecs: Vec<Vec<f32>> = Vec::with_capacity(max_degree);
        let _cand_v = self.get_vec_mixed(cand_idx, existing_count, new_vecs)?;

        for cand in candidates.iter() {
            if pruned.len() >= max_degree {
                break;
            }

            let cand_node_v = self.get_vec_mixed(cand.index, existing_count, new_vecs)?;
            let mut keep = true;

            for p_v in &pruned_vecs {
                let dist_p_cand =
                    compute_distance_trusted(&cand_node_v, p_v, self.inner.config.distance_metric)?;
                if !dist_p_cand.is_finite() || !cand.distance.is_finite() {
                    tracing::warn!(
                        "DiskANN robust-prune: non-finite distance encountered, treating candidate as non-prunable (fail-open, candidate retained)"
                    );
                    continue;
                }
                if alpha * dist_p_cand < cand.distance {
                    keep = false;
                    break;
                }
            }

            if keep {
                pruned.push(cand.index);
                pruned_vecs.push(cand_node_v);
            }
        }
        Ok(pruned)
    }

    /// Inkrementeller Vamana-Insert für neue Vektoren (Streaming DiskANN; arXiv:2602.21514 §4).
    ///
    /// Phase 1: Lade bestehende Graph-Topologie read-only aus Mmap via `load_node`.
    /// Phase 2: Für jeden neuen Vektor: Greedy Beam-Search über den aktuellen Graphen.
    /// Phase 3: RNG-Pruning (α = 1.2) zur Auswahl von maximal `max_degree` Nachbarn.
    /// Phase 4: Rückwärts-Kanten hinzufügen und ggf. Nachbarschaft überschrittener Nachbarn re-prunen.
    /// Phase 5: Geänderte Graph-Struktur und neue Knoten atomar auf Disk schreiben.
    ///
    /// AI-TAG[RESOLVED] Echte inkrementelle Streaming-DiskANN Implementierung mit Beam-Search, RNG-Pruning und Rückwärts-Kanten-Kompression. (TS:2026-09-07T06:15:00Z) (SESSION: f04imm01)
    pub(crate) fn write_incremental_to_file_sync(
        &self,
        tmp_path: &std::path::Path,
        new_vecs: &[(DocId, Vec<f32>)],
    ) -> Result<()> {
        if new_vecs.is_empty() {
            return Ok(());
        }

        let existing_count = self.len_sync();
        if existing_count == 0 {
            let mut all_vecs = Vec::with_capacity(new_vecs.len());
            let mut all_ids = Vec::with_capacity(new_vecs.len());
            for (id, vec) in new_vecs {
                all_vecs.push(vec.clone());
                all_ids.push(*id);
            }
            return self.build_to_path_sync(tmp_path, &all_vecs, &all_ids);
        }

        let total_nodes = existing_count + new_vecs.len();
        let mut graph: Vec<Vec<u32>> = vec![vec![]; total_nodes];

        // Phase 1: Adjazenzlisten bestehender Knoten aus Mmap laden
        #[allow(clippy::needless_range_loop)]
        for i in 0..existing_count {
            let node = self.load_node(i as u32)?;
            graph[i] = node.neighbors;
        }

        let mut all_ids = self.inner.doc_ids.read().clone();
        for (id, _) in new_vecs {
            all_ids.push(*id);
        }

        let entry_point = self.inner.header.read().map(|h| h.entry_point).unwrap_or(0);
        let alpha = 1.2f32;

        // Phase 2, 3 & 4: Inkrementelles Einfügen jedes neuen Vektors
        for (j, (_id, vec)) in new_vecs.iter().enumerate() {
            let new_node_idx = (existing_count + j) as u32;

            // Phase 2: Beam-Search über den bisherigen Graphen (0..new_node_idx)
            let mut candidates = self.search_streaming(
                vec,
                &graph,
                existing_count,
                new_vecs,
                entry_point,
                self.inner.config.beam_width,
            )?;

            // Phase 3: RNG-Pruning
            let pruned = self.prune_streaming(
                new_node_idx,
                &mut candidates,
                existing_count,
                new_vecs,
                self.inner.config.max_degree,
                alpha,
            )?;

            // Phase 4: Rückwärts-Kanten & Re-Pruning bei Grad-Überschreitung
            for &neighbor in &pruned {
                let neighbor_idx = neighbor as usize;
                if !graph[neighbor_idx].contains(&new_node_idx) {
                    graph[neighbor_idx].push(new_node_idx);
                    if graph[neighbor_idx].len() > self.inner.config.max_degree {
                        let nbr_v = self.get_vec_mixed(neighbor, existing_count, new_vecs)?;
                        let mut cand_vec: Vec<SearchCandidate> =
                            Vec::with_capacity(graph[neighbor_idx].len());
                        for &idx in &graph[neighbor_idx] {
                            let idx_v = self.get_vec_mixed(idx, existing_count, new_vecs)?;
                            let dist = compute_distance_trusted(
                                &nbr_v,
                                &idx_v,
                                self.inner.config.distance_metric,
                            )?;
                            cand_vec.push(SearchCandidate {
                                index: idx,
                                distance: dist,
                            });
                        }
                        graph[neighbor_idx] = self.prune_streaming(
                            neighbor,
                            &mut cand_vec,
                            existing_count,
                            new_vecs,
                            self.inner.config.max_degree,
                            alpha,
                        )?;
                    }
                }
            }
            graph[new_node_idx as usize] = pruned;
        }

        // Phase 5: Vektoren für alle Knoten zusammenstellen und in tmp_path serialisieren
        let (mut all_vecs, _) = self.load_all_vectors_from_mmap()?;
        for (_, vec) in new_vecs {
            all_vecs.push(vec.clone());
        }

        self.write_to_path_sync(tmp_path, &graph, &all_vecs, &all_ids)
    }

    pub fn build_to_path_sync(
        &self,
        target_path: &std::path::Path,
        vectors: &[Vec<f32>],
        ids: &[DocId],
    ) -> Result<()> {
        if vectors.is_empty() {
            return Ok(());
        }

        let n = vectors.len();

        // 0. SQ8 Training if needed
        if self.inner.config.quantize {
            let refs: Vec<&[f32]> = vectors.iter().map(|v| v.as_slice()).collect();
            let q = crate::quantize::ScalarQuantizer::train(&refs, self.inner.config.dimension);
            *self.inner.quantizer.write() = Some(q);
        }

        // 1. Initial State (Ring topology)
        let mut graph: Vec<Vec<u32>> = vec![vec![]; n];
        for (i, node_graph) in graph.iter_mut().enumerate() {
            let neighbor = ((i + 1) % n) as u32;
            node_graph.push(neighbor);
        }

        // 2. Vamana In-Memory Build Passes (Pass 1: alpha=1.0, Pass 2: alpha=1.2)
        let entry_point = 0u32;
        for alpha in [1.0f32, 1.2f32] {
            for i in 0..n {
                let mut candidates = self.search_in_memory(
                    &vectors[i],
                    &graph,
                    vectors,
                    entry_point,
                    self.inner.config.beam_width,
                )?;
                let pruned = self.prune_in_memory(
                    &mut candidates,
                    vectors,
                    self.inner.config.max_degree,
                    alpha,
                )?;

                for &neighbor in &pruned {
                    let neighbor_idx = neighbor as usize;
                    if !graph[neighbor_idx].contains(&(i as u32)) {
                        graph[neighbor_idx].push(i as u32);
                        if graph[neighbor_idx].len() > self.inner.config.max_degree {
                            let mut cand_vec: Vec<SearchCandidate> = graph[neighbor_idx]
                                .iter()
                                .map(|&idx| {
                                    let dist = compute_distance_trusted(
                                        &vectors[neighbor_idx],
                                        &vectors[idx as usize],
                                        self.inner.config.distance_metric,
                                    )
                                    .unwrap_or(f32::MAX);
                                    SearchCandidate {
                                        index: idx,
                                        distance: dist,
                                    }
                                })
                                .collect();
                            graph[neighbor_idx] = self.prune_in_memory(
                                &mut cand_vec,
                                vectors,
                                self.inner.config.max_degree,
                                alpha,
                            )?;
                        }
                    }
                }
                graph[i] = pruned;
            }
        }

        // 3. Final Write
        self.write_to_path_sync(target_path, &graph, vectors, ids)
    }

    /// Builds the index from a set of vectors.
    pub async fn build(&self, vectors: &[Vec<f32>], ids: &[DocId]) -> Result<()> {
        self.build_sync(vectors, ids)
    }

    pub fn build_sync(&self, vectors: &[Vec<f32>], ids: &[DocId]) -> Result<()> {
        let tmp_path = self.inner.config.index_path.with_extension("idx.tmp");
        self.build_to_path_sync(&tmp_path, vectors, ids)?;

        std::fs::rename(&tmp_path, &self.inner.config.index_path).map_err(ContextraError::Io)?;

        // Fsync parent directory after rename for POSIX atomic directory entry durability
        if let Some(parent) = self.inner.config.index_path.parent() {
            let parent_dir = std::fs::File::open(parent).map_err(ContextraError::Io)?;
            parent_dir.sync_all().map_err(ContextraError::Io)?;
        }

        let pending_wal = self.inner.config.index_path.with_extension("pending.wal");
        if pending_wal.exists() {
            let _ = std::fs::remove_file(&pending_wal);
        }

        let tombstone_wal = self.inner.config.index_path.with_extension("tombstone.wal");
        if tombstone_wal.exists() {
            let _ = std::fs::remove_file(&tombstone_wal);
        }
        self.inner.tombstones.write().clear();

        self.load_sync()?;
        self.verify_graph_integrity_debug()?;

        Ok(())
    }

    pub(crate) fn verify_graph_integrity_debug(&self) -> Result<()> {
        #[cfg(debug_assertions)]
        {
            let node_count = self
                .inner
                .header
                .read()
                .map(|h| h.node_count as u32)
                .unwrap_or(0);
            let max_degree = self.inner.config.max_degree;
            for i in 0..node_count {
                let node = self.load_node(i)?;
                assert!(
                    node.neighbors.len() <= max_degree,
                    "Graph integrity violation: Node {} has {} neighbors, exceeding max_degree {}",
                    i,
                    node.neighbors.len(),
                    max_degree
                );
            }
        }
        Ok(())
    }
}
