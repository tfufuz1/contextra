//! LlamaIndex `BaseMemory`, `BaseChatStore`, and `VectorStore` adapter.

use numpy::PyReadonlyArray1;
use pyo3::prelude::*;
use pyo3::types::PyDict;
use _contextra::PyCollection;

/// Adapter implementing LlamaIndex's `BaseMemory`, `BaseChatStore`, and `VectorStore` protocols over `PyCollection`.
#[pyclass(name = "LlamaIndexStoreAdapter")]
pub struct LlamaIndexStoreAdapter {
    collection: Py<PyCollection>,
}

#[pymethods]
impl LlamaIndexStoreAdapter {
    /// Creates a new `LlamaIndexStoreAdapter` bound to a `PyCollection`.
    #[new]
    pub fn new(collection: Py<PyCollection>) -> Self {
        Self { collection }
    }

    /// Adds a single document node into the vector store.
    #[pyo3(signature = (node_id, vector, metadata=None))]
    pub fn add_node<'py>(
        &self,
        py: Python<'py>,
        node_id: &Bound<'py, PyAny>,
        vector: PyReadonlyArray1<'py, f32>,
        metadata: Option<Bound<'py, PyDict>>,
    ) -> PyResult<()> {
        let col = self.collection.borrow(py);
        col.insert(py, node_id, vector, metadata)
    }

    /// Queries the vector store for top-k nearest neighbor nodes.
    #[pyo3(signature = (vector, similarity_top_k=10))]
    pub fn query<'py>(
        &self,
        py: Python<'py>,
        vector: PyReadonlyArray1<'py, f32>,
        similarity_top_k: usize,
    ) -> PyResult<Vec<PyObject>> {
        let col = self.collection.borrow(py);
        let results = col.search(py, vector, similarity_top_k)?;
        let mut py_results = Vec::with_capacity(results.len());
        for r in results {
            py_results.push(Py::new(py, r)?.into_any());
        }
        Ok(py_results)
    }

    /// Deletes a node or document by ID from the vector store.
    pub fn delete_node<'py>(
        &self,
        py: Python<'py>,
        node_id: &Bound<'py, PyAny>,
    ) -> PyResult<()> {
        let col = self.collection.borrow(py);
        col.delete(py, node_id)
    }

    /// Stores a list of chat messages for a key in LlamaIndex `BaseChatStore` memory.
    pub fn set_messages(&self, py: Python<'_>, key: &str, messages: Vec<PyObject>) -> PyResult<()> {
        let col = self.collection.borrow(py);
        for (idx, msg) in messages.into_iter().enumerate() {
            let msg_key = format!("llama:{}:msg:{:08}", key, idx);
            let dict = PyDict::new(py);
            dict.set_item("key", key)?;
            dict.set_item("index", idx)?;
            dict.set_item("message", msg)?;
            col.put_kv(py, msg_key.into_pyobject(py)?.as_any(), &dict)?;
        }
        Ok(())
    }

    /// Retrieves chat messages for a key from LlamaIndex `BaseChatStore` memory.
    pub fn get_messages(&self, py: Python<'_>, key: &str) -> PyResult<Vec<PyObject>> {
        let col = self.collection.borrow(py);
        let prefix = format!("llama:{}:msg:", key);
        let items = col.scan_prefix(py, &prefix, None)?;
        let mut messages = Vec::with_capacity(items.len());
        for (_k, val_obj) in items {
            if let Ok(dict) = val_obj.extract::<Bound<'_, PyDict>>(py) {
                if let Ok(Some(msg)) = dict.get_item("message") {
                    messages.push(msg.unbind());
                    continue;
                }
            }
            messages.push(val_obj);
        }
        Ok(messages)
    }

    /// Resets/deletes chat messages for a key in LlamaIndex `BaseChatStore`.
    pub fn reset_messages(&self, py: Python<'_>, key: &str) -> PyResult<()> {
        let col = self.collection.borrow(py);
        let prefix = format!("llama:{}:msg:", key);
        let items = col.scan_prefix(py, &prefix, None)?;
        for (k, _) in items {
            let _ = col.delete(py, k.into_pyobject(py)?.as_any());
        }
        Ok(())
    }
}
