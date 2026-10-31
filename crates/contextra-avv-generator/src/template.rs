//! Vorlagen-Rendering für Vereinbarungen zur Auftragsverarbeitung (AVV) gemäß Art. 28 DSGVO.
//!
//! # Zweck & Architektur
//! Dieses Modul übernimmt die sequentielle Erstellung des Markdown-Dokuments aus einem
//! validierten [`AvvContext`]. Es orchestriert die einzelnen Klausel-Module:
//! - Kopfbereich mit Vertragspartnern und Mandanten-ID
//! - Gegenstand, Dauer, Art und Zweck der Verarbeitung (Art. 28 Abs. 3 lit. a-d DSGVO)
//! - Weisungsbindung ([`crate::instructions`])
//! - Vertraulichkeitsverpflichtung ([`crate::confidentiality`])
//! - Technische und organisatorische Maßnahmen (TOM, Art. 32 DSGVO)
//! - Unterauftragsverarbeiter (Art. 28 Abs. 2 und Abs. 4 DSGVO)
//! - Löschung, Rückgabe und Deletion-SLA ([`crate::termination`])
//! - Unterstützung bei Betroffenenrechten ([`crate::data_subject_rights`])
//! - Unterstützung bei Meldepflichten & DSFA ([`crate::assistance`])
//! - Nachweis- und Kontrollrechte ([`crate::audit`])
//!
//! # Sicherheit gegen Injection-Angriffe
//! Um zu verhindern, dass benutzereigene Strings (z. B. Mandantenname oder Notizen)
//! durch Markdown-Steuerzeichen wie `#`, `*`, `|` oder Zeilenumbrüche die Dokumentstruktur
//! aufbrechen oder verfälschen, werden alle Benutzereingaben vor dem Rendern durch
//! [`escape_markdown`] bereinigt und maskiert.
//!
//! # Ring-Zugehörigkeit
//! Teil von Ring 4 (Erweiterung). Reines String-Building im Arbeitsspeicher ohne I/O.

use crate::error::AvvError;
use crate::{
    assistance, audit, confidentiality, data_subject_rights, instructions, termination, AvvContext,
    TechnicalMeasure,
};
use std::fmt::Write;

// ANCHOR[contextra-avv-generator:TPL-001]
/// Maskiert Markdown-Steuerzeichen in Benutzereingaben zur Verhinderung von Formatierungs-Injections.
///
/// Ersetzt Zeichen mit spezieller Markdown-Bedeutung (wie `*`, `_`, `#`, `\`, `|` usw.)
/// durch deren vorangestellten Backslash (`\`) und wandelt harte Zeilenumbrüche (`\n`)
/// in Leerzeichen um, damit keine gefälschten Absätze oder Überschriften eingeschleust werden.
/// Carriage Returns (`\r`) werden ersatzlos verworfen.
///
/// # Ablauf
/// 1. Allokation eines Zielpuffers mit Kapazität `input.len() + 16`.
/// 2. Iteration über alle Zeichen des Eingabestrings:
///    - Bei Markdown-Sonderzeichen: Ausgabe von `\` gefolgt vom Zeichen.
///    - Bei `\n`: Ausgabe eines Leerzeichens `' '`.
///    - Bei `\r`: Überspringen.
///    - Bei allen anderen Zeichen: unverändertes Anfügen.
/// 3. Rückgabe des bereinigten Strings.
///
/// # Beispiel
/// ```rust
/// use contextra_avv_generator::template::escape_markdown;
///
/// let raw = "Kanzlei *Müller* & Co\n## Hacker-Titel";
/// let safe = escape_markdown(raw);
/// assert_eq!(safe, r"Kanzlei \*Müller\* & Co \#\# Hacker\-Titel");
/// ```
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

