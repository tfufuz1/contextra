use pyo3::prelude::*;

/// A document retrieved from Contextra.
#[pyclass(get_all)]
pub struct PyDocument {
    /// The document ID.
    pub id: String,
    /// Metadata associated with the document.
    pub metadata: Option<PyObject>,
}

#[pymethods]
impl PyDocument {
    fn __repr__(&self) -> String {
        format!("Document(id='{}')", self.id)
    }
}
