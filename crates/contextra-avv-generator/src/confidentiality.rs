//! Vertraulichkeitsverpflichtung gemäß Art. 28 Abs. 3 lit. b DSGVO.
//!
//! # Zweck & Hintergrund
//! Dieser Abschnitt sichert zu, dass sämtliche Personen, die beim Auftragsverarbeiter
//! Zugriff auf personenbezogene Daten erhalten (z. B. Administratoren, Support-Mitarbeiter),
//! zur Vertraulichkeit verpflichtet sind.
//!
//! Gesetzlicher Kernpunkt nach Art. 28 Abs. 3 lit. b DSGVO:
//! Die Befugten müssen entweder vertraglich zur Geheimhaltung verpflichtet worden sein
//! oder einer gesetzlichen Verschwiegenheitspflicht (z. B. Berufsgeheimnis) unterliegen.
//!
//! # Ring-Zugehörigkeit
//! Teil von Ring 4 (Erweiterung). Reines Text-Rendering ohne Nebenwirkungen.

use crate::error::AvvError;
use std::fmt::Write;

// ANCHOR[contextra-avv-generator:CONF-001]
/// Rendert Abschnitt 5: „Vertraulichkeitsverpflichtung“.
///
/// Schreibt den Textblock zur Wahrung des Datengeheimnisses in den Markdown-Ausgabepuffer.
///
/// # Ablauf
/// 1. Überschrift für Abschnitt 5 ausgeben.
/// 2. Absatz 1: Zusicherung der förmlichen Vertraulichkeitsverpflichtung des Personals.
/// 3. Absatz 2: Klarstellung über den Fortbestand der Pflicht auch nach Vertragsende.
///
/// # Fehler
/// Gibt [`AvvError::RenderError`] zurück, falls Schreiboperationen in den Puffer fehlschlagen.
pub fn render_section(out: &mut String) -> Result<(), AvvError> {
    writeln!(out, "\n## 5. Vertraulichkeitsverpflichtung")
        .map_err(|e| AvvError::RenderError(e.to_string()))?;
    writeln!(
        out,
        "1. Der Auftragsverarbeiter gewährleistet, dass sich die zur Verarbeitung der personenbezogenen Daten befugten Personen zur Vertraulichkeit verpflichtet haben oder einer angemessenen gesetzlichen Verschwiegenheitspflicht unterliegen."
    )
    .map_err(|e| AvvError::RenderError(e.to_string()))?;
    writeln!(
        out,
        "2. Die Vertraulichkeitsverpflichtung besteht auch nach Beendigung des Auftragsverarbeitungsvertrages fort."
    )
    .map_err(|e| AvvError::RenderError(e.to_string()))?;
    Ok(())
}
