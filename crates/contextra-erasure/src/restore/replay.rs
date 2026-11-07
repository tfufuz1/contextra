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

/// Führt einen Replay-Vorgang für betroffene Löschungen durch (ER-30).
///
/// Stub: Produktiv wird die Saga für jede betroffene ErasureId neu gestartet.
pub fn replay_erasures(affected_ids: &[ErasureId]) -> Result<ReplayOutcome, ErasureError> {
    // In der vollständigen Implementierung (WP-4.3): die Saga für jede ID
    // aus dem Ledger laden und den Purge-Pfad neu durchlaufen.
    Ok(ReplayOutcome {
        replayed_count: affected_ids.len() as u64,
        store_cleared: true,
    })
}
