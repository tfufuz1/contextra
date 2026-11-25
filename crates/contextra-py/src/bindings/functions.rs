use contextra_db::{Contextra, ContextraConfig};
use pyo3::prelude::*;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use crate::bindings::common::*;
use crate::bindings::db::PyContextra;
use crate::bindings::runtime_state::get_runtime;

/// Opens or creates a Contextra database at the given path.
///
/// Default `dimension` is 768 to align with `ContextraConfig::default().dimension` in `contextra-db`
/// (matching `nomic-embed-text`, the default ONNX embedding model).
///
/// Supports Python context manager protocol:
/// ```python
/// with contextra.open("./data") as db:
///     db.insert("doc1", vector, {"key": "value"})
/// ```
#[pyfunction]
#[pyo3(signature = (path, dimension=768, max_elements=None, encryption_passphrase=None, distance_metric=None))]
pub fn open(
    py: Python<'_>,
    path: &str,
    dimension: usize,
    max_elements: Option<usize>,
    encryption_passphrase: Option<String>,
    distance_metric: Option<String>,
) -> PyResult<PyContextra> {
    validate_db_path(path)?;
    if dimension == 0 || dimension > 10_000 {
        return Err(pyo3::exceptions::PyValueError::new_err(format!(
            "Dimension must be between 1 and 10000. Got: {}",
            dimension
        )));
    }
    let rt = get_runtime(py)?;
    let worker_threads = rt.metrics().num_workers();
    let mut config = ContextraConfig {
        dimension,
        encryption_passphrase,
        ..Default::default()
    };

    if let Some(me) = max_elements {
        if me == 0 {
            return Err(pyo3::exceptions::PyValueError::new_err(
                "max_elements must be greater than 0",
            ));
        }
        config.max_elements = me;
    }

    if let Some(dm) = distance_metric {
        config.distance_metric = match dm.to_lowercase().as_str() {
            "cosine" => contextra_db::DistanceMetric::Cosine,
            "euclidean" | "l2" => contextra_db::DistanceMetric::Euclidean,
            "dot" | "dotproduct" => contextra_db::DistanceMetric::DotProduct,
            _ => {
                return Err(pyo3::exceptions::PyValueError::new_err(format!(
                    "Unsupported distance metric: {}",
                    dm
                )))
            }
        };
    }

    let poisoned = Arc::new(AtomicBool::new(false));
    let path_string = path.to_string();
    let db = run_blocking_ffi(py, &poisoned, || {
        rt.block_on(Contextra::open_with_config(path_string, config))
            .map_err(contextra_err)
    })?;

    Ok(PyContextra::new(Arc::new(db), rt, worker_threads, poisoned))
}

/// Internal helper function to trigger an unwinding panic inside `run_blocking_ffi` for testing FFI panic isolation.
///
/// # Intention
/// Serves exclusively as a test hook for verifying Python FFI panic boundary isolation (`run_blocking_ffi` / `catch_unwind`).
///
/// # Scope
/// Internal test helper, restricted to `_trigger_panic_for_test` for integration and Python test suites.
///
/// # Risks
/// Triggers panic unwinding. If called outside `run_blocking_ffi` or `catch_unwind`, it will abort or unwind across FFI boundaries.
#[doc(hidden)]
pub(crate) fn trigger_panic_helper(
    py: Python<'_>,
    poisoned: &AtomicBool,
    message: Option<String>,
) -> PyResult<()> {
    let msg = message.unwrap_or_else(|| "Test panic for FFI isolation".to_string());
    run_blocking_ffi(py, poisoned, move || -> PyResult<()> {
        std::panic::resume_unwind(Box::new(msg));
    })
}

/// Internal Python binding function for testing FFI panic isolation.
///
/// # Intention
/// Exposed to Python test suite (`tests/test_panic_isolation.py` and `tests/test_panic_to_pyerr.py`) to verify
/// that panics occurring within Rust core operations are caught and mapped to `PyRuntimeError`.
///
/// # Scope
/// Internal test hook only (`#[doc(hidden)]`).
///
/// # Risks
/// Explicitly triggers a panic payload unwind via `std::panic::resume_unwind`.
#[doc(hidden)]
#[pyfunction]
#[pyo3(signature = (message=None))]
pub fn _trigger_panic_for_test(py: Python<'_>, message: Option<String>) -> PyResult<()> {
    let dummy_poison = AtomicBool::new(false);
    trigger_panic_helper(py, &dummy_poison, message)
}
