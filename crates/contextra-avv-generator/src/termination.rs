//! Löschung und Rückgabe nach Beendigung gemäß Art. 28 Abs. 3 lit. g DSGVO.

use crate::error::AvvError;
use crate::AvvContext;
use std::fmt::Write;

/// Renders Section 8: Deletion of Data and Deletion SLA after termination.
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
