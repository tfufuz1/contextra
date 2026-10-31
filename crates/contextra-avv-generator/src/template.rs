//! Template rendering module for AVV (Auftragsverarbeitungsvertrag) according to Art. 28 DSGVO.

use crate::error::AvvError;
use crate::{
    assistance, audit, confidentiality, data_subject_rights, instructions, termination, AvvContext,
    TechnicalMeasure,
};
use std::fmt::Write;

/// Escapes special Markdown control characters in user inputs to prevent structural injection.
pub fn escape_markdown(input: &str) -> String {
    let mut escaped = String::with_capacity(input.len() + 16);
    for c in input.chars() {
        match c {
            '\\' | '`' | '*' | '_' | '{' | '}' | '[' | ']' | '(' | ')' | '#' | '+' | '-' | '!'
            | '|' | '<' | '>' => {
                escaped.push('\\');
                escaped.push(c);
            }
            '\n' => escaped.push(' '),
            '\r' => {}
            _ => escaped.push(c),
        }
    }
    escaped
}

/// Renders the AVV Markdown document from the provided [`AvvContext`].
pub fn render(ctx: &AvvContext) -> Result<String, AvvError> {
    let date_str = ctx
        .contract_date
        .as_deref()
        .unwrap_or("gemäß Ausstellungsdatum");
    render_with_date(ctx, date_str)
}

/// Renders the AVV Markdown document with an explicit date string.
pub fn render_with_date(ctx: &AvvContext, date: &str) -> Result<String, AvvError> {
    let mut out = String::with_capacity(4096);

    render_header(&mut out, ctx, date)?;
    render_scope_and_duration(&mut out)?;
    render_nature_and_purpose(&mut out)?;
    render_data_and_subjects(&mut out)?;
    instructions::render_section(&mut out)?;
    confidentiality::render_section(&mut out)?;
    render_technical_measures(&mut out, &ctx.technical_measures)?;
    render_subprocessors(&mut out, &ctx.subprocessors)?;
    termination::render_section(&mut out, ctx, &escape_markdown(&ctx.tenant_id.to_string()))?;
    data_subject_rights::render_section(&mut out)?;
    assistance::render_section(&mut out)?;
    audit::render_section(&mut out)?;

    Ok(out)
}

fn render_header(out: &mut String, ctx: &AvvContext, date: &str) -> Result<(), AvvError> {
    writeln!(
        out,
        "# Vereinbarung zur Auftragsverarbeitung (AVV) gemäß Art. 28 DSGVO"
    )
    .map_err(|e| AvvError::RenderError(e.to_string()))?;

    writeln!(
        out,
        "\n> **Hinweis:** Entwurf — ersetzt keine Rechtsberatung."
    )
    .map_err(|e| AvvError::RenderError(e.to_string()))?;

    writeln!(
        out,
        "\n**Verantwortlicher (Auftraggeber):** {}",
        escape_markdown(&ctx.controller_name)
    )
    .map_err(|e| AvvError::RenderError(e.to_string()))?;
    writeln!(
        out,
        "**Auftragsverarbeiter (Auftragnehmer):** {}",
        escape_markdown(&ctx.processor_name)
    )
    .map_err(|e| AvvError::RenderError(e.to_string()))?;
    writeln!(
        out,
        "**Mandanten-ID (TenantId):** {}",
        escape_markdown(&ctx.tenant_id.to_string())
    )
    .map_err(|e| AvvError::RenderError(e.to_string()))?;
    writeln!(out, "**Vertragsdatum:** {}", escape_markdown(date))
        .map_err(|e| AvvError::RenderError(e.to_string()))?;

    writeln!(out, "\n---").map_err(|e| AvvError::RenderError(e.to_string()))?;
    Ok(())
}

fn render_scope_and_duration(out: &mut String) -> Result<(), AvvError> {
    writeln!(out, "\n## 1. Gegenstand und Dauer der Verarbeitung")
        .map_err(|e| AvvError::RenderError(e.to_string()))?;
    writeln!(
        out,
        "Gegenstand der Verarbeitung ist die Bereitstellung und Nutzung des Contextra RAG- & Vektordatenbank-Systems zur hochsicheren Speicherung, Indizierung und semantischen Abfrage von Dokumenten und Kontexten."
    )
    .map_err(|e| AvvError::RenderError(e.to_string()))?;
    writeln!(
        out,
        "Die Dauer der Verarbeitung entspricht der Laufzeit des Hauptvertrages zwischen dem Verantwortlichen und dem Auftragsverarbeiter."
    )
    .map_err(|e| AvvError::RenderError(e.to_string()))?;
    Ok(())
}

