// FILE-CONTEXT
// ZWECK: DiskANN-Graphindex für Out-of-Core Approximate Nearest Neighbor Search (WP-4.3).
// INVARIANTEN: Lock-Hierarchie: header -> vfs_file -> cache / quantizer / doc_ids; atomic rename + parent dir sync bei file persistence.
// NICHT-OFFENSICHTLICH: Positional VFS-Lesevorgänge über contextra_ports::Vfs.
// STAND: TS:2026-09-10T19:30:00Z (SESSION: a9d67eae)
use crate::ComputePool;
use contextra_ports::Vfs;
use contextra_types::{DistanceMetric, DocId};
use std::path::PathBuf;
use std::sync::Arc;

/// Fallback behavior policy when DiskANN loading, integrity checks, or reads fail.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DiskAnnFallbackPolicy {
    /// Transparently fall back to in-memory HNSW index on load failure or detected corruption.
    #[default]
    UseHnswOnFailure,
    /// Immediately return a typed error on load failure or index corruption.
    FailFast,
}

/// Configuration for DiskANN index.
#[derive(Clone)]
pub struct DiskAnnConfig {
    /// Path to the on-disk index file.
    pub index_path: PathBuf,
    /// Vector dimension.
    pub dimension: usize,
    /// Maximum graph degree (R).
    pub max_degree: usize,
    /// Beam width for search (W).
    pub beam_width: usize,
    /// Sector size for aligned I/O (typically 4096).
    pub sector_size: usize,
    /// Maximum memory budget in bytes for in-memory caching.
    pub memory_budget: usize,
    /// Distance metric.
    pub distance_metric: DistanceMetric,
    /// Whether to use SQ8 quantization.
    pub quantize: bool,
    /// Fallback policy when index file loading or integrity validation fails.
    pub fallback_policy: DiskAnnFallbackPolicy,
    /// Optionaler Override für den Pending-Flush-Threshold (für Benchmarks/Tests).
    /// None: adaptiver Threshold gemäß ADR-068 (max(50, min(1000, floor(N × 0.05)))).
    pub pending_flush_threshold: Option<u64>,
    /// Optional compute pool for background operations.
    pub compute_pool: Option<ComputePool>,
    /// Injiziertes Virtual File System (VFS).
    pub vfs: Option<Arc<dyn Vfs>>,
}

impl std::fmt::Debug for DiskAnnConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DiskAnnConfig")
            .field("index_path", &self.index_path)
            .field("dimension", &self.dimension)
            .field("max_degree", &self.max_degree)
            .field("beam_width", &self.beam_width)
            .field("sector_size", &self.sector_size)
            .field("memory_budget", &self.memory_budget)
            .field("distance_metric", &self.distance_metric)
            .field("quantize", &self.quantize)
            .field("fallback_policy", &self.fallback_policy)
            .field("pending_flush_threshold", &self.pending_flush_threshold)
            .field("compute_pool", &self.compute_pool)
            .field("vfs", &self.vfs.as_ref().map(|_| "Arc<dyn Vfs>"))
            .finish()
    }
}

impl DiskAnnConfig {
    /// Erzeugt eine Konfiguration mit injiziertem VFS.
    pub fn new_with_vfs(index_path: PathBuf, vfs: Arc<dyn Vfs>) -> Self {
        Self {
            index_path,
            vfs: Some(vfs),
            ..Default::default()
        }
    }
}

impl Default for DiskAnnConfig {
    fn default() -> Self {
        Self {
            index_path: PathBuf::from("diskann.idx"),
            dimension: 128,
            max_degree: 64,
            beam_width: 8,
            sector_size: 4096,
            memory_budget: 128 * 1024 * 1024, // 128MB
            distance_metric: DistanceMetric::Cosine,
            quantize: false,
            fallback_policy: DiskAnnFallbackPolicy::default(),
            pending_flush_threshold: None,
            compute_pool: None,
            vfs: None,
        }
    }
}

/// A node in the DiskANN graph (Cached).
#[derive(Debug, Clone)]
pub(crate) struct CachedNode {
    pub(crate) vector: VectorData,
    pub(crate) neighbors: Vec<u32>,
    pub(crate) doc_id: DocId,
}

#[derive(Debug, Clone)]
pub(crate) enum VectorData {
    F32(Vec<f32>),
    U8(Vec<u8>),
}
