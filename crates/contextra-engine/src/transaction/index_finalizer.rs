use contextra_ports::{StorageEngine, VectorIndex};
use contextra_types::{ContextraError, Result};

/// Trait defining the finalize contract for engine indices (Text, Vector, Graph).
/// Index finalize operations are idempotent and executable during commit or recovery.
pub trait IndexFinalizer {
    /// Finalizes index changes up to transaction sequence `seq`.
    fn finalize(&self, seq: u64) -> Result<()>;

    /// Returns the sequence number of the last finalized transaction for this index.
    /// Returns `Err(ContextraError::CapabilityUnsupported)` if the underlying index
    /// cannot track or expose `last_finalized_seq`.
    fn last_finalized_seq(&self) -> Result<u64>;
}

/// Adapter for InvertedIndex implementing `IndexFinalizer`.
pub struct TextIndexFinalizerAdapter<'a, S: StorageEngine> {
    _index: &'a contextra_text::inverted::InvertedIndex<S>,
}

impl<'a, S: StorageEngine> TextIndexFinalizerAdapter<'a, S> {
    pub fn new(index: &'a contextra_text::inverted::InvertedIndex<S>) -> Self {
        Self { _index: index }
    }
}

impl<'a, S: StorageEngine> IndexFinalizer for TextIndexFinalizerAdapter<'a, S> {
    fn finalize(&self, _seq: u64) -> Result<()> {
        // Text index commit is atomic; finalize is an idempotent no-op confirmation
        Ok(())
    }

    fn last_finalized_seq(&self) -> Result<u64> {
        Err(ContextraError::CapabilityUnsupported {
            capability: "last_finalized_seq".into(),
            reason: "Text index does not expose last_finalized_seq".into(),
        })
    }
}

/// Adapter for VectorIndex implementing `IndexFinalizer`.
pub struct VectorIndexFinalizerAdapter<'a, V: VectorIndex> {
    _index: &'a V,
}

impl<'a, V: VectorIndex> VectorIndexFinalizerAdapter<'a, V> {
    pub fn new(index: &'a V) -> Self {
        Self { _index: index }
    }
}

impl<'a, V: VectorIndex> IndexFinalizer for VectorIndexFinalizerAdapter<'a, V> {
    fn finalize(&self, _seq: u64) -> Result<()> {
        Ok(())
    }

    fn last_finalized_seq(&self) -> Result<u64> {
        Err(ContextraError::CapabilityUnsupported {
            capability: "last_finalized_seq".into(),
            reason: "Vector index does not expose last_finalized_seq".into(),
        })
    }
}

/// Adapter for CsrGraph implementing `IndexFinalizer`.
pub struct GraphIndexFinalizerAdapter<'a> {
    _graph: &'a contextra_graph::CsrGraph,
}

impl<'a> GraphIndexFinalizerAdapter<'a> {
    pub fn new(graph: &'a contextra_graph::CsrGraph) -> Self {
        Self { _graph: graph }
    }
}

impl<'a> IndexFinalizer for GraphIndexFinalizerAdapter<'a> {
    fn finalize(&self, _seq: u64) -> Result<()> {
        Ok(())
    }

    fn last_finalized_seq(&self) -> Result<u64> {
        Err(ContextraError::CapabilityUnsupported {
            capability: "last_finalized_seq".into(),
            reason: "CSR Graph index does not expose last_finalized_seq".into(),
        })
    }
}
