// FILE-CONTEXT
// REVIEW-PASS[1/2] Systematischer Review der contextra-py PyO3 FFI Bindings (PRÜFER-KONTEXT: FRESH) (TS: 2026-09-16T16:47:30Z) (SESSION: 29edb6a4)
// STAND: 2026-09-10T19:22:55Z (SESSION: 0b2ff57d)
// ZWECK: PyO3 FFI bindings bridging Contextra embedded vector DB functionality to Python.
// INVARIANTEN: Zero Rust panics cross FFI boundary; GIL released during block_on async calls; Tokio Runtime bound per interpreter module state.
// NICHT-OFFENSICHTLICH: Per-interpreter Tokio runtime instance attached to Python module state (`PyRuntimeState`) to enforce sub-interpreter isolation (PEP 684).
// HOTSPOTS: [160-205] contextra_err mapping, [270-650] CRUD & search methods FFI boundary validation.
// SIEHE AUCH: crates/contextra-db/AGENTS.md

//! # Contextra Python Bindings
//!
//! This crate provides the Python bridge for the Contextra embedded hybrid-search database.
//! It utilizes PyO3 for the bridge and NumPy for efficient vector operations.
//!
//! ## Architecture Role
//!
//! - **Python Bridge (Layer 3)**: Exposes the core functionality of Contextra to Python.
//! - **Async Orchestration**: Manages a shared Tokio runtime for executing async Rust code
//!   from synchronous Python calls.
//! - **Minimal Copying**: Zero-copy borrowing of input vector data from NumPy arrays into Rust; FlatBuffer responses returned as PyBytes.

#![forbid(unsafe_code)]

use pyo3::prelude::*;
use std::sync::Arc;

mod bindings;

use bindings::*;

#[pymodule]
fn _contextra(_py: Python<'_>, m: &Bound<'_, pyo3::types::PyModule>) -> PyResult<()> {
    check_subinterpreter_guard(_py)?;
    m.add("__version__", "0.1.0")?;

    // Initialize per-interpreter Tokio runtime state
    let worker_threads = parse_worker_threads_env();

    let rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(worker_threads)
        .thread_name("contextra-py-worker")
        .enable_all()
        .build()
        .map_err(|e| {
            pyo3::exceptions::PyRuntimeError::new_err(format!(
                "Failed to create tokio runtime for contextra-py: {}",
                e
            ))
        })?;

    let runtime = Arc::new(rt);
    let state = PyRuntimeState {
        runtime,
        worker_threads,
    };
    m.add("_runtime_state", Py::new(_py, state)?)?;

    m.add_function(wrap_pyfunction!(open, m)?)?;
    m.add_function(wrap_pyfunction!(_trigger_panic_for_test, m)?)?;
    m.add_class::<PyContextra>()?;
    m.add_class::<PyCollection>()?;
    m.add_class::<PySearchResult>()?;
    m.add_class::<PyDocument>()?;
    m.add_class::<PyVectorIndexStats>()?;
    m.add_class::<PyStorageStats>()?;
    m.add_class::<PyDbStats>()?;

    // Exceptions
    m.add("ContextraError", _py.get_type::<ContextraError>())?;
    m.add("ContextraIOError", _py.get_type::<ContextraIOError>())?;
    m.add("ContextraIndexError", _py.get_type::<ContextraIndexError>())?;
    m.add("ContextraValueError", _py.get_type::<ContextraValueError>())?;
    m.add("ContextraCryptoError", _py.get_type::<ContextraCryptoError>())?;
    m.add(
        "ContextraInternalError",
        _py.get_type::<ContextraInternalError>(),
    )?;

    Ok(())
}
