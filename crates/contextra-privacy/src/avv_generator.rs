//! `avv_generator` — Auftragsverarbeitungsvertrag (AVV) Generierung gemäß Art. 28 DSGVO.
//!
//! Generiert aus strukturierten Kontextdaten (`AvvContext`) einen rechtlich orientierten AVV-Entwurf als
//! Markdown-Dokument. Der Entwurf referenziert die spezifischen technischen Garantien des Produkts
//! (z. B. kryptografische Löschbeweise, Cloud Egress Gateway, Mandantentrennung).

use serde::{Deserialize, Serialize};

/// Kontextdaten für die Erstellung eines Auftragsverarbeitungsvertrags (AVV).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AvvContext {
    /// Name oder Bezeichnung des Verantwortlichen (Auftraggeber).
    pub controller_name: String,
    /// Name oder Bezeichnung des Auftragsverarbeiters (Auftragnehmer).
    pub processor_name: String,
    /// Datum des Vertragsabschlusses.
    pub contract_date: String,
    /// Spezifikation des kryptografischen Löschbeweis-Mechanismus.
    pub deletion_proof_mechanism: String,
    /// Status des Cloud Egress Gateways (DLP & Exfiltrationsschutz).
    pub egress_gateway_active: bool,
    /// Modell der Mandantentrennung (z. B. "Logische und kryptografische Mandantenisolation").
    pub tenant_isolation_model: String,
}

