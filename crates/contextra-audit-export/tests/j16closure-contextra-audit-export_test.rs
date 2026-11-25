use contextra_audit_export::{
    render_register_json, testkit::InMemoryProcessingRegisterSource, AuditRegisterExporter,
    DeletionProofSummary, EgressEventSummary, ProcessingRegisterEntry, ProcessingRegisterSource,
};
use contextra_types::TenantId;

#[test]
fn test_closure_render_register_json_direct() -> Result<(), Box<dyn std::error::Error>> {
    let tenant_id = TenantId::try_new(888)?;
    let entry = ProcessingRegisterEntry {
        tenant_id,
        processing_purpose: "Direct JSON Test".to_string(),
        data_categories: vec!["CategoryA".to_string()],
        legal_basis: "Art. 6 Abs. 1 lit. a DSGVO".to_string(),
        egress_events: vec![],
        deletion_proofs: vec![],
        generated_at: 1_700_000_000_000_000_000,
    };

    let json = render_register_json(&[entry.clone()])?;
    assert!(json.contains("Direct JSON Test"));
    assert!(json.contains("CategoryA"));

    let parsed: Vec<ProcessingRegisterEntry> = serde_json::from_str(&json)?;
    assert_eq!(parsed, vec![entry]);

    Ok(())
}

#[test]
fn test_closure_in_memory_source_methods() -> Result<(), Box<dyn std::error::Error>> {
    let tenant_id = TenantId::try_new(999)?;

    // Test InMemoryProcessingRegisterSource::with_sample_data_for
    let mut source = InMemoryProcessingRegisterSource::with_sample_data_for(tenant_id);
    let initial_entries = source.collect_entries(tenant_id)?;
    assert_eq!(initial_entries.len(), 2);

    // Test InMemoryProcessingRegisterSource::add_entry
    let new_entry = ProcessingRegisterEntry {
        tenant_id,
        processing_purpose: "Test Source Add Entry".to_string(),
        data_categories: vec!["Audit Logs".to_string()],
        legal_basis: "Art. 6 Abs. 1 lit. c DSGVO".to_string(),
        egress_events: vec![EgressEventSummary {
            event_id: "evt-999".to_string(),
            destination: "https://audit.example.com".to_string(),
            timestamp_nanos: 1_700_000_000_111,
            detail: "Egress event detail".to_string(),
        }],
        deletion_proofs: vec![DeletionProofSummary {
            proof_id: "del-999".to_string(),
            scope: "user:999".to_string(),
            timestamp_nanos: 1_700_000_000_222,
            verified: true,
        }],
        generated_at: 1_700_000_000_333,
    };

    source.add_entry(new_entry.clone());
    let updated_entries = source.collect_entries(tenant_id)?;
    assert_eq!(updated_entries.len(), 3);
    assert!(updated_entries.contains(&new_entry));

    Ok(())
}

#[test]
fn test_closure_audit_register_exporter_methods() -> Result<(), Box<dyn std::error::Error>> {
    let tenant_id = TenantId::try_new(1001)?;

    // Test AuditRegisterExporter::with_sample_data_for
    let mut exporter = AuditRegisterExporter::with_sample_data_for(tenant_id);

    // Test AuditRegisterExporter::add_entry
    let new_entry = ProcessingRegisterEntry {
        tenant_id,
        processing_purpose: "Test Exporter Add Entry".to_string(),
        data_categories: vec!["Telemetry".to_string()],
        legal_basis: "Art. 6 Abs. 1 lit. f DSGVO".to_string(),
        egress_events: vec![],
        deletion_proofs: vec![],
        generated_at: 1_700_000_000_444,
    };

    exporter.add_entry(new_entry);

    let json_export = exporter.export_json(tenant_id)?;
    assert!(json_export.contains("Kontextsensitive Vektorsuche"));
    assert!(json_export.contains("Test Exporter Add Entry"));

    let md_export = exporter.export_markdown(tenant_id)?;
    assert!(md_export.contains("Test Exporter Add Entry"));

    Ok(())
}
