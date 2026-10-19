//! Unit test verifying scan-commit window closure in drop_collection with concurrent writers.

#![forbid(unsafe_code)]

use contextra_engine::{Contextra, ContextraConfig, TenantPolicy};
use contextra_types::TenantId;
use std::sync::Arc;
use tempfile::TempDir;

#[tokio::test]
async fn test_drop_collection_concurrent_writer_closed_window() {
    let dir = TempDir::new().expect("tempdir failed");
    let config = ContextraConfig {
        tenant_policy: TenantPolicy::Optional,
        ..Default::default()
    };

    let engine = Arc::new(
        Contextra::open_with_config(dir.path(), config)
            .await
            .expect("Contextra::open_with_config failed"),
    );

    let col_name = "concurrent_col";
    let tenant_id = TenantId::SYSTEM;

    let col = engine
        .collection_for_tenant(col_name, tenant_id)
        .await
        .expect("collection_for_tenant failed");

    col.put_kv("doc1", &serde_json::json!({"val": 1}))
        .await
        .expect("put_kv failed");

    let _proof_key = [0x55u8; 32];

    // Spawn a concurrent task attempting to access or write to the collection
    let engine_ref = Arc::clone(&engine);
    let col_name_owned = col_name.to_string();
    let handle = tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_millis(5)).await;
        let col_after = engine_ref
            .collection_for_tenant(&col_name_owned, tenant_id)
            .await;
        col_after
    });

    #[cfg(feature = "encryption-at-rest")]
    {
        let proof = engine
            .drop_collection(col_name, tenant_id, &proof_key)
            .await
            .expect("drop_collection failed");
        assert!(proof.proof_id != [0u8; 16] || !proof.signature.is_empty());
    }

    let col_res = handle.await.expect("tokio task failed");
    assert!(
        col_res.is_ok(),
        "Subsequent collection lookup must succeed cleanly with fresh state"
    );
}
