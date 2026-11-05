//! Tests for Honest Facade (Auftrag J2)
//! Verifies that no stubs return silent success (Ok), and validates strict UTF-8 handling.

use std::time::Duration;
use tokio::runtime::Handle;

use contextra::error::Error;
use contextra::{
    CollectionOptions, Contextra, DocKey, DocRef, Document, EraseMode, EraseRequest, EraseTarget,
    ErasureId, OperatorAttestation, OperatorRef, RestrictedAccess, RestrictedPurpose,
    RetentionAttestation, RetentionCode, ScanRange, Secret, TenantContext, TenantRef, Timing,
    UpdateOptions,
};

fn dummy_tenant_context() -> TenantContext {
    let tenant_bytes = [1u8; 16];
    TenantContext::new(contextra_types::TenantId::from_bytes(tenant_bytes))
}

fn dummy_erase_request() -> EraseRequest {
    EraseRequest {
        target: EraseTarget::Tenant {
            tenant: TenantRef::new("tenant_a").expect("tenant"),
        },
        timing: Timing::Now,
        retention: RetentionAttestation {
            code: RetentionCode::NoObligation,
            attested_by: OperatorRef::new("op1").expect("operator"),
            attested_at: contextra_types::ids::Timestamp::from_secs(1_700_000_000),
        },
        mode: EraseMode::Erase,
    }
}

/// Helper to spin up a Contextra facade instance for testing.
async fn setup_contextra(dir: &std::path::Path) -> Contextra {
    Contextra::builder()
        .data_dir(dir)
        .passphrase(Secret::new(b"a_very_secure_passphrase_123456"))
        .runtime(Handle::current())
        .backup_retention(Duration::from_secs(86400 * 30))
        .dimension(768)
        .open()
        .await
        .expect("open contextra")
}

#[tokio::test]
async fn test_stubs_return_not_implemented() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let ctx = setup_contextra(tmp.path()).await;

    // 1. erase
    let erase_res = ctx.erase(dummy_erase_request()).await;
    match erase_res {
        Err(Error::NotImplemented(method)) => assert_eq!(method, "erase"),
        other => panic!("expected Err(NotImplemented(\"erase\")), got: {other:?}"),
    }

    // 2. erase_status
    let status_res = ctx.erase_status(ErasureId::new([1u8; 16])).await;
    match status_res {
        Err(Error::NotImplemented(method)) => assert_eq!(method, "erase_status"),
        other => panic!("expected Err(NotImplemented(\"erase_status\")), got: {other:?}"),
    }

    // 3. report
    let report_res = ctx.report(ErasureId::new([1u8; 16])).await;
    match report_res {
        Err(Error::NotImplemented(method)) => assert_eq!(method, "report"),
        other => panic!("expected Err(NotImplemented(\"report\")), got: {other:?}"),
    }

    // 4. lift_restriction
    let att = RetentionAttestation {
        code: RetentionCode::NoObligation,
        attested_by: OperatorRef::new("op1").expect("operator"),
        attested_at: contextra_types::ids::Timestamp::from_secs(1_700_000_000),
    };
    let lift_res = ctx.lift_restriction(ErasureId::new([1u8; 16]), att).await;
    match lift_res {
        Err(Error::NotImplemented(method)) => assert_eq!(method, "lift_restriction"),
        other => panic!("expected Err(NotImplemented(\"lift_restriction\")), got: {other:?}"),
    }

    // 5. acknowledge_ledger_reset
    let op_att = OperatorAttestation {
        operator: OperatorRef::new("op1").expect("op"),
        timestamp: contextra_types::ids::Timestamp::from_secs(1_700_000_000),
        signature: vec![1, 2, 3],
    };
    let ack_res = ctx.acknowledge_ledger_reset(op_att).await;
    match ack_res {
        Err(Error::NotImplemented(method)) => assert_eq!(method, "acknowledge_ledger_reset"),
        other => {
            panic!("expected Err(NotImplemented(\"acknowledge_ledger_reset\")), got: {other:?}")
        }
    }

    // 6. change_passphrase
    let old_sec = Secret::new(b"a_very_secure_passphrase_123456");
    let new_sec = Secret::new(b"a_new_very_secure_passphrase_654321");
    let change_res = ctx.change_passphrase(&old_sec, &new_sec).await;
    match change_res {
        Err(Error::NotImplemented(method)) => assert_eq!(method, "change_passphrase"),
        other => panic!("expected Err(NotImplemented(\"change_passphrase\")), got: {other:?}"),
    }

    // Tenant and Collection operations
    let tenant = ctx.tenant(dummy_tenant_context()).await.expect("tenant");
    let collection = tenant
        .collection("test_col", CollectionOptions::default())
        .await
        .expect("collection");

    // 7. replace_erasing
    let doc_ref1 = DocRef {
        subject: None,
        key: DocKey::new("doc1").expect("key"),
    };
    let doc_ref2 = DocRef {
        subject: None,
        key: DocKey::new("doc2").expect("key"),
    };
    let replace_res = collection
        .replace_erasing(
            doc_ref1,
            doc_ref2,
            Document {
                content: "new text".into(),
                metadata: vec![],
                vector: None,
            },
            RetentionAttestation {
                code: RetentionCode::NoObligation,
                attested_by: OperatorRef::new("op1").expect("op"),
                attested_at: contextra_types::ids::Timestamp::from_secs(1_700_000_000),
            },
        )
        .await;
    match replace_res {
        Err(Error::NotImplemented(method)) => assert_eq!(method, "replace_erasing"),
        other => panic!("expected Err(NotImplemented(\"replace_erasing\")), got: {other:?}"),
    }

    // 8. scan
    let scan_res = collection.scan(ScanRange::All).await;
    match scan_res {
        Err(Error::NotImplemented(method)) => assert_eq!(method, "scan"),
        other => panic!("expected Err(NotImplemented(\"scan\")), got: {other:?}"),
    }

    // 9. search_vector
    let vec_res = collection.search_vector(vec![0.1; 768], 5, None).await;
    match vec_res {
        Err(Error::NotImplemented(method)) => assert_eq!(method, "search_vector"),
        other => panic!("expected Err(NotImplemented(\"search_vector\")), got: {other:?}"),
    }

    // 10. load
    let load_res = collection
        .load(contextra::DocHandle::new(
            contextra_types::ids::ClusterTag::new([0u8; 16]),
            contextra_types::ids::DocTag::new([0u8; 16]),
        ))
        .await;
    match load_res {
        Err(Error::NotImplemented(method)) => assert_eq!(method, "load"),
        other => panic!("expected Err(NotImplemented(\"load\")), got: {other:?}"),
    }

    // 11. update_with
    let doc_ref3 = DocRef {
        subject: None,
        key: DocKey::new("doc3").expect("key"),
    };
    let update_res = collection
        .update_with(
            doc_ref3,
            Document {
                content: "updated content".into(),
                metadata: vec![],
                vector: None,
            },
            UpdateOptions {
                supersede: contextra::Supersede::Keep,
            },
        )
        .await;
    match update_res {
        Err(Error::NotImplemented(method)) => assert_eq!(method, "update_with"),
        other => panic!("expected Err(NotImplemented(\"update_with\")), got: {other:?}"),
    }

    // 12. get_restricted & scan_restricted
    let restricted_access = RestrictedAccess {
        purpose: RestrictedPurpose::LegalClaims,
        attestation: OperatorAttestation {
            operator: OperatorRef::new("op1").expect("op"),
            timestamp: contextra_types::ids::Timestamp::from_secs(1_700_000_000),
            signature: vec![1, 2, 3],
        },
    };
    let doc_ref4 = DocRef {
        subject: None,
        key: DocKey::new("doc4").expect("key"),
    };
    let get_restr_res = collection
        .get_restricted(doc_ref4, &restricted_access)
        .await;
    match get_restr_res {
        Err(Error::NotImplemented(method)) => assert_eq!(method, "get_restricted"),
        other => panic!("expected Err(NotImplemented(\"get_restricted\")), got: {other:?}"),
    }

    let scan_restr_res = collection
        .scan_restricted(ScanRange::All, &restricted_access)
        .await;
    match scan_restr_res {
        Err(Error::NotImplemented(method)) => assert_eq!(method, "scan_restricted"),
        other => panic!("expected Err(NotImplemented(\"scan_restricted\")), got: {other:?}"),
    }
}

