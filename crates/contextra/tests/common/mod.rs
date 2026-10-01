// FILE-CONTEXT
// ZWECK: Shared integration test helper functions for the `contextra` primary facade test suite.
// INVARIANTEN: Greift ausschließlich über die öffentliche Facade (contextra::*) zu; no internal crate imports.

use contextra::performance_profile::PerformanceProfile;
use contextra::{builder, Contextra, ContextraError};
use serde_json::{json, Value};
use tempfile::TempDir;

/// Creates a fast temporary `Contextra` instance configured with `PerformanceProfile::BareMetal` for rapid integration tests.
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

/// Generates a normalized dummy vector of specified dimension filled with a constant float value.
pub fn dummy_vector(dimension: usize, value: f32) -> Vec<f32> {
    vec![value; dimension]
}

/// Generates a standard JSON metadata object for integration test documents.
pub fn dummy_metadata(tag: &str, sequence: usize) -> Value {
    json!({
        "tag": tag,
        "sequence": sequence,
        "test_env": "facade_integration"
    })
}
