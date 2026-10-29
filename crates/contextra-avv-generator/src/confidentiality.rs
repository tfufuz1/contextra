//! Vertraulichkeitsverpflichtung gemäß Art. 28 Abs. 3 lit. b DSGVO.

use crate::error::AvvError;
use std::fmt::Write;

/// Renders Section 5: Confidentiality Commitment.
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
