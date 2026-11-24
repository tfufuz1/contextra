// FILE-CONTEXT
// ZWECK: Konstruktoren und Initialisierungslogik fuer HnswIndex.
// INVARIANTEN: Determinismus B11: Zeit- und Zufallsquellen koennen explizit injiziert werden.
// HOTSPOTS: HnswIndex::try_new_with_rng_and_clock
// STAND: TS:2026-11-12T00:00:00Z

use roaring::RoaringTreemap;
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64};

use parking_lot::{Mutex, RwLock};

use crate::distance::DistanceKernel;
use crate::error::Result;

use super::core::{HnswColdCore, HnswHotCore, HnswIndexCore};
use super::data::{DEFAULT_LAYER_SEED, SENTINEL_NO_ENTRY_POINT};
use super::index::HnswIndex;
use crate::hnsw::arena::HnswArena;
use crate::hnsw::config::HnswConfig;
use crate::hnsw::sq8_bias::Sq8Bias;

impl HnswIndex {
    /// Creates a new HNSW index with explicit random number generator and clock injection for full determinism (B11).
    // ANCHOR[contextra-vector:HNSW-NEW-RNG-CLOCK]
    pub fn try_new_with_rng_and_clock(
        config: HnswConfig,
        rng: std::sync::Arc<dyn crate::types::Rng>,
        clock: std::sync::Arc<dyn crate::types::Clock>,
    ) -> Result<Self> {
        config.validate()?;
        let compute_pool = config.compute_pool.clone().unwrap_or_default();
        let ml = 1.0 / (config.m as f64).ln();
        #[cfg(feature = "partial-index-rebuild")]
        let partial_rebuild_config = config.partial_rebuild_config.clone();

        Ok(Self {
            _kernel: std::marker::PhantomData,
            inner: std::sync::Arc::new(HnswIndexCore {
                hot: HnswHotCore {
                    nodes: RwLock::new(Vec::new()),
                    doc_to_node: RwLock::new(BTreeMap::new()),
                    entry_point: AtomicU32::new(SENTINEL_NO_ENTRY_POINT),
                    ram_entry_point: AtomicU32::new(SENTINEL_NO_ENTRY_POINT),
                    max_layer: AtomicU64::new(0),
                    ml,
                    layer_seed: AtomicU64::new(DEFAULT_LAYER_SEED),
                    deleted_count: AtomicU64::new(0),
                    write_mutex: Mutex::new(()),
                    last_tx_id: AtomicU64::new(0),
                    rebuilding: AtomicBool::new(false),
                    rebuild_condvar: parking_lot::Condvar::new(),
                    rebuild_mutex: Mutex::new(()),
                    arena: HnswArena::new(),
                },
                cold: HnswColdCore {
                    config,
                    validation_error: None,
                    tx_buffer: crate::hnsw::staging::VectorTxBuffer::new(
                        10_000,
                        std::time::Duration::from_secs(60),
                    ),
                    quantizer: RwLock::new(None),
                    sq8_bias: RwLock::new(Sq8Bias::default()),
                    mmap_index: RwLock::new(None),
                    seq_log: RwLock::new(contextra_mvcc::SequenceLog::new()),
                    rebuild_count: AtomicU64::new(0),
                    visited_dead_nodes: AtomicU64::new(0),
                    deleted_nodes: RwLock::new(RoaringTreemap::new()),
                    compute_pool,
                    rng,
                    clock: RwLock::new(clock),
                    metrics_sink: RwLock::new(std::sync::Arc::new(crate::types::NoopMetricsSink)),
                    last_rebuild_nanos: AtomicU64::new(0),
                    consecutive_rapid_rebuilds: AtomicU32::new(0),
                    #[cfg(feature = "partial-index-rebuild")]
                    traversal_tracker: if partial_rebuild_config.traversal_window > 0 {
                        Some(RwLock::new(crate::partial_rebuild::TraversalTracker::new(
                            partial_rebuild_config,
                        )))
                    } else {
                        None
                    },
                    #[cfg(test)]
                    fault_injection_insert_target: AtomicU64::new(0),
                    #[cfg(test)]
                    fault_injection_insert_count: AtomicU64::new(0),
                },
            }),
        })
    }

    /// Creates a new HNSW index with an explicit random number generator for deterministic testing or custom seeding.
    ///
    /// Uses [`MonotonicCounterClock`] by default (never `SystemTime`).
    pub fn try_new_with_rng(
        config: HnswConfig,
        rng: std::sync::Arc<dyn crate::types::Rng>,
    ) -> Result<Self> {
        let clock = std::sync::Arc::new(crate::hnsw::clock::MonotonicCounterClock::default());
        Self::try_new_with_rng_and_clock(config, rng, clock)
    }

