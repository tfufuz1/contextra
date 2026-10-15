//! Integration test verifying `drop_collection_in_scope` behavior across `Untenanted` and `Tenant` scopes.

#![cfg(feature = "encryption-at-rest")]

use contextra_engine::Contextra;
use contextra_types::TenantId;
use tempfile::TempDir;

#[tokio::test]
async fn test_drop_collection_scope_isolation_and_fail_closed(
) -> Result<(), Box<dyn std::error::Error>> {
    let tmp = TempDir::new()?;
    let db = Contextra::open(tmp.path()).await?;
    let proof_key = b"01234567890123456789012345678901";

    // 1. Insert document into Untenanted scope (TenantId::SYSTEM)
    let col_untenanted = db.collection("my_col").await?;
    col_untenanted.insert("doc1", &[0.1f32; 768], None).await?;
    db.flush().await?;

    // 2. Attempt to drop in Tenant scope (TenantId 1) -> mismatch: Untenanted data remains
    let _proof_tenant = db
        .drop_collection("my_col", TenantId::try_new(1)?, proof_key)
        .await?;

    // Untenanted document must STILL exist
    assert!(
        col_untenanted.get("doc1").await?.is_some(),
        "Untenanted document should remain after Tenant drop"
    );

    // 3. Drop in Untenanted scope (SYSTEM) -> successfully removes data
    let proof_untenanted = db
        .drop_collection("my_col", TenantId::SYSTEM, proof_key)
        .await?;

    // Document is gone
    assert!(
        col_untenanted.get("doc1").await?.is_none(),
        "Untenanted document must be gone after Untenanted drop"
    );

    // 4. Proof must contain all mandatory ExcludedScope variants
    let exported = proof_untenanted.export_for_audit()?;
    assert!(exported.contains("ConsolidatedAndDistilled"));
    assert!(exported.contains("LlmParameterMemory"));
    assert!(exported.contains("StorageMediaRemanence"));
    assert!(exported.contains("UnregisteredBackups"));
    assert!(exported.contains("OsPageCacheSwapCoreDumps"));
    assert!(exported.contains("ClientAndModelContext"));
    assert!(exported.contains("AggregateDerivedStatistics"));
    assert!(exported.contains("LogsAndTelemetryOutsideProcess"));
    assert!(exported.contains("ExportedData"));

    Ok(())
}
