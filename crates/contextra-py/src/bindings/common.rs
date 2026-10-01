use pyo3::exceptions::{PyKeyError, PyPermissionError, PyRuntimeError, PyValueError};
use pyo3::prelude::*;
use pythonize::{depythonize, pythonize};
use std::sync::atomic::{AtomicBool, Ordering};

use crate::bindings::document::PyDocument;
use crate::bindings::search_result::PySearchResult;

// ─── Custom Exceptions ──────────────────────────────────────────────────────

pyo3::create_exception!(_contextra, ContextraError, pyo3::exceptions::PyException);
pyo3::create_exception!(_contextra, ContextraIOError, ContextraError);
pyo3::create_exception!(_contextra, ContextraIndexError, ContextraError);
pyo3::create_exception!(_contextra, ContextraValueError, ContextraError);
pyo3::create_exception!(_contextra, ContextraCryptoError, ContextraError);
pyo3::create_exception!(_contextra, ContextraInternalError, ContextraError);

// ─── Shared Constants ───────────────────────────────────────────────────────

/// Minimum allowed worker threads for Tokio multi-thread runtime.
pub const MIN_WORKER_THREADS: usize = 1;
/// Maximum allowed worker threads for Tokio multi-thread runtime.
pub const MAX_WORKER_THREADS: usize = 256;

/// Maximum allowed length for document string IDs (1024 characters).
pub const MAX_ID_LENGTH: usize = 1024;
/// Maximum allowed length for relationship labels (256 characters).
pub const MAX_LABEL_LENGTH: usize = 256;
/// Maximum batch size for batch insertion/upsertion (10,000 items).
pub const MAX_BATCH_SIZE: usize = 10_000;

// ─── Shared Helper Functions ────────────────────────────────────────────────

/// Converts a Python dict to a serde_json::Value.
pub fn dict_to_json(d: &pyo3::Bound<'_, pyo3::types::PyDict>) -> PyResult<serde_json::Value> {
    depythonize(d).map_err(|e| PyValueError::new_err(format!("Metadata error: {}", e)))
}

/// Converts an optional Python dict to an optional serde_json::Value.
pub fn opt_dict_to_json(
    metadata: Option<&pyo3::Bound<'_, pyo3::types::PyDict>>,
) -> PyResult<Option<serde_json::Value>> {
    match metadata {
        Some(d) => Ok(Some(dict_to_json(d)?)),
        None => Ok(None),
    }
}

/// Validates that a relationship label is non-empty, contains no null bytes, and does not exceed maximum length.
pub fn validate_label(label: &str) -> PyResult<()> {
    if label.trim().is_empty() {
        return Err(ContextraValueError::new_err(
            "Relationship label cannot be empty or whitespace-only",
        ));
    }
    if label.contains('\0') {
        return Err(ContextraValueError::new_err(
            "Relationship label cannot contain null bytes",
        ));
    }
    if label.len() > MAX_LABEL_LENGTH {
        return Err(ContextraValueError::new_err(format!(
            "Relationship label exceeds maximum length of {} bytes. Got: {}",
            MAX_LABEL_LENGTH,
            label.len()
        )));
    }
    Ok(())
}

/// Validates that a string ID is non-empty, contains no null bytes, and does not exceed maximum length.
pub fn validate_id(id: &str) -> PyResult<()> {
    if id.trim().is_empty() {
        return Err(ContextraValueError::new_err(
            "Document ID cannot be empty or whitespace-only",
        ));
    }
    if id.contains('\0') {
        return Err(ContextraValueError::new_err(
            "Document ID cannot contain null bytes",
        ));
    }
    if id.len() > MAX_ID_LENGTH {
        return Err(ContextraValueError::new_err(format!(
            "Document ID exceeds maximum length of {} bytes. Got: {}",
            MAX_ID_LENGTH,
            id.len()
        )));
    }
    Ok(())
}

/// Validates that a collection name is non-empty, contains no null bytes, and does not exceed maximum length.
pub fn validate_collection_name(name: &str) -> PyResult<()> {
    if name.trim().is_empty() {
        return Err(ContextraValueError::new_err(
            "Collection name cannot be empty or whitespace-only",
        ));
    }
    if name.contains('\0') {
        return Err(ContextraValueError::new_err(
            "Collection name cannot contain null bytes",
        ));
    }
    if name.len() > MAX_ID_LENGTH {
        return Err(ContextraValueError::new_err(format!(
            "Collection name exceeds maximum length of {} bytes. Got: {}",
            MAX_ID_LENGTH,
            name.len()
        )));
    }
    Ok(())
}

