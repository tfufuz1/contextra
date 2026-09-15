//! Core trait definitions for MemFuse subsystems.
//!
//! These traits define the abstract interfaces that concrete implementations
//! must fulfill, enabling modularity and testability.

// FILE-CONTEXT
// STAND: 2026-09-15T00:00:00Z
// ZWECK: Kern-Trait-Hierarchien (StorageEngine, VectorIndex, TextIndex, GraphIndex, Checkpoint, etc.) für Layer 0.
// INVARIANTEN: Downward-only Trait interfaces; neue Trait-Methoden brauchen Default-Impls (Abwärtskompatibilität).
// HOTSPOTS: mod declaration & re-exports
// SIEHE AUCH: rules/tag_taxonomy.md, DECISIONS.md (ADR-024)

// INVARIANT: Trait-Contracts sind das API-Rückgrat des Workspace.
// REGEL: Neue Methoden MÜSSEN Default-Impl haben (backward compat).

use std::future::Future;
use std::pin::Pin;

/// Type alias for a pinned, heap-allocated `Future` that is `Send` and dyn-compatible.
pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// Type alias for a pinned, heap-allocated `Stream` that is `Send` and dyn-compatible.
pub type BoxStream<'a, T> = Pin<Box<dyn futures_util::stream::Stream<Item = T> + Send + 'a>>;

/// Checkpoint and snapshot traits.
pub mod checkpoint;
/// Embedding provider and LLM generation traits.
pub mod embedding;
/// Graph index traits and CSR statistics.
pub mod graph_index;
/// Memory lifecycle, grounding validator, and distance calculator contracts.
pub mod lifecycle;
/// Observability and lifecycle re-exports.
pub mod observability;
/// Key-value storage engine traits.
pub mod storage;
/// Text retrieval and indexing traits (BM25 / Inverted Index).
pub mod text_index;
/// Vector retrieval and HNSW indexing traits.
pub mod vector_index;

pub use checkpoint::*;
pub use embedding::*;
pub use graph_index::*;
pub use lifecycle::*;
pub use storage::*;
pub use text_index::*;
pub use vector_index::*;

#[cfg(test)]
mod dyn_safety {
    use super::*;

    fn _assert_dyn_storage(_: Option<&dyn StorageEngine>) {}
    fn _assert_dyn_graph(_: Option<&dyn GraphIndex>) {}
    fn _assert_dyn_embedding(_: Option<&dyn TextEmbeddingEngine>) {}
    fn _assert_dyn_response_grounding_validator(_: Option<&dyn ResponseGroundingValidator>) {}

    #[test]
    fn test_dyn_safety_compiles() {
        _assert_dyn_storage(None);
        _assert_dyn_graph(None);
        _assert_dyn_embedding(None);
    }
}
