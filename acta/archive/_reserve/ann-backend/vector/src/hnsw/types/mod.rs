// FILE-CONTEXT
// ZWECK: Verzeichnismodul fuer HNSW-Zustandstypen, Re-Exports aller Typen am bisherigen Pfad.
// INVARIANTEN: Keine Pfadaenderungen gegenueber src/hnsw/types.rs. Alle Typen sind 1:1 re-exportiert.
// HOTSPOTS: HnswIndex, HnswNode, VectorData, HnswIndexCore
// STAND: TS:2026-11-12T00:00:00Z

mod builder;
mod core;
mod data;
mod guards;
mod index;
mod persist;
mod seqlog;

// 1:1 Re-Exports aller bisherigen Typen und Konstanten
pub use self::core::{HnswColdCore, HnswHotCore, HnswIndexCore, RebuildStatus};
pub use self::data::{
    Candidate, HnswNode, VectorData, DEFAULT_LAYER_SEED, HNSW_REBUILD_DELETION_RATIO,
    SENTINEL_NO_ENTRY_POINT,
};
pub use self::guards::{RebuildGuard, SnapshotPinGuard};
pub use self::index::{GenericHnswIndex, HnswIndex};
