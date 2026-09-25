//! `processing_registry` — Maschinell erzeugtes Verzeichnis von Verarbeitungstätigkeiten (Art. 30 DSGVO).
//!
//! Dieses Modul modelliert und generiert ein Verzeichnis von Verarbeitungstätigkeiten gemäß Art. 30 Abs. 1 DSGVO.
//!
//! ### Hinweis zur Abhängigkeit `TenantId`:
//! `contextra-privacy` verwendet in diesem Modul den lokalen Typ-Alias `pub type TenantId = String;` anstelle
//! einer Abhängigkeit zu `contextra-types`, um den Abhängigkeitsbaum schlank zu halten und den Scope
//! der Crate-Konfiguration zu wahren.

use serde::{Deserialize, Serialize};

/// Rolle bezüglich der Verarbeitungstätigkeit gemäß DSGVO.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum ProcessorRole {
    /// Verantwortlicher (Art. 4 Nr. 7 DSGVO).
    Controller,
    /// Auftragsverarbeiter (Art. 4 Nr. 8 DSGVO).
    Processor,
}

impl ProcessorRole {
    /// Liefert die deutschsprachige Bezeichnung der Rolle.
    pub fn as_str(&self) -> &'static str {
        match self {
            ProcessorRole::Controller => "Verantwortlicher",
            ProcessorRole::Processor => "Auftragsverarbeiter",
        }
    }
}

/// Identifikator eines Mandanten im System.
pub type TenantId = String;

/// Ein Datensatz einer Verarbeitungstätigkeit gemäß Art. 30 Abs. 1 DSGVO.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProcessingActivityRecord {
    /// Bezeichnung der Verarbeitungstätigkeit.
    pub activity_name: String,
    /// Rolle (Verantwortlicher oder Auftragsverarbeiter).
    pub controller_or_processor_role: ProcessorRole,
    /// Zwecke der Verarbeitung.
    pub purpose: String,
    /// Kategorien personenbezogener Daten.
    pub data_categories: Vec<String>,
    /// Kategorien betroffener Personen.
    pub data_subject_categories: Vec<String>,
    /// Kategorien von Empfängern, gegenüber denen die Daten offengelegt wurden oder werden.
    pub recipients: Vec<String>,
    /// Übermittlungen von Daten an ein Drittland oder eine internationale Organisation.
    pub third_country_transfers: Vec<String>,
    /// Vorgesehene Fristen für die Löschung der verschiedenen Datenkategorien.
    pub retention_period: String,
    /// Allgemeine Beschreibung der technischen und organisatorischen Maßnahmen (TOMs).
    pub technical_and_organizational_measures: Vec<String>,
    /// Mandanten-Identifikator.
    pub tenant_id: TenantId,
}

/// Strukturierte Export-Repräsentation des Verzeichnisses von Verarbeitungstätigkeiten.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProcessingRegistryExport {
    /// Liste der erfassten Verarbeitungstätigkeiten.
    pub records: Vec<ProcessingActivityRecord>,
}

/// Erzeugt aus einer Reihe von Datensätzen eine strukturierte Export-Repräsentation.
pub fn generate_registry(records: &[ProcessingActivityRecord]) -> ProcessingRegistryExport {
    ProcessingRegistryExport {
        records: records.to_vec(),
    }
}

/// Hilfsfunktion zum Maskieren von Pipe-Zeichen und Zeilenumbrüchen für Markdown-Tabellenzellen.
fn sanitize_markdown_cell(input: &str) -> String {
    input
        .replace('|', "\\|")
        .replace("\r\n", "<br/>")
        .replace(['\n', '\r'], "<br/>")
}

