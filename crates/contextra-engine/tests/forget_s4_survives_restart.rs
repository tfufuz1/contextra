use contextra_crypto::deletion_proof::claim_text::WitAssurance;
use contextra_engine::deletion::{ForgetOptions, PurgeMode};
use contextra_engine::{Contextra, ContextraConfig};
use tempfile::TempDir;

#[tokio::test]
async fn test_forget_s4_survives_restart() -> contextra_types::Result<()> {
    let tmp = TempDir::new().map_err(|e| contextra_types::ContextraError::InvalidInput(e.to_string()))?;
    let path = tmp.path().to_path_buf();
    let config = ContextraConfig {
        dimension: 4,
        ..Default::default()
    };

    {
        let db = Contextra::open_with_config(&path, config.clone()).await?;
        db.insert("s4_doc", &[1.0, 0.0, 0.0, 0.0], None).await?;
        let receipt = db
            .forget("s4_doc", ForgetOptions::default())
            .await?;
        assert!(receipt.assurance_reached >= WitAssurance::S1);
        db.close().await?;
    }

    {
        let db = Contextra::open_with_config(&path, config).await?;
        let doc = db.get("s4_doc").await?;
        assert!(doc.is_none());
        db.close().await?;
    }

    Ok(())
}
