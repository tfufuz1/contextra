//! Retention classes and data retention matrix (Datenschutz-Löschkonzept / Art. 30 DSGVO).
//!
//! # Compliance & Standards Notice
//! Maps data categories x legal bases x retention periods x trigger events.
//!
//! **NOTE ON STANDARDS:**
//! Die Norm-Bezeichnung (DIN 66398 bzw. DIN EN ISO/IEC 27555:2025-09) ist noch zu verifizieren.
//!
//! **LEGAL DISCLAIMER:**
//! Vorlage, keine Rechtsberatung.

use crate::error::AuditExportError;
use serde::{Deserialize, Serialize};

/// Trigger event that initiates the retention period counter (Startereignis).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TriggerEvent {
    /// Contract termination or closure of customer account (Vertragsende / Konto-Schließung).
    ContractTermination,
    /// Completion of transaction / invoice issuance (Vertragserfüllung / Rechnungsstellung).
    TransactionCompletion,
    /// Revocation of consent under Art. 6(1)(a) / Art. 7 GDPR (Einwilligungswiderruf).
    ConsentRevocation,
    /// End of calendar year in which data was recorded (Ablauf des Kalenderjahres der Erfassung).
    CalendarYearEnd,
    /// Expiry of statutory limitation or retention period (Ablauf gesetzlicher Aufbewahrungsfrist).
    StatutoryPeriodExpiry,
    /// Custom trigger event description.
    Custom(String),
}

impl TriggerEvent {
    /// Returns a human-readable German designation for audit reports.
    pub fn description(&self) -> &str {
        match self {
            Self::ContractTermination => "Vertragsende / Konto-Schließung",
            Self::TransactionCompletion => "Vertragserfüllung / Rechnungsstellung",
            Self::ConsentRevocation => "Einwilligungswiderruf",
            Self::CalendarYearEnd => "Ablauf des Kalenderjahres der Erfassung",
            Self::StatutoryPeriodExpiry => "Ablauf der gesetzlichen Aufbewahrungsfrist",
            Self::Custom(s) => s.as_str(),
        }
    }
}

/// Regular retention duration (Regelfrist).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RetentionPeriod {
    /// Immediate deletion upon trigger event (Unverzügliche Löschung).
    Immediate,
    /// Specific number of days.
    Days(u32),
    /// Specific number of months.
    Months(u32),
    /// Specific number of years (e.g., 6 years for § 257 HGB, 10 years for § 147 AO).
    Years(u32),
    /// Retained until purpose ceases or consent revoked without fixed calendar limit.
    UntilPurposeEnded,
}

impl RetentionPeriod {
    /// Returns a formatted designation string for matrix exports.
    pub fn display_text(&self) -> String {
        match self {
            Self::Immediate => "Unverzüglich".to_string(),
            Self::Days(d) => format!("{d} Tage"),
            Self::Months(m) => format!("{m} Monate"),
            Self::Years(y) => format!("{y} Jahre"),
            Self::UntilPurposeEnded => "Bis Zweckwegfall / Widerruf".to_string(),
        }
    }
}

/// A retention class definition mapping a category of personal data to a trigger event,
/// retention period, and legal basis reference (Löschklasse / Retention Class).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RetentionClass {
    /// Unique identifier or identifier key for the retention class (e.g. "RK-101").
    pub class_id: String,
    /// Descriptive title of the retention class (Name der Löschklasse).
    pub name: String,
    /// Categories of personal data covered by this retention class.
    pub data_categories: Vec<String>,
    /// Statutory or legitimate legal basis reference (e.g., "Art. 6 Abs. 1 lit. b DSGVO, § 147 AO").
    pub legal_basis_ref: String,
    /// Event initiating the retention period (Startereignis).
    pub trigger_event: TriggerEvent,
    /// Statutory or policy retention period (Regelfrist).
    pub retention_period: RetentionPeriod,
    /// Additional context, technical safeguards, or erasure mode details.
    pub note: String,
}

/// Matrix export container for GDPR Art. 30 retention and deletion schedules (Löschkonzept-Matrix).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RetentionMatrix {
    /// List of retention classes constituting the matrix.
    pub classes: Vec<RetentionClass>,
}

impl RetentionMatrix {
    /// Creates a new [`RetentionMatrix`].
    pub fn new(classes: Vec<RetentionClass>) -> Self {
        Self { classes }
    }

    /// Renders the retention matrix as a formatted Markdown table suitable for GDPR Art. 30 exports.
    pub fn render_markdown(&self) -> String {
        let mut out = String::new();
        out.push_str("# Datenschutz-Löschkonzept & Retention-Matrix (Art. 30 / Art. 17 DSGVO)\n\n");
        out.push_str("> **Hinweis zur Normung:** Die Norm-Bezeichnung (DIN 66398 bzw. DIN EN ISO/IEC 27555:2025-09) ist noch zu verifizieren.\n");
        out.push_str("> **RECHTLICHER HINWEIS:** Vorlage, keine Rechtsberatung.\n\n");

        out.push_str("| Löschklasse-ID | Name / Zweck | Datenkategorien | Rechtsgrundlage | Startereignis | Regelfrist | Anmerkungen |\n");
        out.push_str("| --- | --- | --- | --- | --- | --- | --- |\n");

        for class in &self.classes {
            let id = sanitize_markdown_cell(&class.class_id);
            let name = sanitize_markdown_cell(&class.name);
            let categories = sanitize_markdown_cell(&class.data_categories.join(", "));
            let legal = sanitize_markdown_cell(&class.legal_basis_ref);
            let trigger = sanitize_markdown_cell(class.trigger_event.description());
            let period = sanitize_markdown_cell(&class.retention_period.display_text());
            let note = sanitize_markdown_cell(&class.note);

            out.push_str(&format!(
                "| {id} | {name} | {categories} | {legal} | {trigger} | {period} | {note} |\n"
            ));
        }

        out
    }

    /// Renders the retention matrix as a formatted JSON string.
    pub fn render_json(&self) -> Result<String, AuditExportError> {
        serde_json::to_string_pretty(self).map_err(AuditExportError::from)
    }
}

/// Sanitizes special characters (pipes and newlines) within a markdown table cell.
fn sanitize_markdown_cell(input: &str) -> String {
    input
        .replace('|', "\\|")
        .replace("\r\n", "<br/>")
        .replace(['\n', '\r'], "<br/>")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_retention_matrix_rendering() {
        let rk1 = RetentionClass {
            class_id: "RK-01".to_string(),
            name: "Vertrags- & Abrechnungsdaten".to_string(),
            data_categories: vec!["Stammdaten".to_string(), "Rechnungsdaten".to_string()],
            legal_basis_ref: "§ 147 AO, § 257 HGB".to_string(),
            trigger_event: TriggerEvent::CalendarYearEnd,
            retention_period: RetentionPeriod::Years(10),
            note: "Aufbewahrungspflicht für steuerlich relevante Unterlagen".to_string(),
        };

        let matrix = RetentionMatrix::new(vec![rk1]);
        let md = matrix.render_markdown();

        assert!(md.contains("# Datenschutz-Löschkonzept & Retention-Matrix"));
        assert!(md.contains("DIN 66398 bzw. DIN EN ISO/IEC 27555:2025-09"));
        assert!(md.contains("Vorlage, keine Rechtsberatung"));
        assert!(md.contains("RK-01"));
        assert!(md.contains("10 Jahre"));

        let json = matrix.render_json().unwrap();
        assert!(json.contains("RK-01"));
    }
}
