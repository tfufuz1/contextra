//! Purge-Backend-Schnittstelle und Koordination (SPEC §8.8, §11.4, ST-04, ST-07, ST-09).

use crate::health::reserve::DiskReserve;
use contextra_ports::keys::FileId;
use contextra_types::{key::Matcher, Result};

/// Statistiken über verworfene Memtable-Einträge.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct DrainStats {
    pub entries_drained: usize,
    pub bytes_drained: u64,
}

/// Versiegelungspunkt im aktiven WAL-Segment.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SealPoint {
    pub seq: u64,
    pub tail_mac: [u8; 32],
}

/// Manifest-Marker für entfernte WAL-Segmente.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WalFloor {
    pub seq: u64,
    pub tail_mac: [u8; 32],
    pub removed_file_ids: Vec<FileId>,
}

/// Ergebnis eines SSTable-Rewrites ohne Ziel-Records.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RewriteOutcome {
    pub old_file: FileId,
    pub new_file: FileId,
    pub records_copied: usize,
    pub records_purged: usize,
    pub bytes_written: u64,
}

/// Purge-Primitive (nur über ErasureLayer erreichbar; Reihenfolge fest, jeder Schritt idempotent).
pub trait PurgePrimitives: Send + Sync + 'static {
    /// Memtable-Einträge im Umfang verwerfen (ST-04).
    fn drain_range<'a>(
        &'a self,
        matcher: &'a Matcher,
    ) -> contextra_ports::BoxFuture<'a, Result<DrainStats>>;

    /// Aktives Segment an Sequenzgrenze versiegeln (ST-04).
    fn seal_wal_at<'a>(&'a self) -> contextra_ports::BoxFuture<'a, Result<SealPoint>>;

    /// Segmente <= f_seq als Ganzes entfernen (Dateischlüssel vernichten, unlink) (ST-04).
    fn drop_wal_until<'a>(&'a self, f_seq: u64)
        -> contextra_ports::BoxFuture<'a, Result<WalFloor>>;

    /// Kopiert Nicht-Ziel-Records als byteidentische Envelopes in eine neue Datei (ST-07).
    fn rewrite_without<'a>(
        &'a self,
        file_id: FileId,
        matcher: &'a Matcher,
        reserve: DiskReserve,
    ) -> contextra_ports::BoxFuture<'a, Result<RewriteOutcome>>;

    /// Manifest-Swap dauerhaft -> F-Slot nullen -> unlink und Verzeichnis-fsync (ST-07, ST-08, ST-09).
    fn unlink_verified<'a>(&'a self, file_id: FileId)
        -> contextra_ports::BoxFuture<'a, Result<()>>;
}
