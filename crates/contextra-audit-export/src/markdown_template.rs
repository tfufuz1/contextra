//! Markdown table renderer for GDPR Article 30 processing register entries.

use crate::{AuditExportError, ProcessingRegisterEntry};

/// Sanitizes special characters (pipes and newlines) within a markdown table cell.
fn sanitize_markdown_cell(input: &str) -> String {
    input
        .replace('|', "\\|")
        .replace("\r\n", "<br/>")
        .replace(['\n', '\r'], "<br/>")
}

/// Renders a list of [`ProcessingRegisterEntry`] records as a formatted Markdown document containing
/// a GDPR Article 30 processing register table.
///
/// # Table Columns
/// - Mandant
/// - Zweck
/// - Datenkategorien
/// - Rechtsgrundlage
/// - Löschnachweise-Anzahl
/// - Egress-Ereignisse-Anzahl
/// - Generiert am
pub fn render_register_markdown(
    entries: &[ProcessingRegisterEntry],
) -> Result<String, AuditExportError> {
    let mut out = String::new();
    out.push_str("# Verzeichnis von Verarbeitungstätigkeiten (Art. 30 DSGVO)\n\n");
    out.push_str("| Mandant | Zweck | Datenkategorien | Rechtsgrundlage | Löschnachweise-Anzahl | Egress-Ereignisse-Anzahl | Generiert am |\n");
    out.push_str("| --- | --- | --- | --- | --- | --- | --- |\n");

    for entry in entries {
        let tenant = sanitize_markdown_cell(&entry.tenant_id.to_string());
        let purpose = sanitize_markdown_cell(&entry.processing_purpose);
        let categories = if entry.data_categories.is_empty() {
            "-".to_string()
        } else {
            sanitize_markdown_cell(&entry.data_categories.join(", "))
        };
        let legal_basis = sanitize_markdown_cell(&entry.legal_basis);
        let deletion_proofs_count = entry.deletion_proofs.len();
        let egress_events_count = entry.egress_events.len();
        let generated_at = entry.generated_at;

        out.push_str(&format!(
            "| {tenant} | {purpose} | {categories} | {legal_basis} | {deletion_proofs_count} | {egress_events_count} | {generated_at} |\n"
        ));
    }

    Ok(out)
}
