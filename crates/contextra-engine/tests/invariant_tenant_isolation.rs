use contextra_engine::{Contextra, ContextraConfig};
use contextra_types::{ContextraError, TenantId};
use serde_json::json;
use tempfile::TempDir;

#[tokio::test]
async fn test_tenant_isolation_cross_tenant_document_and_collection_operations(
) -> contextra_types::Result<()> {
    let tmp = TempDir::new().expect("Failed to create temporary directory");
    let config = ContextraConfig {
        dimension: 4,
        consolidation_enabled: false,
        ..Default::default()
    };

    let db = Contextra::open_with_config(tmp.path(), config).await?;

    let tenant_x_id = TenantId::try_new(100).expect("TenantId 100");
    let _tenant_y_id = TenantId::try_new(200).expect("TenantId 200");

    let col_x = db.collection("tenant_x_collection").await?;
    let col_y = db.collection("tenant_y_collection").await?;

    // 1. Insert document under Tenant X
    col_x
        .insert(
            "doc_tenant_x_1",
            &[0.1, 0.2, 0.3, 0.4],
            Some(json!({ "tenant": "X", "confidential": true })),
        )
        .await?;

    // 2. Assert Tenant Y collection CANNOT see Tenant X document
    let doc_from_y = col_y.get("doc_tenant_x_1").await?;
    assert!(
        doc_from_y.is_none(),
        "Document inserted under Tenant X must NOT be visible to Tenant Y"
    );

    // 3. Assert Tenant Y cannot delete Tenant X document
    col_y.delete("doc_tenant_x_1").await?;

    // Verify Tenant X still has its document
    let doc_from_x = col_x.get("doc_tenant_x_1").await?;
    assert!(
        doc_from_x.is_some(),
        "Delete operation on Tenant Y must NOT delete document from Tenant X"
    );

    // 4. Drop collection for Tenant X and verify DeletionProof
    let proof = db
        .drop_collection("tenant_x_collection", tenant_x_id, b"secret_proof_key_32bytes_1234567")
        .await?;

    assert_eq!(
        proof.tenant_id(),
        tenant_x_id,
        "DeletionProof tenant_id must match requested tenant"
    );

    // Tenant Y collection remains intact
    col_y
        .insert(
            "doc_tenant_y_1",
            &[0.5, 0.6, 0.7, 0.8],
            Some(json!({ "tenant": "Y" })),
        )
        .await?;

    let doc_y = col_y.get("doc_tenant_y_1").await?;
    assert!(
        doc_y.is_some(),
        "Tenant Y collection must remain intact after Tenant X drop_collection"
    );

    // 5. Error path: Dropping "default" collection MUST return typed ContextraError::InvalidInput
    let drop_default_res = db
        .drop_collection("default", tenant_x_id, b"secret_proof_key_32bytes_1234567")
        .await;

    match drop_default_res {
        Err(ContextraError::InvalidInput(msg)) => {
            assert!(
                msg.contains("default"),
                "Error message must indicate default collection cannot be dropped"
            );
        }
        other => {
            assert!(
                false,
                "Expected ContextraError::InvalidInput when dropping default collection, got: {:?}",
                other
            );
        }
    }

    db.close().await?;
    Ok(())
}
