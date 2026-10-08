#![cfg(not(loom))]

use contextra_crypto::deletion_proof::DeletionLayer;
use contextra_engine::{Contextra, ContextraConfig};
use contextra_types::TenantId;
use serde_json::json;
use tempfile::TempDir;

/// Test T3 (Layer Coverage in DeletionProof):
///
/// Audit A-03 documents that `drop_collection` claims ONLY `DeletionLayer::LsmMemtable`
/// in its generated `DeletionProof.covered_layers`.
///
/// This test populates a collection with both vector data and text metadata, invokes
/// `drop_collection`, and asserts that `proof.covered_layers` contains strictly
/// `[DeletionLayer::LsmMemtable]`.
#[tokio::test]
async fn test_drop_collection_layer_coverage() -> contextra_types::Result<()> {
    let tmp = TempDir::new().expect("Failed to create temporary directory");
    let config = ContextraConfig {
        dimension: 4,
        consolidation_enabled: false,
        ..Default::default()
    };

    let db = Contextra::open_with_config(tmp.path(), config).await?;
    let tenant_id = TenantId::try_new(777).expect("Valid tenant ID");
    let proof_key = b"secret_proof_key_32bytes_1234567";

    let col = db
        .collection_for_tenant("layer_coverage_col", tenant_id)
        .await?;

    // 1. Populate collection with vector data and text documents
    col.insert(
        "vec_doc_1",
        &[0.1, 0.2, 0.3, 0.4],
        Some(json!({ "text": "Sample text document for LSM and vector index" })),
    )
    .await?;

    col.insert(
        "vec_doc_2",
        &[0.5, 0.6, 0.7, 0.8],
        Some(json!({ "text": "Another text entry" })),
    )
    .await?;

    // 2. Execute drop_collection
    let proof = db
        .drop_collection("layer_coverage_col", tenant_id, proof_key)
        .await?;

    // 3. Verify that covered_layers contains strictly [DeletionLayer::LsmMemtable]
    assert_eq!(
        proof.covered_layers.len(),
        1,
        "covered_layers must contain exactly 1 layer"
    );
    assert_eq!(
        proof.covered_layers[0].layer,
        DeletionLayer::LsmMemtable,
        "covered_layers[0] must be DeletionLayer::LsmMemtable as specified in Audit A-03"
    );

    println!(
        "T3 RESULT: Confirmed proof.covered_layers == [DeletionLayer::LsmMemtable] (matches Audit A-03 prediction)."
    );

    db.close().await?;
    Ok(())
}
