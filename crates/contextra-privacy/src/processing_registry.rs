//! `processing_registry` — Maschinell erzeugtes Verzeichnis von Verarbeitungstätigkeiten (Art. 30 DSGVO).
//!
//! Dieses Modul modelliert und generiert ein Verzeichnis von Verarbeitungstätigkeiten gemäß Art. 30 Abs. 1 DSGVO.
//!
//! ### Kanonischer Mandanten-Typ:
//! Dieses Modul nutzt ausschließlich den kanonischen Domain-Typ [`contextra_types::TenantId`] (`u64`) zur
//! Gewährleistung von systemweiter Mandantentrennung und Isolation.
//!
//! ### Migrationspfad für bestehende Register (Legacy String-TenantId):
//! In früheren Entwicklungsständen wurde der Mandant als `String` (z. B. `"tenant-123"`) gespeichert.
//! Zur Migration bestehender JSON- oder Datenbank-Exporte können numerische IDs extrahiert werden
//! (z. B. `"tenant-123"` -> `123u64`) und über `TenantId::try_new(id)` instanziiert werden.

use crate::egress_guard::{TextSearchEngine, TextSearchResult};
use crate::egress_vault::BoxFuture;
use contextra_types::TenantId;
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

fn default_version() -> u32 {
    1
}

/// Ein Datensatz einer Verarbeitungstätigkeit gemäß Art. 30 Abs. 1 DSGVO.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProcessingActivityRecord {
    /// Bezeichnung der Verarbeitungstätigkeit.
    pub activity_name: String,
    /// Rolle (Verantwortlicher oder Auftragsverarbeiter).
    pub controller_or_processor_role: ProcessorRole,
    /// Zwecke der Verarbeitung.
    pub purpose: String,
    /// Rechtsgrundlage der Verarbeitung (z. B. "Art. 6 Abs. 1 lit. f DSGVO").
    #[serde(default)]
    pub legal_basis: String,
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
    /// Kanonischer Mandanten-Identifikator ([`TenantId`]).
    pub tenant_id: TenantId,
    /// Format- und Schema-Version zur Gewährleistung von Rückwärtskompatibilität (Standard: 1).
    #[serde(default = "default_version")]
    pub version: u32,
    /// Erstellungs-Zeitstempel in Sekunden seit der Unix-Epoche.
    #[serde(default)]
    pub created_at_unix_secs: u64,
}

/// Unveränderliches, append-only Verzeichnis von Verarbeitungstätigkeiten.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProcessingRegistry {
    records: Vec<ProcessingActivityRecord>,
}

impl ProcessingRegistry {
    /// Erstellt ein neues leeres `ProcessingRegistry`.
    pub fn new() -> Self {
        Self { records: Vec::new() }
    }

    /// Fügt einen Datensatz dem Verzeichnis hinzu (append-only) und belegt den Erstellungs-Zeitstempel
    /// über die injizierte Uhr.
    pub fn append_record(&mut self, mut record: ProcessingActivityRecord, clock: &dyn contextra_ports::Clock) {
        let nanos = clock.now_unix_nanos();
        record.created_at_unix_secs = nanos / 1_000_000_000;
        if record.version == 0 {
            record.version = 1;
        }
        self.records.push(record);
    }

    /// Liefert die Datensätze des Verzeichnisses.
    pub fn records(&self) -> &[ProcessingActivityRecord] {
        &self.records
    }

    /// Exportiert das Verzeichnis in eine `ProcessingRegistryExport`-Struktur.
    pub fn export(&self) -> ProcessingRegistryExport {
        ProcessingRegistryExport {
            records: self.records.clone(),
        }
    }
}

/// Strukturierte Export-Repräsentation des Verzeichnisses von Verarbeitungstätigkeiten.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProcessingRegistryExport {
    /// Liste der erfassten Verarbeitungstätigkeiten.
    pub records: Vec<ProcessingActivityRecord>,
}

