//! Art. 28(3)(b) GDPR - Vertraulichkeitsverpflichtung des Personals.

use crate::error::AvvGeneratorError;
use std::fmt::Write;

/// Renders the section for Art. 28(3)(b) GDPR (Personnel confidentiality commitment).
pub fn render_confidentiality(out: &mut String) -> Result<(), AvvGeneratorError> {
    writeln!(out, "\n## 9. Vertraulichkeitsverpflichtung des Personals (Art. 28 Abs. 3 lit. b DSGVO)").map_err(|e| AvvGeneratorError::RenderError(e.to_string()))?;
    writeln!(out, "\n[TEXT AUS JURISTISCH FREIGEGEBENER VORLAGE EINFÜGEN]\n").map_err(|e| AvvGeneratorError::RenderError(e.to_string()))?;
    writeln!(out, "Statutarische Anforderungen gemäß Art. 28 Abs. 3 lit. b DSGVO:").map_err(|e| AvvGeneratorError::RenderError(e.to_string()))?;
    writeln!(out, "- Verpflichtung aller zur Verarbeitung ermächtigten Personen auf die Vertraulichkeit.").map_err(|e| AvvGeneratorError::RenderError(e.to_string()))?;
    writeln!(out, "- Bestehen einer angemessenen gesetzlichen oder vertraglichen Verschwiegenheitspflicht.").map_err(|e| AvvGeneratorError::RenderError(e.to_string()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_confidentiality() {
        let mut out = String::new();
        render_confidentiality(&mut out).expect("Rendering succeeds");
        assert!(out.contains("Art. 28 Abs. 3 lit. b DSGVO"));
        assert!(out.contains("[TEXT AUS JURISTISCH FREIGEGEBENER VORLAGE EINFÜGEN]"));
    }
}
