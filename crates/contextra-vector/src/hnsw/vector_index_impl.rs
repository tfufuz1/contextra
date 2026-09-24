use std::borrow::Cow;
use std::sync::atomic::Ordering;

use contextra_core::{
    ContextraError, DistanceMetric, DocId, IndexOp, Result, ScoredDocument, TxId, VectorIndex, VectorIndexStats,
};

use super::batch::{BatchContext, SearchContext};
use super::config::validate_vector;
use super::sq8_bias::Sq8Bias;
use super::types::{HnswIndex, HnswNode, SnapshotPinGuard, VectorData};

impl VectorIndex for HnswIndex {
    async fn insert(&self, tx: TxId, id: DocId, embedding: &[f32]) -> Result<()> {
        if let Some(ref err) = self.inner.cold.validation_error {
            return Err(ContextraError::invalid_input(format!(
                "Invalid index configuration: {}",
                err
            )));
        }
        if embedding.len() != self.inner.cold.config.dimension {
            return Err(ContextraError::invalid_input(format!(
                "Expected dimension {}, got {}",
                self.inner.cold.config.dimension,
                embedding.len()
            )));
        }

        validate_vector(embedding)?;

        self.inner.cold.tx_buffer.stage(
            tx,
            IndexOp::Insert {
                doc_id: id,
                data: embedding.to_vec(),
            },
        )?;
        Ok(())
    }

    async fn search(&self, query: &[f32], k: usize) -> Result<Vec<ScoredDocument>> {
        if k > contextra_core::MAX_SEARCH_K {
            return Err(ContextraError::invalid_input(format!(
                "Requested k ({}) exceeds maximum allowed search limit ({})",
                k,
                contextra_core::MAX_SEARCH_K
            )));
        }
        if let Some(ref err) = self.inner.cold.validation_error {
            return Err(ContextraError::invalid_input(format!(
                "Invalid index configuration: {}",
                err
            )));
        }
        if query.len() != self.inner.cold.config.dimension {
            return Err(ContextraError::invalid_input(format!(
                "Expected dimension {}, got {}",
                self.inner.cold.config.dimension,
                query.len()
            )));
        }

        for &val in query {
            if !val.is_finite() {
                return Err(ContextraError::invalid_input(
                    "Query vector contains NaN or infinite values",
                ));
            }
        }

        let query_quantized: Option<Vec<u8>> = None;

        let mmap_guard = self.inner.cold.mmap_index.read();
        let mmap_node_count = mmap_guard
            .as_ref()
            .map(|m| m.header.node_count() as usize)
            .unwrap_or(0);

        let mut ep = Vec::new();
        if let Some(global_ep) = self.inner.hot.get_entry_point() {
            ep.push(global_ep);
        }
        if let Some(ram_ep) = self.inner.hot.get_ram_entry_point() {
            if !ep.contains(&ram_ep) {
                ep.push(ram_ep);
            }
        }

        if ep.is_empty() {
            let nodes = self.inner.hot.nodes.read();
            let deleted = self.inner.cold.deleted_nodes.read();
            let mmap_guard = self.inner.cold.mmap_index.read();
            let mmap_node_count = mmap_guard
                .as_ref()
                .map(|m| m.header.node_count() as usize)
                .unwrap_or(0);
            let total = mmap_node_count + nodes.len();
            for i in 0..total {
                if !deleted.contains(i as u64) {
                    ep.push(i);
                    self.inner.hot.set_entry_point(Some(i));
                    break;
                }
            }
        }

        if ep.is_empty() {
            return Ok(Vec::new());
        }

        let max_layer = self.inner.hot.max_layer.load(Ordering::SeqCst) as usize;

        for layer in (1..=max_layer).rev() {
            let layer_ef = 1;
            let best =
                self.inner
                    .search_layer(query, query_quantized.as_deref(), &ep, layer_ef, layer)?;
            if let Some(closest) = best.first() {
                ep = vec![closest.index];
            }
        }

        if let Some(ram_ep) = self.inner.hot.get_ram_entry_point() {
            if !ep.contains(&ram_ep) {
                ep.push(ram_ep);
            }
        }

        let ef = if self.inner.cold.config.quantize {
            self.inner.cold.config.ef_search.max(k) * 4
        } else {
            self.inner.cold.config.ef_search.max(k)
        };
        let candidates = self
            .inner
            .search_layer(query, query_quantized.as_deref(), &ep, ef, 0)?;

        let score = self.inner.connectivity_score();
        if score < self.inner.cold.config.rebuild_threshold {
            let deleted_ratio = (1.0 - score) * 100.0;
            let err = contextra_core::ContextraError::HnswConnectivityDegraded { deleted_ratio };
            tracing::warn!(
                error = %err,
                connectivity_score = score,
                rebuild_threshold = self.inner.cold.config.rebuild_threshold,
                "HNSW index degraded — consider calling rebuild()"
            );
        }

        let nodes = self.inner.hot.nodes.read();
        let deleted = self.inner.cold.deleted_nodes.read();
        let mut results = Vec::with_capacity(k);

        let q_guard = if self.inner.cold.config.quantize {
            Some(self.inner.cold.quantizer.read())
        } else {
            None
        };
        let q_ref = q_guard.as_ref().and_then(|g| g.as_ref());

        let ctx = SearchContext {
            nodes: &nodes,
            mmap: mmap_guard.as_ref(),
            mmap_node_count,
            prior_prepared: &[],
            backlink_map: None,
            quantizer: q_ref.map(Cow::Borrowed),
            arena: &self.inner.hot.arena,
        };

        for c in candidates.iter() {
            if deleted.contains(c.index as u64) {
                continue;
            }
            if c.index >= mmap_node_count {
                if let Some(node) = nodes.get(c.index - mmap_node_count) {
                    if node.committed_tx == 0 {
                        continue;
                    }
                }
            }
            let doc_id = self.inner.resolve_doc_id(c.index, &ctx)?;

            let final_dist = if self.inner.cold.config.quantize {
                self.inner.resolve_dist(c.index, query, None, &ctx)?
            } else {
                c.distance
            };

            let score = match self.inner.cold.config.distance_metric {
                DistanceMetric::Cosine => 1.0 - final_dist,
                DistanceMetric::Euclidean => 1.0 / (1.0 + final_dist),
                DistanceMetric::DotProduct => -final_dist,
                other => {
                    return Err(ContextraError::Index(format!(
                        "Unsupported DistanceMetric variant in search(): {other:?}"
                    )));
                }
            };
            results.push(ScoredDocument::new(doc_id, score));
        }

        if results.len() > k {
            results.select_nth_unstable_by(k - 1, |a, b| {
                b.score
                    .total_cmp(&a.score)
                    .then_with(|| a.doc_id.cmp(&b.doc_id))
            });
            results.truncate(k);
        }
        results.sort_by(|a, b| {
            b.score
                .total_cmp(&a.score)
                .then_with(|| a.doc_id.cmp(&b.doc_id))
        });

        Ok(results)
    }

