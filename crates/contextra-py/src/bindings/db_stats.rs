use pyo3::prelude::*;

use crate::bindings::storage_stats::PyStorageStats;
use crate::bindings::vector_index_stats::PyVectorIndexStats;

/// Overall database statistics and system observability.
#[pyclass(get_all, name = "DbStats")]
#[derive(Clone)]
pub struct PyDbStats {
    /// Lyapunov drift status ("stabil", "warnung", "kritisch", "unbekannt").
    pub drift_status: String,
    /// Expected Calibration Error (ECE) from IsotonicCalibrator if available.
    pub calibration_ece: Option<f32>,
    /// UNIX timestamp of the last calibration model rebuild.
    pub last_calibration_at: Option<u64>,
    /// Total count of active memory documents in default collection.
    pub active_memory_count: usize,
    /// Current PID-regulated candidate pool size if active.
    pub pid_pool_size: Option<usize>,
    /// Statistics for the vector index.
    pub index_stats: PyVectorIndexStats,
    /// Statistics for the LSM storage engine.
    pub storage_stats: PyStorageStats,
}

#[pymethods]
impl PyDbStats {
    fn __repr__(&self) -> String {
        format!(
            "DbStats(vectors={}, active_memories={}, drift='{}', ece={}, size_bytes={})",
            self.index_stats.num_vectors,
            self.active_memory_count,
            self.drift_status,
            self.calibration_ece
                .map(|e| format!("{:.4}", e))
                .unwrap_or_else(|| "None".to_string()),
            self.storage_stats.total_size_bytes + self.storage_stats.memtable_size_bytes
        )
    }
}
