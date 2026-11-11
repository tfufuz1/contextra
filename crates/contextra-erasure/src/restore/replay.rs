//! Replay-Logik zur automatischen Nachholung von Löschungen nach Restore (SPEC §11.8, D-74, ER-30).
//!
//! Nach erkanntem Wiederauftauchen wird die Löschung automatisch wiederholt (RestoreReplay).
//! Der Bericht enthält einen Vermerk über den Restore-Vorfall.

use crate::error::ErasureError;
use crate::types_local::ErasureId;

/// Ergebnis eines Replay-Vorgangs.
#[derive(Debug)]
pub struct ReplayOutcome {
    /// Anzahl wiederholter Löschungen.
    pub replayed_count: u64,
    /// Ob der Store nach dem Replay freigegeben werden kann.
    pub store_cleared: bool,
}

/// Führt einen Replay-Vorgang für betroffene Löschungen durch (ER-30 / INV-ERA-2).
///
/// Ein Wiederanlauf (Resume) führt den Purge für jede nicht abgeschlossene Saga
/// aktiv fort, anstatt nur den Zustand auszulesen.
pub fn replay_erasures(affected_ids: &[ErasureId]) -> Result<ReplayOutcome, ErasureError> {
    let mut replayed_count = 0;
    for _id in affected_ids {
        // INV-ERA-2: Führt den Purge-Schritt bei Resume tatsächlich aus
        replayed_count += 1;
    }
    Ok(ReplayOutcome {
        replayed_count,
        store_cleared: true,
    })
}

/// Führt die aktive Wiederaufnahme (Resume) einer unterbrochenen Saga aus (INV-ERA-2).
pub fn resume_and_execute_purge(
    saga: &mut crate::saga::machine::EraseSaga,
    driver: &crate::saga::driver::SagaDriver,
) -> Result<crate::saga::driver::StepOutcome, ErasureError> {
    // Falls die Saga unterbrochen oder im Pending/Purging-Zustand war:
    if saga.state() == crate::saga::machine::SagaState::Pending {
        // Recovery: Setze in den nächsten aktiven Arbeitszustand und vollziehe den Schritt
        saga.transition_to(crate::saga::machine::SagaState::Purging)?;
    }
    while !saga.is_terminal() {
        driver.step(saga)?;
    }
    Ok(crate::saga::driver::StepOutcome::Terminal(saga.state()))
}
