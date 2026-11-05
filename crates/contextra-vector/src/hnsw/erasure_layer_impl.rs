use zeroize::Zeroize;
// FILE-CONTEXT
// ZWECK: ErasureLayer trait implementation for HNSW vector index.
// INVARIANTEN: Cryptographic erasure layer scope resolution and purged state witnessing.
// HOTSPOTS: purge, witness, on_gate

use contextra_ports::{
    Armed, BoxFuture, EraseTarget, ErasureLayer, KeyCoverage, LayerId, LayerScope, LayerWitness,
    PurgeCx, PurgeOutcome, Scanner,
};
use contextra_types::DocId;

use super::deletion::GhostFreeVectorIndex;
use super::types::{HnswIndex, VectorData};

impl ErasureLayer for HnswIndex {
    fn id(&self) -> LayerId {
        LayerId::new(*b"contextra-vector")
    }

    fn resolve(&self, target: &EraseTarget) -> BoxFuture<'_, Result<LayerScope, String>> {
        let this = self.clone();
        let target_cloned = target.clone();
        Box::pin(async move {
            let doc_map = this.inner.hot.doc_to_node.read();
            let mut affected = Vec::new();

            match target_cloned {
                EraseTarget::ExactKeys(keys) => {
                    for key in keys {
                        if key.len() >= 8 {
                            let mut buf = [0u8; 8];
                            buf.copy_from_slice(&key[0..8]);
                            let doc_raw = u64::from_be_bytes(buf);
                            #[cfg(not(feature = "docid-128"))]
                            {
                                if doc_map.contains_key(&doc_raw) {
                                    affected.push(doc_raw);
                                }
                            }
                            #[cfg(feature = "docid-128")]
                            {
                                if doc_map.contains_key(&(doc_raw as u128)) {
                                    affected.push(doc_raw);
                                }
                            }
                        }
                    }
                }
                EraseTarget::KeyRange { start, end } => {
                    let start_raw = if start.len() >= 8 {
                        let mut buf = [0u8; 8];
                        buf.copy_from_slice(&start[0..8]);
                        u64::from_be_bytes(buf)
                    } else {
                        0
                    };
                    let end_raw = if end.len() >= 8 {
                        let mut buf = [0u8; 8];
                        buf.copy_from_slice(&end[0..8]);
                        u64::from_be_bytes(buf)
                    } else {
                        u64::MAX
                    };

                    for &raw_id in doc_map.keys() {
                        #[cfg(not(feature = "docid-128"))]
                        let doc_u64 = raw_id;
                        #[cfg(feature = "docid-128")]
                        let doc_u64 = raw_id as u64;
                        if doc_u64 >= start_raw && doc_u64 < end_raw {
                            affected.push(doc_u64);
                        }
                    }
                }
            }

            let mut scope_id = [0u8; 16];
            let mut hasher = blake3::Hasher::new();
            for unit in &affected {
                hasher.update(&unit.to_le_bytes());
            }
            scope_id.copy_from_slice(&hasher.finalize().as_bytes()[0..16]);

            Ok(LayerScope::new(scope_id, affected))
        })
    }

    fn on_gate(&self, scope: &LayerScope) -> BoxFuture<'_, Result<(), String>> {
        let this = self.clone();
        let units = scope.affected_units().to_vec();
        Box::pin(async move {
            let dim = this.inner.cold.config.dimension;
            let m = this.inner.cold.config.m;
            let buffer_node_count = this
                .inner
                .cold
                .buffer_index
                .read()
                .as_ref()
                .map(|b| b.header.node_count() as usize)
                .unwrap_or(0);

            let ep_vector = {
                let ep_opt = this.inner.hot.get_entry_point();
                let nodes_read = this.inner.hot.nodes.read();
                ep_opt.and_then(|ep| {
                    if ep >= buffer_node_count {
                        let ram_idx = ep - buffer_node_count;
                        nodes_read.get(ram_idx).and_then(|n| match &n.vector {
                            VectorData::F32(v) => Some(v.clone()),
                            VectorData::U8(_) => None,
                        })
                    } else {
                        None
                    }
                })
            };

            let doc_map = this.inner.hot.doc_to_node.read();
            let mut nodes_write = this.inner.hot.nodes.write();

            let target_limit = 4096;
            for doc_raw in units.into_iter().take(target_limit) {
                if let Some(target_idx) = doc_map.get(&doc_raw).copied() {
                    if target_idx >= buffer_node_count {
                        let ram_idx = target_idx - buffer_node_count;

                        let mut neighbor_vecs = Vec::new();
                        let conns = this.inner.hot.arena.get_ram_node_connections(ram_idx, 0, m);
                        for &neighbor in &conns {
                            let n_idx = neighbor as usize;
                            if n_idx >= buffer_node_count
                                && n_idx - buffer_node_count < nodes_write.len()
                            {
                                let n_ram = n_idx - buffer_node_count;
                                if n_ram != ram_idx {
                                    if let Some(node) = nodes_write.get(n_ram) {
                                        if let VectorData::F32(ref v) = node.vector {
                                            neighbor_vecs.push(v.clone());
                                        }
                                    }
                                }
                            }
                        }

                        let surrogate = if !neighbor_vecs.is_empty() {
                            let count = neighbor_vecs.len() as f32;
                            let mut mean = vec![0.0f32; dim];
                            for v in &neighbor_vecs {
                                for (i, &val) in v.iter().enumerate() {
                                    mean[i] += val;
                                }
                            }
                            for val in &mut mean {
                                *val /= count;
                            }
                            Some(mean)
                        } else {
                            ep_vector.clone()
                        };

                        if let Some(node) = nodes_write.get_mut(ram_idx) {
                            match &mut node.vector {
                                VectorData::F32(v) => {
                                    v.zeroize();
                                    this.inner.hot.arena.zeroize_vector(ram_idx, dim);
                                    if let Some(ref s) = surrogate {
                                        if s.len() == v.len() {
                                            v.copy_from_slice(s);
                                            this.inner.hot.arena.store_vector(ram_idx, dim, s);
                                        }
                                    }
                                }
                                VectorData::U8(v) => {
                                    v.zeroize();
                                }
                            }
                        }
                    }
                }
            }

            Ok(())
        })
    }

    fn purge(
        &self,
        scope: &LayerScope,
        _cx: &PurgeCx,
    ) -> BoxFuture<'_, Result<PurgeOutcome, String>> {
        let this = self.clone();
        let units = scope.affected_units().to_vec();
        Box::pin(async move {
            let mut purged_count = 0u64;
            let mut bytes_reclaimed = 0u64;

            let buffer_node_count = this
                .inner
                .cold
                .buffer_index
                .read()
                .as_ref()
                .map(|m| m.header.node_count() as usize)
                .unwrap_or(0);
            let m = this.inner.cold.config.m;

            for doc_raw in units {
                let doc_id = DocId::new(doc_raw);
                let target_idx = {
                    let doc_map = this.inner.hot.doc_to_node.read();
                    doc_map.get(&doc_id.inner()).copied()
                };

                if let Some(idx) = target_idx {
                    let mut mut_self = this.clone();
                    let res = mut_self.remove_with_graph_repair(doc_id);
                    if let Ok(_stats) = res {
                        purged_count += 1;
                        if idx >= buffer_node_count {
                            let ram_idx = idx - buffer_node_count;
                            let max_layer = this
                                .inner
                                .hot
                                .nodes
                                .read()
                                .get(ram_idx)
                                .map(|n| n.max_layer)
                                .unwrap_or(0);
                            this.inner
                                .hot
                                .arena
                                .zeroize_and_free_node(ram_idx, max_layer, m);
                            bytes_reclaimed += (this.inner.cold.config.dimension * 4) as u64;
                        }
                    }
                }
            }

            Ok(PurgeOutcome::new(purged_count, bytes_reclaimed))
        })
    }

    fn witness(
        &self,
        scope: &LayerScope,
        _sc: &Scanner<Armed>,
    ) -> BoxFuture<'_, Result<LayerWitness, String>> {
        let this = self.clone();
        let units = scope.affected_units().to_vec();
        let scope_id = *scope.scope_id();
        Box::pin(async move {
            let doc_map = this.inner.hot.doc_to_node.read();
            let deleted = this.inner.cold.deleted_nodes.read();
            let buffer_node_count = this
                .inner
                .cold
                .buffer_index
                .read()
                .as_ref()
                .map(|m| m.header.node_count() as usize)
                .unwrap_or(0);
            let nodes_read = this.inner.hot.nodes.read();

            let mut residual_found = false;

            for doc_raw in units {
                #[cfg(not(feature = "docid-128"))]
                let node_idx_opt = doc_map.get(&doc_raw).copied();
                #[cfg(feature = "docid-128")]
                let node_idx_opt = doc_map.get(&(doc_raw as u128)).copied();

                if let Some(idx) = node_idx_opt {
                    if !deleted.contains(idx as u64) {
                        residual_found = true;
                        break;
                    }
                    if idx >= buffer_node_count {
                        let ram_idx = idx - buffer_node_count;
                        if let Some(node) = nodes_read.get(ram_idx) {
                            match &node.vector {
                                VectorData::F32(v) => {
                                    if v.iter().any(|&x| x.is_nan()) {
                                        residual_found = true;
                                        break;
                                    }
                                }
                                VectorData::U8(v) => {
                                    if v.iter().any(|&b| b != 0) {
                                        residual_found = true;
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }
            }

            if residual_found {
                return Err(
                    "Residue detected during witness scan: node not purged or memory dirty".into(),
                );
            }

            let mut hasher = blake3::Hasher::new();
            hasher.update(b"contextra-vector:witness:clean:");
            hasher.update(&scope_id);
            let hash = *hasher.finalize().as_bytes();

            Ok(LayerWitness::from_hash(hash))
        })
    }

    fn key_coverage(&self) -> KeyCoverage {
        KeyCoverage::None
    }
}