/// Generiert einen vollständigen AVV-Entwurf im Markdown-Format.
///
/// Der Entwurf enthält alle nach Art. 28 DSGVO geforderten Abschnitte und referenziert
/// die technischen Garantien aus dem übergebenen `AvvContext`.
pub fn generate_avv_draft(ctx: &AvvContext) -> String {
    let egress_status = if ctx.egress_gateway_active {
        "Aktiviert (Egress Gateway filtert und blockiert unbefugte Datenabflüsse automatisiert)"
    } else {
        "Inaktiviert"
    };

    format!(
        r#"# Vereinbarung zur Auftragsverarbeitung (AVV) gemäß Art. 28 DSGVO

> **Hinweis / Legal Disclaimer:**
> Dieser Entwurf ersetzt keine Rechtsberatung und muss vor Einsatz von einem Juristen geprüft werden.

---

## Vertragsparteien

- **Verantwortlicher (Auftraggeber):** {controller}
- **Auftragsverarbeiter (Auftragnehmer):** {processor}
- **Vertragsdatum:** {contract_date}

---

## § 1 Gegenstand und Dauer

1. Dieser Vertrag regelt die Rechte und Pflichten der Vertragsparteien im Rahmen der Auftragsverarbeitung von personenbezogenen Daten durch den Auftragnehmer für den Auftraggeber.
2. Die Dauer dieser Vereinbarung richtet sich nach der Laufzeit des Hauptvertrages zwischen den Parteien.

## § 2 Art und Zweck der Verarbeitung

1. Die Verarbeitung erfolgt ausschließlich zum Zweck der Bereitstellung und des Betriebs des Contextra-Systems (Kontextspeicher, Suchen und Retrieval-Dienste).
2. Der Auftragnehmer verarbeitet personenbezogene Daten im Auftrag und nach Weisung des Auftraggebers.

## § 3 Art der personenbezogenen Daten

Die im Rahmen der Auftragsverarbeitung verarbeiteten Datenkategorien umfassen:
- Stammdaten und Kontaktdaten von Systemnutzern
- Inhaltsdaten und Freitext-Anfragen im Kontextspeicher
- System- und Zugriffsprotokolle

## § 4 Kategorien betroffener Personen

Die betroffenen Personenkreise umfassen:
- Beschäftigte und Mitarbeiter des Auftraggebers
- Kunden, Partner und Endnutzer des Auftraggebers

## § 5 Pflichten und Rechte des Verantwortlichen

1. Der Auftraggeber ist für die Beurteilung der Zulässigkeit der Verarbeitung sowie für die Wahrung der Rechte der betroffenen Personen verantwortlich.
2. Der Auftraggeber hat das Recht, ergänzende Weisungen bezüglich der Datenverarbeitung zu erteilen.

## § 6 Technische und organisatorische Maßnahmen (TOMs)

Der Auftragnehmer sichert zu, die folgenden produktspezifischen technischen Schutzmechanismen einzusetzen und aufrechtzuerhalten:

1. **Mandantentrennung:** {tenant_isolation}
2. **Cloud Egress Gateway:** {egress_status}
3. **Kryptografischer Löschbeweis:** {deletion_proof}

## § 7 Unterauftragsverhältnisse

1. Der Auftragnehmer darf Unterauftragnehmer nur nach vorheriger schriftlicher oder dokumentierter Zustimmung des Auftraggebers hinzuziehen.
2. Der Auftragnehmer schließt mit Unterauftragnehmern Vereinbarungen ab, die den Schutzstandards dieser Vereinbarung entsprechen.

## § 8 Unterstützungspflichten

1. Der Auftragnehmer unterstützt den Auftraggeber bei der Erfüllung von Anfragen betroffener Personen nach Kapitel III der DSGVO.
2. Der Auftragnehmer unterstützt den Auftraggeber bei der Einhaltung der Pflichten gemäß Art. 32 bis 36 DSGVO (Sicherheit, Meldung von Verletzungen, Datenschutz-Folgenabschätzung).

## § 9 Löschung und Rückgabe nach Vertragsende

1. Nach Abschluss der vertraglich vereinbarten Arbeiten oder nach Ablauf der Speicherfristen werden alle im Auftrag verarbeiteten Daten gelöscht oder zurückgegeben.
2. Die ordnungsgemäße und unwiderrufliche Löschung wird dem Auftraggeber über den definierten Mechanismus nachgewiesen:
   - **Löschnachweis-Verfahren:** {deletion_proof}

## § 10 Nachweispflichten

1. Der Auftragnehmer stellt dem Auftraggeber alle erforderlichen Informationen zum Nachweis der Einhaltung der in Art. 28 DSGVO niedergelegten Pflichten zur Verfügung.
2. Der Auftragnehmer ermöglicht Überprüfungen und Audits durch den Auftraggeber oder einen von diesem beauftragten Prüfer.
"#,
        controller = ctx.controller_name,
        processor = ctx.processor_name,
        contract_date = ctx.contract_date,
        tenant_isolation = ctx.tenant_isolation_model,
        egress_status = egress_status,
        deletion_proof = ctx.deletion_proof_mechanism,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_context() -> AvvContext {
        AvvContext {
            controller_name: "Acme Corp GmbH".to_string(),
            processor_name: "Contextra Cloud Systems AG".to_string(),
            contract_date: "2025-01-15".to_string(),
            deletion_proof_mechanism: "Kryptografischer Löschbeweis, siehe SECURITY.md".to_string(),
            egress_gateway_active: true,
            tenant_isolation_model: "Logische und kryptografische Mandantenisolation (TenantId-Scoped)"
                .to_string(),
        }
    }

    #[test]
    fn test_generate_avv_draft_contains_disclaimer() {
        let ctx = sample_context();
        let draft = generate_avv_draft(&ctx);

        let required_disclaimer =
            "Dieser Entwurf ersetzt keine Rechtsberatung und muss vor Einsatz von einem Juristen geprüft werden.";
        assert!(
            draft.contains(required_disclaimer),
            "AVV draft must contain exact legal disclaimer"
        );
    }

    #[test]
    fn test_generate_avv_draft_contains_all_art28_headings() {
        let ctx = sample_context();
        let draft = generate_avv_draft(&ctx);

        let required_headings = [
            "Gegenstand und Dauer",
            "Art und Zweck der Verarbeitung",
            "Art der personenbezogenen Daten",
            "Kategorien betroffener Personen",
            "Pflichten und Rechte des Verantwortlichen",
            "Technische und organisatorische Maßnahmen",
            "Unterauftragsverhältnisse",
            "Unterstützungspflichten",
            "Löschung und Rückgabe nach Vertragsende",
            "Nachweispflichten",
        ];

        for heading in &required_headings {
            assert!(
                draft.contains(heading),
                "AVV draft missing mandatory Art. 28 heading: '{heading}'"
            );
        }
    }

    #[test]
    fn test_generate_avv_draft_substitutes_context_fields() {
        let ctx = sample_context();
        let draft = generate_avv_draft(&ctx);

        assert!(draft.contains("Acme Corp GmbH"));
        assert!(draft.contains("Contextra Cloud Systems AG"));
        assert!(draft.contains("2025-01-15"));
        assert!(draft.contains("Kryptografischer Löschbeweis, siehe SECURITY.md"));
        assert!(draft.contains("Logische und kryptografische Mandantenisolation"));
        assert!(draft.contains("Aktiviert (Egress Gateway filtert und blockiert unbefugte Datenabflüsse automatisiert)"));
    }

    #[test]
    fn test_generate_avv_draft_with_inactive_egress() {
        let mut ctx = sample_context();
        ctx.egress_gateway_active = false;

        let draft = generate_avv_draft(&ctx);
        assert!(draft.contains("Inaktiviert"));
    }
}
