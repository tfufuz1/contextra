//! Unterstützung bei Betroffenenrechten gemäß Art. 28 Abs. 3 lit. e DSGVO.
//!
//! # Zweck & Hintergrund
//! Dieser Abschnitt spezifiziert die Pflicht des Auftragsverarbeiters, den Verantwortlichen
//! bei Anfragen betroffener Personen nach Kapitel III der DSGVO zu unterstützen.
//!
//! Gesetzlicher Kernpunkt nach Art. 28 Abs. 3 lit. e DSGVO:
//! Da der Verantwortliche primärer Adressat der Betroffenenrechte (Auskunft, Berichtigung,
//! Löschung, Einschränkung, Übertragbarkeit, Widerspruch) ist, muss der Verarbeiter technische
//! und organisatorische Hilfestellungen leisten und eingehende Direktanträge unverzüglich weiterleiten.
//!
//! # Ring-Zugehörigkeit
//! Teil von Ring 4 (Erweiterung).

use crate::error::AvvError;
use std::fmt::Write;

// ANCHOR[contextra-avv-generator:DSR-001]
/// Rendert Abschnitt 9: „Unterstützung bei Betroffenenrechten“.
///
/// Schreibt die Unterstützungszusagen für Rechte betroffener Personen nach Kapitel III DSGVO in den Puffer.
///
/// # Ablauf
/// 1. Überschrift für Abschnitt 9 ausgeben.
/// 2. Absatz 1: Zusage technischer und organisatorischer Hilfestellung für Betroffenenanträge.
/// 3. Absatz 2: Verpflichtung zur unverzüglichen Weiterleitung direkt eingehender Betroffenenanträge.
///
/// # Fehler
/// Gibt [`AvvError::RenderError`] zurück, falls Schreiboperationen in den Puffer fehlschlagen.
pub fn render_section(out: &mut String) -> Result<(), AvvError> {
    writeln!(out, "\n## 9. Unterstützung bei Betroffenenrechten")
        .map_err(|e| AvvError::RenderError(e.to_string()))?;
    writeln!(
        out,
        "1. Der Auftragsverarbeiter unterstützt den Verantwortlichen unter Berücksichtigung der Art der Verarbeitung nach Möglichkeit mit geeigneten technischen und organisatorischen Maßnahmen bei der Erfüllung von dessen Pflicht zur Beantwortung von Anträgen auf Wahrnehmung der in Kapitel III der DSGVO genannten Rechte der betroffenen Person (z. B. Auskunft, Berichtigung, Löschung, Einschränkung der Verarbeitung, Datenübertragbarkeit)."
    )
    .map_err(|e| AvvError::RenderError(e.to_string()))?;
    writeln!(
        out,
        "2. Wendet sich eine betroffene Person direkt an den Auftragsverarbeiter, wird dieser den Antrag unverzüglich an den Verantwortlichen weiterleiten."
    )
    .map_err(|e| AvvError::RenderError(e.to_string()))?;
    Ok(())
}
