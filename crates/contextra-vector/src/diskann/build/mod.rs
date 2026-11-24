// FILE-CONTEXT
// ZWECK: Graphaufbau (Vamana-Algorithmus) und Re-Exports für DiskANN (WP-VEC-07).
// INVARIANTEN: Lock-Hierarchie; atomares Ersetzen temporärer Dateien.
// STAND: 2026-11-11

pub(crate) mod flush;
pub(crate) mod pending_wal;
pub(crate) mod tombstone_wal;

pub(crate) use tombstone_wal::TombstoneSet;

use std::sync::atomic::Ordering;

use super::types::DiskAnnIndex;
use crate::distance::compute_distance_trusted;
use crate::error::{Result, VectorError};
use crate::types::DocId;

impl DiskAnnIndex {
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

        crate::diskann::io::atomic_replace(&tmp_path, &self.inner.config.index_path)?;
        let pending_wal = self.inner.config.index_path.with_extension("pending.wal");
        crate::diskann::io::remove_if_exists(&pending_wal)?;
        let tombstone_wal = self.inner.config.index_path.with_extension("tombstone.wal");
        crate::diskann::io::remove_if_exists(&tombstone_wal)?;
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
                if node.neighbors.len() > max_degree {
                    return Err(VectorError::Index(format!(
                        "Graph integrity violation: Node {} has {} neighbors, exceeding max_degree {}",
                        i,
                        node.neighbors.len(),
                        max_degree
                    )));
                }
            }
        }
        Ok(())
    }
}
