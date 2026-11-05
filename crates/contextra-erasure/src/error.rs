//! Fehlerarten und Fehlerbehandlung des Löschkerns (SPEC §7.5).

use thiserror::Error;

/// Oberster Fehlertyp des Löschkerns.
#[derive(Debug, Error)]
pub enum ErasureError {
    /// I/O-Fehler (D-55): nach diesem Fehler darf derselbe Handle nie wieder verwendet werden.
    #[error("erasure-io: {0}")]
    Io(#[from] std::io::Error),

    /// Kryptografischer Fehler (AEAD, MAC-Verifizierung fehlgeschlagen).
    #[error("erasure-crypto: {0}")]
    Crypto(String),

    /// Slot-MAC-Prüfung fehlgeschlagen → Datei beschädigt.
    #[error("erasure-corrupt: vault slot MAC invalid")]
    Corrupt,

    /// Vault-Kapazität erschöpft (Lastfaktor > 0,7 bei maxDocs).
    #[error("erasure-capacity: vault load factor exceeded")]
    Capacity,

    /// Slot wurde nicht gefunden, obwohl erwartet.
    #[error("erasure-slot-missing: expected slot not present")]
    SlotMissing,

    /// Übergangsfolge in der Zustandsmaschine verletzt.
    #[error("erasure-state: invalid state transition from {from:?} to {to:?}")]
    InvalidTransition {
        /// Aktueller Zustand.
        from: String,
        /// Gewünschter Zielzustand.
        to: String,
    },

    /// Limit überschritten (SPEC §7.5).
    #[error("erasure-limit: {0}")]
    LimitExceeded(String),

    /// Plattenreserve unterschritten.
    #[error("erasure-disk-reserve: insufficient free space")]
    DiskReserve,

    /// Restrisiko festgestellt: Scan ergab Treffer.
    #[error("erasure-residue: scanner detected {0} matches")]
    Residue(u64),

    /// JSON-Kanonisierung oder -Serialisierung fehlgeschlagen.
    #[error("erasure-json: {0}")]
    Json(String),

    /// Ledger-Fehler (Kette unterbrochen, MAC ungültig).
    #[error("erasure-ledger: {0}")]
    Ledger(String),

    /// Retention-Pflicht verletzt: Löschung wurde verweigert.
    #[error("erasure-refused: retention obligation active")]
    Refused,

    /// Merkmal noch nicht implementiert (nur während Entwicklung).
    #[error("erasure-not-impl: {0}")]
    NotImplemented(String),

    /// Interner Fehler (ungültige Zustandsübergänge, Assertions).
    #[error("erasure-internal: {0}")]
    Internal(String),

    /// Ersatzdokument fehlt vor Gate-Schritt (SPEC D-115, FM-19).
    #[error("erasure-replace: replacement missing")]
    ReplacementMissing,

    /// Berichtigte Version fehlt vor Gate-Schritt (SPEC D-117).
    #[error("erasure-supersede: update missing")]
    UpdateMissing,

    /// Ersetzung verweigert wegen Identitätskollision von Alt- und Neudokument (SPEC D-115).
    #[error("erasure-replace: old and new document share same identity")]
    SameIdentity,

    /// Zugriff auf eingeschränkte Daten unzulässig oder Zweck nicht erfüllt (SPEC D-116).
    #[error("erasure-restricted: access violation")]
    RestrictionViolated(String),

    /// Vor Auslieferung eingeschränkter Daten ist zwingend ein Audit-Eintrag erforderlich (SPEC D-116, I-26).
    #[error("erasure-restricted: audit entry required before read")]
    RestrictedReadAuditRequired,
}
