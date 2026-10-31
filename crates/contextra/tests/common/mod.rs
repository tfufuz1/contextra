// FILE-CONTEXT
// ZWECK: Shared integration test helper functions for the `contextra` primary facade test suite.
// INVARIANTEN: Greift ausschließlich über die öffentliche Facade (contextra::*) zu; no internal crate imports.

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

// ANCHOR[contextra:TEST-COM-001]
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

// ANCHOR[contextra:TEST-COM-002]
/// Generiert einen synthetischen Testvektor der gegebenen Dimension mit einem konstanten Wert.
pub fn dummy_vector(dimension: usize, value: f32) -> Vec<f32> {
    vec![value; dimension]
}

// ANCHOR[contextra:TEST-COM-003]
/// Erzeugt standardisierte JSON-Metadaten für Testdokumente mit Tag und Sequenznummer.
pub fn dummy_metadata(tag: &str, sequence: usize) -> Value {
    json!({
        "tag": tag,
        "sequence": sequence,
        "test_env": "facade_integration"
    })
}
