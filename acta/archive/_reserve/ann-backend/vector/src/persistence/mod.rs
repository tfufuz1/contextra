// Persistenzschicht fuer HNSW (Header, MmapIndex, NodeRecord, write_atomic).
// Invarianten: Persistenzdateien muessen atomar geschrieben und ueber BLAKE3 validiert werden.
// HOTSPOTS: persistence/header.rs, persistence/mmap.rs, persistence/atomic.rs

pub mod atomic;
mod header;
pub(crate) mod io;
mod mmap;
mod node;

#[cfg(test)]
pub mod tests;

pub use atomic::write_atomic;
pub use header::{HnswHeader, HNSW_MAGIC, HNSW_VERSION};
pub use mmap::MmapIndex;
pub use node::NodeRecord;
