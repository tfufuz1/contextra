//! # Scanner-Modul fuer Zero-Panic Scans (Ring 1)
//!
//! Stellt Scanner ueber MemTable, SSTables, WAL und Purge-Schnittstellen bereit.

// FILE-CONTEXT
// ZWECK: Export und Orchestrierung der LSM Storage-Scanner und Purger (SPEC §12.7.4, Verträge W1-P01).
// INVARIANTEN: lsm_scanners und lsm_purgers erzeugen nur valide, isolierte Instanzen für Memtable, SSTable und WAL.
// HOTSPOTS: [1-60]

#![forbid(unsafe_code)]

pub mod memtable;
pub mod purge;
pub mod runtime;
pub mod sstable;
pub mod wal;

#[cfg(test)]
mod tests;

pub use memtable::MemtableScanner;
pub use purge::LsmPurger;
pub use runtime::ScannerRuntime;
pub use sstable::SstableScanner;
pub use wal::WalScanner;

use crate::lsm::LsmStorage;
use contextra_ports::scanner::{LayerScanner, Purger};
use std::sync::Arc;
use tokio::runtime::Handle;

/// Erzeugt alle LSM-Storage-Scanner für eine gegebene LsmStorage-Instanz.
/// Erfasst das aktuelle Tokio-Runtime-Handle via `ScannerRuntime::capture()`.
pub fn lsm_scanners(storage: &Arc<LsmStorage>) -> Vec<Arc<dyn LayerScanner>> {
    let runtime = ScannerRuntime::capture();
    lsm_scanners_with_runtime(storage, runtime)
}

/// Erzeugt alle LSM-Storage-Scanner für eine gegebene LsmStorage-Instanz mit explizitem Tokio-Runtime-Handle.
pub fn lsm_scanners_with_handle(
    storage: &Arc<LsmStorage>,
    handle: Handle,
) -> Vec<Arc<dyn LayerScanner>> {
    let runtime = ScannerRuntime::with_handle(handle);
    lsm_scanners_with_runtime(storage, runtime)
}

/// Erzeugt alle LSM-Storage-Scanner mit einer spezifischen `ScannerRuntime`.
pub fn lsm_scanners_with_runtime(
    storage: &Arc<LsmStorage>,
    runtime: ScannerRuntime,
) -> Vec<Arc<dyn LayerScanner>> {
    vec![
        Arc::new(MemtableScanner::with_runtime(Arc::clone(storage), runtime.clone())),
        Arc::new(SstableScanner::with_runtime(Arc::clone(storage), runtime.clone())),
        Arc::new(WalScanner::with_runtime(Arc::clone(storage), runtime)),
    ]
}

/// Erzeugt den LSM-Storage-Purger für eine gegebene LsmStorage-Instanz.
/// Erfasst das aktuelle Tokio-Runtime-Handle via `ScannerRuntime::capture()`.
pub fn lsm_purgers(storage: &Arc<LsmStorage>) -> Vec<Arc<dyn Purger>> {
    let runtime = ScannerRuntime::capture();
    lsm_purgers_with_runtime(storage, runtime)
}

/// Erzeugt den LSM-Storage-Purger für eine gegebene LsmStorage-Instanz mit explizitem Tokio-Runtime-Handle.
pub fn lsm_purgers_with_handle(storage: &Arc<LsmStorage>, handle: Handle) -> Vec<Arc<dyn Purger>> {
    let runtime = ScannerRuntime::with_handle(handle);
    lsm_purgers_with_runtime(storage, runtime)
}

/// Erzeugt den LSM-Storage-Purger mit einer spezifischen `ScannerRuntime`.
pub fn lsm_purgers_with_runtime(
    storage: &Arc<LsmStorage>,
    runtime: ScannerRuntime,
) -> Vec<Arc<dyn Purger>> {
    vec![Arc::new(LsmPurger::with_runtime(Arc::clone(storage), runtime))]
}
