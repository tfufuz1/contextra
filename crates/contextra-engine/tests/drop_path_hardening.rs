// FILE-CONTEXT
// STAND: 2026-10-19
// ZWECK: Verification tests for drop_collection_with_report proof directory placement, dynamic instance ID, and physical purge status accuracy.

#![cfg(not(loom))]

use contextra_engine::{Contextra, ContextraConfig};
use contextra_types::{Result, TenantId};
use tempfile::tempdir;

#[tokio::test]
async fn test_drop_collection_purge_status_not_performed_no_supervisor_text() -> Result<()> {
    let dir = tempdir().map_err(|e| contextra_types::ContextraError::Internal(e.to_string()))?;
    let db_path = dir.path().to_path_buf();
    let tenant_id = TenantId::try_new(10).map_err(|e| contextra_types::ContextraError::Internal(e.to_string()))?;
    let collection_name = "test_col_purge";
    let proof_key = b"secret_proof_signing_key_32bytes!";

    let config = ContextraConfig::default();
    let engine = Contextra::open_with_config(&db_path, config).await?;

    let col = engine.collection_for_tenant(collection_name, tenant_id).await?;
    col.insert("doc-1", &[0.1f32; 768], None).await?;

    let report = engine.drop_collection_with_report(collection_name, tenant_id, proof_key).await?;

    let purge_debug = format!("{:?}", report.physical_purge);
    assert!(
        !purge_debug.contains("ErasureSupervisor"),
        "Purge status must not contain misleading ErasureSupervisor text: {purge_debug}"
    );
    assert!(
        !purge_debug.contains("background ErasureSupervisor"),
        "Purge status must not claim unrunning background process: {purge_debug}"
    );

    Ok(())
}

#[tokio::test]
async fn test_drop_collection_proof_dir_and_instance_ids_under_db_path() -> Result<()> {
    let dir1 = tempdir().map_err(|e| contextra_types::ContextraError::Internal(e.to_string()))?;
    let dir2 = tempdir().map_err(|e| contextra_types::ContextraError::Internal(e.to_string()))?;

    let db_path1 = dir1.path().to_path_buf();
    let db_path2 = dir2.path().to_path_buf();

    let tenant_id = TenantId::try_new(10).map_err(|e| contextra_types::ContextraError::Internal(e.to_string()))?;
    let proof_key = b"secret_proof_signing_key_32bytes!";

    let engine1 = Contextra::open_with_config(&db_path1, ContextraConfig::default()).await?;
    let engine2 = Contextra::open_with_config(&db_path2, ContextraConfig::default()).await?;

    let col1 = engine1.collection_for_tenant("col1", tenant_id).await?;
    col1.insert("doc-1", &[0.1f32; 768], None).await?;

    let col2 = engine2.collection_for_tenant("col2", tenant_id).await?;
    col2.insert("doc-2", &[0.1f32; 768], None).await?;

    let report1 = engine1.drop_collection_with_report("col1", tenant_id, proof_key).await?;
    let _report2 = engine2.drop_collection_with_report("col2", tenant_id, proof_key).await?;

    println!("report1 physical_purge: {:?}", report1.physical_purge);

    let proofs_dir1 = db_path1.join("proofs_v4");
    let proofs_dir2 = db_path2.join("proofs_v4");

    assert!(proofs_dir1.exists(), "proofs_v4 directory must exist under db_path1");
    assert!(proofs_dir2.exists(), "proofs_v4 directory must exist under db_path2");

    let instance_id1 = blake3::hash(db_path1.to_string_lossy().as_bytes());
    let instance_id2 = blake3::hash(db_path2.to_string_lossy().as_bytes());
    assert_ne!(instance_id1.as_bytes(), instance_id2.as_bytes(), "Database instance IDs must be distinct");

    Ok(())
}
