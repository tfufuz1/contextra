//! # Suchergebnis-Repräsentation für Python (`PySearchResult`)
//!
//! Dieses Modul definiert `PySearchResult`, das einzelne Treffer einer
//! Vektor- oder Hybrid-Suche im Python-Ökosystem kapselt.
//!
//! ## Ring-Zugehörigkeit
//! Ring 4 (FFI-Boundary / Bindings).
//!
//! ## Relevanz
//! Wird als Vektor von Suchtreffern von Methoden wie `vector_search` oder `hybrid_search`
//! zurückgegeben und enthält Dokumenten-ID, Ähnlichkeits-Score sowie Metadaten.

use pyo3::prelude::*;

// ANCHOR[contextra-py:SRCH-001]
/// Ein einzelnes Suchergebnis aus einer Vektor- oder Hybrid-Suche.
///
/// # Felder
/// - `id`: Der Primärschlüssel des gefundenen Dokuments.
/// - `score`: Der berechnete Ähnlichkeitswert (höherer Wert bedeutet höhere Relevanz/Ähnlichkeit).
/// - `metadata`: Assoziierte Metadaten des Treffers (falls bei der Suche angefordert).
#[pyclass(get_all)]
pub struct PySearchResult {
    /// Die Dokumenten-ID.
    pub id: String,
    /// Der Ähnlichkeits-Score (höher = relevanter).
    pub score: f32,
    /// Optionale Metadaten des Dokuments.
    pub metadata: Option<PyObject>,
}

#[pymethods]
impl PySearchResult {
    // Liefert eine formatierte Darstellung für Debug-Ausgaben und interaktive Python-Sessions.
    fn __repr__(&self) -> String {
        format!("SearchResult(id='{}', score={:.4})", self.id, self.score)
    }
}
