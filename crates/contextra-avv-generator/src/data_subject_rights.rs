//! Art. 28(3)(e) GDPR - Unterstützung bei Betroffenenrechten.

use crate::error::AvvGeneratorError;
use std::fmt::Write;

/// Renders the section for Art. 28(3)(e) GDPR (Assistance with data-subject rights).
pub fn render_data_subject_rights(out: &mut String) -> Result<(), AvvGeneratorError> {
    writeln!(out, "\n## 10. Unterstützung bei Betroffenenrechten (Art. 28 Abs. 3 lit. e DSGVO)").map_err(|e| AvvGeneratorError::RenderError(e.to_string()))?;
    writeln!(out, "\n[TEXT AUS JURISTISCH FREIGEGEBENER VORLAGE EINFÜGEN]\n").map_err(|e| AvvGeneratorError::RenderError(e.to_string()))?;
    writeln!(out, "Statutarische Anforderungen gemäß Art. 28 Abs. 3 lit. e DSGVO:").map_err(|e| AvvGeneratorError::RenderError(e.to_string()))?;
    writeln!(out, "- Unterstützung des Verantwortlichen durch geeignete technische und organisatorische Maßnahmen bei der Erfüllung von Anträgen auf Wahrnehmung der Betroffenenrechte (Kapitel III DSGVO).").map_err(|e| AvvGeneratorError::RenderError(e.to_string()))?;
    writeln!(out, "- Unverzügliche Weiterleitung direkt beim Auftragsverarbeiter eingehender Betroffenenanträge an den Verantwortlichen.").map_err(|e| AvvGeneratorError::RenderError(e.to_string()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_data_subject_rights() {
        let mut out = String::new();
        render_data_subject_rights(&mut out).expect("Rendering succeeds");
        assert!(out.contains("Art. 28 Abs. 3 lit. e DSGVO"));
        assert!(out.contains("[TEXT AUS JURISTISCH FREIGEGEBENER VORLAGE EINFÜGEN]"));
    }
}
