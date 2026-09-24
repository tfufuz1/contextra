use contextra_core::{FusionWeights, Result};

/// Custom weights for vector, text, and graph signals in hybrid search.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SignalWeights {
    /// Weight for vector similarity signal.
    pub vector: f32,
    /// Weight for BM25 text match signal.
    pub text: f32,
    /// Weight for graph traversal signal.
    pub graph: f32,
}

impl SignalWeights {
    /// Creates a new `SignalWeights` instance, ensuring weights sum to 1.0.
    pub fn new(vector: f32, text: f32, graph: f32) -> Result<Self> {
        let fw = FusionWeights::new(vector, text, graph)?;
        Ok(Self {
            vector: fw.vector(),
            text: fw.text(),
            graph: fw.graph(),
        })
    }
}

impl From<SignalWeights> for FusionWeights {
    fn from(w: SignalWeights) -> Self {
        FusionWeights::new(w.vector, w.text, w.graph).unwrap_or_default()
    }
}

impl From<FusionWeights> for SignalWeights {
    fn from(w: FusionWeights) -> Self {
        Self {
            vector: w.vector(),
            text: w.text(),
            graph: w.graph(),
        }
    }
}
