//! Fehlerbehandlung für die Generierung von AVV-Dokumenten (Auftragsverarbeitungsvertrag).
//!
//! # Zweck & Architektur
//! Dieses Modul definiert den zentralen Fehlertyp [`AvvError`] sowie dessen Alias [`AvvGeneratorError`].
//! Fehler treten typischerweise an zwei Stellen auf:
//! 1. Bei der **semantischen Validierung** der Eingabeparameter ([`crate::AvvContext::validate`]),
//!    z. B. wenn Pflichtfelder fehlen, SLAs unzulässig sind oder Unterauftragnehmer unvollständig angegeben wurden.
//! 2. Bei der **Textgenerierung / Formatierung** ([`crate::template::render`]), wenn der Puffer
//!    oder Formatierungsfunktionen unerwartet fehlschlagen.
//!
//! # Ring-Zugehörigkeit
//! Teil von Ring 4 (Erweiterung / Utility). Hält keine systemweiten Sperren und greift
//! auf keine direkten I/O- oder Betriebssystem-Ressourcen zu.

use thiserror::Error;

// ANCHOR[contextra-avv-generator:ERR-001]
/// Benannte Fehlerzustände während der AVV-Vorlagenvalidierung oder des Markdown-Renderings.
///
/// Alle Varianten implementieren [`thiserror::Error`] und liefern deutschsprachige,
/// für Endanwender oder Protokolle verständliche Fehlermeldungen.
///
/// # Beispiel
/// ```rust
/// use contextra_avv_generator::{AvvContext, AvvError};
///
/// let ctx = AvvContext::default();
/// // Ein Standard-Kontext besitzt noch keinen Namen für den Verantwortlichen:
/// assert_eq!(ctx.validate(), Err(AvvError::EmptyControllerName));
/// ```
#[derive(Debug, Error, PartialEq, Eq, Clone)]
pub enum AvvError {
    /// Name des Verantwortlichen (Auftraggebers) ist leer oder enthält nur Whitespace.
    ///
    /// Gesetzliche Grundlage: Nach Art. 28 Abs. 3 DSGVO müssen die Vertragsparteien
    /// eindeutig identifiziert sein.
    #[error("Verantwortlicher (controller_name) darf nicht leer sein.")]
    EmptyControllerName,

    /// Name des Auftragsverarbeiters (Auftragnehmers) ist leer oder enthält nur Whitespace.
    ///
    /// Gesetzliche Grundlage: Nach Art. 28 Abs. 3 DSGVO müssen die Vertragsparteien
    /// eindeutig identifiziert sein.
    #[error("Auftragsverarbeiter (processor_name) darf nicht leer sein.")]
    EmptyProcessorName,

    /// Vertragsdatum wurde als `Some(...)` übergeben, ist aber leer oder besteht nur aus Whitespace.
    ///
    /// NOTE: Ein nicht gesetztes Datum (`None`) ist erlaubt und fällt auf den Platzhalter
    /// „gemäß Ausstellungsdatum“ zurück. Nur ein explizit leerer String wird hier abgelehnt.
    #[error("Vertragsdatum (contract_date) darf nicht leer sein.")]
    EmptyContractDate,

    /// Löschfrist (SLA = Service Level Agreement) nach Vertragsende ist unzulässig.
    ///
    /// Erlaubter Wertebereich: 1 bis 365 Tage.
    /// - 0 Tage: Technisch oder organisatorisch in der Regel nicht durchführbar.
    /// - > 365 Tage: Unverhältnismäßig lange Aufbewahrungsfrist nach Vertragsende.
    #[error("Ungültiges Deletion-SLA ({days} Tage): {reason}")]
    InvalidDeletionSla {
        /// Die ungültige Fristdauer in Tagen.
        days: u32,
        /// Begründung, warum die Frist unzulässig ist.
        reason: &'static str,
    },

    /// Unterauftragsverarbeiter wurden explizit autorisiert (`subprocessors_allowed = Some(true)`),
    /// es wurden jedoch keine konkreten Unterauftragnehmer in der Liste angegeben.
    ///
    /// Gesetzliche Grundlage: Art. 28 Abs. 2 DSGVO fordert entweder eine konkrete Genehmigung
    /// einer Liste oder eine dokumentierte Genehmigung mit Widerspruchsrecht.
    #[error("Unterauftragsverarbeiter-Kennzeichnung ist gesetzt, aber die Liste der Unterauftragnehmer ist leer.")]
    MissingSubprocessors,

    /// Ein einzelner Unterauftragsverarbeiter-Eintrag ist ungültig (z. B. Leerstring).
    #[error("Ungültiger Unterauftragsverarbeiter-Eintrag: {0}")]
    InvalidSubprocessor(String),

    /// Eine technische und organisatorische Maßnahme (TOM) ist unvollständig definiert.
    ///
    /// Tritt auf, wenn Name, Beschreibung oder DSGVO-Referenzartikel leer sind.
    #[error("Ungültige technische und organisatorische Maßnahme: {0}")]
    InvalidTechnicalMeasure(String),

    /// Generischer Validierungsfehler für unzulässige Kontextkonfigurationen.
    #[error("Ungültiger Kontext: {0}")]
    InvalidContext(String),

    /// Fehler während der Markdown-Generierung (z. B. `std::fmt::Write`-Fehler).
    #[error("Render-Fehler: {0}")]
    RenderError(String),
}

// ANCHOR[contextra-avv-generator:ERR-002]
/// Typ-Alias für Rückwärtskompatibilität mit älteren Schnittstellen ([`AvvGeneratorError`]).
pub type AvvGeneratorError = AvvError;
