//! Einheitliche Lebenszyklus-Zustandsmaschine des physischen Purge (SPEC §11.2, §11.4, WP-2, ER-20).
//!
//! Die Phasenfolge ist streng geordnet (Codes 1..=9):
//!   Admitted(1) → Fenced(2) → Flushed(3) → Rewriting(4) → Disposing(5)
//!   → WalRetiring(6) → Witnessing(7) → Finalizing(8) → Released(9)
//!
//! Übergänge sind ausschließlich erlaubt, wenn `can_transition_to` `true` zurückgibt.

// Herkunft: 4b98cf91b:crates/contextra-store/src/lsm/purge/plan.rs:13

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Fehlerzustände während der einzelnen Purge-Phasen (WP-1, WP-2).
///
/// Jeder Fehler besitzt einen klaren Phasenbezug; Freitext-Stringfehler sind verboten.
#[derive(Debug, Error, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PurgePhaseError {
    /// Ungültiger Phasenübergang in der Zustandsmaschine.
    #[error("invalid purge phase transition from {from:?} to {to:?}")]
    InvalidTransition {
        /// Ausgangsphase.
        from: PurgePhase,
        /// Zielphase.
        to: PurgePhase,
    },

    /// Unbekannter oder ungültiger Phasencode.
    #[error("invalid purge phase code: {0}")]
    InvalidCode(u8),

    /// Fehler während der Phase `Admitted`: Vorbedingungen oder Intent ungültig.
    #[error("purge admission failed: {0:?}")]
    AdmissionFailed(AdmissionFailure),

    /// Fehler während der Phase `Fenced`: Erasure-Fence konnte nicht erlangt werden.
    #[error("purge fence failed: {0:?}")]
    FenceFailed(FenceFailure),

    /// Fehler während der Phase `Flushed`: Memtable- oder Cache-Flush fehlgeschlagen.
    #[error("purge flush failed: {0:?}")]
    FlushFailed(FlushFailure),

    /// Fehler während der Phase `Rewriting`: Segment-/SSTable-Rewrite fehlgeschlagen.
    #[error("purge rewrite failed: {0:?}")]
    RewriteFailed(RewriteFailure),

    /// Fehler während der Phase `Disposing`: Entsorgung oder Scrubbing obsolet gewordener Dateien fehlgeschlagen.
    #[error("purge dispose failed: {0:?}")]
    DisposeFailed(DisposeFailure),

    /// Fehler während der Phase `WalRetiring`: WAL-Entwertung oder Rotation fehlgeschlagen.
    #[error("purge wal retiring failed: {0:?}")]
    WalRetireFailed(WalRetireFailure),

    /// Fehler während der Phase `Witnessing`: Schicht-Messung oder Positivkontrolle ergab Fehler oder Residuen.
    #[error("purge witnessing failed: {0:?}")]
    WitnessFailed(WitnessFailure),

    /// Fehler während der Phase `Finalizing`: Verankerung im Ledger oder Saga-Abschluss fehlgeschlagen.
    #[error("purge finalizing failed: {0:?}")]
    FinalizeFailed(FinalizeFailure),

    /// Fehler während der Phase `Released`: Freigabe des Fences oder Intent-Bereinigung fehlgeschlagen.
    #[error("purge release failed: {0:?}")]
    ReleaseFailed(ReleaseFailure),
}

/// Detaillierte Gründe für das Scheitern von `Admitted`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AdmissionFailure {
    /// Scope-Spezifikation ist leer oder ungültig.
    EmptyScope,
    /// Unzureichende Plattenreserve bereits vor Beginn.
    InsufficientSpace {
        /// Freie Bytes.
        free_bytes: u64,
        /// Benötigte Bytes.
        required_bytes: u64,
    },
    /// Scope-Datei konnte nicht geschrieben werden.
    ScopeIoError,
}

/// Detaillierte Gründe für das Scheitern von `Fenced`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FenceFailure {
    /// Der Scope ist durch einen konkurrierenden Purge-Vorgang gesperrt (Single-Flight).
    ScopeAlreadyLocked,
    /// Compaction-Aktivität blockiert das Errichten des Fences.
    CompactionBusy,
    /// Ein aktiver MVCC-Snapshot pinnt eine Version vor dem Cutoff.
    ActiveSnapshotPinned {
        /// Älteste gepinnte Sequenznummer.
        min_seq: u64,
    },
}

