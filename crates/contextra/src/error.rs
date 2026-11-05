pub use contextra_types::error::{
    ConfigError as TypesConfigError, Conflict as TypesConflict, Limit as TypesLimit,
};

/// Fester, eingefrorener Fehlercode zur stabilen programmatischen Identifikation (SPEC 7.5, D-110).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[non_exhaustive]
pub enum ErrorCode {
    /// Konfigurationsfehler allgemein.
    Config = 1000,
    /// Vault-Platzierung ungültig (D-112).
    ConfigVaultPlacement = 1001,
    /// Vault-Verzeichnis fehlt (D-112).
    ConfigVaultMissing = 1002,
    /// Vault-Parameter stimmen nicht überein (D-112).
    ConfigVaultMismatch = 1003,
    /// Dateisystem wird nicht unterstützt (D-56, D-114).
    ConfigUnsupportedFilesystem = 1004,
    /// Pflichtparameter fehlt.
    MissingParameter = 1005,
    /// Dateisystem- oder I/O-Fehler (D-55).
    Io = 2000,
    /// Datenkorruption (P2).
    Corrupt = 3000,
    /// Authentisierungsfehler (falsche Passphrase).
    Auth = 4000,
    /// Zielobjekt wurde entfernt.
    Erased = 5000,
    /// Verarbeitung eingeschränkt (Art. 18).
    Restricted = 6000,
    /// Objekt nicht gefunden.
    NotFound = 7000,
    /// Konflikt allgemein.
    Conflict = 8000,
    /// Kollision zweier verschiedener Identitäten mit gleichem Tag (D-08).
    ConflictIdCollision = 8001,
    /// Objekt existiert bereits.
    ConflictExists = 8002,
    /// Identitätskonflikt bei Ersetzung (D-115).
    ConflictSameIdentity = 8003,
    /// Aufbewahrungskonflikt (D-119).
    ConflictRetentionMismatch = 8004,
    /// Serialisierungsfehler unter SSI (D-124).
    ConflictSerializationFailure = 8005,
    /// Betroffener ist unveränderlich.
    ConflictSubjectImmutable = 8006,
    /// Limit allgemein.
    Limit = 9000,
    /// Speicherkapazität erschöpft.
    LimitCapacity = 9001,
    /// Plattenreserve unterschritten (D-65).
    LimitDiskReserve = 9002,
    /// Maximale Snapshots erreicht.
    LimitSnapshots = 9003,
    /// Suchanfrage zu groß.
    LimitQueryTooLarge = 9004,
    /// Vektorspeicherlimit überschritten (D-122).
    LimitVectorMemory = 9005,
    /// Vektordimension ungültig.
    LimitDimension = 9006,
    /// Snapshot wurde widerrufen (D-61, D-127).
    SnapshotRevoked = 10000,
    /// Operation abgebrochen.
    Cancelled = 11000,
    /// Operation abgewiesen (z. B. Aufbewahrungspflicht aktiv).
    Refused = 12000,
    /// Interner Systemfehler.
    Internal = 13000,
    /// Merkmal noch nicht implementiert (D-110).
    NotImplemented = 14000,
}

impl ErrorCode {
    /// Liefert den numerischen Code als `u32`.
    pub const fn as_u32(self) -> u32 {
        self as u32
    }
}

/// Detaillierte Konfigurationsfehler gemäß SPEC 7.5 und 5.1.0-Entscheidungen.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum ConfigDetails {
    /// Vault liegt unerlaubt im Datenverzeichnis (D-112).
    #[error("Schlüsseltresor darf nicht im Datenverzeichnis liegen. Bitte separates vault_dir konfigurieren.")]
    VaultPlacement,
    /// Vault-Verzeichnis fehlt (D-112).
    #[error("Schlüsseltresor-Verzeichnis nicht gefunden. Bitte Pfad und Zugriffsrechte prüfen.")]
    VaultMissing,
    /// Vault-Parameter stimmen nicht mit Konfiguration überein (D-112).
    #[error(
        "Schlüsseltresor-Parameter weichen ab. Konfiguration an den vorhandenen Tresor anpassen."
    )]
    VaultMismatch,
    /// Nicht unterstütztes Dateisystem (D-56, D-114).
    #[error("Dateisystem wird nicht unterstützt (Code: {0}). Bitte Tier-1- oder Tier-2-Dateisystem verwenden.")]
    UnsupportedFilesystem(u32),
    /// Allgemeine Konfigurationsmeldung ohne vertrauliche Details.
    #[error("Konfigurationsfehler: {0}. Konfigurationsparameter korrigieren.")]
    Other(String),
}

