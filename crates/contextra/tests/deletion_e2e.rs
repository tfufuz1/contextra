//! # Integrationstest: `deletion_e2e` (SPEC §7)
//!
//! Prüft die Trennung von logischem `soft_delete` (Tombstone) und
//! physischer Löschung `erase` über die Fassade.

use contextra::{
    CollectionOptions, Contextra, DocKey, DocRef, Document, EraseMode, EraseRequest,
    EraseTarget, RetentionAttestation, RetentionCode, Secret, TenantContext, TenantRef, Timing,
};
use tempfile::TempDir;

#[tokio::test]
async fn test_erase_vs_soft_delete_facade() {
    let tmp = TempDir::new().expect("Failed to create tempdir");
    let secret = Secret::new(b"sehr-sichere-passphrase-12345".to_vec());
    let rt = tokio::runtime::Handle::current();

    let builder = Contextra::builder()
        .data_dir(tmp.path())
        .passphrase(secret)
        .runtime(rt);

    let db = builder.open().await.expect("Failed to open Contextra instance");

    let tenant_ctx = TenantContext::try_new(100).expect("Valid tenant id");
    let tenant = db.tenant(tenant_ctx).await.expect("Tenant handle");

    let coll_opts = CollectionOptions::default();
    let coll = tenant.collection("test_col", coll_opts).await.expect("Collection handle");

    let doc_key = DocKey::try_new(b"doc-1".to_vec()).expect("Valid doc key");
    let doc_ref = DocRef {
        subject: None,
        key: doc_key,
    };

    let doc = Document {
        content: "Sensible Daten für Löschtest".to_string(),
        metadata: Vec::new(),
        vector: None,
    };

    // 1. Einfügen
    coll.insert(doc_ref.clone(), doc).await.expect("Insert succeeded");

    // 2. Soft delete (Tombstone)
    coll.soft_delete(doc_ref.clone()).await.expect("Soft delete succeeded");

    // 3. Echtes erase über Contextra-Handle (SPEC 7.3)
    let erase_req = EraseRequest {
        target: EraseTarget::Document {
            tenant: TenantRef::try_new(b"tenant-100".to_vec()).expect("TenantRef"),
            collection: contextra::CollectionRef::try_new(b"test_col".to_vec()).expect("CollectionRef"),
            doc: doc_ref,
        },
        timing: Timing::Now,
        retention: RetentionAttestation {
            code: RetentionCode::NoObligation,
            attested_by: contextra::OperatorRef::try_new(b"operator-1".to_vec()).expect("OperatorRef"),
            attested_at: contextra::Timestamp::now(),
        },
        mode: EraseMode::Erase,
    };

    let receipt = db.erase(erase_req).await.expect("Erase initiated");
    assert_eq!(receipt.status, contextra::ErasureStatus::Gated);

    db.close().await.expect("Close succeeded");
}