fn render_nature_and_purpose(out: &mut String) -> Result<(), AvvError> {
    writeln!(out, "\n## 2. Art und Zweck der Verarbeitung")
        .map_err(|e| AvvError::RenderError(e.to_string()))?;
    writeln!(
        out,
        "Die Verarbeitung umfasst die automatisierte Vektorisierung, Speicherung, semantische Indexierung und den Abruf von Text- und Wissensdaten zum Zweck des wissensbasierten Kontext-Retrievals (RAG) für den Verantwortlichen."
    )
    .map_err(|e| AvvError::RenderError(e.to_string()))?;
    Ok(())
}

fn render_data_and_subjects(out: &mut String) -> Result<(), AvvError> {
    writeln!(
        out,
        "\n## 3. Art der personenbezogenen Daten und Kategorien betroffener Personen"
    )
    .map_err(|e| AvvError::RenderError(e.to_string()))?;
    writeln!(
        out,
        "**Art der personenbezogenen Daten:** In Dokumenten und Texten enthaltene Personenstammdaten, Kommunikationsdaten, Metadaten sowie geschäftsspezifische Inhalte, die im Vektorspeicher abgelegt werden."
    )
    .map_err(|e| AvvError::RenderError(e.to_string()))?;
    writeln!(
        out,
        "**Kategorien betroffener Personen:** Kunden, Beschäftigte, Mandanten, Lieferanten und Vertragspartner des Verantwortlichen."
    )
    .map_err(|e| AvvError::RenderError(e.to_string()))?;
    Ok(())
}

fn render_technical_measures(
    out: &mut String,
    measures: &[TechnicalMeasure],
) -> Result<(), AvvError> {
    writeln!(
        out,
        "\n## 6. Technische und organisatorische Maßnahmen (TOM)"
    )
    .map_err(|e| AvvError::RenderError(e.to_string()))?;
    writeln!(
        out,
        "Der Auftragsverarbeiter garantiert die Umsetzung folgender produktimmanenter technischer Sicherheitsgarantien gemäß Art. 32 DSGVO:"
    )
    .map_err(|e| AvvError::RenderError(e.to_string()))?;

    if measures.is_empty() {
        writeln!(
            out,
            "\n* Keine spezifischen technischen Maßnahmen konfiguriert."
        )
        .map_err(|e| AvvError::RenderError(e.to_string()))?;
    } else {
        for measure in measures {
            writeln!(
                out,
                "\n### {} ({})",
                escape_markdown(&measure.name),
                escape_markdown(&measure.reference_article)
            )
            .map_err(|e| AvvError::RenderError(e.to_string()))?;
            writeln!(out, "{}", escape_markdown(&measure.description))
                .map_err(|e| AvvError::RenderError(e.to_string()))?;
        }
    }
    Ok(())
}

fn render_subprocessors(out: &mut String, subprocessors: &[String]) -> Result<(), AvvError> {
    writeln!(out, "\n## 7. Unterauftragsverarbeiter")
        .map_err(|e| AvvError::RenderError(e.to_string()))?;
    writeln!(
        out,
        "1. Der Auftragsverarbeiter darf Unterauftragnehmer nur nach vorheriger schriftlicher oder dokumentierter Genehmigung des Verantwortlichen hinzuziehen."
    )
    .map_err(|e| AvvError::RenderError(e.to_string()))?;
    writeln!(
        out,
        "2. Der Auftragsverarbeiter informiert den Verantwortlichen immer über jede beabsichtigte Änderung bezüglich der Hinzuziehung oder Ersetzung anderer Unterauftragsverarbeiter. Der Verantwortliche kann gegen derartige Änderungen Einspruch erheben."
    )
    .map_err(|e| AvvError::RenderError(e.to_string()))?;

    if subprocessors.is_empty() {
        writeln!(
            out,
            "3. Keine Unterauftragsverarbeiter. Die Verarbeitung erfolgt ausschließlich auf eigenen Systemen des Auftragsverarbeiters ohne Einbindung Dritter."
        )
        .map_err(|e| AvvError::RenderError(e.to_string()))?;
    } else {
        writeln!(
            out,
            "3. Der Verantwortliche stimmt dem Einsatz folgender Unterauftragsverarbeiter zu:"
        )
        .map_err(|e| AvvError::RenderError(e.to_string()))?;
        for subp in subprocessors {
            writeln!(out, "- {}", escape_markdown(subp))
                .map_err(|e| AvvError::RenderError(e.to_string()))?;
        }
    }
    Ok(())
}
