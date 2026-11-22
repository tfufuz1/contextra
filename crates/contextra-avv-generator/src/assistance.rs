//! Unterstützung bei Art. 32–36 DSGVO gemäß Art. 28 Abs. 3 lit. f DSGVO.

use crate::error::AvvError;
use std::fmt::Write;

/// Renders Section 10: Assistance with Security and Compliance Obligations (Art. 32–36 GDPR).
pub fn render_section(out: &mut String) -> Result<(), AvvError> {
    writeln!(
        out,
        "\n## 10. Unterstützung bei Einhaltung der Pflichten gemäß Art. 32–36 DSGVO"
    )
    .map_err(|e| AvvError::RenderError(e.to_string()))?;
    writeln!(
        out,
        "1. Der Auftragsverarbeiter unterstützt den Verantwortlichen unter Berücksichtigung der Art der Verarbeitung und der ihm zur Verfügung stehenden Informationen bei der Einhaltung der in den Artikeln 32 bis 36 DSGVO genannten Pflichten:"
    )
    .map_err(|e| AvvError::RenderError(e.to_string()))?;
    writeln!(
        out,
        "   - **Art. 32 DSGVO:** Gewährleistung der Sicherheit der Verarbeitung durch geeignete technische und organisatorische Maßnahmen."
    )
    .map_err(|e| AvvError::RenderError(e.to_string()))?;
    writeln!(
        out,
        "   - **Art. 33 & 34 DSGVO:** Unverzügliche Meldung von Verletzungen des Schutzes personenbezogener Daten an den Verantwortlichen sowie Unterstützung bei der Benachrichtigung betroffener Personen."
    )
    .map_err(|e| AvvError::RenderError(e.to_string()))?;
    writeln!(
        out,
        "   - **Art. 35 & 36 DSGVO:** Unterstützung bei der Durchführung von Datenschutz-Folgenabschätzungen und eventuellen vorherigen Konsultationen der Aufsichtsbehörde."
    )
    .map_err(|e| AvvError::RenderError(e.to_string()))?;
    Ok(())
}
