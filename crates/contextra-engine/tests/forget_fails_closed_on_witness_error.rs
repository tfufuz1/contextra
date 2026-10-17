use contextra_crypto::deletion_proof::claim_text::WitAssurance;
use contextra_engine::deletion::{ForgetOptions, PurgeMode};
use contextra_engine::{Contextra, ContextraConfig};
use tempfile::TempDir;

#[tokio::test]
async fn test_forget_fails_closed_on_witness_error() -> contextra_types::Result<()> {
    let tmp = TempDir::new().map_err(|e| contextra_types::ContextraError::InvalidInput(e.to_string()))?;
    let config = ContextraConfig {
        dimension: 4,
        ..Default::default()
    };

    let db = Contextra::open_with_config(tmp.path(), config).await?;

    db.insert("doc_fail", &[0.5, 0.5, 0.0, 0.0], None).await?;

    // Request S4 with keyring available flag and purge mode set, but without initialized keyring record
    let opts = ForgetOptions::new()
        .with_assurance(WitAssurance::S4)
        .with_purge(PurgeMode::Purge)
        .with_keyring_available(true);

    let receipt = db.forget("doc_fail", opts).await?;

    // Should fail closed: assurance_reached == S0, proof == None
    assert_eq!(receipt.assurance_reached, WitAssurance::S0);
    assert!(receipt.proof.is_none());

    db.close().await?;
    Ok(())
}
