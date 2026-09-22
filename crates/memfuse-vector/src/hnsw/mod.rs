// FILE-CONTEXT
// ZWECK: HNSW Vector Index mit Layer Descent, Soft-Deletes und transaktionalem Staging (TxBuffer).
// INVARIANTEN: Lock-Hierarchie: write_mutex (exklusive Mutation/Rebuild) -> entry_point -> nodes / doc_to_node / deleted_nodes (HotState/ColdState).
// NICHT-OFFENSICHTLICH: Multi-threaded Reads sperren nie write_mutex; background rebuild tauscht Core atomar via Swap.
// HOTSPOTS: mod.rs (HnswIndex::insert, search, delete, rebuild, save)
// STAND: TS:2026-08-30T18:53:53Z (SESSION: 37b1d991)

//! HNSW (Hierarchical Navigable Small World) vector index module.

pub mod arena;
pub mod sq8_bias;

pub use arena::{BacklinkTable, HnswArena};
pub use sq8_bias::Sq8Bias;

use crate::distance::compute_distance_trusted;
use ahash::{AHashMap, AHashSet};
use memfuse_core::{
    DistanceMetric, DocId, IndexOp, MemFuseError, Result, ScoredDocument, TxBuffer, TxId,
    VectorIndex, VectorIndexStats,
};
use parking_lot::RwLock;
use rand::Rng;
use roaring::RoaringTreemap;
use std::borrow::Cow;
use std::cmp::Reverse;
use std::collections::BinaryHeap;
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};

/// Sentinel-Wert für ungültigen / nicht gesetzten Entry-Point in HNSW.
pub const SENTINEL_NO_ENTRY_POINT: u32 = u32::MAX;
use tokio::sync::Mutex;

/// Standard-Löschanteil (0.10 = 10 % gelöschte Knoten), ab dem ein Rebuild getriggert wird.
pub const HNSW_REBUILD_DELETION_RATIO: f64 = 0.10;

/// Configuration parameters for the HNSW index.
#[derive(Debug, Clone)]
pub struct HnswConfig {
    /// Vector dimensionality.
    pub dimension: usize,
    /// Maximum number of elements.
    pub max_elements: usize,
    /// Number of connections per element (M parameter).
    pub m: usize,
    /// Dynamic candidate list size during graph construction (`ef_construction`).
    pub ef_construction: usize,
    /// Dynamic candidate list size during search.
    pub ef_search: usize,
    /// Distance metric.
    pub distance_metric: DistanceMetric,
    /// Rebuild threshold.
    pub rebuild_threshold: f64,
    /// Whether to apply SQ8 Scalar Quantization to the index vectors to reduce RAM.
    pub quantize: bool,
    /// Sample size used for ScalarQuantizer recalibration during rebuilds.
    pub quantizer_recalibration_sample_size: usize,
    /// Quantizer drift ratio threshold above which an index rebuild is recommended.
    pub quantizer_drift_threshold: f32,
    /// Partial rebuild configuration for hot-path local rebuilds (F-02).
    #[cfg(feature = "partial-index-rebuild")]
    pub partial_rebuild_config: crate::partial_rebuild::PartialRebuildConfig,
}

impl Default for HnswConfig {
    fn default() -> Self {
        Self {
            dimension: 1536,
            max_elements: 1_000_000,
            m: 16,
            ef_construction: 200,
            ef_search: 64,
            distance_metric: DistanceMetric::Cosine,
            rebuild_threshold: 1.0 - HNSW_REBUILD_DELETION_RATIO,
            quantize: false,
            quantizer_recalibration_sample_size: 10_000,
            quantizer_drift_threshold: 0.10,
            #[cfg(feature = "partial-index-rebuild")]
            partial_rebuild_config: crate::partial_rebuild::PartialRebuildConfig::default(),
        }
    }
}

/// Validates that a vector is non-empty and contains no NaN or Infinite values.
fn validate_vector(vec: &[f32]) -> Result<()> {
    crate::distance::validate_vector(vec)
}

impl HnswConfig {
    /// Validates that the configuration parameters are within acceptable bounds.
    pub fn validate(&self) -> Result<()> {
        if self.dimension == 0 {
            return Err(MemFuseError::invalid_input(
                "dimension must be greater than 0",
            ));
        }
        if self.m == 0 {
            return Err(MemFuseError::invalid_input("m must be greater than 0"));
        }
        if self.ef_search == 0 {
            return Err(MemFuseError::invalid_input(
                "ef_search must be greater than 0",
            ));
        }
        if self.ef_construction < self.m {
            return Err(MemFuseError::invalid_input(format!(
                "ef_construction ({}) must be >= m ({})",
                self.ef_construction, self.m
            )));
        }
        if !(0.0..=1.0).contains(&self.rebuild_threshold) {
            return Err(MemFuseError::invalid_input(format!(
                "rebuild_threshold ({}) must be between 0.0 and 1.0",
                self.rebuild_threshold
            )));
        }
        if !(0.0..=1.0).contains(&self.quantizer_drift_threshold) {
            return Err(MemFuseError::invalid_input(format!(
                "quantizer_drift_threshold ({}) must be between 0.0 and 1.0",
                self.quantizer_drift_threshold
            )));
        }
        Ok(())
    }
}

/// Builder for HnswConfig with resource limit enforcements to prevent OOM.
#[derive(Debug, Clone)]
pub struct HnswConfigBuilder {
    config: HnswConfig,
}

impl HnswConfigBuilder {
    /// Creates a new builder with the chosen dimensionality.
    pub fn new(dimension: usize) -> Self {
        Self {
            config: HnswConfig {
                dimension,
                ..Default::default()
            },
        }
    }

    /// Set max elements with a hardcap limit to avoid OOM.
    pub fn max_elements(mut self, max: usize) -> Self {
        self.config.max_elements = max.min(50_000_000);
        self
    }

    /// Set the number of connections per element (M).
    pub fn m(mut self, m: usize) -> Self {
        self.config.m = m.clamp(4, 256);
        self
    }

    /// Set dynamic candidate list size for construction.
    pub fn ef_construction(mut self, ef: usize) -> Self {
        self.config.ef_construction = ef.min(4000);
        self
    }

    /// Set dynamic candidate list size for search.
    pub fn ef_search(mut self, ef: usize) -> Self {
        self.config.ef_search = ef.min(4000);
        self
    }

    /// Use a specific distance metric.
    pub fn distance_metric(mut self, metric: DistanceMetric) -> Self {
        self.config.distance_metric = metric;
        self
    }

    /// Enable or disable scalar quantization (SQ8) to reduce footprint.
    pub fn quantize(mut self, quantize: bool) -> Self {
        self.config.quantize = quantize;
        self
    }

    /// Sets the sample size used for ScalarQuantizer recalibration during rebuilds.
    pub fn quantizer_recalibration_sample_size(mut self, size: usize) -> Self {
        self.config.quantizer_recalibration_sample_size = size;
        self
    }

    /// Sets the drift ratio threshold for ScalarQuantizer recalibration and rebuilds.
    pub fn quantizer_drift_threshold(mut self, threshold: f32) -> Self {
        self.config.quantizer_drift_threshold = threshold.clamp(0.0, 1.0);
        self
    }

    /// Sets the rebuild threshold.
    pub fn rebuild_threshold(mut self, threshold: f64) -> Self {
        self.config.rebuild_threshold = threshold.clamp(0.0, 1.0);
        self
    }

    /// Sets the partial rebuild configuration for local hot-path rebuilds (F-02).
    #[cfg(feature = "partial-index-rebuild")]
    pub fn partial_rebuild_config(
        mut self,
        config: crate::partial_rebuild::PartialRebuildConfig,
    ) -> Self {
        self.config.partial_rebuild_config = config;
        self
    }

    /// Build the configuration after validating bounds.
    pub fn build(self) -> Result<HnswConfig> {
        self.config.validate()?;
        Ok(self.config)
    }
}

