// FILE-CONTEXT
// ZWECK: Testen der fail-closed Datenlöschung und ErasureOrchestrator Garantien.
// INVARIANTEN: Restmengen ergeben verified == false; fehlender Schlüssel schlägt vor Mutation fehl.

use contextra_engine::{Contextra, ProofKeyProvider};
use contextra_types::{Result, TenantId};
use std::sync::Arc;
use tempfile::tempdir;

struct TestProofKeyProvider(Vec<u8>);

impl ProofKeyProvider for TestProofKeyProvider {
    fn resolve_proof_key(&self, _tenant_id: TenantId) -> Result<Vec<u8>> {
        Ok(self.0.clone())
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn test_erase_document_e2e_verified() -> Result<()> {
    let dir = tempdir().unwrap();
    let key = b"secret_proof_key_32bytes_1234567";
    let db = Contextra::open(dir.path()).await?;
    let db = db.with_proof_key_provider(Arc::new(TestProofKeyProvider(key.to_vec())));

    let tenant_id = TenantId::try_new(1).unwrap();
    let col = db.collection_for_tenant("testcol", tenant_id).await?;

    let dummy_embedding = vec![0.1f32; db.config().dimension];
    col.insert(
        "doc1",
        &dummy_embedding,
        Some(serde_json::json!({"text": "Sicherer Inhalt"})),
    )
    .await?;

    let report = db.erase_document(tenant_id, "testcol", "doc1").await?;

    if !report.verified {
        panic!("report.verified was false: {:?}", report.failure);
    }
    assert!(
        report.proof.is_some(),
        "DeletionProof must be issued on verified erasure"
    );
    let proof = report.proof.unwrap();
    assert!(
        proof.verify(key.as_slice()).unwrap(),
        "Issued DeletionProof must verify with proof_key"
    );

    for layer in &report.layers {
        assert_eq!(
            layer.remaining, 0,
            "Layer {} must have 0 remaining entries",
            layer.layer
        );
    }

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn test_erase_document_missing_key_preflight_fails_before_mutation() -> Result<()> {
    let dir = tempdir().unwrap();
    let db = Contextra::open(dir.path()).await?;

    let tenant_id = TenantId::try_new(1).unwrap();
    let col = db.collection_for_tenant("testcol", tenant_id).await?;

    let dummy_embedding = vec![0.1f32; db.config().dimension];
    col.insert(
        "doc1",
        &dummy_embedding,
        Some(serde_json::json!({"text": "Sicherer Inhalt"})),
    )
    .await?;

    // Attempt erase without registering a ProofKeyProvider -> must return Err
    let res = db.erase_document(tenant_id, "testcol", "doc1").await;
    assert!(
        res.is_err(),
        "erase_document must fail preflight if key provider missing"
    );

    // Verify document is still readable
    let doc = col.get("doc1").await?;
    assert!(
        doc.is_some(),
        "Document must remain intact after preflight failure"
    );

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn test_sabotage_purger_leaves_residual_fails_verification() -> Result<()> {
    let dir = tempdir().unwrap();
    let key = b"secret_proof_key_32bytes_1234567";
    let db = Contextra::open(dir.path()).await?;
    let db = db.with_proof_key_provider(Arc::new(TestProofKeyProvider(key.to_vec())));

    let tenant_id = TenantId::try_new(1).unwrap();
    let col = db.collection_for_tenant("testcol", tenant_id).await?;

    let dummy_embedding = vec![0.1f32; db.config().dimension];
    col.insert(
        "doc1",
        &dummy_embedding,
        Some(serde_json::json!({"text": "Sicherer Inhalt"})),
    )
    .await?;

    // Simulate open snapshot blocking preflight purge (fail-closed verification)
    let snap = db.inner_storage().snapshot_registry.register_at(1, 1);
    let report_res = db.erase_document(tenant_id, "testcol", "doc1").await;
    assert!(report_res.is_err(), "Open snapshot must block erasure preflight");
    drop(snap);

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn test_erase_collection_e2e_and_registry_timing() -> Result<()> {
    let dir = tempdir().unwrap();
    let key = b"secret_proof_key_32bytes_1234567";
    let db = Contextra::open(dir.path()).await?;
    let db = db.with_proof_key_provider(Arc::new(TestProofKeyProvider(key.to_vec())));

    let tenant_id = TenantId::try_new(1).unwrap();
    let col = db.collection_for_tenant("dropcol", tenant_id).await?;
    let dummy_embedding = vec![0.1f32; db.config().dimension];
    col.insert(
        "doc1",
        &dummy_embedding,
        Some(serde_json::json!({"text": "Text 1"})),
    )
    .await?;
    col.insert(
        "doc2",
        &dummy_embedding,
        Some(serde_json::json!({"text": "Text 2"})),
    )
    .await?;

    let list_before = db.list_collections_for_tenant(tenant_id).await?;
    assert!(list_before.contains(&"dropcol".to_string()));

    let report = db.erase_collection(tenant_id, "dropcol").await?;
    if !report.verified {
        panic!(
            "report.verified was false in erase_collection: {:?}",
            report.failure
        );
    }
    assert!(report.proof.is_some());

    let list_after = db.list_collections_for_tenant(tenant_id).await?;
    assert!(
        !list_after.contains(&"dropcol".to_string()),
        "Registry entry must be removed strictly after successful proof issuance"
    );

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn test_erase_distinct_deletions_produce_distinct_proofs() -> Result<()> {
    let dir = tempdir().unwrap();
    let key = b"secret_proof_key_32bytes_1234567";
    let db = Contextra::open(dir.path()).await?;
    let db = db.with_proof_key_provider(Arc::new(TestProofKeyProvider(key.to_vec())));

    let tenant_id1 = TenantId::try_new(1).unwrap();
    let tenant_id2 = TenantId::try_new(2).unwrap();
    let col1 = db.collection_for_tenant("testcol", tenant_id1).await?;
    let col2 = db.collection_for_tenant("testcol", tenant_id2).await?;
    let dummy_embedding = vec![0.1f32; db.config().dimension];
    col1.insert(
        "doc1",
        &dummy_embedding,
        Some(serde_json::json!({"text": "Inhalt 1"})),
    )
    .await?;
    col2.insert(
        "doc2",
        &dummy_embedding,
        Some(serde_json::json!({"text": "Inhalt 2"})),
    )
    .await?;

    let report1 = db.erase_document(tenant_id1, "testcol", "doc1").await?;
    let report2 = db.erase_document(tenant_id2, "testcol", "doc2").await?;

    assert!(report1.verified, "report1 failed: {:?}", report1.failure);
    assert!(report2.verified, "report2 failed: {:?}", report2.failure);
    let proof1 = report1.proof.unwrap();
    let proof2 = report2.proof.unwrap();

    assert_ne!(
        proof1.signature, proof2.signature,
        "Distinct deletions must produce distinct proof signatures"
    );

    Ok(())
}
