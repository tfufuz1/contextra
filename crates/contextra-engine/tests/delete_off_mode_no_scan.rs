#![forbid(unsafe_code)]

use contextra_engine::collection::crud::DocEraseOptions;
use contextra_engine::deletion::forget_options::ForgetOptions;
use contextra_engine::Contextra;
use contextra_store::tenant_codec::TenantScopedStorage;
use contextra_store::{LsmConfig, LsmStorage};
use contextra_testkit::erasure_harness::tenant_matrix::TenantMatrix;
use tempfile::TempDir;

#[tokio::test]
async fn test_off_mode_no_footprint_scan_counter() {
    let tmp = TempDir::new().expect("temp dir");
    let db = Contextra::open(tmp.path()).await.expect("open db");
    let col = db.collection("default").await.expect("col");

    let doc_id = "doc_off_mode_test";
    let embedding = vec![0.1f32; col.dimension()];
    col.insert(doc_id, &embedding, None)
        .await
        .expect("insert");

    let opts_off = DocEraseOptions {
        options: ForgetOptions {
            assurance: contextra_crypto::deletion_proof::claim_text::WitAssurance::S0,
            purge: contextra_engine::deletion::forget_options::PurgeMode::None,
            ..Default::default()
        },
    };

    let erasure = col
        .erase_document(doc_id, &opts_off)
        .await
        .expect("off-mode erase should succeed");

    assert_eq!(
        erasure.footprint_hash,
        [0u8; 32],
        "Footprint hash must be zero in Off mode (no footprint calculation)"
    );
    assert!(
        erasure.prefixes.is_empty(),
        "Prefixes must be empty in Off mode"
    );
    assert_eq!(
        erasure.keys.len(),
        2,
        "Explicit keys must strictly contain user_key and doc_key in Off mode"
    );

    db.close().await.unwrap();
}

#[tokio::test]
async fn test_k1_14_delete_under_lsm_and_tenant_scoped_storage() {
    TenantMatrix::run(|harness| async move {
        let storage = harness.storage();
        let col = harness.collection();

        let doc_id = "doc_tenant_test";
        let embedding = vec![0.2f32; col.dimension()];
        col.insert(doc_id, &embedding, None)
            .await
            .expect("insert");

        let res = col.delete(doc_id).await;
        assert!(
            res.is_ok(),
            "Standard delete under TenantMatrix (LsmStorage and TenantScopedStorage) must succeed without CapabilityUnsupported: {:?}",
            res.err()
        );
    })
    .await;
}
