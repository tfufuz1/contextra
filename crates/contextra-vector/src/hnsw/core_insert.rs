use std::borrow::Cow;
use std::sync::atomic::Ordering;

use contextra_core::{ContextraError, DocId, Result};

use super::batch::{get_neighbor_conns_in_batch, BatchContext, NeighborBacklink, PreparedInsert, SearchContext};
use super::config::validate_vector;
use super::types::{Candidate, HnswIndexCore, HnswNode, VectorData};

impl HnswIndexCore {
    pub(super) fn resolve_connections<'a>(
        &self,
        idx: usize,
        layer: usize,
        ctx: &'a SearchContext,
    ) -> Result<Cow<'a, [u32]>> {
        let base_batch_idx = ctx.mmap_node_count + ctx.nodes.len();
        if idx >= base_batch_idx {
            let prepared_offset = idx - base_batch_idx;
            if let Some(prepared) = ctx.prior_prepared.get(prepared_offset) {
                let conns = prepared
                    .final_connections
                    .get(layer)
                    .cloned()
                    .unwrap_or_default();
                return Ok(Cow::Owned(conns));
            }
        }

        if let Some(mmap) = ctx.mmap {
            if idx < ctx.mmap_node_count {
                let record = mmap.get_node_record(idx)?;
                return Ok(Cow::Owned(mmap.get_connections(&record, layer)?));
            }
            let ram_idx = idx - ctx.mmap_node_count;
            if let Some(bmap) = ctx.backlink_map {
                if let Some(conns) = bmap.get(ram_idx, layer) {
                    return Ok(Cow::Owned(conns.clone()));
                }
            }
            let conns = ctx
                .arena
                .get_ram_node_connections(ram_idx, layer, self.cold.config.m);
            return Ok(Cow::Owned(conns));
        }

        if let Some(bmap) = ctx.backlink_map {
            if let Some(conns) = bmap.get(idx, layer) {
                return Ok(Cow::Owned(conns.clone()));
            }
        }
        let conns = ctx
            .arena
            .get_ram_node_connections(idx, layer, self.cold.config.m);
        Ok(Cow::Owned(conns))
    }

    pub fn compute_insert(&self, id: DocId, vector: &[f32]) -> Result<PreparedInsert> {
        let mut batch_ctx = BatchContext::new(self);
        self.compute_insert_with_context(id, vector, 0, &mut [], &mut batch_ctx)
    }

    pub fn compute_insert_with_context(
        &self,
        id: DocId,
        vector: &[f32],
        offset: usize,
        prior_prepared: &mut [PreparedInsert],
        batch_ctx: &mut BatchContext,
    ) -> Result<PreparedInsert> {
        #[cfg(test)]
        {
            let target = self
                .cold
                .fault_injection_insert_target
                .load(Ordering::SeqCst);
            if target > 0 {
                let current = self
                    .cold
                    .fault_injection_insert_count
                    .fetch_add(1, Ordering::SeqCst)
                    + 1;
                if current == target {
                    return Err(ContextraError::Index(
                        "Fault injection: compute_insert simulated failure".into(),
                    ));
                }
            }
        }

        if vector.len() != self.cold.config.dimension {
            return Err(ContextraError::invalid_input(format!(
                "Dimension mismatch: expected {}, got {}",
                self.cold.config.dimension,
                vector.len()
            )));
        }

        validate_vector(vector)?;

        let vector_data = if self.cold.config.quantize {
            let q_guard = self.cold.quantizer.read();
            if let Some(q) = q_guard.as_ref() {
                VectorData::U8(q.quantize(vector)?)
            } else {
                VectorData::F32(vector.to_vec())
            }
        } else {
            VectorData::F32(vector.to_vec())
        };

        let new_layer = self.random_layer();
        let entry_point_opt = self.hot.get_entry_point();

        let mmap_node_count = self
            .cold
            .mmap_index
            .read()
            .as_ref()
            .map(|m| m.header.node_count() as usize)
            .unwrap_or(0);

        let nodes_read = self.hot.nodes.read();
        let ram_nodes_count = nodes_read.len();
        let new_idx = mmap_node_count + ram_nodes_count + offset;

        let query_quantized: Option<Vec<u8>> = None;

        let mmap_guard = self.cold.mmap_index.read();

        let mut ep = Vec::new();
        let mut batch_global_ep = None;
        let mut batch_ram_ep = None;

        for prepared in &*prior_prepared {
            if prepared.should_update_entry_point {
                batch_global_ep = Some(prepared.new_idx);
            }
            if prepared.should_update_ram_entry_point {
                batch_ram_ep = Some(prepared.new_idx);
            }
        }

        if let Some(b_ep) = batch_global_ep {
            ep.push(b_ep);
        } else if let Some(global_ep) = entry_point_opt {
            ep.push(global_ep);
        }

        if let Some(b_ram_ep) = batch_ram_ep {
            if !ep.contains(&b_ram_ep) {
                ep.push(b_ram_ep);
            }
        } else if let Some(ram_ep) = self.hot.get_ram_entry_point() {
            if !ep.contains(&ram_ep) {
                ep.push(ram_ep);
            }
        }

        if ep.is_empty() {
            let should_update_entry_point =
                !batch_ctx.has_entry_point || new_layer > batch_ctx.running_max_layer;
            let should_update_ram_entry_point =
                !batch_ctx.has_ram_entry_point || new_layer > batch_ctx.running_max_layer;

            if should_update_entry_point {
                batch_ctx.running_max_layer = new_layer;
                batch_ctx.has_entry_point = true;
            }
            if should_update_ram_entry_point {
                batch_ctx.has_ram_entry_point = true;
            }

            return Ok(PreparedInsert {
                doc_id: id,
                vector_data,
                new_layer,
                new_idx,
                final_connections: vec![vec![]; new_layer + 1],
                neighbor_backlinks: Vec::new(),
                should_update_entry_point,
                should_update_ram_entry_point,
            });
        }

        let current_max_layer = batch_ctx.running_max_layer;

        for layer in (new_layer + 1..=current_max_layer).rev() {
            let best = self.search_layer_with_context(
                vector,
                query_quantized.as_deref(),
                &ep,
                1,
                layer,
                prior_prepared,
                Some(&batch_ctx.backlink_map),
            )?;
            if let Some(closest) = best.first() {
                ep = vec![closest.index];
            }
        }

        let mut final_connections = vec![vec![]; new_layer + 1];

        for layer in (0..=new_layer.min(current_max_layer)).rev() {
            let ram_ep_candidate = batch_ram_ep.or_else(|| self.hot.get_ram_entry_point());
            if let Some(ram_ep) = ram_ep_candidate {
                if !ep.contains(&ram_ep) {
                    ep.push(ram_ep);
                }
            }

            let neighbors = self.search_layer_with_context(
                vector,
                query_quantized.as_deref(),
                &ep,
                self.cold.config.ef_construction,
                layer,
                prior_prepared,
                Some(&batch_ctx.backlink_map),
            )?;
            let ctx = SearchContext {
                nodes: &nodes_read,
                mmap: mmap_guard.as_ref(),
                mmap_node_count,
                prior_prepared,
                backlink_map: Some(&batch_ctx.backlink_map),
                quantizer: None,
                arena: &self.hot.arena,
            };
            let selected = self.select_neighbors_heuristic_with_batch(
                &ctx,
                &neighbors,
                self.cold.config.m,
                new_idx,
                &vector_data,
                prior_prepared,
            )?;
            final_connections[layer] = selected;
            ep = neighbors.iter().map(|c| c.index).collect();
        }

        let base_batch_idx = mmap_node_count + ram_nodes_count;
        let mut neighbor_backlinks = Vec::new();

        for layer in (0..=new_layer.min(current_max_layer)).rev() {
            for &ni in &final_connections[layer] {
                let neighbor_idx = ni as usize;

                if neighbor_idx < mmap_node_count {
                    continue;
                }

                let existing_conns = get_neighbor_conns_in_batch(
                    self,
                    neighbor_idx,
                    layer,
                    base_batch_idx,
                    mmap_node_count,
                    &nodes_read,
                    prior_prepared,
                    batch_ctx,
                );

                let mut conn_indices = existing_conns;
                if !conn_indices.contains(&(new_idx as u32)) {
                    conn_indices.push(new_idx as u32);
                }

                let selected = if conn_indices.len() > self.cold.config.m * 2 {
                    let mut conn_cands = Vec::with_capacity(conn_indices.len());
                    for &idx_u32 in &conn_indices {
                        let idx = idx_u32 as usize;
                        let ctx = SearchContext {
                            nodes: &nodes_read,
                            mmap: mmap_guard.as_ref(),
                            mmap_node_count,
                            prior_prepared,
                            backlink_map: Some(&batch_ctx.backlink_map),
                            quantizer: None,
                            arena: &self.hot.arena,
                        };
                        let dist = self.compute_symmetric_distance_hybrid_with_batch(
                            idx,
                            neighbor_idx,
                            &ctx,
                            new_idx,
                            &vector_data,
                            prior_prepared,
                        )?;
                        conn_cands.push(Candidate {
                            index: idx,
                            distance: dist,
                        });
                    }

                    let ctx = SearchContext {
                        nodes: &nodes_read,
                        mmap: mmap_guard.as_ref(),
                        mmap_node_count,
                        prior_prepared,
                        backlink_map: Some(&batch_ctx.backlink_map),
                        quantizer: None,
                        arena: &self.hot.arena,
                    };
                    self.select_neighbors_heuristic_with_batch(
                        &ctx,
                        &conn_cands,
                        self.cold.config.m * 2,
                        new_idx,
                        &vector_data,
                        prior_prepared,
                    )?
                } else {
                    conn_indices
                };

                if neighbor_idx >= base_batch_idx {
                    let prepared_offset = neighbor_idx - base_batch_idx;
                    if let Some(prep) = prior_prepared.get_mut(prepared_offset) {
                        if prep.final_connections.len() > layer {
                            prep.final_connections[layer] = selected;
                        }
                    }
                } else {
                    let neighbor_ram_idx = neighbor_idx - mmap_node_count;
                    batch_ctx
                        .backlink_map
                        .insert(neighbor_ram_idx, layer, selected.clone());
                    neighbor_backlinks.push(NeighborBacklink {
                        neighbor_ram_idx,
                        layer,
                        updated_connections: selected,
                    });
                }
            }
        }

        let should_update_entry_point =
            !batch_ctx.has_entry_point || new_layer > batch_ctx.running_max_layer;
        let should_update_ram_entry_point =
            !batch_ctx.has_ram_entry_point || new_layer > batch_ctx.running_max_layer;

        if should_update_entry_point {
            batch_ctx.running_max_layer = new_layer;
            batch_ctx.has_entry_point = true;
        }
        if should_update_ram_entry_point {
            batch_ctx.has_ram_entry_point = true;
        }

        Ok(PreparedInsert {
            doc_id: id,
            vector_data,
            new_layer,
            new_idx,
            final_connections,
            neighbor_backlinks,
            should_update_entry_point,
            should_update_ram_entry_point,
        })
    }

    pub fn apply_insert(&self, prepared: PreparedInsert, tx_id: u64) {
        let node = HnswNode {
            doc_id: prepared.doc_id,
            vector: prepared.vector_data,
            max_layer: prepared.new_layer,
            committed_tx: tx_id,
        };

        let m = self.cold.config.m;
        self.hot
            .arena
            .allocate_node(prepared.new_layer, m, &prepared.final_connections)
            .ok();

        for backlink in prepared.neighbor_backlinks {
            self.hot
                .arena
                .update_backlink(
                    backlink.neighbor_ram_idx,
                    backlink.layer,
                    m,
                    &backlink.updated_connections,
                )
                .ok();
        }

        self.hot.nodes.write().push(node);
        self.hot
            .doc_to_node
            .write()
            .insert(prepared.doc_id.inner(), prepared.new_idx);

        let current_ep = self.hot.get_entry_point();
        if prepared.should_update_entry_point || current_ep.is_none() {
            self.hot.set_entry_point(Some(prepared.new_idx));
            let current_max = self.hot.max_layer.load(Ordering::Acquire) as usize;
            if prepared.new_layer > current_max || current_ep.is_none() {
                self.hot
                    .max_layer
                    .store(prepared.new_layer as u64, Ordering::Release);
            }
        }

        if prepared.should_update_ram_entry_point || self.hot.get_ram_entry_point().is_none() {
            self.hot.set_ram_entry_point(Some(prepared.new_idx));
        }
    }

    pub(super) fn do_insert(&self, id: DocId, vector: &[f32], tx_id: u64) -> Result<()> {
        let prepared = self.compute_insert(id, vector)?;
        self.apply_insert(prepared, tx_id);
        Ok(())
    }

    pub fn connectivity_score(&self) -> f64 {
        let deleted = self.hot.deleted_count.load(Ordering::SeqCst);
        let mmap_count = self
            .cold
            .mmap_index
            .read()
            .as_ref()
            .map(|m| m.header.node_count() as usize)
            .unwrap_or(0);
        let total = mmap_count + self.hot.nodes.read().len();
        if total == 0 {
            return 1.0;
        }
        (1.0 - deleted as f64 / total as f64).max(0.0)
    }

    pub fn check_connectivity(&self) -> contextra_core::Result<()> {
        let score = self.connectivity_score();
        if score < self.cold.config.rebuild_threshold {
            let deleted_ratio = (1.0 - score) * 100.0;
            return Err(contextra_core::ContextraError::HnswConnectivityDegraded { deleted_ratio });
        }
        Ok(())
    }

}