    async fn search_filtered(
        &self,
        query: &[f32],
        k: usize,
        filter: Option<&(dyn Fn(DocId) -> bool + Send + Sync)>,
    ) -> Result<Vec<ScoredDocument>> {
        self.search_filtered_internal(query, k, filter, None).await
    }

    async fn delete(&self, tx: TxId, id: DocId) -> Result<()> {
        if let Some(ref err) = self.inner.cold.validation_error {
            return Err(ContextraError::invalid_input(format!(
                "Invalid index configuration: {}",
                err
            )));
        }
        self.inner.cold.tx_buffer.stage(
            tx,
            IndexOp::Delete {
                doc_id: id,
                data: None,
            },
        )?;
        Ok(())
    }

    async fn commit(&self, tx: TxId) -> Result<()> {
        if let Some(ref err) = self.inner.cold.validation_error {
            return Err(ContextraError::invalid_input(format!(
                "Invalid index configuration: {}",
                err
            )));
        }
        let _lock = self.inner.hot.write_mutex.lock();
        let ops = self.inner.cold.tx_buffer.drain(tx);

        if self.inner.cold.config.quantize && self.inner.cold.quantizer.read().is_none() {
            let mut train_data = Vec::with_capacity(256.min(ops.len()));
            for op in &ops {
                if let IndexOp::Insert { data, .. } = op {
                    train_data.push(data.clone());
                    if train_data.len() >= 256 {
                        break;
                    }
                }
            }

            if train_data.len() < 256 {
                let nodes = self.inner.hot.nodes.read();
                for node in nodes.iter() {
                    if let VectorData::F32(v) = &node.vector {
                        train_data.push(v.clone());
                        if train_data.len() >= 256 {
                            break;
                        }
                    }
                }
            }

            if train_data.len() >= 50 {
                let training_refs: Vec<&[f32]> = train_data.iter().map(|v| v.as_slice()).collect();
                let q = crate::quantize::ScalarQuantizer::train(
                    &training_refs,
                    self.inner.cold.config.dimension,
                );
                let bias =
                    Sq8Bias::calibrate(&training_refs, &q, self.inner.cold.config.distance_metric);
                *self.inner.cold.quantizer.write() = Some(q.clone());
                *self.inner.cold.sq8_bias.write() = bias;

                let mut nodes = self.inner.hot.nodes.write();
                for node in nodes.iter_mut() {
                    if let VectorData::F32(v) = &node.vector {
                        node.vector = VectorData::U8(q.quantize(v)?);
                    }
                }
            }
        }

        let mut prepared_inserts = Vec::new();
        let mut deletes_to_apply = Vec::new();
        let mut batch_ctx = BatchContext::new(&self.inner);

        for op in &ops {
            match op {
                IndexOp::Insert { doc_id, data } => {
                    let prepared = self.inner.compute_insert_with_context(
                        *doc_id,
                        data,
                        prepared_inserts.len(),
                        &mut prepared_inserts,
                        &mut batch_ctx,
                    )?;
                    prepared_inserts.push(prepared);
                }
                IndexOp::Delete { doc_id, .. } => {
                    deletes_to_apply.push(*doc_id);
                }
                other => {
                    return Err(ContextraError::Index(format!(
                        "HNSW commit received unsupported IndexOp variant: {:?}. \
                         Add a handler arm before enabling this operation.",
                        std::mem::discriminant(other)
                    )));
                }
            }
        }

        for doc_id in deletes_to_apply {
            self.inner.do_delete(doc_id)?;
        }

        let seq = tx.inner();
        let mut inserted_doc_ids = Vec::with_capacity(prepared_inserts.len());
        for prepared in prepared_inserts {
            inserted_doc_ids.push(prepared.doc_id);
            self.inner.apply_insert(prepared, seq);
        }

        let mut seq_log = self.inner.cold.seq_log.write();
        for op in &ops {
            match op {
                IndexOp::Insert { doc_id, .. } => {
                    seq_log.record_insert(*doc_id, seq);
                }
                IndexOp::Delete { doc_id, .. } => {
                    seq_log.record_delete(*doc_id, seq);
                }
                _ => {}
            }
        }
        drop(seq_log);

        if self.inner.is_rebuild_required() {
            tracing::warn!(
                "HNSW index rebuild threshold reached (rebuild_threshold: {:.2}, quantizer_drift_threshold: {:.2})",
                self.inner.cold.config.rebuild_threshold,
                self.inner.cold.config.quantizer_drift_threshold
            );
            self.trigger_rebuild_async();
        }

        #[cfg(feature = "partial-index-rebuild")]
        self.check_and_trigger_partial_rebuild();

        self.inner
            .hot
            .last_tx_id
            .fetch_max(tx.inner(), Ordering::SeqCst);
        Ok(())
    }

