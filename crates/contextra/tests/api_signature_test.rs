//! # API-Signatur-Test für Contextra (SPEC §7, D-66, D-89, T-API-1)
//!
//! Verifiziert, dass alle normativen Typen und Methoden aus SPEC Kapitel 7
//! mit den exakten Signaturen vorhanden sind und das SemVer-Versprechen eingehalten wird.

use contextra::{
    system_defaults, Collection, CollectionOptions, Contextra, DocHandle, DocKey, DocRef, Document,
    EngineBuilder, EraseMode, EraseReceipt, EraseRequest, EraseTarget, ErasureId, ErasureStatus,
    Error, Filter, HealthReport, Hit, Missing, RestrictedAccess, RestrictedPurpose,
    RetentionAttestation, RetentionCode, ScanRange, SearchPage, Secret, Set, SignedReport,
    SnapshotId, Supersede, Tenant, TenantContext, TenantRef, Timing, UpdateOptions, UpdateReceipt,
};
use std::time::Duration;

/// SPEC: T-API-1
#[test]
fn test_api_types_and_traits() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<Contextra>();
    assert_send_sync::<Tenant>();
    assert_send_sync::<Collection>();
    assert_send_sync::<Document>();
    assert_send_sync::<Hit>();
    assert_send_sync::<Filter>();
    assert_send_sync::<SearchPage>();
    assert_send_sync::<SignedReport>();
    assert_send_sync::<HealthReport>();
    assert_send_sync::<Error>();

    let builder: EngineBuilder<Missing, Missing, Missing, Missing> = Contextra::builder();
    let secret = Secret::new(b"sehr-geheime-passphrase-mindestens-12".to_vec());
    let rt = tokio::runtime::Handle::current();

    let _configured: EngineBuilder<Set, Set, Set, Set> = builder
        .data_dir("./target/test_data_dir")
        .passphrase(secret)
        .runtime(rt)
        .backup_retention(Duration::from_secs(365 * 86400))
        .dimension(768)
        .erase_sla(Duration::from_secs(86400));

    let _defaults = system_defaults();
}

/// SPEC: T-API-1
#[test]
fn test_facade_method_signatures_exist() {
    // Compile-time assertion of all method signatures from SPEC §7.1 and §7.2
    fn _check_signatures(
        ctx: &Contextra,
        tenant: &Tenant,
        coll: &Collection,
        req: EraseRequest,
        doc_ref: DocRef,
        doc: Document,
        restricted_acc: &RestrictedAccess,
    ) {
        let _ = async {
            let _ = ctx.tenant(TenantContext::try_new("t").unwrap()).await;
            let _ = ctx.erase(req).await;
            let _ = ctx.erase_status(ErasureId::new([0u8; 16])).await;
            let _ = ctx.report(ErasureId::new([0u8; 16])).await;
            let _ = ctx
                .lift_restriction(
                    ErasureId::new([0u8; 16]),
                    RetentionAttestation {
                        code: RetentionCode::NoObligation,
                        attested_by: contextra_types::refs::OperatorRef::try_new(b"op").unwrap(),
                        attested_at: contextra_types::ids::Timestamp::now(),
                    },
                )
                .await;
            let _ = ctx
                .acknowledge_ledger_reset(contextra::OperatorAttestation {
                    operator: contextra_types::refs::OperatorRef::try_new(b"op").unwrap(),
                    attested_at: contextra_types::ids::Timestamp::now(),
                    reason: "test".to_string(),
                })
                .await;
            let sec = Secret::new(vec![0u8; 16]);
            let _ = ctx.change_passphrase(&sec, &sec).await;
            let _health: HealthReport = ctx.health();

            let _coll = tenant
                .collection("test", CollectionOptions::default())
                .await;

            let _ = coll.insert(doc_ref.clone(), doc.clone()).await;
            let _ = coll.update(doc_ref.clone(), doc.clone()).await;
            let _ = coll.get(doc_ref.clone(), None).await;
            let _ = coll.soft_delete(doc_ref.clone()).await;
            let _ = coll.scan(ScanRange::All).await;
            let _ = coll.search_text("q", 10, None).await;
            let _ = coll.search_vector(vec![0.0; 768], 10, None).await;
            let _ = coll.search_hybrid("q", vec![0.0; 768], 10, None).await;
            let _ = coll.load(DocHandle::from_raw([0u8; 32])).await;
            let _ = coll.snapshot().await;
            let _ = coll
                .update_with(
                    doc_ref.clone(),
                    doc.clone(),
                    UpdateOptions {
                        supersede: Supersede::Keep,
                    },
                )
                .await;
            let _ = coll
                .replace_erasing(
                    doc_ref.clone(),
                    doc_ref.clone(),
                    doc.clone(),
                    RetentionAttestation {
                        code: RetentionCode::NoObligation,
                        attested_by: contextra_types::refs::OperatorRef::try_new(b"op").unwrap(),
                        attested_at: contextra_types::ids::Timestamp::now(),
                    },
                )
                .await;
            let _ = coll.get_restricted(doc_ref.clone(), restricted_acc).await;
            let _ = coll.scan_restricted(ScanRange::All, restricted_acc).await;
        };
    }
}

/// SPEC: T-API-2
#[test]
fn test_retention_attestation_code_variants() {
    let codes = [
        RetentionCode::NoObligation,
        RetentionCode::ObligationExpired,
        RetentionCode::ObligationActive,
        RetentionCode::LegalClaimsPending,
        RetentionCode::DataSubjectRequestedRestriction,
        RetentionCode::OperatorOverride,
    ];
    for code in codes {
        let att = RetentionAttestation {
            code,
            attested_by: contextra_types::refs::OperatorRef::try_new(b"op-1").expect("op"),
            attested_at: contextra_types::ids::Timestamp::now(),
        };
        assert_eq!(att.code, code);
    }
}
