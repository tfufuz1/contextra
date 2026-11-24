//! # Gesamtdatenbank-Statistiken (`PyDbStats`)
//!
//! Dieses Modul definiert `PyDbStats`, welche eine ganzheitliche Sicht auf den
//! Systemzustand, Drift-Metriken, Kalibrierungsfehler sowie Speicher- und Indexstatistiken bietet.
//!
//! ## Ring-Zugehörigkeit
//! Ring 4 (FFI-Boundary / Bindings).
//!
//! ## Erfasste Bereiche
//! - **Drift & Kalibrierung:** Lyapunov-Driftstatus und ECE (Expected Calibration Error) des IsotonicCalibrators.
//! - **PID-Regelung:** Aktuelle dynamische Kandidaten-Pool-Größe für Vektorsuchen.
//! - **Subsysteme:** Eingebettete `PyVectorIndexStats` (HNSW) und `PyStorageStats` (LSM-Tree).

use pyo3::prelude::*;

use crate::bindings::storage_stats::PyStorageStats;
use crate::bindings::vector_index_stats::PyVectorIndexStats;

// ANCHOR[contextra-py:STAT-001]
/// Ganzheitliche Datenbank- und Systembeobachtungs-Statistiken (`DbStats`).
///
/// # Felder
/// - `drift_status`: Zustand der Vektor-Driftanalyse ("stabil", "warnung", "kritisch", "unbekannt").
/// - `calibration_ece`: Erwarteter Kalibrierungsfehler (ECE = Expected Calibration Error, 0.0 bis 1.0).
/// - `last_calibration_at`: UNIX-Zeitstempel der letzten Kalibrierungsmodell-Berechnung.
/// - `active_memory_count`: Anzahl aktiver Dokumente/Memories in der Default-Collection.
/// - `pid_pool_size`: Aktuelle Kandidaten-Pool-Größe des PID-Reglers.
/// - `index_stats`: Detaillierte Statistiken über den HNSW-Vektorindex.
/// - `storage_stats`: Detaillierte Kennzahlen des LSM-Speichers (MemTable, SSTables, WAL).
#[pyclass(get_all, name = "DbStats")]
#[derive(Clone)]
pub struct PyDbStats {
    /// Lyapunov drift status ("stabil", "warnung", "kritisch", "unbekannt").
    pub drift_status: String,
    /// Expected Calibration Error (ECE) from IsotonicCalibrator if available.
    pub calibration_ece: Option<f32>,
    /// UNIX timestamp of the last calibration model rebuild.
    pub last_calibration_at: Option<u64>,
    /// Total count of active memory documents in default collection.
    pub active_memory_count: usize,
    /// Current PID-regulated candidate pool size if active.
    pub pid_pool_size: Option<usize>,
    /// Statistics for the vector index.
    pub index_stats: PyVectorIndexStats,
    /// Statistics for the LSM storage engine.
    pub storage_stats: PyStorageStats,
}

#[pymethods]
impl PyDbStats {
    // Liefert eine formatierte Zusammenfassung der Kernmetriken für Monitoring und Auswertung.
    fn __repr__(&self) -> String {
        format!(
            "DbStats(vectors={}, active_memories={}, drift='{}', ece={}, size_bytes={})",
            self.index_stats.num_vectors,
            self.active_memory_count,
            self.drift_status,
            self.calibration_ece
                .map(|e| format!("{:.4}", e))
                .unwrap_or_else(|| "None".to_string()),
            self.storage_stats.total_size_bytes + self.storage_stats.memtable_size_bytes
        )
    }
}
