use roaring::RoaringTreemap;
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};

use ahash::AHashMap;
use contextra_core::{ContextraError, DocId, Result, TxBuffer};
use parking_lot::{Mutex, RwLock};

use super::arena::HnswArena;
use super::config::HnswConfig;
use super::sq8_bias::Sq8Bias;

/// Sentinel-Wert für ungültigen / nicht gesetzten Entry-Point in HNSW.
pub const SENTINEL_NO_ENTRY_POINT: u32 = u32::MAX;

/// Standard-Löschanteil (0.10 = 10 % gelöschte Knoten), ab dem ein Rebuild getriggert wird.
pub const HNSW_REBUILD_DELETION_RATIO: f64 = 0.10;

#[derive(Debug, Clone)]
pub enum VectorData {
    /// Standard 32-bit floating point vectors.
    F32(Vec<f32>),
    /// 8-bit quantized vectors (SQ8).
    U8(Vec<u8>),
}

/// A node in the HNSW graph.
#[derive(Debug)]
pub struct HnswNode {
    pub(super) doc_id: DocId,
    pub(super) vector: VectorData,
    pub(super) max_layer: usize,
    pub(super) committed_tx: u64,
}

/// Search candidate.
#[derive(Clone, Copy, Debug)]
pub struct Candidate {
    pub index: usize,
    pub distance: f32,
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

pub struct RebuildGuard<'a>(pub &'a AtomicBool);

impl<'a> Drop for RebuildGuard<'a> {
    fn drop(&mut self) {
        self.0.store(false, Ordering::SeqCst);
    }
}

pub struct SnapshotPinGuard<'a>(pub &'a HnswIndexCore, u64);

