//! # Python-Schnittstelle für Sammlungen (`PyCollection`)
//!
//! Dieses Modul definiert `PyCollection`, die Repräsentation einer Dokumentensammlung
//! (Collection) im Python-Namespace.
//!
//! ## Ring-Zugehörigkeit
//! Ring 4 (FFI-Boundary / Bindings).
//!
//! ## Architektur
//! Eine `PyCollection` verweist auf eine zugrundeliegende `contextra_db::Collection`.
//! Sie hält:
//! - Einen referenzgezählten Handle (`Arc<ContextraCollection>`) auf die Rust-Engine.
//! - Den Tokio-Runtime-Handle (`Arc<Runtime>`) zur Ausführung asynchroner Abfragen.
//! - Ein atomares `poisoned`-Flag (`Arc<AtomicBool>`), das bei einem Rust-Panic auf `true`
//!   gesetzt wird, um Folgeaufrufe fail-closed zu blockieren.
//!
//! Die CRUD- und Batch-Methoden werden über die Makros `contextra_crud_methods!` und
//! `contextra_batch_methods!` generiert.

use contextra_db::Collection as ContextraCollection;
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

// ANCHOR[contextra-py:COL-001]
/// Kapselt eine benannte Contextra-Dokumentensammlung für Python.
///
/// # Felder
/// - `inner`: Thread-sicherer Zeiger auf die interne `ContextraCollection`.
/// - `runtime`: Asynchrone Tokio-Laufzeitumgebung.
/// - `poisoned`: Globales Vergiftungsflag bei Panics.
#[pyclass(name = "Collection")]
pub struct PyCollection {
    pub(crate) inner: Arc<ContextraCollection>,
    pub(crate) runtime: Arc<Runtime>,
    pub(crate) poisoned: Arc<AtomicBool>,
}

impl PyCollection {
    /// Erzeugt eine neue `PyCollection`-Instanz.
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
    // ANCHOR[contextra-py:COL-002]
    /// Gibt `True` zurück, wenn die Collection aufgrund eines vorherigen Rust-Panics vergiftet wurde.
    ///
    /// # Invariante (Fail-Closed)
    /// Sobald `is_poisoned` wahr ist, schlagen alle nachfolgenden Datenzugriffe sofort fehl.
    #[getter]
    pub fn is_poisoned(&self) -> bool {
        self.poisoned.load(Ordering::SeqCst)
    }

    // ANCHOR[contextra-py:COL-003]
    /// Interner Testhook zur Verifikation von FFI-Panic-Isolation und Engine-Poisoning.
    ///
    /// # Panics
    /// Bewusstes Auslösen eines Panics zum Testen der Barriere. Wird durch `run_blocking_ffi`
    /// abgefangen und in ein `PyErr` gewandelt.
    #[pyo3(signature = (message=None))]
    pub fn _trigger_panic_for_test(&self, py: Python<'_>, message: Option<String>) -> PyResult<()> {
        crate::bindings::functions::trigger_panic_helper(py, &self.poisoned, message)
    }

    // ANCHOR[contextra-py:COL-004]
    /// Liefert detaillierte Statistiken über den HNSW-Vektorindex dieser Collection.
    ///
    /// # Ablauf
    /// 1. Asynchron `inner.stats()` unter GIL-Release ausführen.
    /// 2. Rust-Statistiken in `PyVectorIndexStats` überführen.
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

    // ANCHOR[contextra-py:COL-005]
    /// Gibt die Gesamtzahl der Dokumente in dieser Collection zurück.
    pub fn len(&self, py: Python<'_>) -> PyResult<usize> {
        let rt = &self.runtime;
        run_blocking_ffi(py, &self.poisoned, || Ok(rt.block_on(self.inner.len())))
    }

    // ANCHOR[contextra-py:COL-006]
    /// Prüft, ob die Collection leer ist (`True` falls keine Dokumente enthalten sind).
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
