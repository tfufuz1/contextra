// FILE-CONTEXT
// ZWECK: DiskANN-Graphindex für Out-of-Core Approximate Nearest Neighbor Search (WP-4.3).
// INVARIANTEN: Lock-Hierarchie: header -> vfs_file -> cache / quantizer / doc_ids; atomic rename + parent dir sync bei file persistence.
// NICHT-OFFENSICHTLICH: Positional VFS-Lesevorgänge über contextra_ports::Vfs.
// STAND: TS:2026-09-10T19:30:00Z (SESSION: a9d67eae)
use super::build::TombstoneSet;
use super::config::{CachedNode, DiskAnnConfig};
use super::format::DiskAnnHeader;
use super::test_vfs::InMemoryVfs;
use crate::ComputePool;
use ahash::AHashMap;
use contextra_ports::{Vfs, VfsFile, VfsOpenOptions};
use contextra_types::{ContextraError, DocId, Result};
use parking_lot::RwLock;
use std::sync::atomic::{AtomicBool, AtomicU64};
use std::sync::Arc;

/// DiskANN out-of-core vector index.
///
/// This index uses `contextra_ports::Vfs` to read vector data and graph edges from disk.
pub struct DiskAnnIndex {
    pub(crate) inner: Arc<DiskAnnIndexInner>,
}

pub(crate) struct DiskAnnIndexInner {
    pub(crate) config: DiskAnnConfig,
    pub(crate) vfs: Arc<dyn Vfs>,
    pub(crate) header: RwLock<Option<DiskAnnHeader>>,
    pub(crate) vfs_file: RwLock<Option<Box<dyn VfsFile>>>,
    pub(crate) node_size_bytes: AtomicU64,
    pub(crate) cache: RwLock<AHashMap<u32, CachedNode>>,
    pub(crate) doc_ids: RwLock<Vec<DocId>>,
    pub(crate) quantizer: RwLock<Option<crate::quantize::ScalarQuantizer>>,
    pub(crate) drift_warn_count: AtomicU64,
    /// Inkrementelle Einfügungen vor dem nächsten persist_delta().
    /// Geschützt durch RwLock — Hot-Path schreibt, persist_delta liest+leert.
    pub(crate) pending_inserts: RwLock<Vec<(DocId, Vec<f32>)>>,
    /// Monotoner Zähler (AtomicU64 für Threshold-Check ohne Lock).
    pub(crate) pending_count: AtomicU64,
    /// Flag um überlappende persist_delta-Hintergrundläufe zu verhindern.
    pub(crate) flushing_in_progress: AtomicBool,
    pub(crate) hnsw_fallback: RwLock<Option<Arc<crate::hnsw::HnswIndex>>>,
    pub(crate) tombstones: RwLock<TombstoneSet>,
    pub(crate) compute_pool: ComputePool,
}

impl Clone for DiskAnnIndex {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_try_new_non_existent_file_succeeds() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("non_existent.dann");
        let config = DiskAnnConfig {
            index_path: path,
            dimension: 4,
            max_degree: 8,
            sector_size: 512,
            ..Default::default()
        };
        let index_res = DiskAnnIndex::try_new(config);
        assert!(index_res.is_ok());
    }

    #[test]
    fn test_try_new_corrupt_magic_fails_immediately() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("corrupt_magic.dann");
        std::fs::write(
            &path,
            b"BADM_corrupt_header_data_bytes_1234567890_padding_bytes",
        )
        .unwrap();

        let config = DiskAnnConfig {
            index_path: path,
            dimension: 4,
            max_degree: 8,
            sector_size: 512,
            ..Default::default()
        };

        let res = DiskAnnIndex::try_new(config);
        assert!(res.is_err());
        let err = res.err().unwrap();
        match err {
            ContextraError::Storage(msg) => assert!(
                msg.contains("bad magic"),
                "Expected bad magic error, got: {msg}"
            ),
            other => panic!("Expected Storage error with bad magic, got {:?}", other),
        }
    }

    #[test]
    fn test_try_new_truncated_file_fails_immediately() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("truncated.dann");
        // Only write 10 bytes (smaller than DiskAnnHeader::SIZE = 40)
        std::fs::write(&path, b"DANN_small").unwrap();

        let config = DiskAnnConfig {
            index_path: path,
            dimension: 4,
            max_degree: 8,
            sector_size: 512,
            ..Default::default()
        };

        let res = DiskAnnIndex::try_new(config);
        assert!(res.is_err());
        let err = res.err().unwrap();
        match err {
            ContextraError::Storage(msg) => assert!(
                msg.contains("Header too small") || msg.contains("too small for header"),
                "Expected Header too small error, got: {msg}"
            ),
            other => panic!(
                "Expected Storage error with Header too small, got {:?}",
                other
            ),
        }
    }

    #[test]
    fn test_try_new_valid_existing_file_succeeds() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("valid_index.dann");
        let config = DiskAnnConfig {
            index_path: path.clone(),
            dimension: 4,
            max_degree: 8,
            sector_size: 512,
            ..Default::default()
        };

        let index = DiskAnnIndex::try_new(config.clone()).unwrap();
        index
            .build_sync(&[vec![1.0, 0.0, 0.0, 0.0]], &[DocId::from(1u64)])
            .unwrap();

        // Re-open with try_new
        let reloaded_res = DiskAnnIndex::try_new(config);
        assert!(reloaded_res.is_ok());
    }
}

