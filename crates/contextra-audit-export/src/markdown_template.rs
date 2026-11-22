//! Markdown table renderer for GDPR Article 30 processing register entries.

use crate::{AuditExportError, ProcessingRegisterEntry};

/// Sanitizes input strings against Markdown injection and table breakage.
///
/// Escapes ampersands (`&`), pipes (`|`), backticks (`` ` ``), newlines (`\r\n`, `\n`, `\r`), and HTML tags / angle brackets (`<`, `>`).
pub fn sanitize_markdown_cell(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('|', "\\|")
        .replace('`', "\\`")
        .replace("\r\n", "<br/>")
        .replace(['\n', '\r'], "<br/>")
}

/// Renders a list of [`ProcessingRegisterEntry`] records as a formatted Markdown document containing
/// a GDPR Article 30 processing register table.
///
/// # Parameters
/// - `entries`: List of processing register entries to export.
/// - `export_date`: Explicit date string (e.g., "2026-10-28") for deterministic export generation.
pub fn render_register_markdown(
    entries: &[ProcessingRegisterEntry],
    export_date: &str,
) -> Result<String, AuditExportError> {
    let mut sorted_entries = entries.to_vec();
    for entry in &mut sorted_entries {
        entry.validate()?;
        entry.sort_internal_collections();
    }
    sorted_entries.sort_by(|a, b| {
        a.tenant_id
            .as_u64()
            .cmp(&b.tenant_id.as_u64())
            .then_with(|| a.processing_purpose.cmp(&b.processing_purpose))
            .then_with(|| {
                a.controller_contact
                    .controller_name
                    .cmp(&b.controller_contact.controller_name)
            })
    });

    let mut out = String::new();
    out.push_str("# Verzeichnis von Verarbeitungstätigkeiten (Art. 30 DSGVO)\n\n");
    out.push_str(&format!(
        "**Exportdatum:** {} | **Rechtsgrundlage:** Art. 30 Abs. 1 DSGVO\n\n",
        sanitize_markdown_cell(export_date)
    ));

    out.push_str("| Mandant | Verantwortlicher / Kontakt | Zweck der Verarbeitung | Betroffene / Datenkategorien | Empfänger | Drittland & Garantien | Löschfristen | TOM (Art. 32) | Rechtsgrundlage | Löschnachweise | Egress-Events |\n");
    out.push_str("| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |\n");

    for entry in &sorted_entries {
        let tenant = sanitize_markdown_cell(&entry.tenant_id.as_u64().to_string());

        let name = sanitize_markdown_cell(&entry.controller_contact.controller_name);
        let contact = sanitize_markdown_cell(&entry.controller_contact.controller_contact);
        let mut controller_parts = vec![format!("{name} ({contact})")];

        if let Some(ref rep) = entry.controller_contact.representative_contact {
            let rep_s = sanitize_markdown_cell(rep);
            controller_parts.push(format!("Vertreter: {rep_s}"));
        }
        if let Some(ref dpo) = entry.controller_contact.dpo_contact {
            let dpo_s = sanitize_markdown_cell(dpo);
            controller_parts.push(format!("DSB: {dpo_s}"));
        }
        let controller_cell = controller_parts.join("<br/>");

        let purpose = sanitize_markdown_cell(&entry.processing_purpose);

        let subjects_clean: Vec<String> = entry
            .data_subject_categories
            .iter()
            .map(|s| sanitize_markdown_cell(s))
            .collect();
        let data_clean: Vec<String> = entry
            .data_categories
            .iter()
            .map(|s| sanitize_markdown_cell(s))
            .collect();
        let categories_cell = format!(
            "Betroffene: {}<br/>Daten: {}",
            subjects_clean.join(", "),
            data_clean.join(", ")
        );

        let recipients_clean: Vec<String> = entry
            .recipient_categories
            .iter()
            .map(|s| sanitize_markdown_cell(s))
            .collect();
        let recipient_cell = recipients_clean.join(", ");

        let third_country_cell = if entry.third_country_transfers.is_empty() {
            "Keine".to_string()
        } else {
            let transfers: Vec<String> = entry
                .third_country_transfers
                .iter()
                .map(|t| {
                    format!(
                        "{}: {}",
                        sanitize_markdown_cell(&t.destination_country),
                        sanitize_markdown_cell(&t.safeguard_guarantee)
                    )
                })
                .collect();
            transfers.join("<br/>")
        };

        let retention_cell = {
            let periods: Vec<String> = entry
                .retention_periods
                .iter()
                .map(|r| {
                    format!(
                        "{}: {}",
                        sanitize_markdown_cell(&r.data_category),
                        sanitize_markdown_cell(&r.retention_period)
                    )
                })
                .collect();
            periods.join("<br/>")
        };

        let toms_clean: Vec<String> = entry
            .technical_organizational_measures
            .iter()
            .map(|s| sanitize_markdown_cell(s))
            .collect();
        let tom_cell = toms_clean.join("<br/>");

        let legal_basis = sanitize_markdown_cell(&entry.legal_basis);
        let deletion_proofs_count = entry.deletion_proofs.len();
        let egress_events_count = entry.egress_events.len();

        out.push_str(&format!(
            "| {tenant} | {controller_cell} | {purpose} | {categories_cell} | {recipient_cell} | {third_country_cell} | {retention_cell} | {tom_cell} | {legal_basis} | {deletion_proofs_count} | {egress_events_count} |\n"
        ));
    }

    Ok(out)
}
