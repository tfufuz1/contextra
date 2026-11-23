//! Fehlerklassen und Hinweise (SP-PORTS §4). Texte ohne Schlüssel- oder Nutzdaten.
//! F-7 bis F-9 sind in SPEC-R8 §3.5 zu verifizieren (Lücke, siehe README).
use thiserror::Error as ThisError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ErrorClass {
    /// F-1: EINTR/EAGAIN, Backoff
    Transient,
    /// F-2: ENOSPC nur bei Vorallokation
    DiskFull,
    /// F-3: Sync-Fehler, Handle vergiftet
    Poisoned,
    /// F-4: Prüfsumme, Siegel, Kette
    Corrupt,
    /// F-5: unbekannte Version oder Tag
    Unsupported,
    /// F-6: Schlüssel fehlt, offline oder verschreddert
    Key,
    /// Eingabe verletzt Grenze oder Vertrag
    Usage,
    /// Anker-Wettlauf verloren
    Conflict,
    /// Externer Anker liegt vor den wiederhergestellten dauerhaften Daten
    Rollback,
    /// Platzhalter oder interner Fehler
    Internal,
}

#[derive(Debug, ThisError)]
#[non_exhaustive]
pub enum Error {
    #[error("transienter I/O-Fehler (Rolle {role}), Wiederholung mit Backoff")]
    Transient { role: &'static str },
    #[error("Speicher voll: Vorallokation fehlgeschlagen")]
    DiskFull,
    #[error("Handle vergiftet nach fehlgeschlagenem Sync ({cause})")]
    Poisoned { cause: &'static str },
    #[error("beschädigte Daten in {place}")]
    Corrupt { place: &'static str },
    #[error("nicht unterstützt: {what}")]
    Unsupported { what: &'static str },
    #[error("Schlüssel nicht verfügbar")]
    KeyUnavailable { retry_after_secs: Option<u64> },
    #[error("Daten sind gelöscht (Crypto-Shredding)")]
    Shredded,
    #[error("Eingabe verletzt Grenze: {limit}")]
    Limit { limit: &'static str },
    #[error("Anker-Wettlauf verloren (CAS)")]
    CasLost,
    #[error("Rollback erkannt: Anker liegt vor Catalog oder Ledger")]
    Rollback,
    #[error("I/O-Fehler (Rolle {role}, Art {kind:?})")]
    Io {
        role: &'static str,
        kind: std::io::ErrorKind,
    },
    #[error("Platzhalter: {sp} ist noch nicht implementiert")]
    NotImplemented { sp: &'static str },
}

impl Error {
    pub fn class(&self) -> ErrorClass {
        match self {
            Self::Transient { .. } | Self::Io { .. } => ErrorClass::Transient,
            Self::DiskFull => ErrorClass::DiskFull,
            Self::Poisoned { .. } => ErrorClass::Poisoned,
            Self::Corrupt { .. } => ErrorClass::Corrupt,
            Self::Unsupported { .. } => ErrorClass::Unsupported,
            Self::KeyUnavailable { .. } | Self::Shredded => ErrorClass::Key,
            Self::Limit { .. } => ErrorClass::Usage,
            Self::CasLost => ErrorClass::Conflict,
            Self::Rollback => ErrorClass::Rollback,
            Self::NotImplemented { .. } => ErrorClass::Internal,
        }
    }

    pub fn hint(&self) -> &'static str {
        match self {
            Self::Transient { .. } | Self::Io { .. } => "Mit Backoff erneut versuchen",
            Self::DiskFull => "Platz schaffen oder Erase-Reserve prüfen",
            Self::Poisoned { .. } => "Speicher neu öffnen; Wiederholung nie auf demselben Handle",
            Self::Corrupt { .. } => {
                "Integritätsbericht prüfen, Datei aus Sicherung wiederherstellen"
            }
            Self::Unsupported { .. } => "Version des Kerns prüfen",
            Self::KeyUnavailable { .. } => "Schlüsselprovider prüfen",
            Self::Shredded => "Daten sind endgültig gelöscht",
            Self::Limit { .. } => "Eingabe verkleinern",
            Self::CasLost => "Anker neu lesen und wiederholen",
            Self::Rollback => "ReadOnly-Diagnose öffnen und Restore-Reconcile ausführen",
            Self::NotImplemented { .. } => "Komponente ist noch Platzhalter",
        }
    }
}
