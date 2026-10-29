// FILE-CONTEXT
// ZWECK: Export und Orchestrierung der LSM Storage-Scanner und Purger (SPEC §12.7.4, Verträge W1-P01).
// INVARIANTEN: lsm_scanners und lsm_purgers erzeugen nur valide, isolierte Instanzen für Memtable, SSTable und WAL.
// HOTSPOTS: [1-50]

#![forbid(unsafe_code)]

pub mod memtable;
pub mod purge;
pub mod sstable;
pub mod wal;

#[cfg(test)]
mod tests;

pub use memtable::MemtableScanner;
pub use purge::LsmPurger;
pub use sstable::SstableScanner;
pub use wal::WalScanner;

use crate::lsm::LsmStorage;
use contextra_ports::scanner::{LayerScanner, Purger};
use std::sync::Arc;

/// Erzeugt alle LSM-Storage-Scanner für eine gegebene LsmStorage-Instanz.
pub fn lsm_scanners(storage: &Arc<LsmStorage>) -> Vec<Arc<dyn LayerScanner>> {
    vec![
        Arc::new(MemtableScanner::new(Arc::clone(storage))),
        Arc::new(SstableScanner::new(Arc::clone(storage))),
        Arc::new(WalScanner::new(Arc::clone(storage))),
    ]
}

/// Erzeugt den LSM-Storage-Purger für eine gegebene LsmStorage-Instanz.
pub fn lsm_purgers(storage: &Arc<LsmStorage>) -> Vec<Arc<dyn Purger>> {
    vec![Arc::new(LsmPurger::new(Arc::clone(storage)))]
}

/// Hilfsfunktion: Prüft, ob ein Byte-Schlüssel zu einem `ScanTarget` passt.
#[allow(dead_code)]
pub(crate) fn key_matches_target(
    key: &[u8],
    target: &contextra_ports::scanner::ScanTarget,
) -> bool {
    let tid = target.target_id.as_bytes();
    if key.starts_with(tid) {
        return true;
    }
    // Fallback für Tenant-Prefixe: "tenant_1" -> "t:1:"
    if let Some(num_str) = target.target_id.strip_prefix("tenant_") {
        if let Ok(tenant_id) = num_str.parse::<u64>() {
            let prefix = format!("t:{}:", tenant_id);
            if key.starts_with(prefix.as_bytes()) {
                return true;
            }
        }
    } else if let Ok(tenant_id) = target.target_id.parse::<u64>() {
        let prefix = format!("t:{}:", tenant_id);
        if key.starts_with(prefix.as_bytes()) {
            return true;
        }
    }
    false
}
