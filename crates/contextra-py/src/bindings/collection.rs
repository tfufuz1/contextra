use contextra_db::Contextra;
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
    pub(crate) db: Arc<Contextra>,
    pub(crate) name: String,
    pub(crate) inner: Arc<
        contextra_db::collection::Collection<
            contextra_store::tenant_codec::TenantScopedStorage<Arc<contextra_store::LsmStorage>>,
        >,
    >,
    pub(crate) runtime: Arc<Runtime>,
    pub(crate) poisoned: Arc<AtomicBool>,
    pub(crate) tenant_id: contextra_types::TenantId,
}

impl PyCollection {
    pub fn new(
        db: Arc<Contextra>,
        name: String,
        inner: Arc<
            contextra_db::collection::Collection<
                contextra_store::tenant_codec::TenantScopedStorage<
                    Arc<contextra_store::LsmStorage>,
                >,
            >,
        >,
        runtime: Arc<Runtime>,
        poisoned: Arc<AtomicBool>,
        tenant_id: contextra_types::TenantId,
    ) -> Self {
        Self {
            db,
            name,
            inner,
            runtime,
            poisoned,
            tenant_id,
        }
    }

    pub(crate) async fn get_target_collection(
        &self,
    ) -> Result<
        Arc<
            contextra_db::collection::Collection<
                contextra_store::tenant_codec::TenantScopedStorage<
                    Arc<contextra_store::LsmStorage>,
                >,
            >,
        >,
        contextra_types::ContextraError,
    > {
        Ok(self.inner.clone())
    }
}

#[pymethods]
impl PyCollection {
    /// Returns the tenant ID associated with this collection context.
    #[getter]
    pub fn tenant_id(&self) -> u64 {
        self.tenant_id.inner()
    }

    /// Returns a new Collection instance bound to the specified tenant ID.
    pub fn with_tenant(&self, tenant_id: u64, py: Python<'_>) -> PyResult<PyCollection> {
        let new_tenant = contextra_types::TenantId::try_new(tenant_id)
            .map_err(|e| PyValueError::new_err(e.to_string()))?;
        let rt = &self.runtime;
        let name_owned = self.name.clone();
        let db = self.db.clone();
        let col = run_blocking_ffi(py, &self.poisoned, || {
            rt.block_on(db.collection_for_tenant(&name_owned, new_tenant))
                .map_err(contextra_err)
        })?;
        Ok(PyCollection::new(
            self.db.clone(),
            self.name.clone(),
            col,
            self.runtime.clone(),
            self.poisoned.clone(),
            new_tenant,
        ))
    }

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
