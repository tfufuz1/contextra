#![cfg(not(loom))]
use contextra_engine::{Contextra, ContextraConfig, TenantPolicy};
use contextra_types::{ContextraError, TenantId};
use tempfile::tempdir;

#[tokio::test]
async fn test_tenant_policy_optional_allows_non_tenant_collections() -> contextra_types::Result<()> {
    let dir = tempdir().unwrap();
    let config = ContextraConfig::default().with_tenant_policy(TenantPolicy::Optional);
    let db = Contextra::open_with_config(dir.path(), config).await?;

    let col = db.collection("default").await;
    assert!(
        col.is_ok(),
        "TenantPolicy::Optional must allow non-tenant collection access"
    );

    db.close().await?;
    Ok(())
}

#[tokio::test]
async fn test_tenant_policy_required_denies_non_tenant_collections() -> contextra_types::Result<()> {
    let dir = tempdir().unwrap();
    let config = ContextraConfig::default().with_tenant_policy(TenantPolicy::Required);
    let db = Contextra::open_with_config(dir.path(), config).await?;

    // Non-tenant collection access must fail
    let col_res = db.collection("default").await;
    assert!(
        col_res.is_err(),
        "TenantPolicy::Required must deny non-tenant collection access"
    );
    if let Err(ContextraError::InvalidInput(msg)) = col_res {
        assert!(
            msg.contains("TenantPolicy::Required") || msg.contains("forbidden") || msg.contains("required"),
            "Error message must clearly explain tenant isolation requirement: {msg}"
        );
    } else {
        panic!("Expected ContextraError::InvalidInput on non-tenant collection access when TenantPolicy::Required");
    }

    // Tenant-specific collection access must succeed
    let tenant_1 = TenantId::try_new(201).unwrap();
    let tenant_col = db.collection_for_tenant("my_col", tenant_1).await;
    assert!(
        tenant_col.is_ok(),
        "TenantPolicy::Required must allow collection_for_tenant"
    );

    db.close().await?;
    Ok(())
}