/// Detaillierte Konfliktursachen gemäß SPEC 7.5 und D-124.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum ConflictDetails {
    /// Tag-Kollision (D-08).
    #[error(
        "Identitätskollision festgestellt. Transaktion wiederholen oder Schlüsselraum prüfen."
    )]
    IdCollision,
    /// Record existiert bereits.
    #[error("Eintrag existiert bereits. Bitte andere Kennung wählen oder Aktualisierung nutzen.")]
    Exists,
    /// Identitätskonflikt bei Ersetzung (D-115).
    #[error("Ersetzung erfordert eine veränderte Identität. Identitätsangaben anpassen.")]
    SameIdentity,
    /// Aufbewahrungskonflikt (D-119).
    #[error("Aufbewahrungsattestierung widerspricht bestehendem Vorgang. Status prüfen.")]
    RetentionMismatch,
    /// Serialisierungsfehler unter SSI (D-124).
    #[error("Serialisierungsfehler in Transaktion. Transaktion erneut ausführen.")]
    SerializationFailure,
    /// Betroffener ist unveränderlich.
    #[error("Betroffener kann nach Anlage nicht mehr geändert werden.")]
    SubjectImmutable,
}

/// Detaillierte Limitüberschreitungen gemäß SPEC 7.5 und D-87.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum LimitDetails {
    /// Speicherkapazität erschöpft.
    #[error("Speicherkapazität erschöpft. Bereinigung durchführen oder Speicherplatz erweitern.")]
    Capacity,
    /// Plattenreserve unterschritten (D-65).
    #[error(
        "Mindest-Plattenreserve unterschritten. Freien Speicherplatz auf dem Medium schaffen."
    )]
    DiskReserve,
    /// Maximale Anzahl aktiver Snapshots erreicht (D-61).
    #[error(
        "Maximale Anzahl aktiver Lese-Snapshots erreicht. Nicht benötigte Snapshots freigeben."
    )]
    Snapshots,
    /// Suchanfrage überschreitet Termgrenze.
    #[error("Suchanfrage überschreitet die zulässige Maximalgröße. Anfrage kürzen.")]
    QueryTooLarge,
    /// Vektorspeicherlimit überschritten (D-122).
    #[error("Vektorspeicher-Budget erschöpft. Speicherbudget erhöhen oder Segmente verkleinern.")]
    VectorMemory,
    /// Vektordimension ungültig.
    #[error("Vektordimension überschreitet die Konfigurationsgrenze. Dimension anpassen.")]
    Dimension,
}

