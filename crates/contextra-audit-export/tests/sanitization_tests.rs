//! Sanitization and Markdown injection prevention test suite.

use contextra_audit_export::{
    markdown_template::sanitize_markdown_cell, render_register_markdown,
    testkit::InMemoryProcessingRegisterSource, ProcessingRegisterSource,
};
use contextra_types::TenantId;

#[test]
fn test_sanitize_pipes() {
    let input = "Firma | Abteilung | Team";
    let sanitized = sanitize_markdown_cell(input);
    assert_eq!(sanitized, "Firma \\| Abteilung \\| Team");
}

#[test]
fn test_sanitize_backticks() {
    let input = "Code `SELECT * FROM users` Executed";
    let sanitized = sanitize_markdown_cell(input);
    assert_eq!(sanitized, "Code \\`SELECT * FROM users\\` Executed");
}

#[test]
fn test_sanitize_newlines() {
    let input = "Zeile 1\r\nZeile 2\nZeile 3\rZeile 4";
    let sanitized = sanitize_markdown_cell(input);
    assert_eq!(sanitized, "Zeile 1<br/>Zeile 2<br/>Zeile 3<br/>Zeile 4");
}

#[test]
fn test_sanitize_html_tags() {
    let input = "<script>alert('xss')</script>";
    let sanitized = sanitize_markdown_cell(input);
    assert_eq!(sanitized, "&lt;script&gt;alert('xss')&lt;/script&gt;");
}

#[test]
fn test_markdown_rendering_sanitization_in_fields() -> Result<(), Box<dyn std::error::Error>> {
    let tenant_id = TenantId::try_new(500)?;
    let source = InMemoryProcessingRegisterSource::with_sample_data_for(tenant_id);

    let mut malicious_entry = source
        .collect_entries(tenant_id)?
        .first()
        .cloned()
        .ok_or("No entry found")?;
    malicious_entry.processing_purpose =
        "Zweck mit | Pipe und `Code` und <script>alert(1)</script>".to_string();
    malicious_entry.controller_contact.controller_name = "Böse Firma | Evil Corp".to_string();

    let rendered = render_register_markdown(&[malicious_entry], "2026-10-28")?;

    assert!(rendered.contains("Böse Firma \\| Evil Corp"));
    assert!(rendered
        .contains("Zweck mit \\| Pipe und \\`Code\\` und &lt;script&gt;alert(1)&lt;/script&gt;"));

    Ok(())
}

#[test]
fn test_unicode_umlaute_preservation() -> Result<(), Box<dyn std::error::Error>> {
    let tenant_id = TenantId::try_new(600)?;
    let source = InMemoryProcessingRegisterSource::with_sample_data_for(tenant_id);

    let mut entry = source
        .collect_entries(tenant_id)?
        .first()
        .cloned()
        .ok_or("No entry found")?;
    entry.processing_purpose =
        "Prüfung von Betroffenenrechten & Löschaufträgen für Ärgernisse, Öllager und Übelkeit"
            .to_string();

    let rendered = render_register_markdown(&[entry], "2026-10-28")?;
    assert!(rendered.contains(
        "Prüfung von Betroffenenrechten &amp; Löschaufträgen für Ärgernisse, Öllager und Übelkeit"
    ));

    Ok(())
}
