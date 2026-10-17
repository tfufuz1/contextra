//! Art. 28(3)(f) GDPR - Unterstützung bei der Einhaltung der Pflichten nach Art. 32–36 DSGVO.

use crate::error::AvvGeneratorError;
use std::fmt::Write;

/// Renders the section for Art. 28(3)(f) GDPR (Assistance with Art. 32-36 compliance).
pub fn render_assistance(out: &mut String) -> Result<(), AvvGeneratorError> {
    writeln!(out, "\n## 11. Unterstützung bei Sicherheit der Verarbeitung und Meldepflichten (Art. 28 Abs. 3 lit. f DSGVO)").map_err(|e| AvvGeneratorError::RenderError(e.to_string()))?;
    writeln!(out, "\n[TEXT AUS JURISTISCH FREIGEGEBENER VORLAGE EINFÜGEN]\n").map_err(|e| AvvGeneratorError::RenderError(e.to_string()))?;
    writeln!(out, "Statutarische Anforderungen gemäß Art. 28 Abs. 3 lit. f DSGVO:").map_err(|e| AvvGeneratorError::RenderError(e.to_string()))?;
    writeln!(out, "- Unterstützung des Verantwortlichen bei der Gewährleistung der Sicherheit der Verarbeitung (Art. 32 DSGVO).").map_err(|e| AvvGeneratorError::RenderError(e.to_string()))?;
    writeln!(out, "- Unverzügliche Meldung von Verletzungen des Schutzes personenbezogener Daten (Art. 33 DSGVO).").map_err(|e| AvvGeneratorError::RenderError(e.to_string()))?;
    writeln!(out, "- Unterstützung bei der Benachrichtigung der von einer Verletzung betroffenen Person (Art. 34 DSGVO).").map_err(|e| AvvGeneratorError::RenderError(e.to_string()))?;
    writeln!(out, "- Unterstützung bei Datenschutz-Folgenabschätzungen (Art. 35 DSGVO) und vorherigen Konsultationen (Art. 36 DSGVO).").map_err(|e| AvvGeneratorError::RenderError(e.to_string()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_assistance() {
        let mut out = String::new();
        render_assistance(&mut out).expect("Rendering succeeds");
        assert!(out.contains("Art. 28 Abs. 3 lit. f DSGVO"));
        assert!(out.contains("[TEXT AUS JURISTISCH FREIGEGEBENER VORLAGE EINFÜGEN]"));
    }
}
