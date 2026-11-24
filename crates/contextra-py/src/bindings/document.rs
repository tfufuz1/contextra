//! # Dokument-Repräsentation für Python (`PyDocument`)
//!
//! Dieses Modul stellt die Klasse `PyDocument` bereit, welche abgerufene
//! Dokumente aus der Contextra-Datenbank im Python-Interpreter repräsentiert.
//!
//! ## Ring-Zugehörigkeit
//! Ring 4 (FFI-Boundary / Bindings).
//!
//! ## Eigenschaften
//! Ein Dokument besteht aus einer eindeutigen Zeichenketten-ID (`id`) und optionalen
//! Metadaten (`metadata`), die als natives Python-Objekt (z. B. Dictionary) gehalten werden.

use pyo3::prelude::*;

// ANCHOR[contextra-py:DOC-001]
/// Ein aus Contextra abgerufener Datensatz (Dokument).
///
/// # Felder
/// - `id`: Der Primärschlüssel des Dokuments als String.
/// - `metadata`: Assoziierte Metadaten des Dokuments (z.B. Python-Dict mit Freitextfeldern).
#[pyclass(get_all)]
pub struct PyDocument {
    /// Die eindeutige Dokumenten-ID.
    pub id: String,
    /// Optionale Metadaten, die mit dem Dokument verknüpft sind.
    pub metadata: Option<PyObject>,
}

#[pymethods]
impl PyDocument {
    // Liefert eine lesbare String-Repräsentation des Dokuments für REPL und Logging.
    fn __repr__(&self) -> String {
        format!("Document(id='{}')", self.id)
    }
}
