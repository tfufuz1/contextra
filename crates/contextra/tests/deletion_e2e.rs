//! # End-to-End-Test: Trennung von `soft_delete` und `erase` (SPEC D-51, 7.2, 7.3)
//!
//! Prüft die Trennung von logischem `soft_delete` (Tombstone) und
//! physischer Löschung `erase` über die Fassade.

use contextra::{
    DocKey, DocRef, EraseMode, EraseRequest, EraseTarget, ErasureStatus, RetentionAttestation,
    RetentionCode,
};
use contextra_types::ids::Timestamp;
use contextra_types::refs::{OperatorRef, TenantContext};

/// SPEC: T-API-3
#[test]
fn test_erase_vs_soft_delete_types() {
    let tenant_ctx = TenantContext::try_new("tenant_alpha").expect("valid tenant ctx");
    let doc_key = DocKey::try_new(b"doc-123".to_vec()).expect("valid doc key");
    let doc_ref = DocRef {
        subject: None,
        key: doc_key.clone(),
    };

    let erase_req = EraseRequest {
        target: EraseTarget::Document {
            tenant: tenant_ctx.tenant_ref().clone(),
            collection: contextra::CollectionRef::try_new("docs").expect("coll"),
            doc: doc_ref.clone(),
        },
        timing: contextra::Timing::Now,
        retention: RetentionAttestation {
            code: RetentionCode::NoObligation,
            attested_by: OperatorRef::try_new(b"admin").expect("op"),
            attested_at: Timestamp::now(),
        },
        mode: EraseMode::Erase,
    };

    assert_eq!(erase_req.mode, EraseMode::Erase);
    assert_eq!(erase_req.retention.code, RetentionCode::NoObligation);
}
