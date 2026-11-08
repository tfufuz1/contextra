//! CSV table renderer for GDPR Article 30 processing register entries.
//!
//! Provides deterministic RFC 4180 compliant CSV export for audit records.

use crate::{AuditExportError, ProcessingRegisterEntry};

/// Escapes a field according to RFC 4180.
///
/// If the field contains commas, quotation marks, carriage returns, or line breaks,
/// it is enclosed in double quotes and inner quotes are doubled.
fn escape_csv_field(input: &str) -> String {
    let needs_quotes =
        input.contains(',') || input.contains('"') || input.contains('\r') || input.contains('\n');

    if needs_quotes {
        let mut escaped = String::with_capacity(input.len() + 8);
        escaped.push('"');
        for ch in input.chars() {
            if ch == '"' {
                escaped.push_str("\"\"");
            } else {
                escaped.push(ch);
            }
        }
        escaped.push('"');
        escaped
    } else {
        input.to_string()
    }
}

/// Renders a list of [`ProcessingRegisterEntry`] records as a formatted RFC 4180 compliant CSV string.
///
/// Entries are sorted deterministically prior to export. All entries are validated
/// according to GDPR Article 30.
///
/// # Errors
/// Returns [`AuditExportError`] if any entry fails validation.
pub fn render_register_csv(
    entries: &[ProcessingRegisterEntry],
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
    // CSV Header row
    out.push_str("tenant_id,controller_name,controller_contact,representative,dpo,processing_purpose,data_subject_categories,data_categories,recipient_categories,third_country_transfers,retention_periods,toms,legal_basis,deletion_proofs_count,egress_events_count,generated_at\r\n");

    for entry in &sorted_entries {
        let tenant_id = entry.tenant_id.as_u64().to_string();
        let controller_name = escape_csv_field(&entry.controller_contact.controller_name);
        let controller_contact = escape_csv_field(&entry.controller_contact.controller_contact);
        let representative = match entry.controller_contact.representative_contact.as_deref() {
            Some(rep) => escape_csv_field(rep),
            None => String::new(),
        };
        let dpo = match entry.controller_contact.dpo_contact.as_deref() {
            Some(dpo_contact) => escape_csv_field(dpo_contact),
            None => String::new(),
        };
        let purpose = escape_csv_field(&entry.processing_purpose);

        let subjects = escape_csv_field(&entry.data_subject_categories.join("; "));
        let data_cats = escape_csv_field(&entry.data_categories.join("; "));
        let recipients = escape_csv_field(&entry.recipient_categories.join("; "));

        let transfers_str: Vec<String> = entry
            .third_country_transfers
            .iter()
            .map(|t| format!("{}: {}", t.destination_country, t.safeguard_guarantee))
            .collect();
        let transfers = escape_csv_field(&transfers_str.join("; "));

        let retentions_str: Vec<String> = entry
            .retention_periods
            .iter()
            .map(|r| format!("{}: {}", r.data_category, r.retention_period))
            .collect();
        let retentions = escape_csv_field(&retentions_str.join("; "));

        let toms = escape_csv_field(&entry.technical_organizational_measures.join("; "));
        let legal_basis = escape_csv_field(&entry.legal_basis);
        let deletion_proofs = entry.deletion_proofs.len().to_string();
        let egress_events = entry.egress_events.len().to_string();
        let generated_at = entry.generated_at.to_string();

        out.push_str(&format!(
            "{tenant_id},{controller_name},{controller_contact},{representative},{dpo},{purpose},{subjects},{data_cats},{recipients},{transfers},{retentions},{toms},{legal_basis},{deletion_proofs},{egress_events},{generated_at}\r\n"
        ));
    }

    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{testkit::InMemoryProcessingRegisterSource, ProcessingRegisterSource, TenantId};

    #[test]
    fn test_escape_csv_field() {
        assert_eq!(escape_csv_field("simple"), "simple");
        assert_eq!(escape_csv_field("with,comma"), "\"with,comma\"");
        assert_eq!(
            escape_csv_field("with \"quotes\""),
            "\"with \"\"quotes\"\"\""
        );
        assert_eq!(escape_csv_field("with\nnewline"), "\"with\nnewline\"");
    }

    #[test]
    fn test_render_register_csv_roundtrip() -> Result<(), Box<dyn std::error::Error>> {
        let tenant_id = TenantId::try_new(101)?;
        let source = InMemoryProcessingRegisterSource::with_sample_data_for(tenant_id);
        let entries = source.collect_entries(tenant_id)?;

        let csv = render_register_csv(&entries)?;
        assert!(csv.starts_with("tenant_id,controller_name"));
        assert!(csv.contains("Musterfirma GmbH"));
        assert!(csv.contains("Kontextsensitive Vektorsuche"));
        assert!(csv.contains("Dokumenten-Graph-Relationierung"));

        // Header + 2 data rows
        let lines: Vec<&str> = csv.split("\r\n").filter(|s| !s.is_empty()).collect();
        assert_eq!(lines.len(), 3);
        Ok(())
    }
}
