//! Weisungsbindung gemäß Art. 28 Abs. 3 lit. a DSGVO.
//!
//! # Zweck & Hintergrund
//! Dieser Abschnitt regelt verbindlich, dass der Auftragsverarbeiter (Contextra-Betreiber)
//! personenbezogene Daten ausschließlich im Rahmen der dokumentierten Weisungen des
//! Verantwortlichen verarbeitet.
//!
//! Gesetzlicher Kernpunkt nach Art. 28 Abs. 3 lit. a DSGVO:
//! Der Verarbeiter darf Daten nicht eigenmächtig für sekundäre Zwecke verwenden und muss
//! den Verantwortlichen unverzüglich warnen, falls eine Weisung gegen Datenschutzrecht verstößt.
//!
//! # Ring-Zugehörigkeit
//! Teil von Ring 4 (Erweiterung). Reines Template-Fragment ohne Systemabhängigkeiten oder Zustand.

use crate::error::AvvError;
use std::fmt::Write;

// ANCHOR[contextra-avv-generator:INST-001]
/// Rendert Abschnitt 4: „Pflichten und Rechte des Verantwortlichen & Weisungsbindung“.
///
/// Schreibt den formatierten Markdown-Abschnitt in den übergebenen Ausgabe-Puffer.
///
/// # Ablauf
/// 1. Überschrift für Abschnitt 4 ausgeben.
/// 2. Absatz 1: Feststellung der alleinigen Verantwortlichkeit des Auftraggebers für die Zulässigkeit.
/// 3. Absatz 2: Strikte Bindung des Auftragsverarbeiters an dokumentierte Weisungen (inkl. Drittlandstransfers).
/// 4. Absatz 3: Gesetzliche Hinweispflicht bei rechtswidrigen Weisungen.
///
/// # Fehler
/// Gibt [`AvvError::RenderError`] zurück, falls das Schreiben in den Puffer fehlschlägt.
pub fn render_section(out: &mut String) -> Result<(), AvvError> {
    writeln!(
        out,
        "\n## 4. Pflichten und Rechte des Verantwortlichen & Weisungsbindung"
    )
    .map_err(|e| AvvError::RenderError(e.to_string()))?;
    writeln!(
        out,
        "1. Der Verantwortliche ist für die Beurteilung der Zulässigkeit der Datenverarbeitung sowie für die Wahrung der Rechte der betroffenen Personen allein verantwortlich."
    )
    .map_err(|e| AvvError::RenderError(e.to_string()))?;
    writeln!(
        out,
        "2. Der Auftragsverarbeiter verarbeitet personenbezogene Daten ausschließlich auf dokumentierte Weisung des Verantwortlichen — einschließlich hinsichtlich der Übermittlung personenbezogener Daten an ein Drittland oder eine internationale Organisation —, sofern er nicht durch das Recht der Union oder der Mitgliedstaaten, dem der Auftragsverarbeiter unterliegt, hierzu verpflichtet ist."
    )
    .map_err(|e| AvvError::RenderError(e.to_string()))?;
    writeln!(
        out,
        "3. Der Auftragsverarbeiter informiert den Verantwortlichen unverzüglich, wenn er der Auffassung ist, dass eine Weisung gegen die DSGVO oder andere Datenschutzbestimmungen der Union oder der Mitgliedstaaten verstößt."
    )
    .map_err(|e| AvvError::RenderError(e.to_string()))?;
    Ok(())
}
