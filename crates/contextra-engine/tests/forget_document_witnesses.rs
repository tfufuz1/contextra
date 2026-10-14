#![forbid(unsafe_code)]

use contextra_crypto::deletion_proof::claim_text::WitAssurance;
use contextra_engine::collection::crud::DocEraseOptions;
use contextra_engine::deletion::forget_options::ForgetOptions;
use contextra_engine::Contextra;
use contextra_types::{ContextraError, DocId};
use tempfile::TempDir;

#[tokio::test]
async fn test_witnesses_not_fabricated_on_erase_document() {
    let tmp = TempDir::new().expect("temp dir");
    let db = Contextra::open(tmp.path()).await.expect("open db");

    let col = db.collection("default").await.expect("col");

    let doc_id = "doc_wit_1";
    let embedding = vec![0.1f32; 768];
    let metadata = serde_json::json!({
        "text": "witness test sample content"
    });

    col.insert(doc_id, &embedding, Some(metadata))
        .await
        .expect("insert");

    let opts = DocEraseOptions {
        options: ForgetOptions {
            assurance: WitAssurance::S0,
            ..ForgetOptions::default()
        },
    };
    let erasure = col.erase_document(doc_id, &opts).await.expect("erase");

    assert!(
        erasure.witnesses.is_empty(),
        "REG-14: erase_document must NOT fabricate witnesses without genuine scan"
    );

    db.close().await.unwrap();
}

#[tokio::test]
async fn test_assurance_above_s0_unsupported() {
    let tmp = TempDir::new().expect("temp dir");
    let db = Contextra::open(tmp.path()).await.expect("open db");
    let col = db.collection("default").await.expect("col");

    let opts = DocEraseOptions {
        options: ForgetOptions {
            assurance: WitAssurance::S1,
            ..ForgetOptions::default()
        },
    };
    let res = col.erase_document("doc_id", &opts).await;
    assert!(
        matches!(res, Err(ContextraError::CapabilityUnsupported { .. })),
        "Assurance > S0 must return CapabilityUnsupported: {:?}",
        res
    );

    db.close().await.unwrap();
}

#[tokio::test]
async fn test_fault_injection_unremoved_edge_causes_witness_failure() {
    let doc_id = DocId::from_key("doc_fault_1").unwrap();
    let scope_hash = [1u8; 32];
    let as_of_tx = contextra_types::TxId(10);

    // Injected remaining edge count > 0 MUST be rejected by witness invariants
    let res = contextra_engine::deletion::relations_witness::issue_relations_witness(
        doc_id,
        100, // edges_scanned
        scope_hash,
        as_of_tx,
        Some(1), // matching_edges_found_override = 1 (fault injected)
    );

    assert!(
        res.is_err(),
        "Relations witness issuance MUST fail if matching edges remain"
    );
}