impl ProcessingRegistryExport {
    /// Erstellt eine neue Export-Instanz mit den angegebenen Datensätzen.
    pub fn new(records: Vec<ProcessingActivityRecord>) -> Self {
        Self { records }
    }

    /// Filtert alle Verarbeitungstätigkeiten nach der angegebenen Mandanten-ID (Art. 15 DSGVO Auskunftsersuchen).
    ///
    /// Garantiert strikte Mandantentrennung bei Auskunftsersuchen, sodass nur Datensätze des Zielmandanten
    /// zurückgegeben werden.
    pub fn filter_by_tenant(&self, tenant_id: TenantId) -> Self {
        let records = self
            .records
            .iter()
            .filter(|r| r.tenant_id == tenant_id)
            .cloned()
            .collect();
        Self { records }
    }
}

impl TextSearchEngine for ProcessingRegistryExport {
    /// Führt eine Volltext-Suche über das Verzeichnis von Verarbeitungstätigkeiten durch (Art. 15 DSGVO).
    fn search_text<'a>(
        &'a self,
        text: &'a str,
        limit: usize,
    ) -> BoxFuture<'a, Result<Vec<TextSearchResult>, String>> {
        Box::pin(async move {
            let query = text.trim().to_lowercase();
            if query.is_empty() {
                return Ok(Vec::new());
            }

            let mut matches = Vec::new();
            for (idx, record) in self.records.iter().enumerate() {
                let mut score = 0.0f32;
                if record.activity_name.to_lowercase().contains(&query) {
                    score += 0.5;
                }
                if record.purpose.to_lowercase().contains(&query) {
                    score += 0.3;
                }
                if record
                    .data_categories
                    .iter()
                    .any(|cat| cat.to_lowercase().contains(&query))
                {
                    score += 0.4;
                }
                if record
                    .data_subject_categories
                    .iter()
                    .any(|sub| sub.to_lowercase().contains(&query))
                {
                    score += 0.4;
                }
                if record
                    .recipients
                    .iter()
                    .any(|rec| rec.to_lowercase().contains(&query))
                {
                    score += 0.2;
                }

                if score > 0.0 {
                    matches.push((idx, score));
                }
            }

            matches.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
            let results = matches
                .into_iter()
                .take(limit)
                .map(|(idx, score)| TextSearchResult {
                    id: format!("record-{}", idx),
                    score: score.min(1.0),
                })
                .collect();

            Ok(results)
        })
    }
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
    out.push_str("| Mandant | Bezeichnung | Rolle | Zweck | Rechtsgrundlage | Datenkategorien | Betroffene | Empfänger | Drittlandtransfers | Löschfrist | TOMs | Erstellt |\n");
    out.push_str("| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |\n");

    for record in &export.records {
        let tenant_str = record.tenant_id.to_string();
        let tenant = sanitize_markdown_cell(&tenant_str);
        let name = sanitize_markdown_cell(&record.activity_name);
        let role = sanitize_markdown_cell(record.controller_or_processor_role.as_str());
        let purpose = sanitize_markdown_cell(&record.purpose);

        let legal_basis = if record.legal_basis.is_empty() {
            "-".to_string()
        } else {
            sanitize_markdown_cell(&record.legal_basis)
        };

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

        let created_at = if record.created_at_unix_secs == 0 {
            "-".to_string()
        } else {
            record.created_at_unix_secs.to_string()
        };

        out.push_str(&format!(
            "| {tenant} | {name} | {role} | {purpose} | {legal_basis} | {data_cats} | {subjects} | {recipients} | {third_country} | {retention} | {toms} | {created_at} |\n"
        ));
    }

    out
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]
mod tests {
    use super::*;

