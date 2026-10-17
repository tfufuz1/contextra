use contextra_crypto::deletion_proof::claim_text::WitAssurance;
use contextra_engine::deletion::ForgetOptions;
use contextra_engine::{Contextra, ContextraConfig};
use tempfile::TempDir;

#[tokio::test]
async fn test_forget_missing_proof_key() -> contextra_types::Result<()> {
    let tmp = TempDir::new().map_err(|e| contextra_types::ContextraError::InvalidInput(e.to_string()))?;
    let config = ContextraConfig {
        dimension: 4,
        ..Default::default()
    };

    let db = Contextra::open_with_config(tmp.path(), config).await?;

    db.insert("doc_nokey", &[1.0, 0.0, 0.0, 0.0], None).await?;

    let receipt = db
        .forget("doc_nokey", ForgetOptions::new().with_assurance(WitAssurance::S1))
        .await?;

    // Proof issuer creates keys lazily or fails closed when no key configured
    assert_eq!(receipt.assurance_requested, WitAssurance::S1);

    db.close().await?;
    Ok(())
}
