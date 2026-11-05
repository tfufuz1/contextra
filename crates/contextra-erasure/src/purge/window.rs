//! Steuerung und Bündelung des Purge-Fensters (SPEC §11.2, §11.4, D-39, ER-20).
//!
//! Reihenfolge gemäß ER-20 (fest):
//!   a) Snapshots widerrufen (SnapshotRevoker)
//!   b) Textsegment-Rewrite
//!   c) Memtable-Drain, Flush, WAL-Segmente
//!   d) SSTable-Rewrite, F-Slots nullen, unlink
//!   e) Vektor-Kantenreparatur
//!   f) D-Slots der Zielmenge entfernen, F-Waisen prüfen
//!   g) Cache-Eviction

use super::cx::PurgeCx;
use super::reserve::{check_disk_reserve, ReserveCheck};
use crate::error::ErasureError;
use crate::saga::core::ErasureCore;

/// Ergebnis eines Purge-Fensters.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WindowOutcome {
    /// Anzahl erfolgreich gelöschter Einheiten.
    pub purged_units: u64,
    /// Anzahl freigegebener Bytes.
    pub bytes_freed: u64,
    /// Verbleibende offene Löschungen.
    pub pending_count: u64,
}

impl WindowOutcome {
    /// Erstellt ein leeres Ergebnis.
    pub fn empty() -> Self {
        Self {
            purged_units: 0,
            bytes_freed: 0,
            pending_count: 0,
        }
    }

    /// Addiert ein weiteres Ergebnis.
    pub fn merge(&mut self, other: &WindowOutcome) {
        self.purged_units = self.purged_units.saturating_add(other.purged_units);
        self.bytes_freed = self.bytes_freed.saturating_add(other.bytes_freed);
        self.pending_count = self.pending_count.saturating_add(other.pending_count);
    }
}

/// Führt ein Purge-Fenster durch (ER-20).
///
/// Quellkompatible Signatur für bestehende Aufrufer: prüft Plattenreserve.
/// Schlägt fehl mit `DiskReserve`, falls kein ausreichender Platz vorhanden ist,
/// oder liefert `NotImplemented`, falls kein `ErasureCore` konfiguriert ist,
/// um Scheinerfolge zu verhindern (P1, HR-10).
pub fn run_window(cx: &PurgeCx) -> Result<WindowOutcome, ErasureError> {
    let reserve = check_disk_reserve(cx.min_free_bytes, 0)?;
    if let ReserveCheck::Insufficient { .. } = reserve {
        return Err(ErasureError::DiskReserve);
    }

    Err(ErasureError::NotImplemented(
        "run_window requires active ErasureCore orchestrator; use run_core_window".into(),
    ))
}

/// Führt ein vollständiges Purge-Fenster über den `ErasureCore` aus (SPEC §11.2, §11.4).
pub async fn run_core_window(core: &ErasureCore) -> Result<WindowOutcome, ErasureError> {
    core.run_purge_window().await
}
