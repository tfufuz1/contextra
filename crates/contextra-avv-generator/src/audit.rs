//! Nachweis- und Kontrollrechte gemäß Art. 28 Abs. 3 lit. h DSGVO.

use crate::error::AvvError;
use std::fmt::Write;

/// Renders Section 11: Information and Audit Rights.
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
