use ahash::AHashSet;
use rand::Rng;
use roaring::RoaringTreemap;
use std::sync::atomic::Ordering;

use contextra_core::{ContextraError, DocId, Result};

use crate::distance::compute_distance_trusted;
use super::arena::HnswArena;
use super::batch::{PreparedInsert, SearchContext};
use super::sq8_bias::Sq8Bias;
use super::types::{
    Candidate, HnswIndex, HnswIndexCore, RebuildGuard,
    RebuildStatus, VectorData,
};

impl HnswIndexCore {
    pub fn rebuild_status(&self) -> RebuildStatus {
        if self.hot.rebuilding.load(Ordering::SeqCst) {
            RebuildStatus::Running
        } else {
            RebuildStatus::Idle
        }
    }

    pub async fn wait_for_rebuild(&self) -> bool {
        self.wait_for_rebuild_with_timeout(std::time::Duration::from_secs(60))
            .await
    }

    pub async fn wait_for_rebuild_with_timeout(&self, timeout: std::time::Duration) -> bool {
        let deadline = std::time::Instant::now() + timeout;
        while self.hot.rebuilding.load(Ordering::Acquire) {
            if std::time::Instant::now() >= deadline {
                return false;
            }
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
        true
    }

    pub(super) fn random_layer(&self) -> usize {
        let mut rng = rand::thread_rng();
        let r: f32 = rng.gen::<f32>();
        let r_clamped = r.max(f32::EPSILON);
        let layer = (-(r_clamped.ln()) as f64 * self.hot.ml) as usize;
        layer.min(32)
    }

    fn compute_distance_with_data(
        &self,
        query_exact: &[f32],
        query_quantized: Option<&[u8]>,
        data: &VectorData,
        quantizer_opt: Option<&crate::quantize::ScalarQuantizer>,
    ) -> Result<f32> {
        match data {
            VectorData::F32(v) => {
                compute_distance_trusted(query_exact, v, self.cold.config.distance_metric)
            }
            VectorData::U8(v) => {
                let guard;
                let q = if let Some(q_ref) = quantizer_opt {
                    q_ref
                } else {
                    guard = self.cold.quantizer.read();
                    guard.as_ref().ok_or_else(|| {
                        contextra_core::ContextraError::Index("Quantizer not trained".into())
                    })?
                };
                if let Some(qq) = query_quantized {
                    q.symmetric_dist(qq, v, self.cold.config.distance_metric)
                } else {
                    q.asymmetric_dist(query_exact, v, self.cold.config.distance_metric)
                }
            }
        }
    }

    fn compute_distance_with_mmap(
        &self,
        query_exact: &[f32],
        query_quantized: Option<&[u8]>,
        mmap: &crate::persistence::MmapIndex,
        record: &crate::persistence::NodeRecord,
        quantizer_opt: Option<&crate::quantize::ScalarQuantizer>,
    ) -> Result<f32> {
        let vector_bytes = mmap.get_vector(record)?;
        if mmap.header.is_quantized() {
            let guard;
            let q = if let Some(q_ref) = quantizer_opt {
                q_ref
            } else {
                guard = self.cold.quantizer.read();
                guard.as_ref().ok_or_else(|| {
                    contextra_core::ContextraError::Index("Quantizer not trained".into())
                })?
            };
            if let Some(qq) = query_quantized {
                q.symmetric_dist(qq, vector_bytes, self.cold.config.distance_metric)
            } else {
                q.asymmetric_dist(query_exact, vector_bytes, self.cold.config.distance_metric)
            }
        } else {
            crate::distance::compute_distance_f32_bytes_trusted(
                query_exact,
                vector_bytes,
                self.cold.config.distance_metric,
            )
        }
    }

    fn compute_symmetric_distance(&self, data_a: &VectorData, data_b: &VectorData) -> Result<f32> {
        match (data_a, data_b) {
            (VectorData::F32(a), VectorData::F32(b)) => {
                compute_distance_trusted(a, b, self.cold.config.distance_metric)
            }
            (VectorData::U8(a), VectorData::U8(b)) => {
                let guard = self.cold.quantizer.read();
                guard
                    .as_ref()
                    .ok_or_else(|| {
                        contextra_core::ContextraError::Index("Quantizer not trained".into())
                    })?
                    .symmetric_dist(a, b, self.cold.config.distance_metric)
            }
            _ => Err(ContextraError::Index(
                "Mixed vector representations (F32/U8) are not supported".into(),
            )),
        }
    }

    pub(super) fn resolve_dist(
        &self,
        idx: usize,
        query: &[f32],
        query_q: Option<&[u8]>,
        ctx: &SearchContext,
    ) -> Result<f32> {
        let q_ref = ctx.quantizer.as_deref();
        let base_batch_idx = ctx.mmap_node_count + ctx.nodes.len();
        if idx >= base_batch_idx {
            let prepared_offset = idx - base_batch_idx;
            if let Some(prepared) = ctx.prior_prepared.get(prepared_offset) {
                return self.compute_distance_with_data(
                    query,
                    query_q,
                    &prepared.vector_data,
                    q_ref,
                );
            }
        }
        if let Some(mmap) = ctx.mmap {
            if idx < ctx.mmap_node_count {
                let record = mmap.get_node_record(idx)?;
                return self.compute_distance_with_mmap(query, query_q, mmap, &record, q_ref);
            }
        }
        let ram_idx = idx.saturating_sub(ctx.mmap_node_count);
        if let Some(node) = ctx.nodes.get(ram_idx) {
            return self.compute_distance_with_data(query, query_q, &node.vector, q_ref);
        }
        Ok(f32::MAX)
    }

    pub(super) fn resolve_doc_id(&self, idx: usize, ctx: &SearchContext) -> Result<DocId> {
        let base_batch_idx = ctx.mmap_node_count + ctx.nodes.len();
        if idx >= base_batch_idx {
            let prepared_offset = idx - base_batch_idx;
            if let Some(prepared) = ctx.prior_prepared.get(prepared_offset) {
                return Ok(prepared.doc_id);
            }
        }
        if let Some(mmap) = ctx.mmap {
            if idx < ctx.mmap_node_count {
                let record = mmap.get_node_record(idx)?;
                return Ok(DocId::new(record.doc_id));
            }
        }
        let ram_idx = idx.saturating_sub(ctx.mmap_node_count);
        if let Some(node) = ctx.nodes.get(ram_idx) {
            return Ok(node.doc_id);
        }
        Err(ContextraError::Index(format!(
            "Node doc_id not found for index {idx}"
        )))
    }

    fn resolve_vector_data_with_batch(
        &self,
        idx: usize,
        ctx: &SearchContext,
        pending_idx: usize,
        pending_vector: &VectorData,
        prior_prepared: &[PreparedInsert],
    ) -> Result<VectorData> {
        if idx == pending_idx {
            return Ok(pending_vector.clone());
        }

        let ram_nodes_count = ctx.nodes.len();
        let base_batch_idx = ctx.mmap_node_count + ram_nodes_count;

        if idx >= base_batch_idx {
            let prepared_offset = idx - base_batch_idx;
            if let Some(prepared) = prior_prepared.get(prepared_offset) {
                return Ok(prepared.vector_data.clone());
            }
        }

        if let Some(mmap) = ctx.mmap {
            if idx < ctx.mmap_node_count {
                let record = mmap.get_node_record(idx)?;
                let bytes = mmap.get_vector(&record)?;
                return if mmap.header.is_quantized() {
                    Ok(VectorData::U8(bytes.to_vec()))
                } else {
                    let mut v = vec![0.0f32; self.cold.config.dimension];
                    for i in 0..self.cold.config.dimension {
                        v[i] = f32::from_le_bytes(bytes[i * 4..(i + 1) * 4].try_into().map_err(
                            |_| ContextraError::Index("Corrupt f32 in mmap vector".into()),
                        )?);
                    }
                    Ok(VectorData::F32(v))
                };
            }
            return Ok(ctx.nodes[idx - ctx.mmap_node_count].vector.clone());
        }

        if idx < ctx.nodes.len() {
            return Ok(ctx.nodes[idx].vector.clone());
        }

        Err(ContextraError::Index(format!(
            "Invalid node index {} in resolve_vector_data_with_batch",
            idx
        )))
    }

    pub(super) fn compute_symmetric_distance_hybrid_with_batch(
        &self,
        idx_a: usize,
        idx_b: usize,
        ctx: &SearchContext,
        pending_idx: usize,
        pending_vector: &VectorData,
        prior_prepared: &[PreparedInsert],
    ) -> Result<f32> {
        let data_a = self.resolve_vector_data_with_batch(
            idx_a,
            ctx,
            pending_idx,
            pending_vector,
            prior_prepared,
        )?;
        let data_b = self.resolve_vector_data_with_batch(
            idx_b,
            ctx,
            pending_idx,
            pending_vector,
            prior_prepared,
        )?;
        self.compute_symmetric_distance(&data_a, &data_b)
    }

    pub(super) fn select_neighbors_heuristic_with_batch(
        &self,
        ctx: &SearchContext,
        candidates: &[Candidate],
        m: usize,
        pending_idx: usize,
        pending_vector: &VectorData,
        prior_prepared: &[PreparedInsert],
    ) -> Result<Vec<u32>> {
        if candidates.is_empty() {
            return Ok(Vec::new());
        }
        if candidates.len() <= m {
            return Ok(candidates.iter().map(|c| c.index as u32).collect());
        }

        let distances: Vec<f32> = candidates.iter().map(|c| c.distance).collect();
        let mean = distances.iter().sum::<f32>() / distances.len() as f32;
        let variance =
            distances.iter().map(|d| (d - mean).powi(2)).sum::<f32>() / distances.len() as f32;
        let std_dev = variance.sqrt();

        let relaxation = if self.cold.config.quantize {
            (0.1 * (1.0 / (1.0 + std_dev))).clamp(0.02, 0.2)
        } else {
            0.0
        };

        let mut result: Vec<Candidate> = Vec::with_capacity(m);
        let mut sorted_cands = candidates.to_vec();
        sorted_cands.sort_by(|a, b| a.distance.total_cmp(&b.distance));

        for closest in sorted_cands {
            if result.len() >= m {
                break;
            }
            let mut keep = true;
            for selected in &result {
                let dist_between = self.compute_symmetric_distance_hybrid_with_batch(
                    closest.index,
                    selected.index,
                    ctx,
                    pending_idx,
                    pending_vector,
                    prior_prepared,
                )?;

                if closest.distance > dist_between * (1.0 + relaxation) {
                    keep = false;
                    break;
                }
            }
            if keep {
                result.push(closest);
            }
        }

        let min_neighbors = m / 2;
        if result.len() < min_neighbors && candidates.len() >= min_neighbors {
            let mut fallback = result;
            let mut sorted_fallback = candidates.to_vec();
            sorted_fallback.sort_by(|a, b| a.distance.total_cmp(&b.distance));

            for cand in sorted_fallback {
                if (fallback.len() >= m
                    || (fallback.len() >= min_neighbors && !fallback.is_empty()))
                    && fallback.len() >= min_neighbors
                {
                    break;
                }
                if !fallback.iter().any(|c| c.index == cand.index) {
                    fallback.push(cand);
                }
            }
            return Ok(fallback.iter().map(|c| c.index as u32).collect());
        }

        Ok(result.iter().map(|c| c.index as u32).collect())
    }

    pub(super) fn do_delete(&self, id: DocId) -> Result<()> {
        let node_idx = self.hot.doc_to_node.write().remove(&id.inner());
        if let Some(idx) = node_idx {
            self.cold.deleted_nodes.write().insert(idx as u64);
            self.hot.deleted_count.fetch_add(1, Ordering::SeqCst);

            let ep_val = self.hot.get_entry_point();
            let ram_ep_val = self.hot.get_ram_entry_point();

            if ep_val == Some(idx) || ram_ep_val == Some(idx) {
                let nodes = self.hot.nodes.read();
                let mmap_guard = self.cold.mmap_index.read();
                let mmap_node_count = mmap_guard
                    .as_ref()
                    .map(|m| m.header.node_count() as usize)
                    .unwrap_or(0);
                let deleted = self.cold.deleted_nodes.read();

                let mut best_node = None;
                let mut best_ram_node = None;
                let mut max_layer = 0;
                let mut max_ram_layer = 0;

                if let Some(mmap) = mmap_guard.as_ref() {
                    for i in 0..mmap_node_count {
                        if i != idx && !deleted.contains(i as u64) {
                            let record = match mmap.get_node_record(i) {
                                Ok(r) => r,
                                Err(_) => continue,
                            };
                            if record.max_layer as usize >= max_layer {
                                max_layer = record.max_layer as usize;
                                best_node = Some(i);
                            }
                        }
                    }
                }

                for (i, node) in nodes.iter().enumerate() {
                    let global_idx = mmap_node_count + i;
                    if global_idx != idx && !deleted.contains(global_idx as u64) {
                        if node.max_layer >= max_layer {
                            max_layer = node.max_layer;
                            best_node = Some(global_idx);
                        }
                        if node.max_layer >= max_ram_layer {
                            max_ram_layer = node.max_layer;
                            best_ram_node = Some(global_idx);
                        }
                    }
                }

                if ep_val == Some(idx) {
                    self.hot.set_entry_point(best_node);
                    if let Some(new_idx) = best_node {
                        let node_max_layer = if let Some(mmap) = mmap_guard.as_ref() {
                            if new_idx < mmap_node_count {
                                mmap.get_node_record(new_idx)
                                    .map(|r| r.max_layer as usize)
                                    .unwrap_or(0)
                            } else {
                                nodes[new_idx - mmap_node_count].max_layer
                            }
                        } else {
                            nodes[new_idx].max_layer
                        };
                        self.hot
                            .max_layer
                            .store(node_max_layer as u64, Ordering::SeqCst);
                    } else {
                        self.hot.max_layer.store(0, Ordering::SeqCst);
                    }
                }

                if ram_ep_val == Some(idx) {
                    self.hot.set_ram_entry_point(best_ram_node);
                }
            }
        }
        Ok(())
    }

    pub fn deleted_ratio(&self) -> f64 {
        1.0 - self.connectivity_score()
    }

    pub fn rebuild_count(&self) -> u64 {
        self.cold.rebuild_count.load(Ordering::SeqCst)
    }

    pub fn visited_dead_nodes(&self) -> u64 {
        self.cold.visited_dead_nodes.load(Ordering::SeqCst)
    }

    pub fn is_rebuild_required(&self) -> bool {
        if self.connectivity_score() < self.cold.config.rebuild_threshold {
            return true;
        }
        if self.cold.config.quantize {
            if let Some(q) = self.cold.quantizer.read().as_ref() {
                if q.is_rebuild_required(self.cold.config.quantizer_drift_threshold) {
                    return true;
                }
            }
        }
        false
    }

    pub async fn rebuild(&self) -> Result<()> {
        self.rebuild_sync()
    }

    pub fn rebuild_sync(&self) -> Result<()> {
        if self.hot.rebuilding.swap(true, Ordering::SeqCst) {
            tracing::debug!("HNSW rebuild already in progress, skipping");
            return Ok(());
        }
        let _guard = RebuildGuard(&self.hot.rebuilding);

        tracing::info!("Starting HNSW index rebuild (Phase 1)");
        let start_time = std::time::Instant::now();

        let (new_index, snapshot_tx) = self.rebuild_phase1_snapshot_and_build()?;

        tracing::info!("HNSW index rebuild Phase 1 completed, starting Phase 2 merge & swap");

        self.rebuild_phase2_merge_and_swap(new_index, snapshot_tx)?;

        self.cold.rebuild_count.fetch_add(1, Ordering::SeqCst);
        tracing::info!("HNSW rebuild completed in {:?}", start_time.elapsed());
        Ok(())
    }

    pub async fn rebuild_region(&self, region_node_ids: Vec<u64>) -> Result<()> {
        self.rebuild_region_sync(region_node_ids)
    }

    pub fn rebuild_region_sync(&self, region_node_ids: Vec<u64>) -> Result<()> {
        if region_node_ids.is_empty() {
            return Ok(());
        }

        let _write_lock = self.hot.write_mutex.lock();

        let region_set: AHashSet<u64> = region_node_ids.into_iter().collect();

        let mmap_count = self
            .cold
            .mmap_index
            .read()
            .as_ref()
            .map(|m| m.header.node_count() as usize)
            .unwrap_or(0);

        let nodes = self.hot.nodes.read();
        let mut deleted_nodes = self.cold.deleted_nodes.write();

        let mut tombstoned_in_region = Vec::new();
        for &node_id in &region_set {
            if deleted_nodes.contains(node_id) {
                tombstoned_in_region.push(node_id);
            }
        }

        if tombstoned_in_region.is_empty() {
            return Ok(());
        }

        let tombstoned_set: AHashSet<u32> =
            tombstoned_in_region.iter().map(|&id| id as u32).collect();

        let m = self.cold.config.m;
        let offsets = self.hot.arena.offsets.read();
        let count_offsets = self.hot.arena.count_offsets.read();
        let mut counts = self.hot.arena.counts.write();
        let mut arena = self.hot.arena.arena.write();

        for (i, node) in nodes.iter().enumerate() {
            let global_idx = mmap_count + i;
            if deleted_nodes.contains(global_idx as u64) {
                continue;
            }

            if i < offsets.len() && i < count_offsets.len() {
                let node_offset = offsets[i];
                let count_start = count_offsets[i];
                let count_end = if i + 1 < count_offsets.len() {
                    count_offsets[i + 1]
                } else {
                    counts.len()
                };
                for layer in 0..node.max_layer + 1 {
                    if count_start + layer < count_end {
                        let l_offset = HnswArena::layer_offset(node_offset, layer, m);
                        let old_len = counts[count_start + layer] as usize;
                        if l_offset + old_len <= arena.len() {
                            let layer_slice = &arena[l_offset..l_offset + old_len];
                            let mut kept = Vec::with_capacity(old_len);
                            for &neighbor_u32 in layer_slice {
                                if !tombstoned_set.contains(&neighbor_u32) {
                                    kept.push(neighbor_u32);
                                }
                            }
                            counts[count_start + layer] = kept.len() as u8;
                            arena[l_offset..l_offset + kept.len()].copy_from_slice(&kept);
                        }
                    }
                }
            }
        }

        let mut doc_map = self.hot.doc_to_node.write();
        for &ts_id in &tombstoned_in_region {
            if ts_id >= mmap_count as u64 {
                let ram_idx = (ts_id as usize) - mmap_count;
                if let Some(node) = nodes.get(ram_idx) {
                    doc_map.remove(&node.doc_id.inner());
                }
            }
            deleted_nodes.insert(ts_id);
        }

        self.hot
            .deleted_count
            .store(deleted_nodes.len(), Ordering::SeqCst);

        tracing::info!(
            rebuilt_region_nodes = region_set.len(),
            pruned_tombstones = tombstoned_in_region.len(),
            "HNSW local partial rebuild completed successfully"
        );

        Ok(())
    }

    fn rebuild_phase1_snapshot_and_build(&self) -> Result<(HnswIndex, u64)> {
        let (all_nodes, config, snapshot_tx) = {
            let nodes = self.hot.nodes.read();
            let mmap_count = self
                .cold
                .mmap_index
                .read()
                .as_ref()
                .map(|m| m.header.node_count() as usize)
                .unwrap_or(0);
            let deleted_nodes = self.cold.deleted_nodes.read();
            let seq_log = self.cold.seq_log.read();
            let min_retention_seq = seq_log.min_retention_seq();
            let snapshot_tx = self.hot.last_tx_id.load(Ordering::SeqCst);
            let mut all = Vec::with_capacity(nodes.len());
            for (i, node) in nodes.iter().enumerate() {
                let global_idx = mmap_count + i;
                if node.committed_tx <= snapshot_tx {
                    let is_deleted = deleted_nodes.contains(global_idx as u64)
                        && seq_log
                            .deletion_seq(node.doc_id)
                            .is_none_or(|del_seq| del_seq <= snapshot_tx);
                    if !is_deleted {
                        all.push((node.doc_id, node.vector.clone(), node.committed_tx, false));
                    } else if let Some(min_ret_seq) = min_retention_seq {
                        if let Some(del_seq) = seq_log.deletion_seq(node.doc_id) {
                            if del_seq >= min_ret_seq {
                                all.push((
                                    node.doc_id,
                                    node.vector.clone(),
                                    node.committed_tx,
                                    true,
                                ));
                            }
                        }
                    }
                }
            }
            (all, self.cold.config.clone(), snapshot_tx)
        };

        let new_index = HnswIndex::try_new(config)?;
        *new_index.inner.cold.seq_log.write() = self.cold.seq_log.read().clone();

        {
            let mmap_guard = self.cold.mmap_index.read();
            if let Some(mmap) = mmap_guard.as_ref() {
                new_index.load_mmap_from_instance(mmap.clone())?;
            }
        }

        let quantizer_guard = self.cold.quantizer.read();
        if let Some(old_q) = quantizer_guard.as_ref() {
            let sample_size = self
                .cold
                .config
                .quantizer_recalibration_sample_size
                .min(all_nodes.len());
            let mut train_data = Vec::with_capacity(sample_size);

            for (_, vector, _, is_deleted) in all_nodes.iter() {
                if !is_deleted {
                    match vector {
                        VectorData::F32(v) => train_data.push(v.clone()),
                        VectorData::U8(v) => train_data.push(old_q.dequantize(v)?),
                    }
                    if train_data.len() >= sample_size {
                        break;
                    }
                }
            }

            if !train_data.is_empty() {
                let training_refs: Vec<&[f32]> = train_data.iter().map(|v| v.as_slice()).collect();
                let new_q = crate::quantize::ScalarQuantizer::train(
                    &training_refs,
                    self.cold.config.dimension,
                );
                let bias =
                    Sq8Bias::calibrate(&training_refs, &new_q, self.cold.config.distance_metric);
                *new_index.inner.cold.quantizer.write() = Some(new_q);
                *new_index.inner.cold.sq8_bias.write() = bias;
            } else {
                *new_index.inner.cold.quantizer.write() = Some(old_q.clone());
            }
        }

        for (doc_id, vector, committed_tx, is_deleted) in all_nodes {
            match vector {
                VectorData::F32(v) => {
                    new_index.inner.do_insert(doc_id, &v, committed_tx)?;
                }
                VectorData::U8(v) => {
                    let dequantized = {
                        let q = quantizer_guard.as_ref().ok_or_else(|| {
                            ContextraError::Index("Quantizer missing during rebuild".into())
                        })?;
                        q.dequantize(&v)?
                    };
                    new_index
                        .inner
                        .do_insert(doc_id, &dequantized, committed_tx)?;
                }
            }
            if is_deleted {
                new_index.inner.do_delete(doc_id)?;
            }
        }

        Ok((new_index, snapshot_tx))
    }

    fn rebuild_phase2_merge_and_swap(&self, new_index: HnswIndex, snapshot_tx: u64) -> Result<()> {
        let _write_lock = self.hot.write_mutex.lock();

        let delta_changes = self.cold.seq_log.read().changes_since(snapshot_tx);

        for change in delta_changes {
            match change {
                contextra_core::SeqLogChange::Insert { doc_id, seq } => {
                    let vector_opt = {
                        let doc_map = self.hot.doc_to_node.read();
                        let mmap_count = self
                            .cold
                            .mmap_index
                            .read()
                            .as_ref()
                            .map(|m| m.header.node_count() as usize)
                            .unwrap_or(0);
                        if let Some(&global_idx) = doc_map.get(&doc_id.inner()) {
                            if global_idx >= mmap_count {
                                let ram_idx = global_idx - mmap_count;
                                let nodes = self.hot.nodes.read();
                                nodes.get(ram_idx).map(|n| n.vector.clone())
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    };

                    if let Some(vector) = vector_opt {
                        let f32_vec = match vector {
                            VectorData::F32(v) => v,
                            VectorData::U8(v) => {
                                let q_guard = self.cold.quantizer.read();
                                let q = q_guard.as_ref().ok_or_else(|| {
                                    ContextraError::Index(
                                        "Quantizer missing during rebuild phase 2 delta replay"
                                            .into(),
                                    )
                                })?;
                                q.dequantize(&v)?
                            }
                        };

                        new_index.inner.do_insert(doc_id, &f32_vec, seq)?;
                    }
                }
                contextra_core::SeqLogChange::Delete { doc_id, .. } => {
                    new_index.inner.do_delete(doc_id)?;
                }
            }
        }

        {
            let mut nodes = self.hot.nodes.write();
            let mut doc_to_node = self.hot.doc_to_node.write();
            let mut deleted_nodes = self.cold.deleted_nodes.write();
            let mut offsets = self.hot.arena.offsets.write();
            let mut capacities = self.hot.arena.capacities.write();
            let mut count_offsets = self.hot.arena.count_offsets.write();
            let mut counts = self.hot.arena.counts.write();
            let mut arena = self.hot.arena.arena.write();

            let new_nodes = std::mem::take(&mut *new_index.inner.hot.nodes.write());
            let new_doc_to_node = std::mem::take(&mut *new_index.inner.hot.doc_to_node.write());
            let new_entry_point = new_index.inner.hot.get_entry_point();
            let new_ram_entry_point = new_index.inner.hot.get_ram_entry_point();

            let new_offsets = std::mem::take(&mut *new_index.inner.hot.arena.offsets.write());
            let new_capacities = std::mem::take(&mut *new_index.inner.hot.arena.capacities.write());
            let new_count_offsets =
                std::mem::take(&mut *new_index.inner.hot.arena.count_offsets.write());
            let new_counts = std::mem::take(&mut *new_index.inner.hot.arena.counts.write());
            let new_arena = std::mem::take(&mut *new_index.inner.hot.arena.arena.write());

            let new_quantizer = new_index.inner.cold.quantizer.write().take();
            if new_quantizer.is_some() {
                *self.cold.quantizer.write() = new_quantizer;
            }

            *nodes = new_nodes;
            *doc_to_node = new_doc_to_node;
            self.hot.set_entry_point(new_entry_point);
            self.hot.set_ram_entry_point(new_ram_entry_point);
            *offsets = new_offsets;
            *capacities = new_capacities;
            *count_offsets = new_count_offsets;
            *counts = new_counts;
            *arena = new_arena;

            self.hot.max_layer.store(
                new_index.inner.hot.max_layer.load(Ordering::SeqCst),
                Ordering::SeqCst,
            );

            let mmap_count = self
                .cold
                .mmap_index
                .read()
                .as_ref()
                .map(|m| m.header.node_count() as usize)
                .unwrap_or(0);
            let mut new_deleted = RoaringTreemap::new();
            for del_idx in deleted_nodes.iter() {
                if (del_idx as usize) < mmap_count {
                    new_deleted.insert(del_idx);
                }
            }
            for del_idx in new_index.inner.cold.deleted_nodes.read().iter() {
                new_deleted.insert(del_idx);
            }

            *deleted_nodes = new_deleted;
            self.hot
                .deleted_count
                .store(deleted_nodes.len(), Ordering::SeqCst);
        }

        Ok(())
    }
}
