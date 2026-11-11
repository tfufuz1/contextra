// FILE-CONTEXT
// ZWECK: Hauptstruktur GenericHnswIndex und oeffentliche Abfrage-Methoden.
// INVARIANTEN: Lock-Hierarchie: write_mutex -> entry_point -> nodes / doc_to_node / deleted_nodes -> arena.
// HOTSPOTS: GenericHnswIndex::node_idx_for_doc, GenericHnswIndex::layer0_connections
// STAND: TS:2026-11-12T00:00:00Z

#[cfg(feature = "partial-index-rebuild")]
use std::collections::BTreeMap;
#[cfg(test)]
use std::sync::atomic::Ordering;

use crate::distance::{DistanceKernel, ScalarKernel};
use crate::error::Result;
use crate::types::DocId;

use super::core::{HnswIndexCore, RebuildStatus};
use crate::hnsw::sq8_bias::Sq8Bias;

// ANCHOR[contextra-vector:HNSW-GENERIC-INDEX]
/// Generischer HNSW (Hierarchical Navigable Small World) Vektorindex parametrisiert ueber DistanceKernel.
pub struct GenericHnswIndex<K: DistanceKernel = ScalarKernel> {
    pub(crate) inner: std::sync::Arc<HnswIndexCore>,
    pub(crate) _kernel: std::marker::PhantomData<K>,
}

/// Der Standard-HNSW-Vektorindex mit dem skalaren Referenz-Distanzkernel.
pub type HnswIndex = GenericHnswIndex<ScalarKernel>;

impl<K: DistanceKernel> GenericHnswIndex<K> {
    /// Gibt den aktuellen globalen Entry-Point-Knotenindex zurueck (falls vorhanden).
    pub fn entry_point(&self) -> Option<usize> {
        self.inner.hot.get_entry_point()
    }

    /// Gibt den internen Knotenindex fuer eine `DocId` zurueck (falls vorhanden und nicht geloescht).
    pub fn node_idx_for_doc(&self, doc_id: DocId) -> Option<usize> {
        let map = self.inner.hot.doc_to_node.read();
        let deleted = self.inner.cold.deleted_nodes.read();
        map.get(&doc_id.inner())
            .copied()
            .filter(|&idx| !deleted.contains(idx as u64))
    }

    /// Gibt die Layer-0-Verbindungen fuer einen Knotenindex zurueck.
    pub fn layer0_connections(&self, node_idx: usize) -> Vec<u32> {
        let mmap_count = self
            .inner
            .cold
            .mmap_index
            .read()
            .as_ref()
            .map(|m| m.header.node_count() as usize)
            .unwrap_or(0);
        let m = self.inner.cold.config.m;
        if node_idx < mmap_count {
            let mmap_guard = self.inner.cold.mmap_index.read();
            if let Some(mmap) = mmap_guard.as_ref() {
                if let Ok(rec) = mmap.get_node_record(node_idx) {
                    mmap.get_connections(&rec, 0).unwrap_or_default()
                } else {
                    Vec::new()
                }
            } else {
                Vec::new()
            }
        } else {
            let ram_idx = node_idx - mmap_count;
            self.inner.hot.arena.get_ram_node_connections(ram_idx, 0, m)
        }
    }

    pub fn all_doc_ids_from_map(&self) -> Vec<DocId> {
        let map = self.inner.hot.doc_to_node.read();
        let deleted = self.inner.cold.deleted_nodes.read();
        map.iter()
            .filter(|(&_doc_id_raw, &node_idx)| !deleted.contains(node_idx as u64))
            .map(|(&doc_id_raw, _)| DocId::new(doc_id_raw))
            .collect()
    }

    /// Returns all non-deleted document IDs and their assigned HNSW max layers.
    pub fn all_doc_ids_and_layers(&self) -> Vec<(DocId, usize)> {
        let map = self.inner.hot.doc_to_node.read();
        let nodes = self.inner.hot.nodes.read();
        let deleted = self.inner.cold.deleted_nodes.read();
        let mut list: Vec<(DocId, usize)> = map
            .iter()
            .filter(|(&_doc_id_raw, &node_idx)| !deleted.contains(node_idx as u64))
            .map(|(&doc_id_raw, &node_idx)| (DocId::new(doc_id_raw), nodes[node_idx].max_layer))
            .collect();
        list.sort_by_key(|(id, _)| id.inner());
        list
    }

    pub fn deleted_ratio(&self) -> f64 {
        self.inner.deleted_ratio()
    }

    pub fn rebuild_count(&self) -> u64 {
        self.inner.rebuild_count()
    }

    pub fn visited_dead_nodes(&self) -> u64 {
        self.inner.visited_dead_nodes()
    }

    pub fn connectivity_score(&self) -> f64 {
        self.inner.connectivity_score()
    }

    pub fn check_connectivity(&self) -> crate::error::Result<()> {
        self.inner.check_connectivity()
    }

    pub fn is_rebuild_required(&self) -> bool {
        self.inner.is_rebuild_required()
    }

