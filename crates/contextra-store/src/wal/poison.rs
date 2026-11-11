//! Typisierte Ursache für das Vergiften eines WAL-Handles (SPEC §3.5, §12.1, INV-FSYNC-1).
//!
//! Invarianten:
//! - INV-FSYNC-1: Nach fsync-, Schreib-, Truncate- oder HMAC-Kettendivergenz-Fehlern wird das Handle
//!   irreversibel vergiftet. Einzig bei partiellem Abbruch (`TornWrite`) ist Recovery erlaubt.
//! - Discriminant-Stabilität: Numerische Werte 1 bis 5 bleiben strikt unverändert.

/// Typisierte Ursache für das Vergiften eines WAL-Handles (SPEC §3.5, §12.1, INV-FSYNC-1).
///
/// Nach einem Fsync-, Write-, Truncate- oder HMAC-Kettendivergenz-Fehler wird das WAL-Handle
/// irreversibel vergiftet. Einzig bei einem partiellen Schreibabbruch (`TornWrite`) darf
/// das Handle über `recover_from_poison()` repariert werden.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum PoisonCause {
    /// Ein `fsync` (`sync_all` oder Verzeichnis-fsync) ist fehlgeschlagen. Irreversibel (fsyncgate).
    FsyncFailed = 1,
    /// Ein physischer Schreibaufruf (`write`/`write_all`/`flush`) ist fehlgeschlagen. Irreversibel.
    WriteFailed = 2,
    /// `set_len` / Dateitrunkierung ist fehlgeschlagen. Irreversibel.
    TruncateFailed = 3,
    /// HMAC-Kette ist divergiert (z. B. Rollback-Lock-Contention oder ungültiger Zustand). Irreversibel.
    HmacChainDiverged = 4,
    /// Verdacht auf unvollständigen Schreibvorgang am Dateiende (Torn Write). Behebbar via Replay.
    TornWrite = 5,
}

impl PoisonCause {
    #[inline]
    pub const fn to_u8(self) -> u8 {
        self as u8
    }

    #[inline]
    pub const fn from_u8(val: u8) -> Option<Self> {
        match val {
            1 => Some(Self::FsyncFailed),
            2 => Some(Self::WriteFailed),
            3 => Some(Self::TruncateFailed),
            4 => Some(Self::HmacChainDiverged),
            5 => Some(Self::TornWrite),
            _ => None,
        }
    }
}
