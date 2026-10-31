//! Unterstützung bei Art. 32–36 DSGVO gemäß Art. 28 Abs. 3 lit. f DSGVO.
//!
//! # Zweck & Hintergrund
//! Dieser Abschnitt regelt die Unterstützungspflichten des Auftragsverarbeiters bei
//! sicherheits- und compliancebezogenen Anforderungen der DSGVO.
//!
//! Gesetzlicher Kernpunkt nach Art. 28 Abs. 3 lit. f DSGVO:
//! Der Verarbeiter unterstützt den Verantwortlichen bei:
//! - **Art. 32 DSGVO:** Technische und organisatorische Maßnahmen zur Sicherheit der Verarbeitung.
//! - **Art. 33 & 34 DSGVO:** Meldung von Verletzungen des Schutzes personenbezogener Daten (Data Breaches)
//!   an Aufsichtsbehörden und Benachrichtigung betroffener Personen.
//! - **Art. 35 & 36 DSGVO:** Durchführung von Datenschutz-Folgenabschätzungen (DSFA) sowie
//!   etwaige vorherige Konsultationen der Aufsichtsbehörde.
//!
//! # Ring-Zugehörigkeit
//! Teil von Ring 4 (Erweiterung).

use crate::error::AvvError;
use std::fmt::Write;

// ANCHOR[contextra-avv-generator:ASST-001]
/// Rendert Abschnitt 10: „Unterstützung bei Einhaltung der Pflichten gemäß Art. 32–36 DSGVO“.
///
/// Schreibt die detaillierten Unterstützungspflichten für Datensicherheit, Breach-Notification
/// und Folgenabschätzungen in den Ausgabepuffer.
///
/// # Ablauf
/// 1. Überschrift für Abschnitt 10 ausgeben.
/// 2. Einleitungssatz zur Unterstützungszusage unter Berücksichtigung verfügbarer Informationen.
/// 3. Unterpunkt Art. 32 DSGVO: Gewährleistung der Sicherheit durch TOMs.
/// 4. Unterpunkt Art. 33 & 34 DSGVO: Unverzügliche Meldung von Datenschutzvorfällen.
/// 5. Unterpunkt Art. 35 & 36 DSGVO: Mitwirkung bei Datenschutz-Folgenabschätzungen.
///
/// # Fehler
/// Gibt [`AvvError::RenderError`] zurück, falls Schreiboperationen in den Puffer fehlschlagen.
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
