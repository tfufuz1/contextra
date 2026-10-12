#![cfg(not(feature = "encryption-at-rest"))]

use contextra_engine::{Contextra, ContextraConfig};
use contextra_types::{ContextraError, TenantId};
use tempfile::TempDir;

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[tokio::test]
async fn test_no_proof_generated_without_encryption_at_rest_feature() -> TestResult {
    let tmp = TempDir::new()?;
    let config = ContextraConfig {
        dimension: 4,
        consolidation_enabled: false,
        ..Default::default()
    };

    let db = Contextra::open_with_config(tmp.path(), config).await?;
    let tenant_id = TenantId::try_new(100).expect("TenantId 100");
    let proof_key = b"secret_proof_signing_key_32bytes!";

    // Create a collection under tenant
    let col = db.collection_for_tenant("test_col", tenant_id).await?;
    col.insert(
        "doc-1",
        &[0.1, 0.2, 0.3, 0.4],
        Some(serde_json::json!({ "text": "sample" })),
    )
    .await?;

    // Calling drop_collection without encryption-at-rest feature MUST return typed CapabilityUnsupported error
    let res = db.drop_collection("test_col", tenant_id, proof_key).await;

    match res {
        Err(ContextraError::CapabilityUnsupported { capability, reason }) => {
            assert_eq!(capability, "deletion_proof");
            assert!(
                reason.contains("encryption-at-rest"),
                "Error message should explain that encryption-at-rest feature is required"
            );
        }
        Ok(_fake_proof) => {
            return Err(
                "VETO VIOLATION: Issued a deletion proof stub without encryption-at-rest feature!"
                    .into(),
            );
        }
        Err(other) => {
            return Err(format!(
                "Expected ContextraError::CapabilityUnsupported, got unexpected error: {:?}",
                other
            )
            .into());
        }
    }

    db.close().await?;
    Ok(())
}
