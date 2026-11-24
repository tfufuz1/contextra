//! # LSM-Speicher-Statistiken (`PyStorageStats`)
//!
//! Dieses Modul definiert `PyStorageStats`, welche Kennzahlen über die
//! zugrundeliegende LSM-Speicherengine (Log-Structured Merge-Tree) bereitstellt.
//!
//! ## Ring-Zugehörigkeit
//! Ring 4 (FFI-Boundary / Bindings).
//!
//! ## Hintergrund
//! Im LSM-Tree werden neue Schreiboperationen zuerst im Hauptspeicher (MemTable)
//! gepuffert und später in unveränderliche Segmentdateien (SSTables) auf die Festplatte
//! geschrieben. Die Metriken spiegeln diesen Lebenszyklus wider.

use pyo3::prelude::*;

// ANCHOR[contextra-py:STAT-002]
/// Statistiken der persistenten LSM-Speicherengine (`StorageStats`).
///
/// # Felder
/// - `num_segments`: Anzahl der auf der Festplatte persistierten SSTable-Segmente.
/// - `total_size_bytes`: Gesamter belegter Festplattenplatz aller SSTable-Dateien in Bytes.
/// - `memtable_size_bytes`: Aktuelle Belegung der In-Memory-MemTables im Arbeitsspeicher in Bytes.
#[pyclass(get_all, name = "StorageStats")]
#[derive(Clone)]
pub struct PyStorageStats {
    /// Number of SSTable segments.
    pub num_segments: usize,
    /// Total size of all SSTables in bytes.
    pub total_size_bytes: u64,
    /// Total size of memtables in bytes.
    pub memtable_size_bytes: u64,
}

#[pymethods]
impl PyStorageStats {
    // Liefert eine formatierte Zusammenfassung der Speichergrößen für Monitoring und Logging.
    fn __repr__(&self) -> String {
        format!(
            "StorageStats(num_segments={}, total_size_bytes={}, memtable_size_bytes={})",
            self.num_segments, self.total_size_bytes, self.memtable_size_bytes
        )
    }
}
