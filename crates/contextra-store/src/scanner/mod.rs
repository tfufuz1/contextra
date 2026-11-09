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

/// Status eines untersuchten Schlüssels in den Speicherebenen.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ResidualInspectionReport {
    /// Ob der Schlüssel in aktiven oder unveränderlichen Memtables gefunden wurde.
    pub in_memtable: bool,
    /// Ob der Schlüssel in persistenten SSTables gefunden wurde.
    pub in_sstable: bool,
    /// Ob Mutationen für den Schlüssel im WAL-Log vorhanden sind.
    pub in_wal: bool,
}

impl ResidualInspectionReport {
    /// Liefert true zurück, falls der Schlüssel in mindestens einer Speicherebene existiert.
    #[must_use]
    pub const fn has_residual(&self) -> bool {
        self.in_memtable || self.in_sstable || self.in_wal
    }
}

/// Lesende, neutrale Inspektionssicht für den LSM-Speicherkern.
/// Prüft das Vorhandensein von Schlüsseln oder Präfixen in Memtable, SSTables und WAL,
/// ohne jegliche Annahmen über Löschketten, Zeugen oder Rechtssemantik.
#[derive(Clone)]
pub struct StoreInspector {
    storage: Arc<LsmStorage>,
    runtime: ScannerRuntime,
}

impl StoreInspector {
    /// Erzeugt eine neue Instanz von [`StoreInspector`].
    #[must_use]
    pub fn new(storage: Arc<LsmStorage>) -> Self {
        let runtime = ScannerRuntime::capture();
        Self { storage, runtime }
    }

    /// Erzeugt eine neue Instanz mit expliziter [`ScannerRuntime`].
    #[must_use]
    pub fn with_runtime(storage: Arc<LsmStorage>, runtime: ScannerRuntime) -> Self {
        Self { storage, runtime }
    }

    /// Inspiziert das Vorkommen eines Schlüssels über alle Speicherebenen hinweg.
    ///
    /// # Errors
    /// Liefert einen `ContextraError::Storage`, falls das Scannen der Speicherebenen fehlschlägt.
    pub fn inspect_key(&self, key: &[u8]) -> contextra_types::Result<ResidualInspectionReport> {
        let mem_scanner =
            MemtableScanner::with_runtime(Arc::clone(&self.storage), self.runtime.clone());
        let sst_scanner =
            SstableScanner::with_runtime(Arc::clone(&self.storage), self.runtime.clone());
        let wal_scanner = WalScanner::with_runtime(Arc::clone(&self.storage), self.runtime.clone());

        let target = contextra_ports::scanner::ScanTarget::Key(key.to_vec());
        let ctx = contextra_types::RequestContext::new_root();
        let technique = contextra_ports::scanner::ScanTechnique::DirectLookup;
        let clock = contextra_ports::SystemClock::new();

        let in_mem = mem_scanner
            .scan(&ctx, target.clone(), technique.clone(), &clock)
            .map_err(|e| {
                contextra_types::ContextraError::Storage(format!("Memtable scan error: {e:?}"))
            })?
            .found();

        let in_sst = sst_scanner
            .scan(&ctx, target.clone(), technique.clone(), &clock)
            .map_err(|e| {
                contextra_types::ContextraError::Storage(format!("SSTable scan error: {e:?}"))
            })?
            .found();

        let in_wal = wal_scanner
            .scan(&ctx, target, technique, &clock)
            .map_err(|e| {
                contextra_types::ContextraError::Storage(format!("WAL scan error: {e:?}"))
            })?
            .found();

        Ok(ResidualInspectionReport {
            in_memtable: in_mem,
            in_sstable: in_sst,
            in_wal,
        })
    }

    /// Prüft, ob ein Schlüssel in mindestens einer Speicherebene physisch vorhanden ist.
    ///
    /// # Errors
    /// Liefert einen `ContextraError::Storage`, wenn die Inspektion fehlschlägt.
    pub fn has_residual_key(&self, key: &[u8]) -> contextra_types::Result<bool> {
        self.inspect_key(key).map(|r| r.has_residual())
    }
}

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
        Arc::new(MemtableScanner::with_runtime(
            Arc::clone(storage),
            runtime.clone(),
        )),
        Arc::new(SstableScanner::with_runtime(
            Arc::clone(storage),
            runtime.clone(),
        )),
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
    vec![Arc::new(LsmPurger::with_runtime(
        Arc::clone(storage),
        runtime,
    ))]
}