/// Detaillierte Gründe für das Scheitern von `Flushed`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FlushFailure {
    /// E/A-Fehler beim Schreiben des Memtable-Flushes.
    IoError,
    /// Timeout beim Warten auf In-Flight-Schreiber.
    Timeout,
}

/// Detaillierte Gründe für das Scheitern von `Rewriting`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RewriteFailure {
    /// Plattenreserve unterschritten während des Rewrite-Vorgangs.
    DiskReserveExceeded,
    /// E/A- oder Integritätsfehler beim Lesen oder Schreiben von Segmenten.
    SegmentIoError,
    /// Dateikopf- oder Signaturkorruption erkannt.
    CorruptSegment,
}

/// Detaillierte Gründe für das Scheitern von `Disposing`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DisposeFailure {
    /// Nullungs-Vorgang (Scrub) fehlgeschlagen.
    ScrubFailed,
    /// Löschen der physischen Datei (`unlink`) verweigert.
    UnlinkRefused,
}

/// Detaillierte Gründe für das Scheitern von `WalRetiring`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum WalRetireFailure {
    /// WAL-Segment konnte nicht rotiert oder genullt werden.
    WalSyncError,
    /// WAL-Integritätskette beschädigt.
    WalCorrupt,
}

/// Detaillierte Gründe für das Scheitern von `Witnessing`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum WitnessFailure {
    /// Positivkontrolle schlug fehl (Scanner unzuverlässig).
    PositiveControlFailed,
    /// Unerwartete Restspuren im gescannten Medium gefunden.
    ResidueDetected {
        /// Anzahl gefundener Spuren.
        count: u64,
    },
    /// Schicht-Zeuge konnte nicht signiert oder generiert werden.
    WitnessGenerationFailed,
}

/// Detaillierte Gründe für das Scheitern von `Finalizing`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FinalizeFailure {
    /// Ledger-Append fehlgeschlagen.
    LedgerAppendFailed,
    /// Ledger-Commit/fsync fehlgeschlagen.
    LedgerSyncFailed,
}

/// Detaillierte Gründe für das Scheitern von `Released`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReleaseFailure {
    /// Fence-Freigabe konnte nicht atomar persistiert werden.
    FenceReleaseFailed,
    /// Bereinigung der Intent-Datei fehlgeschlagen.
    IntentCleanupFailed,
}

/// Einheitliche Enum für den gesamten Lebenszyklus des physischen Purge (WP-2).
///
/// Repräsentiert als `u8`-Codes (1..=9) zur sicheren Persistenz und Wiederherstellung.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum PurgePhase {
    /// 1: Purge-Auftrag angenommen, Intent & Scope persistent geschrieben.
    Admitted = 1,
    /// 2: Erasure-Fence auf betroffenen Scope errichtet, Schreiber blockiert.
    Fenced = 2,
    /// 3: Aktive Memtables und temporäre Puffer auf Platte persistiert.
    Flushed = 3,
    /// 4: Segmente / SSTables ohne gelöschte Einheiten neu geschrieben.
    Rewriting = 4,
    /// 5: Obsolete Altdateien genullt und aus dem Dateisystem entfernt.
    Disposing = 5,
    /// 6: WAL-Segmente des betroffenen Scopes entwertet / rotiert.
    WalRetiring = 6,
    /// 7: Schicht-Messung durchgeführt, Zeugen generiert und verifiziert.
    Witnessing = 7,
    /// 8: Löschung im Ledger verankert und Zustand finalisiert.
    Finalizing = 8,
    /// 9: Erasure-Fence freigegeben, Intent/Scope entfernt (Endzustand).
    Released = 9,
}

impl PurgePhase {
    /// Liefert den numerischen `u8`-Code der Phase (1..=9).
    #[inline]
    #[must_use]
    pub const fn code(self) -> u8 {
        self as u8
    }

