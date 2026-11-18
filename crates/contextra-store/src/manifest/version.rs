//! # Immutable Version & Generation Management (P3)
//!
//! Stellt eine unveränderliche Generation des LSM-Speicherzustands dar ([`Version`]).
//! Leser pinnen eine Version (via `Arc<Version>`); Modifikationen (Flush, Compaction, Purge)
//! erzeugen atomar eine neue Generation via Manifest-Edit-Batch und führen einen kurzen Pointer-Swap durch.
//!
//! Invarianten:
//! - Lock-Freie Leser: Leser halten keinen exklusiven Lock und machen kein I/O unter Sperren.
//! - Determinismus: Die Liste aktiver SSTables und der `wal_floor` sind atomar und konsistent.

use crate::sstable::SstableReader;
use std::fmt;
use std::sync::Arc;

/// Eine unveränderliche Generation (Snapshot des physischen Layouts) der LSM-Engine.
#[derive(Clone)]
pub struct Version {
    /// Eindeutige, monoton steigende Generations-ID.
    pub generation_id: u64,
    /// Aktuell aktive SSTable-Reader, geordnet nach Sequenznummer.
    pub sstables: Vec<Arc<SstableReader>>,
    /// Monoton steigender WAL-Floor: Segmente vor dieser Sequenz dürfen ungelinkt werden.
    pub wal_floor: u64,
}

impl fmt::Debug for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Version")
            .field("generation_id", &self.generation_id)
            .field("sstables_count", &self.sstables.len())
            .field("wal_floor", &self.wal_floor)
            .finish()
    }
}

impl Version {
    /// Erstellt eine initiale, leere Version (Generation 0).
    pub fn empty() -> Self {
        Self {
            generation_id: 0,
            sstables: Vec::new(),
            wal_floor: 0,
        }
    }

    /// Erstellt eine neue Generation basierend auf hinzugefügten und entfernten SSTables sowie neuem WAL-Floor.
    pub fn update(
        &self,
        to_add: Vec<Arc<SstableReader>>,
        to_remove: &[Arc<SstableReader>],
        new_wal_floor: Option<u64>,
    ) -> Self {
        let mut new_sstables = self.sstables.clone();
        new_sstables.retain(|sst| !to_remove.iter().any(|rem| Arc::ptr_eq(sst, rem)));
        new_sstables.extend(to_add);
        new_sstables.sort_by_key(|sst| sst.metadata().max_seq & !contextra_ids::TOMBSTONE_BIT);

        Self {
            generation_id: self.generation_id + 1,
            sstables: new_sstables,
            wal_floor: new_wal_floor.unwrap_or(self.wal_floor).max(self.wal_floor),
        }
    }
}
