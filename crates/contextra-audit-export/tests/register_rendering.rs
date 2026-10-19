use contextra_audit_export::{
    render_register_json, render_register_markdown, testkit::InMemoryProcessingRegisterSource,
    AuditRegisterExporter, ProcessingRegisterEntry, ProcessingRegisterSource,
};
use contextra_types::TenantId;

#[test]
fn test_json_rendering_roundtrip() -> Result<(), Box<dyn std::error::Error>> {
    let tenant_id = TenantId::try_new(100)?;
    let source = InMemoryProcessingRegisterSource::with_sample_data_for(tenant_id);

    let entries = source.collect_entries(tenant_id)?;
    assert_eq!(entries.len(), 2);

    let json_output = render_register_json(&entries)?;
    assert!(!json_output.is_empty());

    let deserialized: Vec<ProcessingRegisterEntry> = serde_json::from_str(&json_output)?;
    assert_eq!(deserialized, entries);

    Ok(())
}

#[test]
fn test_audit_register_exporter_api() -> Result<(), Box<dyn std::error::Error>> {
    let tenant_id = TenantId::try_new(300)?;
    let mut exporter = AuditRegisterExporter::with_sample_data_for(tenant_id);

    // Test source() and source_mut()
    let initial_entries = exporter.source().collect_entries(tenant_id)?;
    assert_eq!(initial_entries.len(), 2);

    let new_entry = ProcessingRegisterEntry {
        tenant_id,
        processing_purpose: "Test purpose added via source_mut".to_string(),
        data_categories: vec!["Audit Logs".to_string()],
        legal_basis: "Art. 6 Abs. 1 lit. c DSGVO".to_string(),
        egress_events: vec![],
        deletion_proofs: vec![],
        generated_at: 1_700_000_200_000_000_000,
    };

    exporter.source_mut().add_entry(new_entry.clone());

    let updated_entries = exporter.source().collect_entries(tenant_id)?;
    assert_eq!(updated_entries.len(), 3);

    // Test export_json()
    let json_export = exporter.export_json(tenant_id)?;
    let parsed: Vec<ProcessingRegisterEntry> = serde_json::from_str(&json_export)?;
    assert_eq!(parsed.len(), 3);
    assert_eq!(parsed[2], new_entry);

    // Test export_markdown()
    let markdown_export = exporter.export_markdown(tenant_id)?;
    assert!(markdown_export.contains("Test purpose added via source_mut"));

    Ok(())
}

#[test]
fn test_markdown_rendering_data_rows() -> Result<(), Box<dyn std::error::Error>> {
    let tenant_id = TenantId::try_new(200)?;
    let source = InMemoryProcessingRegisterSource::with_sample_data_for(tenant_id);

    let entries = source.collect_entries(tenant_id)?;
    assert_eq!(entries.len(), 2);

    let markdown_output = render_register_markdown(&entries)?;
    assert!(markdown_output.contains("# Verzeichnis von Verarbeitungstätigkeiten (Art. 30 DSGVO)"));

    // Count data rows (lines starting with '|') excluding header and separator rows.
    let table_lines: Vec<&str> = markdown_output
        .lines()
        .filter(|line| line.starts_with('|'))
        .collect();

    // Table lines must be 2 header/separator lines + 2 data rows = 4 table lines
    assert_eq!(table_lines.len(), 4);

    let data_rows: Vec<&&str> = table_lines
        .iter()
        .filter(|line| !line.contains("---") && !line.contains("Mandant"))
        .collect();

    assert_eq!(data_rows.len(), entries.len());

    Ok(())
}

#[test]
fn test_empty_entries_rendering() -> Result<(), Box<dyn std::error::Error>> {
    let empty_entries: Vec<ProcessingRegisterEntry> = vec![];

    let json_output = render_register_json(&empty_entries)?;
    assert_eq!(json_output.trim(), "[]");

    let deserialized: Vec<ProcessingRegisterEntry> = serde_json::from_str(&json_output)?;
    assert!(deserialized.is_empty());

    let markdown_output = render_register_markdown(&empty_entries)?;
    assert!(markdown_output.contains("# Verzeichnis von Verarbeitungstätigkeiten (Art. 30 DSGVO)"));

    let table_lines: Vec<&str> = markdown_output
        .lines()
        .filter(|line| line.starts_with('|'))
        .collect();

    // Header + separator lines, 0 data rows
    assert_eq!(table_lines.len(), 2);

    let data_rows: Vec<&&str> = table_lines
        .iter()
        .filter(|line| !line.contains("---") && !line.contains("Mandant"))
        .collect();

    assert_eq!(data_rows.len(), 0);

    Ok(())
}
