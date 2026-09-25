//! Template rendering module for AVV (Auftragsverarbeitungsvertrag) according to Art. 28 DSGVO.

use crate::error::AvvGeneratorError;
use crate::{AvvContext, TechnicalMeasure};
use std::fmt::Write;

/// Renders the AVV Markdown document from the provided [`AvvContext`].
pub fn render(ctx: &AvvContext) -> Result<String, AvvGeneratorError> {
    if ctx.controller_name.trim().is_empty() {
        return Err(AvvGeneratorError::InvalidContext(
            "Verantwortlicher (controller_name) darf nicht leer sein.".to_string(),
        ));
    }
    if ctx.processor_name.trim().is_empty() {
        return Err(AvvGeneratorError::InvalidContext(
            "Auftragsverarbeiter (processor_name) darf nicht leer sein.".to_string(),
        ));
    }

    let mut out = String::with_capacity(2048);

    writeln!(
        out,
        "# Vereinbarung zur Auftragsverarbeitung (AVV) gemäß Art. 28 DSGVO"
    )
    .map_err(|e| AvvGeneratorError::RenderError(e.to_string()))?;

    writeln!(out, "\n**Verantwortlicher (Auftraggeber):** {}", ctx.controller_name)
        .map_err(|e| AvvGeneratorError::RenderError(e.to_string()))?;
    writeln!(
        out,
        "**Auftragsverarbeiter (Auftragnehmer):** {}",
        ctx.processor_name
    )
    .map_err(|e| AvvGeneratorError::RenderError(e.to_string()))?;
    writeln!(out, "**Mandanten-ID (TenantId):** {}", ctx.tenant_id)
        .map_err(|e| AvvGeneratorError::RenderError(e.to_string()))?;

    writeln!(out, "\n---").map_err(|e| AvvGeneratorError::RenderError(e.to_string()))?;

    // 1. Gegenstand und Dauer
    writeln!(out, "\n## 1. Gegenstand und Dauer der Verarbeitung").map_err(|e| AvvGeneratorError::RenderError(e.to_string()))?;
    writeln!(
        out,
        "Gegenstand der Verarbeitung ist die Bereitstellung und Nutzung des Contextra RAG- & Vektordatenbank-Systems zur hochsicheren Speicherung, Indizierung und semantischen Abfrage von Dokumenten und Kontexten."
    )
    .map_err(|e| AvvGeneratorError::RenderError(e.to_string()))?;
    writeln!(
        out,
        "Die Dauer der Verarbeitung entspricht der Laufzeit des Hauptvertrages zwischen dem Verantwortlichen und dem Auftragsverarbeiter."
    )
    .map_err(|e| AvvGeneratorError::RenderError(e.to_string()))?;

    // 2. Art und Zweck
    writeln!(out, "\n## 2. Art und Zweck der Verarbeitung").map_err(|e| AvvGeneratorError::RenderError(e.to_string()))?;
    writeln!(
        out,
        "Die Verarbeitung umfasst die automatisierte Vektorisierung, Speicherung, semantische Indexierung und den Abruf von Text- und Wissensdaten zum Zweck des wissensbasierten Kontext-Retrievals (RAG) für den Verantwortlichen."
    )
    .map_err(|e| AvvGeneratorError::RenderError(e.to_string()))?;

    // 3. Art der Daten & Betroffene
    writeln!(
        out,
        "\n## 3. Art der personenbezogenen Daten und Kategorien betroffener Personen"
    )
    .map_err(|e| AvvGeneratorError::RenderError(e.to_string()))?;
    writeln!(
        out,
        "**Art der personenbezogenen Daten:** In Dokumenten und Texten enthaltene Personenstammdaten, Kommunikationsdaten, Metadaten sowie geschäftsspezifische Inhalte, die im Vektorspeicher abgelegt werden."
    )
    .map_err(|e| AvvGeneratorError::RenderError(e.to_string()))?;
    writeln!(
        out,
        "**Kategorien betroffener Personen:** Kunden, Beschäftigte, Mandanten, Lieferanten und Vertragspartner des Verantwortlichen."
    )
    .map_err(|e| AvvGeneratorError::RenderError(e.to_string()))?;

    // 4. Pflichten und Rechte des Verantwortlichen
    writeln!(out, "\n## 4. Pflichten und Rechte des Verantwortlichen").map_err(|e| AvvGeneratorError::RenderError(e.to_string()))?;
    writeln!(
        out,
        "Der Verantwortliche ist für die Beurteilung der Zulässigkeit der Datenverarbeitung sowie für die Wahrung der Rechte der betroffenen Personen allein verantwortlich. Er ist berechtigt, Weisungen bezüglich der Datenverarbeitung zu erteilen."
    )
    .map_err(|e| AvvGeneratorError::RenderError(e.to_string()))?;

    // 5. Technische und organisatorische Maßnahmen (TOM)
    writeln!(
        out,
        "\n## 5. Technische und organisatorische Maßnahmen (TOM)"
    )
    .map_err(|e| AvvGeneratorError::RenderError(e.to_string()))?;
    writeln!(
        out,
        "Der Auftragsverarbeiter garantiert die Umsetzung folgender produktimmanenter technischer Sicherheitsgarantien gemäß Art. 32 DSGVO:"
    )
    .map_err(|e| AvvGeneratorError::RenderError(e.to_string()))?;

    if ctx.technical_measures.is_empty() {
        writeln!(
            out,
            "\n* Keine spezifischen technischen Maßnahmen konfiguriert."
        )
        .map_err(|e| AvvGeneratorError::RenderError(e.to_string()))?;
    } else {
        for measure in &ctx.technical_measures {
            format_technical_measure(&mut out, measure)?;
        }
    }

    // 6. Unterauftragsverarbeiter
    writeln!(out, "\n## 6. Unterauftragsverarbeiter").map_err(|e| AvvGeneratorError::RenderError(e.to_string()))?;
    if ctx.subprocessors.is_empty() {
        writeln!(
            out,
            "Keine Unterauftragsverarbeiter. Die Verarbeitung erfolgt ausschließlich auf eigenen Systemen des Auftragsverarbeiters ohne Einbindung Dritter."
        )
        .map_err(|e| AvvGeneratorError::RenderError(e.to_string()))?;
    } else {
        writeln!(
            out,
            "Der Verantwortliche stimmt dem Einsatz folgender Unterauftragsverarbeiter zu:"
        )
        .map_err(|e| AvvGeneratorError::RenderError(e.to_string()))?;
        for subp in &ctx.subprocessors {
            writeln!(out, "- {}", subp).map_err(|e| AvvGeneratorError::RenderError(e.to_string()))?;
        }
    }

    // 7. Löschung von Daten und SLA
    writeln!(out, "\n## 7. Löschung von Daten und SLA").map_err(|e| AvvGeneratorError::RenderError(e.to_string()))?;
    writeln!(
        out,
        "Nach Beendigung der Leistungserbringung oder bei konkreter Löschanforderung löscht der Auftragsverarbeiter alle verarbeiteten Daten des Mandanten (`{}`) innerhalb des vereinbarten Deletion-SLA von **{} Tagen**. Auf Wunsch wird dem Verantwortlichen ein kryptografischer Löschbeweis übermittelt.",
        ctx.tenant_id, ctx.deletion_sla_days
    )
    .map_err(|e| AvvGeneratorError::RenderError(e.to_string()))?;

    Ok(out)
}

fn format_technical_measure(out: &mut String, measure: &TechnicalMeasure) -> Result<(), AvvGeneratorError> {
    writeln!(
        out,
        "\n### {} ({})",
        measure.name, measure.reference_article
    )
    .map_err(|e| AvvGeneratorError::RenderError(e.to_string()))?;
    writeln!(out, "{}", measure.description)
        .map_err(|e| AvvGeneratorError::RenderError(e.to_string()))?;
    Ok(())
}
