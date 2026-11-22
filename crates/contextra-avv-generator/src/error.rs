//! Error types for AVV document generation.

use thiserror::Error;

/// Named errors that can occur during AVV template validation or rendering.
#[derive(Debug, Error, PartialEq, Eq, Clone)]
pub enum AvvError {
    /// Controller name is empty or whitespace-only.
    #[error("Verantwortlicher (controller_name) darf nicht leer sein.")]
    EmptyControllerName,

    /// Processor name is empty or whitespace-only.
    #[error("Auftragsverarbeiter (processor_name) darf nicht leer sein.")]
    EmptyProcessorName,

    /// Contract date is empty or whitespace-only when provided.
    #[error("Vertragsdatum (contract_date) darf nicht leer sein.")]
    EmptyContractDate,

    /// Deletion SLA days is invalid (e.g. 0 days or > 365 days).
    #[error("Ungültiges Deletion-SLA ({days} Tage): {reason}")]
    InvalidDeletionSla {
        /// The invalid SLA duration in days.
        days: u32,
        /// Reason explaining why the duration is invalid.
        reason: &'static str,
    },

    /// Subprocessors flag is enabled (`subprocessors_allowed = Some(true)`), but subprocessor list is empty.
    #[error("Unterauftragsverarbeiter-Kennzeichnung ist gesetzt, aber die Liste der Unterauftragnehmer ist leer.")]
    MissingSubprocessors,

    /// Invalid subprocessor entry (e.g. empty name).
    #[error("Ungültiger Unterauftragsverarbeiter-Eintrag: {0}")]
    InvalidSubprocessor(String),

    /// Invalid technical measure entry (e.g. empty name, description, or reference article).
    #[error("Ungültige technische und organisatorische Maßnahme: {0}")]
    InvalidTechnicalMeasure(String),

    /// A required field in the context was empty or invalid.
    #[error("Ungültiger Kontext: {0}")]
    InvalidContext(String),

    /// Error during template rendering or formatting.
    #[error("Render-Fehler: {0}")]
    RenderError(String),
}

/// Alias for backwards compatibility with existing [`AvvGeneratorError`].
pub type AvvGeneratorError = AvvError;
