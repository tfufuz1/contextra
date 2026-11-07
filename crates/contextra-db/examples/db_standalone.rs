//! # Contextra-DB Standalone Example (M10)
//!
//! Demonstrates isolated embedding and standalone usage of `contextra-db`
//! without external multi-crate dependencies.

use contextra_db::{Contextra, ContextraConfig, DistanceMetric};
use serde_json::json;
use tempfile::TempDir;

#[tokio::main]
async fn main() -> contextra_types::Result<()> {
    let tmp_dir = TempDir::new().map_err(|e| contextra_types::ContextraError::Io(e))?;
    let config = ContextraConfig {
        dimension: 4,
        max_elements: 1_000,
        distance_metric: DistanceMetric::Cosine,
        ..Default::default()
    };

    let db = Contextra::open_with_config(tmp_dir.path(), config).await?;
    println!(
        "Contextra-DB standalone instance successfully opened at {:?}",
        tmp_dir.path()
    );

    db.insert(
        "doc-42",
        &[1.0, 0.0, 0.0, 0.0],
        Some(json!({ "topic": "modular-architecture", "status": "standalone" })),
    )
    .await?;

    let results = db.search(&[1.0, 0.0, 0.0, 0.0], 5).await?;
    println!("Retrieved {} results in standalone mode:", results.len());
    for res in results {
        println!(" - Doc ID: {}, Score: {:.4}", res.id, res.score);
    }

    db.close().await?;
    println!("Contextra-DB standalone run completed successfully.");
    Ok(())
}
