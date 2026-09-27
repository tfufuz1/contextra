//! LangGraph `BaseStore` state persistence adapter.

use pyo3::prelude::*;
use pyo3::types::PyDict;
use _contextra::PyCollection;

/// Adapter implementing LangGraph's `BaseStore` state persistence protocol over `PyCollection`.
#[pyclass(name = "LangGraphStoreAdapter")]
pub struct LangGraphStoreAdapter {
    collection: Py<PyCollection>,
}

impl LangGraphStoreAdapter {
    /// Formats a namespace vector and key into a canonical KV key.
    fn format_key(&self, namespace: &[String], key: &str) -> String {
        if namespace.is_empty() {
            format!("lg:default:{}", key)
        } else {
            format!("lg:{}:{}", namespace.join("/"), key)
        }
    }

    /// Formats a namespace vector into a KV key prefix for range scanning.
    fn format_prefix(&self, namespace: &[String]) -> String {
        if namespace.is_empty() {
            "lg:default:".to_string()
        } else {
            format!("lg:{}:", namespace.join("/"))
        }
    }
}

#[pymethods]
impl LangGraphStoreAdapter {
    /// Creates a new `LangGraphStoreAdapter` bound to a `PyCollection`.
    #[new]
    pub fn new(collection: Py<PyCollection>) -> Self {
        Self { collection }
    }

    /// Saves (puts) a state checkpoint value for a given namespace and key.
    pub fn put(
        &self,
        py: Python<'_>,
        namespace: Vec<String>,
        key: String,
        value: Bound<'_, PyDict>,
    ) -> PyResult<()> {
        let full_key = self.format_key(&namespace, &key);
        let col = self.collection.borrow(py);
        col.put_kv(py, full_key.into_pyobject(py)?.as_any(), &value)
    }

    /// Async-style alias `aput` for Python async store protocol compatibility.
    pub fn aput(
        &self,
        py: Python<'_>,
        namespace: Vec<String>,
        key: String,
        value: Bound<'_, PyDict>,
    ) -> PyResult<()> {
        self.put(py, namespace, key, value)
    }

    /// Retrieves (gets) a state checkpoint value for a given namespace and key.
    pub fn get(
        &self,
        py: Python<'_>,
        namespace: Vec<String>,
        key: String,
    ) -> PyResult<Option<PyObject>> {
        let full_key = self.format_key(&namespace, &key);
        let col = self.collection.borrow(py);
        col.get_kv(py, full_key.into_pyobject(py)?.as_any())
    }

    /// Async-style alias `aget` for Python async store protocol compatibility.
    pub fn aget(
        &self,
        py: Python<'_>,
        namespace: Vec<String>,
        key: String,
    ) -> PyResult<Option<PyObject>> {
        self.get(py, namespace, key)
    }

    /// Searches state items within a namespace by prefix scan.
    #[pyo3(signature = (namespace, limit=None))]
    pub fn search(
        &self,
        py: Python<'_>,
        namespace: Vec<String>,
        limit: Option<usize>,
    ) -> PyResult<Vec<(String, PyObject)>> {
        let prefix = self.format_prefix(&namespace);
        let col = self.collection.borrow(py);
        col.scan_prefix(py, &prefix, limit)
    }

    /// Async-style alias `asearch` for Python async store protocol compatibility.
    #[pyo3(signature = (namespace, limit=None))]
    pub fn asearch(
        &self,
        py: Python<'_>,
        namespace: Vec<String>,
        limit: Option<usize>,
    ) -> PyResult<Vec<(String, PyObject)>> {
        self.search(py, namespace, limit)
    }

    /// Deletes a state checkpoint value for a given namespace and key.
    pub fn delete(&self, py: Python<'_>, namespace: Vec<String>, key: String) -> PyResult<()> {
        let full_key = self.format_key(&namespace, &key);
        let col = self.collection.borrow(py);
        col.delete(py, full_key.into_pyobject(py)?.as_any())
    }
}
