// FILE-CONTEXT
// STAND:       2026-10-13
// ZWECK:       Encapsulate database opening via ContextraBuilder for binaries and IPC interfaces
// INVARIANTEN: Opens database exclusively via ContextraBuilder

use contextra::{Contextra, ContextraBuilder};
use contextra_types::Result;
use std::path::Path;

/// Opens a Contextra database instance using ContextraBuilder configured for environment.
pub async fn open_db_from_env(path: &Path) -> Result<Contextra> {
    ContextraBuilder::new(768)
        .with_storage_path(path)
        .build()
        .await
}
