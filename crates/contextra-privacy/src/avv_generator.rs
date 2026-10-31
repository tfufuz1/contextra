//! `avv_generator` — Re-Export-Schicht auf `contextra-avv-generator`.
//!
//! # Abkündigung & Konsolidierung (AV-1)
//! Dieses Modul diente historisch der Generierung von Entwürfen für Auftragsverarbeitungsverträge
//! (AVV) gemäß Art. 28 DSGVO (Datenschutz-Grundverordnung). Die Logik wurde in das dedizierte Crate
//! `contextra-avv-generator` ausgelagert.
//!
//! # Ring-Zugehörigkeit
//! Ring 3 (Domain / Compliance).
//!
//! # Warnung
//! Veraltet (deprecated). Neue Implementierungen sollten direkt das Crate `contextra-avv-generator` verwenden.

#![deprecated(note = "use contextra-avv-generator crate directly")]

// ANCHOR[contextra-privacy:AVV-001]
/// Re-Export des Kontexts für die AVV-Generierung aus `contextra-avv-generator`.
#[deprecated(note = "use contextra_avv_generator::AvvContext")]
pub use contextra_avv_generator::AvvContext;

// ANCHOR[contextra-privacy:AVV-002]
/// Generiert einen AVV-Entwurf im Markdown-Format anhand des übergebenen Kontexts.
///
/// # Ablauf
/// 1. Übergibt die Kontextstruktur an `contextra_avv_generator::render_avv_markdown`.
/// 2. Im Erfolgsfall wird der generierte Markdown-Text zurückgegeben.
/// 3. Tritt bei der Formatierung ein Fehler auf, wird ein Fehlerhinweis als Markdown formatiert.
///
/// # Parameter
/// * `ctx`: Referenz auf den AVV-Kontext mit Angaben zu Verantwortlichem und Auftragsverarbeiter.
///
/// # Rückgabe
/// Markdown-formatiertes Dokument als `String`.
#[deprecated(note = "use contextra_avv_generator::render_avv_markdown")]
pub fn generate_avv_draft(ctx: &contextra_avv_generator::AvvContext) -> String {
    contextra_avv_generator::render_avv_markdown(ctx).unwrap_or_else(|e| format!("# Fehler: {e}"))
}

#[cfg(test)]
#[allow(deprecated, clippy::field_reassign_with_default)]
mod tests {
    use super::*;

    // ANCHOR[contextra-privacy:TEST-AVV-001]
    // Prüft die Äquivalenz der AVV-Generierung über den Re-Export mit den Standardfeldern.
    #[test]
    fn test_generate_avv_draft_equivalence() {
        let mut ctx = AvvContext::default();
        ctx.controller_name = "Acme Corp".to_string();
        ctx.processor_name = "Contextra AG".to_string();
        let md = generate_avv_draft(&ctx);
        assert!(md.contains("Acme Corp") && md.contains("Art. 28 DSGVO"));
    }
}
