//! Fehlerarten und Fehlerbehandlung des Löschkerns (SPEC §7.5).

use thiserror::Error;

/// Oberster Fehlertyp des Löschkerns.
#[derive(Debug, Error)]
pub enum ErasureError {

    /// I/O-Fehler (D-55): nach diesem Fehler darf derselbe Handle nie wieder verwendet werden.
    #[error("erasure-io: {0}")]
    Io(#[from] std::io::Error),
    /// Speicherfehler.
    #[error("storage error: {0}")]
    Storage(String),


    /// Slot-MAC-Prüfung fehlgeschlagen → Datei beschädigt.
    #[error("erasure-corrupt: vault slot MAC invalid")]
    Corrupt,

    /// Vault-Kapazität erschöpft (Lastfaktor > 0,7 bei maxDocs).
    #[error("erasure-capacity: vault load factor exceeded")]
    Capacity,

    /// Slot wurde nicht gefunden, obwohl erwartet.
    #[error("erasure-slot-missing: expected slot not present")]
    SlotMissing,

    /// Vor Auslieferung eingeschränkter Daten ist zwingend ein Audit-Eintrag erforderlich (SPEC §12 T-DEL-13).
    #[error("erasure-restricted: audit entry required before read")]
    RestrictedReadAuditRequired,

    /// Schlüsselvernichtung nicht verifiziert: Slot ist nicht vollständig genullt (SPEC §5.3 Schritt 3.6).
    #[error("erasure-shred: key shredding not verified, slot not zeroed")]
    ShredVerificationFailed,

    /// Übergangsfolge in der Zustandsmaschine verletzt.
    #[error("erasure-state: invalid state transition from {from:?} to {to:?}")]
    InvalidTransition {
        /// Aktueller Zustand.
        from: String,
        /// Gewünschter Zielzustand.
        to: String,
    },


    /// Plattenreserve unterschritten.
    #[error("erasure-disk-reserve: insufficient free space")]
    DiskReserve,

    /// Restrisiko festgestellt: Scan ergab Treffer.
    #[error("erasure-residue: scanner detected {0} matches")]
    Residue(u64),

    /// JSON-Kanonisierung oder -Serialisierung fehlgeschlagen.
    #[error("erasure-json: {0}")]
    Json(String),



    /// Merkmal noch nicht implementiert (nur während Entwicklung).
    #[error("erasure-not-impl: {0}")]
    NotImplemented(String),

    /// Interner Fehler (ungültige Zustandsübergänge, Assertions).
    #[error("erasure-internal: {0}")]
    Internal(String),







}
