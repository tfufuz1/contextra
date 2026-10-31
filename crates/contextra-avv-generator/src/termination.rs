//! Löschung und Rückgabe nach Beendigung gemäß Art. 28 Abs. 3 lit. g DSGVO.
//!
//! # Zweck & Hintergrund
//! Regelt das Schicksal der Mandantendaten nach Beendigung des Hauptvertrages oder nach
//! gesonderter Löschanforderung.
//!
//! Gesetzlicher Kernpunkt nach Art. 28 Abs. 3 lit. g DSGVO:
//! Der Auftragsverarbeiter muss alle personenbezogenen Daten nach Wahl des Verantwortlichen
//! entweder löschen oder zurückgeben, sofern keine gesetzlichen Aufbewahrungsfristen entgegenstehen.
//!
//! Contextra verknüpft diese rechtliche Verpflichtung direkt mit messbaren technischen Garantien:
//! - Definierte Löschfrist (SLA = Service Level Agreement in Tagen).
//! - Optionaler kryptografischer Löschbeweis (Ed25519-Signatur), welcher die physikalische
//!   Entfernung der Daten aus Speicherschichten und Caches mathematisch nachweist.
//!
//! # Ring-Zugehörigkeit
//! Teil von Ring 4 (Erweiterung).

use crate::error::AvvError;
use crate::AvvContext;
use std::fmt::Write;

// ANCHOR[contextra-avv-generator:TERM-001]
/// Rendert Abschnitt 8: „Löschung von Daten und SLA“.
///
/// Schreibt die Löschklausel unter Einbindung der Mandanten-ID ([`contextra_types::TenantId`])
/// und des konfigurierten Lösch-SLAs in Tagen in den Zielpuffer.
///
/// # Parameter
/// - `out`: Der Puffer, in den der Markdown-Text geschrieben wird.
/// - `ctx`: Der Kontext mit dem vereinbarten Deletion-SLA (`deletion_sla_days`).
/// - `escaped_tenant_id`: Die bereits gegen Markdown-Injection maskierte Mandanten-ID.
///
/// # Ablauf
/// 1. Überschrift für Abschnitt 8 ausgeben.
/// 2. Absatz 1: Gesetzliche Grundpflicht zur Löschung oder Rückgabe nach Art. 28 Abs. 3 lit. g DSGVO.
/// 3. Absatz 2: Fristgebundene Löschzusage bezogen auf die konkrete Mandanten-ID (`escaped_tenant_id`).
/// 4. Absatz 3: Bereitstellung eines kryptografischen Ed25519-Löschbeweises.
///
/// # Fehler
/// Gibt [`AvvError::RenderError`] zurück, falls ein Formatierungsfehler auftritt.
pub fn render_section(
    out: &mut String,
    ctx: &AvvContext,
    escaped_tenant_id: &str,
) -> Result<(), AvvError> {
    writeln!(out, "\n## 8. Löschung von Daten und SLA")
        .map_err(|e| AvvError::RenderError(e.to_string()))?;
    writeln!(
        out,
        "1. Der Auftragsverarbeiter löscht oder gibt nach Abschluss der Erbringung der Verarbeitungsleistungen alle personenbezogenen Daten nach Wahl des Verantwortlichen zurück und löscht bestehende Kopien, sofern nicht nach dem Recht der Union oder der Mitgliedstaaten eine Verpflichtung zur Speicherung der personenbezogenen Daten besteht."
    )
    .map_err(|e| AvvError::RenderError(e.to_string()))?;
    writeln!(
        out,
        "2. Bei Beendigung der Leistungserbringung oder konkreter Löschanforderung löscht der Auftragsverarbeiter alle verarbeiteten Daten des Mandanten (`{}`) innerhalb des vereinbarten Deletion-SLA von **{} Tagen**.",
        escaped_tenant_id, ctx.deletion_sla_days
    )
    .map_err(|e| AvvError::RenderError(e.to_string()))?;
    writeln!(
        out,
        "3. Auf Wunsch stellt der Auftragsverarbeiter dem Verantwortlichen einen fälschungssicheren, kryptografischen Löschbeweis (Ed25519) zum Nachweis der unwiderruflichen Löschung bereit."
    )
    .map_err(|e| AvvError::RenderError(e.to_string()))?;
    Ok(())
}
