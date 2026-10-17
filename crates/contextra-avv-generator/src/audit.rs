//! Art. 28(3)(h) GDPR - Nachweis- und Überprüfungsrechte.

use crate::error::AvvGeneratorError;
use std::fmt::Write;

/// Renders the section for Art. 28(3)(h) GDPR (Audit rights and evidence).
pub fn render_audit(out: &mut String) -> Result<(), AvvGeneratorError> {
    writeln!(out, "\n## 13. Nachweispflichten und Überprüfungsrechte (Art. 28 Abs. 3 lit. h DSGVO)").map_err(|e| AvvGeneratorError::RenderError(e.to_string()))?;
    writeln!(out, "\n[TEXT AUS JURISTISCH FREIGEGEBENER VORLAGE EINFÜGEN]\n").map_err(|e| AvvGeneratorError::RenderError(e.to_string()))?;
    writeln!(out, "Statutarische Anforderungen gemäß Art. 28 Abs. 3 lit. h DSGVO:").map_err(|e| AvvGeneratorError::RenderError(e.to_string()))?;
    writeln!(out, "- Bereitstellung aller erforderlichen Informationen zum Nachweis der Einhaltung der in Art. 28 DSGVO niedergelegten Pflichten.").map_err(|e| AvvGeneratorError::RenderError(e.to_string()))?;
    writeln!(out, "- Ermöglichung und Beitrag zu Überprüfungen und Inspektionen, die vom Verantwortlichen oder einem von diesem beauftragten Prüfer durchgeführt werden.").map_err(|e| AvvGeneratorError::RenderError(e.to_string()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_audit() {
        let mut out = String::new();
        render_audit(&mut out).expect("Rendering succeeds");
        assert!(out.contains("Art. 28 Abs. 3 lit. h DSGVO"));
        assert!(out.contains("[TEXT AUS JURISTISCH FREIGEGEBENER VORLAGE EINFÜGEN]"));
    }
}
