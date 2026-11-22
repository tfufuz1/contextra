// FILE-CONTEXT
// ZWECK: Hintergrund-Worker-Tasks zur TTL-Löschung, Entropie-Pruning und Bereinigung verwaister Transaktionen (Orphan Cleanup).
// INVARIANTEN: Geordnete Abschaltung via CancellationToken; Beschränkung der pro Tick verarbeiteten Elemente.
// NICHT-OFFENSICHTLICH: Orphan Cleanup Worker triggert bei HNSW-Indextrennung automatischen Rebuild mit Timeout.
// STAND: TS:2026-08-29T17:22:29Z (SESSION: 0dcb9f3b)

mod config;
mod expiry_workers;
mod hyperedge_worker;
mod orphan_workers;

#[cfg(test)]
mod tests;

pub use config::{calculate_rebuild_cooldown, OrphanCleanupBackoffConfig, OrphanCleanupIndex};
#[cfg(feature = "background-maintenance")]
#[allow(deprecated)]
pub use expiry_workers::{start_decay_cleanup_worker, start_thermostat_reaper};
#[allow(deprecated)]
pub use expiry_workers::{start_expiry_cleanup_worker, start_expiry_reaper, MAX_EXPIRED_PER_TICK};
pub use hyperedge_worker::{
    start_hyperedge_cascade_deferred_worker, DeferredHyperedgeQueue,
    MAX_DEFERRED_HYPEREDGES_PER_TICK,
};
#[allow(deprecated)]
pub use orphan_workers::{
    start_orphan_cleanup_worker, start_orphan_cleanup_worker_with_config, start_orphan_reaper,
    MAX_ORPHANS_PER_TICK,
};