#[derive(Clone, Debug)]
pub(crate) struct SearchCandidate {
    pub(crate) index: u32,
    pub(crate) distance: f32,
}

impl PartialEq for SearchCandidate {
    fn eq(&self, other: &Self) -> bool {
        self.distance == other.distance
    }
}
impl Eq for SearchCandidate {}
impl PartialOrd for SearchCandidate {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for SearchCandidate {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.distance.total_cmp(&other.distance)
    }
}

impl DiskAnnIndex {
    pub fn try_new(config: DiskAnnConfig) -> Result<Self> {
        if !config.sector_size.is_power_of_two() {
            return Err(ContextraError::InvalidInput(
                "Sector size must be a power of 2".to_string(),
            ));
        }

        let compute_pool = config.compute_pool.clone().unwrap_or_default();

        let vfs: Arc<dyn Vfs> = match &config.vfs {
            Some(v) => Arc::clone(v),
            None => Arc::new(InMemoryVfs::new()),
        };

        let vector_size = if config.quantize {
            config.dimension
        } else {
            config.dimension * 4
        };
        let neighbors_size = 4 + (config.max_degree * 4);
        let doc_id_size = 8;
        let raw_node_size = vector_size + neighbors_size + doc_id_size;
        let node_size_bytes = raw_node_size.div_ceil(config.sector_size) * config.sector_size;

        if config.index_path.extension().and_then(|e| e.to_str()) == Some("dann")
            && vfs.exists(&config.index_path)?
        {
            let file_len = vfs.len(&config.index_path)? as usize;

            if file_len < DiskAnnHeader::SIZE {
                return Err(ContextraError::Storage(
                    "DiskANN file too small for header".into(),
                ));
            }

            let file_handle = vfs.open(&config.index_path, &VfsOpenOptions::read_only())?;
            let mut header_buf = [0u8; DiskAnnHeader::SIZE];
            let read_bytes = file_handle
                .read_at(&mut header_buf, 0)
                .map_err(ContextraError::Io)?;
            if read_bytes < DiskAnnHeader::SIZE {
                return Err(ContextraError::Storage(
                    "DiskANN file too small for header".into(),
                ));
            }

            if &header_buf[0..4] != crate::diskann::format::DISKANN_MAGIC {
                return Err(ContextraError::Storage(
                    "Invalid DiskANN file: bad magic".into(),
                ));
            }
        }

        Ok(Self {
            inner: Arc::new(DiskAnnIndexInner {
                config,
                vfs,
                header: RwLock::new(None),
                vfs_file: RwLock::new(None),
                node_size_bytes: AtomicU64::new(node_size_bytes as u64),
                cache: RwLock::new(AHashMap::new()),
                doc_ids: RwLock::new(Vec::new()),
                quantizer: RwLock::new(None),
                drift_warn_count: AtomicU64::new(0),
                pending_inserts: RwLock::new(Vec::new()),
                pending_count: AtomicU64::new(0),
                flushing_in_progress: AtomicBool::new(false),
                hnsw_fallback: RwLock::new(None),
                tombstones: RwLock::new(TombstoneSet::new()),
                compute_pool,
            }),
        })
    }

    /// Stößt `persist_delta()` asynchron im Tokio-Hintergrund-Task an,
    /// falls nicht bereits ein persist_delta-Lauf aktiv ist.
    pub fn len_sync(&self) -> usize {
        self.inner.doc_ids.read().len()
    }

    pub(crate) fn check_quantizer_drift(&self, vector: &[f32]) {
        let q_guard = self.inner.quantizer.read();
        if let Some(ref q) = *q_guard {
            let drift = q.calculate_vector_drift(vector);
            if drift > 0.10 {
                use std::sync::atomic::Ordering;
                let count = self.inner.drift_warn_count.fetch_add(1, Ordering::Relaxed) + 1;
                tracing::warn!(
                    drift = %format!("{:.1}%", drift * 100.0),
                    warn_count = count,
                    "Quantization drift > 10% detected — ScalarQuantizer recalibration recommended."
                );
            }
        }
    }
}