    async fn search_at(&self, query: &[f32], k: usize, seq_no: u64) -> Result<Vec<ScoredDocument>> {
        let _pin_guard = SnapshotPinGuard::new(&self.inner, seq_no);
        let log = self.inner.cold.seq_log.read().clone();
        let filter_fn = move |doc_id: DocId| -> bool { log.is_visible(doc_id, seq_no) };
        self.search_filtered_internal(query, k, Some(&filter_fn), Some(seq_no))
            .await
    }

    async fn rollback(&self, tx: TxId) -> Result<()> {
        if let Some(ref err) = self.inner.cold.validation_error {
            return Err(ContextraError::invalid_input(format!(
                "Invalid index configuration: {}",
                err
            )));
        }
        self.inner.cold.tx_buffer.discard(tx);
        Ok(())
    }

    async fn rollback_to_tx(&self, tx_id: TxId) -> Result<()> {
        if let Some(ref err) = self.inner.cold.validation_error {
            return Err(ContextraError::invalid_input(format!(
                "Invalid index configuration: {}",
                err
            )));
        }

        let target = tx_id.inner();

        let _guard = self.inner.hot.write_mutex.lock();

        let indices_to_remove: Vec<usize> = {
            let nodes = self.inner.hot.nodes.read();
            nodes
                .iter()
                .enumerate()
                .filter(|(_, node)| node.committed_tx > target && node.committed_tx != 0)
                .map(|(i, _)| i)
                .collect()
        };

        if indices_to_remove.is_empty() {
            self.inner.hot.last_tx_id.store(target, Ordering::SeqCst);
            return Ok(());
        }

        {
            let nodes = self.inner.hot.nodes.read();
            let mut map = self.inner.hot.doc_to_node.write();
            for &idx in &indices_to_remove {
                if let Some(node) = nodes.get(idx) {
                    map.remove(&node.doc_id.inner());
                }
            }
        }

        {
            let mmap_count = self
                .inner
                .cold
                .mmap_index
                .read()
                .as_ref()
                .map(|m| m.header.node_count() as usize)
                .unwrap_or(0);
            let mut deleted = self.inner.cold.deleted_nodes.write();
            for &idx in &indices_to_remove {
                deleted.insert((mmap_count + idx) as u64);
            }
            self.inner
                .hot
                .deleted_count
                .fetch_add(indices_to_remove.len() as u64, Ordering::SeqCst);
        }

        self.inner.cold.tx_buffer.discard(tx_id);
        self.inner.hot.last_tx_id.store(target, Ordering::SeqCst);

        tracing::info!(
            removed = indices_to_remove.len(),
            rollback_target = target,
            "HNSW physical rollback completed"
        );

        Ok(())
    }