    fn create_sample_record(tenant_id: u64) -> ProcessingActivityRecord {
        ProcessingActivityRecord {
            activity_name: "Kundendaten-Analyse".to_string(),
            controller_or_processor_role: ProcessorRole::Controller,
            purpose: "Bereitstellung von Kontext-Suchen".to_string(),
            legal_basis: "Art. 6 Abs. 1 lit. f DSGVO".to_string(),
            data_categories: vec!["E-Mail".to_string(), "IP-Adresse".to_string()],
            data_subject_categories: vec!["Kunden".to_string(), "Endnutzer".to_string()],
            recipients: vec!["Hosting-Provider".to_string()],
            third_country_transfers: vec![],
            retention_period: "30 Tage nach Kündigung".to_string(),
            technical_and_organizational_measures: vec![
                "Verschlüsselung at rest".to_string(),
                "TLS 1.3".to_string(),
            ],
            tenant_id: TenantId::try_new(tenant_id).unwrap_or(TenantId::SYSTEM),
            version: 1,
            created_at_unix_secs: 1_700_000_000,
        }
    }

    #[test]
    fn test_empty_records_list() {
        let export = generate_registry(&[]);
        assert!(export.records.is_empty());

        let md = render_markdown(&export);
        assert!(md.contains("# Verzeichnis von Verarbeitungstätigkeiten (Art. 30 DSGVO)"));
        assert!(md.contains("| Mandant | Bezeichnung | Rolle | Zweck | Rechtsgrundlage | Datenkategorien | Betroffene | Empfänger | Drittlandtransfers | Löschfrist | TOMs | Erstellt |"));
        let lines: Vec<&str> = md.trim().lines().collect();
        assert_eq!(lines.len(), 4);
    }

    #[test]
    fn test_single_record() {
        let rec = create_sample_record(123);
        let export = generate_registry(std::slice::from_ref(&rec));
        assert_eq!(export.records.len(), 1);

        let md = render_markdown(&export);
        assert!(md.contains("| TenantId(123) | Kundendaten-Analyse | Verantwortlicher | Bereitstellung von Kontext-Suchen | Art. 6 Abs. 1 lit. f DSGVO | E-Mail, IP-Adresse | Kunden, Endnutzer | Hosting-Provider | Keine | 30 Tage nach Kündigung | Verschlüsselung at rest, TLS 1.3 | 1700000000 |"));
    }

    #[test]
    fn test_multiple_records_with_special_characters_and_escaping() {
        let mut rec1 = create_sample_record(123);
        rec1.activity_name = "Verarbeitung | mit | Pipes".to_string();
        rec1.purpose = "Zeilenumordnung\nund | Mehrzeiler".to_string();

        let mut rec2 = create_sample_record(456);
        rec2.controller_or_processor_role = ProcessorRole::Processor;
        rec2.third_country_transfers = vec!["USA (EU-US Data Privacy Framework)".to_string()];

        let export = generate_registry(&[rec1, rec2]);
        let md = render_markdown(&export);

        assert!(md.contains("Verarbeitung \\| mit \\| Pipes"));
        assert!(md.contains("Zeilenumordnung<br/>und \\| Mehrzeiler"));
        assert!(md.contains("Auftragsverarbeiter"));
        assert!(md.contains("USA (EU-US Data Privacy Framework)"));

        let lines: Vec<&str> = md.trim().lines().collect();
        assert_eq!(lines.len(), 6);
    }

    #[tokio::test]
    async fn test_text_search_engine_and_tenant_filter_art15() {
        let rec1 = create_sample_record(100);
        let mut rec2 = create_sample_record(200);
        rec2.activity_name = "Gehaltsabrechnung".to_string();
        rec2.data_categories = vec!["Bankverbindung".to_string(), "Steuer-ID".to_string()];

        let export = generate_registry(&[rec1, rec2]);

        // Search text across all tenants
        let results = export.search_text("Bankverbindung", 10).await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "record-1");

        // Filter by tenant 100 for Art. 15 request
        let tenant100_export = export.filter_by_tenant(TenantId::try_new(100).unwrap());
        assert_eq!(tenant100_export.records.len(), 1);
        assert_eq!(
            tenant100_export.records[0].tenant_id,
            TenantId::try_new(100).unwrap()
        );

        let tenant100_results = tenant100_export
            .search_text("Bankverbindung", 10)
            .await
            .unwrap();
        assert!(tenant100_results.is_empty());
    }
}
