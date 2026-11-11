//! # Scanner-Kompatibilitätsbrücke
//!
//! Dieses Modul implementiert die Schnittstellen [`LayerScanner`] und [`Purger`]
//! aus `contextra-ports` für `contextra-store`.

pub mod memtable;
pub mod sstable;
pub mod wal;
pub mod runtime;
pub mod store_scan_view;
pub mod purge;
pub mod factory;

#[cfg(test)]
pub mod tests;

pub use memtable::MemtableScanner;
pub use sstable::SstableScanner;
pub use wal::WalScanner;
pub use runtime::ScannerRuntime;
pub use store_scan_view::StoreScanView;
pub use purge::LsmPurger;
pub use factory::{
    lsm_scanners, lsm_scanners_with_handle, lsm_scanners_with_runtime,
    lsm_purgers, lsm_purgers_with_handle, lsm_purgers_with_runtime,
};
