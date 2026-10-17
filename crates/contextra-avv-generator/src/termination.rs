//! Art. 28(3)(g) GDPR - Rückgabe oder Löschung personenbezogener Daten nach Abschluss.

use crate::error::AvvGeneratorError;
use std::fmt::Write;

/// Renders the section for Art. 28(3)(g) GDPR (Return or deletion of personal data).
pub fn render_termination(out: &mut String, deletion_sla_days: u32) -> Result<(), AvvGeneratorError> {
    writeln!(out, "\n## 12. Rückgabe oder Löschung personenbezogener Daten (Art. 28 Abs. 3 lit. g DSGVO)").map_err(|e| AvvGeneratorError::RenderError(e.to_string()))?;
    writeln!(out, "\n[TEXT AUS JURISTISCH FREIGEGEBENER VORLAGE EINFÜGEN]\n").map_err(|e| AvvGeneratorError::RenderError(e.to_string()))?;
    writeln!(out, "Statutarische Anforderungen gemäß Art. 28 Abs. 3 lit. g DSGVO:").map_err(|e| AvvGeneratorError::RenderError(e.to_string()))?;
    writeln!(out, "- Verpflichtung zur Löschung oder Rückgabe aller personenbezogenen Daten nach Abschluss der Erbringung der Verarbeitungsleistungen nach Wahl des Verantwortlichen.").map_err(|e| AvvGeneratorError::RenderError(e.to_string()))?;
    writeln!(out, "- Löschung vorhandener Kopien, sofern nicht nach dem Recht der Europäischen Union oder der Mitgliedstaaten eine Verpflichtung zur Speicherung besteht.").map_err(|e| AvvGeneratorError::RenderError(e.to_string()))?;
    writeln!(out, "- Umsetzung der Löschung innerhalb des vereinbarten Deletion-SLA von {} Tagen.", deletion_sla_days).map_err(|e| AvvGeneratorError::RenderError(e.to_string()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_termination() {
        let mut out = String::new();
        render_termination(&mut out, 14).expect("Rendering succeeds");
        assert!(out.contains("Art. 28 Abs. 3 lit. g DSGVO"));
        assert!(out.contains("[TEXT AUS JURISTISCH FREIGEGEBENER VORLAGE EINFÜGEN]"));
        assert!(out.contains("14 Tagen"));
    }
}
