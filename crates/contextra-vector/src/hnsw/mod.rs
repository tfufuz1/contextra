// FILE-CONTEXT
// ZWECK: HNSW Vector Index mit Layer Descent, Soft-Deletes und transaktionalem Staging (TxBuffer).
// INVARIANTEN: Lock-Hierarchie: write_mutex (exklusive Mutation/Rebuild) -> entry_point -> nodes / doc_to_node / deleted_nodes (HotState/ColdState).
// NICHT-OFFENSICHTLICH: Multi-threaded Reads sperren nie write_mutex; background rebuild tauscht Core atomar via Swap.
// HOTSPOTS: mod.rs (HnswIndex::insert, search, delete, rebuild, save)

//! HNSW (Hierarchical Navigable Small World) vector index module.

pub mod arena;
pub mod sq8_bias;

mod batch;
mod config;
mod core_insert;
mod core_rebuild;
mod core_search;
mod types;
mod vector_index_impl;

#[cfg(test)]
mod tests;

pub use arena::{BacklinkTable, HnswArena};
pub use batch::{BatchContext, NeighborBacklink, PreparedInsert};
pub use config::{HnswConfig, HnswConfigBuilder};
pub use sq8_bias::Sq8Bias;
pub use types::{
    Candidate, HnswColdCore, HnswHotCore, HnswIndex, HnswIndexCore, HnswNode, RebuildGuard,
    RebuildStatus, SnapshotPinGuard, VectorData, HNSW_REBUILD_DELETION_RATIO,
    SENTINEL_NO_ENTRY_POINT,
};
