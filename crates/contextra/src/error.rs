//! Fehlertypen der öffentlichen Contextra-Fassade.
//!
//! # Architektur-Rolle: Ring 4 (Fassade)
//!
//! Gemäß SPEC D-89 und §7:
//! Alle Typen sind mit `#[non_exhaustive]` versehen.
//! Fehlertexte enthalten konkrete Handlungsempfehlungen für den Betreiber (SPEC §6.4),
//! geben niemals Geheimnisse preis und verwenden keine nach SPEC §6.4 verbotenen Begriffe.

pub use contextra_types::error::{ConfigError, Conflict, Limit};
use std::fmt;

/// Öffentlicher Fehlertyp der Fassade.
///
/// Bietet klare Handlungsempfehlungen bei Fehlern und stellt sicher,
/// dass sensible Informationen (wie Passphrasen oder Schlüsselmaterial)
/// weder in `Display` noch in `Debug` erscheinen.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    /// Konfigurationsfehler beim Öffnen oder Starten des Systems.
    #[error("Konfigurationsfehler: {message}. Bitte Konfigurationsparameter prüfen.")]
    Config {
        /// Detailbeschreibung des Konfigurationsfehlers ohne sensible Daten.
        message: String,
    },

    /// Pflichtparameter fehlt beim Erstellen der Instanz oder des Handles.
    #[error("Pflichtparameter fehlt: {parameter}. Bitte den erforderlichen Parameter angeben.")]
    MissingParameter {
        /// Name des fehlenden Parameters.
        parameter: &'static str,
    },

    /// Dateisystem- oder I/O-Fehler beim Zugriff auf das Datenverzeichnis.
    #[error("Dateisystem- oder I/O-Fehler. Bitte Schreibrechte, Speicherplatz und Pfade prüfen.")]
    Io,

    /// Erkannte Datenkorruption im Speicherbestand.
    #[error("Datenkorruption festgestellt. Bitte Sicherung oder Restore-Verfahren hinzuziehen.")]
    Corrupt,

    /// Authentisierungsfehler (z. B. fehlerhafte Passphrase).
    #[error("Authentisierung fehlgeschlagen. Bitte Passphrase überprüfen.")]
    Auth,

    /// Zielobjekt wurde durch einen Löschvorgang entfernt.
    #[error("Das angeforderte Zielobjekt wurde entfernt.")]
    Erased,

    /// Die Verarbeitung dieses Objekts ist gemäß Richtlinie eingeschränkt.
    #[error("Die Verarbeitung des Objekts ist eingeschränkt (z. B. Art. 18).")]
    Restricted,

    /// Angefordertes Objekt wurde nicht gefunden.
    #[error("Objekt nicht gefunden.")]
    NotFound,

    /// Konflikt bei Identität oder Transaktion.
    #[error("Konflikt festgestellt: {0}. Transaktion wiederholen.")]
    Conflict(#[from] Conflict),

    /// Systemgrenze oder Ressourcenlimit erreicht.
    #[error("Systemgrenze erreicht: {0}. Parameter oder Datenmenge anpassen.")]
    Limit(#[from] Limit),

    /// Snapshot-Gültigkeit abgelaufen oder widerrufen.
    #[error("Snapshot wurde widerrufen. Neuen Lese-Snapshot anfordern.")]
    SnapshotRevoked,

    /// Operation wurde abgebrochen.
    #[error("Operation wurde vorzeitig abgebrochen.")]
    Cancelled,

    /// Operation wurde abgewiesen (z. B. Aufbewahrungspflicht aktiv).
    #[error("Operation abgewiesen. Prüfen Sie die bestehenden Aufbewahrungspflichten.")]
    Refused,

    /// Interner Systemfehler.
    #[error("Interner Systemfehler.")]
    Internal,
}

impl From<contextra_types::error::Error> for Error {
    fn from(err: contextra_types::error::Error) -> Self {
        match err {
            contextra_types::error::Error::Config(c) => Error::Config {
                message: c.to_string(),
            },
            contextra_types::error::Error::Io => Error::Io,
            contextra_types::error::Error::Corrupt => Error::Corrupt,
            contextra_types::error::Error::Auth => Error::Auth,
            contextra_types::error::Error::Erased => Error::Erased,
            contextra_types::error::Error::Restricted => Error::Restricted,
            contextra_types::error::Error::NotFound => Error::NotFound,
            contextra_types::error::Error::Conflict(c) => Error::Conflict(c),
            contextra_types::error::Error::Limit(l) => Error::Limit(l),
            contextra_types::error::Error::SnapshotRevoked => Error::SnapshotRevoked,
            contextra_types::error::Error::Cancelled => Error::Cancelled,
            contextra_types::error::Error::Refused => Error::Refused,
            contextra_types::error::Error::Internal => Error::Internal,
            _ => Error::Internal,
        }
    }
}

impl From<contextra_types::ContextraError> for Error {
    fn from(err: contextra_types::ContextraError) -> Self {
        match err {
            contextra_types::ContextraError::Io(_) => Error::Io,
            contextra_types::ContextraError::InvalidInput(msg) => Error::Config { message: msg },
            contextra_types::ContextraError::NotFound(_) => Error::NotFound,
            contextra_types::ContextraError::AuthenticationFailed => Error::Auth,
            contextra_types::ContextraError::Corruption(_) => Error::Corrupt,
            contextra_types::ContextraError::SnapshotRevoked(_) => Error::SnapshotRevoked,
            _ => Error::Internal,
        }
    }
}

/// Typalias für Result-Typen der Fassade.
pub type Result<T> = std::result::Result<T, Error>;
