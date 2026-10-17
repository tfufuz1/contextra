use contextra_crypto::deletion_proof::claim_text::WitAssurance;
use contextra_engine::deletion::ForgetOptions;
use contextra_engine::{Contextra, ContextraConfig};
use tempfile::TempDir;

#[tokio::test]
async fn test_forget_collection_uses_witness_path() -> contextra_types::Result<()> {
    let tmp = TempDir::new().map_err(|e| contextra_types::ContextraError::InvalidInput(e.to_string()))?;
    let config = ContextraConfig {
        dimension: 4,
        ..Default::default()
    };

    let db = Contextra::open_with_config(tmp.path(), config).await?;

    let col = db.collection("test_forget_col").await?;
    col.insert("c1_doc", &[1.0, 0.0, 0.0, 0.0], None).await?;

    let receipt = db
        .forget_collection("test_forget_col", ForgetOptions::default())
        .await?;

    assert_eq!(receipt.assurance_requested, WitAssurance::S1);
    assert!(!receipt.layers.is_empty());

    db.close().await?;
    Ok(())
}