#[derive(Debug, Clone)]
/// Represents the format of vector data stored in the index.
pub enum VectorData {
    /// Standard 32-bit floating point vectors.
    F32(Vec<f32>),
    /// 8-bit quantized vectors (SQ8).
    U8(Vec<u8>),
}

/// A node in the HNSW graph.
#[derive(Debug)]
pub struct HnswNode {
    doc_id: DocId,
    vector: VectorData,
    max_layer: usize,
    committed_tx: u64,
}

/// Search candidate.
#[derive(Clone, Copy, Debug)]
struct Candidate {
    index: usize,
    distance: f32,
}

impl PartialEq for Candidate {
    fn eq(&self, other: &Self) -> bool {
        self.distance == other.distance
    }
}
impl Eq for Candidate {}

impl PartialOrd for Candidate {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Candidate {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.distance.total_cmp(&other.distance)
    }
}

struct RebuildGuard<'a>(&'a AtomicBool);

impl<'a> Drop for RebuildGuard<'a> {
    fn drop(&mut self) {
        self.0.store(false, Ordering::SeqCst);
    }
}

struct SnapshotPinGuard<'a>(&'a HnswIndexCore, u64);

impl<'a> SnapshotPinGuard<'a> {
    fn new(core: &'a HnswIndexCore, seq_no: u64) -> Self {
        core.cold.seq_log.write().pin_snapshot(seq_no);
        Self(core, seq_no)
    }
}

impl<'a> Drop for SnapshotPinGuard<'a> {
    fn drop(&mut self) {
        self.0.cold.seq_log.write().unpin_snapshot(self.1);
    }
}

/// The HNSW (Hierarchical Navigable Small World) vector index.
pub struct HnswIndex {
    inner: std::sync::Arc<HnswIndexCore>,
}

#[repr(align(64))]
pub struct HnswHotCore {
    pub nodes: RwLock<Vec<HnswNode>>,
    #[cfg(not(feature = "docid-128"))]
    pub doc_to_node: RwLock<AHashMap<u64, usize>>,
    #[cfg(feature = "docid-128")]
    pub doc_to_node: RwLock<AHashMap<u128, usize>>,
    pub entry_point: AtomicU32,
    pub ram_entry_point: AtomicU32,
    pub max_layer: AtomicU64,
    pub ml: f64,
    pub deleted_count: AtomicU64,
    pub write_mutex: Mutex<()>,
    pub last_tx_id: AtomicU64,
    pub rebuilding: AtomicBool,
    pub arena: HnswArena,
}

impl HnswHotCore {
    #[inline]
    pub fn get_entry_point(&self) -> Option<usize> {
        let ep = self.entry_point.load(Ordering::Acquire);
        if ep == SENTINEL_NO_ENTRY_POINT {
            None
        } else {
            Some(ep as usize)
        }
    }

    #[inline]
    pub fn set_entry_point(&self, ep: Option<usize>) {
        let val = match ep {
            Some(idx) => idx as u32,
            None => SENTINEL_NO_ENTRY_POINT,
        };
        self.entry_point.store(val, Ordering::Release);
    }

    #[inline]
    pub fn get_ram_entry_point(&self) -> Option<usize> {
        let ep = self.ram_entry_point.load(Ordering::Acquire);
        if ep == SENTINEL_NO_ENTRY_POINT {
            None
        } else {
            Some(ep as usize)
        }
    }

    #[inline]
    pub fn set_ram_entry_point(&self, ep: Option<usize>) {
        let val = match ep {
            Some(idx) => idx as u32,
            None => SENTINEL_NO_ENTRY_POINT,
        };
        self.ram_entry_point.store(val, Ordering::Release);
    }

    pub fn get_ram_node_connections(&self, ram_idx: usize, layer: usize, m: usize) -> Vec<u32> {
        self.arena.get_ram_node_connections(ram_idx, layer, m)
    }
}

pub struct HnswColdCore {
    pub config: HnswConfig,
    pub validation_error: Option<String>,
    pub tx_buffer: TxBuffer<Vec<f32>>,
    pub quantizer: RwLock<Option<crate::quantize::ScalarQuantizer>>,
    pub sq8_bias: RwLock<Sq8Bias>,
    pub mmap_index: RwLock<Option<crate::persistence::MmapIndex>>,
    pub seq_log: RwLock<memfuse_core::SequenceLog>,
    pub rebuild_count: AtomicU64,
    pub visited_dead_nodes: AtomicU64,
    pub deleted_nodes: RwLock<RoaringTreemap>,
    #[cfg(feature = "partial-index-rebuild")]
    pub traversal_tracker: RwLock<crate::partial_rebuild::TraversalTracker>,
    #[cfg(test)]
    pub fault_injection_insert_target: AtomicU64,
    #[cfg(test)]
    pub fault_injection_insert_count: AtomicU64,
}

/// The core implementation of the HNSW index.
pub struct HnswIndexCore {
    pub hot: HnswHotCore,
    pub cold: HnswColdCore,
}

impl HnswIndex {
    /// Creates a new HNSW index, validating configuration upfront.
    pub fn try_new(config: HnswConfig) -> Result<Self> {
        config.validate()?;
        let ml = 1.0 / (config.m as f64).ln();
        #[cfg(feature = "partial-index-rebuild")]
        let partial_rebuild_config = config.partial_rebuild_config.clone();

        Ok(Self {
            inner: std::sync::Arc::new(HnswIndexCore {
                hot: HnswHotCore {
                    nodes: RwLock::new(Vec::new()),
                    doc_to_node: RwLock::new(AHashMap::new()),
                    entry_point: AtomicU32::new(SENTINEL_NO_ENTRY_POINT),
                    ram_entry_point: AtomicU32::new(SENTINEL_NO_ENTRY_POINT),
                    max_layer: AtomicU64::new(0),
                    ml,
                    deleted_count: AtomicU64::new(0),
                    write_mutex: Mutex::new(()),
                    last_tx_id: AtomicU64::new(0),
                    rebuilding: AtomicBool::new(false),
                    arena: HnswArena::new(),
                },
                cold: HnswColdCore {
                    config,
                    validation_error: None,
                    tx_buffer: TxBuffer::new_with_config(16, std::time::Duration::from_secs(60)),
                    quantizer: RwLock::new(None),
                    sq8_bias: RwLock::new(Sq8Bias::default()),
                    mmap_index: RwLock::new(None),
                    seq_log: RwLock::new(memfuse_core::SequenceLog::new()),
                    rebuild_count: AtomicU64::new(0),
                    visited_dead_nodes: AtomicU64::new(0),
                    deleted_nodes: RwLock::new(RoaringTreemap::new()),
                    #[cfg(feature = "partial-index-rebuild")]
                    traversal_tracker: RwLock::new(crate::partial_rebuild::TraversalTracker::new(
                        partial_rebuild_config,
                    )),
                    #[cfg(test)]
                    fault_injection_insert_target: AtomicU64::new(0),
                    #[cfg(test)]
                    fault_injection_insert_count: AtomicU64::new(0),
                },
            }),
        })
    }

