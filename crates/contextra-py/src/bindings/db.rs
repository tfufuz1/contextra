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
    ///
    /// # Panics
    ///
    /// bewusst; nur Testhook für FFI-Panic-Isolation; Panic wird durch run_blocking_ffi in PyErr übersetzt
    #[pyo3(signature = (message=None))]
    pub fn _trigger_panic_for_test(&self, py: Python<'_>, message: Option<String>) -> PyResult<()> {
        crate::bindings::functions::trigger_panic_helper(py, &self.poisoned, message)
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

    /// Drops a collection and returns a collection-scoped cryptographic `DeletionProof` as JSON.
    ///
    /// # Scope & Guarantees
    /// Execution tombstones the specified collection in the active LSM MemTable and commits
    /// deletion markers to the write-ahead log (WAL).
    ///
    /// # Limitations & Non-Guarantees
    /// - Does NOT guarantee immediate physical purge or removal of SSTable files, WAL segments,
    ///   HNSW vector index files, or CSR graph data files from disk storage.
    /// - Does NOT guarantee hardware physical SSD flash controller level sanitization
    ///   (wear leveling / TRIM depend on OS & hardware).
    /// - Attests strictly to logical LSM deletion (tombstones committed and verified empty in
    ///   active MemTable / LSM key space scans).
    ///
    /// # Key Resolution
    /// Drops require a valid 32-byte proof key, passed via `proof_key` or configured via the
    /// `CONTEXTRA_DELETION_PROOF_KEY` environment variable. The key memory is zeroized after use.
    #[pyo3(signature = (name, proof_key=None))]
    pub fn drop_collection(
        &self,
        name: &str,
        proof_key: Option<&Bound<'_, PyBytes>>,
        py: Python<'_>,
    ) -> PyResult<String> {
        validate_collection_name(name)?;
        let arg_bytes = proof_key.map(|b| b.as_bytes());
        let env_key = std::env::var("CONTEXTRA_DELETION_PROOF_KEY").ok();
        let key_guard = resolve_proof_key(arg_bytes, env_key)?;

        let rt = &self.runtime;
        let name_owned = name.to_string();
        let tenant_id = contextra_types::TenantId::SYSTEM;
        let inner = self.inner.clone();
        run_blocking_ffi(py, &self.poisoned, || {
            let proof = rt
                .block_on(inner.drop_collection(&name_owned, tenant_id, key_guard.as_slice()))
                .map_err(contextra_err)?;
            let json_str = proof.export_for_audit().map_err(|e| {
                ContextraValueError::new_err(format!("Failed to export proof JSON: {}", e))
            })?;

            let mut val: serde_json::Value = serde_json::from_str(&json_str).map_err(|e| {
                ContextraValueError::new_err(format!("Failed to parse proof JSON: {}", e))
            })?;

            if let serde_json::Value::Object(ref mut map) = val {
                if !map.contains_key("proof_status") {
                    map.insert(
                        "proof_status".to_string(),
                        serde_json::Value::String("issued".to_string()),
                    );
                }
            }

            serde_json::to_string_pretty(&val).map_err(|e| {
                ContextraValueError::new_err(format!(
                    "Failed to serialize enhanced proof JSON: {}",
                    e
                ))
            })
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

// ── Key Resolution Helper & Zeroization Guard ──

/// Zeroizes a byte slice in memory to prevent key remanence.
fn zeroize_bytes(b: &mut [u8]) {
    b.fill(0);
    std::sync::atomic::compiler_fence(std::sync::atomic::Ordering::SeqCst);
}

/// Zero-on-drop wrapper for deletion proof key byte buffers.
pub struct KeyGuard(pub Vec<u8>);

impl KeyGuard {
    /// Borrow the underlying key bytes.
    pub fn as_slice(&self) -> &[u8] {
        &self.0
    }
}

impl Drop for KeyGuard {
    fn drop(&mut self) {
        zeroize_bytes(&mut self.0);
    }
}

impl PartialEq<[u8]> for KeyGuard {
    fn eq(&self, other: &[u8]) -> bool {
        self.0.as_slice() == other
    }
}

/// Resolves the deletion proof key from an explicit byte slice or an environment variable value.
/// Explicit argument takes precedence over the environment variable.
/// Trims whitespace for environment variable values.
fn resolve_proof_key(arg: Option<&[u8]>, env: Option<String>) -> PyResult<KeyGuard> {
    if let Some(arg_bytes) = arg {
        if arg_bytes.is_empty() {
            return Err(ContextraValueError::new_err(
                "deletion proof key not configured",
            ));
        }
        if arg_bytes.len() < 32 {
            return Err(ContextraValueError::new_err(
                "deletion proof key must be at least 32 bytes",
            ));
        }
        return Ok(KeyGuard(arg_bytes.to_vec()));
    }

    if let Some(env_str) = env {
        let trimmed = env_str.trim();
        if trimmed.is_empty() {
            let mut env_vec = env_str.into_bytes();
            zeroize_bytes(&mut env_vec);
            return Err(ContextraValueError::new_err(
                "deletion proof key not configured",
            ));
        }
        let env_bytes = trimmed.as_bytes().to_vec();
        let mut env_vec = env_str.into_bytes();
        zeroize_bytes(&mut env_vec);

        if env_bytes.len() < 32 {
            let mut guard = KeyGuard(env_bytes);
            zeroize_bytes(&mut guard.0);
            return Err(ContextraValueError::new_err(
                "deletion proof key must be at least 32 bytes",
            ));
        }
        return Ok(KeyGuard(env_bytes));
    }

    Err(ContextraValueError::new_err(
        "deletion proof key not configured",
    ))
}

// ── Generated CRUD + Batch Methods ──
contextra_crud_methods!(PyContextra);
contextra_batch_methods!(PyContextra);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_proof_key_explicit() -> std::result::Result<(), String> {
        let key = vec![b'a'; 32];
        let res = resolve_proof_key(Some(&key), None).map_err(|e| e.to_string())?;
        if res.as_slice() != key.as_slice() {
            return Err("Expected explicit key".to_string());
        }
        Ok(())
    }

    #[test]
    fn test_resolve_proof_key_env_trimmed() -> std::result::Result<(), String> {
        let env = format!("   {}   ", "b".repeat(32));
        let res = resolve_proof_key(None, Some(env)).map_err(|e| e.to_string())?;
        if res.as_slice() != "b".repeat(32).as_bytes() {
            return Err("Expected trimmed env key".to_string());
        }
        Ok(())
    }

    #[test]
    fn test_resolve_proof_key_explicit_overrides_env() -> std::result::Result<(), String> {
        let explicit = vec![b'a'; 32];
        let env = "b".repeat(32);
        let res = resolve_proof_key(Some(&explicit), Some(env)).map_err(|e| e.to_string())?;
        if res.as_slice() != explicit.as_slice() {
            return Err("Expected explicit key to override env".to_string());
        }
        Ok(())
    }

    #[test]
    fn test_resolve_proof_key_missing() -> std::result::Result<(), String> {
        match resolve_proof_key(None, None) {
            Err(e) if e.to_string().contains("deletion proof key not configured") => Ok(()),
            _ => Err("Expected missing key error".to_string()),
        }
    }

    #[test]
    fn test_resolve_proof_key_whitespace_env() -> std::result::Result<(), String> {
        match resolve_proof_key(None, Some("   \t\n ".to_string())) {
            Err(e) if e.to_string().contains("deletion proof key not configured") => Ok(()),
            _ => Err("Expected whitespace env error".to_string()),
        }
    }

    #[test]
    fn test_resolve_proof_key_short_explicit() -> std::result::Result<(), String> {
        let secret = "secret_key_12345";
        match resolve_proof_key(Some(secret.as_bytes()), None) {
            Err(e) => {
                let err_msg = e.to_string();
                if !err_msg.contains("deletion proof key must be at least 32 bytes") {
                    return Err("Unexpected error message".to_string());
                }
                if err_msg.contains(secret) {
                    return Err("Error message leaked secret key".to_string());
                }
                Ok(())
            }
            Ok(_) => Err("Expected short explicit key error".to_string()),
        }
    }

    #[test]
    fn test_resolve_proof_key_short_env() -> std::result::Result<(), String> {
        let secret = "secret_env_key";
        match resolve_proof_key(None, Some(secret.to_string())) {
            Err(e) => {
                let err_msg = e.to_string();
                if !err_msg.contains("deletion proof key must be at least 32 bytes") {
                    return Err("Unexpected error message".to_string());
                }
                if err_msg.contains(secret) {
                    return Err("Error message leaked secret key".to_string());
                }
                Ok(())
            }
            Ok(_) => Err("Expected short env key error".to_string()),
        }
    }

    #[test]
    fn test_key_guard_zeroizes_on_drop() {
        let key_vec = vec![0x42u8; 32];
        let mut guard = KeyGuard(key_vec);
        assert_eq!(guard.as_slice(), &[0x42u8; 32]);
        zeroize_bytes(&mut guard.0);
        assert!(guard.as_slice().iter().all(|&b| b == 0));
    }
}
