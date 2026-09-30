use contextra_engine::{Contextra, ContextraConfig};
use contextra_types::TenantId;
use tempfile::tempdir;

#[tokio::test]
async fn test_tenant_index_persistence_and_vector_search_reload() -> contextra_types::Result<()> {
    let dir = tempdir().unwrap();
    let tenant_1 = TenantId::try_new(301).unwrap();
    let tenant_2 = TenantId::try_new(302).unwrap();

    let vec_query = vec![0.5f32; 768];

    // Phase 1: Open DB, insert document for tenant_1
    {
        let db = Contextra::open_with_config(dir.path(), ContextraConfig::default()).await?;
        let col_t1 = db.collection_for_tenant("persistent_col", tenant_1).await?;
        col_t1
            .insert(
                "doc_t1",
                &vec_query,
                Some(serde_json::json!({"text": "important tenant 1 knowledge"})),
            )
            .await?;

        // Vector search on t1 finds the doc
        let results = col_t1.search(&vec_query, 5).await?;
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "doc_t1");

        // Close DB & drop all handles
        db.close().await?;
    }

    // Phase 2: Re-open DB on same directory
    {
        let db = Contextra::open_with_config(dir.path(), ContextraConfig::default()).await?;

        // Tenant 1 reloads collection
        let col_t1 = db.collection_for_tenant("persistent_col", tenant_1).await?;
        let results_t1 = col_t1.search(&vec_query, 5).await?;
        assert_eq!(
            results_t1.len(),
            1,
            "Tenant 1 vector search must find persisted document after reload"
        );
        assert_eq!(results_t1[0].id, "doc_t1");

        // Tenant 2 loads same collection name
        let col_t2 = db.collection_for_tenant("persistent_col", tenant_2).await?;
        let results_t2 = col_t2.search(&vec_query, 5).await?;
        assert_eq!(
            results_t2.len(),
            0,
            "Tenant 2 must see 0 documents in their isolated partition of persistent_col"
        );

        db.close().await?;
    }

    Ok(())
}

#[tokio::test]
async fn test_tenant_drop_collection_invalidates_cache_and_clears_data() -> contextra_types::Result<()> {
    let dir = tempdir().unwrap();
    let db = Contextra::open_with_config(dir.path(), ContextraConfig::default()).await?;

    let tenant_1 = TenantId::try_new(303).unwrap();
    let proof_key = b"test_hmac_proof_key_123456789012";

    // 1. Create collection & insert document
    let col1 = db.collection_for_tenant("drop_col", tenant_1).await?;
    let vec = vec![0.2f32; 768];
    col1.insert("doc_drop", &vec, Some(serde_json::json!({"text": "to be dropped"})))
        .await?;

    // 2. Drop collection for tenant
    let proof = db.drop_collection("drop_col", tenant_1, proof_key).await?;
    assert_eq!(proof.tenant_id(), tenant_1);

    // 3. Requesting collection again returns a fresh, empty handle
    let col2 = db.collection_for_tenant("drop_col", tenant_1).await?;
    assert!(
        !std::sync::Arc::ptr_eq(&col1, &col2),
        "Handle after drop_collection must be a new instance, not the old cached handle"
    );

    let doc = col2.get("doc_drop").await?;
    assert!(
        doc.is_none(),
        "New handle after drop_collection must return no old data"
    );

    db.close().await?;
    Ok(())
}
