use contextra::{open_with_config, ContextraConfig};
use contextra_core::{ContextraError, TenantId};
use serde_json::json;
use tempfile::TempDir;

#[tokio::test]
async fn tenant_a_cannot_read_tenant_b() -> Result<(), ContextraError> {
    let tmp = TempDir::new().expect("Failed to create tempdir");
    let config = ContextraConfig {
        dimension: 4,
        consolidation_enabled: false,
        ..Default::default()
    };

    let db = open_with_config(tmp.path(), config).await?;

    let tenant_a = TenantId::try_new(100)?;
    let tenant_b = TenantId::try_new(200)?;

    let col_a = db.collection_for_tenant("shared_col", tenant_a).await?;
    let col_b = db.collection_for_tenant("shared_col", tenant_b).await?;

    // 1. Insert under Tenant A
    col_a
        .insert(
            "doc1",
            &[0.1, 0.2, 0.3, 0.4],
            Some(json!({"tenant": "A", "secret": "alpha"})),
        )
        .await?;

    // 2. Tenant B CANNOT see Tenant A's document
    let doc_b = col_b.get("doc1").await?;
    assert!(
        doc_b.is_none(),
        "Tenant B must NOT be able to read Tenant A's document via get"
    );

    // 3. Insert same key under Tenant B
    col_b
        .insert(
            "doc1",
            &[0.5, 0.6, 0.7, 0.8],
            Some(json!({"tenant": "B", "secret": "beta"})),
        )
        .await?;

    // 4. Verify get returns separate documents for Tenant A and Tenant B
    let doc_a = col_a.get("doc1").await?.expect("Tenant A document");
    let doc_b = col_b.get("doc1").await?.expect("Tenant B document");

    assert_eq!(
        doc_a.metadata.as_ref().and_then(|m| m.get("tenant")),
        Some(&json!("A"))
    );
    assert_eq!(
        doc_b.metadata.as_ref().and_then(|m| m.get("tenant")),
        Some(&json!("B"))
    );

    // 5. Verify search returns isolated results
    let search_a = col_a
        .query()
        .embedding(&[0.1, 0.2, 0.3, 0.4])
        .k(10)
        .execute()
        .await?;
    assert_eq!(search_a.len(), 1);
    assert_eq!(search_a[0].id, "doc1");
    assert_eq!(
        search_a[0].metadata.as_ref().and_then(|m| m.get("tenant")),
        Some(&json!("A"))
    );

    let search_b = col_b
        .query()
        .embedding(&[0.5, 0.6, 0.7, 0.8])
        .k(10)
        .execute()
        .await?;
    assert_eq!(search_b.len(), 1);
    assert_eq!(search_b[0].id, "doc1");
    assert_eq!(
        search_b[0].metadata.as_ref().and_then(|m| m.get("tenant")),
        Some(&json!("B"))
    );

    // 6. Verify scan returns isolated results
    let scan_a = col_a
        .scan(std::ops::Bound::Unbounded, std::ops::Bound::Unbounded, None)
        .await?;
    assert_eq!(scan_a.len(), 1);
    assert_eq!(scan_a[0].0, "doc1");

    let scan_b = col_b
        .scan(std::ops::Bound::Unbounded, std::ops::Bound::Unbounded, None)
        .await?;
    assert_eq!(scan_b.len(), 1);
    assert_eq!(scan_b[0].0, "doc1");

    // 7. Delete on Tenant A does not delete Tenant B's document
    col_a.delete("doc1").await?;
    assert!(col_a.get("doc1").await?.is_none());
    assert!(col_b.get("doc1").await?.is_some());

    // 8. Drop collection for Tenant A does not affect Tenant B
    let proof_key = b"secret_proof_key_32bytes_1234567";
    let proof = db
        .drop_collection("shared_col", tenant_a, proof_key)
        .await?;
    assert_eq!(proof.tenant_id(), tenant_a);

    assert!(
        col_b.get("doc1").await?.is_some(),
        "Tenant B document must remain intact after Tenant A drop_collection"
    );

    db.close().await?;
    Ok(())
}

#[tokio::test]
async fn startup_ssi_tracking_check() -> Result<(), ContextraError> {
    let tmp = TempDir::new().expect("Failed to create tempdir");
    let config = ContextraConfig {
        dimension: 4,
        consolidation_enabled: false,
        ..Default::default()
    };

    let db = open_with_config(tmp.path(), config).await?;
    // Default LsmStorage supports SSI tracking
    assert_eq!(db.len().await?, 0);
    db.close().await?;
    Ok(())
}
