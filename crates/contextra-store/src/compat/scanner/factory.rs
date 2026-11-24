//! # Scanner Factory & Facade
//!
//! Dieses Modul stellt Fabrikfunktionen bereit, um Instanzen von [`LayerScanner`] und [`Purger`]
//! für das LSM-Speichertriebwerk bereitzustellen.
//!
//! Invarianten:
//! - INV-STORE-SCANNER: Scanner erfassen Memtables, SSTables und WALs konsistent über eine Snapshot-Sicht.

use std::sync::Arc;
use crate::lsm::LsmStorage;
use contextra_ports::scanner::{LayerScanner, Purger};
use crate::compat::scanner::{MemtableScanner, SstableScanner, WalScanner, LsmPurger, ScannerRuntime};

/// Erzeugt die Standard-LSM-Scanner unter Verwendung der aktuellen Tokio-Laufzeitumgebung.
pub fn lsm_scanners(storage: &Arc<LsmStorage>) -> Vec<Arc<dyn LayerScanner>> {
    let runtime = ScannerRuntime::capture();
    lsm_scanners_with_runtime(storage, runtime)
}

/// Erzeugt die Standard-LSM-Scanner unter Verwendung eines expliziten Tokio-Runtime-Handles.
pub fn lsm_scanners_with_handle(
    storage: &Arc<LsmStorage>,
    handle: tokio::runtime::Handle,
) -> Vec<Arc<dyn LayerScanner>> {
    let runtime = ScannerRuntime::with_handle(handle);
    lsm_scanners_with_runtime(storage, runtime)
}

/// Erzeugt die Standard-LSM-Scanner unter Verwendung einer übergebenen [`ScannerRuntime`].
pub fn lsm_scanners_with_runtime(
    storage: &Arc<LsmStorage>,
    runtime: ScannerRuntime,
) -> Vec<Arc<dyn LayerScanner>> {
    let view = match runtime.block_on(async { Ok(storage.scan_view().await) }) {
        Ok(v) => v,
        Err(_) => return Vec::new(),
    };
    vec![
        Arc::new(MemtableScanner::with_runtime(
            view.clone(),
            runtime.clone(),
        )),
        Arc::new(SstableScanner::with_runtime(
            view.clone(),
            runtime.clone(),
        )),
        Arc::new(WalScanner::with_runtime(view, runtime)),
    ]
}

/// Erzeugt die LSM-Purger unter Verwendung der aktuellen Tokio-Laufzeitumgebung.
pub fn lsm_purgers(storage: &Arc<LsmStorage>) -> Vec<Arc<dyn Purger>> {
    let runtime = ScannerRuntime::capture();
    lsm_purgers_with_runtime(storage, runtime)
}

/// Erzeugt die LSM-Purger unter Verwendung eines expliziten Tokio-Runtime-Handles.
pub fn lsm_purgers_with_handle(
    storage: &Arc<LsmStorage>,
    handle: tokio::runtime::Handle,
) -> Vec<Arc<dyn Purger>> {
    let runtime = ScannerRuntime::with_handle(handle);
    lsm_purgers_with_runtime(storage, runtime)
}

/// Erzeugt die LSM-Purger unter Verwendung einer übergebenen [`ScannerRuntime`].
pub fn lsm_purgers_with_runtime(
    storage: &Arc<LsmStorage>,
    runtime: ScannerRuntime,
) -> Vec<Arc<dyn Purger>> {
    vec![Arc::new(LsmPurger::with_runtime(
        Arc::clone(storage),
        runtime,
    ))]
}
