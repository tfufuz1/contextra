#![cfg(not(loom))]
use contextra_engine::{Contextra, ContextraConfig};
use contextra_types::{ContextraError, Result};
use tempfile::TempDir;

#[tokio::test]
async fn kv_store_unsupported_is_err_or_removed() -> Result<()> {
    let tmp_dir = TempDir::new().expect("Failed to create temporary directory");
    let config = ContextraConfig {
        dimension: 4,
        consolidation_enabled: false,
        ..Default::default()
    };

    let db = Contextra::open_with_config(tmp_dir.path(), config).await?;
    let col = db.collection("default").await?;

    let res = col.kv_store();
    #[cfg(feature = "encryption-at-rest")]
    {
        // When kv_store is not attached
        assert!(matches!(
            res,
            Err(ContextraError::CapabilityUnsupported { .. })
        ));
    }

    #[cfg(not(feature = "encryption-at-rest"))]
    {
        assert!(matches!(
            res,
            Err(ContextraError::CapabilityUnsupported { .. })
        ));
    }

    Ok(())
}