    /// Creates a new HNSW index, validating configuration upfront.
    ///
    /// Uses an internal, non-cryptographic, reproducible default RNG seeded with `DEFAULT_LAYER_SEED`
    /// and a deterministic [`MonotonicCounterClock`].
    pub fn try_new(config: HnswConfig) -> Result<Self> {
        let rng = std::sync::Arc::new(crate::types::SeededRng::new(DEFAULT_LAYER_SEED));
        Self::try_new_with_rng(config, rng)
    }

    /// Creates a new HNSW index with a custom distance kernel.
    pub fn try_new_with_kernel(
        config: HnswConfig,
        _kernel: std::sync::Arc<dyn DistanceKernel>,
    ) -> Result<Self> {
        let rng = std::sync::Arc::new(crate::types::SeededRng::new(DEFAULT_LAYER_SEED));
        Self::try_new_with_rng(config, rng)
    }

    /// Creates a new HNSW index.
    #[deprecated(
        note = "Nutze try_new() für sofortige Fehlererkennung — new() versteckt Konfigurationsfehler bis zum ersten insert()/search()"
    )]
    pub fn new(config: HnswConfig) -> Self {
        let validation_error = config.validate().err().map(|e| e.to_string());
        let compute_pool = config.compute_pool.clone().unwrap_or_default();
        let ml = 1.0 / (config.m as f64).ln();
        let rng: std::sync::Arc<dyn crate::types::Rng> =
            std::sync::Arc::new(crate::types::SeededRng::new(DEFAULT_LAYER_SEED));
        let clock: std::sync::Arc<dyn crate::types::Clock> =
            std::sync::Arc::new(crate::hnsw::clock::MonotonicCounterClock::default());
        #[cfg(feature = "partial-index-rebuild")]
        let partial_rebuild_config = config.partial_rebuild_config.clone();

        Self {
            _kernel: std::marker::PhantomData,
            inner: std::sync::Arc::new(HnswIndexCore {
                hot: HnswHotCore {
                    nodes: RwLock::new(Vec::new()),
                    doc_to_node: RwLock::new(BTreeMap::new()),
                    entry_point: AtomicU32::new(SENTINEL_NO_ENTRY_POINT),
                    ram_entry_point: AtomicU32::new(SENTINEL_NO_ENTRY_POINT),
                    max_layer: AtomicU64::new(0),
                    ml,
                    layer_seed: AtomicU64::new(DEFAULT_LAYER_SEED),
                    deleted_count: AtomicU64::new(0),
                    write_mutex: Mutex::new(()),
                    last_tx_id: AtomicU64::new(0),
                    rebuilding: AtomicBool::new(false),
                    rebuild_condvar: parking_lot::Condvar::new(),
                    rebuild_mutex: Mutex::new(()),
                    arena: HnswArena::new(),
                },
                cold: HnswColdCore {
                    config,
                    validation_error,
                    tx_buffer: crate::hnsw::staging::VectorTxBuffer::new(
                        10_000,
                        std::time::Duration::from_secs(60),
                    ),
                    quantizer: RwLock::new(None),
                    sq8_bias: RwLock::new(Sq8Bias::default()),
                    mmap_index: RwLock::new(None),
                    seq_log: RwLock::new(contextra_mvcc::SequenceLog::new()),
                    rebuild_count: AtomicU64::new(0),
                    visited_dead_nodes: AtomicU64::new(0),
                    deleted_nodes: RwLock::new(RoaringTreemap::new()),
                    compute_pool,
                    rng,
                    clock: RwLock::new(clock),
                    metrics_sink: RwLock::new(std::sync::Arc::new(crate::types::NoopMetricsSink)),
                    last_rebuild_nanos: AtomicU64::new(0),
                    consecutive_rapid_rebuilds: AtomicU32::new(0),
                    #[cfg(feature = "partial-index-rebuild")]
                    traversal_tracker: if partial_rebuild_config.traversal_window > 0 {
                        Some(RwLock::new(crate::partial_rebuild::TraversalTracker::new(
                            partial_rebuild_config,
                        )))
                    } else {
                        None
                    },
                    #[cfg(test)]
                    fault_injection_insert_target: AtomicU64::new(0),
                    #[cfg(test)]
                    fault_injection_insert_count: AtomicU64::new(0),
                },
            }),
        }
    }
}
