//! Weisungsbindung gemäß Art. 28 Abs. 3 lit. a DSGVO.

use crate::error::AvvError;
use std::fmt::Write;

/// Renders Section 4: Obligations and Rights of Controller & Instruction Binding.
pub fn render_section(out: &mut String) -> Result<(), AvvError> {
    writeln!(
        out,
        "\n## 4. Pflichten und Rechte des Verantwortlichen & Weisungsbindung"
    )
    .map_err(|e| AvvError::RenderError(e.to_string()))?;
    writeln!(
        out,
        "1. Der Verantwortliche ist für die Beurteilung der Zulässigkeit der Datenverarbeitung sowie für die Wahrung der Rechte der betroffenen Personen allein verantwortlich."
    )
    .map_err(|e| AvvError::RenderError(e.to_string()))?;
    writeln!(
        out,
        "2. Der Auftragsverarbeiter verarbeitet personenbezogene Daten ausschließlich auf dokumentierte Weisung des Verantwortlichen — einschließlich hinsichtlich der Übermittlung personenbezogener Daten an ein Drittland oder eine internationale Organisation —, sofern er nicht durch das Recht der Union oder der Mitgliedstaaten, dem der Auftragsverarbeiter unterliegt, hierzu verpflichtet ist."
    )
    .map_err(|e| AvvError::RenderError(e.to_string()))?;
    writeln!(
        out,
        "3. Der Auftragsverarbeiter informiert den Verantwortlichen unverzüglich, wenn er der Auffassung ist, dass eine Weisung gegen die DSGVO oder andere Datenschutzbestimmungen der Union oder der Mitgliedstaaten verstößt."
    )
    .map_err(|e| AvvError::RenderError(e.to_string()))?;
    Ok(())
}
