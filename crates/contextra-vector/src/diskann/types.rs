// FILE-CONTEXT
// ZWECK: DiskANN-Graphindex für Out-of-Core Approximate Nearest Neighbor Search (WP-4.3).
// INVARIANTEN: Lock-Hierarchie: header -> mmap -> cache / quantizer / doc_ids; atomic rename + parent dir sync bei file persistence.
// NICHT-OFFENSICHTLICH: Mmap für Vektor- & Graphlesezugriffe, ausgelagert an contextra-sys::mmap_readonly.
// STAND: TS:2026-09-10T19:30:00Z (SESSION: a9d67eae)
use super::build::TombstoneSet;
use super::config::{CachedNode, DiskAnnConfig};
use super::format::DiskAnnHeader;
use crate::ComputePool;
use ahash::AHashMap;
use contextra_core::{ContextraError, DocId, Result};
use memmap2::Mmap;
use parking_lot::RwLock;
use std::io::Read;
use std::sync::atomic::{AtomicBool, AtomicU64};
use std::sync::Arc;

/// DiskANN out-of-core vector index.
///
/// This index uses `memmap2` to offload vector data and graph edges to disk.
/// Note that mmap operations and page faults can cause the current thread to block
/// while data is being loaded from the disk. For high-concurrency environments,
/// consider using `ComputePool` or a background thread pool when calling methods on this index
/// if latency spikes are a concern.
pub struct DiskAnnIndex {
    pub(crate) inner: Arc<DiskAnnIndexInner>,
}

pub(crate) struct DiskAnnIndexInner {
    pub(crate) config: DiskAnnConfig,
    pub(crate) header: RwLock<Option<DiskAnnHeader>>,
    pub(crate) mmap: RwLock<Option<Mmap>>,
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

        let vector_size = if config.quantize {
            config.dimension
        } else {
            config.dimension * 4
        };
        let neighbors_size = 4 + (config.max_degree * 4);
        let doc_id_size = 8;
        let raw_node_size = vector_size + neighbors_size + doc_id_size;
        let node_size_bytes = raw_node_size.div_ceil(config.sector_size) * config.sector_size;

        if config.index_path.exists() {
            let mut file = std::fs::File::open(&config.index_path).map_err(ContextraError::Io)?;
            let metadata = file.metadata().map_err(ContextraError::Io)?;
            let file_len = metadata.len() as usize;

            if file_len < DiskAnnHeader::SIZE {
                return Err(ContextraError::Storage(
                    "DiskANN file too small for header".into(),
                ));
            }

            let mut header_buf = [0u8; DiskAnnHeader::SIZE];
            file.read_exact(&mut header_buf)
                .map_err(ContextraError::Io)?;

            let header = DiskAnnHeader::try_from_bytes(&header_buf)?;

            if config.sector_size != header.sector_size as usize {
                return Err(ContextraError::Index(format!(
                    "DiskANN-Index inkompatibel: Config-sector_size={} stimmt nicht mit \
                     Header-sector_size={} überein. Index muss neu aufgebaut werden.",
                    config.sector_size, header.sector_size
                )));
            }

            let header_sector_size = header.sector_size as usize;
            if header_sector_size == 0 {
                return Err(ContextraError::Storage(
                    "DiskANN header corrupt: sector_size is 0".into(),
                ));
            }

            let header_vector_size = if header.quantized != 0 {
                header.dimension as usize
            } else {
                header.dimension as usize * 4
            };
            let header_neighbors_size = 4 + (header.max_degree as usize * 4);
            let header_raw_node_size = header_vector_size + header_neighbors_size + doc_id_size;
            let header_node_size_bytes =
                header_raw_node_size.div_ceil(header_sector_size) * header_sector_size;
            let start_offset =
                DiskAnnHeader::SIZE.div_ceil(header_sector_size) * header_sector_size;
            let expected_min_size = start_offset.saturating_add(
                (header.node_count as usize).saturating_mul(header_node_size_bytes),
            );
            if file_len < expected_min_size {
                return Err(ContextraError::Storage(format!(
                    "DiskANN file truncated or corrupt node_count: file len {}, expected at least {}",
                    file_len, expected_min_size
                )));
            }
        }

        Ok(Self {
            inner: Arc::new(DiskAnnIndexInner {
                config,
                header: RwLock::new(None),
                mmap: RwLock::new(None),
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
