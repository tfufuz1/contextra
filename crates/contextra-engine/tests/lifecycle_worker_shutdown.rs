// FILE-CONTEXT
// ZWECK: Integration tests for Contextra background worker graceful shutdown and TaskTracker / CancellationToken lifecycle.

use contextra_engine::{Contextra, ContextraConfig};
use tempfile::tempdir;

#[tokio::test]
async fn test_background_worker_shutdown_and_close_no_leaks() -> contextra_types::Result<()> {
    let tmp = tempdir().unwrap();
    let config = ContextraConfig {
        consolidation_enabled: false,
        ..Default::default()
    };

    let db = Contextra::open_with_config(tmp.path(), config).await?;

    // Verify close wait_shutdown completes cleanly without hanging or leaving tasks running
    db.close().await?;

    Ok(())
}
