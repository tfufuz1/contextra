use contextra_audit_export::{
    AuditRegisterExporter, DeletionProofSummary, EgressEventSummary, ProcessingRegisterEntry,
};
use contextra_types::TenantId;

#[test]
fn test_audit_register_exporter_wiring() -> Result<(), Box<dyn std::error::Error>> {
    let tenant_id = TenantId::try_new(777)?;

    // Test with_sample_data_for wiring
    let mut exporter = AuditRegisterExporter::with_sample_data_for(tenant_id);

    // Test add_entry wiring
    let extra_entry = ProcessingRegisterEntry {
        tenant_id,
        processing_purpose: "Extra Zweck".to_string(),
        data_categories: vec!["Protokoll".to_string()],
        legal_basis: "Art. 6 Abs. 1 lit. f DSGVO".to_string(),
        egress_events: vec![EgressEventSummary {
            event_id: "evt-extra".to_string(),
            destination: "https://extra.destination".to_string(),
            timestamp_nanos: 1_700_000_200_000_000_000,
            detail: "Extra egress".to_string(),
        }],
        deletion_proofs: vec![DeletionProofSummary {
            proof_id: "proof-extra".to_string(),
            scope: "scope-extra".to_string(),
            timestamp_nanos: 1_700_000_250_000_000_000,
            verified: true,
        }],
        generated_at: 1_700_000_300_000_000_000,
    };

    exporter.add_entry(extra_entry);

    // Test export_json wiring (which invokes render_register_json)
    let json_output = exporter.export_json(tenant_id)?;
    assert!(json_output.contains("Kontextsensitive Vektorsuche"));
    assert!(json_output.contains("Extra Zweck"));
    assert!(json_output.contains("evt-extra"));

    let deserialized: Vec<ProcessingRegisterEntry> = serde_json::from_str(&json_output)?;
    assert_eq!(deserialized.len(), 3);

    // Test export_markdown wiring
    let md_output = exporter.export_markdown(tenant_id)?;
    assert!(md_output.contains("# Verzeichnis von Verarbeitungstätigkeiten (Art. 30 DSGVO)"));
    assert!(md_output.contains("Extra Zweck"));

    Ok(())
}
