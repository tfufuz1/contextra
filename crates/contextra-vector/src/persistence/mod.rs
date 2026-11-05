// FILE-CONTEXT
// ZWECK: Persistenz-Schicht für HNSW-Dateiserialisierung (`.hnsw`) und gepuffertes Lesen.
// INVARIANTEN: Zero-Panic bei Deserialisierung.
// HOTSPOTS: persistence/header.rs (HnswHeader::try_from_bytes), persistence/buffer.rs (BufferIndex::open)

//! HNSW Persistence Layer — Serialisierung und gepuffertes Lesen für Vektor-Indizes.

mod buffer;
mod header;
mod node;

#[cfg(test)]
mod tests;

pub use buffer::BufferIndex;
pub use header::{HnswHeader, HNSW_MAGIC, HNSW_VERSION};
pub use node::NodeRecord;

pub mod snapshot;
pub use snapshot::{
    load_snapshot_vfs, save_snapshot_vfs, VectorSnapshotHeader, SNAPSHOT_FORMAT_VERSION,
    SNAPSHOT_KIND_VECTOR, SNAPSHOT_MAGIC,
};
