#![forbid(unsafe_code)]

use contextra_crypto::deletion_proof::claim_text::WitAssurance;
use contextra_engine::collection::crud::DocEraseOptions;
use contextra_engine::Contextra;
use contextra_types::ContextraError;
use tempfile::TempDir;

#[tokio::test]
async fn test_hotfix_f01_no_witness_without_real_scan() {
    let tmp = TempDir::new().expect("temp dir");
    let db = Contextra::open(tmp.path()).await.expect("open db");
    let col = db.collection("default").await.expect("col");

    let doc_id = "doc_f01_test";
    let embedding = vec![0.1f32; col.dimension()];
    let metadata = serde_json::json!({ "text": "Hotfix F01 test data" });

    col.insert(doc_id, &embedding, Some(metadata))
        .await
        .expect("insert");

    // 1. Assurance == S0 must NOT return fabricated witnesses (witnesses MUST be empty)
    let opts_s0 = DocEraseOptions {
        options: contextra_engine::deletion::forget_options::ForgetOptions {
            assurance: WitAssurance::S0,
            ..Default::default()
        },
    };
    let erasure = col
        .erase_document(doc_id, &opts_s0)
        .await
        .expect("erase document with S0 should succeed");

    assert!(
        erasure.witnesses.is_empty(),
        "erase_document with Assurance S0 must return empty witnesses (no fabrication allowed), got {} witnesses",
        erasure.witnesses.len()
    );

    db.close().await.unwrap();
}

#[tokio::test]
async fn test_hotfix_f01_assurance_above_s0_returns_capability_unsupported() {
    let tmp = TempDir::new().expect("temp dir");
    let db = Contextra::open(tmp.path()).await.expect("open db");
    let col = db.collection("default").await.expect("col");

    let doc_id = "doc_f01_s1";
    let embedding = vec![0.1f32; col.dimension()];
    col.insert(doc_id, &embedding, None).await.expect("insert");

    // 2. Assurance > S0 must return CapabilityUnsupported until real scan infrastructure is wired in M5
    let assurances = [
        WitAssurance::S1,
        WitAssurance::S2,
        WitAssurance::S3,
        WitAssurance::S4,
    ];

    for assurance in assurances {
        let opts = DocEraseOptions {
            options: contextra_engine::deletion::forget_options::ForgetOptions {
                assurance,
                ..Default::default()
            },
        };
        let err = col
            .erase_document(doc_id, &opts)
            .await
            .expect_err("erase with assurance > S0 must fail with CapabilityUnsupported");

        match err {
            ContextraError::CapabilityUnsupported { capability, .. } => {
                assert!(
                    capability.contains("erasure") || capability.contains("scan"),
                    "Expected capability name related to erasure/scan, got: '{capability}'"
                );
            }
            other => panic!("Expected CapabilityUnsupported, got: {other:?}"),
        }
    }

    db.close().await.unwrap();
}
