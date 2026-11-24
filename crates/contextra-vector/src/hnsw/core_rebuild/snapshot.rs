// FILE-CONTEXT
// ZWECK: Phase-1 Rebuild: Snapshot des Index nehmen, Quantisierer re-kalibrieren und neuen Kern aufbauen.
// INVARIANTEN: Deterministische Nachbarschaftsauswahl und Distanzberechnung; INV-INDEX-APPLY-1.
// HOTSPOTS: rebuild_phase1_snapshot_and_build, select_neighbors_heuristic_with_batch
// STAND: TS:2026-11-11T02:00:00Z

use std::sync::atomic::Ordering;

use crate::distance::compute_distance_trusted;
use crate::error::{Result, VectorError};
use crate::hnsw::batch::{PreparedInsert, SearchContext};
use crate::hnsw::sq8_bias::Sq8Bias;
use crate::hnsw::types::{Candidate, HnswIndex, HnswIndexCore, VectorData};
use crate::types::DocId;

impl HnswIndexCore {
    /// Weist einem Knoten eine Zufallsebene basierend auf geometrischer Verteilung zu.
    pub(crate) fn random_layer(&self) -> usize {
        let r = self.cold.rng.next_unit_f64() as f32;
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
                        crate::error::VectorError::Index("Quantizer not trained".into())
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
                    crate::error::VectorError::Index("Quantizer not trained".into())
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
                        crate::error::VectorError::Index("Quantizer not trained".into())
                    })?
                    .symmetric_dist(a, b, self.cold.config.distance_metric)
            }
            _ => Err(VectorError::Index(
                "Mixed vector representations (F32/U8) are not supported".into(),
            )),
        }
    }

    pub(crate) fn resolve_dist(
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

    pub(crate) fn resolve_doc_id(&self, idx: usize, ctx: &SearchContext) -> Result<DocId> {
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
        Err(VectorError::Index(format!(
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

        let base_batch_idx = ctx.mmap_node_count + ctx.nodes.len();
        if idx >= base_batch_idx {
            let prepared_offset = idx - base_batch_idx;
            if let Some(prepared) = prior_prepared.get(prepared_offset) {
                return Ok(prepared.vector_data.clone());
            }
        }

        if let Some(mmap) = ctx.mmap {
            if idx < ctx.mmap_node_count {
                let record = mmap.get_node_record(idx)?;
                let vector_bytes = mmap.get_vector(&record)?;
                if mmap.header.is_quantized() {
                    return Ok(VectorData::U8(vector_bytes.to_vec()));
                } else {
                    let mut floats = vec![0.0f32; vector_bytes.len() / 4];
                    for (chunk, f) in vector_bytes.chunks_exact(4).zip(floats.iter_mut()) {
                        let mut arr = [0u8; 4];
                        arr.copy_from_slice(chunk);
                        *f = f32::from_le_bytes(arr);
                    }
                    return Ok(VectorData::F32(floats));
                }
            }
            return Ok(ctx.nodes[idx - ctx.mmap_node_count].vector.clone());
        }

        if idx < ctx.nodes.len() {
            return Ok(ctx.nodes[idx].vector.clone());
        }

        Err(VectorError::Index(format!(
            "Invalid node index {} in resolve_vector_data_with_batch",
            idx
        )))
    }

    pub(crate) fn compute_symmetric_distance_hybrid_with_batch(
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

    pub(crate) fn select_neighbors_heuristic_with_batch(
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
        sorted_cands.sort_by(|a, b| {
            a.distance
                .total_cmp(&b.distance)
                .then_with(|| a.index.cmp(&b.index))
        });

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
            sorted_fallback.sort_by(|a, b| {
                a.distance
                    .total_cmp(&b.distance)
                    .then_with(|| a.index.cmp(&b.index))
            });

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

    // ANCHOR[contextra-vector:REBUILD-PHASE1]
    pub(super) fn rebuild_phase1_snapshot_and_build(&self) -> Result<(HnswIndex, u64)> {
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

        let rng = self.cold.rng.clone();
        let clock = self.cold.clock.read().clone();
        let new_index = HnswIndex::try_new_with_rng_and_clock(config, rng, clock)?;
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
                            VectorError::Index("Quantizer missing during rebuild".into())
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
}
