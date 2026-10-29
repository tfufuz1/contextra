//! Error formatting and edge cases test suite.

use contextra_audit_export::{
    render_register_json, render_register_markdown, testkit::InMemoryProcessingRegisterSource,
    AuditExportError, ControllerContact, ProcessingRegisterSource,
};
use contextra_types::TenantId;

#[test]
fn test_error_display_formatting() {
    let err_missing = AuditExportError::MissingField {
        field: "processing_purpose".to_string(),
    };
    assert_eq!(
        err_missing.to_string(),
        "Missing mandatory field: processing_purpose"
    );

    let err_source = AuditExportError::SourceCollection("Database offline".to_string());
    assert_eq!(
        err_source.to_string(),
        "Source collection error: Database offline"
    );

    let err_tenant = AuditExportError::InvalidTenant("0".to_string());
    assert_eq!(
        err_tenant.to_string(),
        "Requested tenant identifier is invalid or missing: 0"
    );
}

#[test]
fn test_source_collection_empty_returns_empty_vec() -> Result<(), Box<dyn std::error::Error>> {
    let source = InMemoryProcessingRegisterSource::new();
    let tenant_id = TenantId::try_new(999)?;
    let entries = source.collect_entries(tenant_id)?;
    assert!(entries.is_empty());
    Ok(())
}

#[test]
fn test_empty_optional_contacts_rendering() -> Result<(), Box<dyn std::error::Error>> {
    let tenant_id = TenantId::try_new(700)?;
    let source = InMemoryProcessingRegisterSource::with_sample_data_for(tenant_id);

    let mut entry = source
        .collect_entries(tenant_id)?
        .first()
        .cloned()
        .ok_or("No entry found")?;
    entry.controller_contact = ControllerContact {
        controller_name: "Minimal GmbH".to_string(),
        controller_contact: "info@minimal.de".to_string(),
        representative_contact: None,
        dpo_contact: None,
    };

    let rendered_md = render_register_markdown(&[entry.clone()], "2026-10-28")?;
    assert!(rendered_md.contains("Minimal GmbH (info@minimal.de)"));
    assert!(!rendered_md.contains("Vertreter:"));
    assert!(!rendered_md.contains("DSB:"));

    let rendered_json = render_register_json(&[entry])?;
    assert!(rendered_json.contains("\"representative_contact\": null"));
    assert!(rendered_json.contains("\"dpo_contact\": null"));

    Ok(())
}