/// Validates that a database storage path is non-empty and contains no null bytes.
pub fn validate_db_path(path: &str) -> PyResult<()> {
    if path.trim().is_empty() {
        return Err(ContextraValueError::new_err(
            "Database path cannot be empty or whitespace-only",
        ));
    }
    if path.contains('\0') {
        return Err(ContextraValueError::new_err(
            "Database path cannot contain null bytes",
        ));
    }
    Ok(())
}

/// Validates that a search query text is non-empty, contains no null bytes, and does not exceed maximum length.
pub fn validate_query_text(text: &str) -> PyResult<()> {
    if text.trim().is_empty() {
        return Err(ContextraValueError::new_err(
            "Search query text cannot be empty or whitespace-only",
        ));
    }
    if text.contains('\0') {
        return Err(ContextraValueError::new_err(
            "Search query text cannot contain null bytes",
        ));
    }
    if text.len() > MAX_ID_LENGTH {
        return Err(ContextraValueError::new_err(format!(
            "Query text exceeds maximum length of {} bytes. Got: {}",
            MAX_ID_LENGTH,
            text.len()
        )));
    }
    Ok(())
}

/// Validates batch size against maximum resource allocation limits.
pub fn validate_batch_size(size: usize) -> PyResult<()> {
    if size == 0 {
        return Err(ContextraValueError::new_err("Batch cannot be empty"));
    }
    if size > MAX_BATCH_SIZE {
        return Err(ContextraValueError::new_err(format!(
            "Batch size {} exceeds maximum allowed limit of {}",
            size, MAX_BATCH_SIZE
        )));
    }
    Ok(())
}

/// Validates that a vector slice is non-empty and contains no NaN or infinite values.
pub fn validate_vector(vector: &[f32]) -> PyResult<()> {
    if vector.is_empty() {
        return Err(ContextraValueError::new_err("Vector cannot be empty"));
    }
    if vector.iter().any(|x| x.is_nan() || x.is_infinite()) {
        return Err(ContextraValueError::new_err(
            "Vector contains NaN or infinite float values",
        ));
    }
    Ok(())
}

/// Validates a document ID provided as a string or numeric value.
pub fn validate_id_obj(id_obj: &pyo3::Bound<'_, pyo3::types::PyAny>) -> PyResult<String> {
    if let Ok(id_str) = id_obj.extract::<String>() {
        validate_id(&id_str)?;
        Ok(id_str)
    } else if let Ok(id_int) = id_obj.extract::<i128>() {
        if id_int < 0 {
            return Err(ContextraValueError::new_err(
                "Document ID cannot be a negative integer",
            ));
        }
        #[cfg(not(feature = "docid-128"))]
        if id_int > (u64::MAX as i128) {
            return Err(ContextraValueError::new_err(
                "Document ID integer value exceeds maximum allowed bound (u64::MAX)",
            ));
        }
        Ok(id_int.to_string())
    } else if id_obj.is_instance_of::<pyo3::types::PyInt>() {
        Err(ContextraValueError::new_err(
            "Document ID integer value exceeds maximum allowed bound",
        ))
    } else {
        Err(ContextraValueError::new_err(
            "Document ID must be a string or non-negative integer",
        ))
    }
}

/// Explicitly checks whether the module is being imported inside a CPython sub-interpreter.
///
/// If running inside a sub-interpreter (interpreter ID != 0), returns `PyImportError` explaining
/// that sub-interpreters are not supported due to per-process Tokio runtime isolation.
pub fn check_subinterpreter_guard(py: Python<'_>) -> PyResult<()> {
    let current_id: Option<i64> = if let Ok(interp_mod) = py.import("_xxsubinterpreters") {
        interp_mod
            .call_method0("get_current")
            .ok()
            .and_then(|id| id.extract().ok())
    } else if let Ok(interp_mod) = py.import("_interpreters") {
        interp_mod
            .call_method0("get_current")
            .ok()
            .and_then(|id| id.extract().ok())
    } else {
        None
    };

    if let Some(id) = current_id {
        if id != 0 {
            return Err(pyo3::exceptions::PyImportError::new_err(
                "contextra does not support loading in sub-interpreters due to per-process Tokio runtime isolation",
            ));
        }
    }
    Ok(())
}