/// Erzeugt eine druckfertige, deutschsprachige Markdown-Tabelle aus dem Export.
///
/// Die Tabelle enthält eine Zeile pro `ProcessingActivityRecord` und Spalten für alle
/// nach Art. 30 Abs. 1 DSGVO geforderten Angaben.
pub fn render_markdown(export: &ProcessingRegistryExport) -> String {
    let mut out = String::new();
    out.push_str("# Verzeichnis von Verarbeitungstätigkeiten (Art. 30 DSGVO)\n\n");
    out.push_str("| Mandant | Bezeichnung | Rolle | Zweck | Datenkategorien | Betroffene | Empfänger | Drittlandtransfers | Löschfrist | TOMs |\n");
    out.push_str("| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |\n");

    for record in &export.records {
        let tenant = sanitize_markdown_cell(&record.tenant_id);
        let name = sanitize_markdown_cell(&record.activity_name);
        let role = sanitize_markdown_cell(record.controller_or_processor_role.as_str());
        let purpose = sanitize_markdown_cell(&record.purpose);

        let data_cats = if record.data_categories.is_empty() {
            "-".to_string()
        } else {
            sanitize_markdown_cell(&record.data_categories.join(", "))
        };

        let subjects = if record.data_subject_categories.is_empty() {
            "-".to_string()
        } else {
            sanitize_markdown_cell(&record.data_subject_categories.join(", "))
        };

        let recipients = if record.recipients.is_empty() {
            "-".to_string()
        } else {
            sanitize_markdown_cell(&record.recipients.join(", "))
        };

        let third_country = if record.third_country_transfers.is_empty() {
            "Keine".to_string()
        } else {
            sanitize_markdown_cell(&record.third_country_transfers.join(", "))
        };

        let retention = sanitize_markdown_cell(&record.retention_period);

        let toms = if record.technical_and_organizational_measures.is_empty() {
            "-".to_string()
        } else {
            sanitize_markdown_cell(&record.technical_and_organizational_measures.join(", "))
        };

        out.push_str(&format!(
            "| {tenant} | {name} | {role} | {purpose} | {data_cats} | {subjects} | {recipients} | {third_country} | {retention} | {toms} |\n"
        ));
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_sample_record() -> ProcessingActivityRecord {
        ProcessingActivityRecord {
            activity_name: "Kundendaten-Analyse".to_string(),
            controller_or_processor_role: ProcessorRole::Controller,
            purpose: "Bereitstellung von Kontext-Suchen".to_string(),
            data_categories: vec!["E-Mail".to_string(), "IP-Adresse".to_string()],
            data_subject_categories: vec!["Kunden".to_string(), "Endnutzer".to_string()],
            recipients: vec!["Hosting-Provider".to_string()],
            third_country_transfers: vec![],
            retention_period: "30 Tage nach Kündigung".to_string(),
            technical_and_organizational_measures: vec![
                "Verschlüsselung at rest".to_string(),
                "TLS 1.3".to_string(),
            ],
            tenant_id: "tenant-123".to_string(),
        }
    }

    #[test]
    fn test_empty_records_list() {
        let export = generate_registry(&[]);
        assert!(export.records.is_empty());

        let md = render_markdown(&export);
        assert!(md.contains("# Verzeichnis von Verarbeitungstätigkeiten (Art. 30 DSGVO)"));
        assert!(md.contains("| Mandant | Bezeichnung | Rolle | Zweck | Datenkategorien | Betroffene | Empfänger | Drittlandtransfers | Löschfrist | TOMs |"));
        // Header title + blank line + table header + separator = 4 lines total
        let lines: Vec<&str> = md.trim().lines().collect();
        assert_eq!(lines.len(), 4);
    }

    #[test]
    fn test_single_record() {
        let rec = create_sample_record();
        let export = generate_registry(&[rec.clone()]);
        assert_eq!(export.records.len(), 1);

        let md = render_markdown(&export);
        assert!(md.contains("| tenant-123 | Kundendaten-Analyse | Verantwortlicher | Bereitstellung von Kontext-Suchen | E-Mail, IP-Adresse | Kunden, Endnutzer | Hosting-Provider | Keine | 30 Tage nach Kündigung | Verschlüsselung at rest, TLS 1.3 |"));
    }

    #[test]
    fn test_multiple_records_with_special_characters_and_escaping() {
        let mut rec1 = create_sample_record();
        rec1.activity_name = "Verarbeitung | mit | Pipes".to_string();
        rec1.purpose = "Zeilenumordnung\nund | Mehrzeiler".to_string();

        let mut rec2 = create_sample_record();
        rec2.tenant_id = "tenant-456".to_string();
        rec2.controller_or_processor_role = ProcessorRole::Processor;
        rec2.third_country_transfers = vec!["USA (EU-US Data Privacy Framework)".to_string()];

        let export = generate_registry(&[rec1, rec2]);
        let md = render_markdown(&export);

        // Check escaping of pipe symbol
        assert!(md.contains("Verarbeitung \\| mit \\| Pipes"));
        assert!(md.contains("Zeilenumordnung<br/>und \\| Mehrzeiler"));
        assert!(md.contains("Auftragsverarbeiter"));
        assert!(md.contains("USA (EU-US Data Privacy Framework)"));

        // Ensure each record creates exactly 1 table row (no stray unescaped newlines in rows)
        let lines: Vec<&str> = md.trim().lines().collect();
        // 4 header setup lines + 2 data rows = 6 lines total
        assert_eq!(lines.len(), 6);
    }
}
