// FILE-CONTEXT
// ZWECK: Persistenz-Schicht für HNSW-Dateiserialisierung (`.hnsw`) und mmap-basiertes Lesen.
// INVARIANTEN: Zero-Panic bei Deserialisierung; mmap-Reads überleben POSIX file replace.
// NICHT-OFFENSICHTLICH: MmapIndex hält read-only FD; Schreibvorgänge laufen atomar über .tmp und Rename.
// HOTSPOTS: persistence/header.rs (HnswHeader::try_from_bytes), persistence/mmap.rs (MmapIndex::open)
// STAND: TS:2026-08-30T18:53:53Z (SESSION: 37b1d991)

//! HNSW Persistence Layer — Serialisierung und mmap-Mapping für Vektor-Indizes.

mod header;
mod mmap;
mod node;

#[cfg(test)]
mod tests;

pub use header::{HnswHeader, HNSW_MAGIC, HNSW_VERSION};
pub use mmap::MmapIndex;
pub use node::NodeRecord;
