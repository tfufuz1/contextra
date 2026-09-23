use pyo3::prelude::*;

/// Statistics for a vector index.
#[pyclass(get_all, name = "VectorIndexStats")]
#[derive(Clone)]
pub struct PyVectorIndexStats {
    /// Number of active (non-deleted) vectors.
    pub num_vectors: usize,
    /// Estimated memory usage in bytes.
    pub memory_usage_bytes: usize,
    /// Number of HNSW layers.
    pub num_layers: usize,
}

#[pymethods]
impl PyVectorIndexStats {
    fn __repr__(&self) -> String {
        format!(
            "VectorIndexStats(num_vectors={}, memory_usage_bytes={}, num_layers={})",
            self.num_vectors, self.memory_usage_bytes, self.num_layers
        )
    }
}