    async fn last_tx_id(&self) -> Result<TxId> {
        Ok(TxId::new(self.inner.hot.last_tx_id.load(Ordering::SeqCst)))
    }

    async fn all_doc_ids(&self) -> Result<Vec<DocId>> {
        if self.inner.cold.validation_error.is_some() {
            return Ok(Vec::new());
        }
        let nodes = self.inner.hot.nodes.read();
        let mmap_guard = self.inner.cold.mmap_index.read();
        let mmap_node_count = mmap_guard
            .as_ref()
            .map(|m| m.header.node_count() as usize)
            .unwrap_or(0);
        let deleted = self.inner.cold.deleted_nodes.read();

        let ctx = SearchContext {
            nodes: &nodes,
            mmap: mmap_guard.as_ref(),
            mmap_node_count,
            prior_prepared: &[],
            backlink_map: None,
            quantizer: None,
            arena: &self.inner.hot.arena,
        };

        let total_nodes = mmap_node_count + nodes.len();
        let mut ids = Vec::with_capacity(total_nodes.saturating_sub(deleted.len() as usize));

        for i in 0..total_nodes {
            if !deleted.contains(i as u64) {
                ids.push(self.inner.resolve_doc_id(i, &ctx)?);
            }
        }
        Ok(ids)
    }

    async fn len(&self) -> usize {
        if self.inner.cold.validation_error.is_some() {
            return 0;
        }
        let mmap_count = self
            .inner
            .cold
            .mmap_index
            .read()
            .as_ref()
            .map(|m| m.header.node_count() as usize)
            .unwrap_or(0);
        let total = mmap_count + self.inner.hot.nodes.read().len();
        let deleted = self.inner.hot.deleted_count.load(Ordering::SeqCst) as usize;
        total.saturating_sub(deleted)
    }

    fn is_rebuild_required(&self) -> bool {
        self.inner.is_rebuild_required()
    }

    fn trigger_rebuild_async(&self) {
        self.trigger_rebuild_async();
    }

    async fn stats(&self) -> Result<VectorIndexStats> {
        let nodes = self.inner.hot.nodes.read();
        let mmap_guard = self.inner.cold.mmap_index.read();
        let mmap_count = mmap_guard
            .as_ref()
            .map(|m| m.header.node_count() as usize)
            .unwrap_or(0);

        let deleted_count = self.inner.hot.deleted_count.load(Ordering::SeqCst) as usize;
        let num_vectors = (mmap_count + nodes.len()).saturating_sub(deleted_count);

        let mut vector_memory: usize = nodes
            .iter()
            .map(|n| match &n.vector {
                VectorData::F32(v) => v.len() * std::mem::size_of::<f32>(),
                VectorData::U8(v) => v.len() * std::mem::size_of::<u8>(),
            })
            .sum();

        let connection_memory: usize =
            self.inner.hot.arena.arena.read().len() * std::mem::size_of::<u32>();

        if let Some(mmap) = mmap_guard.as_ref() {
            vector_memory += mmap.mmap.len();
        }

        Ok(VectorIndexStats {
            num_vectors,
            memory_usage_bytes: vector_memory
                + connection_memory
                + (nodes.len() * std::mem::size_of::<HnswNode>()),
            num_layers: self.inner.hot.max_layer.load(Ordering::SeqCst) as usize + 1,
            deleted_ratio: self.inner.deleted_ratio(),
            rebuild_count: self.inner.rebuild_count(),
        })
    }
}
