//! Unterstützung bei Betroffenenrechten gemäß Art. 28 Abs. 3 lit. e DSGVO.

use crate::error::AvvError;
use std::fmt::Write;

/// Renders Section 9: Support for Data Subject Rights (Chapter III GDPR).
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