impl<'a> SnapshotPinGuard<'a> {
    pub(super) fn new(core: &'a HnswIndexCore, seq_no: u64) -> Self {
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
    pub(super) inner: std::sync::Arc<HnswIndexCore>,
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
    pub seq_log: RwLock<contextra_core::SequenceLog>,
    pub rebuild_count: AtomicU64,
    pub visited_dead_nodes: AtomicU64,
    pub deleted_nodes: RwLock<RoaringTreemap>,
    pub compute_pool: crate::compute_pool::ComputePool,
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

#[derive(Debug, Clone, PartialEq)]
pub enum RebuildStatus {
    /// Kein Rebuild läuft oder geplant.
    Idle,
    /// Rebuild läuft gerade im Hintergrund.
    Running,
    /// Rebuild wurde getriggert, startet bald.
    Pending,
}

impl HnswIndex {
    /// Creates a new HNSW index, validating configuration upfront.
    pub fn try_new(config: HnswConfig) -> Result<Self> {
        config.validate()?;
        let compute_pool = config.compute_pool.clone().unwrap_or_default();
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
                    seq_log: RwLock::new(contextra_core::SequenceLog::new()),
                    rebuild_count: AtomicU64::new(0),
                    visited_dead_nodes: AtomicU64::new(0),
                    deleted_nodes: RwLock::new(RoaringTreemap::new()),
                    compute_pool,
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
        let compute_pool = config.compute_pool.clone().unwrap_or_default();
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
                    seq_log: RwLock::new(contextra_core::SequenceLog::new()),
                    rebuild_count: AtomicU64::new(0),
                    visited_dead_nodes: AtomicU64::new(0),
                    deleted_nodes: RwLock::new(RoaringTreemap::new()),
                    compute_pool,
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

    pub fn check_connectivity(&self) -> contextra_core::Result<()> {
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
    pub fn check_and_trigger_partial_rebuild(&self) {
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
            self.inner.cold.compute_pool.execute(move || {
                let res = inner.rebuild_region_sync(region_node_ids);
                if let Err(ref e) = res {
                    tracing::error!("Failed local partial rebuild: {}", e);
                }
            });
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

    pub fn rebuild_status(&self) -> RebuildStatus {
        self.inner.rebuild_status()
    }

    pub async fn wait_for_rebuild(&self) -> bool {
        self.inner.wait_for_rebuild().await
    }

    pub async fn wait_for_rebuild_with_timeout(&self, timeout: std::time::Duration) -> bool {
        self.inner.wait_for_rebuild_with_timeout(timeout).await
    }

    pub async fn load_mmap(&self, path: impl AsRef<std::path::Path> + Send) -> Result<()> {
        let mmap_index = crate::persistence::MmapIndex::open_async(path).await?;
        self.load_mmap_from_instance(mmap_index)
    }

    pub(super) fn load_mmap_from_instance(
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
                        ContextraError::Storage("Calibration segment out of bounds".into())
                    })?;

                for i in 0..dim {
                    let min_bytes = &cal_bytes[i * 4..(i + 1) * 4];
                    mins.push(f32::from_le_bytes(min_bytes.try_into().map_err(|_| {
                        ContextraError::Storage("Invalid calibration min float".into())
                    })?));
                }
                for i in 0..dim {
                    let max_bytes = &cal_bytes[(dim + i) * 4..(dim + i + 1) * 4];
                    maxes.push(f32::from_le_bytes(max_bytes.try_into().map_err(|_| {
                        ContextraError::Storage("Invalid calibration max float".into())
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

    pub async fn save(&self, path: impl AsRef<std::path::Path>) -> Result<()> {
        let _lock = self.inner.hot.write_mutex.lock();
        let inner = &self.inner;
        let path_buf = path.as_ref().to_path_buf();

        use std::io::{Seek, Write};

        let nodes = inner.hot.nodes.read();
        let entry_point = inner.hot.get_entry_point();
        let q_guard = inner.cold.quantizer.read();

        let temp_path = path_buf.with_extension("hnsw.tmp");
        let file = std::fs::File::create(&temp_path).map_err(|e| {
            ContextraError::Storage(format!("Failed to create temporary HNSW file: {}", e))
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
            .map_err(|e| ContextraError::Storage(e.to_string()))?;

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
                .map_err(|e| ContextraError::Storage(e.to_string()))?;
        }

        let mut current_pos = vectors_offset;
        for (i, node) in nodes.iter().enumerate() {
            node_records[i].doc_id = node.doc_id.inner();
            node_records[i].max_layer = node.max_layer as u8;
            node_records[i].vector_offset = current_pos;

            match &node.vector {
                VectorData::F32(v) => {
                    for &val in v.iter() {
                        writer
                            .write_all(&val.to_le_bytes())
                            .map_err(|e| ContextraError::Storage(e.to_string()))?;
                    }
                    current_pos += (v.len() * 4) as u64;
                }
                VectorData::U8(v) => {
                    writer
                        .write_all(v.as_slice())
                        .map_err(|e| ContextraError::Storage(e.to_string()))?;
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
                .map_err(|e| ContextraError::Storage(e.to_string()))?;
        }

        let mut conn_pos = connections_offset;
        for (i, node) in nodes.iter().enumerate() {
            node_records[i].connections_offset = conn_pos;
            let num_layers = (node.max_layer + 1) as u8;
            writer
                .write_all(&[num_layers])
                .map_err(|e| ContextraError::Storage(e.to_string()))?;
            conn_pos += 1;

            for layer in 0..num_layers as usize {
                let layer_conns = inner
                    .hot
                    .get_ram_node_connections(i, layer, inner.cold.config.m);
                let len = layer_conns.len() as u32;
                writer
                    .write_all(&len.to_le_bytes())
                    .map_err(|e| ContextraError::Storage(e.to_string()))?;
                for &conn in layer_conns.iter() {
                    writer
                        .write_all(&conn.to_le_bytes())
                        .map_err(|e| ContextraError::Storage(e.to_string()))?;
                }
                conn_pos += 4 + (len as u64) * 4;
            }
        }

        let cal_offset = conn_pos;
        let mut cal_len = 0u32;

        if let Some(ref q) = *q_guard {
            let dim = q.mins.len();
            cal_len = (dim * 4 * 2) as u32;
            for &m in &q.mins {
                writer
                    .write_all(&m.to_le_bytes())
                    .map_err(|e| ContextraError::Storage(e.to_string()))?;
            }
            for &m in &q.maxes {
                writer
                    .write_all(&m.to_le_bytes())
                    .map_err(|e| ContextraError::Storage(e.to_string()))?;
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
            .map_err(|e| ContextraError::Storage(e.to_string()))?;
        let mut file = writer.into_inner().map_err(|_| {
            ContextraError::Storage("Failed to retrieve file from BufWriter".into())
        })?;

        file.seek(std::io::SeekFrom::Start(0))
            .map_err(|e| ContextraError::Storage(e.to_string()))?;
        file.write_all(&header.to_bytes())
            .map_err(|e| ContextraError::Storage(e.to_string()))?;
        file.seek(std::io::SeekFrom::Start(nodes_offset))
            .map_err(|e| ContextraError::Storage(e.to_string()))?;
        for record in &node_records {
            file.write_all(&record.to_bytes())
                .map_err(|e| ContextraError::Storage(e.to_string()))?;
        }
        file.sync_all()
            .map_err(|e| ContextraError::Storage(e.to_string()))?;

        std::fs::rename(&temp_path, &path_buf).map_err(|e| {
            ContextraError::Storage(format!("Failed to rename temporary HNSW file: {}", e))
        })?;

        if let Some(parent) = path_buf.parent() {
            if let Ok(parent_dir) = std::fs::File::open(parent) {
                parent_dir.sync_all().map_err(|e| {
                    ContextraError::Storage(format!(
                        "Failed to fsync parent directory after rename: {}",
                        e
                    ))
                })?;
            }
        }

        Ok(())
    }
}