#[tokio::test]
async fn test_passphrase_invalid_utf8_rejected() {
    let tmp = tempfile::tempdir().expect("tempdir");

    // Two passphrases with distinct invalid UTF-8 bytes that from_utf8_lossy would replace with U+FFFD
    let p1 = Secret::new(b"valid_prefix_\xFF_more_bytes");
    let p2 = Secret::new(b"valid_prefix_\xFE_more_bytes");

    let b1 = Contextra::builder()
        .data_dir(tmp.path().join("d1"))
        .passphrase(p1)
        .runtime(Handle::current())
        .backup_retention(Duration::from_secs(86400 * 30))
        .dimension(768)
        .open()
        .await;

    let b2 = Contextra::builder()
        .data_dir(tmp.path().join("d2"))
        .passphrase(p2)
        .runtime(Handle::current())
        .backup_retention(Duration::from_secs(86400 * 30))
        .dimension(768)
        .open()
        .await;

    assert!(b1.is_err(), "Invalid UTF-8 passphrase p1 must be rejected");
    assert!(b2.is_err(), "Invalid UTF-8 passphrase p2 must be rejected");
}

#[tokio::test]
async fn test_doc_key_invalid_utf8_rejected() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let ctx = setup_contextra(tmp.path()).await;
    let tenant = ctx.tenant(dummy_tenant_context()).await.expect("tenant");
    let collection = tenant
        .collection("test_col", CollectionOptions::default())
        .await
        .expect("collection");

    // Creating DocKey with invalid UTF-8 bytes
    let key1 = DocKey::from_bytes(b"doc_\xFF_key");
    let key2 = DocKey::from_bytes(b"doc_\xFE_key");

    if let Ok(k) = key1 {
        let res = collection
            .insert(
                DocRef {
                    subject: None,
                    key: k,
                },
                Document::default(),
            )
            .await;
        assert!(res.is_err(), "Invalid UTF-8 DocKey must result in an error");
    }

    if let Ok(k) = key2 {
        let res = collection
            .insert(
                DocRef {
                    subject: None,
                    key: k,
                },
                Document::default(),
            )
            .await;
        assert!(res.is_err(), "Invalid UTF-8 DocKey must result in an error");
    }
}