// ANCHOR[contextra-avv-generator:TPL-002]
/// Rendert das AVV-Markdown-Dokument basierend auf den Parametern in [`AvvContext`].
///
/// Falls kein explizites Vertragsdatum in `ctx.contract_date` hinterlegt ist,
/// wird der Standardwert „gemäß Ausstellungsdatum“ verwendet.
///
/// # Ablauf
/// 1. Ermittlung der Datumszeichenkette (entweder aus `ctx.contract_date` oder Fallback).
/// 2. Aufruf von [`render_with_date`].
///
/// # Fehler
/// Gibt [`AvvError::RenderError`] zurück, wenn die Pufferformatierung fehlschlägt.
pub fn render(ctx: &AvvContext) -> Result<String, AvvError> {
    let date_str = ctx
        .contract_date
        .as_deref()
        .unwrap_or("gemäß Ausstellungsdatum");
    render_with_date(ctx, date_str)
}

// ANCHOR[contextra-avv-generator:TPL-003]
/// Rendert das AVV-Markdown-Dokument mit einer expliziten Datumsangabe.
///
/// # Ablauf
/// 1. Allokation eines Ergebnis-Puffers mit einer Startkapazität von 4096 Bytes.
/// 2. Rendern des Kopfbereichs (`render_header`).
/// 3. Rendern von Abschnitt 1 (Gegenstand & Dauer, `render_scope_and_duration`).
/// 4. Rendern von Abschnitt 2 (Art & Zweck, `render_nature_and_purpose`).
/// 5. Rendern von Abschnitt 3 (Datenkategorien & Betroffene, `render_data_and_subjects`).
/// 6. Rendern von Abschnitt 4 (Weisungsbindung via [`instructions::render_section`]).
/// 7. Rendern von Abschnitt 5 (Vertraulichkeit via [`confidentiality::render_section`]).
/// 8. Rendern von Abschnitt 6 (TOMs, `render_technical_measures`).
/// 9. Rendern von Abschnitt 7 (Unterauftragnehmer, `render_subprocessors`).
/// 10. Rendern von Abschnitt 8 (Löschung & Deletion-SLA via [`termination::render_section`]).
/// 11. Rendern von Abschnitt 9 (Betroffenenrechte via [`data_subject_rights::render_section`]).
/// 12. Rendern von Abschnitt 10 (Sicherheit & Unterstützung via [`assistance::render_section`]).
/// 13. Rendern von Abschnitt 11 (Audit- & Nachweisrechte via [`audit::render_section`]).
/// 14. Rückgabe des zusammengesetzten Markdown-Textes.
///
/// # Fehler
/// Gibt [`AvvError::RenderError`] zurück, wenn ein Schreibvorgang in den String fehlschlägt.
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

// ANCHOR[contextra-avv-generator:TPL-004]
// Rendert den Dokumentkopf mit Titel, rechtlichem Haftungsausschluss (Disclaimer),
// den Vertragsparteien (Verantwortlicher, Verarbeiter), der Mandanten-ID und dem Vertragsdatum.
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

// ANCHOR[contextra-avv-generator:TPL-005]
// Rendert Abschnitt 1: Gegenstand (Nutzung der Contextra RAG- & Vektordatenbank)
// sowie Dauer der Verarbeitung (gekoppelt an den Hauptvertrag).
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

// ANCHOR[contextra-avv-generator:TPL-006]
// Rendert Abschnitt 2: Art und Zweck der Verarbeitung
// (automatisierte Vektorisierung, Indexierung, RAG = Retrieval-Augmented Generation).
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

// ANCHOR[contextra-avv-generator:TPL-007]
// Rendert Abschnitt 3: Kategorien der verarbeiteten personenbezogenen Daten
// und die Kategorien betroffener Personen (Kunden, Mitarbeiter, Mandanten usw.).
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

// ANCHOR[contextra-avv-generator:TPL-008]
// Rendert Abschnitt 6: Technische und organisatorische Maßnahmen (TOM) nach Art. 32 DSGVO.
// Gibt entweder eine Hinweisklausel bei leerer Liste aus oder listet jede Maßnahme mit
// Überschrift, gesetzlichem Referenzartikel und Detailbeschreibung auf.
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

// ANCHOR[contextra-avv-generator:TPL-009]
// Rendert Abschnitt 7: Regelungen für Unterauftragsverarbeiter nach Art. 28 Abs. 2 und 4 DSGVO.
// Definiert die Genehmigungspflicht, Informationspflicht bei Wechseln und listet
// entweder bestehende Unterauftragnehmer auf oder stellt klar, dass keine Dritten eingesetzt werden.
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
