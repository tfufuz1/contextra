use contextra_engine::Collection as ContextraCollection;
use numpy::PyReadonlyArray1;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::PyBytes;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::runtime::Runtime;

use crate::bindings::common::*;
use crate::bindings::document::PyDocument;
use crate::bindings::search_result::PySearchResult;
use crate::bindings::vector_index_stats::PyVectorIndexStats;

// ─── PyCollection ───────────────────────────────────────────────────────────

#[pyclass(name = "Collection")]
pub struct PyCollection {
    pub(crate) inner: Arc<ContextraCollection>,
    pub(crate) runtime: Arc<Runtime>,
    pub(crate) poisoned: Arc<AtomicBool>,
}

impl PyCollection {
    pub fn new(
        inner: Arc<ContextraCollection>,
        runtime: Arc<Runtime>,
        poisoned: Arc<AtomicBool>,
    ) -> Self {
        Self {
            inner,
            runtime,
            poisoned,
        }
    }
}

#[pymethods]
impl PyCollection {
    /// Returns true if the collection/database engine was poisoned by a previous caught Rust panic.
    #[getter]
    pub fn is_poisoned(&self) -> bool {
        self.poisoned.load(Ordering::SeqCst)
    }

    /// Internal helper method for testing FFI panic isolation and engine poisoning.
    ///
    /// # Panics
    ///
    /// bewusst; nur Testhook für FFI-Panic-Isolation; Panic wird durch run_blocking_ffi in PyErr übersetzt
    #[pyo3(signature = (message=None))]
    pub fn _trigger_panic_for_test(&self, py: Python<'_>, message: Option<String>) -> PyResult<()> {
        crate::bindings::functions::trigger_panic_helper(py, &self.poisoned, message)
    }

    /// Returns statistics for the collection's vector index.
    pub fn stats(&self, py: Python<'_>) -> PyResult<PyVectorIndexStats> {
        let rt = &self.runtime;
        let stats = run_blocking_ffi(py, &self.poisoned, || {
            rt.block_on(self.inner.stats()).map_err(contextra_err)
        })?;

        Ok(PyVectorIndexStats {
            num_vectors: stats.num_vectors,
            memory_usage_bytes: stats.memory_usage_bytes,
            num_layers: stats.num_layers,
        })
    }

    /// Returns the number of documents.
    pub fn len(&self, py: Python<'_>) -> PyResult<usize> {
        let rt = &self.runtime;
        run_blocking_ffi(py, &self.poisoned, || Ok(rt.block_on(self.inner.len())))
    }

    /// Returns true if the collection is empty.
    pub fn is_empty(&self, py: Python<'_>) -> PyResult<bool> {
        let rt = &self.runtime;
        run_blocking_ffi(
            py,
            &self.poisoned,
            || Ok(rt.block_on(self.inner.is_empty())),
        )
    }
}

// ── Generated CRUD + Batch Methods ──
contextra_crud_methods!(PyCollection);
contextra_batch_methods!(PyCollection);
