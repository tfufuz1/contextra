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

    let builder: EngineBuilder<Missing, Missing, Missing> = Contextra::builder();
    let secret = Secret::new(b"sehr-geheime-passphrase-mindestens-12".to_vec());
    let rt = tokio::runtime::Handle::current();

    let _configured: EngineBuilder<Set, Set, Set> = builder
        .data_dir("./target/test_data_dir")
        .passphrase(secret)
        .runtime(rt)
        .backup_retention(Duration::from_secs(365 * 86400))
        .erase_sla(Duration::from_secs(86400));

    let _defaults = system_defaults();
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
