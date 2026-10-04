//! Markdown table renderer for GDPR Article 30 processing register entries.

use crate::{Art30ProcessingRegisterEntry, AuditExportError, ProcessingRegisterEntry};

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

/// Renders a list of [`Art30ProcessingRegisterEntry`] records as a formatted Markdown document containing
/// all mandatory Article 30 GDPR attributes (lit. a–g).
///
/// # Table / Section Columns (Art. 30 Abs. 1 lit. a–g)
/// - **Verantwortlicher / Kontaktdaten** (lit. a)
/// - **Zwecke der Verarbeitung** (lit. b)
/// - **Kategorien betroffener Personen** (lit. c)
/// - **Kategorien personenbezogener Daten** (lit. c)
/// - **Kategorien von Empfängern** (lit. d)
/// - **Drittlandübermittlungen** (lit. e)
/// - **Löschfristen** (lit. f)
/// - **Technische und organisatorische Maßnahmen** (lit. g)
/// - **Rechtsgrundlage**
/// - **Mandant & Audit-Nachweise**
pub fn render_art30_register_markdown(
    entries: &[Art30ProcessingRegisterEntry],
) -> Result<String, AuditExportError> {
    let mut out = String::new();
    out.push_str("# Verzeichnis von Verarbeitungstätigkeiten (Art. 30 Abs. 1 lit. a–g DSGVO)\n\n");

    if entries.is_empty() {
        out.push_str("*Keine Einträge vorhanden.*\n");
        return Ok(out);
    }

    out.push_str("| Mandant | Verantwortlicher (lit. a) | Zweck (lit. b) | Betroffene Personen (lit. c) | Datenkategorien (lit. c) | Empfänger (lit. d) | Drittlandübermittlung (lit. e) | Löschfristen (lit. f) | TOMs (lit. g) | Rechtsgrundlage | Löschnachweise | Egress-Events | Generiert am |\n");
    out.push_str("| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |\n");

    for entry in entries {
        let tenant = sanitize_markdown_cell(&entry.tenant_id.to_string());
        let controller = sanitize_markdown_cell(&entry.controller_details);
        let purpose = sanitize_markdown_cell(&entry.processing_purpose);

        let subject_categories = if entry.data_subject_categories.is_empty() {
            "-".to_string()
        } else {
            sanitize_markdown_cell(&entry.data_subject_categories.join(", "))
        };

        let data_categories = if entry.data_categories.is_empty() {
            "-".to_string()
        } else {
            sanitize_markdown_cell(&entry.data_categories.join(", "))
        };

        let recipient_categories = if entry.recipient_categories.is_empty() {
            "-".to_string()
        } else {
            sanitize_markdown_cell(&entry.recipient_categories.join(", "))
        };

        let third_country = match &entry.third_country_transfers {
            Some(tc) if !tc.trim().is_empty() => sanitize_markdown_cell(tc),
            _ => "Keine".to_string(),
        };

        let erasure = sanitize_markdown_cell(&entry.erasure_deadlines);

        let toms = if entry.technical_organizational_measures.is_empty() {
            "-".to_string()
        } else {
            sanitize_markdown_cell(&entry.technical_organizational_measures.join(", "))
        };

        let legal_basis = sanitize_markdown_cell(&entry.legal_basis);
        let deletion_proofs_count = entry.deletion_proofs.len();
        let egress_events_count = entry.egress_events.len();
        let generated_at = entry.generated_at;

        out.push_str(&format!(
            "| {tenant} | {controller} | {purpose} | {subject_categories} | {data_categories} | {recipient_categories} | {third_country} | {erasure} | {toms} | {legal_basis} | {deletion_proofs_count} | {egress_events_count} | {generated_at} |\n"
        ));
    }

    Ok(out)
}
