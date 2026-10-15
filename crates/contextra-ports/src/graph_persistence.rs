//! Graph persistence port traits (§4.3).

#![forbid(unsafe_code)]

use contextra_types::Result;

/// Abstract persistence contract for graph CSR structures and adjacency lists (§4.3).
///
/// Decouples Graph-Modul from `contextra-store`.
pub trait GraphPersistence: Send + Sync {
    /// Persists raw graph segment or snapshot data identified by `key`.
    fn persist_graph_segment(&self, key: &[u8], bytes: &[u8]) -> Result<()>;

    /// Loads graph segment or snapshot data identified by `key`.
    fn load_graph_segment(&self, key: &[u8]) -> Result<Option<Vec<u8>>>;

    /// Deletes graph segment data identified by `key`.
    fn delete_graph_segment(&self, key: &[u8]) -> Result<()>;
}
