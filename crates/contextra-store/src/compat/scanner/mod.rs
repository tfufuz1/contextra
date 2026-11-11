pub mod memtable;
pub mod sstable;
pub mod wal;
pub mod runtime;
pub mod store_scan_view;
pub mod purge;

#[cfg(test)]
pub mod tests;

pub use memtable::MemtableScanner;
pub use sstable::SstableScanner;
pub use wal::WalScanner;
pub use runtime::ScannerRuntime;
pub use store_scan_view::StoreScanView;
pub use purge::LsmPurger;

use std::sync::Arc;
use crate::lsm::LsmStorage;
use contextra_ports::scanner::LayerScanner;
use contextra_types::RequestContext;

pub fn lsm_scanners(storage: &Arc<LsmStorage>) -> Vec<Arc<dyn LayerScanner>> {
    let runtime = ScannerRuntime::capture();
    lsm_scanners_with_runtime(storage, runtime)
}

pub fn lsm_scanners_with_handle(
    storage: &Arc<LsmStorage>,
    handle: tokio::runtime::Handle,
) -> Vec<Arc<dyn LayerScanner>> {
    let runtime = ScannerRuntime::with_handle(handle);
    lsm_scanners_with_runtime(storage, runtime)
}

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

pub fn lsm_purgers(storage: &Arc<LsmStorage>) -> Vec<Arc<dyn contextra_ports::scanner::Purger>> {
    let runtime = ScannerRuntime::capture();
    lsm_purgers_with_runtime(storage, runtime)
}

pub fn lsm_purgers_with_handle(
    storage: &Arc<LsmStorage>,
    handle: tokio::runtime::Handle,
) -> Vec<Arc<dyn contextra_ports::scanner::Purger>> {
    let runtime = ScannerRuntime::with_handle(handle);
    lsm_purgers_with_runtime(storage, runtime)
}

pub fn lsm_purgers_with_runtime(
    storage: &Arc<LsmStorage>,
    runtime: ScannerRuntime,
) -> Vec<Arc<dyn contextra_ports::scanner::Purger>> {
    vec![Arc::new(LsmPurger::with_runtime(
        Arc::clone(storage),
        runtime,
    ))]
}
