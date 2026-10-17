use contextra_crypto::deletion_proof::claim_text::WitAssurance;
use contextra_engine::deletion::{ForgetOptions, PurgeMode};
use contextra_engine::{Contextra, ContextraConfig};
use tempfile::TempDir;

#[tokio::test]
async fn test_forget_returns_receipt_with_stage() -> contextra_types::Result<()> {
    let tmp = TempDir::new().map_err(|e| contextra_types::ContextraError::InvalidInput(e.to_string()))?;
    let config = ContextraConfig {
        dimension: 4,
        ..Default::default()
    };

    let db = Contextra::open_with_config(tmp.path(), config).await?;

    db.insert("doc1", &[1.0, 0.0, 0.0, 0.0], None).await?;

    let receipt = db
        .forget("doc1", ForgetOptions::default())
        .await?;

    assert_eq!(receipt.assurance_requested, WitAssurance::S1);
    assert_eq!(receipt.assurance_reached, WitAssurance::S1);
    assert!(!receipt.layers.is_empty());
    assert!(receipt.proof.is_some());

    db.close().await?;
    Ok(())
}
