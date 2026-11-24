//! # StoreScanView — Entkoppelter Scan-Kontext für Scanner und Purger (Ring 1)
//!
//! [`StoreScanView`] kapselt den minimalen Lesezugriff, den Scanner und Purger auf den
//! LSM-Speicher benötigen. Statt `Arc<LsmStorage>` direkt zu halten, erhalten Scanner
//! eine schlanke, unveränderliche Sicht auf die Daten — ohne Zugriff auf WAL-Lock,
//! Commit-Mutex oder interne Zustandsmaschine.
//!
//! ## Entkopplungsziel (DEBT-01 / WP-STO-2)
//! Vor dieser Änderung hielten `MemtableScanner`, `SstableScanner`, `WalScanner` und
//! `LsmPurger` jeweils `Arc<LsmStorage>` und riefen `pub(crate)`-Methoden direkt auf.
//! Das machte Unit-Tests ohne vollständige `LsmStorage`-Instanz unmöglich.
//!
//! Nach dieser Änderung:
//! - Scanner halten `StoreScanView` (kein `Arc<LsmStorage>` mehr).
//! - `StoreScanView::from_storage` erzeugt die View aus einem `Arc<LsmStorage>` im `lsm`-Modul.
//! - Für Tests kann `StoreScanView::new_for_test` mit isolierten Fake-Daten befüllt werden.
//!
//! ## Zugreifbare Felder (read-only Snapshots, kein Lock-Contention im Scanner)
//! - `memtables`: Snapshot aktiver und immutabler MemTables.
//! - `sstables`: Snapshot der offenen SSTable-Reader.
//! - `block_cache`: Gemeinsamer Block-Cache für SSTable-Lesezugriffe.
//! - `data_path`: Pfad zum Datenverzeichnis (für Orphan-Scan).
//! - `vfs`: VFS-Instanz für Dateioperationen.
//! - `snapshot_registry`: Für Purge-Sperrprüfung (Snapshot-Sperre).

use crate::memtable::MemTable;
use crate::sstable::{BlockCache, SstableReader};
use contextra_mvcc::SnapshotRegistry;
use crate::io_seam::Vfs;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Schlanke, entkoppelte Lesezugriffssicht auf LSM-Speicherschichten für Scanner und Purger.
///
/// Kann direkt aus einem `Arc<LsmStorage>` erzeugt oder für Tests mit isolierten Daten
/// befüllt werden.
#[derive(Clone)]
pub struct StoreScanView {
    /// Aktive MemTable (Snapshot zum Erstellungszeitpunkt).
    pub(crate) active_memtable: Arc<MemTable>,
    /// Immutable MemTables (Snapshot zum Erstellungszeitpunkt).
    pub(crate) immutable_memtables: Vec<Arc<MemTable>>,
    /// Live-Referenz auf SSTable-Liste (Lesesperre beim Zugriff).
    pub(crate) sstables: Arc<RwLock<Vec<Arc<SstableReader>>>>,
    /// Gemeinsamer Block-Cache.
    pub(crate) block_cache: Arc<BlockCache>,
    /// Pfad zum Datenverzeichnis.
    pub(crate) data_path: PathBuf,
    /// VFS-Instanz für Dateioperationen.
    pub(crate) vfs: Arc<dyn Vfs>,
    /// SnapshotRegistry für Purge-Sperrprüfung.
    pub(crate) snapshot_registry: Arc<SnapshotRegistry>,
}

impl StoreScanView {
    /// Erstellt einen `StoreScanView` für Tests mit vollständig isolierten Daten.
    ///
    /// Ermöglicht Unit-Tests ohne `LsmStorage`-Instanz.
    #[cfg(any(test, feature = "test-hooks"))]
    pub fn new_for_test(
        active_memtable: Arc<MemTable>,
        immutable_memtables: Vec<Arc<MemTable>>,
        sstables: Arc<RwLock<Vec<Arc<SstableReader>>>>,
        block_cache: Arc<BlockCache>,
        data_path: PathBuf,
        vfs: Arc<dyn Vfs>,
        snapshot_registry: Arc<SnapshotRegistry>,
    ) -> Self {
        Self {
            active_memtable,
            immutable_memtables,
            sstables,
            block_cache,
            data_path,
            vfs,
            snapshot_registry,
        }
    }

    /// Gibt die aktive MemTable zurück.
    pub fn active_memtable(&self) -> &Arc<MemTable> {
        &self.active_memtable
    }

    /// Gibt alle immutablen MemTables zurück.
    pub fn immutable_memtables(&self) -> &[Arc<MemTable>] {
        &self.immutable_memtables
    }

    /// Gibt einen Iterator über alle MemTables (aktiv + immutable) zurück.
    pub fn all_memtables(&self) -> impl Iterator<Item = &Arc<MemTable>> {
        std::iter::once(&self.active_memtable).chain(self.immutable_memtables.iter())
    }

    /// Gibt die SSTable-Liste als `Arc<RwLock<...>>` zurück (für Purger-Schreibzugriff).
    pub fn sstables_arc(&self) -> &Arc<RwLock<Vec<Arc<SstableReader>>>> {
        &self.sstables
    }

    /// Gibt den gemeinsamen Block-Cache zurück.
    pub fn block_cache(&self) -> &Arc<BlockCache> {
        &self.block_cache
    }

    /// Gibt den Pfad zum Datenverzeichnis zurück.
    pub fn data_path(&self) -> &std::path::Path {
        &self.data_path
    }

    /// Gibt die VFS-Instanz zurück.
    pub fn vfs(&self) -> &Arc<dyn Vfs> {
        &self.vfs
    }

    /// Gibt die SnapshotRegistry zurück (für Purge-Sperrprüfung).
    pub fn snapshot_registry(&self) -> &Arc<SnapshotRegistry> {
        &self.snapshot_registry
    }
}
