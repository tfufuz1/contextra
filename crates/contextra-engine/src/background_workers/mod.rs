// FILE-CONTEXT
// ZWECK: Hintergrund-Worker-Tasks zur TTL-Löschung, Entropie-Pruning und Bereinigung verwaister Transaktionen (Orphan Cleanup).
// INVARIANTEN: Geordnete Abschaltung via CancellationToken; Beschränkung der pro Tick verarbeiteten Elemente.
// NICHT-OFFENSICHTLICH: Orphan Cleanup Worker triggert bei HNSW-Indextrennung automatischen Rebuild mit Timeout.
// STAND: TS:2026-08-29T17:22:29Z (SESSION: 0dcb9f3b)

pub mod config;
pub mod expiry_workers;
pub mod hyperedge_worker;
pub mod orphan_workers;

#[cfg(test)]
mod tests;

pub use config::*;
pub use expiry_workers::*;
pub use hyperedge_worker::*;
pub use orphan_workers::*;
