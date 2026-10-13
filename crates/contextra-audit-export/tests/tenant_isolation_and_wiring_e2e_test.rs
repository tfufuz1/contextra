use contextra_audit_export::{
    testkit::InMemoryProcessingRegisterSource, wiring::AuditExportWiring, DeletionProofSummary,
    EgressEventSummary, ProcessingRegisterEntry,
};
use contextra_ports::clock::ManualClock;
use contextra_types::TenantId;
use std::sync::Arc;

#[test]
fn test_veto_f10_tenant_isolation_and_end_to_end_export_wiring() {
    let tenant1 = TenantId::try_new(100).unwrap();
    let tenant2 = TenantId::try_new(200).unwrap();

    let mut source = InMemoryProcessingRegisterSource::new();

    let entry_t1 = ProcessingRegisterEntry {
        tenant_id: tenant1,
        processing_purpose: "Vektorsuche Tenant 100".to_string(),
        data_categories: vec!["Vertragsdaten T100".to_string()],
        legal_basis: "Art. 6 Abs. 1 lit. b DSGVO".to_string(),
        egress_events: vec![EgressEventSummary {
            event_id: "egress-t100-01".to_string(),
            destination: "https://llm.provider.example/v1".to_string(),
            timestamp_nanos: 1000,
            detail: "Query T100".to_string(),
        }],
        deletion_proofs: vec![DeletionProofSummary {
            proof_id: "proof-t100-01".to_string(),
            scope: "doc_100".to_string(),
            timestamp_nanos: 1005,
            verified: true,
        }],
        generated_at: 1010,
    };

    let entry_t2 = ProcessingRegisterEntry {
        tenant_id: tenant2,
        processing_purpose: "Graphsuche Tenant 200".to_string(),
        data_categories: vec!["Gesundheitsdaten T200".to_string()],
        legal_basis: "Art. 9 Abs. 2 lit. a DSGVO".to_string(),
        egress_events: vec![EgressEventSummary {
            event_id: "egress-t200-01".to_string(),
            destination: "https://llm.provider.example/v2".to_string(),
            timestamp_nanos: 2000,
            detail: "Query T200".to_string(),
        }],
        deletion_proofs: vec![DeletionProofSummary {
            proof_id: "proof-t200-01".to_string(),
            scope: "doc_200".to_string(),
            timestamp_nanos: 2005,
            verified: true,
        }],
        generated_at: 2010,
    };

    source.add_entry(entry_t1);
    source.add_entry(entry_t2);

    let clock = Arc::new(ManualClock::new(9_999_999));
    let wiring = AuditExportWiring::build(source, clock).expect("AuditExportWiring build");

    // 1. Export JSON for Tenant 100
    let json_t1 = wiring.export_json(tenant1).expect("export json t1");
    assert!(json_t1.contains("Vektorsuche Tenant 100"));
    assert!(json_t1.contains("Vertragsdaten T100"));
    assert!(!json_t1.contains("Tenant 200"));
    assert!(!json_t1.contains("Gesundheitsdaten T200"));

    // 2. Export JSON for Tenant 200
    let json_t2 = wiring.export_json(tenant2).expect("export json t2");
    assert!(json_t2.contains("Graphsuche Tenant 200"));
    assert!(json_t2.contains("Gesundheitsdaten T200"));
    assert!(!json_t2.contains("Tenant 100"));
    assert!(!json_t2.contains("Vertragsdaten T100"));

    // 3. Export Markdown for Tenant 100
    let md_t1 = wiring.export_markdown(tenant1).expect("export markdown t1");
    assert!(md_t1.contains("Vektorsuche Tenant 100"));
    assert!(!md_t1.contains("Graphsuche Tenant 200"));

    let lines_t1: Vec<&str> = md_t1.trim().lines().map(|l| l.trim()).collect();
    assert!(lines_t1.first().unwrap().contains("Vorlage, keine Rechtsberatung"));
    assert!(lines_t1.last().unwrap().contains("Vorlage, keine Rechtsberatung"));

    // 4. Test entry creation via Clock port
    let created_entry = wiring.create_entry(
        tenant1,
        "Zweck via Wiring",
        vec!["Kat 1".to_string()],
        "Art. 6 DSGVO",
        vec![],
        vec![],
    );
    assert_eq!(created_entry.generated_at, 9_999_999);
}
