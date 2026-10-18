// FILE-CONTEXT
// STAND: 2026-10-15
// ZWECK: Tests for erasure_backend port functionality, TenantScopedStorage in delete path, and EraseError::NotAvailable.

#![cfg(not(loom))]
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use contextra_engine::collection::crud::delete::DocEraseOptions;
use contextra_engine::deletion::forget_options::{ForgetOptions, PurgeMode};
use contextra_engine::Collection;
use contextra_ports::{BoxFuture, StorageEngine, StorageStats, VectorIndex};
use contextra_store::lsm::{LsmConfig, LsmStorage};
use contextra_store::tenant_codec::TenantScopedStorage;
use contextra_testkit::InMemoryStorageEngine;
use contextra_types::{ContextraError, Result, TenantId};
use contextra_vector::HnswIndex;
use std::sync::atomic::AtomicU64;
use std::sync::Arc;
use tempfile::tempdir;

#[tokio::test]
async fn test_tenant_scoped_storage_delete_mode_off_works() -> Result<()> {
    let dir = tempdir().unwrap();
    let lsm_config = LsmConfig {
        path: dir.path().to_path_buf(),
        ..Default::default()
    };
    let lsm = Arc::new(LsmStorage::new(lsm_config).await.unwrap());
    let tenant_id = TenantId::try_new(100).unwrap();
    let storage = Arc::new(TenantScopedStorage::new(lsm, tenant_id));

    let index = Arc::new(
        HnswIndex::try_new(contextra_vector::HnswConfig {
            dimension: 4,
            ..Default::default()
        })
        .unwrap(),
    );
    let next_tx = Arc::new(AtomicU64::new(1));

    let col = Collection::new(
        "default".to_string(),
        storage,
        index,
        next_tx,
        4,
        contextra_text::Language::English,
    );

    let doc_id = "doc_test_tenant";
    let insert_tx = col.allocate_tx()?;
    let user_key = col.namespaced_key(doc_id.as_bytes(), 0);
    col.storage().put(insert_tx, &user_key, b"test_payload").await?;
    col.storage().commit(insert_tx).await?;

    // Delete in mode Off
    let delete_res = col.delete(doc_id).await;
    assert!(
        delete_res.is_ok(),
        "Delete in mode Off with TenantScopedStorage must succeed"
    );

    Ok(())
}

#[tokio::test]
async fn test_storage_without_erasure_backend_returns_not_available_in_assured_mode() -> Result<()> {
    let storage = Arc::new(InMemoryStorageEngine::new());
    let index = Arc::new(
        HnswIndex::try_new(contextra_vector::HnswConfig {
            dimension: 4,
            ..Default::default()
        })
        .unwrap(),
    );
    let next_tx = Arc::new(AtomicU64::new(1));

    let col = Collection::new(
        "default".to_string(),
        storage,
        index,
        next_tx,
        4,
        contextra_text::Language::English,
    );

    let opts = DocEraseOptions {
        options: ForgetOptions {
            purge: PurgeMode::Purge,
            ..ForgetOptions::default()
        },
    };

    let err = col
        .erase_document("doc_no_backend", &opts)
        .await
        .expect_err("erase_document in Assured mode on engine without ErasureBackend must fail");

    match err {
        ContextraError::CapabilityUnsupported { capability, reason } => {
            assert_eq!(capability, "erasure_backend");
            assert!(reason.contains("Erasure backend not available"));
        }
        other => panic!("Expected CapabilityUnsupported for erasure_backend, got: {:?}", other),
    }

    Ok(())
}
