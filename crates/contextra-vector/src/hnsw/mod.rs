// FILE-CONTEXT
// ZWECK: HNSW Vector Index mit Layer Descent, Soft-Deletes und transaktionalem Staging (TxBuffer).
// INVARIANTEN: Lock-Hierarchie: write_mutex (exklusive Mutation/Rebuild) -> entry_point -> nodes / doc_to_node / deleted_nodes (HotState/ColdState).
// NICHT-OFFENSICHTLICH: Multi-threaded Reads sperren nie write_mutex; background rebuild tauscht Core atomar via Swap.
// HOTSPOTS: mod.rs (HnswIndex::insert, search, delete, rebuild, save)
// STAND: 2026-11-12

//! HNSW (Hierarchical Navigable Small World) vector index module.
//!
//! ### Lock-Hierarchie (§3.5, INV-LOCK-ORDER-1)
//! Um Deadlocks mathematisch auszuschließen, gilt eine strikte, globale Akquisitionsreihenfolge:
//! 1. `write_mutex`: Exklusiver Schreib- & Rebuild-Mutex (HotState). Schützt koordinierte Graph-Mutationen.
//! 2. `entry_point` / `ram_entry_point`: Atomare Entry-Point Pointer (`AtomicU32` mit Acquire/Release Ordering).
//! 3. `nodes` / `doc_to_node` / `deleted_nodes`: Read/Write Locks auf Knotenlisten, Dokument-Mappings und Soft-Deletes.
//! 4. `arena`: Interne Arena-Puffer-Locks (offsets, capacities, count_offsets, counts, arena).
//!
//! Invariante: Kein Lock-Guard darf über I/O oder `.await`-Grenzen gehalten werden.

// SECTION Öffentliche HNSW-Submodule

// ANCHOR[contextra-vector:HNSW-MOD-ADAPTIVE-EF]
// Dynamische Anpassung des ef-Search-Parameters basierend auf Selektivität und Latenz.
pub mod adaptive_ef;
// ANCHOR[contextra-vector:HNSW-MOD-ARENA]
// Lock-freie und cache-effiziente Kanten-Arena für Graph-Nachbarschaften.
pub mod arena;
// ANCHOR[contextra-vector:HNSW-MOD-CLOCK]
// Monotoner Zähler-Takt für deterministische Tests und Benchmarks.
pub mod clock;
// ANCHOR[contextra-vector:HNSW-MOD-DELETION]
// Tombstone-Verwaltung, Slot-Nullung und GhostFreeVectorIndex.
pub mod deletion;
// ANCHOR[contextra-vector:HNSW-MOD-SQ8-BIAS]
// Asymmetrische Distanzkorrektur für quantisierte Vektoren.
pub mod sq8_bias;
// !SECTION

// SECTION Kompatibilitäts-Exporte (Transitional Bridge)

#[allow(deprecated)]
#[cfg(feature = "ports-compat")]
pub use crate::compat::purge::{hnsw_purger, HnswPurger};
#[allow(deprecated)]
#[cfg(feature = "ports-compat")]
pub use crate::compat::scanner::{hnsw_scanner, HnswScanner};
// !SECTION

// SECTION Private & Interne Module

mod acorn_filtered;
pub(crate) mod batch;
pub(crate) mod config;
mod core_insert;
mod core_rebuild;
mod core_search;
pub mod staging;
pub(crate) mod types;

// !SECTION

// SECTION Re-Exports der HNSW-Komponenten

#[cfg(feature = "ports-compat")]
pub use crate::compat::hnsw_vector_index::{HnswCompactStats, Index, Maintain};
pub use adaptive_ef::{AdaptiveEfPolicy, AdaptiveEfStateMachine, AdaptiveEfStats};
pub use arena::{BacklinkTable, HnswArena};
pub use batch::{BatchContext, NeighborBacklink, PreparedInsert};
pub use clock::MonotonicCounterClock;
pub use config::{HnswConfig, HnswConfigBuilder};
pub use deletion::{DeletionStats, GhostFreeVectorIndex};
pub use sq8_bias::Sq8Bias;
pub use types::{
    Candidate, HnswColdCore, HnswHotCore, HnswIndex, HnswIndexCore, HnswNode, RebuildGuard,
    RebuildStatus, SnapshotPinGuard, VectorData, HNSW_REBUILD_DELETION_RATIO,
    SENTINEL_NO_ENTRY_POINT,
};

pub use staging::{VectorStagingOp, VectorTxBuffer};
// !SECTION
