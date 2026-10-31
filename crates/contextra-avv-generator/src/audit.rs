//! Nachweis- und Kontrollrechte gemäß Art. 28 Abs. 3 lit. h DSGVO.
//!
//! # Zweck & Hintergrund
//! Dieser Abschnitt sichert dem Auftraggeber die gesetzlich vorgeschriebenen Informations-,
//! Überprüfungs- und Inspektionsrechte zu.
//!
//! Gesetzlicher Kernpunkt nach Art. 28 Abs. 3 lit. h DSGVO:
//! Der Auftragsverarbeiter muss dem Verantwortlichen alle erforderlichen Informationen
//! zum Nachweis der Einhaltung der Pflichten nach Art. 28 bereitstellen und Überprüfungen
//! (einschließlich Inspektionen und Audits) durch den Auftraggeber oder von diesem
//! beauftragte Prüfer dulden und unterstützen.
//!
//! # Ring-Zugehörigkeit
//! Teil von Ring 4 (Erweiterung).

use crate::error::AvvError;
use std::fmt::Write;

// ANCHOR[contextra-avv-generator:AUD-001]
/// Rendert Abschnitt 11: „Nachweis- und Kontrollrechte“.
///
/// Schreibt die Klauseln zu Auskunftsrechten und Vor-Ort- bzw. Fern-Audits in den Ausgabepuffer.
///
/// # Ablauf
/// 1. Überschrift für Abschnitt 11 ausgeben.
/// 2. Absatz 1: Pflicht zur Bereitstellung aller notwendigen Nachweisdokumente.
/// 3. Absatz 2: Duldungs- und Mitwirkungspflicht bei Audits und Inspektionen.
///
/// # Fehler
/// Gibt [`AvvError::RenderError`] zurück, falls Schreiboperationen in den Puffer fehlschlagen.
pub fn render_section(out: &mut String) -> Result<(), AvvError> {
    writeln!(out, "\n## 11. Nachweis- und Kontrollrechte")
        .map_err(|e| AvvError::RenderError(e.to_string()))?;
    writeln!(
        out,
        "1. Der Auftragsverarbeiter stellt dem Verantwortlichen alle erforderlichen Informationen zum Nachweis der Einhaltung der in Art. 28 DSGVO niedergelegten Pflichten zur Verfügung."
    )
    .map_err(|e| AvvError::RenderError(e.to_string()))?;
    writeln!(
        out,
        "2. Der Auftragsverarbeiter ermöglicht Überprüfungen — einschließlich Inspektionen —, die vom Verantwortlichen oder einem anderen von diesem beauftragten Prüfer durchgeführt werden, und trägt zu diesen Überprüfungen bei."
    )
    .map_err(|e| AvvError::RenderError(e.to_string()))?;
    Ok(())
}