/// Safely executes a blocking closure across FFI boundaries with thread state release
/// and panic containment to guarantee no Rust panic propagates across FFI boundaries into Python.
///
/// Checks the `poisoned` flag prior to execution and sets `poisoned = true` if a Rust panic is caught,
/// preventing further operations against a potentially corrupted internal state.
pub fn run_blocking_ffi<F, R>(py: Python<'_>, poisoned: &AtomicBool, f: F) -> PyResult<R>
where
    F: FnOnce() -> PyResult<R> + Send,
    R: Send,
{
    // Poison-nach-Panic-Invariante (APM-PY-A): Check poison flag before executing FFI operation
    if poisoned.load(Ordering::SeqCst) {
        return Err(pyo3::exceptions::PyRuntimeError::new_err(
            "engine poisoned after previous panic, create a new instance",
        ));
    }

    let panic_result =
        py.allow_threads(|| std::panic::catch_unwind(std::panic::AssertUnwindSafe(f)));

    match panic_result {
        Ok(res) => res,
        Err(panic_payload) => {
            // Set poison flag atomically so subsequent calls are immediately blocked
            poisoned.store(true, Ordering::SeqCst);
            let panic_msg = if let Some(s) = panic_payload.downcast_ref::<&str>() {
                (*s).to_string()
            } else if let Some(s) = panic_payload.downcast_ref::<String>() {
                s.clone()
            } else {
                "Unknown panic inside Rust core".to_string()
            };
            Err(pyo3::exceptions::PyRuntimeError::new_err(format!(
                "Rust panic caught at FFI boundary: {}",
                panic_msg
            )))
        }
    }
}

/// Converts a serde_json::Value to a Python object.
pub fn json_to_py(py: Python<'_>, val: &serde_json::Value) -> PyResult<PyObject> {
    pythonize(py, val)
        .map(|o| o.unbind())
        .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(format!("Metadata error: {}", e)))
}

/// Converts a contextra_db::Document to a PyDocument.
pub fn doc_to_py(py: Python<'_>, d: contextra_db::Document) -> PyResult<PyDocument> {
    let meta_py = match d.metadata {
        Some(ref m) => Some(json_to_py(py, m)?),
        None => None,
    };
    Ok(PyDocument {
        id: d.id,
        metadata: meta_py,
    })
}

/// Converts a Vec of SearchResult to Vec of PySearchResult.
pub fn results_to_py(
    py: Python<'_>,
    results: Vec<contextra_db::SearchResult>,
) -> PyResult<Vec<PySearchResult>> {
    let mut py_res = Vec::with_capacity(results.len());
    for r in results {
        let meta_py = match r.metadata {
            Some(ref m) => Some(json_to_py(py, m)?),
            None => None,
        };
        py_res.push(PySearchResult {
            id: r.id,
            score: r.score,
            metadata: meta_py,
        });
    }
    Ok(py_res)
}

/// Maps a ContextraError into a structured Python PyErr with `kind`, `message`, and `details` attributes.
pub fn contextra_err(e: contextra_core::ContextraError) -> PyErr {
    let dto = contextra_core::ContextraErrorDto::from(&e);
    Python::with_gil(|py| {
        let py_err = match dto.kind.as_str() {
            "NotFound" => PyKeyError::new_err(dto.message.clone()),
            "Conflict" | "Transaction" | "TransactionTimeout" => {
                PyRuntimeError::new_err(dto.message.clone())
            }
            "PolicyViolation"
            | "NamespaceViolation"
            | "Sandbox"
            | "MemoryLimitExceeded"
            | "SandboxTimeout" => PyPermissionError::new_err(dto.message.clone()),
            "InvalidInput"
            | "Serialization"
            | "Json"
            | "ParseError"
            | "Bincode"
            | "InvalidSequenceNumber"
            | "CheckpointNotFound"
            | "LimitExceeded" => ContextraValueError::new_err(dto.message.clone()),
            "Storage" | "Io" | "WalCorruption" | "ChecksumMismatch" => {
                ContextraIOError::new_err(dto.message.clone())
            }
            "Index" | "HnswConnectivityDegraded" | "Text" => {
                ContextraIndexError::new_err(dto.message.clone())
            }
            "Crypto" => ContextraCryptoError::new_err(dto.message.clone()),
            "MemoryBudgetExceeded" => pyo3::exceptions::PyMemoryError::new_err(dto.message.clone()),
            "CapabilityUnsupported" => {
                pyo3::exceptions::PyNotImplementedError::new_err(dto.message.clone())
            }
            "Internal" | "Cluster" => ContextraInternalError::new_err(dto.message.clone()),
            _ => ContextraError::new_err(dto.message.clone()),
        };
        let value = py_err.value(py);
        if value.setattr("kind", dto.kind).is_err() {
            // Ignore non-fatal attribute attachment error if py_err instance doesn't support setattr
        }
        if value.setattr("message", dto.message).is_err() {
            // Ignore non-fatal attribute attachment error
        }
        if let Some(ref details) = dto.details {
            if let Ok(details_py) = json_to_py(py, details) {
                if value.setattr("details", details_py).is_err() {
                    // Ignore non-fatal attribute attachment error
                }
            }
        }
        py_err
    })
}
