// FILE-CONTEXT
// STAND: 2026-10-13
// ZWECK: Integrationstest fuer drop_collection Layer-Claims und Physikalischen Purge-Status (F2 / A02).
// INVARIANTEN: LsmStorage::purge_keys wird nach Commit aufgerufen. Covered Layers enthalten nur per Scan belegte Layer. Bei PurgeBlocked::Unimplemented enthaelt covered_layers NUR DeletionLayer::LsmMemtable.

#![cfg(not(loom))]
#![cfg(feature = "encryption-at-rest")]
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use contextra_engine::{Contextra, ContextraConfig, DeletionLayer};
use contextra_ports::storage::StorageEngine;
use contextra_types::{Result, TenantId};
use tempfile::tempdir;

#[tokio::test]
async fn test_drop_collection_layer_claims_and_purge_status() -> Result<()> {
    let dir = tempdir().unwrap();
    let data_path = dir.path().to_path_buf();

    let tenant_id = TenantId::try_new(777).unwrap();
    let collection_name = "canary_col";
    let valid_proof_key = b"canary_proof_signing_key_32bytes!!";
    let invalid_short_key = b"short_key";
    let invalid_zero_key = [0u8; 32];

    let config = ContextraConfig::default();
    let engine = Contextra::open_with_config(&data_path, config).await?;

    // 1. Invalid proof key checks (must return InvalidInput)
    let err_short = engine
        .drop_collection_with_report(collection_name, tenant_id, invalid_short_key)
        .await;
    assert!(
        err_short.is_err(),
        "Proof key < 32 bytes must return an error"
    );

    let err_zero = engine
        .drop_collection_with_report(collection_name, tenant_id, &invalid_zero_key)
        .await;
    assert!(
        err_zero.is_err(),
        "All-zero proof key must return an error"
    );

    // 2. Create collection and write canary document
    let col = engine
        .collection_for_tenant(collection_name, tenant_id)
        .await?;

    let vec_sample = vec![0.5f32; 768];
    col.insert(
        "canary-doc-01",
        &vec_sample,
        Some(serde_json::json!({
            "canary_marker": "CANARY_DATA_VALUE_12345"
        })),
    )
    .await?;

    let doc = col.get("canary-doc-01").await?;
    assert!(doc.is_some(), "Canary document must be present");

    // 3. Flush Memtable to SSTable to ensure canary data exists in physical SSTable files on disk
    engine.flush().await?;

    // Verify raw SSTable storage directly still contains canary data before drop
    let tenant_storage = contextra_store::tenant_codec::TenantScopedStorage::new(
        engine.inner_storage(),
        tenant_id,
    );
    let col_prefix = format!("__col:{}:", collection_name);
    let scan_before = tenant_storage
        .scan_prefix(col_prefix.as_bytes())
        .await?;
    assert!(
        !scan_before.is_empty(),
        "Storage prefix must contain keys in SSTable"
    );

    // 4. Execute drop_collection_with_report
    let report = engine
        .drop_collection_with_report(collection_name, tenant_id, valid_proof_key)
        .await?;

    // 5. Verify report status and covered_layers
    let purge_str = format!("{:?}", report.physical_purge);
    assert!(
        purge_str.contains("NotPerformed"),
        "Expected physical purge to be NotPerformed on HEAD, got: {purge_str}"
    );
    assert!(
        purge_str.contains("Unimplemented") || purge_str.contains("ActiveSnapshot") || purge_str.contains("InsufficientDiskSpace"),
        "Expected physical purge reason to contain PurgeBlocked variant, got: {purge_str}"
    );

    let covered_layers = &report.proof.covered_layers;
    assert!(
        covered_layers.contains(&DeletionLayer::LsmMemtable),
        "covered_layers MUST contain DeletionLayer::LsmMemtable"
    );

    assert!(
        !covered_layers.contains(&DeletionLayer::SsTableAllLevels),
        "covered_layers MUST NOT claim DeletionLayer::SsTableAllLevels when physical purge was NotPerformed"
    );
    assert!(
        !covered_layers.contains(&DeletionLayer::WalAllSegments { seq_after: 0 }),
        "covered_layers MUST NOT claim WalAllSegments without physical scan"
    );

    // 6. Verify logical deletion
    let scan_after = tenant_storage
        .scan_prefix(col_prefix.as_bytes())
        .await?;
    assert!(
        scan_after.is_empty(),
        "Post-commit logical scan in active state must return empty"
    );

    Ok(())
}
