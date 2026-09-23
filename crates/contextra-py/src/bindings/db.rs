use contextra_db::Contextra;
use numpy::PyReadonlyArray1;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::PyBytes;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::runtime::Runtime;

use crate::bindings::collection::PyCollection;
use crate::bindings::common::*;
use crate::bindings::db_stats::PyDbStats;
use crate::bindings::document::PyDocument;
use crate::bindings::search_result::PySearchResult;
use crate::bindings::storage_stats::PyStorageStats;
use crate::bindings::vector_index_stats::PyVectorIndexStats;

// ─── PyContextra (Database Facade) ────────────────────────────────────────────

#[pyclass(name = "Db")]
pub struct PyContextra {
    pub(crate) inner: Arc<Contextra>,
    pub(crate) runtime: Arc<Runtime>,
    pub(crate) worker_threads: usize,
    pub(crate) poisoned: Arc<AtomicBool>,
}

impl PyContextra {
    pub fn new(
        inner: Arc<Contextra>,
        runtime: Arc<Runtime>,
        worker_threads: usize,
        poisoned: Arc<AtomicBool>,
    ) -> Self {
        Self {
            inner,
            runtime,
            worker_threads,
            poisoned,
        }
    }
}

#[pymethods]
impl PyContextra {
    /// Returns the number of worker threads configured in this database's Tokio runtime.
    #[getter]
    pub fn worker_threads(&self) -> usize {
        self.worker_threads
    }

    /// Returns true if the database engine was poisoned by a previous caught Rust panic.
    #[getter]
    pub fn is_poisoned(&self) -> bool {
        self.poisoned.load(Ordering::SeqCst)
    }

    /// Internal helper method for testing FFI panic isolation and engine poisoning.
    #[pyo3(signature = (message=None))]
    #[allow(clippy::panic)]
    pub fn _trigger_panic_for_test(&self, py: Python<'_>, message: Option<String>) -> PyResult<()> {
        let msg = message.unwrap_or_else(|| "Test panic for FFI isolation".to_string());
        run_blocking_ffi(py, &self.poisoned, move || -> PyResult<()> {
            panic!("{}", msg);
        })
    }

    // ── Context Manager Protocol ──

    /// Enters the context manager. Returns `self`.
    fn __enter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    /// Exits the context manager, flushing all pending writes.
    #[pyo3(signature = (_exc_type=None, _exc_val=None, _exc_tb=None))]
    fn __exit__(
        &self,
        py: Python<'_>,
        _exc_type: Option<&Bound<'_, PyAny>>,
        _exc_val: Option<&Bound<'_, PyAny>>,
        _exc_tb: Option<&Bound<'_, PyAny>>,
    ) -> PyResult<bool> {
        let rt = &self.runtime;
        run_blocking_ffi(py, &self.poisoned, || {
            rt.block_on(self.inner.flush()).map_err(contextra_err)
        })?;
        Ok(false) // Do not suppress exceptions
    }

    // ── Collection Management ──

    /// Returns a specific collection (namespace).
    /// Creates the collection if it does not already exist.
    pub fn collection(&self, name: &str, py: Python<'_>) -> PyResult<PyCollection> {
        validate_collection_name(name)?;
        let rt = &self.runtime;
        let name_owned = name.to_string();
        let col = run_blocking_ffi(py, &self.poisoned, || {
            rt.block_on(self.inner.collection(&name_owned))
                .map_err(contextra_err)
        })?;
        Ok(PyCollection::new(
            col,
            self.runtime.clone(),
            self.poisoned.clone(),
        ))
    }

    /// Lists all existing collection names.
    pub fn list_collections(&self, py: Python<'_>) -> PyResult<Vec<String>> {
        let rt = &self.runtime;
        run_blocking_ffi(py, &self.poisoned, || {
            rt.block_on(self.inner.list_collections())
                .map_err(contextra_err)
        })
    }

    /// Drops a collection, removing all its data from storage.
    pub fn drop_collection(&self, name: &str, py: Python<'_>) -> PyResult<()> {
        validate_collection_name(name)?;
        let rt = &self.runtime;
        let name_owned = name.to_string();
        let tenant_id = contextra_core::TenantId::try_new(1).map_err(contextra_err)?;
        run_blocking_ffi(py, &self.poisoned, || {
            rt.block_on(
                self.inner
                    .drop_collection(&name_owned, tenant_id, &[0u8; 32]),
            )
            .map(|_| ())
            .map_err(contextra_err)
        })
    }

    /// Flushes all pending writes to disk.
    pub fn flush(&self, py: Python<'_>) -> PyResult<()> {
        let rt = &self.runtime;
        run_blocking_ffi(py, &self.poisoned, || {
            rt.block_on(self.inner.flush()).map_err(contextra_err)
        })
    }

    /// Returns combined statistics for the vector index and storage engine.
    pub fn stats(&self, py: Python<'_>) -> PyResult<PyDbStats> {
        let rt = &self.runtime;
        let stats = run_blocking_ffi(py, &self.poisoned, || {
            rt.block_on(self.inner.stats()).map_err(contextra_err)
        })?;

        Ok(PyDbStats {
            drift_status: stats.drift_status,
            calibration_ece: stats.calibration_ece,
            last_calibration_at: stats.last_calibration_at,
            active_memory_count: stats.active_memory_count,
            pid_pool_size: stats.pid_pool_size,
            index_stats: PyVectorIndexStats {
                num_vectors: stats.index_stats.num_vectors,
                memory_usage_bytes: stats.index_stats.memory_usage_bytes,
                num_layers: stats.index_stats.num_layers,
            },
            storage_stats: PyStorageStats {
                num_segments: stats.storage_stats.num_segments,
                total_size_bytes: stats.storage_stats.total_size_bytes,
                memtable_size_bytes: stats.storage_stats.memtable_size_bytes,
            },
        })
    }

    /// Returns the number of documents.
    pub fn len(&self, py: Python<'_>) -> PyResult<usize> {
        let rt = &self.runtime;
        run_blocking_ffi(py, &self.poisoned, || {
            rt.block_on(self.inner.len()).map_err(contextra_err)
        })
    }

    /// Returns true if the collection/database is empty.
    pub fn is_empty(&self, py: Python<'_>) -> PyResult<bool> {
        let rt = &self.runtime;
        run_blocking_ffi(py, &self.poisoned, || {
            rt.block_on(self.inner.is_empty()).map_err(contextra_err)
        })
    }
}

// ── Generated CRUD + Batch Methods ──
contextra_crud_methods!(PyContextra);
contextra_batch_methods!(PyContextra);
