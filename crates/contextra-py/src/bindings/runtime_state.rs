use pyo3::prelude::*;
use std::sync::Arc;
use tokio::runtime::Runtime;

use crate::bindings::common::{MAX_WORKER_THREADS, MIN_WORKER_THREADS};

/// Parses and clamps `CONTEXTRA_WORKER_THREADS` environment variable to `[MIN_WORKER_THREADS, MAX_WORKER_THREADS]`.
///
/// If `CONTEXTRA_WORKER_THREADS` is unset, invalid, or 0, clamps to `MIN_WORKER_THREADS..=MAX_WORKER_THREADS`
/// to guarantee that Tokio multi-thread runtime builder is never called with 0 worker threads.
pub fn parse_worker_threads_env() -> usize {
    let default_threads = (std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4)
        / 2)
    .clamp(MIN_WORKER_THREADS, MAX_WORKER_THREADS);

    std::env::var("CONTEXTRA_WORKER_THREADS")
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .map(|val| val.clamp(MIN_WORKER_THREADS, MAX_WORKER_THREADS))
        .unwrap_or(default_threads)
}

/// Holds the per-interpreter/per-module Tokio runtime state and worker thread configuration.
#[pyclass(name = "RuntimeState")]
#[derive(Clone)]
pub struct PyRuntimeState {
    pub runtime: Arc<Runtime>,
    pub worker_threads: usize,
}

/// Retrieves or initializes the per-interpreter Tokio runtime attached to the `_contextra` module state.
///
/// Reads `CONTEXTRA_WORKER_THREADS` on initialization for the current interpreter/module context.
/// Evaluates the `Result` of attaching `_runtime_state` to the `_contextra` module via `setattr`.
/// On failure, propagates a `PyRuntimeError` to prevent runtime leaks and repeated runtime instantiation.
pub fn get_runtime(py: Python<'_>) -> PyResult<Arc<Runtime>> {
    let module = py
        .import("contextra._contextra")
        .or_else(|_| py.import("_contextra"))?;
    if let Ok(state_attr) = module.getattr("_runtime_state") {
        if let Ok(state) = state_attr.extract::<PyRef<'_, PyRuntimeState>>() {
            return Ok(state.runtime.clone());
        }
    }

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
        runtime: runtime.clone(),
        worker_threads,
    };

    let py_state = Py::new(py, state)?;
    module.setattr("_runtime_state", py_state).map_err(|e| {
        pyo3::exceptions::PyRuntimeError::new_err(format!(
            "Failed to attach '_runtime_state' to '_contextra' module: {}",
            e
        ))
    })?;

    Ok(runtime)
}
