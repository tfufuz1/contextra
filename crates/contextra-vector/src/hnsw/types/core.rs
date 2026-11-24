// FILE-CONTEXT
// ZWECK: Aufteilung des HNSW-Graphzustands in Hot- und Cold-Datenstrukturen (Cache-Line-Alignment).
// INVARIANTEN: Lock-Hierarchie: write_mutex -> entry_point -> nodes / doc_to_node / deleted_nodes -> arena.
// HOTSPOTS: HnswHotCore, HnswColdCore, HnswIndexCore
// STAND: TS:2026-11-12T00:00:00Z

use roaring::RoaringTreemap;
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};

use parking_lot::{Mutex, RwLock};

use super::data::{HnswNode, SENTINEL_NO_ENTRY_POINT};
use crate::hnsw::arena::HnswArena;
use crate::hnsw::config::HnswConfig;
use crate::hnsw::sq8_bias::Sq8Bias;

// ANCHOR[contextra-vector:HNSW-HOT-CORE]
/// Heisser Kern des HNSW-Index (fuer Lese- und Schreibpfade im Graph optimiert).
#[repr(align(64))]
pub struct HnswHotCore {
    pub nodes: RwLock<Vec<HnswNode>>,
    #[cfg(not(feature = "docid-128"))]
    pub doc_to_node: RwLock<BTreeMap<u64, usize>>,
    #[cfg(feature = "docid-128")]
    pub doc_to_node: RwLock<BTreeMap<u128, usize>>,
    pub entry_point: AtomicU32,
    pub ram_entry_point: AtomicU32,
    pub max_layer: AtomicU64,
    pub ml: f64,
    pub layer_seed: AtomicU64,
    pub deleted_count: AtomicU64,
    pub write_mutex: Mutex<()>,
    pub last_tx_id: AtomicU64,
    pub rebuilding: AtomicBool,
    pub rebuild_condvar: parking_lot::Condvar,
    pub rebuild_mutex: Mutex<()>,
    pub arena: HnswArena,
}

impl HnswHotCore {
    #[inline]
    pub fn get_entry_point(&self) -> Option<usize> {
        // NOTE: Acquire ensures visibility of nodes and layers created prior to publishing entry point.
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
        // NOTE: Release ensures node connections and memory are fully visible before entry point is published.
        self.entry_point.store(val, Ordering::Release);
    }

    #[inline]
    pub fn get_ram_entry_point(&self) -> Option<usize> {
        // NOTE: Acquire ensures visibility of RAM nodes linked before entry point publication.
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
        // NOTE: Release synchronizes RAM node initialization with concurrent readers.
        self.ram_entry_point.store(val, Ordering::Release);
    }

    pub fn get_ram_node_connections(&self, ram_idx: usize, layer: usize, m: usize) -> Vec<u32> {
        self.arena.get_ram_node_connections(ram_idx, layer, m)
    }
}

// ANCHOR[contextra-vector:HNSW-COLD-CORE]
/// Kalter Kern des HNSW-Index (Konfiguration, Puffer, Metriken, Quantisierer).
pub struct HnswColdCore {
    pub config: HnswConfig,
    pub validation_error: Option<String>,
    pub tx_buffer: crate::hnsw::staging::VectorTxBuffer,
    pub quantizer: RwLock<Option<crate::quantize::ScalarQuantizer>>,
    pub sq8_bias: RwLock<Sq8Bias>,
    pub mmap_index: RwLock<Option<crate::persistence::MmapIndex>>,
    pub seq_log: RwLock<contextra_mvcc::SequenceLog>,
    pub rebuild_count: AtomicU64,
    pub visited_dead_nodes: AtomicU64,
    pub deleted_nodes: RwLock<RoaringTreemap>,
    pub compute_pool: crate::compute_pool::ComputePool,
    pub rng: std::sync::Arc<dyn crate::types::Rng>,
    pub clock: RwLock<std::sync::Arc<dyn crate::types::Clock>>,
    pub metrics_sink: RwLock<std::sync::Arc<dyn crate::types::MetricsSink>>,
    pub last_rebuild_nanos: AtomicU64,
    pub consecutive_rapid_rebuilds: AtomicU32,
    #[cfg(feature = "partial-index-rebuild")]
    pub traversal_tracker: Option<RwLock<crate::partial_rebuild::TraversalTracker>>,
    #[cfg(test)]
    pub fault_injection_insert_target: AtomicU64,
    #[cfg(test)]
    pub fault_injection_insert_count: AtomicU64,
}

// ANCHOR[contextra-vector:HNSW-INDEX-CORE]
/// Die Kern-Implementierung des HNSW-Index aus Hot- und Cold-Strukturen.
pub struct HnswIndexCore {
    pub hot: HnswHotCore,
    pub cold: HnswColdCore,
}

impl HnswIndexCore {
    /// Sets the random seed used for HNSW layer selection during insertions.
    pub fn set_layer_seed(&self, seed: u64) {
        // NOTE: Relaxed ordering is sufficient for independent pseudo-random seed updates.
        self.hot.layer_seed.store(seed, Ordering::Relaxed);
    }

    /// Returns the metrics sink.
    pub fn metrics_sink(&self) -> std::sync::Arc<dyn crate::types::MetricsSink> {
        self.cold.metrics_sink.read().clone()
    }

    /// Sets the metrics sink.
    pub fn set_metrics_sink(&self, sink: std::sync::Arc<dyn crate::types::MetricsSink>) {
        *self.cold.metrics_sink.write() = sink;
    }

    /// Returns the clock instance used by this index.
    pub fn clock(&self) -> std::sync::Arc<dyn crate::types::Clock> {
        self.cold.clock.read().clone()
    }

    /// Sets the clock instance.
    pub fn set_clock(&self, clock: std::sync::Arc<dyn crate::types::Clock>) {
        *self.cold.clock.write() = clock;
    }
}

/// Status eines Index-Rebuilds.
#[derive(Debug, Clone, PartialEq)]
pub enum RebuildStatus {
    /// Kein Rebuild laeuft oder geplant.
    Idle,
    /// Rebuild laeuft gerade im Hintergrund.
    Running,
    /// Rebuild wurde getriggert, startet bald.
    Pending,
}