/// Öffentlicher Fehlertyp der Fassade (SPEC 7.5).
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    /// Konfigurationsfehler beim Öffnen oder Starten des Systems.
    #[error("{0}")]
    Config(#[from] ConfigDetails),
    /// Pflichtparameter fehlt beim Erstellen der Instanz oder des Handles.
    #[error("Pflichtparameter fehlt: {parameter}. Bitte den erforderlichen Parameter angeben.")]
    MissingParameter {
        /// Name des fehlenden Parameters.
        parameter: &'static str,
    },
    /// Dateisystem- oder I/O-Fehler beim Zugriff auf das Datenverzeichnis (D-55).
    #[error("Dateisystem- oder I/O-Fehler. Bitte Schreibrechte, Speicherplatz und Pfade prüfen.")]
    Io,
    /// Erkannte Datenkorruption im Speicherbestand (P2).
    #[error("Datenkorruption festgestellt. Bitte Sicherung oder Restore-Verfahren hinzuziehen.")]
    Corrupt,
    /// Authentisierungsfehler (z. B. fehlerhafte Passphrase).
    #[error("Authentisierung fehlgeschlagen. Bitte Passphrase überprüfen.")]
    Auth,
    /// Zielobjekt wurde durch einen Löschvorgang entfernt (SPEC 7.2, 11.3).
    #[error("Das angeforderte Zielobjekt wurde entfernt.")]
    Erased,
    /// Die Verarbeitung dieses Objekts ist gemäß Richtlinie eingeschränkt (SPEC 7.2, Art. 18).
    #[error("Die Verarbeitung des Objekts ist eingeschränkt (z. B. Art. 18).")]
    Restricted,
    /// Angefordertes Objekt wurde nicht gefunden.
    #[error("Objekt nicht gefunden. Vorgang mit gültiger Kennung wiederholen.")]
    NotFound,
    /// Konflikt bei Identität oder Transaktion.
    #[error("{0}")]
    Conflict(#[from] ConflictDetails),
    /// Systemgrenze oder Ressourcenlimit erreicht.
    #[error("{0}")]
    Limit(#[from] LimitDetails),
    /// Snapshot-Gültigkeit abgelaufen oder widerrufen (D-61, D-127).
    #[error("Snapshot wurde widerrufen. Neuen Lese-Snapshot anfordern.")]
    SnapshotRevoked,
    /// Operation wurde abgebrochen.
    #[error("Operation wurde vorzeitig abgebrochen.")]
    Cancelled,
    /// Operation wurde abgewiesen (z. B. Aufbewahrungspflicht aktiv).
    #[error("Operation abgewiesen. Prüfen Sie die bestehenden Aufbewahrungspflichten.")]
    Refused,
    /// Interner Systemfehler.
    #[error("Interner Systemfehler. Systemzustand prüfen und Vorgang wiederholen.")]
    Internal,
    /// Merkmal noch nicht implementiert (D-110).
    #[error("Funktion noch nicht implementiert: {0}.")]
    NotImplemented(&'static str),
}

impl Error {
    /// Liefert den zugehörigen `ErrorCode` (SPEC 7.5, D-110).
    pub fn code(&self) -> ErrorCode {
        match self {
            Error::Config(ConfigDetails::VaultPlacement) => ErrorCode::ConfigVaultPlacement,
            Error::Config(ConfigDetails::VaultMissing) => ErrorCode::ConfigVaultMissing,
            Error::Config(ConfigDetails::VaultMismatch) => ErrorCode::ConfigVaultMismatch,
            Error::Config(ConfigDetails::UnsupportedFilesystem(_)) => {
                ErrorCode::ConfigUnsupportedFilesystem
            }
            Error::Config(ConfigDetails::Other(_)) => ErrorCode::Config,
            Error::MissingParameter { .. } => ErrorCode::MissingParameter,
            Error::Io => ErrorCode::Io,
            Error::Corrupt => ErrorCode::Corrupt,
            Error::Auth => ErrorCode::Auth,
            Error::Erased => ErrorCode::Erased,
            Error::Restricted => ErrorCode::Restricted,
            Error::NotFound => ErrorCode::NotFound,
            Error::Conflict(ConflictDetails::IdCollision) => ErrorCode::ConflictIdCollision,
            Error::Conflict(ConflictDetails::Exists) => ErrorCode::ConflictExists,
            Error::Conflict(ConflictDetails::SameIdentity) => ErrorCode::ConflictSameIdentity,
            Error::Conflict(ConflictDetails::RetentionMismatch) => {
                ErrorCode::ConflictRetentionMismatch
            }
            Error::Conflict(ConflictDetails::SerializationFailure) => {
                ErrorCode::ConflictSerializationFailure
            }
            Error::Conflict(ConflictDetails::SubjectImmutable) => {
                ErrorCode::ConflictSubjectImmutable
            }
            Error::Limit(LimitDetails::Capacity) => ErrorCode::LimitCapacity,
            Error::Limit(LimitDetails::DiskReserve) => ErrorCode::LimitDiskReserve,
            Error::Limit(LimitDetails::Snapshots) => ErrorCode::LimitSnapshots,
            Error::Limit(LimitDetails::QueryTooLarge) => ErrorCode::LimitQueryTooLarge,
            Error::Limit(LimitDetails::VectorMemory) => ErrorCode::LimitVectorMemory,
            Error::Limit(LimitDetails::Dimension) => ErrorCode::LimitDimension,
            Error::SnapshotRevoked => ErrorCode::SnapshotRevoked,
            Error::Cancelled => ErrorCode::Cancelled,
            Error::Refused => ErrorCode::Refused,
            Error::Internal => ErrorCode::Internal,
            Error::NotImplemented(_) => ErrorCode::NotImplemented,
        }
    }
}

impl From<TypesConflict> for Error {
    fn from(c: TypesConflict) -> Self {
        match c {
            TypesConflict::IdCollision => Error::Conflict(ConflictDetails::IdCollision),
            TypesConflict::Exists => Error::Conflict(ConflictDetails::Exists),
        }
    }
}

impl From<TypesLimit> for Error {
    fn from(l: TypesLimit) -> Self {
        match l {
            TypesLimit::Capacity => Error::Limit(LimitDetails::Capacity),
            TypesLimit::DiskReserve => Error::Limit(LimitDetails::DiskReserve),
            TypesLimit::Snapshots => Error::Limit(LimitDetails::Snapshots),
            TypesLimit::QueryTooLarge => Error::Limit(LimitDetails::QueryTooLarge),
            TypesLimit::Dimension => Error::Limit(LimitDetails::Dimension),
        }
    }
}

impl From<TypesConfigError> for Error {
    fn from(c: TypesConfigError) -> Self {
        Error::Config(ConfigDetails::Other(c.to_string()))
    }
}

impl From<contextra_types::error::Error> for Error {
    fn from(err: contextra_types::error::Error) -> Self {
        match err {
            contextra_types::error::Error::Config(c) => c.into(),
            contextra_types::error::Error::Io => Error::Io,
            contextra_types::error::Error::Corrupt => Error::Corrupt,
            contextra_types::error::Error::Auth => Error::Auth,
            contextra_types::error::Error::Erased => Error::Erased,
            contextra_types::error::Error::Restricted => Error::Restricted,
            contextra_types::error::Error::NotFound => Error::NotFound,
            contextra_types::error::Error::Conflict(c) => c.into(),
            contextra_types::error::Error::Limit(l) => l.into(),
            contextra_types::error::Error::SnapshotRevoked => Error::SnapshotRevoked,
            contextra_types::error::Error::Cancelled => Error::Cancelled,
            contextra_types::error::Error::Refused => Error::Refused,
            _ => Error::Internal,
        }
    }
}

impl From<contextra_types::ContextraError> for Error {
    fn from(err: contextra_types::ContextraError) -> Self {
        match err {
            contextra_types::ContextraError::Io(_) => Error::Io,
            contextra_types::ContextraError::InvalidInput(msg) => {
                Error::Config(ConfigDetails::Other(msg))
            }
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
