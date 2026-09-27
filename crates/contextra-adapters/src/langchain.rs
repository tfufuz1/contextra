//! LangChain `BaseChatMessageHistory` protocol adapter.

use numpy::PyReadonlyArray1;
use pyo3::prelude::*;
use pyo3::types::PyDict;
use _contextra::PyCollection;

/// Adapter implementing LangChain's `BaseChatMessageHistory` protocol over `PyCollection`.
#[pyclass(name = "LangChainChatMessageHistory")]
pub struct LangChainChatMessageHistory {
    collection: Py<PyCollection>,
    session_id: String,
}

pub type LangChainMemoryAdapter = LangChainChatMessageHistory;

#[pymethods]
impl LangChainChatMessageHistory {
    /// Creates a new `LangChainChatMessageHistory` bound to a `session_id` and `PyCollection`.
    #[new]
    #[pyo3(signature = (collection, session_id))]
    pub fn new(collection: Py<PyCollection>, session_id: String) -> Self {
        Self {
            collection,
            session_id,
        }
    }

    /// Returns the session ID associated with this chat history.
    #[getter]
    pub fn session_id(&self) -> &str {
        &self.session_id
    }

    /// Adds a single message object to the chat history.
    pub fn add_message(&self, py: Python<'_>, message: PyObject) -> PyResult<()> {
        let col = self.collection.borrow(py);
        let msg_id = format!(
            "{}:msg:{}",
            self.session_id,
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        );

        let dict = PyDict::new(py);
        dict.set_item("session_id", &self.session_id)?;
        dict.set_item("message", message)?;

        col.put_kv(py, msg_id.into_pyobject(py)?.as_any(), &dict)?;
        Ok(())
    }

    /// Adds multiple message objects to the chat history.
    pub fn add_messages(&self, py: Python<'_>, messages: Vec<PyObject>) -> PyResult<()> {
        for msg in messages {
            self.add_message(py, msg)?;
        }
        Ok(())
    }

    /// Retrieves all chat messages for this session ID.
    pub fn get_messages(&self, py: Python<'_>) -> PyResult<Vec<PyObject>> {
        let col = self.collection.borrow(py);
        let prefix = format!("{}:msg:", self.session_id);
        let items = col.scan_prefix(py, &prefix, None)?;

        let mut messages = Vec::with_capacity(items.len());
        for (_key, val_obj) in items {
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

    /// Performs vector similarity search over message embeddings (`contextra_search`).
    #[pyo3(signature = (vector, k=10))]
    pub fn search_messages<'py>(
        &self,
        py: Python<'py>,
        vector: PyReadonlyArray1<'py, f32>,
        k: usize,
    ) -> PyResult<Vec<PyObject>> {
        let col = self.collection.borrow(py);
        let results = col.search(py, vector, k)?;
        let mut py_results = Vec::with_capacity(results.len());
        for r in results {
            py_results.push(Py::new(py, r)?.into_any());
        }
        Ok(py_results)
    }

    /// Inserts a message document directly with embedding vector into Contextra collection.
    #[pyo3(signature = (id, vector, metadata=None))]
    pub fn insert_message<'py>(
        &self,
        py: Python<'py>,
        id: &Bound<'py, PyAny>,
        vector: PyReadonlyArray1<'py, f32>,
        metadata: Option<Bound<'py, PyDict>>,
    ) -> PyResult<()> {
        let col = self.collection.borrow(py);
        col.insert(py, id, vector, metadata)
    }

    /// Clears all messages for this session ID.
    pub fn clear(&self, py: Python<'_>) -> PyResult<()> {
        let col = self.collection.borrow(py);
        let prefix = format!("{}:msg:", self.session_id);
        let items = col.scan_prefix(py, &prefix, None)?;
        for (key, _) in items {
            let _ = col.delete(py, key.into_pyobject(py)?.as_any());
        }
        Ok(())
    }
}
