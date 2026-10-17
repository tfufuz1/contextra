//! Art. 28(3)(a) GDPR - Weisungsgebundenheit der Verarbeitung.

use crate::error::AvvGeneratorError;
use std::fmt::Write;

/// Renders the section for Art. 28(3)(a) GDPR (Instruction-bound processing).
pub fn render_instructions(out: &mut String) -> Result<(), AvvGeneratorError> {
    writeln!(out, "\n## 8. Pflicht zur verarbeitungsbezogenen Weisung (Art. 28 Abs. 3 lit. a DSGVO)").map_err(|e| AvvGeneratorError::RenderError(e.to_string()))?;
    writeln!(out, "\n[TEXT AUS JURISTISCH FREIGEGEBENER VORLAGE EINFÜGEN]\n").map_err(|e| AvvGeneratorError::RenderError(e.to_string()))?;
    writeln!(out, "Statutarische Anforderungen gemäß Art. 28 Abs. 3 lit. a DSGVO:").map_err(|e| AvvGeneratorError::RenderError(e.to_string()))?;
    writeln!(out, "- Verarbeitung personenbezogener Daten ausschließlich auf dokumentierte Weisung des Verantwortlichen.").map_err(|e| AvvGeneratorError::RenderError(e.to_string()))?;
    writeln!(out, "- Unterrichtung des Verantwortlichen, falls eine Weisung gegen die DSGVO oder andere Datenschutzbestimmungen verstößt.").map_err(|e| AvvGeneratorError::RenderError(e.to_string()))?;
    writeln!(out, "- Verbot von Datenübermittlungen an ein Drittland ohne ausdrückliche rechtliche Verpflichtung oder Weisung.").map_err(|e| AvvGeneratorError::RenderError(e.to_string()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_instructions() {
        let mut out = String::new();
        render_instructions(&mut out).expect("Rendering succeeds");
        assert!(out.contains("Art. 28 Abs. 3 lit. a DSGVO"));
        assert!(out.contains("[TEXT AUS JURISTISCH FREIGEGEBENER VORLAGE EINFÜGEN]"));
    }
}
