// FILE-CONTEXT
// STAND: 2026-10-06
// ZWECK: Durability and crash/reopen simulation test for drop_collection and DeletionProof issuance (P02 / F-03 HIGH).
// INVARIANTEN: Deletion tombstones must be durable on disk prior to DeletionProof issuance. Upon crash/reopen without graceful shutdown, dropped keys must not reappear.

#![cfg(not(loom))]
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use contextra_engine::{Contextra, ContextraConfig};
use contextra_types::{Result, TenantId};
use tempfile::tempdir;

#[tokio::test]
async fn test_drop_collection_proof_durability_crash_reopen() -> Result<()> {
    let dir = tempdir().unwrap();
    let data_path = dir.path().to_path_buf();

    let tenant_id = TenantId::try_new(42).unwrap();
    let collection_name = "durable_drop_col";
    let proof_key = b"secret_proof_signing_key_32bytes!";

    let vec_sample = vec![0.1f32; 768];

    // Phase 1: Initialize engine, create collection, and insert data
    {
        let config = ContextraConfig::default();

        let engine = Contextra::open_with_config(&data_path, config).await?;
        let col = engine
            .collection_for_tenant(collection_name, tenant_id)
            .await?;

        col.insert(
            "doc-100",
            &vec_sample,
            Some(serde_json::json!({
                "text": "Document content that will be dropped and verified for durability."
            })),
        )
        .await?;

        // Verify document is present before drop
        let doc_retrieved = col.get("doc-100").await?;
        assert!(doc_retrieved.is_some(), "Document must exist before drop");

        // Execute drop_collection and obtain DeletionProof
        let proof = engine
            .drop_collection(collection_name, tenant_id, proof_key)
            .await?;

        // Basic verification of proof
        assert_eq!(
            proof.tenant_id(),
            tenant_id,
            "Proof tenant_id must match requested tenant"
        );

        // Verify in-memory handle is removed
        let col_list = engine.list_collections_for_tenant(tenant_id).await?;
        assert!(
            !col_list.contains(&collection_name.to_string()),
            "Dropped collection must not appear in list_collections_for_tenant"
        );

        // Drop the engine handle immediately without clean shutdown / explicit flush
        drop(engine);
    }

    // Phase 2: Reopen engine from the exact same storage directory
    {
        let config = ContextraConfig::default();

        let engine = Contextra::open_with_config(&data_path, config).await?;

        // Verify collection index list after recovery scan
        let col_list = engine.list_collections_for_tenant(tenant_id).await?;
        assert!(
            !col_list.contains(&collection_name.to_string()),
            "Dropped collection must remain absent after restart"
        );

        // Accessing the collection again creates a fresh empty collection handle
        let col = engine
            .collection_for_tenant(collection_name, tenant_id)
            .await?;

        let doc_retrieved = col.get("doc-100").await?;
        assert!(
            doc_retrieved.is_none(),
            "Dropped document doc-100 must not reappear after crash/reopen recovery"
        );
    }

    Ok(())
}