    /// Creates a new HNSW index.
    #[deprecated(
        note = "Nutze try_new() für sofortige Fehlererkennung — new() versteckt Konfigurationsfehler bis zum ersten insert()/search()"
    )]
    pub fn new(config: HnswConfig) -> Self {
        let validation_error = config.validate().err().map(|e| e.to_string());
        let ml = 1.0 / (config.m as f64).ln();
        #[cfg(feature = "partial-index-rebuild")]
        let partial_rebuild_config = config.partial_rebuild_config.clone();

        Self {
            inner: std::sync::Arc::new(HnswIndexCore {
                hot: HnswHotCore {
                    nodes: RwLock::new(Vec::new()),
                    doc_to_node: RwLock::new(AHashMap::new()),
                    entry_point: AtomicU32::new(SENTINEL_NO_ENTRY_POINT),
                    ram_entry_point: AtomicU32::new(SENTINEL_NO_ENTRY_POINT),
                    max_layer: AtomicU64::new(0),
                    ml,
                    deleted_count: AtomicU64::new(0),
                    write_mutex: Mutex::new(()),
                    last_tx_id: AtomicU64::new(0),
                    rebuilding: AtomicBool::new(false),
                    arena: HnswArena::new(),
                },
                cold: HnswColdCore {
                    config,
                    validation_error,
                    tx_buffer: TxBuffer::new_with_config(16, std::time::Duration::from_secs(60)),
                    quantizer: RwLock::new(None),
                    sq8_bias: RwLock::new(Sq8Bias::default()),
                    mmap_index: RwLock::new(None),
                    seq_log: RwLock::new(memfuse_core::SequenceLog::new()),
                    rebuild_count: AtomicU64::new(0),
                    visited_dead_nodes: AtomicU64::new(0),
                    deleted_nodes: RwLock::new(RoaringTreemap::new()),
                    #[cfg(feature = "partial-index-rebuild")]
                    traversal_tracker: RwLock::new(crate::partial_rebuild::TraversalTracker::new(
                        partial_rebuild_config,
                    )),
                    #[cfg(test)]
                    fault_injection_insert_target: AtomicU64::new(0),
                    #[cfg(test)]
                    fault_injection_insert_count: AtomicU64::new(0),
                },
            }),
        }
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

    /// Returns a snapshot clone of the current quantizer if trained.
    pub fn quantizer(&self) -> Option<crate::quantize::ScalarQuantizer> {
        self.inner.cold.quantizer.read().clone()
    }

    /// Returns the calibrated SQ8 quantization bias statistics.
    pub fn sq8_bias(&self) -> Sq8Bias {
        *self.inner.cold.sq8_bias.read()
    }

    /// Returns a reference to the quantizer RwLock for crate-internal access.
    #[allow(dead_code)]
    pub(crate) fn quantizer_lock(&self) -> &RwLock<Option<crate::quantize::ScalarQuantizer>> {
        &self.inner.cold.quantizer
    }

    async fn search_filtered_internal(
        &self,
        query: &[f32],
        k: usize,
        filter: Option<&(dyn Fn(DocId) -> bool + Send + Sync)>,
        snapshot_seq: Option<u64>,
    ) -> Result<Vec<ScoredDocument>> {
        if let Some(ref err) = self.inner.cold.validation_error {
            return Err(MemFuseError::invalid_input(format!(
                "Invalid index configuration: {}",
                err
            )));
        }
        if query.len() != self.inner.cold.config.dimension {
            return Err(MemFuseError::invalid_input(format!(
                "Expected dimension {}, got {}",
                self.inner.cold.config.dimension,
                query.len()
            )));
        }

        if k == 0 {
            return Ok(Vec::new());
        }
        if k > memfuse_core::MAX_SEARCH_K {
            return Err(MemFuseError::invalid_input(format!(
                "Requested k ({k}) exceeds maximum allowed search limit ({}). \
                 Use Collection::query() with appropriate k bounds.",
                memfuse_core::MAX_SEARCH_K
            )));
        }
        for (i, &val) in query.iter().enumerate() {
            if !val.is_finite() {
                return Err(MemFuseError::invalid_input(format!(
                    "Query vector element at index {i} is not finite (value: {val}). \
                     NaN/Inf values corrupt HNSW distance computation and heap ordering. \
                     Validate embedding outputs before search."
                )));
            }
        }

        let query_quantized = if self.inner.cold.config.quantize {
            self.inner
                .cold
                .quantizer
                .read()
                .as_ref()
                .map(|q| q.quantize(query))
                .transpose()?
        } else {
            None
        };

        let mut ep = Vec::new();
        if let Some(global_ep) = self.inner.hot.get_entry_point() {
            ep.push(global_ep);
        }
        if let Some(ram_ep) = self.inner.hot.get_ram_entry_point() {
            if !ep.contains(&ram_ep) {
                ep.push(ram_ep);
            }
        }

        let nodes = self.inner.hot.nodes.read();
        let deleted = self.inner.cold.deleted_nodes.read();

        let mut filter_eps = Vec::new();
        if let Some(f) = filter {
            let mmap_guard = self.inner.cold.mmap_index.read();
            let mmap_node_count = mmap_guard
                .as_ref()
                .map(|m| m.header.node_count() as usize)
                .unwrap_or(0);
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

            let factor = if self.inner.cold.config.quantize {
                4
            } else {
                2
            };
            let max_filter_eps = self.inner.cold.config.ef_search.max(k) * factor;

            let total_nodes = mmap_node_count + nodes.len();
            for i in (0..total_nodes).rev() {
                if !ep.contains(&(i as _)) {
                    if snapshot_seq.is_none() && deleted.contains(i as u64) {
                        continue;
                    }
                    if let Ok(doc_id) = self.inner.resolve_doc_id(i, &ctx) {
                        if f(doc_id) {
                            ep.push(i);
                            filter_eps.push(i);
                            if filter_eps.len() >= max_filter_eps {
                                break;
                            }
                        }
                    }
                }
            }
        }

        if ep.is_empty() {
            return Ok(Vec::new());
        }

        let max_layer = self.inner.hot.max_layer.load(Ordering::SeqCst) as usize;

        for layer in (1..=max_layer).rev() {
            let best = self
                .inner
                .search_layer(query, query_quantized.as_deref(), &ep, 1, layer)?;
            if let Some(closest) = best.first() {
                ep = vec![closest.index];
            }
        }

        if let Some(ram_ep) = self.inner.hot.get_ram_entry_point() {
            if !ep.contains(&ram_ep) {
                ep.push(ram_ep);
            }
        }

        for f_ep in filter_eps {
            if !ep.contains(&f_ep) {
                ep.push(f_ep);
            }
        }

        let factor = if self.inner.cold.config.quantize {
            4
        } else {
            2
        };
        let ef = self.inner.cold.config.ef_search.max(k) * factor;
        let candidates = self
            .inner
            .search_layer(query, query_quantized.as_deref(), &ep, ef, 0)?;

        let score = self.inner.connectivity_score();
        if score < self.inner.cold.config.rebuild_threshold {
            let deleted_ratio = (1.0 - score) * 100.0;
            let err = memfuse_core::MemFuseError::HnswConnectivityDegraded { deleted_ratio };
            tracing::warn!(
                error = %err,
                connectivity_score = score,
                rebuild_threshold = self.inner.cold.config.rebuild_threshold,
                "HNSW index degraded — consider calling rebuild()"
            );
        }

        let mut results = Vec::with_capacity(k);

        let seq_log_guard = if snapshot_seq.is_some() {
            Some(self.inner.cold.seq_log.read())
        } else {
            None
        };

        for c in candidates.iter() {
            let node = nodes.get(c.index).ok_or_else(|| {
                MemFuseError::Index(format!("HNSW candidate node missing at index {}", c.index))
            })?;
            if node.committed_tx == 0 {
                continue;
            }
            let doc_id = node.doc_id;

            if let Some(snap_seq) = snapshot_seq {
                if let Some(ref seq_log) = seq_log_guard {
                    if !seq_log.is_visible(doc_id, snap_seq) {
                        continue;
                    }
                }
            } else {
                if deleted.contains(c.index as u64) {
                    continue;
                }
            }

            if let Some(f) = filter {
                if !f(doc_id) {
                    continue;
                }
            }

            let final_dist = if self.inner.cold.config.quantize {
                if let VectorData::U8(v) = &node.vector {
                    let guard = self.inner.cold.quantizer.read();
                    let q = guard.as_ref().ok_or_else(|| {
                        memfuse_core::MemFuseError::Index("Quantizer not trained".into())
                    })?;
                    q.asymmetric_dist(query, v, self.inner.cold.config.distance_metric)?
                } else {
                    c.distance
                }
            } else {
                c.distance
            };

            let score = match self.inner.cold.config.distance_metric {
                DistanceMetric::Cosine => 1.0 - final_dist,
                DistanceMetric::Euclidean => 1.0 / (1.0 + final_dist),
                DistanceMetric::DotProduct => -final_dist,
                other => {
                    return Err(MemFuseError::Index(format!(
                        "Unsupported DistanceMetric variant in search_filtered(): {other:?}"
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

    pub fn check_connectivity(&self) -> memfuse_core::Result<()> {
        self.inner.check_connectivity()
    }

    pub fn is_rebuild_required(&self) -> bool {
        self.inner.is_rebuild_required()
    }

    pub async fn rebuild(&self) -> Result<()> {
        self.inner.rebuild().await
    }

    pub async fn rebuild_region(&self, region_node_ids: Vec<u64>) -> Result<()> {
        self.inner.rebuild_region(region_node_ids).await
    }

    #[cfg(feature = "partial-index-rebuild")]
    pub fn check_and_trigger_partial_rebuild(&self) -> Option<tokio::task::JoinHandle<Result<()>>> {
        let global_tombstone_ratio = self.deleted_ratio() as f32;
        let tracker = self.inner.cold.traversal_tracker.read();

        let mut tombstone_map = ahash::AHashMap::new();
        let total_nodes = self.inner.hot.nodes.read().len();
        let deleted_guard = self.inner.cold.deleted_nodes.read();

        for i in 0..total_nodes {
            let id = i as u64;
            tombstone_map.insert(id, deleted_guard.contains(id));
        }

        let std_map: std::collections::HashMap<u64, bool> = tombstone_map.into_iter().collect();

        let regions = crate::partial_rebuild::should_trigger_partial_rebuild(
            &tracker,
            &std_map,
            global_tombstone_ratio,
            &self.inner.cold.config.partial_rebuild_config,
        );

        if let Some(region_node_ids) = regions {
            let inner = std::sync::Arc::clone(&self.inner);
            Some(tokio::spawn(async move {
                let res = inner.rebuild_region(region_node_ids).await;
                if let Err(ref e) = res {
                    tracing::error!("Failed local partial rebuild: {}", e);
                }
                res
            }))
        } else {
            None
        }
    }

    pub fn compact_seq_log(&self, min_active_seqno: u64) {
        self.inner.cold.seq_log.write().compact(min_active_seqno);
    }

    pub fn pin_snapshot(&self, seq_no: u64) {
        self.inner.cold.seq_log.write().pin_snapshot(seq_no);
    }

    pub fn unpin_snapshot(&self, seq_no: u64) {
        self.inner.cold.seq_log.write().unpin_snapshot(seq_no);
    }

    pub fn all_doc_ids_from_map(&self) -> Vec<DocId> {
        let map = self.inner.hot.doc_to_node.read();
        let deleted = self.inner.cold.deleted_nodes.read();
        map.iter()
            .filter(|(&_doc_id_raw, &node_idx)| !deleted.contains(node_idx as u64))
            .map(|(&doc_id_raw, _)| DocId::new(doc_id_raw))
            .collect()
    }

    pub fn trigger_rebuild_async(&self) -> Option<tokio::task::JoinHandle<Result<()>>> {
        if self.is_rebuild_required() {
            let inner = std::sync::Arc::clone(&self.inner);
            Some(tokio::spawn(async move {
                let res = inner.rebuild().await;
                if let Err(ref e) = res {
                    tracing::error!("Failed to rebuild HNSW index: {}", e);
                }
                res
            }))
        } else {
            None
        }
    }

    pub fn rebuild_status(&self) -> RebuildStatus {
        self.inner.rebuild_status()
    }

    pub async fn wait_for_rebuild(&self) -> bool {
        self.inner.wait_for_rebuild().await
    }

    pub async fn wait_for_rebuild_with_timeout(&self, timeout: std::time::Duration) -> bool {
        self.inner.wait_for_rebuild_with_timeout(timeout).await
    }

    pub async fn save(&self, path: impl AsRef<std::path::Path>) -> Result<()> {
        let _lock = self.inner.hot.write_mutex.lock().await;
        let inner = std::sync::Arc::clone(&self.inner);
        let path_buf = path.as_ref().to_path_buf();

        tokio::task::spawn_blocking(move || {
            use std::io::{Seek, Write};

            let nodes = inner.hot.nodes.read();
            let entry_point = inner.hot.get_entry_point();
            let q_guard = inner.cold.quantizer.read();

            let temp_path = path_buf.with_extension("hnsw.tmp");
            let file = std::fs::File::create(&temp_path).map_err(|e| {
                MemFuseError::Storage(format!("Failed to create temporary HNSW file: {}", e))
            })?;
            let mut writer = std::io::BufWriter::new(file);

            let node_count = nodes.len();
            let nodes_offset = crate::persistence::HnswHeader::SIZE as u64;
            let vectors_offset =
                nodes_offset + (node_count * crate::persistence::NodeRecord::SIZE) as u64;

            let (q_min, q_max) = if let Some(q) = q_guard.as_ref() {
                (
                    q.mins.first().copied().unwrap_or(0.0),
                    q.maxes.first().copied().unwrap_or(0.0),
                )
            } else {
                (0.0, 0.0)
            };

            let bias = *inner.cold.sq8_bias.read();

            let mut header = crate::persistence::HnswHeader::new_v2_with_bias(
                inner.cold.config.dimension as u32,
                inner.cold.config.m as u32,
                inner.cold.config.distance_metric as u8,
                if inner.cold.config.quantize { 1 } else { 0 },
                q_min,
                q_max,
                node_count as u64,
                entry_point.map(|i| i as i64).unwrap_or(-1),
                nodes_offset,
                0,
                inner.hot.last_tx_id.load(Ordering::SeqCst),
                0,
                0,
                bias.mean_bias,
                bias.variance_bias,
            );

            writer
                .write_all(&header.to_bytes())
                .map_err(|e| MemFuseError::Storage(e.to_string()))?;

            let mut node_records = Vec::with_capacity(node_count);
            for _ in 0..node_count {
                node_records.push(crate::persistence::NodeRecord {
                    doc_id: 0,
                    max_layer: 0,
                    vector_offset: 0,
                    connections_offset: 0,
                });
            }
            for record in &node_records {
                writer
                    .write_all(&record.to_bytes())
                    .map_err(|e| MemFuseError::Storage(e.to_string()))?;
            }

            let mut current_pos = vectors_offset;
            for (i, node) in nodes.iter().enumerate() {
                node_records[i].doc_id = node.doc_id.inner();
                node_records[i].max_layer = node.max_layer as u8;
                node_records[i].vector_offset = current_pos;

                match &node.vector {
                    VectorData::F32(v) => {
                        for &val in v {
                            writer
                                .write_all(&val.to_le_bytes())
                                .map_err(|e| MemFuseError::Storage(e.to_string()))?;
                        }
                        current_pos += (v.len() * 4) as u64;
                    }
                    VectorData::U8(v) => {
                        writer
                            .write_all(v)
                            .map_err(|e| MemFuseError::Storage(e.to_string()))?;
                        current_pos += v.len() as u64;
                    }
                }
            }

            let connections_offset = (current_pos + 3) & !3;
            header.set_connections_offset(connections_offset);

            if connections_offset > current_pos {
                let padding = [0u8; 4];
                writer
                    .write_all(&padding[..(connections_offset - current_pos) as usize])
                    .map_err(|e| MemFuseError::Storage(e.to_string()))?;
            }

            let mut conn_pos = connections_offset;
            for (i, node) in nodes.iter().enumerate() {
                node_records[i].connections_offset = conn_pos;
                let num_layers = (node.max_layer + 1) as u8;
                writer
                    .write_all(&[num_layers])
                    .map_err(|e| MemFuseError::Storage(e.to_string()))?;
                conn_pos += 1;

                for layer in 0..num_layers as usize {
                    let layer_conns =
                        inner
                            .hot
                            .get_ram_node_connections(i, layer, inner.cold.config.m);
                    let len = layer_conns.len() as u32;
                    writer
                        .write_all(&len.to_le_bytes())
                        .map_err(|e| MemFuseError::Storage(e.to_string()))?;
                    for &conn in layer_conns.iter() {
                        writer
                            .write_all(&conn.to_le_bytes())
                            .map_err(|e| MemFuseError::Storage(e.to_string()))?;
                    }
                    conn_pos += 4 + (len as u64) * 4;
                }
            }

            let cal_offset = conn_pos;
            let mut cal_len = 0u32;

            if let Some(ref q) = *q_guard {
                let dim = q.mins().len();
                cal_len = (dim * 4 * 2) as u32;
                for &m in q.mins() {
                    writer
                        .write_all(&m.to_le_bytes())
                        .map_err(|e| MemFuseError::Storage(e.to_string()))?;
                }
                for &m in q.maxes() {
                    writer
                        .write_all(&m.to_le_bytes())
                        .map_err(|e| MemFuseError::Storage(e.to_string()))?;
                }
            }

            header = crate::persistence::HnswHeader::new_v2_with_bias(
                inner.cold.config.dimension as u32,
                inner.cold.config.m as u32,
                inner.cold.config.distance_metric as u8,
                if inner.cold.config.quantize { 1 } else { 0 },
                q_min,
                q_max,
                node_count as u64,
                entry_point.map(|i| i as i64).unwrap_or(-1),
                nodes_offset,
                connections_offset,
                inner.hot.last_tx_id.load(Ordering::SeqCst),
                cal_offset,
                cal_len,
                bias.mean_bias,
                bias.variance_bias,
            );

            writer
                .flush()
                .map_err(|e| MemFuseError::Storage(e.to_string()))?;
            let mut file = writer.into_inner().map_err(|_| {
                MemFuseError::Storage("Failed to retrieve file from BufWriter".into())
            })?;

            file.seek(std::io::SeekFrom::Start(0))
                .map_err(|e| MemFuseError::Storage(e.to_string()))?;
            file.write_all(&header.to_bytes())
                .map_err(|e| MemFuseError::Storage(e.to_string()))?;
            file.seek(std::io::SeekFrom::Start(nodes_offset))
                .map_err(|e| MemFuseError::Storage(e.to_string()))?;
            for record in &node_records {
                file.write_all(&record.to_bytes())
                    .map_err(|e| MemFuseError::Storage(e.to_string()))?;
            }
            file.sync_all()
                .map_err(|e| MemFuseError::Storage(e.to_string()))?;

            std::fs::rename(&temp_path, &path_buf).map_err(|e| {
                MemFuseError::Storage(format!("Failed to rename temporary HNSW file: {}", e))
            })?;

            if let Some(parent) = path_buf.parent() {
                if let Ok(parent_dir) = std::fs::File::open(parent) {
                    parent_dir.sync_all().map_err(|e| {
                        MemFuseError::Storage(format!(
                            "Failed to fsync parent directory after rename: {}",
                            e
                        ))
                    })?;
                }
            }

            Ok::<(), MemFuseError>(())
        })
        .await
        .map_err(|e| MemFuseError::Storage(format!("Join error: {}", e)))??;

        Ok(())
    }

    pub async fn load_mmap(&self, path: impl AsRef<std::path::Path> + Send) -> Result<()> {
        let mmap_index = crate::persistence::MmapIndex::open_async(path).await?;
        self.load_mmap_from_instance(mmap_index)
    }

    pub(crate) fn load_mmap_from_instance(
        &self,
        mmap_index: crate::persistence::MmapIndex,
    ) -> Result<()> {
        let ep = if mmap_index.header.entry_point() >= 0 {
            Some(mmap_index.header.entry_point() as usize)
        } else {
            None
        };

        let max_layer = if let Some(e) = ep {
            let record = mmap_index.get_node_record(e)?;
            record.max_layer as u64
        } else {
            0
        };

        {
            self.inner.hot.set_entry_point(ep);
            self.inner.hot.set_ram_entry_point(None);
            self.inner.hot.max_layer.store(max_layer, Ordering::SeqCst);
        }

        *self.inner.cold.sq8_bias.write() = mmap_index.header.sq8_bias();

        if mmap_index.header.is_quantized() {
            let dim = self.inner.cold.config.dimension;
            let header = &mmap_index.header;

            let mut mins = Vec::new();
            let mut maxes = Vec::new();

            if header.version() == 2
                && header.quant_calibration_offset() > 0
                && header.quant_calibration_len() as usize >= dim * 4 * 2
            {
                let offset = header.quant_calibration_offset() as usize;
                let cal_bytes = mmap_index
                    .mmap
                    .get(offset..offset + dim * 4 * 2)
                    .ok_or_else(|| {
                        MemFuseError::Storage("Calibration segment out of bounds".into())
                    })?;

                for i in 0..dim {
                    let min_bytes = &cal_bytes[i * 4..(i + 1) * 4];
                    mins.push(f32::from_le_bytes(min_bytes.try_into().map_err(|_| {
                        MemFuseError::Storage("Invalid calibration min float".into())
                    })?));
                }
                for i in 0..dim {
                    let max_bytes = &cal_bytes[(dim + i) * 4..(dim + i + 1) * 4];
                    maxes.push(f32::from_le_bytes(max_bytes.try_into().map_err(|_| {
                        MemFuseError::Storage("Invalid calibration max float".into())
                    })?));
                }
            } else {
                let q_min = header.q_min();
                let q_max = header.q_max();
                mins = vec![q_min; dim];
                maxes = vec![q_max; dim];
            }

            let mut scales = Vec::with_capacity(dim);
            let mut inv_scales = Vec::with_capacity(dim);

            for i in 0..dim {
                let mut range = maxes[i] - mins[i];
                if range.abs() < f32::EPSILON {
                    range = 1e-6;
                }
                scales.push(255.0 / range);
                inv_scales.push(range / 255.0);
            }

            let mut q_guard = self.inner.cold.quantizer.write();
            *q_guard = Some(crate::quantize::ScalarQuantizer {
                mins,
                maxes,
                scales,
                inv_scales,
                dimension: dim,
                total_queries: AtomicU64::new(0),
                out_of_range_queries: AtomicU64::new(0),
            });
        }

        let mut guard = self.inner.cold.mmap_index.write();
        self.inner
            .hot
            .last_tx_id
            .store(mmap_index.header.last_tx_id(), Ordering::SeqCst);
        *guard = Some(mmap_index);
        Ok(())
    }
}

/// Prepared insert operation containing all fallible calculations prior to state mutation.
#[derive(Debug)]
pub struct PreparedInsert {
    pub doc_id: DocId,
    pub vector_data: VectorData,
    pub new_layer: usize,
    pub new_idx: usize,
    pub final_connections: Vec<Vec<u32>>,
    pub neighbor_backlinks: Vec<NeighborBacklink>,
    pub should_update_entry_point: bool,
    pub should_update_ram_entry_point: bool,
}

/// Back-link connection update for a neighbor node at a specific layer.
#[derive(Debug)]
pub struct NeighborBacklink {
    pub neighbor_ram_idx: usize,
    pub layer: usize,
    pub updated_connections: Vec<u32>,
}

/// Batch tracking context for running state across multi-operation transaction commits.
#[derive(Debug)]
pub struct BatchContext {
    pub running_max_layer: usize,
    pub has_entry_point: bool,
    pub has_ram_entry_point: bool,
    pub backlink_map: BacklinkTable,
}

impl BatchContext {
    pub fn new(core: &HnswIndexCore) -> Self {
        Self {
            running_max_layer: core.hot.max_layer.load(Ordering::SeqCst) as usize,
            has_entry_point: core.hot.get_entry_point().is_some(),
            has_ram_entry_point: core.hot.get_ram_entry_point().is_some(),
            backlink_map: BacklinkTable::new(),
        }
    }
}

fn get_neighbor_conns_in_batch(
    core: &HnswIndexCore,
    neighbor_idx: usize,
    layer: usize,
    base_batch_idx: usize,
    mmap_node_count: usize,
    _nodes_read: &[HnswNode],
    prior_prepared: &[PreparedInsert],
    batch_ctx: &BatchContext,
) -> Vec<u32> {
    if neighbor_idx >= base_batch_idx {
        let offset = neighbor_idx - base_batch_idx;
        return prior_prepared
            .get(offset)
            .and_then(|p| p.final_connections.get(layer))
            .cloned()
            .unwrap_or_default();
    }

    if neighbor_idx < mmap_node_count {
        return Vec::new();
    }

    let neighbor_ram_idx = neighbor_idx - mmap_node_count;
    if let Some(conns) = batch_ctx.backlink_map.get(neighbor_ram_idx, layer) {
        return conns.clone();
    }

    core.hot
        .get_ram_node_connections(neighbor_ram_idx, layer, core.cold.config.m)
}

/// Helper for hybrid resolution of nodes (RAM vs Mmap, plus in-flight batch prepared inserts).
struct SearchContext<'a> {
    nodes: &'a [HnswNode],
    mmap: Option<&'a crate::persistence::MmapIndex>,
    mmap_node_count: usize,
    prior_prepared: &'a [PreparedInsert],
    backlink_map: Option<&'a BacklinkTable>,
    quantizer: Option<Cow<'a, crate::quantize::ScalarQuantizer>>,
    arena: &'a HnswArena,
}

/// Liefert den aktuellen Rebuild-Status des Index.
#[derive(Debug, Clone, PartialEq)]
pub enum RebuildStatus {
    /// Kein Rebuild läuft oder geplant.
    Idle,
    /// Rebuild läuft gerade im Hintergrund.
    Running,
    /// Rebuild wurde getriggert, startet bald.
    Pending,
}

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
        let deadline = tokio::time::Instant::now() + timeout;
        while self.hot.rebuilding.load(Ordering::Acquire) {
            if tokio::time::Instant::now() >= deadline {
                return false;
            }
            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        }
        true
    }

    fn random_layer(&self) -> usize {
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
                        memfuse_core::MemFuseError::Index("Quantizer not trained".into())
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
                    memfuse_core::MemFuseError::Index("Quantizer not trained".into())
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
                        memfuse_core::MemFuseError::Index("Quantizer not trained".into())
                    })?
                    .symmetric_dist(a, b, self.cold.config.distance_metric)
            }
            _ => Err(MemFuseError::Index(
                "Mixed vector representations (F32/U8) are not supported".into(),
            )),
        }
    }

    fn resolve_dist(
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
        Err(MemFuseError::Index(format!(
            "Node vector not found for index {idx}"
        )))
    }

    fn resolve_connections<'a>(
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

    fn resolve_doc_id(&self, idx: usize, ctx: &SearchContext) -> Result<DocId> {
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
        Err(MemFuseError::Index(format!(
            "Node doc_id not found for index {idx}"
        )))
    }

    fn search_layer(
        &self,
        query: &[f32],
        query_quantized: Option<&[u8]>,
        entry_points: &[usize],
        ef: usize,
        layer: usize,
    ) -> Result<Vec<Candidate>> {
        self.search_layer_with_context(query, query_quantized, entry_points, ef, layer, &[], None)
    }

    fn search_layer_with_context(
        &self,
        query: &[f32],
        query_quantized: Option<&[u8]>,
        entry_points: &[usize],
        ef: usize,
        layer: usize,
        prior_prepared: &[PreparedInsert],
        backlink_map: Option<&BacklinkTable>,
    ) -> Result<Vec<Candidate>> {
        let nodes_guard = self.hot.nodes.read();
        let mmap_guard = self.cold.mmap_index.read();
        let mmap_node_count = mmap_guard
            .as_ref()
            .map(|m| m.header.node_count() as usize)
            .unwrap_or(0);

        let q_guard = if self.cold.config.quantize {
            Some(self.cold.quantizer.read())
        } else {
            None
        };
        let q_ref = q_guard.as_ref().and_then(|g| g.as_ref());

        let ctx = SearchContext {
            nodes: &nodes_guard,
            mmap: mmap_guard.as_ref(),
            mmap_node_count,
            prior_prepared,
            backlink_map,
            quantizer: q_ref.map(Cow::Borrowed),
            arena: &self.hot.arena,
        };

        let mut visited = AHashSet::with_capacity(ef.saturating_mul(4));
        let mut candidates = BinaryHeap::new();
        let mut results = BinaryHeap::new();

        #[cfg(feature = "partial-index-rebuild")]
        let mut visited_node_ids = Vec::new();

        for &ep in entry_points {
            if visited.insert(ep) {
                #[cfg(feature = "partial-index-rebuild")]
                visited_node_ids.push(ep as u64);

                let dist = self.resolve_dist(ep, query, query_quantized, &ctx)?;
                let cand = Candidate {
                    index: ep,
                    distance: dist,
                };
                candidates.push(Reverse(cand));
                results.push(cand);
            }
        }

        let deleted_snapshot = self.cold.deleted_nodes.read();

        while let Some(Reverse(current)) = candidates.pop() {
            if let Some(worst_result) = results.peek() {
                if current.distance > worst_result.distance && results.len() >= ef {
                    break;
                }
            }

            let connections = self.resolve_connections(current.index, layer, &ctx)?;
            let mut has_dead_neighbors = false;

            for &neighbor_u32 in connections.iter() {
                let neighbor = neighbor_u32 as usize;
                if deleted_snapshot.contains(neighbor as u64) {
                    self.cold.visited_dead_nodes.fetch_add(1, Ordering::Relaxed);
                    has_dead_neighbors = true;
                }
                if visited.insert(neighbor) {
                    #[cfg(feature = "partial-index-rebuild")]
                    visited_node_ids.push(neighbor as u64);

                    let dist = self.resolve_dist(neighbor, query, query_quantized, &ctx)?;
                    let is_better = match results.peek() {
                        Some(worst) => dist < worst.distance,
                        None => true,
                    };

                    if is_better || results.len() < ef {
                        let cand = Candidate {
                            index: neighbor,
                            distance: dist,
                        };
                        candidates.push(Reverse(cand));
                        results.push(cand);
                        if results.len() > ef {
                            results.pop();
                        }
                    }
                }
            }

            if has_dead_neighbors && current.index >= mmap_node_count {
                let ram_idx = current.index - mmap_node_count;
                let seq_log = self.cold.seq_log.read();
                let min_retention_seq = seq_log.min_retention_seq();
                let m = self.cold.config.m;
                let layer_slice = self.hot.arena.get_ram_node_connections(ram_idx, layer, m);
                let mut kept = Vec::with_capacity(layer_slice.len());
                for &neighbor_u32 in &layer_slice {
                    if !deleted_snapshot.contains(neighbor_u32 as u64) {
                        kept.push(neighbor_u32);
                    } else if let Some(min_ret_seq) = min_retention_seq {
                        let neighbor_idx = neighbor_u32 as usize;
                        if neighbor_idx >= mmap_node_count {
                            let neighbor_ram_idx = neighbor_idx - mmap_node_count;
                            if let Some(neighbor_node) = nodes_guard.get(neighbor_ram_idx) {
                                if let Some(del_seq) = seq_log.deletion_seq(neighbor_node.doc_id) {
                                    if del_seq >= min_ret_seq {
                                        kept.push(neighbor_u32);
                                    }
                                }
                            }
                        }
                    }
                }
                self.hot
                    .arena
                    .try_relink_pruned_neighbors(ram_idx, layer, m, &kept);
            }
        }
        #[cfg(feature = "partial-index-rebuild")]
        if layer == 0 && !visited_node_ids.is_empty() {
            self.cold
                .traversal_tracker
                .write()
                .record_traversal(visited_node_ids);
        }

        let mut vec = results.into_vec();
        vec.sort_by(|a, b| a.distance.total_cmp(&b.distance));
        Ok(vec)
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
                            |_| MemFuseError::Index("Corrupt f32 in mmap vector".into()),
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

        Err(MemFuseError::Index(format!(
            "Invalid node index {} in resolve_vector_data_with_batch",
            idx
        )))
    }

    fn compute_symmetric_distance_hybrid_with_batch(
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

    fn select_neighbors_heuristic_with_batch(
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
                if fallback.len() >= m || (fallback.len() >= min_neighbors && !fallback.is_empty())
                {
                    if fallback.len() >= min_neighbors {
                        break;
                    }
                }
                if !fallback.iter().any(|c| c.index == cand.index) {
                    fallback.push(cand);
                }
            }
            return Ok(fallback.iter().map(|c| c.index as u32).collect());
        }

        Ok(result.iter().map(|c| c.index as u32).collect())
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
                    return Err(MemFuseError::Index(
                        "Fault injection: compute_insert simulated failure".into(),
                    ));
                }
            }
        }

        if vector.len() != self.cold.config.dimension {
            return Err(MemFuseError::invalid_input(format!(
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

    pub fn apply_insert(&self, prepared: PreparedInsert) {
        let node = HnswNode {
            doc_id: prepared.doc_id,
            vector: prepared.vector_data,
            max_layer: prepared.new_layer,
            committed_tx: 0,
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

        if prepared.should_update_entry_point {
            self.hot.set_entry_point(Some(prepared.new_idx));
            self.hot
                .max_layer
                .store(prepared.new_layer as u64, Ordering::SeqCst);
        }

        if prepared.should_update_ram_entry_point {
            self.hot.set_ram_entry_point(Some(prepared.new_idx));
        }
    }

    fn do_insert(&self, id: DocId, vector: &[f32]) -> Result<()> {
        let prepared = self.compute_insert(id, vector)?;
        self.apply_insert(prepared);
        Ok(())
    }

    fn do_delete(&self, id: DocId) -> Result<()> {
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

    pub fn check_connectivity(&self) -> memfuse_core::Result<()> {
        let score = self.connectivity_score();
        if score < self.cold.config.rebuild_threshold {
            let deleted_ratio = (1.0 - score) * 100.0;
            return Err(memfuse_core::MemFuseError::HnswConnectivityDegraded { deleted_ratio });
        }
        Ok(())
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
        if self.hot.rebuilding.swap(true, Ordering::SeqCst) {
            tracing::debug!("HNSW rebuild already in progress, skipping");
            return Ok(());
        }
        let _guard = RebuildGuard(&self.hot.rebuilding);

        tracing::info!("Starting HNSW index rebuild (Phase 1)");
        let start_time = std::time::Instant::now();

        let (new_index, snapshot_tx) = self.rebuild_phase1_snapshot_and_build()?;

        tracing::info!("HNSW index rebuild Phase 1 completed, starting Phase 2 merge & swap");

        self.rebuild_phase2_merge_and_swap(new_index, snapshot_tx)
            .await?;

        self.cold.rebuild_count.fetch_add(1, Ordering::SeqCst);
        tracing::info!("HNSW rebuild completed in {:?}", start_time.elapsed());
        Ok(())
    }

    pub async fn rebuild_region(&self, region_node_ids: Vec<u64>) -> Result<()> {
        if region_node_ids.is_empty() {
            return Ok(());
        }

        let _write_lock = self.hot.write_mutex.lock().await;

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
                    let is_deleted = deleted_nodes.contains(global_idx as u64);
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
                    new_index.inner.do_insert(doc_id, &v)?;
                }
                VectorData::U8(v) => {
                    let dequantized = {
                        let q = quantizer_guard.as_ref().ok_or_else(|| {
                            MemFuseError::Index("Quantizer missing during rebuild".into())
                        })?;
                        q.dequantize(&v)?
                    };
                    new_index.inner.do_insert(doc_id, &dequantized)?;
                }
            }
            let mmap_count = new_index
                .inner
                .cold
                .mmap_index
                .read()
                .as_ref()
                .map(|m| m.header.node_count() as usize)
                .unwrap_or(0);
            if let Some(&global_idx) = new_index.inner.hot.doc_to_node.read().get(&doc_id.inner()) {
                if global_idx >= mmap_count {
                    let ram_idx = global_idx - mmap_count;
                    let mut nodes = new_index.inner.hot.nodes.write();
                    if let Some(node) = nodes.get_mut(ram_idx) {
                        node.committed_tx = committed_tx;
                    }
                }
            }
            if is_deleted {
                new_index.inner.do_delete(doc_id)?;
            }
        }

        Ok((new_index, snapshot_tx))
    }

    async fn rebuild_phase2_merge_and_swap(
        &self,
        new_index: HnswIndex,
        snapshot_tx: u64,
    ) -> Result<()> {
        let _write_lock = self.hot.write_mutex.lock().await;

        let delta_changes = self.cold.seq_log.read().changes_since(snapshot_tx);

        for change in delta_changes {
            match change {
                memfuse_core::SeqLogChange::Insert { doc_id, seq } => {
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
                                    MemFuseError::Index(
                                        "Quantizer missing during rebuild phase 2 delta replay"
                                            .into(),
                                    )
                                })?;
                                q.dequantize(&v)?
                            }
                        };

                        new_index.inner.do_insert(doc_id, &f32_vec)?;

                        let mmap_count = new_index
                            .inner
                            .cold
                            .mmap_index
                            .read()
                            .as_ref()
                            .map(|m| m.header.node_count() as usize)
                            .unwrap_or(0);
                        if let Some(&global_idx) =
                            new_index.inner.hot.doc_to_node.read().get(&doc_id.inner())
                        {
                            if global_idx >= mmap_count {
                                let ram_idx = global_idx - mmap_count;
                                let mut nodes = new_index.inner.hot.nodes.write();
                                if let Some(node) = nodes.get_mut(ram_idx) {
                                    node.committed_tx = seq;
                                }
                            }
                        }
                    }
                }
                memfuse_core::SeqLogChange::Delete { doc_id, .. } => {
                    new_index.inner.do_delete(doc_id)?;
                }
            }
        }

        {
            let mut nodes = self.hot.nodes.write();
            let mut doc_to_node = self.hot.doc_to_node.write();
            let mut deleted_nodes = self.cold.deleted_nodes.write();

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
            *self.hot.arena.offsets.write() = new_offsets;
            *self.hot.arena.capacities.write() = new_capacities;
            *self.hot.arena.count_offsets.write() = new_count_offsets;
            *self.hot.arena.counts.write() = new_counts;
            *self.hot.arena.arena.write() = new_arena;

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

impl VectorIndex for HnswIndex {
    async fn insert(&self, tx: TxId, id: DocId, embedding: &[f32]) -> Result<()> {
        if let Some(ref err) = self.inner.cold.validation_error {
            return Err(MemFuseError::invalid_input(format!(
                "Invalid index configuration: {}",
                err
            )));
        }
        if embedding.len() != self.inner.cold.config.dimension {
            return Err(MemFuseError::invalid_input(format!(
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
        if k > memfuse_core::MAX_SEARCH_K {
            return Err(MemFuseError::invalid_input(format!(
                "Requested k ({}) exceeds maximum allowed search limit ({})",
                k,
                memfuse_core::MAX_SEARCH_K
            )));
        }
        if let Some(ref err) = self.inner.cold.validation_error {
            return Err(MemFuseError::invalid_input(format!(
                "Invalid index configuration: {}",
                err
            )));
        }
        if query.len() != self.inner.cold.config.dimension {
            return Err(MemFuseError::invalid_input(format!(
                "Expected dimension {}, got {}",
                self.inner.cold.config.dimension,
                query.len()
            )));
        }

        for &val in query {
            if !val.is_finite() {
                return Err(MemFuseError::invalid_input(
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
            let err = memfuse_core::MemFuseError::HnswConnectivityDegraded { deleted_ratio };
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
                    return Err(MemFuseError::Index(format!(
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
            return Err(MemFuseError::invalid_input(format!(
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
            return Err(MemFuseError::invalid_input(format!(
                "Invalid index configuration: {}",
                err
            )));
        }
        let _lock = self.inner.hot.write_mutex.lock().await;
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
                    return Err(MemFuseError::Index(format!(
                        "HNSW commit received unsupported IndexOp variant: {:?}. \
                         Add a handler arm before enabling this operation.",
                        std::mem::discriminant(other)
                    )));
                }
            }
        }

        let mut inserted_doc_ids = Vec::with_capacity(prepared_inserts.len());
        for prepared in prepared_inserts {
            inserted_doc_ids.push(prepared.doc_id);
            self.inner.apply_insert(prepared);
        }

        for doc_id in deletes_to_apply {
            self.inner.do_delete(doc_id)?;
        }

        let seq = tx.inner();
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

        if !inserted_doc_ids.is_empty() {
            let mmap_count = self
                .inner
                .cold
                .mmap_index
                .read()
                .as_ref()
                .map(|m| m.header.node_count() as usize)
                .unwrap_or(0);
            let doc_map = self.inner.hot.doc_to_node.read();
            let mut nodes = self.inner.hot.nodes.write();

            for doc_id in inserted_doc_ids {
                if let Some(&global_idx) = doc_map.get(&doc_id.inner()) {
                    if global_idx >= mmap_count {
                        let ram_idx = global_idx - mmap_count;
                        if let Some(node) = nodes.get_mut(ram_idx) {
                            node.committed_tx = tx.inner();
                        }
                    }
                }
            }
        }

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
            .store(tx.inner(), Ordering::SeqCst);
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
            return Err(MemFuseError::invalid_input(format!(
                "Invalid index configuration: {}",
                err
            )));
        }
        self.inner.cold.tx_buffer.discard(tx);
        Ok(())
    }

    async fn rollback_to_tx(&self, tx_id: TxId) -> Result<()> {
        if let Some(ref err) = self.inner.cold.validation_error {
            return Err(MemFuseError::invalid_input(format!(
                "Invalid index configuration: {}",
                err
            )));
        }

        let target = tx_id.inner();

        let _guard = self.inner.hot.write_mutex.lock().await;

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
        self.is_rebuild_required()
    }

    fn trigger_rebuild_async(&self) {
        drop(HnswIndex::trigger_rebuild_async(self));
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
            deleted_ratio: self.deleted_ratio(),
            rebuild_count: self.rebuild_count(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config(dim: usize) -> HnswConfig {
        HnswConfig {
            dimension: dim,
            max_elements: 10_000,
            m: 8,
            ef_construction: 100,
            ef_search: 64,

            distance_metric: DistanceMetric::Euclidean,
            rebuild_threshold: 0.8,
            quantize: false,
            ..Default::default()
        }
    }

    #[test]
    fn test_try_new_invalid_config_fails_immediately() {
        let config = HnswConfig {
            ef_construction: 1,
            m: 100,
            ..test_config(4)
        };
        let result = HnswIndex::try_new(config);
        assert!(
            result.is_err(),
            "try_new must fail immediately on invalid config"
        );
        let err_msg = format!("{}", result.err().unwrap());
        assert!(
            err_msg.contains("ef_construction (1) must be >= m (100)"),
            "Unexpected error message: {}",
            err_msg
        );
    }

    #[test]
    fn test_hnsw_config_builder_and_validation() {
        let config = HnswConfigBuilder::new(128)
            .max_elements(1000)
            .m(32)
            .ef_construction(128)
            .ef_search(64)
            .distance_metric(DistanceMetric::Cosine)
            .quantize(true)
            .quantizer_recalibration_sample_size(500)
            .build()
            .expect("valid builder config");

        assert_eq!(config.dimension, 128);
        assert_eq!(config.max_elements, 1000);
        assert_eq!(config.m, 32);
        assert_eq!(config.ef_construction, 128);
        assert_eq!(config.ef_search, 64);
        assert_eq!(config.distance_metric, DistanceMetric::Cosine);
        assert!(config.quantize);
        assert_eq!(config.quantizer_recalibration_sample_size, 500);

        let res_ef_c = HnswConfig {
            m: 16,
            ef_construction: 8,
            ..Default::default()
        }
        .validate();
        assert!(matches!(res_ef_c, Err(MemFuseError::InvalidInput(_))));

        let res_builder_err = HnswConfigBuilder::new(128).m(16).ef_construction(8).build();
        assert!(matches!(
            res_builder_err,
            Err(MemFuseError::InvalidInput(_))
        ));
    }

    #[tokio::test]
    async fn test_compact_seq_log() {
        let config = HnswConfig {
            dimension: 4,
            ..Default::default()
        };
        let index = HnswIndex::try_new(config).expect("valid config");
        let tx = TxId::new(1);
        let doc_id = DocId::from(1u64);
        let vec = vec![1.0, 2.0, 3.0, 4.0];

        index.insert(tx, doc_id, &vec).await.expect("insert");
        index.commit(tx).await.expect("commit");

        index.compact_seq_log(10);
        assert_eq!(index.len().await, 1);
    }

    #[tokio::test]
    async fn test_invalid_config_error() {
        let config = HnswConfig {
            ef_construction: 5,
            m: 10,
            ..test_config(4)
        };
        #[allow(deprecated)]
        let index = HnswIndex::new(config);
        let tx = TxId::new(1);
        let result = index
            .insert(tx, DocId::from(1u64), &[1.0, 0.0, 0.0, 0.0])
            .await;
        assert!(result.is_err());
        let err_msg = format!("{}", result.unwrap_err());
        assert!(err_msg.contains("Invalid index configuration"));
        assert!(err_msg.contains("ef_construction (5) must be >= m (10)"));
    }

    #[tokio::test]
    async fn test_insert_and_search() {
        let index = HnswIndex::try_new(test_config(4)).unwrap();
        let tx = TxId::new(1);

        index
            .insert(tx, DocId::from(1u64), &[1.0, 0.0, 0.0, 0.0])
            .await
            .expect("insert 1");
        index
            .insert(tx, DocId::from(2u64), &[0.0, 1.0, 0.0, 0.0])
            .await
            .expect("insert 2");
        index
            .insert(tx, DocId::from(3u64), &[0.9, 0.1, 0.0, 0.0])
            .await
            .expect("insert 3");
        index.commit(tx).await.expect("commit");

        let results = index
            .search(&[1.0, 0.0, 0.0, 0.0], 2)
            .await
            .expect("search");
        assert!(!results.is_empty());
        assert_eq!(results[0].doc_id, DocId::from(1u64));
    }

    #[tokio::test]
    async fn test_delete() {
        let index = HnswIndex::try_new(test_config(4)).unwrap();

        let tx1 = TxId::new(1);
        index
            .insert(tx1, DocId::from(1u64), &[1.0, 0.0, 0.0, 0.0])
            .await
            .expect("insert");
        index.commit(tx1).await.expect("commit");

        assert_eq!(index.len().await, 1);

        let tx2 = TxId::new(2);
        index.delete(tx2, DocId::from(1u64)).await.expect("delete");
        index.commit(tx2).await.expect("commit");

        assert_eq!(index.len().await, 0);
    }
}
