use contextra_audit_export::{
    bsi_mapping_table, render_art30_register_markdown, render_bsi_mapping_markdown,
    render_register_markdown, testkit::InMemoryProcessingRegisterSource, ProcessingRegisterSource,
};
use contextra_types::TenantId;

fn assert_disclaimer_first_and_last_line(markdown: &str, path_label: &str) {
    let lines: Vec<&str> = markdown.trim().lines().map(|l| l.trim()).collect();
    assert!(
        !lines.is_empty(),
        "Rendered markdown for {} must not be empty",
        path_label
    );

    let first = lines.first().unwrap();
    let last = lines.last().unwrap();

    assert!(
        first.contains("Vorlage, keine Rechtsberatung"),
        "First line of {} rendered output must contain 'Vorlage, keine Rechtsberatung', got: '{}'",
        path_label,
        first
    );

    assert!(
        last.contains("Vorlage, keine Rechtsberatung"),
        "Last line of {} rendered output must contain 'Vorlage, keine Rechtsberatung', got: '{}'",
        path_label,
        last
    );
}

#[test]
fn test_audit_export_renders_have_legal_disclaimer() {
    let tenant_id = TenantId::try_new(42).unwrap();
    let source = InMemoryProcessingRegisterSource::with_sample_data_for(tenant_id);
    let entries = source.collect_entries(tenant_id).unwrap();

    let reg_md = render_register_markdown(&entries).unwrap();
    assert_disclaimer_first_and_last_line(&reg_md, "render_register_markdown");

    let art30_md = render_art30_register_markdown(&[]).unwrap();
    assert_disclaimer_first_and_last_line(&art30_md, "render_art30_register_markdown (empty)");

    let bsi_md = render_bsi_mapping_markdown(&bsi_mapping_table());
    assert_disclaimer_first_and_last_line(&bsi_md, "render_bsi_mapping_markdown");
}

#[test]
fn test_audit_export_wiring_exists() {
    use contextra_ports::clock::ManualClock;
    use std::sync::Arc;

    let tenant_id = TenantId::try_new(42).unwrap();
    let source = InMemoryProcessingRegisterSource::with_sample_data_for(tenant_id);
    let clock: Arc<dyn contextra_ports::clock::Clock> = Arc::new(ManualClock::new(1_000_000));

    let wiring = contextra_audit_export::wiring::AuditExportWiring::build(source, clock)
        .expect("AuditExportWiring::build failed");

    let exported_md = wiring.export_markdown(tenant_id).unwrap();
    assert_disclaimer_first_and_last_line(&exported_md, "AuditExportWiring::export_markdown");
}
