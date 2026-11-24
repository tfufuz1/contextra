//! # HNSW-Vektorindex-Statistiken (`PyVectorIndexStats`)
//!
//! Dieses Modul definiert `PyVectorIndexStats` für Einblicke in den internen
//! HNSW-Graphen (Hierarchical Navigable Small World) des Vektorindex.
//!
//! ## Ring-Zugehörigkeit
//! Ring 4 (FFI-Boundary / Bindings).
//!
//! ## Bedeutung der Metriken
//! - Vektoranzahl (`num_vectors`): Aktive Vektoren im Index.
//! - Speicherverbrauch (`memory_usage_bytes`): RAM-Verbrauch des Index-Graphen.
//! - Schichten (`num_layers`): Tiefe der HNSW-Hierarchie für logarithmische Suchzeit.

use pyo3::prelude::*;

// ANCHOR[contextra-py:STAT-003]
/// Statistiken über den HNSW-Vektorindex (`VectorIndexStats`).
///
/// # Felder
/// - `num_vectors`: Anzahl der aktuell aktiven (nicht als gelöscht markierten) Vektoren im Index.
/// - `memory_usage_bytes`: Geschätzter Arbeitsspeicherverbrauch des Vektorgraphen in Bytes.
/// - `num_layers`: Anzahl der Schichten (Layer) im hierarchischen HNSW-Graph.
#[pyclass(get_all, name = "VectorIndexStats")]
#[derive(Clone)]
pub struct PyVectorIndexStats {
    /// Number of active (non-deleted) vectors.
    pub num_vectors: usize,
    /// Estimated memory usage in bytes.
    pub memory_usage_bytes: usize,
    /// Number of HNSW layers.
    pub num_layers: usize,
}

#[pymethods]
impl PyVectorIndexStats {
    // Liefert eine formatierte Zusammenfassung der Index-Kennzahlen für Logging und Performance-Audits.
    fn __repr__(&self) -> String {
        format!(
            "VectorIndexStats(num_vectors={}, memory_usage_bytes={}, num_layers={})",
            self.num_vectors, self.memory_usage_bytes, self.num_layers
        )
    }
}