    /// Gibt an, ob dieser Zustand der finale Abschlusszustand des Purge-Ablaufes ist.
    #[inline]
    #[must_use]
    pub const fn is_terminal(self) -> bool {
        matches!(self, Self::Released)
    }

    /// Prüft, ob ein Übergang von `self` zu `next` zulässig ist.
    ///
    /// Zulässige Übergänge:
    /// - Genau der nächste lineare Schritt: `Admitted → Fenced → Flushed → Rewriting → Disposing → WalRetiring → Witnessing → Finalizing → Released`
    /// - Idempotente Wiederholung derselben Phase (`self == next`)
    ///
    /// Alle anderen Übergänge (Rücksprünge, Überspringen von Schritten) sind unzulässig (`false`).
    #[must_use]
    pub const fn can_transition_to(self, next: PurgePhase) -> bool {
        let current_code = self.code();
        let next_code = next.code();

        // Idempotenz: Derselbe Zustand ist immer gültig
        if current_code == next_code {
            return true;
        }

        // Streng monotoner Vorwärtsschritt um genau 1
        current_code + 1 == next_code
    }

    /// Ermittelt die deterministische Folgephase im regulären Lebenszyklus.
    ///
    /// Liefert `None`, falls sich die Phase bereits im Endzustand `Released` befindet.
    #[must_use]
    pub const fn next_phase(self) -> Option<PurgePhase> {
        match self {
            Self::Admitted => Some(Self::Fenced),
            Self::Fenced => Some(Self::Flushed),
            Self::Flushed => Some(Self::Rewriting),
            Self::Rewriting => Some(Self::Disposing),
            Self::Disposing => Some(Self::WalRetiring),
            Self::WalRetiring => Some(Self::Witnessing),
            Self::Witnessing => Some(Self::Finalizing),
            Self::Finalizing => Some(Self::Released),
            Self::Released => None,
        }
    }

    /// Prüft, ob die angegebene Zielphase `target` noch ausgeführt werden muss (`self <= target`).
    #[inline]
    #[must_use]
    pub fn must_run(self, target: PurgePhase) -> bool {
        self.code() <= target.code()
    }

    /// Führt einen überprüften Phasenübergang durch.
    ///
    /// Schlägt mit [`PurgePhaseError::InvalidTransition`] fehl, wenn `can_transition_to` `false` ist.
    pub fn transition_to(&mut self, next: PurgePhase) -> Result<(), PurgePhaseError> {
        if !self.can_transition_to(next) {
            return Err(PurgePhaseError::InvalidTransition {
                from: *self,
                to: next,
            });
        }
        *self = next;
        Ok(())
    }

    /// Lädt eine Phase aus ihrem persistierten `u8`-Code (1..=9).
    pub fn from_code(code: u8) -> Result<Self, PurgePhaseError> {
        Self::try_from(code)
    }
}

impl TryFrom<u8> for PurgePhase {
    type Error = PurgePhaseError;

    fn try_from(val: u8) -> Result<Self, Self::Error> {
        match val {
            1 => Ok(PurgePhase::Admitted),
            2 => Ok(PurgePhase::Fenced),
            3 => Ok(PurgePhase::Flushed),
            4 => Ok(PurgePhase::Rewriting),
            5 => Ok(PurgePhase::Disposing),
            6 => Ok(PurgePhase::WalRetiring),
            7 => Ok(PurgePhase::Witnessing),
            8 => Ok(PurgePhase::Finalizing),
            9 => Ok(PurgePhase::Released),
            other => Err(PurgePhaseError::InvalidCode(other)),
        }
    }
}

impl std::fmt::Display for PurgePhase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Admitted => write!(f, "Admitted"),
            Self::Fenced => write!(f, "Fenced"),
            Self::Flushed => write!(f, "Flushed"),
            Self::Rewriting => write!(f, "Rewriting"),
            Self::Disposing => write!(f, "Disposing"),
            Self::WalRetiring => write!(f, "WalRetiring"),
            Self::Witnessing => write!(f, "Witnessing"),
            Self::Finalizing => write!(f, "Finalizing"),
            Self::Released => write!(f, "Released"),
        }
    }
}