    pub fn rebuild(&self) -> Result<()> {
        self.inner.rebuild()
    }

    pub fn rebuild_sync(&self) -> Result<()> {
        self.inner.rebuild_sync()
    }

    pub fn rebuild_region(&self, region_node_ids: Vec<u64>) -> Result<()> {
        self.inner.rebuild_region(region_node_ids)
    }

    pub fn rebuild_region_sync(&self, region_node_ids: Vec<u64>) -> Result<()> {
        self.inner.rebuild_region_sync(region_node_ids)
    }

    pub fn trigger_rebuild_async(&self) {
        if self.is_rebuild_required() {
            let inner = std::sync::Arc::clone(&self.inner);
            self.inner.cold.compute_pool.execute(move || {
                let res = inner.rebuild_sync();
                if let Err(ref e) = res {
                    tracing::error!("Failed to rebuild HNSW index: {}", e);
                }
            });
        }
    }

    /// Returns the clock instance used by this index.
    pub fn clock(&self) -> std::sync::Arc<dyn crate::types::Clock> {
        self.inner.clock()
    }

    pub fn rebuild_status(&self) -> RebuildStatus {
        self.inner.rebuild_status()
    }

    pub fn wait_for_rebuild_blocking(
        &self,
        timeout: std::time::Duration,
        clock: &dyn crate::types::Clock,
    ) -> bool {
        self.inner.wait_for_rebuild_blocking(timeout, clock)
    }

    pub fn wait_for_rebuild(&self) -> bool {
        self.inner.wait_for_rebuild()
    }

    pub fn wait_for_rebuild_with_timeout(&self, timeout: std::time::Duration) -> bool {
        self.inner.wait_for_rebuild_with_timeout(timeout)
    }

    /// Returns a snapshot clone of the current quantizer if trained.
    pub fn quantizer(&self) -> Option<crate::quantize::ScalarQuantizer> {
        self.inner.cold.quantizer.read().clone()
    }

    /// Returns the calibrated SQ8 quantization bias statistics.
    pub fn sq8_bias(&self) -> Sq8Bias {
        *self.inner.cold.sq8_bias.read()
    }

    /// Returns the metrics sink.
    pub fn metrics_sink(&self) -> std::sync::Arc<dyn crate::types::MetricsSink> {
        self.inner.metrics_sink()
    }

    /// Sets the metrics sink.
    pub fn set_metrics_sink(&self, sink: std::sync::Arc<dyn crate::types::MetricsSink>) {
        self.inner.set_metrics_sink(sink);
    }

    /// Builder method to set the metrics sink.
    pub fn with_metrics_sink(self, sink: std::sync::Arc<dyn crate::types::MetricsSink>) -> Self {
        self.set_metrics_sink(sink);
        self
    }

    /// Sets the clock instance.
    pub fn set_clock(&self, clock: std::sync::Arc<dyn crate::types::Clock>) {
        self.inner.set_clock(clock);
    }

    /// Builder method to set the clock port.
    pub fn with_clock(self, clock: std::sync::Arc<dyn crate::types::Clock>) -> Self {
        self.set_clock(clock);
        self
    }

    /// Sets the random seed used for HNSW layer selection during insertions.
    pub fn set_layer_seed(&self, seed: u64) {
        self.inner.set_layer_seed(seed);
    }

    /// Sets the target insertion count for simulating compute_insert fault injection in tests.
    #[cfg(test)]
    pub fn set_fault_injection_insert_target(&self, target: u64) {
        self.inner
            .cold
            .fault_injection_insert_target
            .store(target, Ordering::SeqCst);
        self.inner
            .cold
            .fault_injection_insert_count
            .store(0, Ordering::SeqCst);
    }

    #[cfg(feature = "partial-index-rebuild")]
    pub fn check_and_trigger_partial_rebuild(&self) {
        let Some(ref tracker_lock) = self.inner.cold.traversal_tracker else {
            return;
        };
        let global_tombstone_ratio = self.deleted_ratio() as f32;
        let tracker = tracker_lock.read();

        let mut tombstone_map = BTreeMap::new();
        let total_nodes = self.inner.hot.nodes.read().len();
        let deleted_guard = self.inner.cold.deleted_nodes.read();

        for i in 0..total_nodes {
            let id = i as u64;
            tombstone_map.insert(id, deleted_guard.contains(id));
        }

        let regions = crate::partial_rebuild::should_trigger_partial_rebuild(
            &tracker,
            &tombstone_map,
            global_tombstone_ratio,
            &self.inner.cold.config.partial_rebuild_config,
        );

        if let Some(region_node_ids) = regions {
            let inner = std::sync::Arc::clone(&self.inner);
            self.inner.cold.compute_pool.execute(move || {
                let res = inner.rebuild_region_sync(region_node_ids);
                if let Err(ref e) = res {
                    tracing::error!("Failed local partial rebuild: {}", e);
                }
            });
        }
    }
}
