
//! # Modul: Gemeinsame Integrations-Test-Hilfsfunktionen (`common`)
//!
//! Stellt wiederverwendbare Hilfsfunktionen für die Integrationstests der Fassade bereit.
//!
//! ## Invarianten
//! - Verwendet ausschließlich die öffentliche Schnittstelle von `contextra`.
//! - Keine direkten internen Crate-Imports aus tieferen Ringen.

use contextra::performance_profile::PerformanceProfile;
use contextra::{builder, Contextra, ContextraError};
use serde_json::{json, Value};
use tempfile::TempDir;

/// Erstellt eine schnelle, temporäre `Contextra`-Testdatenbank mit `PerformanceProfile::BareMetal`.
///
/// Vermeidet Festplatten-Syncs im WAL für maximale Testausführungsgeschwindigkeit.
pub async fn create_test_db(
    tmp_dir: &TempDir,
    dimension: usize,
) -> Result<Contextra, ContextraError> {
    builder(dimension)
        .with_storage_path(tmp_dir.path())
        .with_performance_profile(PerformanceProfile::BareMetal)
        .build()
        .await
}

/// Generiert einen synthetischen Testvektor der gegebenen Dimension mit einem konstanten Wert.
pub fn dummy_vector(dimension: usize, value: f32) -> Vec<f32> {
    vec![value; dimension]
}

/// Erzeugt standardisierte JSON-Metadaten für Testdokumente mit Tag und Sequenznummer.
pub fn dummy_metadata(tag: &str, sequence: usize) -> Value {
    json!({
        "tag": tag,
        "sequence": sequence,
        "test_env": "facade_integration"
    })
}
