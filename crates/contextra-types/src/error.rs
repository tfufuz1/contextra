//! Fehlermodell und Taxonomie für Contextra (SPEC 7.5, I-11, HR-4, TY-12, L-10).
//!
//! `Error` ist eine geschlossene Enum mit festen `&'static str`-Texten über `Display`.
//! Kein Feld trägt Nutzerinhalt, Schlüssel oder Klartextkennungen (I-11).

use thiserror::Error as ThisError;

/// Typisierter Result-Alias.
pub type Result<T> = std::result::Result<T, Error>;

/// Konfigurations- und Eingabefehler.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, ThisError)]
pub enum ConfigError {
    /// Identifier ist leer.
    #[error("identifier is empty")]
    EmptyIdentifier,
    /// Identifier überschreitet Maximallänge.
    #[error("identifier exceeds maximum length")]
    IdentifierTooLong,
    /// Ungültige Bytegröße.
    #[error("invalid byte length")]
    InvalidLength,
}

/// Konflikte bei Einfüge- oder Update-Operationen.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, ThisError)]
pub enum Conflict {
    /// Kollision bei Identifikatoren oder kryptografischen Tags (SPEC D-08).
    #[error("identifier collision")]
    IdCollision,
    /// Objekt oder Dokument existiert bereits.
    #[error("record already exists")]
    Exists,
}

/// Kapazitäts- und Ressourcenlimits (SPEC 14.4, D-87, L-10).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, ThisError)]
pub enum Limit {
    /// Speicherkapazität erschöpft.
    #[error("limit exceeded: storage capacity")]
    Capacity,
    /// Mindest-Plattenreserve unterschritten (SPEC D-65).
    #[error("limit exceeded: disk reserve exhausted")]
    DiskReserve,
    /// Maximale Anzahl aktiver Mandanten-Snapshots erreicht.
    #[error("limit exceeded: snapshots")]
    Snapshots,
    /// Suchanfrage überschreitet Termgrenze.
    #[error("limit exceeded: query too large")]
    QueryTooLarge,
    /// Vektordimension überschreitet Grenze (L-10).
    #[error("limit exceeded: dimension")]
    Dimension,
}

/// Kanonische geschlossene Fehler-Enum für alle Schichten (SPEC 7.5, TY-12).
#[derive(Debug, Clone, PartialEq, Eq, ThisError)]
#[non_exhaustive]
pub enum Error {
    /// Konfigurations- oder Konstruktionsfehler.
    #[error("configuration error: {0}")]
    Config(#[from] ConfigError),

    /// Ein-/Ausgabefehler oder Handle-Vergiftung (SPEC D-55).
    #[error("i/o error")]
    Io,

    /// Datenkorruption oder unerwartetes Format (fatal für das Objekt, P2).
    #[error("data corruption detected")]
    Corrupt,

    /// Authentisierungsfehler (z. B. falsche Passphrase).
    #[error("authentication failed")]
    Auth,

    /// Dokument oder Ziel ist gelöscht (SPEC 11.3).
    #[error("target is erased")]
    Erased,

    /// Verarbeitung ist eingeschränkt (SPEC 11.3, Art. 18 DSGVO).
    #[error("target is restricted")]
    Restricted,

    /// Nicht gefunden.
    #[error("not found")]
    NotFound,

    /// Konflikt bei Transaktion oder Identität.
    #[error("conflict: {0}")]
    Conflict(#[from] Conflict),

    /// Ressourcen- oder Systemgrenze überschritten.
    #[error("{0}")]
    Limit(#[from] Limit),

    /// Snapshot wurde durch Purge oder Timeout widerrufen (SPEC D-61).
    #[error("snapshot revoked")]
    SnapshotRevoked,

    /// Operation wurde abgebrochen.
    #[error("operation cancelled")]
    Cancelled,

    /// Operation durch Richtlinie oder Aufbewahrung verweigert.
    #[error("operation refused")]
    Refused,

    /// Interner Fehler.
    #[error("internal system error")]
    Internal,
}
